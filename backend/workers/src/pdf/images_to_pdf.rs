use std::path::PathBuf;

use lopdf::{Document, Object, Stream, Dictionary};

use crate::nats::progress::ProgressReporter;
use tools_common::types::Job;

/// Convert images into a single PDF.
pub async fn process(
    job: &Job,
    output_dir: &PathBuf,
    _progress: &ProgressReporter,
) -> Result<PathBuf, Box<dyn std::error::Error + Send + Sync>> {
    let output_path = output_dir.join(format!("{}_images.pdf", job.id));

    let mut image_paths = vec![job.file_path.clone()];
    if let Some(files) = job.options.get("files").and_then(|v| v.as_array()) {
        for f in files {
            if let Some(path) = f.as_str() {
                image_paths.push(path.to_string());
            }
        }
    }

    let mut doc = Document::new();
    let pages_id = doc.new_object_id();
    let mut kids = Vec::new();

    for img_path in &image_paths {
        let img_data = std::fs::read(img_path)?;
        let format = image::guess_format(&img_data).unwrap_or(image::ImageFormat::Jpeg);
        let img = image::load_from_memory(&img_data)
            .map_err(|e| format!("Cannot load image: {}", e))?;

        let jpeg_data = if format == image::ImageFormat::Jpeg {
            img_data
        } else {
            let mut buf = Vec::new();
            let rgb = img.to_rgb8();
            let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buf, 85);
            encoder.encode(rgb.as_raw(), rgb.width(), rgb.height(), image::ExtendedColorType::Rgb8)?;
            buf
        };

        let page_width = 595.28;
        let page_height = 841.89;
        let scale = (page_width / img.width() as f64).min(page_height / img.height() as f64) * 0.9;
        let ox = (page_width - img.width() as f64 * scale) / 2.0;
        let oy = (page_height - img.height() as f64 * scale) / 2.0;

        // Image XObject
        let mut img_dict = Dictionary::new();
        img_dict.set("Type", Object::Name("XObject".as_bytes().to_vec()));
        img_dict.set("Subtype", Object::Name("Image".as_bytes().to_vec()));
        img_dict.set("Width", Object::Integer(img.width() as i64));
        img_dict.set("Height", Object::Integer(img.height() as i64));
        img_dict.set("ColorSpace", Object::Name("DeviceRGB".as_bytes().to_vec()));
        img_dict.set("BitsPerComponent", Object::Integer(8));
        img_dict.set("Filter", Object::Name("DCTDecode".as_bytes().to_vec()));

        let img_stream = Stream::new(img_dict, jpeg_data);
        let img_id = doc.add_object(Object::Stream(img_stream));

        // Content
        let content = format!("q\n{} 0 0 {} {} {} cm\n/Im0 Do\nQ\n",
            img.width() as f64 * scale, img.height() as f64 * scale, ox, oy).into_bytes();
        let content_stream = Stream::new(Dictionary::new(), content);
        let content_id = doc.add_object(Object::Stream(content_stream));

        // Resources
        let mut xobj = Dictionary::new();
        xobj.set("Im0", Object::Reference(img_id));
        let mut resources = Dictionary::new();
        resources.set("XObject", Object::Dictionary(xobj));

        // Page
        let mut page = Dictionary::new();
        page.set("Type", Object::Name("Page".as_bytes().to_vec()));
        page.set("Parent", Object::Reference(pages_id));
        page.set("MediaBox", Object::Array(vec![
            Object::Real(0.0), Object::Real(0.0),
            Object::Real(page_width as f32), Object::Real(page_height as f32),
        ]));
        page.set("Contents", Object::Reference(content_id));
        page.set("Resources", Object::Dictionary(resources));

        let page_id = doc.new_object_id();
        kids.push(Object::Reference(page_id));
        doc.objects.insert(page_id, Object::Dictionary(page));
    }

    // Pages tree
    let mut pages_dict = Dictionary::new();
    pages_dict.set("Type", Object::Name("Pages".as_bytes().to_vec()));
    pages_dict.set("Count", Object::Integer(kids.len() as i64));
    pages_dict.set("Kids", Object::Array(kids));
    doc.objects.insert(pages_id, Object::Dictionary(pages_dict));

    doc.save_to(&mut std::fs::File::create(&output_path)?)?;
    Ok(output_path)
}