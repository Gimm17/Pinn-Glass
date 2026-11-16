use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use crossbeam_channel::{Receiver, Sender};
use crate::codecs::{is_supported_image, load_image, load_thumbnail, DecodedImage};
use crate::core::photo::PhotoItem;
use crate::platform::natural_sort_paths;

pub struct LoadRequest {
    pub path: PathBuf,
    pub index: usize,
    pub epoch: u64,
    pub is_thumbnail: bool,
}

pub struct LoadResult {
    pub index: usize,
    pub epoch: u64,
    pub is_thumbnail: bool,
    pub result: Result<Arc<DecodedImage>, String>,
}

pub struct PhotoManager {
    pub photos: Vec<PhotoItem>,
    pub current_index: usize,
    pub epoch: u64,
    current_epoch_atomic: Arc<AtomicU64>,
    req_sender: Sender<LoadRequest>,
    res_receiver: Receiver<LoadResult>,
}

impl PhotoManager {
    pub fn new() -> Self {
        let (req_sender, req_receiver) = crossbeam_channel::unbounded::<LoadRequest>();
        let (res_sender, res_receiver) = crossbeam_channel::unbounded::<LoadResult>();
        let current_epoch_atomic = Arc::new(AtomicU64::new(0));

        // Spawn background worker pool (4 worker threads for fast multi-core decode)
        let num_threads = std::thread::available_parallelism()
            .map(|n| n.get().clamp(2, 6))
            .unwrap_or(4);

        for _ in 0..num_threads {
            let req_rx = req_receiver.clone();
            let res_tx = res_sender.clone();
            let epoch_atomic = current_epoch_atomic.clone();

            std::thread::spawn(move || {
                while let Ok(req) = req_rx.recv() {
                    // Quick check if epoch is already stale (unless it's a thumbnail request)
                    let current_ep = epoch_atomic.load(Ordering::Relaxed);
                    if !req.is_thumbnail && req.epoch < current_ep {
                        continue;
                    }

                    let result = if req.is_thumbnail {
                        load_thumbnail(&req.path, 160).map(Arc::new)
                    } else {
                        load_image(&req.path).map(Arc::new)
                    };

                    let _ = res_tx.send(LoadResult {
                        index: req.index,
                        epoch: req.epoch,
                        is_thumbnail: req.is_thumbnail,
                        result,
                    });
                }
            });
        }

        Self {
            photos: Vec::new(),
            current_index: 0,
            epoch: 0,
            current_epoch_atomic,
            req_sender,
            res_receiver,
        }
    }

    pub fn current_photo(&self) -> Option<&PhotoItem> {
        self.photos.get(self.current_index)
    }

    pub fn current_photo_mut(&mut self) -> Option<&mut PhotoItem> {
        self.photos.get_mut(self.current_index)
    }

