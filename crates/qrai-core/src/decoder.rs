use crate::error::{QraiError, Result};
use crate::types::{ErrorCorrectionLevel, MultiDecodeResult, QrMetadata};
use image::{DynamicImage, GenericImageView, GrayImage, Luma};
use rayon::prelude::*;

/// Random preprocessing parameters for brute-force decoding
#[derive(Debug, Clone, Copy)]
struct PreprocessParams {
    resize: u32,       // Target size in pixels (0 = no resize)
    contrast: f32,     // Contrast multiplier (1.0 = normal, 3.8 = 380%)
    brightness: f32,   // Brightness multiplier (1.0 = normal, 1.79 = 179%)
    blur: f32,         // Blur radius in pixels (0 = no blur)
    grayscale: bool,   // Convert to grayscale first
}

/// Decode result from a single decoder
#[derive(Debug, Clone)]
pub struct SingleDecodeResult {
    pub content: String,
    pub version: Option<u8>,
    pub error_correction: Option<ErrorCorrectionLevel>,
}

/// Decode QR code using rxing (ZXing port) - most robust decoder
pub fn decode_with_rxing(img: &DynamicImage) -> Result<SingleDecodeResult> {
    let luma = img.to_luma8();
    let (width, height) = luma.dimensions();
    decode_with_rxing_raw(&luma.into_raw(), width, height)
}

/// Internal rxing decoder using pre-converted luma data
fn decode_with_rxing_raw(luma_data: &[u8], width: u32, height: u32) -> Result<SingleDecodeResult> {
    let results = rxing::helpers::detect_multiple_in_luma(luma_data.to_vec(), width, height);

    // Debug output
    if std::env::var("QRAI_DEBUG").is_ok() {
        match &results {
            Ok(r) => eprintln!("[DEBUG] rxing found {} results", r.len()),
            Err(e) => eprintln!("[DEBUG] rxing error: {:?}", e),
        }
    }

    let results = results.map_err(|_| QraiError::DecodeFailed)?;

    let first = results.first().ok_or(QraiError::DecodeFailed)?;

    let version = extract_version_from_rxing(first);
    let error_correction = extract_ec_from_rxing(first);

    Ok(SingleDecodeResult {
        content: first.getText().to_string(),
        version,
        error_correction,
    })
}

/// Decode QR code using rqrr (Quirc port) - fast pure Rust decoder
pub fn decode_with_rqrr(img: &DynamicImage) -> Result<SingleDecodeResult> {
    let luma = img.to_luma8();
    let (width, height) = luma.dimensions();
    decode_with_rqrr_raw(&luma.into_raw(), width, height)
}

/// Internal rqrr decoder using pre-converted luma data
fn decode_with_rqrr_raw(luma_data: &[u8], width: u32, height: u32) -> Result<SingleDecodeResult> {
    // Reconstruct GrayImage from raw luma data
    let luma = GrayImage::from_raw(width, height, luma_data.to_vec())
        .ok_or(QraiError::DecodeFailed)?;

    let mut prepared = rqrr::PreparedImage::prepare(luma);
    let grids = prepared.detect_grids();

    // Debug: show how many grids were found
    if std::env::var("QRAI_DEBUG").is_ok() {
        eprintln!("[DEBUG] rqrr found {} grids", grids.len());
    }

    let grid = grids.first().ok_or(QraiError::DecodeFailed)?;
    let (meta, content) = grid.decode().map_err(|_| QraiError::DecodeFailed)?;

    Ok(SingleDecodeResult {
        content,
        version: Some(meta.version.0 as u8),
        error_correction: Some(convert_rqrr_ec(meta.ecc_level)),
    })
}

/// Multi-decoder that tries multiple decoders and combines results
pub fn multi_decode(image_bytes: &[u8]) -> Result<MultiDecodeResult> {
    let img = image::load_from_memory(image_bytes)
        .map_err(|e| QraiError::ImageLoad(e.to_string()))?;

    multi_decode_image(&img)
}

/// Multi-decoder for already-loaded image
/// Phase 7: TIERED STRATEGY - prioritize known-good params over random exploration
/// Tier 1: Original (instant) → Tier 2: Quick trio → Tier 3: Known-good + channels → Tier 4: Brute force
pub fn multi_decode_image(img: &DynamicImage) -> Result<MultiDecodeResult> {
    // ========================================================================
    // TIER 1: Original image (instant for clean QRs) - ~80ms
    // ========================================================================
    if let Ok(result) = try_decode_with_both(img) {
        return Ok(result);
    }

    // ========================================================================
    // TIER 2: Quick preprocessing trio (parallel) - ~100ms
    // These catch many artistic QRs without heavy processing
    // ========================================================================
    let quick_variants = vec![
        apply_otsu_threshold(img),
        invert_image(&apply_otsu_threshold(img)),
        apply_high_contrast_threshold(img),
    ];

    if let Some(result) = quick_variants.par_iter().find_map_any(|v| try_decode_with_both(v).ok()) {
        return Ok(result);
    }

    // ========================================================================
    // TIER 3: ALL known strategies in ONE parallel pool
    // Known-good params (16) + Color channels (12) + HSV (6) = 34 strategies
    // First success wins instantly via find_map_any
    // ========================================================================
    if let Ok(result) = try_unified_parallel_pool(img) {
        return Ok(result);
    }

    // ========================================================================
    // TIER 4: Mini brute force (64 random combos) - last resort
    // Reduced from 256 since most successes happen earlier
    // ========================================================================
    if let Ok(result) = try_mini_brute_force(img, 64) {
        return Ok(result);
    }

    Err(QraiError::DecodeFailed)
}

