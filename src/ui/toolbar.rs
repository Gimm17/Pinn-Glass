use egui::{Button, Color32, CornerRadius, RichText, Stroke, Ui, Vec2};
use crate::ui::viewport::ViewportState;

pub struct ToolbarAction {
    pub prev: bool,
    pub next: bool,
    pub zoom_in: bool,
    pub zoom_out: bool,
    pub zoom_100: bool,
    pub zoom_fit: bool,
    pub rotate_left: bool,
    pub rotate_right: bool,
    pub flip_h: bool,
    pub flip_v: bool,
    pub toggle_exif: bool,
    pub toggle_gallery: bool,
    pub toggle_fullscreen: bool,
    pub open_file: bool,
}

pub fn render_toolbar(
    ui: &mut Ui,
    viewport: &mut ViewportState,
    current_index: usize,
    total_photos: usize,
    show_exif: bool,
    show_gallery: bool,
    is_fullscreen: bool,
) -> ToolbarAction {
    let mut action = ToolbarAction {
        prev: false,
        next: false,
        zoom_in: false,
        zoom_out: false,
        zoom_100: false,
        zoom_fit: false,
        rotate_left: false,
        rotate_right: false,
        flip_h: false,
        flip_v: false,
        toggle_exif: false,
        toggle_gallery: false,
        toggle_fullscreen: false,
        open_file: false,
    };

    let frame = egui::Frame::new()
        .fill(Color32::from_rgba_unmultiplied(18, 18, 22, 220))
        .stroke(Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 25)))
        .corner_radius(CornerRadius::same(8))
        .inner_margin(egui::Margin::symmetric(10, 6));

    frame.show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = Vec2::new(6.0, 0.0);

            // Open button
            if ui.add(Button::new(RichText::new("📁 Open").size(13.0)).fill(Color32::from_rgb(32, 34, 40))).clicked() {
                action.open_file = true;
            }

            ui.separator();

            // Navigation
            let has_photos = total_photos > 0;
            if ui.add_enabled(has_photos, Button::new("◀").min_size(Vec2::new(26.0, 24.0))).clicked() {
                action.prev = true;
            }

            let index_text = if total_photos > 0 {
                format!("{} / {}", current_index + 1, total_photos)
            } else {
                "- / -".to_string()
            };

            ui.label(RichText::new(index_text).size(13.0).color(Color32::from_rgb(200, 200, 215)));

            if ui.add_enabled(has_photos, Button::new("▶").min_size(Vec2::new(26.0, 24.0))).clicked() {
                action.next = true;
            }

            ui.separator();

            // Zoom controls
            if ui.add(Button::new("🔍-").min_size(Vec2::new(28.0, 24.0))).clicked() {
                action.zoom_out = true;
            }

            let zoom_pct = format!("{:.0}%", viewport.zoom * 100.0);
            ui.label(RichText::new(zoom_pct).size(13.0).strong().color(Color32::from_rgb(220, 220, 230)));

            if ui.add(Button::new("🔍+").min_size(Vec2::new(28.0, 24.0))).clicked() {
                action.zoom_in = true;
            }

            if ui.add(Button::new("1:1").min_size(Vec2::new(32.0, 24.0))).clicked() {
                action.zoom_100 = true;
            }

            if ui.add(Button::new("⛶ Fit").min_size(Vec2::new(36.0, 24.0))).clicked() {
                action.zoom_fit = true;
            }

            ui.separator();

            // Transformations
            if ui.add(Button::new("↺").min_size(Vec2::new(26.0, 24.0))).on_hover_text("Rotate Left (90°)").clicked() {
                action.rotate_left = true;
            }
            if ui.add(Button::new("↻").min_size(Vec2::new(26.0, 24.0))).on_hover_text("Rotate Right (90°)").clicked() {
                action.rotate_right = true;
            }
            if ui.add(Button::new("⇄").min_size(Vec2::new(26.0, 24.0))).on_hover_text("Flip Horizontal").clicked() {
                action.flip_h = true;
            }
            if ui.add(Button::new("⇅").min_size(Vec2::new(26.0, 24.0))).on_hover_text("Flip Vertical").clicked() {
                action.flip_v = true;
            }

            ui.separator();

            // Toggles
            let exif_btn = Button::new(RichText::new("ℹ EXIF").color(if show_exif { Color32::from_rgb(100, 180, 255) } else { Color32::LIGHT_GRAY }));
            if ui.add(exif_btn).clicked() {
                action.toggle_exif = true;
            }

            let gallery_btn = Button::new(RichText::new("🖼 Gallery").color(if show_gallery { Color32::from_rgb(100, 180, 255) } else { Color32::LIGHT_GRAY }));
            if ui.add(gallery_btn).clicked() {
                action.toggle_gallery = true;
            }

            let fs_label = if is_fullscreen { "⤓ Exit FS" } else { "⛶ Fullscreen" };
            if ui.add(Button::new(fs_label)).clicked() {
                action.toggle_fullscreen = true;
            }
        });
    });

    action
}
