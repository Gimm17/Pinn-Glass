use std::fs::File;
use std::io::BufReader;
use std::path::Path;

#[derive(Debug, Clone, Default)]
pub struct ExifMetadata {
    pub make: Option<String>,
    pub model: Option<String>,
    pub lens_model: Option<String>,
    pub date_time: Option<String>,
    pub iso: Option<u32>,
    pub f_number: Option<String>,
    pub exposure_time: Option<String>,
    pub focal_length: Option<String>,
    pub orientation: Option<u32>,
}

pub fn extract_exif(path: &Path) -> Option<ExifMetadata> {
    let file = File::open(path).ok()?;
    let mut bufreader = BufReader::new(file);
    let exifreader = exif::Reader::new();
    let exif = exifreader.read_from_container(&mut bufreader).ok()?;

    let mut meta = ExifMetadata::default();

    if let Some(field) = exif.get_field(exif::Tag::Make, exif::In::PRIMARY) {
        meta.make = Some(field.display_value().to_string().trim_matches('"').to_string());
    }

    if let Some(field) = exif.get_field(exif::Tag::Model, exif::In::PRIMARY) {
        meta.model = Some(field.display_value().to_string().trim_matches('"').to_string());
    }

    if let Some(field) = exif.get_field(exif::Tag::LensModel, exif::In::PRIMARY) {
        meta.lens_model = Some(field.display_value().to_string().trim_matches('"').to_string());
    }

    if let Some(field) = exif.get_field(exif::Tag::DateTimeOriginal, exif::In::PRIMARY) {
        meta.date_time = Some(field.display_value().to_string());
    }

    if let Some(field) = exif.get_field(exif::Tag::PhotographicSensitivity, exif::In::PRIMARY) {
        if let exif::Value::Short(ref v) = field.value {
            if let Some(&iso) = v.first() {
                meta.iso = Some(iso as u32);
            }
        }
    }

    if let Some(field) = exif.get_field(exif::Tag::FNumber, exif::In::PRIMARY) {
        meta.f_number = Some(format!("f/{}", field.display_value()));
    }

    if let Some(field) = exif.get_field(exif::Tag::ExposureTime, exif::In::PRIMARY) {
        meta.exposure_time = Some(format!("{}s", field.display_value()));
    }

    if let Some(field) = exif.get_field(exif::Tag::FocalLength, exif::In::PRIMARY) {
        meta.focal_length = Some(format!("{} mm", field.display_value()));
    }

    if let Some(field) = exif.get_field(exif::Tag::Orientation, exif::In::PRIMARY) {
        if let Some(val) = field.value.get_uint(0) {
            meta.orientation = Some(val);
        }
    }

    Some(meta)
}
