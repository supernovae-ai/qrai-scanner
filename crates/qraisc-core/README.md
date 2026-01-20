<div align="center">

# qraisc-core

**High-performance QR code validation and scannability scoring**

[![Crates.io](https://img.shields.io/crates/v/qraisc-core?style=flat-square&logo=rust&logoColor=white&color=orange)](https://crates.io/crates/qraisc-core)
[![Docs.rs](https://img.shields.io/docsrs/qraisc-core?style=flat-square&logo=docs.rs&logoColor=white)](https://docs.rs/qraisc-core)
[![License](https://img.shields.io/crates/l/qraisc-core?style=flat-square&color=blue)](LICENSE)
[![Downloads](https://img.shields.io/crates/d/qraisc-core?style=flat-square&color=green)](https://crates.io/crates/qraisc-core)

<br/>

🎨 Artistic QR &nbsp;•&nbsp; 🖼️ Image QR &nbsp;•&nbsp; 🎯 Custom QR &nbsp;•&nbsp; 📊 Scannability Score

<br/>

*Decode the undecodable. Built for AI-generated and stylized QR codes<br/>that break standard scanners.*

<br/>

[Features](#what-it-does) · [Install](#installation) · [Quick Start](#quick-start) · [API](#api-reference) · [QR Code AI](https://qrcode-ai.com)

</div>

<br/>

## What it does

Validate and score QR codes that standard scanners can't read:

- **Artistic QR codes** — AI-generated, stylized, custom designs
- **Image QR codes** — QR embedded in photos, illustrations
- **Custom QR codes** — Logos, colors, gradients, rounded corners
- **Degraded QR codes** — Blurred, low contrast, small size

## Installation

```toml
[dependencies]
qraisc-core = "0.1"
```

## Quick Start

```rust
use qraisc_core::{validate, is_valid, score};

fn main() {
    // Simple validation - just check if QR is readable
    if let Some(content) = is_valid("qr.png") {
        println!("QR contains: {}", content);
    }

    // Get scannability score (0-100)
    let s = score("qr.png");
    println!("Scannability: {}/100", s);

    // Full validation with stress tests
    let bytes = std::fs::read("qr.png").unwrap();
    let result = validate(&bytes).unwrap();

    println!("Score: {}", result.score);
    println!("Content: {:?}", result.content);
}
```

## API Reference

### Main Functions

| Function | Description | Returns |
|----------|-------------|---------|
| `validate(&[u8])` | Full validation with stress tests | `Result<ValidationResult>` |
| `validate_fast(&[u8])` | Reduced stress tests (~2x faster) | `Result<ValidationResult>` |
| `decode_only(&[u8])` | Decode without scoring (fastest) | `Result<DecodeResult>` |

### Convenience Helpers

| Function | Description | Returns |
|----------|-------------|---------|
| `is_valid(path)` | Check if QR is valid | `Option<String>` |
| `score(path)` | Get scannability score | `u8` (0-100) |
| `score_bytes(&[u8])` | Score from bytes | `u8` (0-100) |
| `passes_threshold(path, min)` | Check minimum score | `bool` |
| `summarize(path)` | Get simple summary | `QrSummary` |

## Types

```rust
pub struct ValidationResult {
    pub score: u8,                    // 0-100
    pub decodable: bool,
    pub content: Option<String>,
    pub metadata: Option<QrMetadata>,
    pub stress_results: StressResults,
}

pub struct QrMetadata {
    pub version: u8,                  // 1-40
    pub error_correction: ErrorCorrectionLevel,
    pub modules: u8,                  // 21-177
    pub decoders_success: Vec<String>,
}

pub struct StressResults {
    pub original: bool,
    pub downscale_50: bool,
    pub downscale_25: bool,
    pub blur_light: bool,
    pub blur_medium: bool,
    pub low_contrast: bool,
}
```

## Score Interpretation

| Score | Rating | Description |
|:-----:|--------|-------------|
| **80-100** | Excellent | Safe for all devices |
| **70-79** | Good | Production ready |
| **60-69** | Acceptable | May fail on older phones |
| **40-59** | Fair | Consider regenerating |
| **0-39** | Poor | Needs redesign |

## Performance

| Operation | Clean QR | Artistic QR |
|-----------|:--------:|:-----------:|
| `decode_only` | ~20ms | ~200ms |
| `validate_fast` | ~50ms | ~500ms |
| `validate` | ~80ms | ~1000ms |

## License

MIT

---

<div align="center">

Part of [**QR Code AI**](https://qrcode-ai.com) by **Thibaut MÉLEN** & [**SuperNovae AGI**](https://supernovae.studio)

<br/>

<a href="https://github.com/ThibautMelen">
  <img src="https://avatars.githubusercontent.com/u/20891897?s=200&v=4" alt="Thibaut MÉLEN" width="32"/>
</a>
&nbsp;&nbsp;
<a href="https://github.com/SuperNovae-studio">
  <img src="https://avatars.githubusercontent.com/u/33066282?s=200&v=4" alt="SuperNovae AGI" width="32"/>
</a>

</div>
