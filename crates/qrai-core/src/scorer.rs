//! Scannability scoring module
//!
//! Runs stress tests on QR images and computes a score 0-100.

use crate::decoder::multi_decode_image;
use crate::error::{QraiError, Result};
use crate::types::StressResults;
use image::imageops::FilterType;
use image::{DynamicImage, GenericImageView};
use rayon::prelude::*;

/// Weights for each stress test component
const WEIGHT_ORIGINAL: u32 = 20;
const WEIGHT_DOWNSCALE_50: u32 = 15;
const WEIGHT_DOWNSCALE_25: u32 = 10;
const WEIGHT_BLUR_LIGHT: u32 = 15;
const WEIGHT_BLUR_MEDIUM: u32 = 10;
const WEIGHT_LOW_CONTRAST: u32 = 15;
const WEIGHT_MULTI_DECODER: u32 = 15;

const TOTAL_WEIGHT: u32 = WEIGHT_ORIGINAL
    + WEIGHT_DOWNSCALE_50
    + WEIGHT_DOWNSCALE_25
    + WEIGHT_BLUR_LIGHT
    + WEIGHT_BLUR_MEDIUM
    + WEIGHT_LOW_CONTRAST
    + WEIGHT_MULTI_DECODER;

/// Run all stress tests on an image (from bytes)
pub fn run_stress_tests(image_bytes: &[u8]) -> Result<StressResults> {
    let img = image::load_from_memory(image_bytes)
        .map_err(|e| QraiError::ImageLoad(e.to_string()))?;

    run_stress_tests_on_image(&img)
}

/// Run stress tests on an already-loaded image
///
/// Uses parallel execution with rayon for better performance.
pub fn run_stress_tests_on_image(img: &DynamicImage) -> Result<StressResults> {
    // Test original first (most important, fast path)
    let original = test_decode(img);

    // If original fails, no point in running other tests
    if !original {
        return Ok(StressResults {
            original: false,
            downscale_50: false,
            downscale_25: false,
            blur_light: false,
            blur_medium: false,
            low_contrast: false,
        });
    }

    // Prepare all image variants in parallel
    let variants: Vec<(&str, DynamicImage)> = [
        ("downscale_50", downscale(img, 0.5)),
        ("downscale_25", downscale(img, 0.25)),
        ("blur_light", apply_blur(img, 1.0)),
        ("blur_medium", apply_blur(img, 2.0)),
        ("low_contrast", reduce_contrast(img, 0.5)),
    ]
    .into_iter()
    .collect();

    // Test all variants in parallel
    let results: Vec<(&str, bool)> = variants
        .par_iter()
        .map(|(name, variant)| (*name, test_decode(variant)))
        .collect();

    // Collect results
    let mut stress = StressResults {
        original: true,
        downscale_50: false,
        downscale_25: false,
        blur_light: false,
        blur_medium: false,
        low_contrast: false,
    };

    for (name, passed) in results {
        match name {
            "downscale_50" => stress.downscale_50 = passed,
            "downscale_25" => stress.downscale_25 = passed,
            "blur_light" => stress.blur_light = passed,
            "blur_medium" => stress.blur_medium = passed,
            "low_contrast" => stress.low_contrast = passed,
            _ => {}
        }
    }

    Ok(stress)
}

/// Fast stress tests - only run a subset for quick validation
pub fn run_fast_stress_tests(img: &DynamicImage) -> Result<StressResults> {
    let original = test_decode(img);

    if !original {
        return Ok(StressResults::default());
    }

    // Only test downscale_50 and blur_light for fast mode
    let downscale_50 = test_decode(&downscale(img, 0.5));
    let blur_light = test_decode(&apply_blur(img, 1.0));

    Ok(StressResults {
        original: true,
        downscale_50,
        downscale_25: false, // Skip
        blur_light,
        blur_medium: false, // Skip
        low_contrast: false, // Skip
    })
}

/// Calculate score from stress test results
pub fn calculate_score(stress: &StressResults, num_decoders: usize) -> u8 {
    let mut score: u32 = 0;

    if stress.original {
        score += WEIGHT_ORIGINAL;
    }
    if stress.downscale_50 {
        score += WEIGHT_DOWNSCALE_50;
    }
    if stress.downscale_25 {
        score += WEIGHT_DOWNSCALE_25;
    }
    if stress.blur_light {
        score += WEIGHT_BLUR_LIGHT;
    }
    if stress.blur_medium {
        score += WEIGHT_BLUR_MEDIUM;
    }
    if stress.low_contrast {
        score += WEIGHT_LOW_CONTRAST;
    }

    // Bonus for multiple decoders succeeding
    if num_decoders >= 2 {
        score += WEIGHT_MULTI_DECODER;
    }

    // Normalize to 0-100
    ((score * 100) / TOTAL_WEIGHT).min(100) as u8
}

