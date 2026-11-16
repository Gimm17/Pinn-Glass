use egui::{Color32, Key, Rect, Vec2};
use std::path::PathBuf;
use std::time::Instant;

use crate::core::config::{AppConfig, ZoomMode};
use crate::core::manager::PhotoManager;
use crate::platform::open_file_dialog;
use crate::ui::{
    render_exif_hud, render_gallery, render_nav_buttons, render_toolbar, render_viewport,
    GalleryState, ViewportState,
};

pub struct PinnGlassApp {
    pub config: AppConfig,
    pub manager: PhotoManager,
    pub viewport: ViewportState,
    pub gallery_state: GalleryState,

    // Current full-resolution GPU texture
    pub current_texture: Option<(usize, egui::TextureHandle)>,

    // Animation state for GIF/Animated WebP
    pub anim_timer: Instant,
    pub anim_frame_idx: usize,

    // UI visibility toggles
    pub show_exif: bool,
    pub show_gallery: bool,
    pub show_toolbar: bool,
    pub is_fullscreen: bool,

    // Cached current index to detect changes
    pub last_displayed_index: Option<usize>,
}

impl PinnGlassApp {
    pub fn new(initial_file: Option<PathBuf>) -> Self {
        let config = AppConfig::load();
        let mut manager = PhotoManager::new();

        if let Some(ref path) = initial_file {
            manager.open_path(path);
        }

        Self {
            show_exif: config.show_exif_hud,
            show_gallery: config.show_gallery,
            show_toolbar: config.show_toolbar,
            config,
            manager,
            viewport: ViewportState::default(),
            gallery_state: GalleryState::new(),
            current_texture: None,
            anim_timer: Instant::now(),
            anim_frame_idx: 0,
            is_fullscreen: false,
            last_displayed_index: None,
        }
    }

    fn open_dialog(&mut self) {
        if let Some(path) = open_file_dialog() {
            self.manager.open_path(&path);
            self.viewport.reset_view();
            self.current_texture = None;
        }
    }

    fn update_window_title(&self, ctx: &egui::Context) {
        let title = if let Some(photo) = self.manager.current_photo() {
            let dim_str = photo
                .dimensions
                .map(|(w, h)| format!(" [{}×{}]", w, h))
                .unwrap_or_default();

            let zoom_str = format!(" [{:.0}%]", self.viewport.zoom * 100.0);

            format!(
                "{} ({}/{}){}{}{} - PinnGlass",
                photo.file_name,
                self.manager.current_index + 1,
                self.manager.photos.len(),
                dim_str,
                zoom_str,
                if photo.is_animated { " [ANIMATED]" } else { "" }
            )
        } else {
            "PinnGlass - Ultra-Fast Image Viewer".to_string()
        };

        ctx.send_viewport_cmd(egui::ViewportCommand::Title(title));
    }
}