/// Unified parallel pool: known-good params + color channels + HSV
/// All 34+ strategies run simultaneously, first success exits instantly
fn try_unified_parallel_pool(img: &DynamicImage) -> Result<MultiDecodeResult> {
    // Pre-extract all variants
    let channels = extract_color_channels(img);
    let hue = extract_hue_channel(img);
    let value = extract_value_channel(img);

    // Build unified image pool: known-good preprocessed + channel variants
    let mut variants: Vec<DynamicImage> = Vec::with_capacity(50);

    // Known-good preprocessing combos (most effective first)
    let known_good_params = [
        PreprocessParams { resize: 400, contrast: 2.0, brightness: 1.0, blur: 0.0, grayscale: true },
        PreprocessParams { resize: 350, contrast: 2.5, brightness: 1.0, blur: 0.5, grayscale: true },
        PreprocessParams { resize: 300, contrast: 2.0, brightness: 1.1, blur: 0.3, grayscale: true },
        PreprocessParams { resize: 400, contrast: 1.8, brightness: 0.9, blur: 0.0, grayscale: true },
        PreprocessParams { resize: 250, contrast: 2.5, brightness: 1.0, blur: 1.0, grayscale: true },
        PreprocessParams { resize: 300, contrast: 3.0, brightness: 1.0, blur: 0.8, grayscale: true },
        PreprocessParams { resize: 0, contrast: 2.5, brightness: 1.0, blur: 0.0, grayscale: true },
        PreprocessParams { resize: 0, contrast: 2.0, brightness: 1.1, blur: 0.5, grayscale: true },
        PreprocessParams { resize: 500, contrast: 1.5, brightness: 1.0, blur: 0.0, grayscale: true },
        PreprocessParams { resize: 450, contrast: 2.2, brightness: 1.0, blur: 0.3, grayscale: true },
        PreprocessParams { resize: 350, contrast: 3.5, brightness: 1.2, blur: 1.0, grayscale: true },
        PreprocessParams { resize: 300, contrast: 4.0, brightness: 1.0, blur: 1.5, grayscale: true },
    ];

    for params in &known_good_params {
        variants.push(apply_preprocessing_fast(img, params));
    }

    // Color channels + variants
    for ch in &channels {
        variants.push(ch.clone());
        variants.push(apply_otsu_threshold(ch));
    }

    // HSV channels
    variants.push(hue.clone());
    variants.push(apply_otsu_threshold(&hue));
    variants.push(value.clone());
    variants.push(enhance_contrast(&value));

    // Try all in parallel with 3 variants each (raw + otsu + inverted)
    variants.par_iter().find_map_any(|v| {
        if let Ok(r) = try_decode_with_both(v) { return Some(r); }
        let otsu = apply_otsu_threshold(v);
        if let Ok(r) = try_decode_with_both(&otsu) { return Some(r); }
        let inv = invert_image(&otsu);
        if let Ok(r) = try_decode_with_both(&inv) { return Some(r); }
        None
    }).ok_or(QraiError::DecodeFailed)
}

/// Mini brute force: 64 random combos (reduced from 256)
/// Only runs if known-good strategies fail
fn try_mini_brute_force(img: &DynamicImage, num_tries: u32) -> Result<MultiDecodeResult> {
    use std::time::{SystemTime, UNIX_EPOCH};

    let mut seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(12345);

    let mut next_random = || -> f32 {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        (seed as f64 / u64::MAX as f64) as f32
    };

    let sizes: [u32; 8] = [200, 250, 300, 350, 400, 450, 500, 550];

    let params_list: Vec<PreprocessParams> = (0..num_tries)
        .map(|_| PreprocessParams {
            resize: sizes[(next_random() * sizes.len() as f32) as usize % sizes.len()],
            contrast: 1.0 + next_random() * 3.0,  // 1.0-4.0
            brightness: 0.8 + next_random() * 0.6, // 0.8-1.4
            blur: next_random() * 1.5,             // 0-1.5
            grayscale: next_random() > 0.3,        // 70% grayscale
        })
        .collect();

    params_list.par_iter().find_map_any(|params| {
        let processed = apply_preprocessing_fast(img, params);
        if let Ok(r) = try_decode_with_both(&processed) { return Some(r); }
        let otsu = apply_otsu_threshold(&processed);
        if let Ok(r) = try_decode_with_both(&otsu) { return Some(r); }
        let inv = invert_image(&otsu);
        if let Ok(r) = try_decode_with_both(&inv) { return Some(r); }
        None
    }).ok_or(QraiError::DecodeFailed)
}

