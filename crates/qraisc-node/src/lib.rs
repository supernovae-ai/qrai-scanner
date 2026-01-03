use napi::bindgen_prelude::*;
use napi_derive::napi;
use qraisc_core::{
    decode_only as core_decode_only, validate as core_validate,
    validate_fast as core_validate_fast, ErrorCorrectionLevel,
};

/// QR code validation result
#[napi(object)]
pub struct ValidationResult {
    /// Scannability score from 0-100
    pub score: u8,
    /// Whether the QR code was successfully decoded
    pub decodable: bool,
    /// Decoded content of the QR code
    pub content: Option<String>,
    /// QR code version (1-40)
    pub version: Option<u8>,
    /// Error correction level (L, M, Q, H)
    pub error_correction: Option<String>,
    /// Number of modules in the QR code
    pub modules: Option<u8>,
    /// List of decoders that successfully decoded the QR
    pub decoders_success: Vec<String>,
    /// Whether original image was decodable
    pub stress_original: bool,
    /// Whether 50% downscaled image was decodable
    pub stress_downscale_50: bool,
    /// Whether 25% downscaled image was decodable
    pub stress_downscale_25: bool,
    /// Whether lightly blurred image was decodable
    pub stress_blur_light: bool,
    /// Whether medium blurred image was decodable
    pub stress_blur_medium: bool,
    /// Whether low contrast image was decodable
    pub stress_low_contrast: bool,
}

/// Simple decode result (without stress tests)
#[napi(object)]
pub struct DecodeResult {
    /// Decoded content of the QR code
    pub content: String,
    /// QR code version (1-40)
    pub version: Option<u8>,
    /// Error correction level (L, M, Q, H)
    pub error_correction: Option<String>,
    /// Number of modules in the QR code
    pub modules: Option<u8>,
}

/// Validate a QR code image and compute scannability score
///
/// @param imageBuffer - Raw image bytes (PNG, JPEG, etc.)
/// @returns ValidationResult with score, content, and metadata
#[napi]
pub fn validate(image_buffer: Buffer) -> Result<ValidationResult> {
    let result = core_validate(&image_buffer)
        .map_err(|e| Error::from_reason(e.to_string()))?;

    let (version, error_correction, modules, decoders_success) =
        if let Some(ref meta) = result.metadata {
            (
                Some(meta.version),
                Some(ec_to_string(meta.error_correction)),
                Some(meta.modules),
                meta.decoders_success.clone(),
            )
        } else {
            (None, None, None, vec![])
        };

    Ok(ValidationResult {
        score: result.score,
        decodable: result.decodable,
        content: result.content,
        version,
        error_correction,
        modules,
        decoders_success,
        stress_original: result.stress_results.original,
        stress_downscale_50: result.stress_results.downscale_50,
        stress_downscale_25: result.stress_results.downscale_25,
        stress_blur_light: result.stress_results.blur_light,
        stress_blur_medium: result.stress_results.blur_medium,
        stress_low_contrast: result.stress_results.low_contrast,
    })
}

/// Fast decode without stress tests (for when you only need content)
///
/// @param imageBuffer - Raw image bytes (PNG, JPEG, etc.)
/// @returns DecodeResult with content and basic metadata
#[napi]
pub fn decode(image_buffer: Buffer) -> Result<DecodeResult> {
    let result = core_decode_only(&image_buffer)
        .map_err(|e| Error::from_reason(e.to_string()))?;

    let (version, error_correction, modules) = if let Some(ref meta) = result.metadata {
        (
            Some(meta.version),
            Some(ec_to_string(meta.error_correction)),
            Some(meta.modules),
        )
    } else {
        (None, None, None)
    };

    Ok(DecodeResult {
        content: result.content,
        version,
        error_correction,
        modules,
    })
}

/// Fast validation with reduced stress tests (~2x faster)
///
/// Good for real-time feedback during QR editing.
///
/// @param imageBuffer - Raw image bytes (PNG, JPEG, etc.)
/// @returns ValidationResult with score, content, and metadata
#[napi]
pub fn validate_fast(image_buffer: Buffer) -> Result<ValidationResult> {
    let result = core_validate_fast(&image_buffer)
        .map_err(|e| Error::from_reason(e.to_string()))?;

    let (version, error_correction, modules, decoders_success) =
        if let Some(ref meta) = result.metadata {
            (
                Some(meta.version),
                Some(ec_to_string(meta.error_correction)),
                Some(meta.modules),
                meta.decoders_success.clone(),
            )
        } else {
            (None, None, None, vec![])
        };

    Ok(ValidationResult {
        score: result.score,
        decodable: result.decodable,
        content: result.content,
        version,
        error_correction,
        modules,
        decoders_success,
        stress_original: result.stress_results.original,
        stress_downscale_50: result.stress_results.downscale_50,
        stress_downscale_25: result.stress_results.downscale_25,
        stress_blur_light: result.stress_results.blur_light,
        stress_blur_medium: result.stress_results.blur_medium,
        stress_low_contrast: result.stress_results.low_contrast,
    })
}

/// Get only the scannability score (0-100)
///
/// @param imageBuffer - Raw image bytes (PNG, JPEG, etc.)
/// @returns Score from 0 (unreadable) to 100 (highly scannable)
#[napi]
pub fn validate_score_only(image_buffer: Buffer) -> Result<u8> {
    let result = core_validate(&image_buffer)
        .map_err(|e| Error::from_reason(e.to_string()))?;
    Ok(result.score)
}

