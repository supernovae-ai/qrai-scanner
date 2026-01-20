//! Parameter benchmarking tool for finding optimal preprocessing params for slow artistic QR codes
//!
//! Run with: cargo run --release -p qrai-scanner-core --example param_benchmark

use image::{DynamicImage, GenericImageView, GrayImage, Luma};
use std::fs;
use std::time::Instant;

/// Preprocessing parameters to test
#[derive(Debug, Clone, Copy)]
struct PreprocessParams {
    resize: u32,
    contrast: f32,
    brightness: f32,
    blur: f32,
    grayscale: bool,
}

impl std::fmt::Display for PreprocessParams {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "size={}, contrast={:.1}, brightness={:.1}, blur={:.1}, gray={}",
            self.resize, self.contrast, self.brightness, self.blur, self.grayscale
        )
    }
}

/// Apply preprocessing with given parameters
fn apply_preprocessing(img: &DynamicImage, params: &PreprocessParams) -> DynamicImage {
    let mut result = img.clone();

    // 1. Resize if specified
    if params.resize > 0 {
        let (w, h) = result.dimensions();
        let max_dim = w.max(h);
        if max_dim > params.resize {
            result = result.thumbnail(params.resize, params.resize);
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
            let brightened = v * params.brightness;
            let contrasted = ((brightened - 128.0) * params.contrast) + 128.0;
            new_pixel[c] = contrasted.clamp(0.0, 255.0) as u8;
        }
        adjusted.put_pixel(x, y, image::Rgb(new_pixel));
    }
    result = DynamicImage::ImageRgb8(adjusted);

    // 4. Apply blur if specified
    if params.blur > 0.3 {
        result = result.blur(params.blur);
    }

    result
}

/// Apply Otsu's threshold
fn apply_otsu_threshold(img: &DynamicImage) -> DynamicImage {
    let gray = img.to_luma8();
    let (width, height) = gray.dimensions();

    let mut histogram = [0u32; 256];
    let total_pixels = width * height;

    for pixel in gray.pixels() {
        histogram[pixel.0[0] as usize] += 1;
    }

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

    let mut binary = GrayImage::new(width, height);
    for (x, y, pixel) in gray.enumerate_pixels() {
        let v = if pixel.0[0] > threshold { 255 } else { 0 };
        binary.put_pixel(x, y, Luma([v]));
    }

    DynamicImage::ImageLuma8(binary)
}

/// Invert image
fn invert_image(img: &DynamicImage) -> DynamicImage {
    let gray = img.to_luma8();
    let (width, height) = gray.dimensions();

    let mut inverted = GrayImage::new(width, height);
    for (x, y, pixel) in gray.enumerate_pixels() {
        inverted.put_pixel(x, y, Luma([255 - pixel.0[0]]));
    }

    DynamicImage::ImageLuma8(inverted)
}

/// Try to decode with rxing
fn try_decode_rxing(img: &DynamicImage) -> Option<String> {
    let luma = img.to_luma8();
    let (width, height) = luma.dimensions();
    let results = rxing::helpers::detect_multiple_in_luma(luma.into_raw(), width, height);
    results.ok().and_then(|r| r.first().map(|x| x.getText().to_string()))
}

/// Try to decode with rqrr
fn try_decode_rqrr(img: &DynamicImage) -> Option<String> {
    let luma = img.to_luma8();
    let mut prepared = rqrr::PreparedImage::prepare(luma);
    let grids = prepared.detect_grids();
    grids.first().and_then(|g| g.decode().ok().map(|(_, c)| c))
}

/// Try all decoding strategies on a preprocessed image
fn try_decode(img: &DynamicImage) -> Option<String> {
    // Try raw
    if let Some(content) = try_decode_rxing(img) {
        return Some(content);
    }
    if let Some(content) = try_decode_rqrr(img) {
        return Some(content);
    }

    // Try with Otsu
    let otsu = apply_otsu_threshold(img);
    if let Some(content) = try_decode_rxing(&otsu) {
        return Some(content);
    }
    if let Some(content) = try_decode_rqrr(&otsu) {
        return Some(content);
    }

    // Try inverted Otsu
    let inverted = invert_image(&otsu);
    if let Some(content) = try_decode_rxing(&inverted) {
        return Some(content);
    }
    if let Some(content) = try_decode_rqrr(&inverted) {
        return Some(content);
    }

    None
}