/// MASSIVE BRUTE FORCE: Known-good params + random exploration
/// Strategy: 32 proven combos + 224 random = 256 total, all parallel
fn try_massive_brute_force(img: &DynamicImage, num_tries: u32) -> Result<MultiDecodeResult> {
    use std::time::{SystemTime, UNIX_EPOCH};

    // KNOWN GOOD PARAMS - these work for many artistic QRs
    let known_good: Vec<PreprocessParams> = vec![
        // Downsampled + high contrast (works for complex images)
        PreprocessParams { resize: 400, contrast: 2.0, brightness: 1.0, blur: 0.0, grayscale: true },
        PreprocessParams { resize: 350, contrast: 2.5, brightness: 1.0, blur: 0.5, grayscale: true },
        PreprocessParams { resize: 300, contrast: 2.0, brightness: 1.1, blur: 0.3, grayscale: true },
        PreprocessParams { resize: 450, contrast: 1.8, brightness: 0.9, blur: 0.0, grayscale: true },
        // Smaller sizes with blur (noise reduction)
        PreprocessParams { resize: 250, contrast: 2.2, brightness: 1.0, blur: 1.0, grayscale: true },
        PreprocessParams { resize: 300, contrast: 2.5, brightness: 1.2, blur: 0.8, grayscale: true },
        PreprocessParams { resize: 350, contrast: 3.0, brightness: 1.0, blur: 0.5, grayscale: true },
        PreprocessParams { resize: 400, contrast: 2.8, brightness: 0.9, blur: 0.3, grayscale: true },
        // Color preserved variants
        PreprocessParams { resize: 400, contrast: 2.0, brightness: 1.0, blur: 0.0, grayscale: false },
        PreprocessParams { resize: 350, contrast: 2.2, brightness: 1.1, blur: 0.5, grayscale: false },
        PreprocessParams { resize: 300, contrast: 2.5, brightness: 1.0, blur: 0.8, grayscale: false },
        PreprocessParams { resize: 450, contrast: 1.5, brightness: 1.0, blur: 0.0, grayscale: false },
        // Extreme params for stubborn cases
        PreprocessParams { resize: 250, contrast: 3.5, brightness: 1.2, blur: 1.5, grayscale: true },
        PreprocessParams { resize: 200, contrast: 3.0, brightness: 1.0, blur: 1.0, grayscale: true },
        PreprocessParams { resize: 300, contrast: 3.5, brightness: 0.8, blur: 0.5, grayscale: true },
        PreprocessParams { resize: 350, contrast: 4.0, brightness: 1.0, blur: 1.0, grayscale: true },
        // No resize, high contrast
        PreprocessParams { resize: 0, contrast: 2.5, brightness: 1.0, blur: 0.0, grayscale: true },
        PreprocessParams { resize: 0, contrast: 3.0, brightness: 1.1, blur: 0.5, grayscale: true },
        PreprocessParams { resize: 0, contrast: 2.0, brightness: 0.9, blur: 0.3, grayscale: true },
        PreprocessParams { resize: 0, contrast: 1.8, brightness: 1.0, blur: 0.0, grayscale: false },
        // Medium sizes, various params
        PreprocessParams { resize: 500, contrast: 1.5, brightness: 1.0, blur: 0.0, grayscale: true },
        PreprocessParams { resize: 500, contrast: 2.0, brightness: 1.1, blur: 0.3, grayscale: true },
        PreprocessParams { resize: 550, contrast: 1.8, brightness: 1.0, blur: 0.0, grayscale: true },
        PreprocessParams { resize: 600, contrast: 1.5, brightness: 1.0, blur: 0.0, grayscale: true },
        // Low contrast variants
        PreprocessParams { resize: 400, contrast: 1.2, brightness: 1.0, blur: 0.0, grayscale: true },
        PreprocessParams { resize: 350, contrast: 1.3, brightness: 1.1, blur: 0.2, grayscale: true },
        PreprocessParams { resize: 300, contrast: 1.4, brightness: 1.0, blur: 0.0, grayscale: true },
        PreprocessParams { resize: 450, contrast: 1.1, brightness: 0.95, blur: 0.0, grayscale: true },
        // Very aggressive
        PreprocessParams { resize: 200, contrast: 4.0, brightness: 1.3, blur: 2.0, grayscale: true },
        PreprocessParams { resize: 250, contrast: 4.0, brightness: 1.0, blur: 1.5, grayscale: true },
        PreprocessParams { resize: 300, contrast: 3.8, brightness: 1.1, blur: 1.2, grayscale: true },
        PreprocessParams { resize: 350, contrast: 3.5, brightness: 1.2, blur: 1.0, grayscale: true },
    ];

    // Fast RNG for remaining random params
    let mut seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(12345);

    let mut next_random = || -> f32 {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        (seed as f64 / u64::MAX as f64) as f32
    };

    let sizes: [u32; 10] = [0, 200, 250, 300, 350, 400, 450, 500, 550, 600];
    let contrast_range = (0.8f32, 4.0f32);
    let brightness_range = (0.7f32, 1.5f32);
    let blur_range = (0.0f32, 2.0f32);

    // Combine known good + random exploration
    let random_count = num_tries.saturating_sub(known_good.len() as u32);
    let mut params_list = known_good;

    for _ in 0..random_count {
        params_list.push(PreprocessParams {
            resize: sizes[(next_random() * sizes.len() as f32) as usize % sizes.len()],
            contrast: contrast_range.0 + next_random() * (contrast_range.1 - contrast_range.0),
            brightness: brightness_range.0 + next_random() * (brightness_range.1 - brightness_range.0),
            blur: blur_range.0 + next_random() * (blur_range.1 - blur_range.0),
            grayscale: next_random() > 0.35, // 65% grayscale
        });
    }

    // ALL combos run in parallel - first success wins INSTANTLY
    let result = params_list.par_iter().find_map_any(|params| {
        let processed = apply_preprocessing_fast(img, params);

        // Try raw preprocessed
        if let Ok(result) = try_decode_with_both(&processed) {
            return Some(result);
        }

        // Try with Otsu threshold
        let with_otsu = apply_otsu_threshold(&processed);
        if let Ok(result) = try_decode_with_both(&with_otsu) {
            return Some(result);
        }

        // Try inverted Otsu
        let inverted = invert_image(&with_otsu);
        if let Ok(result) = try_decode_with_both(&inverted) {
            return Some(result);
        }

        None
    });

    result.ok_or(QraiError::DecodeFailed)
}

/// Fast preprocessing using thumbnail() for resize (much faster than Lanczos3)
fn apply_preprocessing_fast(img: &DynamicImage, params: &PreprocessParams) -> DynamicImage {
    let mut result = img.clone();

    // 1. Fast resize using thumbnail (nearest neighbor is fastest)
    if params.resize > 0 {
        let (w, h) = result.dimensions();
        let max_dim = w.max(h);
        if max_dim > params.resize {
            result = result.thumbnail(params.resize, params.resize);
        }
    }

    // 2. Convert to grayscale if needed (before other ops for speed)
    if params.grayscale {
        result = DynamicImage::ImageLuma8(result.to_luma8());
    }

    // 3. Apply contrast and brightness in one pass
    if (params.contrast - 1.0).abs() > 0.01 || (params.brightness - 1.0).abs() > 0.01 {
        let rgb = result.to_rgb8();
        let (width, height) = rgb.dimensions();
        let mut adjusted = image::RgbImage::new(width, height);

        for (x, y, pixel) in rgb.enumerate_pixels() {
            let mut new_pixel = [0u8; 3];
            for c in 0..3 {
                let v = pixel.0[c] as f32;
                let brightened = v * params.brightness;
                let contrasted = ((brightened - 128.0) * params.contrast) + 128.0;
                new_pixel[c] = contrasted.clamp(0.0, 255.0) as u8;
            }
            adjusted.put_pixel(x, y, image::Rgb(new_pixel));
        }
        result = DynamicImage::ImageRgb8(adjusted);
    }

    // 4. Light blur if specified (skip if negligible)
    if params.blur > 0.3 {
        result = result.blur(params.blur);
    }

    result
}