impl eframe::App for PinnGlassApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();

        // 1. Poll background worker decode results
        let _ = self.manager.poll_results(self.config.max_memory_cache_mb);

        // 2. Handle drag and drop files from OS
        ctx.input(|i| {
            if !i.raw.dropped_files.is_empty() {
                for file in &i.raw.dropped_files {
                    let path = file.path();
                    if !path.as_os_str().is_empty() {
                        self.manager.open_path(path);
                        self.viewport.reset_view();
                        self.current_texture = None;
                        break;
                    }
                }
            }
        });

        // 3. Handle Keyboard Shortcuts
        ctx.input(|i| {
            // Next photo: Right, D, Space, PageDown
            if i.key_pressed(Key::ArrowRight) || i.key_pressed(Key::D) || i.key_pressed(Key::Space) || i.key_pressed(Key::PageDown) {
                self.manager.next(self.config.loop_navigation);
            }

            // Prev photo: Left, A, Backspace, PageUp
            if i.key_pressed(Key::ArrowLeft) || i.key_pressed(Key::A) || i.key_pressed(Key::Backspace) || i.key_pressed(Key::PageUp) {
                self.manager.prev(self.config.loop_navigation);
            }

            // Home / End
            if i.key_pressed(Key::Home) {
                self.manager.goto(0);
            }
            if i.key_pressed(Key::End) && !self.manager.photos.is_empty() {
                self.manager.goto(self.manager.photos.len() - 1);
            }

            // Open File: Ctrl+O
            if i.modifiers.command && i.key_pressed(Key::O) {
                self.open_dialog();
            }

            // Rotate: Ctrl+, or Ctrl+.
            if i.modifiers.command && i.key_pressed(Key::Comma) {
                self.viewport.rotate_left();
            }
            if i.modifiers.command && i.key_pressed(Key::Period) {
                self.viewport.rotate_right();
            }

            // Actual Size: Ctrl+0 or 0
            if (i.modifiers.command && i.key_pressed(Key::Num0)) || i.key_pressed(Key::Num0) {
                self.viewport.zoom = 1.0;
                self.viewport.pan = Vec2::ZERO;
            }

            // Fit: F
            if i.key_pressed(Key::F) && !i.modifiers.command {
                self.viewport.zoom_mode = ZoomMode::FitToWindow;
                self.viewport.last_image_dim = None; // Trigger re-fit
            }

            // EXIF toggle: I
            if i.key_pressed(Key::I) && !i.modifiers.command {
                self.show_exif = !self.show_exif;
            }

            // Gallery toggle: G
            if i.key_pressed(Key::G) && !i.modifiers.command {
                self.show_gallery = !self.show_gallery;
            }

            // Toolbar toggle: T
            if i.key_pressed(Key::T) && !i.modifiers.command {
                self.show_toolbar = !self.show_toolbar;
            }

            // Fullscreen: F11 or Alt+Enter
            if i.key_pressed(Key::F11) || (i.modifiers.alt && i.key_pressed(Key::Enter)) {
                self.is_fullscreen = !self.is_fullscreen;
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(self.is_fullscreen));
            }

            // Escape: Exit fullscreen or close EXIF
            if i.key_pressed(Key::Escape) {
                if self.is_fullscreen {
                    self.is_fullscreen = false;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
                } else if self.show_exif {
                    self.show_exif = false;
                }
            }
        });

        // 4. Check if current photo changed -> update GPU Texture
        let cur_idx = self.manager.current_index;
        if self.last_displayed_index != Some(cur_idx) {
            self.last_displayed_index = Some(cur_idx);
            self.current_texture = None;
            self.anim_frame_idx = 0;
            self.anim_timer = Instant::now();
        }

        let (is_loading, error_msg, current_image) = if let Some(photo) = self.manager.current_photo() {
            (photo.is_loading, photo.error.clone(), photo.image.clone())
        } else {
            (false, None, None)
        };

        // Update animation frame if animated GIF
        if let Some(ref img) = current_image {
            if img.is_animated && !img.frames.is_empty() {
                let frame = &img.frames[self.anim_frame_idx % img.frames.len()];
                if self.anim_timer.elapsed().as_millis() >= frame.duration_ms as u128 {
                    self.anim_frame_idx = (self.anim_frame_idx + 1) % img.frames.len();
                    self.anim_timer = Instant::now();

                    // Update frame texture
                    let next_frame = &img.frames[self.anim_frame_idx];
                    let color_img = egui::ColorImage::from_rgba_unmultiplied(
                        [next_frame.width as usize, next_frame.height as usize],
                        &next_frame.rgba,
                    );
                    self.current_texture = Some((
                        cur_idx,
                        ctx.load_texture("current_animated_frame", color_img, egui::TextureOptions::LINEAR),
                    ));
                }
                // Request repaint for smooth animation playback
                ctx.request_repaint();
            } else if self.current_texture.as_ref().map(|(idx, _)| *idx) != Some(cur_idx) {
                // Static image: load texture into GPU once
                let color_img = egui::ColorImage::from_rgba_unmultiplied(
                    [img.width as usize, img.height as usize],
                    &img.rgba,
                );
                self.current_texture = Some((
                    cur_idx,
                    ctx.load_texture("current_image", color_img, egui::TextureOptions::LINEAR),
                ));
            }
        }

        // 5. Update window titlebar
        self.update_window_title(&ctx);

        // 6. Request gallery thumbnails for all photos in folder
        if self.show_gallery && !self.manager.photos.is_empty() {
            let total = self.manager.photos.len();
            self.manager.request_thumbnails_in_range(0, total);
        }

        // 7. Render UI Panels

        // Top Toolbar
        let mut toolbar_action = None;
        if self.show_toolbar {
            egui::Panel::top("top_toolbar")
                .frame(egui::Frame::new())
                .show(ui, |ui| {
                    let action = render_toolbar(
                        ui,
                        &mut self.viewport,
                        self.manager.current_index,
                        self.manager.photos.len(),
                        self.show_exif,
                        self.show_gallery,
                        self.is_fullscreen,
                    );
                    toolbar_action = Some(action);
                });
        }

        if let Some(action) = toolbar_action {
            if action.prev { self.manager.prev(self.config.loop_navigation); }
            if action.next { self.manager.next(self.config.loop_navigation); }
            if action.open_file { self.open_dialog(); }
            if action.zoom_in { self.viewport.zoom = (self.viewport.zoom * 1.2).min(100.0); }
            if action.zoom_out { self.viewport.zoom = (self.viewport.zoom / 1.2).max(0.01); }
            if action.zoom_100 { self.viewport.zoom = 1.0; self.viewport.pan = Vec2::ZERO; }
            if action.zoom_fit {
                self.viewport.zoom_mode = ZoomMode::FitToWindow;
                self.viewport.last_image_dim = None;
            }
            if action.rotate_left { self.viewport.rotate_left(); }
            if action.rotate_right { self.viewport.rotate_right(); }
            if action.flip_h { self.viewport.toggle_flip_h(); }
            if action.flip_v { self.viewport.toggle_flip_v(); }
            if action.toggle_exif { self.show_exif = !self.show_exif; }
            if action.toggle_gallery { self.show_gallery = !self.show_gallery; }
            if action.toggle_fullscreen {
                self.is_fullscreen = !self.is_fullscreen;
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(self.is_fullscreen));
            }
        }

        // Bottom Gallery Strip
        let mut gallery_selected_idx = None;
        if self.show_gallery && !self.manager.photos.is_empty() {
            egui::Panel::bottom("bottom_gallery")
                .frame(egui::Frame::new())
                .show(ui, |ui| {
                    render_gallery(
                        ui,
                        &self.manager.photos,
                        self.manager.current_index,
                        &mut self.gallery_state,
                        &mut gallery_selected_idx,
                    );
                });
        }

        if let Some(idx) = gallery_selected_idx {
            self.manager.goto(idx);
        }

        // Central Viewport Area
        let mut nav_action = None;
        let mut close_exif_requested = false;

        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(Color32::from_rgb(
                self.config.bg_color[0],
                self.config.bg_color[1],
                self.config.bg_color[2],
            )))
            .show(ui, |ui| {
                let rect = ui.available_rect_before_wrap();

                // 1. Render GPU Viewport (Checkerboard + Image Quad)
                let _ = render_viewport(
                    ui,
                    &mut self.viewport,
                    current_image.as_ref(),
                    self.current_texture.as_ref().map(|(_, t)| t),
                    is_loading,
                    error_msg.as_deref(),
                );

                // 2. Render Left / Right Hover Navigation Buttons
                let nav_res = render_nav_buttons(ui, rect, !self.manager.photos.is_empty());
                if nav_res.prev_clicked {
                    nav_action = Some(false);
                } else if nav_res.next_clicked {
                    nav_action = Some(true);
                }

                // 3. Render Floating EXIF HUD
                if self.show_exif {
                    if let Some(photo) = self.manager.current_photo() {
                        let hud_pos = rect.min + Vec2::new(16.0, 16.0);
                        let hud_rect = Rect::from_min_size(hud_pos, Vec2::new(320.0, 400.0));
                        ui.scope_builder(egui::UiBuilder::new().max_rect(hud_rect), |ui| {
                            render_exif_hud(ui, photo, &mut close_exif_requested);
                        });
                    }
                }
            });

        if close_exif_requested {
            self.show_exif = false;
        }

        if let Some(next) = nav_action {
            if next {
                self.manager.next(self.config.loop_navigation);
            } else {
                self.manager.prev(self.config.loop_navigation);
            }
        }
    }
}
