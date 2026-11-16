use egui::{Color32, CornerRadius, Grid, RichText, Stroke, Ui, Vec2};
use crate::core::photo::PhotoItem;

pub fn render_exif_hud(ui: &mut Ui, photo: &PhotoItem, close_requested: &mut bool) {
    let frame = egui::Frame::new()
        .fill(Color32::from_rgba_unmultiplied(16, 16, 20, 235))
        .stroke(Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 30)))
        .corner_radius(CornerRadius::same(8))
        .inner_margin(egui::Margin::symmetric(14, 12));

    frame.show(ui, |ui| {
        ui.set_max_width(320.0);

        ui.horizontal(|ui| {
            ui.label(RichText::new("📷 Image Information").size(15.0).strong().color(Color32::WHITE));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("✖").clicked() {
                    *close_requested = true;
                }
            });
        });

        ui.separator();

        Grid::new("exif_info_grid")
            .num_columns(2)
            .spacing(Vec2::new(12.0, 6.0))
            .show(ui, |ui| {
                // File name
                ui.label(RichText::new("File:").color(Color32::from_rgb(150, 150, 165)));
                ui.label(RichText::new(&photo.file_name).strong().color(Color32::WHITE));
                ui.end_row();

                // File size
                ui.label(RichText::new("Size:").color(Color32::from_rgb(150, 150, 165)));
                ui.label(RichText::new(photo.formatted_size()).color(Color32::from_rgb(200, 200, 220)));
                ui.end_row();

                // Dimensions
                if let Some((w, h)) = photo.dimensions {
                    let mp = (w as f64 * h as f64) / 1_000_000.0;
                    ui.label(RichText::new("Resolution:").color(Color32::from_rgb(150, 150, 165)));
                    ui.label(format!("{} × {} ({:.1} MP)", w, h, mp));
                    ui.end_row();
                }

                // EXIF details if present
                if let Some(img) = &photo.image {
                    if let Some(exif) = &img.exif {
                        if let Some(ref make) = exif.make {
                            ui.label(RichText::new("Camera:").color(Color32::from_rgb(150, 150, 165)));
                            let model = exif.model.as_deref().unwrap_or("");
                            ui.label(format!("{} {}", make, model));
                            ui.end_row();
                        }

                        if let Some(ref lens) = exif.lens_model {
                            ui.label(RichText::new("Lens:").color(Color32::from_rgb(150, 150, 165)));
                            ui.label(lens);
                            ui.end_row();
                        }

                        if exif.exposure_time.is_some() || exif.f_number.is_some() || exif.iso.is_some() {
                            ui.label(RichText::new("Exposure:").color(Color32::from_rgb(150, 150, 165)));
                            let exp = exif.exposure_time.as_deref().unwrap_or("-");
                            let f = exif.f_number.as_deref().unwrap_or("-");
                            let iso = exif.iso.map(|v| format!("ISO {}", v)).unwrap_or_default();
                            ui.label(format!("{}  {}  {}", exp, f, iso));
                            ui.end_row();
                        }

                        if let Some(ref focal) = exif.focal_length {
                            ui.label(RichText::new("Focal:").color(Color32::from_rgb(150, 150, 165)));
                            ui.label(focal);
                            ui.end_row();
                        }

                        if let Some(ref date) = exif.date_time {
                            ui.label(RichText::new("Date:").color(Color32::from_rgb(150, 150, 165)));
                            ui.label(date);
                            ui.end_row();
                        }
                    }
                }
            });
    });
}