/// Get score using fast validation (~2x faster)
///
/// @param imageBuffer - Raw image bytes (PNG, JPEG, etc.)
/// @returns Score from 0 (unreadable) to 100 (highly scannable)
#[napi]
pub fn validate_score_fast(image_buffer: Buffer) -> Result<u8> {
    let result = core_validate_fast(&image_buffer)
        .map_err(|e| Error::from_reason(e.to_string()))?;
    Ok(result.score)
}

fn ec_to_string(ec: ErrorCorrectionLevel) -> String {
    match ec {
        ErrorCorrectionLevel::L => "L".to_string(),
        ErrorCorrectionLevel::M => "M".to_string(),
        ErrorCorrectionLevel::Q => "Q".to_string(),
        ErrorCorrectionLevel::H => "H".to_string(),
    }
}

// ============================================================================
// CONVENIENCE HELPERS - Simple one-liners for common tasks
// ============================================================================

/// Simple summary of QR validation
#[napi(object)]
pub struct QrSummary {
    /// Whether the QR is valid and decodable
    pub valid: bool,
    /// Scannability score (0-100)
    pub score: u8,
    /// Decoded content (empty if invalid)
    pub content: String,
    /// Error correction level (L/M/Q/H or "N/A")
    pub error_correction: String,
    /// Human-readable rating (Excellent/Good/Fair/Poor)
    pub rating: String,
    /// Whether this QR is production-ready (score >= 70)
    pub production_ready: bool,
}

/// Check if QR code is valid (returns content or null)
///
/// @example
/// ```typescript
/// const content = isValid(buffer);
/// if (content) {
///   console.log(`QR contains: ${content}`);
/// }
/// ```
///
/// @param imageBuffer - Raw image bytes (PNG, JPEG, etc.)
/// @returns Decoded content string, or null if QR is invalid
#[napi]
pub fn is_valid(image_buffer: Buffer) -> Option<String> {
    core_decode_only(&image_buffer).ok().map(|r| r.content)
}

/// Get scannability score (0-100)
///
/// @example
/// ```typescript
/// const s = score(buffer);
/// console.log(`Scannability: ${s}/100`);
/// ```
///
/// @param imageBuffer - Raw image bytes (PNG, JPEG, etc.)
/// @returns Score from 0 (unreadable) to 100 (highly scannable)
#[napi]
pub fn score(image_buffer: Buffer) -> u8 {
    core_validate(&image_buffer)
        .map(|r| r.score)
        .unwrap_or(0)
}

/// Check if QR meets minimum score threshold
///
/// @example
/// ```typescript
/// if (passesThreshold(buffer, 70)) {
///   console.log('Production ready!');
/// }
/// ```
///
/// @param imageBuffer - Raw image bytes (PNG, JPEG, etc.)
/// @param minScore - Minimum score required (0-100)
/// @returns true if score >= minScore
#[napi]
pub fn passes_threshold(image_buffer: Buffer, min_score: u8) -> bool {
    score(image_buffer) >= min_score
}

/// Get production readiness (score >= 70)
///
/// @example
/// ```typescript
/// if (isProductionReady(buffer)) {
///   await uploadQr(buffer);
/// }
/// ```
///
/// @param imageBuffer - Raw image bytes (PNG, JPEG, etc.)
/// @returns true if QR is production-ready
#[napi]
pub fn is_production_ready(image_buffer: Buffer) -> bool {
    passes_threshold(image_buffer, 70)
}

/// Get simple summary of QR validation
///
/// @example
/// ```typescript
/// const summary = summarize(buffer);
/// console.log(`${summary.rating}: ${summary.score}/100`);
/// if (summary.productionReady) {
///   console.log(`Content: ${summary.content}`);
/// }
/// ```
///
/// @param imageBuffer - Raw image bytes (PNG, JPEG, etc.)
/// @returns QrSummary with all key info
#[napi]
pub fn summarize(image_buffer: Buffer) -> QrSummary {
    match core_validate(&image_buffer) {
        Ok(result) => {
            let score_val = result.score;
            let rating = match score_val {
                80..=100 => "Excellent",
                60..=79 => "Good",
                40..=59 => "Fair",
                _ => "Poor",
            }
            .to_string();

            QrSummary {
                valid: result.decodable,
                score: score_val,
                content: result.content.unwrap_or_default(),
                error_correction: result
                    .metadata
                    .map(|m| ec_to_string(m.error_correction))
                    .unwrap_or_else(|| "N/A".to_string()),
                rating,
                production_ready: score_val >= 70,
            }
        }
        Err(_) => QrSummary {
            valid: false,
            score: 0,
            content: String::new(),
            error_correction: "N/A".to_string(),
            rating: "Invalid".to_string(),
            production_ready: false,
        },
    }
}

/// Get human-readable rating for a score
///
/// @example
/// ```typescript
/// const rating = getRating(85); // "Excellent"
/// ```
///
/// @param score - Score from 0-100
/// @returns Rating string (Excellent/Good/Fair/Poor)
#[napi]
pub fn get_rating(score: u8) -> String {
    match score {
        80..=100 => "Excellent",
        60..=79 => "Good",
        40..=59 => "Fair",
        _ => "Poor",
    }
    .to_string()
}
