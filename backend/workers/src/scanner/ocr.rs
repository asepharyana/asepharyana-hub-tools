use image::GrayImage;
use tools_common::error::PipelineError;

/// OCR result with text and word-level bounding boxes.
pub struct OcrResult {
    pub full_text: String,
    pub words: Vec<OcrWord>,
    pub confidence: f32,
}

/// A single word detected by OCR with its bounding box.
#[derive(Debug, Clone)]
pub struct OcrWord {
    pub text: String,
    pub bbox: Bbox,
    pub confidence: i32,
}

/// Bounding box coordinates.
#[derive(Debug, Clone)]
pub struct Bbox {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

/// Run OCR on a grayscale image and return extracted text.
/// Requires the "tesseract" feature. Falls back gracefully when unavailable.
#[cfg(feature = "tesseract")]
pub fn ocr_text(img: &GrayImage, lang: &str) -> Result<OcrResult, PipelineError> {
    let tessdata_prefix = std::env::var("TESSDATA_PREFIX")
        .unwrap_or_else(|_| "/usr/share/tesseract-ocr/5/tessdata".to_string());

    let mut tess = leptess::LepTess::new(Some(&tessdata_prefix), lang)
        .map_err(|e| PipelineError::Ocr(format!("Failed to init Tesseract: {}", e)))?;

    tess.set_source_resolution(300);

    // Set image from raw bytes
    let bytes = img.to_vec();
    let w = img.width() as i32;
    let h = img.height() as i32;

    // leptess expects raw 8-bit grayscale data
    // Use set_image_from_mem which loads from memory buffer
    if let Err(e) = tess.set_image_from_mem(&bytes) {
        tracing::warn!("set_image_from_mem failed: {:?}, trying set_image", e);
        return Err(PipelineError::Ocr(format!("set_image_from_mem failed: {:?}", e)));
    }

    tess.recognize();

    let text = tess.get_utf8_text().unwrap_or_default();
    let confidence = tess.mean_text_conf() as f32;

    // Get word-level bounding boxes from LSTM box text
    let words = if let Ok(box_text) = tess.get_lstm_box_text(1) {
        parse_lstm_boxes(&box_text)
    } else {
        Vec::new()
    };

    Ok(OcrResult {
        full_text: text,
        words,
        confidence,
    })
}

/// Parse LSTM box text format: "word x1 y1 x2 y2 confidence"
fn parse_lstm_boxes(box_text: &str) -> Vec<OcrWord> {
    let mut words = Vec::new();
    for line in box_text.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 5 {
            if let (Ok(x), Ok(y), Ok(x2), Ok(y2)) = (
                parts[1].parse::<i32>(),
                parts[2].parse::<i32>(),
                parts[3].parse::<i32>(),
                parts[4].parse::<i32>(),
            ) {
                let confidence = parts.get(5).and_then(|v| v.parse::<i32>().ok()).unwrap_or(0);
                words.push(OcrWord {
                    text: parts[0].to_string(),
                    bbox: Bbox {
                        x,
                        y,
                        width: (x2 - x).max(1),
                        height: (y2 - y).max(1),
                    },
                    confidence,
                });
            }
        }
    }
    words
}

/// Non-tesseract fallback: return empty result.
#[cfg(not(feature = "tesseract"))]
pub fn ocr_text(_img: &GrayImage, _lang: &str) -> Result<OcrResult, PipelineError> {
    tracing::warn!("Tesseract feature not enabled, OCR returning placeholder");
    Ok(OcrResult {
        full_text: String::new(),
        words: Vec::new(),
        confidence: 0.0,
    })
}