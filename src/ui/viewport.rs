use egui::{Color32, Pos2, Rect, Sense, Stroke, Ui, Vec2};
use std::sync::Arc;
use crate::codecs::DecodedImage;
use crate::core::config::ZoomMode;

#[derive(Debug, Clone)]
pub struct ViewportState {
    pub zoom: f32,
    pub pan: Vec2,
    pub rotation: u32, // 0, 90, 180, 270
    pub flip_h: bool,
    pub flip_v: bool,
    pub zoom_mode: ZoomMode,
    pub last_image_dim: Option<(u32, u32)>,
}

impl Default for ViewportState {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            pan: Vec2::ZERO,
            rotation: 0,
            flip_h: false,
            flip_v: false,
            zoom_mode: ZoomMode::FitToWindow,
            last_image_dim: None,
        }
    }
}

impl ViewportState {
    pub fn reset_view(&mut self) {
        self.zoom = 1.0;
        self.pan = Vec2::ZERO;
        self.rotation = 0;
        self.flip_h = false;
        self.flip_v = false;
    }

    pub fn rotate_right(&mut self) {
        self.rotation = (self.rotation + 90) % 360;
    }

    pub fn rotate_left(&mut self) {
        self.rotation = (self.rotation + 270) % 360;
    }

    pub fn toggle_flip_h(&mut self) {
        self.flip_h = !self.flip_h;
    }

    pub fn toggle_flip_v(&mut self) {
        self.flip_v = !self.flip_v;
    }

    pub fn fit_to_rect(&mut self, img_w: f32, img_h: f32, viewport_rect: Rect) {
        if img_w <= 0.0 || img_h <= 0.0 {
            return;
        }

        let is_rotated_90 = self.rotation == 90 || self.rotation == 270;
        let (effective_w, effective_h) = if is_rotated_90 {
            (img_h, img_w)
        } else {
            (img_w, img_h)
        };

        let pad = 16.0;
        let avail_w = (viewport_rect.width() - pad * 2.0).max(10.0);
        let avail_h = (viewport_rect.height() - pad * 2.0).max(10.0);

        let scale_w = avail_w / effective_w;
        let scale_h = avail_h / effective_h;
        self.zoom = scale_w.min(scale_h).min(1.0); // Fit within bounds, max 1.0 by default unless small
        self.pan = Vec2::ZERO;
    }
}

pub struct ViewportRenderOutput {
    pub requested_next: bool,
    pub requested_prev: bool,
}

