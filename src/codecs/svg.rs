use std::path::Path;
use resvg::tiny_skia::{Pixmap, Transform};

pub fn render_svg(path: &Path, max_dim: Option<u32>) -> Result<(u32, u32, Vec<u8>), String> {
    let svg_data = std::fs::read(path).map_err(|e| e.to_string())?;
    let opt = usvg::Options::default();
    let tree = usvg::Tree::from_data(&svg_data, &opt).map_err(|e| e.to_string())?;

    let size = tree.size();
    let original_w = size.width();
    let original_h = size.height();

    let (target_w, target_h, scale) = if let Some(max_d) = max_dim {
        let max_d = max_d as f32;
        let s = (max_d / original_w).min(max_d / original_h).min(1.0);
        ((original_w * s).round() as u32, (original_h * s).round() as u32, s)
    } else {
        // If image is too small or standard vector, render at high quality default
        let scale = if original_w < 512.0 && original_h < 512.0 {
            (1024.0 / original_w.max(original_h)).clamp(1.0, 4.0)
        } else {
            1.0
        };
        ((original_w * scale).round() as u32, (original_h * scale).round() as u32, scale)
    };

    let target_w = target_w.max(1);
    let target_h = target_h.max(1);

    let mut pixmap = Pixmap::new(target_w, target_h)
        .ok_or_else(|| "Failed to allocate pixmap for SVG".to_string())?;

    let transform = Transform::from_scale(scale, scale);
    resvg::render(&tree, transform, &mut pixmap.as_mut());

    // Convert premultiplied RGBA from tiny-skia to standard unmultiplied RGBA8
    let mut rgba = pixmap.take();
    for pixel in rgba.chunks_exact_mut(4) {
        let a = pixel[3] as f32 / 255.0;
        if a > 0.0 && a < 1.0 {
            pixel[0] = ((pixel[0] as f32 / a).min(255.0)) as u8;
            pixel[1] = ((pixel[1] as f32 / a).min(255.0)) as u8;
            pixel[2] = ((pixel[2] as f32 / a).min(255.0)) as u8;
        }
    }

    Ok((target_w, target_h, rgba))
}
