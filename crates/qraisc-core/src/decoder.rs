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
    // TIER 4: Full brute force (256 random combos) - last resort
    // Some images like 3eb25154 need many tries to find winning params
    // ========================================================================
    if let Ok(result) = try_mini_brute_force(img, 256) {
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
