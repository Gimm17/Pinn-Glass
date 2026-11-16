use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZoomMode {
    FitToWindow,
    ActualSize,
    ScaleToFill,
    LockZoom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub window_width: f32,
    pub window_height: f32,
    pub zoom_mode: ZoomMode,
    pub max_memory_cache_mb: usize,
    pub show_toolbar: bool,
    pub show_gallery: bool,
    pub show_exif_hud: bool,
    pub loop_navigation: bool,
    pub bg_color: [u8; 4],
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            window_width: 1280.0,
            window_height: 800.0,
            zoom_mode: ZoomMode::FitToWindow,
            max_memory_cache_mb: 512,
            show_toolbar: true,
            show_gallery: true,
            show_exif_hud: false,
            loop_navigation: true,
            bg_color: [24, 24, 28, 255], // Sleek modern dark background
        }
    }
}

impl AppConfig {
    /// Determines the config directory based on portable mode (.igportable marker)
    pub fn config_dir() -> PathBuf {
        let exe_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| PathBuf::from("."));

        // If .igportable exists next to the binary, use portable mode
        let portable_marker = exe_dir.join(".igportable");
        if portable_marker.exists() {
            return exe_dir;
        }

        // Otherwise use standard AppData\Local\PinnGlass
        if let Some(proj_dirs) = directories::ProjectDirs::from("com", "Lyvn", "PinnGlass") {
            let config_dir = proj_dirs.config_local_dir().to_path_buf();
            let _ = fs::create_dir_all(&config_dir);
            config_dir
        } else {
            exe_dir
        }
    }

    pub fn config_path() -> PathBuf {
        Self::config_dir().join("pinnglass.json")
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(cfg) = serde_json::from_str::<AppConfig>(&content) {
                    return cfg;
                }
            }
        }
        Self::default()
    }

    /// Atomic write: writes to temp file first, then renames over target
    pub fn save(&self) -> Result<(), String> {
        let path = Self::config_path();
        let pid = std::process::id();
        let tmp_path = path.with_extension(format!("tmp.{}", pid));

        let content = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        fs::write(&tmp_path, content).map_err(|e| e.to_string())?;

        // Atomic rename over target file
        fs::rename(&tmp_path, &path).map_err(|e| {
            let _ = fs::remove_file(&tmp_path);
            e.to_string()
        })?;

        Ok(())
    }
}
