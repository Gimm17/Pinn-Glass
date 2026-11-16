use std::cmp::Ordering;
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::path::PathBuf;

#[cfg(windows)]
#[link(name = "shlwapi")]
extern "system" {
    fn StrCmpLogicalW(psz1: *const u16, psz2: *const u16) -> i32;
}

/// Convert an OsStr to a null-terminated UTF-16 vector
pub fn to_wide_null(s: &OsStr) -> Vec<u16> {
    s.encode_wide().chain(std::iter::once(0)).collect()
}

/// Compares two strings using Windows Explorer natural sort order (StrCmpLogicalW)
#[cfg(windows)]
pub fn compare_natural(a: &OsStr, b: &OsStr) -> Ordering {
    let wide_a = to_wide_null(a);
    let wide_b = to_wide_null(b);

    let res = unsafe { StrCmpLogicalW(wide_a.as_ptr(), wide_b.as_ptr()) };
    res.cmp(&0)
}

#[cfg(not(windows))]
pub fn compare_natural(a: &OsStr, b: &OsStr) -> Ordering {
    a.cmp(b)
}

/// Sort a list of PathBuf using Windows Explorer natural sorting logic
pub fn natural_sort_paths(paths: &mut [PathBuf]) {
    paths.sort_by(|a, b| {
        let name_a = a.file_name().unwrap_or_default();
        let name_b = b.file_name().unwrap_or_default();
        compare_natural(name_a, name_b)
    });
}

#[cfg(windows)]
#[repr(C)]
#[allow(non_snake_case)]
struct OPENFILENAMEW {
    lStructSize: u32,
    hwndOwner: *mut std::ffi::c_void,
    hInstance: *mut std::ffi::c_void,
    lpstrFilter: *const u16,
    lpstrCustomFilter: *mut u16,
    nMaxCustFilter: u32,
    nFilterIndex: u32,
    lpstrFile: *mut u16,
    nMaxFile: u32,
    lpstrFileTitle: *mut u16,
    nMaxFileTitle: u32,
    lpstrInitialDir: *const u16,
    lpstrTitle: *const u16,
    Flags: u32,
    nFileOffset: u16,
    nFileExtension: u16,
    lpstrDefExt: *const u16,
    lCustData: usize,
    lpfnHook: Option<unsafe extern "system" fn() -> usize>,
    lpTemplateName: *const u16,
    pvReserved: *mut std::ffi::c_void,
    dwReserved: u32,
    FlagsEx: u32,
}

#[cfg(windows)]
#[link(name = "comdlg32")]
extern "system" {
    fn GetOpenFileNameW(ofn: *mut OPENFILENAMEW) -> i32;
}

#[cfg(windows)]
pub fn open_file_dialog() -> Option<PathBuf> {
    let mut file_buf = vec![0u16; 2048];
    // Filter: "All Supported Images\0*.jpg;*.jpeg;*.png;*.webp;*.gif;*.bmp;*.tiff;*.hdr;*.svg\0All Files (*.*)\0*.*\0\0"
    let filter: Vec<u16> = "All Supported Images\0*.jpg;*.jpeg;*.jpe;*.jfif;*.png;*.webp;*.gif;*.bmp;*.tiff;*.tif;*.hdr;*.svg;*.svgz\0All Files (*.*)\0*.*\0\0"
        .encode_utf16()
        .collect();

    let mut ofn: OPENFILENAMEW = unsafe { std::mem::zeroed() };
    ofn.lStructSize = std::mem::size_of::<OPENFILENAMEW>() as u32;
    ofn.lpstrFile = file_buf.as_mut_ptr();
    ofn.nMaxFile = file_buf.len() as u32;
    ofn.lpstrFilter = filter.as_ptr();
    ofn.nFilterIndex = 1;
    ofn.Flags = 0x00080000 | 0x00001000 | 0x00000800; // OFN_EXPLORER | OFN_FILEMUSTEXIST | OFN_PATHMUSTEXIST

    let res = unsafe { GetOpenFileNameW(&mut ofn) };
    if res != 0 {
        let len = file_buf.iter().position(|&c| c == 0).unwrap_or(file_buf.len());
        let path_str = String::from_utf16_lossy(&file_buf[..len]);
        Some(PathBuf::from(path_str))
    } else {
        None
    }
}

#[cfg(not(windows))]
pub fn open_file_dialog() -> Option<PathBuf> {
    None
}

/// Set dark mode titlebar for Windows 10 (build 1809+) and Windows 11
#[cfg(windows)]
pub fn enable_dark_mode_for_window(hwnd: isize) {
    use windows_sys::Win32::Graphics::Dwm::{DwmSetWindowAttribute, DWMWA_USE_IMMERSIVE_DARK_MODE};
    let use_dark_mode: i32 = 1;
    unsafe {
        DwmSetWindowAttribute(
            hwnd as _,
            DWMWA_USE_IMMERSIVE_DARK_MODE as _,
            &use_dark_mode as *const _ as *const _,
            std::mem::size_of::<i32>() as u32,
        );
    }
}

#[cfg(not(windows))]
pub fn enable_dark_mode_for_window(_hwnd: isize) {}
