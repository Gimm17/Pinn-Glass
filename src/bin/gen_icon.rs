use std::fs::File;
use std::io::BufWriter;
use std::path::Path;
use image::codecs::ico::IcoEncoder;
use image::ImageEncoder;
use resvg::tiny_skia::{Pixmap, Transform};

const SVG_SOURCE: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512" width="512" height="512">
  <defs>
    <linearGradient id="bgGrad" x1="0%" y1="0%" x2="100%" y2="100%">
      <stop offset="0%" stop-color="#161824"/>
      <stop offset="100%" stop-color="#0a0b10"/>
    </linearGradient>

    <linearGradient id="borderGrad" x1="0%" y1="0%" x2="100%" y2="100%">
      <stop offset="0%" stop-color="#00F5A0" stop-opacity="0.9"/>
      <stop offset="45%" stop-color="#00D9F5" stop-opacity="0.7"/>
      <stop offset="100%" stop-color="#9d4edd" stop-opacity="0.9"/>
    </linearGradient>

    <linearGradient id="brandGrad" x1="0%" y1="0%" x2="100%" y2="100%">
      <stop offset="0%" stop-color="#00F5A0"/>
      <stop offset="40%" stop-color="#00D9F5"/>
      <stop offset="100%" stop-color="#9d4edd"/>
    </linearGradient>

    <linearGradient id="pStem" x1="0%" y1="0%" x2="100%" y2="100%">
      <stop offset="0%" stop-color="#ffffff"/>
      <stop offset="100%" stop-color="#d0d5e5"/>
    </linearGradient>

    <linearGradient id="glassShine" x1="0%" y1="0%" x2="0%" y2="100%">
      <stop offset="0%" stop-color="#ffffff" stop-opacity="0.18"/>
      <stop offset="100%" stop-color="#ffffff" stop-opacity="0.0"/>
    </linearGradient>
  </defs>

  <!-- Modern Dark Squircle Base -->
  <rect x="24" y="24" width="464" height="464" rx="108" fill="url(#bgGrad)" stroke="url(#borderGrad)" stroke-width="8"/>

  <!-- Glass Sheen Overlay -->
  <path d="M 24,132 C 24,60 60,24 132,24 L 380,24 C 452,24 488,60 488,132 L 488,230 C 340,280 170,260 24,200 Z" fill="url(#glassShine)"/>

  <!-- Outer Aperture Prism Frame -->
  <path d="M 256,86 L 406,172 L 406,340 L 256,426 L 106,340 L 106,172 Z" 
        fill="none" stroke="url(#brandGrad)" stroke-width="22" stroke-linejoin="round"/>

  <!-- Stylized Bold 'P' Camera Prism Shape -->
  <!-- Stem of P -->
  <rect x="180" y="150" width="38" height="212" rx="19" fill="url(#pStem)"/>
  
  <!-- Loop of P -->
  <path d="M 199,150 L 285,150 C 345,150 378,178 378,228 C 378,278 345,306 285,306 L 199,306 Z" 
        fill="none" stroke="url(#brandGrad)" stroke-width="38" stroke-linecap="round" stroke-linejoin="round"/>

  <!-- Optical Lens Focus Pupil -->
  <circle cx="282" cy="228" r="30" fill="#00F5A0"/>
  <circle cx="292" cy="218" r="10" fill="#ffffff"/>
</svg>"##;

fn render_svg_at_size(tree: &usvg::Tree, size: u32) -> Vec<u8> {
    let mut pixmap = Pixmap::new(size, size).expect("Failed to allocate pixmap");
    let scale = size as f32 / 512.0;
    let transform = Transform::from_scale(scale, scale);
    resvg::render(tree, transform, &mut pixmap.as_mut());

    let mut rgba = pixmap.take();
    // Demultiply alpha for clean RGBA8
    for pixel in rgba.chunks_exact_mut(4) {
        let a = pixel[3] as f32 / 255.0;
        if a > 0.0 && a < 1.0 {
            pixel[0] = ((pixel[0] as f32 / a).min(255.0)) as u8;
            pixel[1] = ((pixel[1] as f32 / a).min(255.0)) as u8;
            pixel[2] = ((pixel[2] as f32 / a).min(255.0)) as u8;
        }
    }
    rgba
}

fn main() {
    let opt = usvg::Options::default();
    let tree = usvg::Tree::from_str(SVG_SOURCE, &opt).expect("Failed to parse SVG");

    // Render 256x256 image
    let size = 256;
    let rgba256 = render_svg_at_size(&tree, size);

    // Save app.ico with 256x256 resolution
    let file = File::create("app.ico").expect("Failed to create app.ico");
    let writer = BufWriter::new(file);
    let encoder = IcoEncoder::new(writer);
    encoder
        .write_image(&rgba256, size, size, image::ExtendedColorType::Rgba8)
        .expect("Failed to write app.ico");

    println!("Successfully generated app.ico with PinnGlass brand emblem!");
}
