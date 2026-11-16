use egui::{Color32, CornerRadius, Rect, ScrollArea, Sense, Stroke, Ui, Vec2};
use std::collections::HashMap;
use crate::core::photo::PhotoItem;

pub struct GalleryState {
    pub texture_cache: HashMap<usize, egui::TextureHandle>,
}

impl GalleryState {
    pub fn new() -> Self {
        Self {
            texture_cache: HashMap::new(),
        }
    }
}

pub fn render_gallery(
    ui: &mut Ui,
    photos: &[PhotoItem],
    current_index: usize,
    state: &mut GalleryState,
    on_select: &mut Option<usize>,
) {
    if photos.is_empty() {
        return;
    }

    let frame = egui::Frame::new()
        .fill(Color32::from_rgba_unmultiplied(14, 14, 18, 235))
        .stroke(Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 20)))
        .inner_margin(egui::Margin::symmetric(8, 6));

    frame.show(ui, |ui| {
        ScrollArea::horizontal()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = Vec2::new(6.0, 0.0);

                    let thumb_height = 54.0;

                    for (idx, photo) in photos.iter().enumerate() {
                        let is_active = idx == current_index;
                        let item_size = Vec2::new(54.0, thumb_height);

                        let (rect, response) = ui.allocate_exact_size(item_size, Sense::click());

                        if response.clicked() {
                            *on_select = Some(idx);
                        }

                        // Background card
                        let bg_color = if is_active {
                            Color32::from_rgb(35, 45, 60)
                        } else if response.hovered() {
                            Color32::from_rgb(30, 30, 36)
                        } else {
                            Color32::from_rgb(22, 22, 26)
                        };

                        ui.painter().rect_filled(rect, CornerRadius::same(4), bg_color);

                        // If thumbnail decoded, prepare GPU texture handle
                        if let Some(thumb) = &photo.thumbnail {
                            let tex_handle = state.texture_cache.entry(idx).or_insert_with(|| {
                                let color_image = egui::ColorImage::from_rgba_unmultiplied(
                                    [thumb.width as usize, thumb.height as usize],
                                    &thumb.rgba,
                                );
                                ui.ctx().load_texture(
                                    format!("thumb_{}", idx),
                                    color_image,
                                    egui::TextureOptions::LINEAR,
                                )
                            });

                            // Calculate aspect ratio inside the box
                            let img_w = thumb.width as f32;
                            let img_h = thumb.height as f32;
                            let scale = (item_size.x / img_w).min(item_size.y / img_h).min(1.0);
                            let target_size = Vec2::new(img_w * scale, img_h * scale);
                            let target_rect = Rect::from_center_size(rect.center(), target_size);

                            ui.painter().image(
                                tex_handle.id(),
                                target_rect,
                                Rect::from_min_max(egui::Pos2::ZERO, egui::Pos2::new(1.0, 1.0)),
                                Color32::WHITE,
                            );
                        } else {
                            // Placeholder
                            ui.painter().text(
                                rect.center(),
                                egui::Align2::CENTER_CENTER,
                                format!("{}", idx + 1),
                                egui::FontId::proportional(11.0),
                                Color32::from_rgb(100, 100, 115),
                            );
                        }

                        // Active outline border
                        if is_active {
                            ui.painter().rect_stroke(
                                rect,
                                CornerRadius::same(4),
                                Stroke::new(2.0, Color32::from_rgb(80, 160, 255)),
                                egui::StrokeKind::Outside,
                            );
                        }
                    }
                });
            });
    });
}