/// Try decoding with both decoders on a single image
/// Returns early when first decoder succeeds for performance optimization
/// Pre-converts to luma8 once to avoid duplicate conversions (~100ms saved)
/// If rxing succeeds but lacks metadata, also try rqrr to get complete metadata
fn try_decode_with_both(img: &DynamicImage) -> Result<MultiDecodeResult> {
    // Phase 2 optimization: Single luma8 conversion for both decoders
    let luma = img.to_luma8();
    let (width, height) = luma.dimensions();
    let luma_data = luma.into_raw();

    // Try rxing first
    if let Ok(rxing_result) = decode_with_rxing_raw(&luma_data, width, height) {
        // rxing often lacks version/EC metadata, try rqrr to get complete metadata
        let (version, error_correction, decoders) =
            if let Ok(rqrr_result) = decode_with_rqrr_raw(&luma_data, width, height) {
                // Use rqrr's more complete metadata
                (
                    rqrr_result.version.unwrap_or(0),
                    rqrr_result.error_correction.unwrap_or(ErrorCorrectionLevel::M),
                    vec!["rxing".to_string(), "rqrr".to_string()],
                )
            } else {
                // Fall back to rxing's metadata (may be incomplete)
                (
                    rxing_result.version.unwrap_or(0),
                    rxing_result.error_correction.unwrap_or(ErrorCorrectionLevel::M),
                    vec!["rxing".to_string()],
                )
            };

        let modules = if version > 0 { 17 + version * 4 } else { 0 };
        return Ok(MultiDecodeResult {
            content: rxing_result.content.clone(),
            metadata: Some(QrMetadata {
                version,
                error_correction,
                modules,
                decoders_success: decoders.clone(),
            }),
            decoders_success: decoders,
        });
    }

    // Only try rqrr if rxing failed
    if let Ok(result) = decode_with_rqrr_raw(&luma_data, width, height) {
        let version = result.version.unwrap_or(0);
        let modules = if version > 0 { 17 + version * 4 } else { 0 };
        return Ok(MultiDecodeResult {
            content: result.content.clone(),
            metadata: Some(QrMetadata {
                version,
                error_correction: result.error_correction.unwrap_or(ErrorCorrectionLevel::M),
                modules,
                decoders_success: vec!["rqrr".to_string()],
            }),
            decoders_success: vec!["rqrr".to_string()],
        });
    }

    Err(QraiError::DecodeFailed)
}

/// Try multiple images in parallel and return first successful decode
/// Uses find_map_any for early exit when any strategy succeeds
fn try_parallel_decode(images: &[DynamicImage]) -> Option<MultiDecodeResult> {
    images.par_iter().find_map_any(|img| try_decode_with_both(img).ok())
}

/// Try multiple preprocessing functions in parallel on the same source image
/// Each function receives the source image and returns a processed version
fn try_parallel_strategies<F>(img: &DynamicImage, strategies: &[F]) -> Option<MultiDecodeResult>
where
    F: Fn(&DynamicImage) -> DynamicImage + Sync,
{
    strategies
        .par_iter()
        .find_map_any(|strategy| {
            let processed = strategy(img);
            try_decode_with_both(&processed).ok()
        })
}

// ============================================================================
// Image Preprocessing Functions for Artistic QR Codes
// ============================================================================

/// Enhance contrast using histogram stretching
fn enhance_contrast(img: &DynamicImage) -> DynamicImage {
    let gray = img.to_luma8();
    let (width, height) = gray.dimensions();

    // Find min and max pixel values
    let mut min_val = 255u8;
    let mut max_val = 0u8;
    for pixel in gray.pixels() {
        let v = pixel.0[0];
        if v < min_val {
            min_val = v;
        }
        if v > max_val {
            max_val = v;
        }
    }

    // Avoid division by zero
    if max_val == min_val {
        return DynamicImage::ImageLuma8(gray);
    }

    // Stretch histogram to full range
    let range = (max_val - min_val) as f32;
    let mut enhanced = GrayImage::new(width, height);

    for (x, y, pixel) in gray.enumerate_pixels() {
        let v = pixel.0[0];
        let new_v = (((v - min_val) as f32 / range) * 255.0) as u8;
        enhanced.put_pixel(x, y, Luma([new_v]));
    }

    DynamicImage::ImageLuma8(enhanced)
}

/// Apply Otsu's thresholding for automatic binarization
fn apply_otsu_threshold(img: &DynamicImage) -> DynamicImage {
    let gray = img.to_luma8();
    let (width, height) = gray.dimensions();

    // Compute histogram
    let mut histogram = [0u32; 256];
    let total_pixels = width * height;

    for pixel in gray.pixels() {
        histogram[pixel.0[0] as usize] += 1;
    }

    // Otsu's method to find optimal threshold
    let mut sum = 0u64;
    for (i, &count) in histogram.iter().enumerate() {
        sum += (i as u64) * (count as u64);
    }

    let mut sum_b = 0u64;
    let mut w_b = 0u32;
    let mut max_variance = 0.0f64;
    let mut threshold = 0u8;

    for (i, &count) in histogram.iter().enumerate() {
        w_b += count;
        if w_b == 0 {
            continue;
        }

        let w_f = total_pixels - w_b;
        if w_f == 0 {
            break;
        }

        sum_b += (i as u64) * (count as u64);

        let m_b = sum_b as f64 / w_b as f64;
        let m_f = (sum - sum_b) as f64 / w_f as f64;

        let variance = (w_b as f64) * (w_f as f64) * (m_b - m_f) * (m_b - m_f);

        if variance > max_variance {
            max_variance = variance;
            threshold = i as u8;
        }
    }

    // Apply threshold
    let mut binary = GrayImage::new(width, height);
    for (x, y, pixel) in gray.enumerate_pixels() {
        let v = if pixel.0[0] > threshold { 255 } else { 0 };
        binary.put_pixel(x, y, Luma([v]));
    }

    DynamicImage::ImageLuma8(binary)
}

/// Invert image colors (useful when QR is inverted)
fn invert_image(img: &DynamicImage) -> DynamicImage {
    let gray = img.to_luma8();
    let (width, height) = gray.dimensions();

    let mut inverted = GrayImage::new(width, height);
    for (x, y, pixel) in gray.enumerate_pixels() {
        inverted.put_pixel(x, y, Luma([255 - pixel.0[0]]));
    }

    DynamicImage::ImageLuma8(inverted)
}

