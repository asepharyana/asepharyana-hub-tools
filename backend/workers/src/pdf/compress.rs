use std::path::{Path, PathBuf};

use lopdf::Document;

use crate::nats::progress::ProgressReporter;

/// Compress PDF by re-saving with compression.
pub async fn process(
    input_path: &Path,
    _options: &serde_json::Value,
    output_dir: &PathBuf,
    _progress: &ProgressReporter,
) -> Result<PathBuf, Box<dyn std::error::Error + Send + Sync>> {
    let mut doc = Document::load(input_path)?;

    let output_path = output_dir.join(format!("compressed_{}", input_path.file_name().unwrap_or_default().to_string_lossy()));
    doc.save_to(&mut std::fs::File::create(&output_path)?)?;

    Ok(output_path)
}