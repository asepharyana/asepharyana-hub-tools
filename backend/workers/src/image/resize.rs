use std::path::PathBuf;

use image::{DynamicImage, ImageFormat};
use image::imageops::FilterType;

use crate::nats::progress::ProgressReporter;

pub async fn process(
    img: &DynamicImage,
    options: &serde_json::Value,
    output_dir: &PathBuf,
    _progress: &ProgressReporter,
) -> Result<PathBuf, Box<dyn std::error::Error + Send + Sync>> {
    let width = options.get("width").and_then(|v| v.as_u64()).map(|v| v as u32);
    let height = options.get("height").and_then(|v| v.as_u64()).map(|v| v as u32);
    let quality = options.get("quality").and_then(|v| v.as_u64()).unwrap_or(85) as u8;
    let fit = options.get("fit").and_then(|v| v.as_str()).unwrap_or("inside");
    let fmt = options.get("format").and_then(|v| v.as_str()).unwrap_or("jpeg");

    let (new_w, new_h) = match (width, height) {
        (Some(w), Some(h)) => (w, h),
        (Some(w), None) => {
            let ratio = w as f64 / img.width() as f64;
            (w, (img.height() as f64 * ratio).round() as u32)
        }
        (None, Some(h)) => {
            let ratio = h as f64 / img.height() as f64;
            ((img.width() as f64 * ratio).round() as u32, h)
        }
        (None, None) => (img.width(), img.height()),
    };

    let new_w = new_w.max(1).min(10000);
    let new_h = new_h.max(1).min(10000);

    let resized = match fit {
        "fill" => img.resize_exact(new_w, new_h, FilterType::Lanczos3),
        "crop" => img.resize_to_fill(new_w, new_h, FilterType::Lanczos3),
        _ => img.resize(new_w, new_h, FilterType::Lanczos3), // "inside" = fit within bounds
    };

    let image_format = match fmt {
        "png" => ImageFormat::Png,
        "webp" => ImageFormat::WebP,
        "gif" => ImageFormat::Gif,
        "bmp" => ImageFormat::Bmp,
        _ => ImageFormat::Jpeg,
    };

    let ext = match image_format {
        ImageFormat::Jpeg => "jpg",
        ImageFormat::Png => "png",
        ImageFormat::WebP => "webp",
        ImageFormat::Gif => "gif",
        ImageFormat::Bmp => "bmp",
        _ => "bin",
    };

    let output_path = output_dir.join(format!("resized.{}", ext));

    // Save with appropriate encoder
    match image_format {
        ImageFormat::Jpeg => {
            let mut rgb = resized.to_rgb8();
            let mut file = std::fs::File::create(&output_path)?;
            let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut file, quality);
            encoder.encode(rgb.as_raw(), resized.width(), resized.height(), image::ExtendedColorType::Rgb8)?;
        }
        ImageFormat::Png | ImageFormat::WebP | ImageFormat::Gif | ImageFormat::Bmp => {
            resized.save(&output_path)?;
        }
        _ => {
            resized.save(&output_path)?;
        }
    }

    Ok(output_path)
}