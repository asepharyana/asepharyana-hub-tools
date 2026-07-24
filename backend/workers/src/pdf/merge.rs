use std::path::PathBuf;

use lopdf::{Document, Object, Dictionary};

use crate::nats::progress::ProgressReporter;
use tools_common::types::Job;

/// Merge multiple PDF files into one.
pub async fn process(
    job: &Job,
    output_dir: &PathBuf,
    _progress: &ProgressReporter,
) -> Result<PathBuf, Box<dyn std::error::Error + Send + Sync>> {
    let output_path = output_dir.join(format!("{}_merged.pdf", job.id));

    let mut file_paths = vec![job.file_path.clone()];
    if let Some(files) = job.options.get("files").and_then(|v| v.as_array()) {
        for f in files {
            if let Some(path) = f.as_str() {
                file_paths.push(path.to_string());
            }
        }
    }

    let mut merged = Document::new();

    // Pages tree for the merged doc
    let pages_id = merged.new_object_id();
    let mut kids = Vec::new();
    let mut page_count = 0u32;

    for path in &file_paths {
        let doc = Document::load(path)?;
        let src_pages = doc.get_pages();

        for (_, obj_id) in &src_pages {
            if let Ok(obj) = doc.get_object(*obj_id) {
                let mut page = obj.clone();

                // Set parent to merged pages
                if let Object::Dictionary(ref mut dict) = page {
                    dict.set("Parent", Object::Reference(pages_id));
                }

                // Add to merged document
                let new_id = merged.new_object_id();
                merged.objects.insert(new_id, page);
                kids.push(Object::Reference(new_id));
            }
        }
        page_count += src_pages.len() as u32;
    }

    // Build Pages dictionary
    let mut pages_dict = Dictionary::new();
    pages_dict.set("Type", Object::Name("Pages".as_bytes().to_vec()));
    pages_dict.set("Count", Object::Integer(page_count as i64));
    pages_dict.set("Kids", Object::Array(kids));
    merged.objects.insert(pages_id, Object::Dictionary(pages_dict));

    merged.save_to(&mut std::fs::File::create(&output_path)?)?;
    Ok(output_path)
}