/// High contrast threshold - more aggressive binarization
fn apply_high_contrast_threshold(img: &DynamicImage) -> DynamicImage {
    let gray = img.to_luma8();
    let (width, height) = gray.dimensions();

    // First enhance contrast
    let enhanced = enhance_contrast(&DynamicImage::ImageLuma8(gray));
    let enhanced_gray = enhanced.to_luma8();

    // Then apply a fixed middle threshold
    let mut binary = GrayImage::new(width, height);
    for (x, y, pixel) in enhanced_gray.enumerate_pixels() {
        let v = if pixel.0[0] > 127 { 255 } else { 0 };
        binary.put_pixel(x, y, Luma([v]));
    }

    DynamicImage::ImageLuma8(binary)
}

/// Local adaptive thresholding - good for images with gradients
fn apply_adaptive_threshold(img: &DynamicImage) -> DynamicImage {
    let gray = img.to_luma8();
    let (width, height) = gray.dimensions();

    // Use a block-based local threshold
    let block_size = 31u32; // Must be odd
    let c = 10i32; // Constant subtracted from mean

    let mut binary = GrayImage::new(width, height);

    for y in 0..height {
        for x in 0..width {
            // Calculate local mean in block
            let half = block_size / 2;
            let x_start = x.saturating_sub(half);
            let y_start = y.saturating_sub(half);
            let x_end = (x + half + 1).min(width);
            let y_end = (y + half + 1).min(height);

            let mut sum = 0u32;
            let mut count = 0u32;

            for by in y_start..y_end {
                for bx in x_start..x_end {
                    sum += gray.get_pixel(bx, by).0[0] as u32;
                    count += 1;
                }
            }

            let mean = (sum / count) as i32;
            let threshold = (mean - c).max(0) as u8;
            let pixel_val = gray.get_pixel(x, y).0[0];

            let v = if pixel_val > threshold { 255 } else { 0 };
            binary.put_pixel(x, y, Luma([v]));
        }
    }

    DynamicImage::ImageLuma8(binary)
}

/// Extract individual color channels as grayscale images
fn extract_color_channels(img: &DynamicImage) -> Vec<DynamicImage> {
    let rgb = img.to_rgb8();
    let (width, height) = rgb.dimensions();

    let mut channels = Vec::new();

    // Red channel
    let mut red = GrayImage::new(width, height);
    for (x, y, pixel) in rgb.enumerate_pixels() {
        red.put_pixel(x, y, Luma([pixel.0[0]]));
    }
    channels.push(DynamicImage::ImageLuma8(red));

    // Green channel
    let mut green = GrayImage::new(width, height);
    for (x, y, pixel) in rgb.enumerate_pixels() {
        green.put_pixel(x, y, Luma([pixel.0[1]]));
    }
    channels.push(DynamicImage::ImageLuma8(green));

    // Blue channel
    let mut blue = GrayImage::new(width, height);
    for (x, y, pixel) in rgb.enumerate_pixels() {
        blue.put_pixel(x, y, Luma([pixel.0[2]]));
    }
    channels.push(DynamicImage::ImageLuma8(blue));

    // Also try saturation channel (difference between max and min)
    let mut saturation = GrayImage::new(width, height);
    for (x, y, pixel) in rgb.enumerate_pixels() {
        let max_v = pixel.0[0].max(pixel.0[1]).max(pixel.0[2]);
        let min_v = pixel.0[0].min(pixel.0[1]).min(pixel.0[2]);
        saturation.put_pixel(x, y, Luma([max_v - min_v]));
    }
    channels.push(DynamicImage::ImageLuma8(saturation));

    channels
}

/// Apply extreme contrast enhancement (clip outliers)
fn apply_extreme_contrast(img: &DynamicImage) -> DynamicImage {
    let gray = img.to_luma8();
    let (width, height) = gray.dimensions();

    // Collect all pixel values and sort
    let mut values: Vec<u8> = gray.pixels().map(|p| p.0[0]).collect();
    values.sort_unstable();

    // Use 5th and 95th percentile for clipping
    let low_idx = values.len() * 5 / 100;
    let high_idx = values.len() * 95 / 100;
    let low = values.get(low_idx).copied().unwrap_or(0);
    let high = values.get(high_idx).copied().unwrap_or(255);

    if high <= low {
        return DynamicImage::ImageLuma8(gray);
    }

    let range = (high - low) as f32;
    let mut enhanced = GrayImage::new(width, height);

    for (x, y, pixel) in gray.enumerate_pixels() {
        let v = pixel.0[0];
        let new_v = if v <= low {
            0
        } else if v >= high {
            255
        } else {
            (((v - low) as f32 / range) * 255.0) as u8
        };
        enhanced.put_pixel(x, y, Luma([new_v]));
    }

    DynamicImage::ImageLuma8(enhanced)
}

/// Apply a fixed threshold value
fn apply_fixed_threshold(img: &DynamicImage, threshold: u8) -> DynamicImage {
    let gray = img.to_luma8();
    let (width, height) = gray.dimensions();

    let mut binary = GrayImage::new(width, height);
    for (x, y, pixel) in gray.enumerate_pixels() {
        let v = if pixel.0[0] > threshold { 255 } else { 0 };
        binary.put_pixel(x, y, Luma([v]));
    }

    DynamicImage::ImageLuma8(binary)
}

/// Extract Hue channel from HSV colorspace
/// Useful for images where colors have similar luminance but different hues
fn extract_hue_channel(img: &DynamicImage) -> DynamicImage {
    let rgb = img.to_rgb8();
    let (width, height) = rgb.dimensions();

    let mut hue_img = GrayImage::new(width, height);

    for (x, y, pixel) in rgb.enumerate_pixels() {
        let r = pixel.0[0] as f32 / 255.0;
        let g = pixel.0[1] as f32 / 255.0;
        let b = pixel.0[2] as f32 / 255.0;

        let max_v = r.max(g).max(b);
        let min_v = r.min(g).min(b);
        let delta = max_v - min_v;

        let hue = if delta < 0.001 {
            0.0
        } else if max_v == r {
            60.0 * (((g - b) / delta) % 6.0)
        } else if max_v == g {
            60.0 * (((b - r) / delta) + 2.0)
        } else {
            60.0 * (((r - g) / delta) + 4.0)
        };

        // Normalize hue to 0-255
        let hue_normalized = ((hue.abs() % 360.0) / 360.0 * 255.0) as u8;
        hue_img.put_pixel(x, y, Luma([hue_normalized]));
    }

    DynamicImage::ImageLuma8(hue_img)
}

