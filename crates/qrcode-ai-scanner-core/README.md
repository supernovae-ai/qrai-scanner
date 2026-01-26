<div align="center">

# qrcode-ai-scanner-core

**High-performance QR code validation and scannability scoring**

[![Crates.io](https://img.shields.io/crates/v/qrcode-ai-scanner-core?style=flat-square&logo=rust&logoColor=white&color=orange)](https://crates.io/crates/qrcode-ai-scanner-core)
[![Docs.rs](https://img.shields.io/docsrs/qrcode-ai-scanner-core?style=flat-square&logo=docs.rs&logoColor=white)](https://docs.rs/qrcode-ai-scanner-core)
[![License](https://img.shields.io/crates/l/qrcode-ai-scanner-core?style=flat-square&color=blue)](LICENSE)
[![Downloads](https://img.shields.io/crates/d/qrcode-ai-scanner-core?style=flat-square&color=green)](https://crates.io/crates/qrcode-ai-scanner-core)

<br/>

🎨 Artistic &nbsp;•&nbsp; 🖼️ Image-embedded &nbsp;•&nbsp; 🎯 Custom styled &nbsp;•&nbsp; 📸 Photo-captured

<br/>

*Decode the undecodable. Built for QR codes that break standard scanners.*

<br/>

[Features](#what-it-does) · [Install](#installation) · [Quick Start](#quick-start) · [API](#api-reference) · [QR Code AI](https://qrcode-ai.com) · [GitHub](https://github.com/supernovae-st/qrcode-ai)

</div>

<br/>

## What it does

Validate and score QR codes that standard scanners can't read:

- **Artistic QR codes** — AI-generated, stylized, custom art designs
- **Image-embedded QR codes** — QR inside photos, illustrations, backgrounds
- **Custom styled QR codes** — Logos, colors, gradients, blur, rounded corners
- **Photo-captured QR codes** — Camera photos with lighting, angle, compression
- **Multi-pattern QR codes** — Textures, overlays, complex visual effects

## Installation

```toml
[dependencies]
qrcode-ai-scanner-core = "0.1"
```

## Quick Start

```rust
use qrcode_ai_scanner_core::{validate, is_valid, score};

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

Part of [**QR Code AI**](https://qrcode-ai.com) by **Thibaut MÉLEN** & [**SuperNovae Studio**](https://supernovae.studio)

<br/>

<a href="https://github.com/ThibautMelen">
  <img src="https://avatars.githubusercontent.com/u/20891897?s=200&v=4" alt="Thibaut MÉLEN" width="32"/>
</a>
&nbsp;&nbsp;
<a href="https://github.com/supernovae-st">
  <img src="https://avatars.githubusercontent.com/u/33066282?s=200&v=4" alt="SuperNovae Studio" width="32"/>
</a>

</div>
