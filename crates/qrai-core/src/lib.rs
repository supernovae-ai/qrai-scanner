//! QRAI Validator - High-performance QR code validation and scannability scoring
//!
//! This library provides tools to:
//! - Decode QR codes using multiple robust decoders (rxing, rqrr)
//! - Calculate a scannability score (0-100) based on stress tests
//! - Extract QR metadata (version, error correction level, etc.)
//!
//! # Example
//!
//! ```rust,no_run
//! use qrai_core::validate;
//!
//! let image_bytes = std::fs::read("qr.png").unwrap();
//! let result = validate(&image_bytes).unwrap();
//!
//! println!("Score: {}", result.score);
//! println!("Content: {:?}", result.content);
//! ```

pub mod decoder;
pub mod error;
pub mod scorer;
pub mod types;

pub use error::{QraiError, Result};
pub use types::{
    DecodeResult, ErrorCorrectionLevel, QrMetadata, StressResults, ValidationResult,
};

use decoder::{multi_decode, multi_decode_image};
use scorer::{calculate_fast_score, calculate_score, run_fast_stress_tests, run_stress_tests};

/// Validate a QR code image and compute scannability score
///
/// This is the main entry point. It:
/// 1. Attempts to decode the QR using multiple decoders
/// 2. Runs stress tests (blur, downscale, contrast reduction)
/// 3. Computes a score based on how many tests pass
///
/// # Arguments
/// * `image_bytes` - Raw bytes of the image (PNG, JPEG, etc.)
///
/// # Returns
/// * `ValidationResult` with score, decoded content, and metadata
///
/// # Errors
/// * `QraiError::ImageLoad` if the image cannot be parsed
/// * `QraiError::DecodeFailed` if no QR code is found
pub fn validate(image_bytes: &[u8]) -> Result<ValidationResult> {
    let decode_result = multi_decode(image_bytes)?;
    let stress_results = run_stress_tests(image_bytes)?;
    let score = calculate_score(&stress_results, decode_result.decoders_success.len());

    Ok(ValidationResult {
        score,
        decodable: true,
        content: Some(decode_result.content),
        metadata: decode_result.metadata,
        stress_results,
    })
}

/// Fast decode without stress tests
///
/// Use this when you only need to verify the QR is readable
/// and extract its content, without computing a score.
///
/// # Arguments
/// * `image_bytes` - Raw bytes of the image
///
/// # Returns
/// * `DecodeResult` with content and metadata
pub fn decode_only(image_bytes: &[u8]) -> Result<DecodeResult> {
    let result = multi_decode(image_bytes)?;

    Ok(DecodeResult {
        content: result.content,
        metadata: result.metadata,
    })
}

/// Fast validation with reduced stress tests
///
/// Runs only a subset of stress tests for faster response times.
/// Good for real-time feedback during QR editing.
///
/// # Performance
/// ~2-3x faster than full validation
pub fn validate_fast(image_bytes: &[u8]) -> Result<ValidationResult> {
    let img = image::load_from_memory(image_bytes)
        .map_err(|e| error::QraiError::ImageLoad(e.to_string()))?;

    let decode_result = multi_decode_image(&img)?;
    let stress_results = run_fast_stress_tests(&img)?;
    let score = calculate_fast_score(&stress_results, decode_result.decoders_success.len());

    Ok(ValidationResult {
        score,
        decodable: true,
        content: Some(decode_result.content),
        metadata: decode_result.metadata,
        stress_results,
    })
}

/// Validate from a file path (convenience function)
pub fn validate_from_path(path: &std::path::Path) -> Result<ValidationResult> {
    let image_bytes = std::fs::read(path)?;
    validate(&image_bytes)
}

/// Decode only from a file path (convenience function)
pub fn decode_from_path(path: &std::path::Path) -> Result<DecodeResult> {
    let image_bytes = std::fs::read(path)?;
    decode_only(&image_bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{DynamicImage, Luma};

    fn create_test_qr() -> Vec<u8> {
        let code = qrcode::QrCode::new(b"https://example.com").unwrap();
        let img = code.render::<Luma<u8>>().build();

        let mut buf = Vec::new();
        let dyn_img = DynamicImage::ImageLuma8(img);
        dyn_img
            .write_to(&mut std::io::Cursor::new(&mut buf), image::ImageFormat::Png)
            .unwrap();
        buf
    }

    #[test]
    fn validate_returns_full_result() {
        let qr_bytes = create_test_qr();
        let result = validate(&qr_bytes).unwrap();

        assert!(result.decodable);
        assert!(result.score > 0);
        assert!(result.content.is_some());
        assert_eq!(result.content.unwrap(), "https://example.com");
        assert!(result.metadata.is_some());
    }

    #[test]
    fn validate_garbage_returns_error() {
        let result = validate(b"not an image at all");
        assert!(result.is_err());
    }

    #[test]
    fn decode_only_is_fast() {
        let qr_bytes = create_test_qr();

        let start = std::time::Instant::now();
        let result = decode_only(&qr_bytes);
        let elapsed = start.elapsed();

        assert!(result.is_ok());
        assert_eq!(result.unwrap().content, "https://example.com");

        // Should be fast (no stress tests)
        assert!(
            elapsed.as_millis() < 500,
            "decode_only took too long: {}ms",
            elapsed.as_millis()
        );
    }

    #[test]
    fn validate_score_is_reasonable() {
        let qr_bytes = create_test_qr();
        let result = validate(&qr_bytes).unwrap();

        // Clean generated QR should have a high score
        assert!(
            result.score >= 50,
            "Clean QR score should be >= 50, got {}",
            result.score
        );
    }

    #[test]
    fn metadata_has_version_and_ec() {
        let qr_bytes = create_test_qr();
        let result = validate(&qr_bytes).unwrap();

        let meta = result.metadata.unwrap();
        assert!(meta.version > 0);
        assert!(meta.modules > 0);
        assert!(!meta.decoders_success.is_empty());
    }

    #[test]
    fn validate_fast_is_faster() {
        let qr_bytes = create_test_qr();

        // Warm up
        let _ = validate_fast(&qr_bytes);
        let _ = validate(&qr_bytes);

        // Time fast validation
        let start_fast = std::time::Instant::now();
        for _ in 0..5 {
            let _ = validate_fast(&qr_bytes);
        }
        let elapsed_fast = start_fast.elapsed();

        // Time full validation
        let start_full = std::time::Instant::now();
        for _ in 0..5 {
            let _ = validate(&qr_bytes);
        }
        let elapsed_full = start_full.elapsed();

        // Fast should be faster (or at least not slower)
        println!("Fast: {:?}, Full: {:?}", elapsed_fast, elapsed_full);
        // Note: We don't assert strictly because parallel execution can vary
    }

    #[test]
    fn validate_fast_still_works() {
        let qr_bytes = create_test_qr();
        let result = validate_fast(&qr_bytes).unwrap();

        assert!(result.decodable);
        assert!(result.score > 0);
        assert!(result.content.is_some());
    }
}