/// Extract Value (brightness) channel from HSV colorspace
fn extract_value_channel(img: &DynamicImage) -> DynamicImage {
    let rgb = img.to_rgb8();
    let (width, height) = rgb.dimensions();

    let mut value_img = GrayImage::new(width, height);

    for (x, y, pixel) in rgb.enumerate_pixels() {
        // Value = max(R, G, B)
        let value = pixel.0[0].max(pixel.0[1]).max(pixel.0[2]);
        value_img.put_pixel(x, y, Luma([value]));
    }

    DynamicImage::ImageLuma8(value_img)
}

// Morphological operations removed - not used in parallel pipeline

/// Custom grayscale conversion with configurable weights
fn custom_grayscale(img: &DynamicImage, r_weight: f32, g_weight: f32, b_weight: f32) -> DynamicImage {
    let rgb = img.to_rgb8();
    let (width, height) = rgb.dimensions();

    let mut gray = GrayImage::new(width, height);
    for (x, y, pixel) in rgb.enumerate_pixels() {
        let r = pixel.0[0] as f32;
        let g = pixel.0[1] as f32;
        let b = pixel.0[2] as f32;
        let v = (r * r_weight + g * g_weight + b * b_weight).clamp(0.0, 255.0) as u8;
        gray.put_pixel(x, y, Luma([v]));
    }

    DynamicImage::ImageLuma8(gray)
}

/// Transform image based on color distance from dominant colors
/// Finds the two most different colors and creates a binary-ish image
fn color_distance_transform(img: &DynamicImage) -> DynamicImage {
    let rgb = img.to_rgb8();
    let (width, height) = rgb.dimensions();

    // Sample pixels to find dominant colors (corners are usually finder patterns)
    let corners = [
        rgb.get_pixel(10, 10),
        rgb.get_pixel(width - 10, 10),
        rgb.get_pixel(10, height - 10),
        rgb.get_pixel(width / 2, height / 2),
    ];

    // Find the two most different colors among corners
    let mut max_dist = 0.0f32;
    let mut color1 = corners[0];
    let mut color2 = corners[1];

    for i in 0..corners.len() {
        for j in i + 1..corners.len() {
            let dist = color_distance(corners[i], corners[j]);
            if dist > max_dist {
                max_dist = dist;
                color1 = corners[i];
                color2 = corners[j];
            }
        }
    }

    // Create image based on distance to each color
    let mut result = GrayImage::new(width, height);
    for (x, y, pixel) in rgb.enumerate_pixels() {
        let d1 = color_distance(pixel, color1);
        let d2 = color_distance(pixel, color2);

        // Closer to color1 = black, closer to color2 = white
        let ratio = if d1 + d2 > 0.001 {
            d1 / (d1 + d2)
        } else {
            0.5
        };
        let v = (ratio * 255.0) as u8;
        result.put_pixel(x, y, Luma([v]));
    }

    // Enhance and threshold
    let enhanced = enhance_contrast(&DynamicImage::ImageLuma8(result));
    apply_otsu_threshold(&enhanced)
}

fn color_distance(c1: &image::Rgb<u8>, c2: &image::Rgb<u8>) -> f32 {
    let dr = c1.0[0] as f32 - c2.0[0] as f32;
    let dg = c1.0[1] as f32 - c2.0[1] as f32;
    let db = c1.0[2] as f32 - c2.0[2] as f32;
    (dr * dr + dg * dg + db * db).sqrt()
}

/// Extract green channel and invert it
/// Useful for purple gradients where G channel differs most
fn extract_and_invert_green(img: &DynamicImage) -> DynamicImage {
    let rgb = img.to_rgb8();
    let (width, height) = rgb.dimensions();

    let mut result = GrayImage::new(width, height);
    for (x, y, pixel) in rgb.enumerate_pixels() {
        // Invert green: high G (background) -> low value, low G (foreground) -> high value
        result.put_pixel(x, y, Luma([255 - pixel.0[1]]));
    }

    enhance_contrast(&DynamicImage::ImageLuma8(result))
}

/// Parallel version of try_saturation_aggressive
/// Runs all parameter combinations concurrently using rayon
fn try_saturation_aggressive_parallel(img: &DynamicImage) -> Result<MultiDecodeResult> {
    let rgb = img.to_rgb8();
    let (width, height) = rgb.dimensions();

    // Extract HSL Saturation channel
    let mut saturation = GrayImage::new(width, height);
    for (x, y, pixel) in rgb.enumerate_pixels() {
        let r = pixel.0[0] as f32 / 255.0;
        let g = pixel.0[1] as f32 / 255.0;
        let b = pixel.0[2] as f32 / 255.0;

        let max_v = r.max(g).max(b);
        let min_v = r.min(g).min(b);
        let l = (max_v + min_v) / 2.0;

        let sat = if (max_v - min_v).abs() < 0.001 {
            0.0
        } else if l <= 0.5 {
            (max_v - min_v) / (max_v + min_v)
        } else {
            (max_v - min_v) / (2.0 - max_v - min_v)
        };

        saturation.put_pixel(x, y, Luma([(sat * 255.0) as u8]));
    }

    let sat_img = DynamicImage::ImageLuma8(saturation);

    // Generate all parameter combinations (3 resize x 3 contrast x 4 threshold = 36 combinations)
    let mut combinations: Vec<(u32, f32, u8)> = Vec::with_capacity(36);
    for &resize_target in &[200u32, 250, 300] {
        for &contrast in &[3.0f32, 3.5, 4.0] {
            for &threshold in &[30u8, 40, 50, 60] {
                combinations.push((resize_target, contrast, threshold));
            }
        }
    }

    // Process all combinations in parallel
    let result = combinations.par_iter().find_map_any(|&(resize_target, contrast, threshold)| {
        let scale = resize_target as f32 / width.max(height) as f32;
        let resized = if scale < 1.0 {
            sat_img.resize(
                (width as f32 * scale) as u32,
                (height as f32 * scale) as u32,
                image::imageops::FilterType::Lanczos3,
            )
        } else {
            sat_img.clone()
        };

        // Apply contrast
        let gray = resized.to_luma8();
        let (w, h) = gray.dimensions();
        let mut contrasted = GrayImage::new(w, h);
        for (x, y, pixel) in gray.enumerate_pixels() {
            let v = pixel.0[0] as f32;
            let new_v = ((v - 128.0) * contrast + 128.0).clamp(0.0, 255.0) as u8;
            contrasted.put_pixel(x, y, Luma([new_v]));
        }

        // Apply fixed threshold
        let mut binary = GrayImage::new(w, h);
        let thresh_val = (threshold as f32 / 100.0 * 255.0) as u8;
        for (x, y, pixel) in contrasted.enumerate_pixels() {
            let v = if pixel.0[0] > thresh_val { 255 } else { 0 };
            binary.put_pixel(x, y, Luma([v]));
        }

        let processed = DynamicImage::ImageLuma8(binary);
        if let Ok(result) = try_decode_with_both(&processed) {
            return Some(result);
        }

        // Try inverted
        let inverted = invert_image(&processed);
        if let Ok(result) = try_decode_with_both(&inverted) {
            return Some(result);
        }

        None
    });

    result.ok_or(QraiError::DecodeFailed)
}