/// Calculate score for fast mode (adjusted weights)
pub fn calculate_fast_score(stress: &StressResults, num_decoders: usize) -> u8 {
    // Fast mode only uses original, downscale_50, blur_light
    let fast_total = WEIGHT_ORIGINAL + WEIGHT_DOWNSCALE_50 + WEIGHT_BLUR_LIGHT + WEIGHT_MULTI_DECODER;
    let mut score: u32 = 0;

    if stress.original {
        score += WEIGHT_ORIGINAL;
    }
    if stress.downscale_50 {
        score += WEIGHT_DOWNSCALE_50;
    }
    if stress.blur_light {
        score += WEIGHT_BLUR_LIGHT;
    }
    if num_decoders >= 2 {
        score += WEIGHT_MULTI_DECODER;
    }

    ((score * 100) / fast_total).min(100) as u8
}

/// Test if an image variant can be decoded
#[inline]
fn test_decode(img: &DynamicImage) -> bool {
    multi_decode_image(img).is_ok()
}

/// Downscale image by a factor (0.5 = half size)
/// Uses Triangle filter for speed (vs Lanczos3 for quality)
#[inline]
fn downscale(img: &DynamicImage, factor: f32) -> DynamicImage {
    let (w, h) = img.dimensions();
    let new_w = ((w as f32) * factor).max(1.0) as u32;
    let new_h = ((h as f32) * factor).max(1.0) as u32;
    // Triangle is faster than Lanczos3, good enough for stress tests
    img.resize(new_w, new_h, FilterType::Triangle)
}

/// Apply Gaussian blur with given sigma
#[inline]
fn apply_blur(img: &DynamicImage, sigma: f32) -> DynamicImage {
    img.blur(sigma)
}

/// Reduce contrast by given factor (0.5 = 50% contrast)
#[inline]
fn reduce_contrast(img: &DynamicImage, factor: f32) -> DynamicImage {
    // Negative value reduces contrast
    img.adjust_contrast((1.0 - factor) * -50.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_qr() -> Vec<u8> {
        use image::Luma;

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
    fn score_all_pass_is_100() {
        let stress = StressResults {
            original: true,
            downscale_50: true,
            downscale_25: true,
            blur_light: true,
            blur_medium: true,
            low_contrast: true,
        };
        let score = calculate_score(&stress, 2);
        assert_eq!(score, 100);
    }

    #[test]
    fn score_all_fail_is_zero() {
        let stress = StressResults {
            original: false,
            downscale_50: false,
            downscale_25: false,
            blur_light: false,
            blur_medium: false,
            low_contrast: false,
        };
        let score = calculate_score(&stress, 0);
        assert_eq!(score, 0);
    }

    #[test]
    fn score_only_original_is_low() {
        let stress = StressResults {
            original: true,
            downscale_50: false,
            downscale_25: false,
            blur_light: false,
            blur_medium: false,
            low_contrast: false,
        };
        let score = calculate_score(&stress, 1);
        assert!(score < 25);
        assert!(score > 15);
    }

    #[test]
    fn score_without_multi_decoder_bonus() {
        let stress = StressResults {
            original: true,
            downscale_50: true,
            downscale_25: true,
            blur_light: true,
            blur_medium: true,
            low_contrast: true,
        };
        let score = calculate_score(&stress, 1);
        assert!(score > 80);
        assert!(score < 100);
    }

    #[test]
    fn stress_test_clean_qr_passes_most() {
        let qr_bytes = create_test_qr();
        let img = image::load_from_memory(&qr_bytes).unwrap();
        let stress = run_stress_tests_on_image(&img).unwrap();

        assert!(stress.original);
        assert!(stress.downscale_50);
        assert!(stress.blur_light);
    }

    #[test]
    fn fast_stress_test_runs_subset() {
        let qr_bytes = create_test_qr();
        let img = image::load_from_memory(&qr_bytes).unwrap();
        let stress = run_fast_stress_tests(&img).unwrap();

        assert!(stress.original);
        // Fast mode skips some tests
        assert!(!stress.downscale_25);
        assert!(!stress.blur_medium);
        assert!(!stress.low_contrast);
    }

    #[test]
    fn downscale_reduces_dimensions() {
        let qr_bytes = create_test_qr();
        let img = image::load_from_memory(&qr_bytes).unwrap();
        let (orig_w, orig_h) = img.dimensions();

        let scaled = downscale(&img, 0.5);
        let (new_w, new_h) = scaled.dimensions();

        assert!(new_w < orig_w);
        assert!(new_h < orig_h);
        assert_eq!(new_w, orig_w / 2);
        assert_eq!(new_h, orig_h / 2);
    }

    #[test]
    fn parallel_stress_tests_consistent() {
        let qr_bytes = create_test_qr();
        let img = image::load_from_memory(&qr_bytes).unwrap();

        // Run multiple times to ensure parallel execution is deterministic
        let stress1 = run_stress_tests_on_image(&img).unwrap();
        let stress2 = run_stress_tests_on_image(&img).unwrap();

        assert_eq!(stress1.original, stress2.original);
        assert_eq!(stress1.downscale_50, stress2.downscale_50);
        assert_eq!(stress1.blur_light, stress2.blur_light);
    }
}
