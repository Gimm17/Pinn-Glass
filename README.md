<div align="center">

# 🖼️ PinnGlass

### Ultra-Fast, GPU-Accelerated Image Viewer in Rust for Windows 10/11

[![Rust](https://img.shields.io/badge/Rust-2021-orange?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Rendering](https://img.shields.io/badge/GPU-wgpu%20%2B%20SIMD-1f6feb?style=for-the-badge)](https://wgpu.rs/)
[![Platform](https://img.shields.io/badge/Platform-Windows%2010%20%2F%2011-0078D6?style=for-the-badge&logo=windows&logoColor=white)](https://www.microsoft.com/windows)

*Inspired by [ImageGlass v10](https://imageglass.org) — engineered with deterministic RAII memory management, lock-free worker channels, and hardware-accelerated viewport rendering.*

</div>

---

## ⚡ Key Highlights vs ImageGlass v10

| Feature | ImageGlass v10 (.NET 10 + Avalonia + Skia) | PinnGlass (Rust + `wgpu` + SIMD) |
| :--- | :--- | :--- |
| **Cold Start** | ~150 – 400 ms | **< 20 ms (Instant)** |
| **Memory Footprint** | ~80 – 160 MB (GC heap + visual tree) | **~20 – 45 MB** (Deterministic RAII, zero GC pauses) |
| **Binary Size** | ~75 – 140 MB | **~8 – 14 MB (Single standalone `.exe`)** |
| **Directory Navigation** | C# Async Tasks with Lock | **Lock-free worker channels + Spiral Prefetch Cache** |
| **Folder Sorting** | P/Invoke `StrCmpLogicalW` | **Native Windows Explorer Sort (`StrCmpLogicalW`)** |
| **Rendering** | SkiaSharp CPU/GPU Surface | **Hardware GPU Accelerated (`wgpu`) + Procedural Shaders** |

---

## 🚀 Features

- **Multi-Format Codec Pipeline**
  - Raster formats: `JPEG`, `PNG`, `WEBP`, `BMP`, `ICO`, `TIFF`, `HDR` (accelerated via `zune-jpeg` SIMD).
  - Animated `GIF` frame scheduler with real-time frame timing.
  - Dynamic vector rendering for `SVG` / `SVGZ` via `resvg` and `tiny-skia`.
- **Spiral Prefetch Cache (`+1, -1, +2, -2...`)** — Pre-decodes adjacent images onto GPU textures with configurable LRU eviction (`max_memory_cache_mb`, default `512 MB`).
- **Interactive GPU Viewport** — Cursor-centered wheel zoom, double-click `100%` / `Fit` toggle, drag-to-pan, checkerboard transparency shader, lossless 90° rotation, and horizontal/vertical flip.
- **Minimal UI Overlays** — Collapsible top toolbar (`T`), virtualized bottom gallery strip (`G`), edge navigation buttons, and floating EXIF HUD (`I`).
- **Windows Native Integration** — Natural Explorer sorting (`StrCmpLogicalW`), dark mode titlebar (`DwmSetWindowAttribute`), native Open File Dialog (`comdlg32`), drag-and-drop, and portable mode (`.igportable`).

---

## ⌨️ Keyboard & Mouse Shortcuts

| Shortcut | Action |
| :--- | :--- |
| `Right Arrow` / `D` / `Space` / `PageDown` | Next image |
| `Left Arrow` / `A` / `Backspace` / `PageUp` | Previous image |
| `Home` / `End` | First / Last image in folder |
| `Mouse Wheel` / `Ctrl + Scroll` | Zoom in / out (cursor-centered) |
| `Left Drag` | Pan image across viewport |
| `Double Click` | Toggle 100% zoom and Fit to Window |
| `Ctrl + 0` / `0` | Reset zoom to 100% (1:1 actual size) |
| `F` | Scale to Fit Window |
| `Ctrl + ,` / `Ctrl + .` | Rotate Left / Right (90°) |
| `Ctrl + O` | Open File Dialog |
| `I` | Toggle Floating EXIF HUD |
| `G` | Toggle Bottom Gallery Thumbnail Strip |
| `T` | Toggle Top Toolbar |
| `F11` / `Alt + Enter` | Toggle Fullscreen |
| `Escape` | Exit Fullscreen / Close EXIF HUD |

---

## 📁 Project Structure

```text
Pinn-Glass/
├── src/
│   ├── main.rs            # Application entry point
│   ├── app.rs             # Main eframe/wgpu UI & event loop
│   ├── lib.rs             # Shared library exports
│   ├── codecs/            # Image, EXIF, and SVG/SVGZ decoders
│   ├── core/              # Config, photo state, spiral cache manager
│   ├── platform/          # Win32 native integration (sorting, DWM, dialogs)
│   ├── ui/                # Toolbar, viewport, gallery strip, EXIF HUD, nav buttons
│   └── bin/gen_icon.rs    # Icon generator utility
├── tests/
│   └── test_codecs.rs     # Codec integration tests
├── install.bat / install.ps1
├── set_default.ps1
└── Cargo.toml
```

---

## 🛠️ Building & Running

### Prerequisites
- [Rust 1.88+](https://rustup.rs/) (Windows MSVC or GNU toolchain)

### Development & Testing
```powershell
# Run codec tests
cargo test

# Open with a specific image
cargo run -- "C:\Path\To\Your\Photo.jpg"

# Or launch empty (ready for drag-and-drop or Ctrl+O)
cargo run
```

### Release Build & Windows Install
```powershell
cargo build --release
```
The standalone executable is produced at `target/release/pinnglass.exe`. You can also use `install.ps1` / `set_default.ps1` to register PinnGlass on Windows.
