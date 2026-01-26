use anyhow::{Context, Result};
use clap::Parser;
use qrcode_ai_scanner_core::{decode_only, validate, validate_fast, ValidationResult, DecodeResult};
use std::path::{Path, PathBuf};
use std::time::Instant;

/// QRAI Validator - QR code validation and scannability scoring
#[derive(Parser, Debug)]
#[command(name = "qrcode-ai")]
#[command(author = "Thibaut @ SuperNovae Studio")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "Validate QR codes and compute scannability score")]
struct Cli {
    /// Image file to validate (PNG, JPEG, etc.)
    image: PathBuf,

    /// Output only the score (0-100), useful for scripts
    #[arg(long, short = 's')]
    score_only: bool,

    /// Decode only mode: skip stress tests entirely (fastest)
    #[arg(long, short = 'd')]
    decode_only: bool,

    /// Fast validation: reduced stress tests (~2x faster)
    #[arg(long, short = 'f')]
    fast: bool,

    /// JSON output instead of visual
    #[arg(long, short = 'j')]
    json: bool,

    /// Show timing information
    #[arg(long, short = 't')]
    timing: bool,

    /// Quiet mode: minimal output
    #[arg(long, short = 'q')]
    quiet: bool,
}

// ANSI color codes
mod colors {
    pub const RESET: &str = "\x1b[0m";
    pub const BOLD: &str = "\x1b[1m";
    pub const DIM: &str = "\x1b[2m";

    pub const RED: &str = "\x1b[31m";
    pub const GREEN: &str = "\x1b[32m";
    pub const YELLOW: &str = "\x1b[33m";
    pub const BLUE: &str = "\x1b[34m";
    pub const MAGENTA: &str = "\x1b[35m";
    pub const CYAN: &str = "\x1b[36m";
    pub const WHITE: &str = "\x1b[37m";

}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let start = Instant::now();

    // Show banner unless quiet or json mode
    if !cli.quiet && !cli.json && !cli.score_only {
        print_banner();
    }

    let image_bytes = std::fs::read(&cli.image)
        .with_context(|| format!("Failed to read image file: {:?}", cli.image))?;

    let read_time = start.elapsed();

    if cli.decode_only {
        let result = decode_only(&image_bytes)
            .with_context(|| "Failed to decode QR code")?;
        let total_time = start.elapsed();

        if cli.json {
            println!("{}", serde_json::to_string_pretty(&result)?);
        } else if !cli.quiet {
            print_decode_result(&result, &cli.image, total_time.as_millis() as u64);
        }

        if cli.timing {
            eprintln!("{}⏱  Read: {:?}, Decode: {:?}, Total: {:?}{}",
                colors::DIM, read_time, total_time - read_time, total_time, colors::RESET);
        }
    } else if cli.score_only {
        let result = if cli.fast {
            validate_fast(&image_bytes)
        } else {
            validate(&image_bytes)
        }.with_context(|| "Failed to validate QR code")?;

        println!("{}", result.score);

        if cli.timing {
            let total_time = start.elapsed();
            eprintln!("{}⏱  Total: {:?}{}",
                colors::DIM, total_time, colors::RESET);
        }
    } else {
        let result = if cli.fast {
            validate_fast(&image_bytes)
        } else {
            validate(&image_bytes)
        }.with_context(|| "Failed to validate QR code")?;

        let total_time = start.elapsed();

        if cli.json {
            println!("{}", serde_json::to_string_pretty(&result)?);
        } else if cli.quiet {
            println!("{}", result.score);
        } else {
            print_validation_result(&result, &cli.image, total_time.as_millis() as u64, cli.fast);
        }

        if cli.timing && !cli.json {
            eprintln!("\n{}⏱  Read: {:?}, Validate: {:?}, Total: {:?}{}",
                colors::DIM, read_time, total_time - read_time, total_time, colors::RESET);
        }
    }

    Ok(())
}