    pub fn open_path(&mut self, target: &Path) {
        let (dir, target_file) = if target.is_dir() {
            (target.to_path_buf(), None)
        } else {
            let parent = target.parent().unwrap_or_else(|| Path::new(".")).to_path_buf();
            (parent, Some(target.to_path_buf()))
        };

        let mut paths = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_file() && is_supported_image(&p) {
                    paths.push(p);
                }
            }
        }

        // Apply Windows Explorer natural sort order!
        natural_sort_paths(&mut paths);

        let initial_index = if let Some(ref target) = target_file {
            paths.iter().position(|p| p == target).unwrap_or(0)
        } else {
            0
        };

        self.photos = paths.into_iter().map(PhotoItem::new).collect();
        self.current_index = initial_index.min(self.photos.len().saturating_sub(1));
        self.trigger_navigation();
        self.request_all_thumbnails();
    }

    pub fn next(&mut self, loop_nav: bool) {
        if self.photos.is_empty() {
            return;
        }
        if self.current_index + 1 < self.photos.len() {
            self.current_index += 1;
            self.trigger_navigation();
        } else if loop_nav {
            self.current_index = 0;
            self.trigger_navigation();
        }
    }

    pub fn prev(&mut self, loop_nav: bool) {
        if self.photos.is_empty() {
            return;
        }
        if self.current_index > 0 {
            self.current_index -= 1;
            self.trigger_navigation();
        } else if loop_nav {
            self.current_index = self.photos.len().saturating_sub(1);
            self.trigger_navigation();
        }
    }

    pub fn goto(&mut self, index: usize) {
        if index < self.photos.len() && index != self.current_index {
            self.current_index = index;
            self.trigger_navigation();
        }
    }

    fn trigger_navigation(&mut self) {
        self.epoch += 1;
        self.current_epoch_atomic.store(self.epoch, Ordering::Relaxed);

        if self.photos.is_empty() {
            return;
        }

        // 1. Mark current photo loading if not loaded
        let cur_idx = self.current_index;
        if self.photos[cur_idx].image.is_none() {
            self.photos[cur_idx].is_loading = true;
            let _ = self.req_sender.send(LoadRequest {
                path: self.photos[cur_idx].path.clone(),
                index: cur_idx,
                epoch: self.epoch,
                is_thumbnail: false,
            });
        }

        // 2. Queue spiral prefetch (+1, -1, +2, -2, ...)
        let total = self.photos.len();
        let prefetch_depth = 4; // prefetch 4 photos ahead and 4 behind
        for step in 1..=prefetch_depth {
            // Forward
            let fwd = (cur_idx + step) % total;
            if self.photos[fwd].image.is_none() {
                let _ = self.req_sender.send(LoadRequest {
                    path: self.photos[fwd].path.clone(),
                    index: fwd,
                    epoch: self.epoch,
                    is_thumbnail: false,
                });
            }

            // Backward
            let bwd = (cur_idx + total - (step % total)) % total;
            if self.photos[bwd].image.is_none() {
                let _ = self.req_sender.send(LoadRequest {
                    path: self.photos[bwd].path.clone(),
                    index: bwd,
                    epoch: self.epoch,
                    is_thumbnail: false,
                });
            }
        }
    }

    pub fn request_thumbnails_in_range(&mut self, start: usize, end: usize) {
        let end = end.min(self.photos.len());
        for idx in start..end {
            let photo = &mut self.photos[idx];
            if photo.thumbnail.is_none() && !photo.thumbnail_requested {
                photo.thumbnail_requested = true;
                let _ = self.req_sender.send(LoadRequest {
                    path: photo.path.clone(),
                    index: idx,
                    epoch: self.epoch,
                    is_thumbnail: true,
                });
            }
        }
    }

    pub fn request_all_thumbnails(&mut self) {
        let total = self.photos.len();
        self.request_thumbnails_in_range(0, total);
    }

    /// Process received results from background threads (call on every frame)
    pub fn poll_results(&mut self, max_budget_mb: usize) -> bool {
        let mut updated = false;

        while let Ok(res) = self.res_receiver.try_recv() {
            if let Some(photo) = self.photos.get_mut(res.index) {
                if res.is_thumbnail {
                    if let Ok(thumb) = res.result {
                        photo.thumbnail = Some(thumb);
                        updated = true;
                    }
                } else {
                    // Full image decode result
                    photo.is_loading = false;
                    match res.result {
                        Ok(img) => {
                            photo.dimensions = Some((img.width, img.height));
                            photo.is_animated = img.is_animated;
                            photo.image = Some(img);
                            photo.error = None;
                            updated = true;
                        }
                        Err(e) => {
                            photo.error = Some(e);
                            updated = true;
                        }
                    }
                }
            }
        }

        if updated {
            self.enforce_memory_budget(max_budget_mb);
        }

        updated
    }

    /// Evicts photos outside the spiral prefetch window when exceeding memory budget
    fn enforce_memory_budget(&mut self, max_budget_mb: usize) {
        let budget_bytes = max_budget_mb * 1024 * 1024;
        let mut total_used: usize = self
            .photos
            .iter()
            .filter_map(|p| p.image.as_ref())
            .map(|img| img.memory_size_bytes())
            .sum();

        if total_used <= budget_bytes || self.photos.is_empty() {
            return;
        }

        let cur = self.current_index;
        let total = self.photos.len();

        // Evict items from furthest spiral distance inward
        let mut distances: Vec<(usize, usize)> = (0..total)
            .filter(|&i| i != cur && self.photos[i].image.is_some())
            .map(|i| {
                let diff = if i > cur { i - cur } else { cur - i };
                let spiral_dist = diff.min(total - diff);
                (i, spiral_dist)
            })
            .collect();

        // Sort by distance descending (furthest first)
        distances.sort_by(|a, b| b.1.cmp(&a.1));

        for (idx, _) in distances {
            if total_used <= budget_bytes {
                break;
            }
            if let Some(img) = self.photos[idx].image.take() {
                total_used = total_used.saturating_sub(img.memory_size_bytes());
            }
        }
    }
}
