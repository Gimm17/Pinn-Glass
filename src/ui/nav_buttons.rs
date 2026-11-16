use egui::{Color32, CornerRadius, Pos2, Rect, Sense, Stroke, Ui, Vec2};

pub struct NavButtonsResult {
    pub prev_clicked: bool,
    pub next_clicked: bool,
}

pub fn render_nav_buttons(
    ui: &mut Ui,
    window_rect: Rect,
    has_photos: bool,
) -> NavButtonsResult {
    let mut result = NavButtonsResult {
        prev_clicked: false,
        next_clicked: false,
    };

    if !has_photos {
        return result;
    }

    let pointer_pos = ui.input(|i| i.pointer.hover_pos());
    let edge_width = 100.0;
    let btn_size = Vec2::new(48.0, 64.0);

    // Left Button Zone
    let left_zone = Rect::from_min_size(
        window_rect.min,
        Vec2::new(edge_width, window_rect.height()),
    );

    let is_left_hovered = pointer_pos.map(|p| left_zone.contains(p)).unwrap_or(false);
    let left_btn_rect = Rect::from_center_size(
        Pos2::new(window_rect.min.x + 36.0, window_rect.center().y),
        btn_size,
    );

    if is_left_hovered {
        let resp = ui.allocate_rect(left_btn_rect, Sense::click());
        let bg_color = if resp.hovered() {
            Color32::from_rgba_unmultiplied(35, 35, 45, 230)
        } else {
            Color32::from_rgba_unmultiplied(20, 20, 28, 180)
        };

        ui.painter().rect_filled(left_btn_rect, CornerRadius::same(8), bg_color);
        ui.painter().rect_stroke(
            left_btn_rect,
            CornerRadius::same(8),
            Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 40)),
            egui::StrokeKind::Outside,
        );
        ui.painter().text(
            left_btn_rect.center(),
            egui::Align2::CENTER_CENTER,
            "◀",
            egui::FontId::proportional(22.0),
            Color32::WHITE,
        );

        if resp.clicked() {
            result.prev_clicked = true;
        }
    }

    // Right Button Zone
    let right_zone = Rect::from_min_size(
        Pos2::new(window_rect.max.x - edge_width, window_rect.min.y),
        Vec2::new(edge_width, window_rect.height()),
    );

    let is_right_hovered = pointer_pos.map(|p| right_zone.contains(p)).unwrap_or(false);
    let right_btn_rect = Rect::from_center_size(
        Pos2::new(window_rect.max.x - 36.0, window_rect.center().y),
        btn_size,
    );

    if is_right_hovered {
        let resp = ui.allocate_rect(right_btn_rect, Sense::click());
        let bg_color = if resp.hovered() {
            Color32::from_rgba_unmultiplied(35, 35, 45, 230)
        } else {
            Color32::from_rgba_unmultiplied(20, 20, 28, 180)
        };

        ui.painter().rect_filled(right_btn_rect, CornerRadius::same(8), bg_color);
        ui.painter().rect_stroke(
            right_btn_rect,
            CornerRadius::same(8),
            Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 40)),
            egui::StrokeKind::Outside,
        );
        ui.painter().text(
            right_btn_rect.center(),
            egui::Align2::CENTER_CENTER,
            "▶",
            egui::FontId::proportional(22.0),
            Color32::WHITE,
        );

        if resp.clicked() {
            result.next_clicked = true;
        }
    }

    result
}