fn print_banner() {
    println!(r#"
{}{}   ___  ____      _    ___      {}
{}{}  / _ \|  _ \    / \  |_ _|     {}
{}{} | | | | |_) |  / _ \  | |      {}
{}{} | |_| |  _ <  / ___ \ | |      {}
{}{}  \__\_\_| \_\/_/   \_\___|     {}
{}                                {}
{}  ╔═══════════════════════════╗ {}
{}  ║   QR CODE VALIDATOR       ║ {}
{}  ╚═══════════════════════════╝ {}
"#,
        colors::BOLD, colors::CYAN, colors::RESET,
        colors::BOLD, colors::CYAN, colors::RESET,
        colors::BOLD, colors::CYAN, colors::RESET,
        colors::BOLD, colors::CYAN, colors::RESET,
        colors::BOLD, colors::CYAN, colors::RESET,
        colors::DIM, colors::RESET,
        colors::YELLOW, colors::RESET,
        colors::YELLOW, colors::RESET,
        colors::YELLOW, colors::RESET,
    );
}

fn print_decode_result(result: &DecodeResult, path: &Path, time_ms: u64) {
    println!("{}╔══════════════════════════════════════════════════════════════════╗{}",
        colors::GREEN, colors::RESET);
    println!("{}║  {}✓ QR CODE DECODED{}                                               ║{}",
        colors::GREEN, colors::BOLD, colors::RESET, colors::RESET);
    println!("{}╚══════════════════════════════════════════════════════════════════╝{}",
        colors::GREEN, colors::RESET);

    println!();
    println!("  {}📄 File:{}    {}", colors::DIM, colors::RESET, path.display());
    println!("  {}⏱  Time:{}    {}ms", colors::DIM, colors::RESET, time_ms);
    println!();

    println!("  {}╭─────────────────────────────────────────────────────────────────╮{}",
        colors::BLUE, colors::RESET);
    println!("  {}│{} 📝 CONTENT                                                      {}│{}",
        colors::BLUE, colors::RESET, colors::BLUE, colors::RESET);
    println!("  {}├─────────────────────────────────────────────────────────────────┤{}",
        colors::BLUE, colors::RESET);

    // Wrap content if too long
    let content = &result.content;
    if content.len() <= 60 {
        println!("  {}│{} {}{}{}",
            colors::BLUE, colors::RESET,
            colors::BOLD, content, colors::RESET);
    } else {
        for chunk in content.as_bytes().chunks(60) {
            let s = String::from_utf8_lossy(chunk);
            println!("  {}│{} {}", colors::BLUE, colors::RESET, s);
        }
    }

    println!("  {}╰─────────────────────────────────────────────────────────────────╯{}",
        colors::BLUE, colors::RESET);

    if let Some(ref meta) = result.metadata {
        println!();
        println!("  {}📊 METADATA{}", colors::DIM, colors::RESET);
        println!("  {}├── Version:    {}v{}{}",
            colors::DIM, colors::WHITE, meta.version, colors::RESET);
        println!("  {}├── EC Level:   {}{}{}",
            colors::DIM, colors::WHITE, meta.error_correction, colors::RESET);
        println!("  {}└── Modules:    {}{}x{}{}",
            colors::DIM, colors::WHITE, meta.modules, meta.modules, colors::RESET);
    }

    println!();
}