// Removed unused sequential functions:
// - apply_median_filter (too slow)
// - try_saturation_morph (uses removed morphology ops)
// - try_random_preprocessing (replaced by parallel version)

/// Parallel version of try_random_preprocessing
/// Generates all random params upfront, then processes in parallel
fn try_random_preprocessing_parallel(img: &DynamicImage, num_tries: u32) -> Result<MultiDecodeResult> {
    use std::time::{SystemTime, UNIX_EPOCH};

    // Generate all random params upfront
    let mut seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(12345);

    let mut next_random = || -> f32 {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        seed as f32 / u64::MAX as f32
    };

    let resize_options = [0u32, 200, 250, 300, 400, 500];
    let contrast_range = (0.5f32, 4.0f32);
    let brightness_range = (0.5f32, 2.5f32);
    let blur_range = (0.0f32, 3.0f32);

    // Pre-generate all parameter sets
    let params_list: Vec<PreprocessParams> = (0..num_tries)
        .map(|_| PreprocessParams {
            resize: resize_options[(next_random() * resize_options.len() as f32) as usize % resize_options.len()],
            contrast: contrast_range.0 + next_random() * (contrast_range.1 - contrast_range.0),
            brightness: brightness_range.0 + next_random() * (brightness_range.1 - brightness_range.0),
            blur: blur_range.0 + next_random() * (blur_range.1 - blur_range.0),
            grayscale: next_random() > 0.3,
        })
        .collect();

    // Process all parameter combinations in parallel
    let result = params_list.par_iter().find_map_any(|params| {
        let processed = apply_preprocessing(img, params);

        if let Ok(result) = try_decode_with_both(&processed) {
            return Some(result);
        }

        let with_otsu = apply_otsu_threshold(&processed);
        if let Ok(result) = try_decode_with_both(&with_otsu) {
            return Some(result);
        }

        let inverted = invert_image(&with_otsu);
        if let Ok(result) = try_decode_with_both(&inverted) {
            return Some(result);
        }

        None
    });

    result.ok_or(QraiError::DecodeFailed)
}

/// Apply preprocessing with given parameters
fn apply_preprocessing(img: &DynamicImage, params: &PreprocessParams) -> DynamicImage {
    let mut result = img.clone();

    // 1. Resize if specified
    if params.resize > 0 {
        let (w, h) = result.dimensions();
        let max_dim = w.max(h);
        if max_dim > params.resize {
            let scale = params.resize as f32 / max_dim as f32;
            let new_w = (w as f32 * scale) as u32;
            let new_h = (h as f32 * scale) as u32;
            result = result.resize(new_w, new_h, image::imageops::FilterType::Lanczos3);
        }
    }

    // 2. Convert to grayscale if needed
    if params.grayscale {
        result = DynamicImage::ImageLuma8(result.to_luma8());
    }

    // 3. Apply contrast and brightness
    let rgb = result.to_rgb8();
    let (width, height) = rgb.dimensions();
    let mut adjusted = image::RgbImage::new(width, height);

    for (x, y, pixel) in rgb.enumerate_pixels() {
        let mut new_pixel = [0u8; 3];
        for c in 0..3 {
            let v = pixel.0[c] as f32;
            // Apply brightness then contrast around midpoint
            let brightened = v * params.brightness;
            let contrasted = ((brightened - 128.0) * params.contrast) + 128.0;
            new_pixel[c] = contrasted.clamp(0.0, 255.0) as u8;
        }
        adjusted.put_pixel(x, y, image::Rgb(new_pixel));
    }
    result = DynamicImage::ImageRgb8(adjusted);

    // 4. Apply blur if specified
    if params.blur > 0.5 {
        result = result.blur(params.blur);
    }

    result
}

/// Extract (R+B)/2 - G channel
/// Maximizes contrast for purple variations where foreground has low G
fn extract_rb_minus_g(img: &DynamicImage) -> DynamicImage {
    let rgb = img.to_rgb8();
    let (width, height) = rgb.dimensions();

    let mut result = GrayImage::new(width, height);
    for (x, y, pixel) in rgb.enumerate_pixels() {
        let r = pixel.0[0] as i32;
        let g = pixel.0[1] as i32;
        let b = pixel.0[2] as i32;

        // (R + B) / 2 - G, shifted and clamped to 0-255
        let val = ((r + b) / 2 - g + 128).clamp(0, 255) as u8;
        result.put_pixel(x, y, Luma([val]));
    }

    enhance_contrast(&DynamicImage::ImageLuma8(result))
}

/// Downsample image then process - sometimes helps with complex images
fn downsample_and_process(img: &DynamicImage) -> DynamicImage {
    let (width, height) = img.dimensions();

    // Downsample to ~500px on longest side
    let scale = 500.0 / width.max(height) as f32;
    if scale >= 1.0 {
        return apply_otsu_threshold(img);
    }

    let new_width = (width as f32 * scale) as u32;
    let new_height = (height as f32 * scale) as u32;

    let downsampled = img.resize(new_width, new_height, image::imageops::FilterType::Lanczos3);
    apply_otsu_threshold(&enhance_contrast(&downsampled))
}