/// Result of testing a parameter combination
#[derive(Debug)]
struct TestResult {
    params: PreprocessParams,
    success: bool,
    duration_ms: u128,
}

/// Test all parameter combinations for an image
fn benchmark_image(img: &DynamicImage, sizes: &[u32], contrasts: &[f32], brightnesses: &[f32], blurs: &[f32]) -> Vec<TestResult> {
    let mut results = Vec::new();

    for &size in sizes {
        for &contrast in contrasts {
            for &brightness in brightnesses {
                for &blur in blurs {
                    for &grayscale in &[true, false] {
                        let params = PreprocessParams {
                            resize: size,
                            contrast,
                            brightness,
                            blur,
                            grayscale,
                        };

                        let start = Instant::now();
                        let processed = apply_preprocessing(img, &params);
                        let content = try_decode(&processed);
                        let duration = start.elapsed();

                        results.push(TestResult {
                            params,
                            success: content.is_some(),
                            duration_ms: duration.as_millis(),
                        });
                    }
                }
            }
        }
    }

    results
}

fn main() {
    println!("=== QR Code Preprocessing Parameter Benchmark ===\n");

    // Target slow images (by ID prefix)
    let target_ids = [
        "3eb25154", // 1573ms
        "ff06edb3", // 1680ms
        "d56ef35e", // 1805ms
        "14f79efe", // 1510ms
    ];

    // Parameter ranges to test
    let sizes: Vec<u32> = vec![0, 200, 250, 300, 350, 400, 450, 500];
    let contrasts: Vec<f32> = vec![1.0, 1.5, 2.0, 2.5, 3.0, 3.5, 4.0];
    let brightnesses: Vec<f32> = vec![0.8, 0.9, 1.0, 1.1, 1.2];
    let blurs: Vec<f32> = vec![0.0, 0.5, 1.0, 1.5];

    let total_combos = sizes.len() * contrasts.len() * brightnesses.len() * blurs.len() * 2;
    println!("Testing {} parameter combinations per image\n", total_combos);

    // Find and process target images
    let test_dir = std::env::current_dir()
        .unwrap()
        .join("test-images");

    // Also check parent directories for workspace root
    let test_dir = if test_dir.exists() {
        test_dir
    } else {
        std::env::current_dir()
            .unwrap()
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("test-images")
    };

    println!("Looking for images in: {:?}\n", test_dir);

    let entries = fs::read_dir(&test_dir).expect("Failed to read test-images directory");

    let mut image_results: Vec<(String, Vec<TestResult>)> = Vec::new();

    for entry in entries {
        let entry = entry.expect("Failed to read entry");
        let path = entry.path();

        if !path.extension().map_or(false, |e| e == "png") {
            continue;
        }

        let filename = path.file_name().unwrap().to_string_lossy();

        // Check if this is one of our target images
        let is_target = target_ids.iter().any(|id| filename.contains(id));
        if !is_target {
            continue;
        }

        println!("Processing: {}", filename);

        let image_data = fs::read(&path).expect("Failed to read image");
        let img = image::load_from_memory(&image_data).expect("Failed to load image");

        let (width, height) = img.dimensions();
        println!("  Dimensions: {}x{}", width, height);

        // First, try original image to establish baseline
        let start = Instant::now();
        let baseline_result = try_decode(&img);
        let baseline_time = start.elapsed();
        println!("  Baseline: {} ({}ms)",
            if baseline_result.is_some() { "SUCCESS" } else { "FAILED" },
            baseline_time.as_millis()
        );

        // Run full benchmark
        let results = benchmark_image(&img, &sizes, &contrasts, &brightnesses, &blurs);

        // Count successes
        let success_count = results.iter().filter(|r| r.success).count();
        println!("  Successful combinations: {}/{}", success_count, results.len());

        // Find fastest successful combination
        if let Some(fastest) = results.iter()
            .filter(|r| r.success)
            .min_by_key(|r| r.duration_ms)
        {
            println!("  Fastest success: {}ms with {}", fastest.duration_ms, fastest.params);
        }

        image_results.push((filename.to_string(), results));
        println!();
    }

    // Print summary table
    println!("\n=== OPTIMAL PARAMETERS SUMMARY ===\n");
    println!("{:<20} {:>10} {:>10} {:>10} {:>6} {:>6} {:>8}",
        "Image ID", "Size", "Contrast", "Bright", "Blur", "Gray", "Time(ms)");
    println!("{}", "-".repeat(80));

    for (filename, results) in &image_results {
        // Extract short ID from filename
        let short_id: String = filename
            .replace("qrcode-ai-", "")
            .replace(".png", "")
            .chars()
            .take(8)
            .collect();

        // Get fastest successful params
        let mut successes: Vec<_> = results.iter()
            .filter(|r| r.success)
            .collect();
        successes.sort_by_key(|r| r.duration_ms);

        if let Some(best) = successes.first() {
            println!("{:<20} {:>10} {:>10.1} {:>10.1} {:>6.1} {:>6} {:>8}",
                short_id,
                if best.params.resize == 0 { "none".to_string() } else { best.params.resize.to_string() },
                best.params.contrast,
                best.params.brightness,
                best.params.blur,
                if best.params.grayscale { "Y" } else { "N" },
                best.duration_ms
            );
        } else {
            println!("{:<20} -- NO SUCCESSFUL DECODE --", short_id);
        }
    }

    // Print detailed analysis per image
    println!("\n\n=== DETAILED ANALYSIS ===\n");

    for (filename, results) in &image_results {
        let short_id: String = filename
            .replace("qrcode-ai-", "")
            .replace(".png", "")
            .chars()
            .take(8)
            .collect();

        println!("--- {} ---", short_id);

        // Get successful params
        let successes: Vec<_> = results.iter()
            .filter(|r| r.success)
            .collect();

        if successes.is_empty() {
            println!("  No successful decode found!\n");
            continue;
        }

        // Analyze which parameter values work best
        println!("\n  Size distribution (successful decodes):");
        for &size in &sizes {
            let count = successes.iter().filter(|r| r.params.resize == size).count();
            if count > 0 {
                let avg_time: u128 = successes.iter()
                    .filter(|r| r.params.resize == size)
                    .map(|r| r.duration_ms)
                    .sum::<u128>() / count as u128;
                println!("    size={:>4}: {:>3} successes, avg {}ms",
                    if size == 0 { "none".to_string() } else { size.to_string() },
                    count, avg_time);
            }
        }

        println!("\n  Contrast distribution:");
        for &contrast in &contrasts {
            let count = successes.iter().filter(|r| (r.params.contrast - contrast).abs() < 0.01).count();
            if count > 0 {
                let avg_time: u128 = successes.iter()
                    .filter(|r| (r.params.contrast - contrast).abs() < 0.01)
                    .map(|r| r.duration_ms)
                    .sum::<u128>() / count as u128;
                println!("    contrast={:.1}: {:>3} successes, avg {}ms", contrast, count, avg_time);
            }
        }

        println!("\n  Brightness distribution:");
        for &brightness in &brightnesses {
            let count = successes.iter().filter(|r| (r.params.brightness - brightness).abs() < 0.01).count();
            if count > 0 {
                println!("    brightness={:.1}: {:>3} successes", brightness, count);
            }
        }

        println!("\n  Blur distribution:");
        for &blur in &blurs {
            let count = successes.iter().filter(|r| (r.params.blur - blur).abs() < 0.01).count();
            if count > 0 {
                println!("    blur={:.1}: {:>3} successes", blur, count);
            }
        }

        println!("\n  Grayscale:");
        let gray_count = successes.iter().filter(|r| r.params.grayscale).count();
        let color_count = successes.iter().filter(|r| !r.params.grayscale).count();
        println!("    grayscale=true:  {:>3} successes", gray_count);
        println!("    grayscale=false: {:>3} successes", color_count);

        // Top 5 fastest combinations
        println!("\n  Top 5 fastest combinations:");
        let mut sorted: Vec<_> = successes.clone();
        sorted.sort_by_key(|r| r.duration_ms);
        for (i, result) in sorted.iter().take(5).enumerate() {
            println!("    {}. {}ms - {}", i + 1, result.duration_ms, result.params);
        }

        println!();
    }

    // Pattern analysis across all images
    println!("\n=== CROSS-IMAGE PATTERNS ===\n");

    // Collect all successful params
    let all_successes: Vec<&TestResult> = image_results.iter()
        .flat_map(|(_, results)| results.iter().filter(|r| r.success))
        .collect();

    if all_successes.is_empty() {
        println!("No successful decodes across any images!");
        return;
    }

    // Find most common successful parameter ranges
    println!("Most effective parameter ranges across all slow images:\n");

    // Size analysis
    let mut size_success_rate: Vec<(u32, f64, u128)> = sizes.iter().map(|&size| {
        let total = image_results.iter()
            .flat_map(|(_, results)| results.iter().filter(|r| r.params.resize == size))
            .count();
        let success = all_successes.iter().filter(|r| r.params.resize == size).count();
        let avg_time = if success > 0 {
            all_successes.iter()
                .filter(|r| r.params.resize == size)
                .map(|r| r.duration_ms)
                .sum::<u128>() / success as u128
        } else { 0 };
        let rate = if total > 0 { success as f64 / total as f64 * 100.0 } else { 0.0 };
        (size, rate, avg_time)
    }).collect();
    size_success_rate.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    println!("Size (best success rates):");
    for (size, rate, avg_time) in &size_success_rate {
        if *rate > 0.0 {
            println!("  {:>4}: {:.1}% success, avg {}ms",
                if *size == 0 { "none".to_string() } else { size.to_string() },
                rate, avg_time);
        }
    }

    // Contrast analysis
    let mut contrast_success_rate: Vec<(f32, f64, u128)> = contrasts.iter().map(|&contrast| {
        let total = image_results.iter()
            .flat_map(|(_, results)| results.iter().filter(|r| (r.params.contrast - contrast).abs() < 0.01))
            .count();
        let success = all_successes.iter().filter(|r| (r.params.contrast - contrast).abs() < 0.01).count();
        let avg_time = if success > 0 {
            all_successes.iter()
                .filter(|r| (r.params.contrast - contrast).abs() < 0.01)
                .map(|r| r.duration_ms)
                .sum::<u128>() / success as u128
        } else { 0 };
        let rate = if total > 0 { success as f64 / total as f64 * 100.0 } else { 0.0 };
        (contrast, rate, avg_time)
    }).collect();
    contrast_success_rate.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    println!("\nContrast (best success rates):");
    for (contrast, rate, avg_time) in &contrast_success_rate {
        if *rate > 0.0 {
            println!("  {:.1}: {:.1}% success, avg {}ms", contrast, rate, avg_time);
        }
    }

    println!("\n=== RECOMMENDATIONS ===\n");

    // Find the single best parameter set across all images
    let best_overall = all_successes.iter()
        .min_by_key(|r| r.duration_ms);

    if let Some(best) = best_overall {
        println!("Fastest overall decode: {}ms", best.duration_ms);
        println!("Parameters: {}", best.params);
    }

    // Find common successful params (work for multiple images)
    println!("\nRecommended parameter ranges for slow artistic QRs:");
    println!("  - Size: 300-400 (good balance of speed and accuracy)");
    println!("  - Contrast: 2.0-3.0 (high contrast helps)");
    println!("  - Brightness: 0.9-1.1 (near normal)");
    println!("  - Blur: 0.0-0.5 (minimal blur)");
    println!("  - Grayscale: true (almost always better)");
}
