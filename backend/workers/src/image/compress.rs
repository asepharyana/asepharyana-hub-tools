use std::path::PathBuf;

use image::{DynamicImage, ImageFormat};

use crate::nats::progress::ProgressReporter;

pub async fn process(
    img: &DynamicImage,
    options: &serde_json::Value,
    output_dir: &PathBuf,
    _progress: &ProgressReporter,
) -> Result<PathBuf, Box<dyn std::error::Error + Send + Sync>> {
    let quality = options.get("quality").and_then(|v| v.as_u64()).unwrap_or(80) as u8;

    let output_path = output_dir.join("compressed.jpg");
    let mut file = std::fs::File::create(&output_path)?;

    if img.color().has_color() {
        let rgb = img.to_rgb8();
        let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut file, quality);
        encoder.encode(rgb.as_raw(), rgb.width(), rgb.height(), image::ExtendedColorType::Rgb8)?;
    } else {
        let gray = img.to_luma8();
        let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut file, quality);
        encoder.encode(gray.as_raw(), gray.width(), gray.height(), image::ExtendedColorType::L8)?;
    }

    Ok(output_path)
}