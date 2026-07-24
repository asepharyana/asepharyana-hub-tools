use std::path::PathBuf;

use image::{DynamicImage, ImageFormat};

use crate::nats::progress::ProgressReporter;

pub async fn process(
    img: &DynamicImage,
    options: &serde_json::Value,
    output_dir: &PathBuf,
    _progress: &ProgressReporter,
) -> Result<PathBuf, Box<dyn std::error::Error + Send + Sync>> {
    let fmt = options.get("format").and_then(|v| v.as_str()).unwrap_or("jpeg");
    let quality = options.get("quality").and_then(|v| v.as_u64()).unwrap_or(85) as u8;

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

    let output_path = output_dir.join(format!("converted.{}", ext));

    match image_format {
        ImageFormat::Jpeg => {
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
        }
        ImageFormat::Png | ImageFormat::WebP | ImageFormat::Gif | ImageFormat::Bmp => {
            img.save(&output_path)?;
        }
        _ => {
            img.save(&output_path)?;
        }
    }

    Ok(output_path)
}