fn print_validation_result(result: &ValidationResult, path: &Path, time_ms: u64, fast_mode: bool) {
    let score = result.score;
    let (score_color, score_emoji, score_label) = get_score_style(score);

    // Header
    println!();
    println!("  {}╔══════════════════════════════════════════════════════════════════╗{}",
        score_color, colors::RESET);
    println!("  {}║                                                                  ║{}",
        score_color, colors::RESET);
    println!("  {}║  {}{}  SCANNABILITY SCORE: {:>3}{}                                   {}║{}",
        score_color, colors::BOLD, score_emoji, score, colors::RESET, score_color, colors::RESET);
    println!("  {}║                                                                  ║{}",
        score_color, colors::RESET);
    println!("  {}╚══════════════════════════════════════════════════════════════════╝{}",
        score_color, colors::RESET);

    // Score bar visualization
    println!();
    print_score_bar(score);
    println!();

    // Score interpretation
    println!("  {}{}  {}{}", score_color, score_emoji, score_label, colors::RESET);
    println!();

    // File info
    println!("  {}📄 File:{}    {}", colors::DIM, colors::RESET, path.display());
    println!("  {}⏱  Time:{}    {}ms {}",
        colors::DIM, colors::RESET, time_ms,
        if fast_mode { "(fast mode)" } else { "" });
    println!();

    // Content
    if let Some(ref content) = result.content {
        println!("  {}╭─────────────────────────────────────────────────────────────────╮{}",
            colors::BLUE, colors::RESET);
        println!("  {}│{} 📝 DECODED CONTENT                                              {}│{}",
            colors::BLUE, colors::RESET, colors::BLUE, colors::RESET);
        println!("  {}├─────────────────────────────────────────────────────────────────┤{}",
            colors::BLUE, colors::RESET);

        if content.len() <= 60 {
            println!("  {}│{} {}{}{}",
                colors::BLUE, colors::RESET,
                colors::BOLD, content, colors::RESET);
        } else {
            for chunk in content.as_bytes().chunks(58) {
                let s = String::from_utf8_lossy(chunk);
                println!("  {}│{}  {}", colors::BLUE, colors::RESET, s);
            }
        }

        println!("  {}╰─────────────────────────────────────────────────────────────────╯{}",
            colors::BLUE, colors::RESET);
        println!();
    }

    // Stress test results
    println!("  {}╭─────────────────────────────────────────────────────────────────╮{}",
        colors::MAGENTA, colors::RESET);
    println!("  {}│{} 🧪 STRESS TEST RESULTS                                          {}│{}",
        colors::MAGENTA, colors::RESET, colors::MAGENTA, colors::RESET);
    println!("  {}├─────────────────────────────────────────────────────────────────┤{}",
        colors::MAGENTA, colors::RESET);

    print_stress_row("Original", result.stress_results.original, true);
    print_stress_row("Downscale 50%", result.stress_results.downscale_50, true);
    print_stress_row("Downscale 25%", result.stress_results.downscale_25, !fast_mode);
    print_stress_row("Blur (light)", result.stress_results.blur_light, true);
    print_stress_row("Blur (medium)", result.stress_results.blur_medium, !fast_mode);
    print_stress_row("Low Contrast", result.stress_results.low_contrast, !fast_mode);

    println!("  {}╰─────────────────────────────────────────────────────────────────╯{}",
        colors::MAGENTA, colors::RESET);

    // Metadata
    if let Some(ref meta) = result.metadata {
        println!();
        println!("  {}╭─────────────────────────────────────────────────────────────────╮{}",
            colors::CYAN, colors::RESET);
        println!("  {}│{} 📊 QR METADATA                                                  {}│{}",
            colors::CYAN, colors::RESET, colors::CYAN, colors::RESET);
        println!("  {}├─────────────────────────────────────────────────────────────────┤{}",
            colors::CYAN, colors::RESET);
        println!("  {}│{}  Version:          {}v{:<3}{}  (size complexity)                    {}│{}",
            colors::CYAN, colors::RESET, colors::BOLD, meta.version, colors::RESET, colors::CYAN, colors::RESET);
        println!("  {}│{}  Error Correction: {}{}{}    ({})                              {}│{}",
            colors::CYAN, colors::RESET, colors::BOLD, meta.error_correction, colors::RESET,
            get_ec_description(meta.error_correction), colors::CYAN, colors::RESET);
        println!("  {}│{}  Modules:          {}{}x{}{}  (grid size)                         {}│{}",
            colors::CYAN, colors::RESET, colors::BOLD, meta.modules, meta.modules, colors::RESET,
            colors::CYAN, colors::RESET);
        println!("  {}│{}  Decoders:         {}{}{}                                     {}│{}",
            colors::CYAN, colors::RESET, colors::GREEN,
            meta.decoders_success.join(", "), colors::RESET, colors::CYAN, colors::RESET);
        println!("  {}╰─────────────────────────────────────────────────────────────────╯{}",
            colors::CYAN, colors::RESET);
    }

    println!();
}

