use std::path::{Path, PathBuf};

use lopdf::{Document, Object, Dictionary};

use crate::nats::progress::ProgressReporter;

/// Split a PDF by extracting page ranges.
pub async fn process(
    input_path: &Path,
    options: &serde_json::Value,
    output_dir: &PathBuf,
    _progress: &ProgressReporter,
) -> Result<PathBuf, Box<dyn std::error::Error + Send + Sync>> {
    let page_spec = options.get("pages").and_then(|v| v.as_str()).unwrap_or("1");
    let doc = Document::load(input_path)?;
    let src_pages = doc.get_pages();
    let total = src_pages.len() as u32;

    // Parse page spec: "1-3,5,7-9"
    let mut pages = Vec::new();
    for part in page_spec.split(',') {
        let part = part.trim();
        if let Some((start, end)) = part.split_once('-') {
            let s: u32 = start.trim().parse().unwrap_or(1);
            let e: u32 = end.trim().parse().unwrap_or(total);
            for p in s..=e.min(total) {
                pages.push(p);
            }
        } else if let Ok(p) = part.parse::<u32>() {
            pages.push(p);
        }
    }
    pages.sort();
    pages.dedup();

    let output_path = output_dir.join(format!("split_{}.pdf", uuid::Uuid::new_v4()));

    let mut new_doc = Document::new();
    let pages_id = new_doc.new_object_id();
    let mut kids = Vec::new();

    for page_num in &pages {
        if let Some(obj_id) = src_pages.get(page_num) {
            if let Ok(obj) = doc.get_object(*obj_id) {
                let mut page = obj.clone();
                if let Object::Dictionary(ref mut dict) = page {
                    dict.set("Parent", Object::Reference(pages_id));
                }
                let new_id = new_doc.new_object_id();
                new_doc.objects.insert(new_id, page);
                kids.push(Object::Reference(new_id));
            }
        }
    }

    let mut pages_dict = Dictionary::new();
    pages_dict.set("Type", Object::Name("Pages".as_bytes().to_vec()));
    pages_dict.set("Count", Object::Integer(kids.len() as i64));
    pages_dict.set("Kids", Object::Array(kids));
    new_doc.objects.insert(pages_id, Object::Dictionary(pages_dict));

    new_doc.save_to(&mut std::fs::File::create(&output_path)?)?;
    Ok(output_path)
}