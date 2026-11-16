use std::path::Path;
use pinnglass::codecs::load_image;

#[test]
fn test_decode_png() {
    let path = Path::new(r"C:\Users\Lyvn\Documents\CODINGAN\BOT\Image-Viewer\source\__assets\logo_c_512.png");
    if path.exists() {
        let res = load_image(path).expect("Failed to decode PNG");
        assert_eq!(res.width, 512);
        assert_eq!(res.height, 512);
        assert_eq!(res.rgba.len(), 512 * 512 * 4);
    }
}

#[test]
fn test_decode_svg() {
    let path = Path::new(r"C:\Users\Lyvn\Documents\CODINGAN\BOT\Image-Viewer\source\__assets\logo_c.svg");
    if path.exists() {
        let res = load_image(path).expect("Failed to decode SVG");
        assert!(res.width > 0);
        assert!(res.height > 0);
        assert_eq!(res.rgba.len(), (res.width * res.height * 4) as usize);
    }
}

#[test]
fn test_decode_ico() {
    let path = Path::new(r"C:\Users\Lyvn\Documents\CODINGAN\BOT\Image-Viewer\source\ImageGlass.Win32\icon256.ico");
    if path.exists() {
        let res = load_image(path).expect("Failed to decode ICO");
        assert_eq!(res.width, 256);
        assert_eq!(res.height, 256);
    }
}