pub fn render_viewport(
    ui: &mut Ui,
    state: &mut ViewportState,
    image: Option<&Arc<DecodedImage>>,
    texture_handle: Option<&egui::TextureHandle>,
    is_loading: bool,
    error: Option<&str>,
) -> ViewportRenderOutput {
    let output = ViewportRenderOutput {
        requested_next: false,
        requested_prev: false,
    };

    let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::click_and_drag());
    let rect = response.rect;

    // 1. Draw procedural transparent checkerboard background
    let check_size = 16.0;
    let cols = (rect.width() / check_size).ceil() as usize;
    let rows = (rect.height() / check_size).ceil() as usize;

    let dark_gray = Color32::from_rgb(20, 20, 24);
    let light_gray = Color32::from_rgb(28, 28, 34);

    painter.rect_filled(rect, 0.0, dark_gray);
    for r in 0..rows {
        for c in 0..cols {
            if (r + c) % 2 == 1 {
                let x = rect.min.x + c as f32 * check_size;
                let y = rect.min.y + r as f32 * check_size;
                let cell_rect = Rect::from_min_size(
                    Pos2::new(x, y),
                    Vec2::new(check_size, check_size),
                ).intersect(rect);
                painter.rect_filled(cell_rect, 0.0, light_gray);
            }
        }
    }

    // 2. Handle Loading or Error states
    if is_loading {
        let center = rect.center();
        painter.text(
            center,
            egui::Align2::CENTER_CENTER,
            "Loading image...",
            egui::FontId::proportional(20.0),
            Color32::from_rgb(180, 180, 200),
        );
        return output;
    }

    if let Some(err_msg) = error {
        let center = rect.center();
        painter.text(
            center,
            egui::Align2::CENTER_CENTER,
            format!("Error loading image: {}", err_msg),
            egui::FontId::proportional(18.0),
            Color32::from_rgb(255, 90, 90),
        );
        return output;
    }

    let (img, texture) = match (image, texture_handle) {
        (Some(i), Some(t)) => (i, t),
        _ => {
            let center = rect.center();
            painter.text(
                center,
                egui::Align2::CENTER_CENTER,
                "No image loaded\nDrag and drop an image or press Ctrl+O to open",
                egui::FontId::proportional(18.0),
                Color32::from_rgb(140, 140, 160),
            );
            return output;
        }
    };

    let img_w = img.width as f32;
    let img_h = img.height as f32;

    // Check if new image dimensions loaded -> auto fit
    if state.last_image_dim != Some((img.width, img.height)) {
        state.last_image_dim = Some((img.width, img.height));
        if state.zoom_mode == ZoomMode::FitToWindow {
            state.fit_to_rect(img_w, img_h, rect);
        } else if state.zoom_mode == ZoomMode::ActualSize {
            state.zoom = 1.0;
            state.pan = Vec2::ZERO;
        }
    }

    // 3. Pan and Zoom Input Handling
    if response.dragged() {
        state.pan += response.drag_delta();
    }

    // Double click to toggle 100% and Fit
    if response.double_clicked() {
        if (state.zoom - 1.0).abs() < 0.05 {
            state.fit_to_rect(img_w, img_h, rect);
        } else {
            state.zoom = 1.0;
            state.pan = Vec2::ZERO;
        }
    }

    // Mouse wheel zoom centered on cursor
    let scroll_delta = ui.input(|i| i.smooth_scroll_delta.y);
    if scroll_delta != 0.0 {
        if let Some(pointer_pos) = response.hover_pos() {
            let zoom_factor = if scroll_delta > 0.0 { 1.15 } else { 1.0 / 1.15 };
            let new_zoom = (state.zoom * zoom_factor).clamp(0.01, 100.0);

            // Cursor-centered zoom formula:
            // Adjust pan so the point under cursor remains static
            let viewport_center = rect.center() + state.pan;
            let cursor_offset = pointer_pos - viewport_center;
            let pan_delta = cursor_offset * (1.0 - zoom_factor);
            state.pan += pan_delta;
            state.zoom = new_zoom;
        }
    }

    // 4. Calculate final image destination quad with rotation & flip
    let is_rotated_90 = state.rotation == 90 || state.rotation == 270;
    let (draw_w, draw_h) = if is_rotated_90 {
        (img_h * state.zoom, img_w * state.zoom)
    } else {
        (img_w * state.zoom, img_h * state.zoom)
    };

    let draw_center = rect.center() + state.pan;
    let draw_rect = Rect::from_center_size(draw_center, Vec2::new(draw_w, draw_h));

    // UV coordinates with rotation and flips
    let mut uv_min = Pos2::new(0.0, 0.0);
    let mut uv_max = Pos2::new(1.0, 1.0);

    if state.flip_h {
        std::mem::swap(&mut uv_min.x, &mut uv_max.x);
    }
    if state.flip_v {
        std::mem::swap(&mut uv_min.y, &mut uv_max.y);
    }

    let uv = Rect::from_min_max(uv_min, uv_max);

    // Draw the image quad
    painter.image(
        texture.id(),
        draw_rect,
        uv,
        Color32::WHITE,
    );

    // Subtle border around image if zoomed in or transparent
    painter.rect_stroke(
        draw_rect,
        0.0,
        Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 20)),
        egui::StrokeKind::Outside,
    );

    output
}
