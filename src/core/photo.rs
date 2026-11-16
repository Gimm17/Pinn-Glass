use std::path::PathBuf;
use std::sync::Arc;
use crate::codecs::DecodedImage;

#[derive(Clone, Debug)]
pub struct PhotoItem {
    pub path: PathBuf,
    pub file_name: String,
    pub file_size_bytes: u64,
    pub dimensions: Option<(u32, u32)>,
    pub is_animated: bool,
    pub image: Option<Arc<DecodedImage>>,
    pub thumbnail: Option<Arc<DecodedImage>>,
    pub is_loading: bool,
    pub thumbnail_requested: bool,
    pub error: Option<String>,
}

impl PhotoItem {
    pub fn new(path: PathBuf) -> Self {
        let file_name = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or_default()
            .to_string();

        let file_size_bytes = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);

        Self {
            path,
            file_name,
            file_size_bytes,
            dimensions: None,
            is_animated: false,
            image: None,
            thumbnail: None,
            is_loading: false,
            thumbnail_requested: false,
            error: None,
        }
    }

    pub fn formatted_size(&self) -> String {
        let bytes = self.file_size_bytes;
        if bytes >= 1024 * 1024 * 1024 {
            format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
        } else if bytes >= 1024 * 1024 {
            format!("{:.2} MB", bytes as f64 / (1024.0 * 1024.0))
        } else if bytes >= 1024 {
            format!("{:.1} KB", bytes as f64 / 1024.0)
        } else {
            format!("{} B", bytes)
        }
    }
}