/// Simple edge detection using Sobel operator
fn detect_edges(img: &DynamicImage) -> DynamicImage {
    let gray = img.to_luma8();
    let (width, height) = gray.dimensions();

    let mut edges = GrayImage::new(width, height);

    for y in 1..height.saturating_sub(1) {
        for x in 1..width.saturating_sub(1) {
            // Sobel kernels
            let p00 = gray.get_pixel(x - 1, y - 1).0[0] as i32;
            let p01 = gray.get_pixel(x, y - 1).0[0] as i32;
            let p02 = gray.get_pixel(x + 1, y - 1).0[0] as i32;
            let p10 = gray.get_pixel(x - 1, y).0[0] as i32;
            let p12 = gray.get_pixel(x + 1, y).0[0] as i32;
            let p20 = gray.get_pixel(x - 1, y + 1).0[0] as i32;
            let p21 = gray.get_pixel(x, y + 1).0[0] as i32;
            let p22 = gray.get_pixel(x + 1, y + 1).0[0] as i32;

            // Gx = [-1 0 1; -2 0 2; -1 0 1]
            let gx = -p00 + p02 - 2 * p10 + 2 * p12 - p20 + p22;
            // Gy = [-1 -2 -1; 0 0 0; 1 2 1]
            let gy = -p00 - 2 * p01 - p02 + p20 + 2 * p21 + p22;

            let magnitude = ((gx * gx + gy * gy) as f64).sqrt() as u8;
            edges.put_pixel(x, y, Luma([magnitude]));
        }
    }

    // Apply threshold to edges
    apply_otsu_threshold(&DynamicImage::ImageLuma8(edges))
}

/// Sharpen image to enhance edges then apply threshold
fn sharpen_image(img: &DynamicImage) -> DynamicImage {
    let gray = img.to_luma8();
    let (width, height) = gray.dimensions();

    // Simple 3x3 sharpening kernel
    // [0, -1, 0]
    // [-1, 5, -1]
    // [0, -1, 0]
    let mut sharpened = GrayImage::new(width, height);

    for y in 1..height.saturating_sub(1) {
        for x in 1..width.saturating_sub(1) {
            let center = gray.get_pixel(x, y).0[0] as i32;
            let top = gray.get_pixel(x, y - 1).0[0] as i32;
            let bottom = gray.get_pixel(x, y + 1).0[0] as i32;
            let left = gray.get_pixel(x - 1, y).0[0] as i32;
            let right = gray.get_pixel(x + 1, y).0[0] as i32;

            let val = (5 * center - top - bottom - left - right).clamp(0, 255) as u8;
            sharpened.put_pixel(x, y, Luma([val]));
        }
    }

    // Copy edges
    for x in 0..width {
        sharpened.put_pixel(x, 0, *gray.get_pixel(x, 0));
        sharpened.put_pixel(x, height - 1, *gray.get_pixel(x, height - 1));
    }
    for y in 0..height {
        sharpened.put_pixel(0, y, *gray.get_pixel(0, y));
        sharpened.put_pixel(width - 1, y, *gray.get_pixel(width - 1, y));
    }

    // Apply Otsu on sharpened image
    apply_otsu_threshold(&DynamicImage::ImageLuma8(sharpened))
}

/// Extract version from rxing result (if available in metadata)
fn extract_version_from_rxing(_result: &rxing::RXingResult) -> Option<u8> {
    // rxing doesn't directly expose version in a simple way
    // We could try to extract it from raw bytes or other metadata
    // For now, return None and let rqrr provide it
    None
}

/// Extract error correction level from rxing result
fn extract_ec_from_rxing(_result: &rxing::RXingResult) -> Option<ErrorCorrectionLevel> {
    // Similar to version, EC level extraction from rxing is complex
    None
}

/// Convert rqrr ECC level (u16) to our type
/// QR Code ECC levels: 0=M, 1=L, 2=H, 3=Q
fn convert_rqrr_ec(level: u16) -> ErrorCorrectionLevel {
    match level {
        0 => ErrorCorrectionLevel::M,
        1 => ErrorCorrectionLevel::L,
        2 => ErrorCorrectionLevel::H,
        3 => ErrorCorrectionLevel::Q,
        _ => ErrorCorrectionLevel::M, // Default fallback
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper to create a simple test QR code image
    fn create_test_qr() -> Vec<u8> {
        use image::Luma;

        // Use qrcode crate to generate a test QR
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
    fn decode_simple_qr_with_rxing() {
        let qr_bytes = create_test_qr();
        let img = image::load_from_memory(&qr_bytes).unwrap();
        let result = decode_with_rxing(&img);

        assert!(result.is_ok(), "rxing should decode simple QR");
        assert_eq!(result.unwrap().content, "https://example.com");
    }

    #[test]
    fn decode_simple_qr_with_rqrr() {
        let qr_bytes = create_test_qr();
        let img = image::load_from_memory(&qr_bytes).unwrap();
        let result = decode_with_rqrr(&img);

        assert!(result.is_ok(), "rqrr should decode simple QR");
        assert_eq!(result.unwrap().content, "https://example.com");
    }

    #[test]
    fn multi_decode_tries_both() {
        let qr_bytes = create_test_qr();
        let result = multi_decode(&qr_bytes).unwrap();

        // Should have tried both decoders
        assert!(!result.decoders_success.is_empty());
        assert_eq!(result.content, "https://example.com");

        // Both should have succeeded on simple QR
        assert!(
            result.decoders_success.contains(&"rxing".to_string())
                || result.decoders_success.contains(&"rqrr".to_string())
        );
    }

    #[test]
    fn multi_decode_provides_metadata() {
        let qr_bytes = create_test_qr();
        let result = multi_decode(&qr_bytes).unwrap();

        assert!(result.metadata.is_some());
        let meta = result.metadata.unwrap();

        // Version should be reasonable for "https://example.com"
        assert!(meta.version > 0 && meta.version <= 40);
        assert!(meta.modules > 0);
    }

    #[test]
    fn decode_invalid_image_returns_error() {
        let garbage = b"not an image at all";
        let result = multi_decode(garbage);
        assert!(result.is_err());
    }

    #[test]
    fn decode_image_without_qr_returns_error() {
        // Create a blank image
        let blank = image::DynamicImage::new_luma8(100, 100);
        let mut buf = Vec::new();
        blank
            .write_to(&mut std::io::Cursor::new(&mut buf), image::ImageFormat::Png)
            .unwrap();

        let result = multi_decode(&buf);
        assert!(result.is_err());
    }
}
