<div align="center">

# QR Code AI Scanner

**Decode the undecodable.**

High-performance QR code validation & scannability scoring for artistic, styled, and damaged QR codes.

[![crates.io](https://img.shields.io/crates/v/qrcode-ai-scanner-core?style=flat-square&logo=rust&logoColor=white&label=crates.io&color=dea584)](https://crates.io/crates/qrcode-ai-scanner-core)
[![npm](https://img.shields.io/npm/v/@supernovae-st/qrcode-ai-scanner?style=flat-square&logo=npm&logoColor=white&label=npm&color=CB3837)](https://www.npmjs.com/package/@supernovae-st/qrcode-ai-scanner)
[![License](https://img.shields.io/badge/license-AGPL--3.0-blue?style=flat-square)](LICENSE)
[![GitHub stars](https://img.shields.io/github/stars/supernovae-st/qrcode-ai-scanner?style=flat-square&logo=github)](https://github.com/supernovae-st/qrcode-ai-scanner/stargazers)
[![CI](https://img.shields.io/github/actions/workflow/status/supernovae-st/qrcode-ai-scanner/ci.yml?style=flat-square&logo=github&label=CI)](https://github.com/supernovae-st/qrcode-ai-scanner/actions)

<br>

**Part of the [QR Code AI](https://qrcode-ai.com) ecosystem**

[`qrcode-ai`](https://github.com/supernovae-st/qrcode-ai) · [`qrcode-ai-scanner`](https://github.com/supernovae-st/qrcode-ai-scanner) · [`qrcode-ai.com`](https://qrcode-ai.com)

<br>

[Why This Scanner?](#why-this-scanner) · [Features](#features) · [Installation](#installation) · [Quick Start](#quick-start) · [API](#api-reference) · [Benchmarks](#benchmarks)

</div>

---

## Why This Scanner?

Standard QR scanners fail on **89% of artistic QR codes**. AI-generated styles, embedded images, custom colors, and real-world camera captures break conventional decoders.

> Based on our benchmark of 74 artistic QR codes from [QR Code AI](https://qrcode-ai.com).

```mermaid
%%{init: {'theme': 'base', 'themeVariables': {'lineColor': '#64748b'}}}%%
flowchart LR
    accTitle: Standard vs QR Code AI Scanner
    accDescr: Comparison showing 89% success rate vs 11% for standard scanners

    classDef success fill:#10b981,stroke:#059669,stroke-width:2px,color:#ffffff
    classDef error fill:#ef4444,stroke:#dc2626,stroke-width:2px,color:#ffffff
    classDef process fill:#6366f1,stroke:#4f46e5,stroke-width:2px,color:#ffffff
    classDef data fill:#06b6d4,stroke:#0891b2,stroke-width:2px,color:#ffffff

    ART[🎨 Artistic QR]:::data --> STD[Standard Scanner]:::process
    ART --> SCANNER[QR Code AI Scanner]:::process

    STD --> FAIL[❌ 11% Success]:::error
    SCANNER --> WIN[✅ 89% Success]:::success
```

| QR Type | Challenge | Why Scanners Fail |
|---------|-----------|-------------------|
| 🎨 **Artistic** | AI-generated art styles | Extreme visual noise, pattern interference |
| 🖼️ **Image-embedded** | QR inside photos | Background confusion, perspective distortion |
| 🎯 **Custom styled** | Colors, logos, blur | Non-black/white, central obstructions |
| 📸 **Photo-captured** | Camera photos | Lighting, blur, angle, compression |

---

## Features

```mermaid
%%{init: {'theme': 'base', 'themeVariables': {'lineColor': '#64748b'}}}%%
flowchart TB
    accTitle: QR Code AI Scanner Features
    accDescr: Core capabilities of the scanner

    classDef feature fill:#6366f1,stroke:#4f46e5,stroke-width:2px,color:#ffffff
    classDef benefit fill:#10b981,stroke:#059669,stroke-width:2px,color:#ffffff
    classDef data fill:#06b6d4,stroke:#0891b2,stroke-width:2px,color:#ffffff

    subgraph DECODE[" 🔍 Smart Decoding "]
        D1[4-Tier Progressive Strategy]:::feature
        D2[Dual Decoder Fallback]:::feature
        D3[Image Preprocessing]:::feature
    end

    subgraph SCORE[" 📊 Scannability Scoring "]
        S1[0-100 Score]:::feature
        S2[7 Stress Tests]:::feature
        S3[Production Readiness]:::feature
    end

    subgraph PLATFORM[" 🚀 Multi-Platform "]
        P1[Node.js via napi-rs]:::feature
        P2[Rust Native]:::feature
        P3[CLI Tool]:::feature
    end

    DECODE --> RESULT[89% Success Rate]:::benefit
    SCORE --> RESULT
    PLATFORM --> CROSS[Cross-Platform]:::data

    style DECODE fill:#dbeafe,stroke:#3b82f6,stroke-width:2px,color:#1e3a8a
    style SCORE fill:#d1fae5,stroke:#10b981,stroke-width:2px,color:#064e3b
    style PLATFORM fill:#e0e7ff,stroke:#6366f1,stroke-width:2px,color:#312e81
```

| Feature | Description |
|---------|-------------|
| **4-Tier Decoding** | Progressive strategy from fast to thorough |
| **Dual Decoder** | rxing (ZXing) + rqrr (Quirc) fallback |
| **Scannability Score** | 0-100 rating based on 7 stress tests |
| **Multi-Platform** | Node.js, Rust, CLI with native performance |
| **Production Ready** | Security hardened with DoS protection |

---

## The Solution: 4-Tier Progressive Decoding

```mermaid
%%{init: {'theme': 'base', 'themeVariables': {'lineColor': '#64748b'}}}%%
flowchart TD
    accTitle: 4-Tier Decoding Strategy
    accDescr: Progressive decoding from fast to thorough

    classDef success fill:#10b981,stroke:#059669,stroke-width:2px,color:#ffffff
    classDef process fill:#6366f1,stroke:#4f46e5,stroke-width:2px,color:#ffffff
    classDef decision fill:#f59e0b,stroke:#d97706,stroke-width:2px,color:#ffffff
    classDef error fill:#ef4444,stroke:#dc2626,stroke-width:2px,color:#ffffff
    classDef data fill:#06b6d4,stroke:#0891b2,stroke-width:2px,color:#ffffff

    INPUT[📷 QR Image]:::data --> T1

    subgraph TIER1[" Tier 1: Original ~80ms "]
        T1[Direct Decode]:::process --> D1{OK?}:::decision
    end

    D1 -->|Yes| SUCCESS
    D1 -->|No| T2

    subgraph TIER2[" Tier 2: Quick Trio ~100ms "]
        T2[Otsu + Invert + Contrast]:::process --> D2{OK?}:::decision
    end

    D2 -->|Yes| SUCCESS
    D2 -->|No| T3

    subgraph TIER3[" Tier 3: Parallel Pool ~500ms "]
        T3[R/G/B + HSV + Grayscale]:::process --> D3{OK?}:::decision
    end

    D3 -->|Yes| SUCCESS
    D3 -->|No| T4

    subgraph TIER4[" Tier 4: Brute Force ~2s "]
        T4[256 Random Combinations]:::process --> D4{OK?}:::decision
    end

    D4 -->|Yes| SUCCESS
    D4 -->|No| FAIL

    SUCCESS[✅ Decoded!]:::success
    FAIL[❌ Unscannable]:::error

    style TIER1 fill:#d1fae5,stroke:#10b981,stroke-width:2px,color:#064e3b
    style TIER2 fill:#dbeafe,stroke:#3b82f6,stroke-width:2px,color:#1e3a8a
    style TIER3 fill:#e0e7ff,stroke:#6366f1,stroke-width:2px,color:#312e81
    style TIER4 fill:#fef3c7,stroke:#f59e0b,stroke-width:2px,color:#78350f
```

---

## Installation

### Node.js

```bash
npm install @supernovae-st/qrcode-ai-scanner
```

### Rust CLI

```bash
cargo install qrcode-ai-scanner-cli
```

### Rust Library

```bash
cargo add qrcode-ai-scanner-core
```

**Requirements:** Node.js 18+ | Rust 1.75+

---

## Quick Start

### Node.js

```typescript
import { isValid, score, validate } from '@supernovae-st/qrcode-ai-scanner';
import { readFileSync } from 'fs';

const qr = readFileSync('artistic-qr.png');

// Quick checks
const content = isValid(qr);           // "https://example.com" or null
const scannability = score(qr);        // 0-100

// Production check
if (scannability >= 70) {
  console.log('Ready for production!');
}

// Full validation with metadata
const result = validate(qr);
console.log(result.score, result.content, result.errorCorrection);
```

### Rust

```rust
use qrcode_ai_scanner_core::{is_valid, score, validate};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Quick checks
    if let Some(content) = is_valid("qr.png") {
        println!("QR contains: {}", content);
    }

    let scannability = score("qr.png");  // 0-100

    // Full validation
    let bytes = std::fs::read("qr.png")?;
    let result = validate(&bytes)?;
    println!("Score: {}, Content: {:?}", result.score, result.content);

    Ok(())
}
```

### CLI

```bash
# Full validation (JSON output)
qrcode-ai image.png

# Score only
qrcode-ai -s image.png
# Output: 85

# Decode only (fastest)
qrcode-ai -d image.png
# Output: https://example.com

# Batch processing
qrcode-ai *.png --json > results.json
```

---

## API Reference

### Core Functions

| Function | Description | Speed |
|----------|-------------|-------|
| `validate(bytes)` | Full validation + stress tests | ~1s |
| `validate_fast(bytes)` | Reduced stress tests | ~500ms |
| `decode_only(bytes)` | Just decode, no score | ~100ms |

### Convenience Helpers

| Function | Returns | Description |
|----------|---------|-------------|
| `is_valid(path)` | `string \| null` | Content if valid |
| `score(path)` | `0-100` | Scannability score |
| `passes_threshold(path, min)` | `boolean` | Score >= min? |
| `summarize(path)` | `QrSummary` | Full summary |

### Scannability Score

The score is calculated from 7 stress tests that simulate real-world scanning conditions:

```mermaid
%%{init: {'theme': 'base', 'themeVariables': {'lineColor': '#64748b'}}}%%
flowchart TD
    accTitle: Scannability Score Components
    accDescr: How the score is calculated from stress tests

    classDef test fill:#6366f1,stroke:#4f46e5,stroke-width:2px,color:#ffffff
    classDef weight fill:#f59e0b,stroke:#d97706,stroke-width:2px,color:#ffffff
    classDef result fill:#10b981,stroke:#059669,stroke-width:2px,color:#ffffff
    classDef data fill:#06b6d4,stroke:#0891b2,stroke-width:2px,color:#ffffff

    QR[📷 QR Image]:::data --> TESTS

    subgraph TESTS[" Stress Tests "]
        T1[Original 20pts]:::test
        T2[50% Scale 15pts]:::test
        T3[25% Scale 10pts]:::test
        T4[Blur σ=1 15pts]:::test
        T5[Blur σ=2 10pts]:::test
        T6[Low Contrast 15pts]:::test
        T7[Multi-decoder 15pts]:::test
    end

    TESTS --> SUM[Sum Points]:::weight
    SUM --> SCORE[Score 0-100]:::result

    style TESTS fill:#f8fafc,stroke:#64748b,stroke-width:2px,color:#334155
```

| Score | Rating | Recommendation |
|-------|--------|----------------|
| **80-100** | Excellent | Safe for all devices |
| **60-79** | Good | Works on most devices |
| **40-59** | Fair | May fail on older phones |
| **0-39** | Poor | Consider regenerating |

---

## Architecture

```mermaid
%%{init: {'theme': 'base', 'themeVariables': {'lineColor': '#64748b'}}}%%
flowchart TB
    accTitle: QR Code AI Scanner Architecture
    accDescr: Monorepo structure with core, CLI, and Node bindings

    classDef core fill:#6366f1,stroke:#4f46e5,stroke-width:2px,color:#ffffff
    classDef cli fill:#10b981,stroke:#059669,stroke-width:2px,color:#ffffff
    classDef node fill:#f59e0b,stroke:#d97706,stroke-width:2px,color:#ffffff
    classDef module fill:#8b5cf6,stroke:#7c3aed,stroke-width:2px,color:#ffffff

    subgraph REPO[" qrcode-ai-scanner/ "]
        subgraph CRATES[" crates/ "]
            CORE[qrcode-ai-scanner-core]:::core
            CLI[qrcode-ai-scanner-cli]:::cli
            NODE[qrcode-ai-scanner-node]:::node
        end

        subgraph MODULES[" Core Modules "]
            DEC[decoder.rs]:::module
            SCO[scorer.rs]:::module
            ERR[error.rs]:::module
        end
    end

    CORE --> DEC
    CORE --> SCO
    CORE --> ERR
    CLI --> CORE
    NODE --> CORE

    style REPO fill:#f8fafc,stroke:#64748b,stroke-width:2px,color:#334155
    style CRATES fill:#f1f5f9,stroke:#94a3b8,stroke-width:1px,color:#475569
    style MODULES fill:#f1f5f9,stroke:#94a3b8,stroke-width:1px,color:#475569
```

### Dual Decoder System

```mermaid
%%{init: {'theme': 'base', 'themeVariables': {'lineColor': '#64748b'}}}%%
flowchart LR
    accTitle: Dual Decoder System
    accDescr: rxing as primary, rqrr as fallback

    classDef primary fill:#6366f1,stroke:#4f46e5,stroke-width:2px,color:#ffffff
    classDef fallback fill:#8b5cf6,stroke:#7c3aed,stroke-width:2px,color:#ffffff
    classDef success fill:#10b981,stroke:#059669,stroke-width:2px,color:#ffffff
    classDef data fill:#06b6d4,stroke:#0891b2,stroke-width:2px,color:#ffffff

    IMG[📷 Image]:::data --> RXING[rxing<br/>ZXing Port]:::primary
    RXING -->|Success| DONE[✅ Decoded]:::success
    RXING -->|Fail| RQRR[rqrr<br/>Quirc Port]:::fallback
    RQRR --> DONE
```

| Decoder | Origin | Best For |
|---------|--------|----------|
| [rxing](https://crates.io/crates/rxing) | ZXing (Java) | Noisy images |
| [rqrr](https://crates.io/crates/rqrr) | Quirc (C) | Clean images |

### Platform Support

| Platform | Node.js | CLI | Rust |
|----------|:-------:|:---:|:----:|
| macOS (x64, arm64) | ✅ | ✅ | ✅ |
| Linux (x64, arm64) | ✅ | ✅ | ✅ |
| Windows (x64) | ✅ | ✅ | ✅ |

---

## Benchmarks

### Test Dataset: 74 Artistic QR Codes

| Metric | Value | Notes |
|--------|-------|-------|
| **Success Rate** | 89.2% (66/74) | vs ~10% for standard scanners |
| **Average Time** | 967ms | Includes all tiers |
| **Fastest** | 77ms | Clean QRs (Tier 1) |
| **P95** | ~2s | Artistic QRs (Tier 3-4) |

### Performance Journey

| Phase | Avg Time | Improvement |
|-------|----------|-------------|
| Initial | 8000ms | Baseline |
| Phase 1 | 2000ms | 4x faster |
| Phase 2 | 1500ms | 5.3x faster |
| Phase 3 | 1000ms | 8x faster |
| **Final** | **967ms** | **8.3x faster** |

---

## Development

```bash
# Run tests
cargo test --workspace

# Build release
cargo build --release

# Run benchmarks
cargo bench

# Format & lint
cargo fmt && cargo clippy
```

---

## Comparison

| Feature | QR Code AI Scanner | Standard Scanners |
|---------|:------------------:|:-----------------:|
| Artistic QR support | ✅ 89% | ❌ ~10% |
| Multi-decoder fallback | ✅ | ❌ |
| Scannability scoring | ✅ 0-100 | ❌ Binary |
| Stress test validation | ✅ 7 tests | ❌ None |
| Production readiness check | ✅ | ❌ |
| DoS protection | ✅ | ❌ |

---

## License

**AGPL-3.0** — Network service modifications require source disclosure.

[Read the full license](https://www.gnu.org/licenses/agpl-3.0.en.html)

---

<div align="center">

**Built by [Thibaut MÉLEN](https://github.com/ThibautMelen) & [SuperNovae Studio](https://supernovae.studio)**

<a href="https://github.com/ThibautMelen"><img src="https://avatars.githubusercontent.com/u/20891897?s=64&v=4" width="32" alt="Thibaut MÉLEN"></a>
&nbsp;
<a href="https://github.com/supernovae-st"><img src="https://avatars.githubusercontent.com/u/33066282?s=64&v=4" width="32" alt="SuperNovae Studio"></a>

</div>
