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
