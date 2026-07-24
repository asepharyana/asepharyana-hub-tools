use std::path::PathBuf;

/// Convert PDF pages to images.
/// TODO: Phase 3.1 - requires rendering PDF pages to bitmaps
pub fn process() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing::warn!("PDF to images not yet implemented (requires PDF renderer)");
    Ok(())
}