fn print_score_bar(score: u8) {
    let bar_width = 60;
    let filled = (score as usize * bar_width) / 100;

    let score_color = if score >= 80 {
        colors::GREEN
    } else if score >= 40 {
        colors::YELLOW
    } else {
        colors::RED
    };

    // Top border
    println!("  {}╔{}╗{}",
        colors::DIM, "═".repeat(bar_width + 12), colors::RESET);

    // Score percentage display - BIG and centered
    let pct_str = format!("{}%", score);
    let padding = (bar_width + 12 - pct_str.len() - 2) / 2;
    println!("  {}║{}{}{}{}{}{}║{}",
        colors::DIM,
        " ".repeat(padding),
        score_color, colors::BOLD,
        pct_str,
        colors::RESET,
        " ".repeat(bar_width + 12 - padding - pct_str.len()),
        colors::RESET);

    // Empty line
    println!("  {}║{}║{}",
        colors::DIM, " ".repeat(bar_width + 12), colors::RESET);

    // Progress bar with gradient
    print!("  {}║{} ", colors::DIM, colors::RESET);
    print!("{}│", colors::DIM);

    for i in 0..bar_width {
        if i < filled {
            // Gradient from red to yellow to green
            let segment_color = if i < bar_width * 40 / 100 {
                colors::RED
            } else if i < bar_width * 70 / 100 {
                colors::YELLOW
            } else {
                colors::GREEN
            };
            print!("{}█{}", segment_color, colors::RESET);
        } else {
            print!("{}░{}", colors::DIM, colors::RESET);
        }
    }
    println!("{}│{} {}║{}", colors::DIM, colors::RESET, " ".repeat(8), colors::RESET);

    // Scale markers
    print!("  {}║{} ", colors::DIM, colors::RESET);
    print!("{}0", colors::DIM);
    print!("{}", " ".repeat(bar_width / 4 - 1));
    print!("25");
    print!("{}", " ".repeat(bar_width / 4 - 2));
    print!("50");
    print!("{}", " ".repeat(bar_width / 4 - 2));
    print!("75");
    print!("{}", " ".repeat(bar_width / 4 - 2));
    println!("100{}{}║{}", colors::RESET, " ".repeat(4), colors::RESET);

    // Bottom border
    println!("  {}╚{}╝{}",
        colors::DIM, "═".repeat(bar_width + 12), colors::RESET);

    // Score indicator below
    let arrow_pos = (score as usize * bar_width) / 100 + 5;
    println!("  {}{}{}▼{}",
        " ".repeat(arrow_pos),
        score_color, colors::BOLD,
        colors::RESET);
}

fn print_stress_row(name: &str, passed: bool, enabled: bool) {
    let (icon, status, color) = if !enabled {
        ("○", "skipped", colors::DIM)
    } else if passed {
        ("✓", "PASS", colors::GREEN)
    } else {
        ("✗", "FAIL", colors::RED)
    };

    println!("  {}│{}  {}{}{} {:<20} {}[{}]{}",
        colors::MAGENTA, colors::RESET,
        color, icon, colors::RESET,
        name,
        color, status, colors::RESET);
}

fn get_score_style(score: u8) -> (&'static str, &'static str, &'static str) {
    match score {
        90..=100 => (colors::GREEN, "🌟", "EXCELLENT - Highly scannable in any condition"),
        80..=89 => (colors::GREEN, "✨", "GREAT - Very reliable scanning"),
        70..=79 => (colors::YELLOW, "👍", "GOOD - Should scan in most conditions"),
        60..=69 => (colors::YELLOW, "⚠️ ", "FAIR - May have issues in poor conditions"),
        40..=59 => (colors::RED, "⚡", "WEAK - Scanning may be unreliable"),
        _ => (colors::RED, "❌", "POOR - High risk of scan failures"),
    }
}

fn get_ec_description(ec: qrcode_ai_scanner_core::ErrorCorrectionLevel) -> &'static str {
    match ec {
        qrcode_ai_scanner_core::ErrorCorrectionLevel::L => "~7% recovery",
        qrcode_ai_scanner_core::ErrorCorrectionLevel::M => "~15% recovery",
        qrcode_ai_scanner_core::ErrorCorrectionLevel::Q => "~25% recovery",
        qrcode_ai_scanner_core::ErrorCorrectionLevel::H => "~30% recovery",
    }
}
