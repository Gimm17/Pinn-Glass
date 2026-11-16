#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::PathBuf;
use pinnglass::PinnGlassApp;

fn main() -> eframe::Result<()> {
    // 1. Parse initial image path from command line arguments
    let args: Vec<String> = std::env::args().collect();
    let initial_path = if args.len() > 1 {
        let p = PathBuf::from(&args[1]);
        if p.exists() {
            Some(p)
        } else {
            None
        }
    } else {
        None
    };

    // 2. Configure native eframe window options for Windows 10/11
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("PinnGlass")
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([480.0, 320.0])
            .with_drag_and_drop(true)
            .with_active(true),
        ..Default::default()
    };

    // 3. Launch application
    eframe::run_native(
        "PinnGlass",
        options,
        Box::new(|_cc| Ok(Box::new(PinnGlassApp::new(initial_path)))),
    )
}
