# QR Code AI Scanner

<div align="center">

<h3>High-Performance QR Code Validation & Scannability Scoring</h3>

<p>
<strong>Decode the undecodable.</strong> Built for QR codes that break standard scanners:<br/>
🎨 <strong>Artistic</strong> · 🖼️ <strong>Image-embedded</strong> · 🎯 <strong>Custom styled</strong> · 📸 <strong>Photo-captured</strong>
</p>

<p><em>Part of the <a href="https://qrcode-ai.com">QR Code AI</a> ecosystem · <a href="https://github.com/supernovae-st/qrcode-ai">GitHub</a></em></p>

[![Rust](https://img.shields.io/badge/Rust-1.75+-orange?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-AGPL--3.0-blue?logo=opensourceinitiative&logoColor=white)](LICENSE)
[![Success Rate](https://img.shields.io/badge/Success_Rate-89.2%25-brightgreen?logo=checkmarx&logoColor=white)](README.md#benchmarks)
[![Avg Time](https://img.shields.io/badge/Avg_Time-967ms-green?logo=speedtest&logoColor=white)](README.md#benchmarks)
[![Node.js](https://img.shields.io/badge/Node.js-Bindings-339933?logo=nodedotjs&logoColor=white)](README.md#nodejs)
[![crates.io](https://img.shields.io/crates/v/qrcode-ai-scanner-core?logo=rust&logoColor=white&label=crates.io)](https://crates.io/crates/qrcode-ai-scanner-core)
[![npm](https://img.shields.io/npm/v/@supernovae-st/qrcode-ai-scanner?logo=npm&logoColor=white&label=npm)](https://www.npmjs.com/package/@supernovae-st/qrcode-ai-scanner)

<br>

[Installation](#installation) · [Quick Start](#quick-start) · [Why This Scanner?](#why-this-scanner) · [Benchmarks](#benchmarks) · [API Reference](#api-reference)

</div>

> **License:** This software is [AGPL-3.0](LICENSE) licensed. If you modify it and provide it as a network service, you must make your source code available. [Learn more](https://www.gnu.org/licenses/agpl-3.0.en.html).

---

## At a Glance

```mermaid
%%{init: {'theme': 'base', 'themeVariables': {'lineColor': '#64748b'}}}%%
flowchart LR
    classDef input fill:#06b6d4,stroke:#0891b2,stroke-width:2px,color:#ffffff
    classDef process fill:#6366f1,stroke:#4f46e5,stroke-width:2px,color:#ffffff
    classDef output fill:#10b981,stroke:#059669,stroke-width:2px,color:#ffffff

    A[📷 Image]:::input --> B[Scanner<br>4-Tier Decode]:::process
    B --> C[✅ Content]:::output
    B --> D[📊 Score]:::output
    B --> E[📋 Metadata]:::output
```

---

## Installation

### Node.js

Requires [Node.js](https://nodejs.org/) 18+ (includes npm).

```bash
npm install @supernovae-st/qrcode-ai-scanner
```

```typescript
import { validate, score } from '@supernovae-st/qrcode-ai-scanner';
```

### Rust CLI

Requires [Rust](https://rustup.rs/) toolchain.

```bash
cargo install qrcode-ai-scanner-cli
```

```bash
qrcode-ai image.png        # Full validation
qrcode-ai -s image.png     # Score only
```

### Rust Library

```bash
cargo add qrcode-ai-scanner-core
```

```rust
use qrcode_ai_scanner_core::{validate, score};
```

---

## Quick Start

### One-liner Rust

```rust
use qrcode_ai_scanner_core::is_valid;

// Check if QR is valid and get content
if let Some(content) = is_valid("qr.png") {
    println!("QR contains: {}", content);
}
```

### Score Check

```rust
use qrcode_ai_scanner_core::{score, passes_threshold};

// Get scannability score (0-100)
let s = score("qr.png");
println!("Score: {}/100", s);

// Check if production-ready (score >= 70)
if passes_threshold("qr.png", 70) {
    println!("Ready for production!");
}
```

### Full Validation

```rust
use qrcode_ai_scanner_core::validate;

let bytes = std::fs::read("qr.png")?;
let result = validate(&bytes)?;

println!("Score: {}", result.score);           // 0-100
println!("Content: {:?}", result.content);      // Decoded text
println!("Version: {:?}", result.metadata);     // QR metadata
```

### Node.js

```typescript
import { validate, decode } from '@supernovae-st/qrcode-ai-scanner';
import { readFileSync } from 'fs';

const result = validate(readFileSync('qr.png'));
console.log(`Score: ${result.score}/100`);
console.log(`Content: ${result.content}`);
```

### CLI

```bash
# Full validation (JSON)
qrcode-ai image.png

# Score only (for scripts)
qrcode-ai -s image.png    # Output: 85

# Decode only (fast)
qrcode-ai -d image.png
```

---

## Why This Scanner?

```mermaid
%%{init: {'theme': 'base', 'themeVariables': {'lineColor': '#64748b'}}}%%
flowchart LR
    accTitle: Standard vs QR Code AI Scanner
    accDescr: Comparison showing 89% success rate vs 11% for standard scanners

    classDef success fill:#10b981,stroke:#059669,stroke-width:2px,color:#ffffff
    classDef error fill:#ef4444,stroke:#dc2626,stroke-width:2px,color:#ffffff
    classDef process fill:#6366f1,stroke:#4f46e5,stroke-width:2px,color:#ffffff
    classDef data fill:#06b6d4,stroke:#0891b2,stroke-width:2px,color:#ffffff

    ART[Artistic QR]:::data --> STD[Standard Scanner]:::process
    ART --> SCANNER[QR Code AI Scanner]:::process

    STD --> FAIL[11% Success]:::error
    SCANNER --> WIN[89% Success]:::success
```

### The Problem

Standard QR scanners fail on non-standard QR codes:

| QR Type | Challenge | Why Scanners Fail |
|---------|-----------|-------------------|
| 🎨 **Artistic** | AI-generated art styles | Extreme visual noise, pattern interference |
| 🖼️ **Image-embedded** | QR inside photos | Background confusion, perspective distortion |
| 🎯 **Custom styled** | Colors, logos, blur effects | Non-black/white colors, central obstructions |
| 📸 **Photo-captured** | Camera photos of QR codes | Lighting, blur, angle, compression artifacts |
| 🔀 **Multi-pattern** | Gradients, textures, rounded corners | False edges, module boundary confusion |

### The Solution

QR Code AI Scanner uses a **4-tier progressive decoding strategy** that applies increasingly aggressive preprocessing until successful decode:

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

    INPUT[QR Image]:::data --> T1

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

    SUCCESS[Decoded!]:::success
    FAIL[Unscannable]:::error

    style TIER1 fill:#d1fae5,stroke:#10b981,stroke-width:2px,color:#064e3b
    style TIER2 fill:#dbeafe,stroke:#3b82f6,stroke-width:2px,color:#1e3a8a
    style TIER3 fill:#e0e7ff,stroke:#6366f1,stroke-width:2px,color:#312e81
    style TIER4 fill:#fef3c7,stroke:#f59e0b,stroke-width:2px,color:#78350f
```

---

## Benchmarks

### Test Results: 74 Artistic QR Codes

```mermaid
%%{init: {'theme': 'base', 'themeVariables': {'lineColor': '#64748b'}}}%%
pie showData
    accTitle: Scanner Success Rate
    accDescr: 89.2% of artistic QR codes successfully decoded
    title Success Rate (74 Artistic QRs)
    "Decoded" : 66
    "Failed" : 8
```

| Metric | Value | Notes |
|--------|-------|-------|
| **Success Rate** | 66/74 (89.2%) | vs ~10% for standard scanners |
| **Average Time** | 967ms | Includes all tiers |
| **Fastest** | 77ms | Clean QRs (Tier 1) |
| **P95** | ~2000ms | Artistic QRs (Tier 3-4) |

### Speed Distribution by Tier

```mermaid
%%{init: {'theme': 'base', 'themeVariables': {'lineColor': '#64748b'}}}%%
pie showData
    accTitle: Speed Distribution
    accDescr: Distribution of QR codes across speed tiers
    title Decode Time Distribution
    "Tier 1 - Instant (<200ms)" : 15
    "Tier 2 - Fast (200-500ms)" : 9
    "Tier 3 - Medium (500-1500ms)" : 33
    "Tier 4 - Slow (>1500ms)" : 9
    "Failed" : 8
```

### Optimization Journey

```mermaid
%%{init: {'theme': 'base', 'themeVariables': {'lineColor': '#64748b'}}}%%
xychart-beta
    accTitle: Performance Optimization Timeline
    accDescr: Shows the 11x improvement in decode time
    title "Average Decode Time (ms)"
    x-axis ["Initial", "Phase 1", "Phase 2", "Phase 3", "Phase 4"]
    y-axis "Time (ms)" 0 --> 8000
    bar [8000, 2000, 1500, 1000, 967]
```

| Phase | Strategy | Time | Speedup |
|-------|----------|------|---------|
| Initial | Baseline | 5-11s | — |
| Phase 1 | Remove slow strategies | ~2s | 5x |
| Phase 2 | Single luma8 conversion | ~1.5s | 7x |
| Phase 3 | Strategy reordering | ~1s | 10x |
| Phase 4 | Rayon parallelization | ~967ms | **11x** |

### Score vs Speed Analysis

| Category | Score | Speed | Description |
|----------|-------|-------|-------------|
| **Clean QRs** | High (80+) | Fast (<200ms) | Standard QRs, Tier 1 decode |
| **Light Artistic** | Good (60-80) | Medium (200-500ms) | Subtle effects, Tier 2 |
| **Heavy Artistic** | Fair (40-60) | Slow (500-1500ms) | Strong effects, Tier 3 |
| **Failed** | Poor (<40) | Variable | Undecodable or unreliable |

---

## API Reference

### Core Functions

| Function | Description | Performance |
|----------|-------------|-------------|
| `validate()` | Full validation with score | ~1s |
| `validate_fast()` | Reduced stress tests | ~500ms |
| `decode_only()` | Just decode, no score | ~100ms |

### Convenience Helpers

| Function | Description | Returns |
|----------|-------------|---------|
| `is_valid(path)` | Check if QR is valid | `Option<String>` |
| `score(path)` | Get scannability score | `u8 (0-100)` |
| `score_bytes(bytes)` | Score from bytes | `u8 (0-100)` |
| `passes_threshold(path, min)` | Check minimum score | `bool` |
| `summarize(path)` | Get simple summary | `QrSummary` |

### Scannability Score

The score (0-100) indicates how reliably the QR will scan across devices:

```mermaid
%%{init: {'theme': 'base', 'themeVariables': {'lineColor': '#64748b'}}}%%
flowchart LR
    accTitle: Scannability Score Components
    accDescr: How the score is calculated from stress tests

    classDef test fill:#6366f1,stroke:#4f46e5,stroke-width:2px,color:#ffffff
    classDef weight fill:#f59e0b,stroke:#d97706,stroke-width:2px,color:#ffffff
    classDef result fill:#10b981,stroke:#059669,stroke-width:2px,color:#ffffff

    QR[QR Image] --> T1[Original<br>20pts]:::test
    QR --> T2[50% Scale<br>15pts]:::test
    QR --> T3[25% Scale<br>10pts]:::test
    QR --> T4[Blur σ=1<br>15pts]:::test
    QR --> T5[Blur σ=2<br>10pts]:::test
    QR --> T6[Low Contrast<br>15pts]:::test
    QR --> T7[Multi-decoder<br>15pts]:::test

    T1 & T2 & T3 & T4 & T5 & T6 & T7 --> SUM[Sum Points]:::weight --> SCORE[Score 0-100]:::result
```

| Score | Rating | Production Use |
|-------|--------|----------------|
| 80-100 | Excellent | Safe for all devices |
| 60-79 | Good | Works on most devices |
| 40-59 | Fair | May fail on older phones |
| 0-39 | Poor | Consider regenerating |

---

## Node.js Integration

### Installation

```bash
cd crates/qrcode-ai-scanner-node
npm install && npm run build
```

### One-liner Examples

```typescript
import { isValid, score, isProductionReady, summarize } from '@supernovae-st/qrcode-ai-scanner';
import { readFileSync } from 'fs';

const buffer = readFileSync('qr.png');

// Check if QR is valid
const content = isValid(buffer);
if (content) {
  console.log(`QR contains: ${content}`);
}

// Get scannability score
console.log(`Score: ${score(buffer)}/100`);

// Check production readiness
if (isProductionReady(buffer)) {
  console.log('Ready for production!');
}
```

### Full Validation

```typescript
import { validate, validateFast, decode } from '@supernovae-st/qrcode-ai-scanner';
import { readFileSync } from 'fs';

const buffer = readFileSync('qr.png');

// Full validation with stress tests (~1s)
const result = validate(buffer);
console.log(`Score: ${result.score}/100`);
console.log(`Content: ${result.content}`);
console.log(`EC Level: ${result.errorCorrection}`);

// Fast validation (~500ms)
const fast = validateFast(buffer);

// Decode only, no score (~100ms)
const decoded = decode(buffer);
```

### Summary Helper

```typescript
import { summarize } from '@supernovae-st/qrcode-ai-scanner';

const summary = summarize(readFileSync('qr.png'));

console.log(summary);
// {
//   valid: true,
//   score: 85,
//   content: 'https://example.com',
//   errorCorrection: 'H',
//   rating: 'Excellent',
//   productionReady: true
// }

if (summary.productionReady) {
  await uploadToProduction(summary.content);
}
```

### API Reference (Node.js)

#### Core Functions

| Function | Description | Performance |
|----------|-------------|-------------|
| `validate(buffer)` | Full validation with score | ~1s |
| `validateFast(buffer)` | Reduced stress tests | ~500ms |
| `decode(buffer)` | Just decode, no score | ~100ms |

#### Convenience Helpers

| Function | Description | Returns |
|----------|-------------|---------|
| `isValid(buffer)` | Check if valid | `string \| null` |
| `score(buffer)` | Get score | `number (0-100)` |
| `passesThreshold(buffer, min)` | Check threshold | `boolean` |
| `isProductionReady(buffer)` | Score >= 70? | `boolean` |
| `summarize(buffer)` | Get summary | `QrSummary` |
| `getRating(score)` | Score to rating | `string` |

#### Types

```typescript
interface ValidationResult {
  score: number;              // 0-100
  decodable: boolean;
  content: string | null;
  version: number | null;     // QR version 1-40
  errorCorrection: string | null; // L/M/Q/H
  modules: number | null;
  decodersSuccess: string[];
  stressOriginal: boolean;
  stressDownscale50: boolean;
  stressDownscale25: boolean;
  stressBlurLight: boolean;
  stressBlurMedium: boolean;
  stressLowContrast: boolean;
}

interface QrSummary {
  valid: boolean;
  score: number;
  content: string;
  errorCorrection: string;
  rating: string;           // Excellent/Good/Fair/Poor
  productionReady: boolean; // score >= 70
}
```

---

## Installation

### Node.js (npm)

```bash
# From npm (recommended)
npm install @supernovae-st/qrcode-ai-scanner

# From GitHub
npm install github:supernovae-st/qrcode-ai-scanner

# Build from source
git clone https://github.com/supernovae-st/qrcode-ai-scanner.git
cd qrcode-ai-scanner/crates/qrcode-ai-scanner-node
npm install && npm run build
```

Then use it:

```typescript
import { validate, isValid, score } from '@supernovae-st/qrcode-ai-scanner';
import { readFileSync } from 'fs';

const buffer = readFileSync('qr.png');
console.log(`Score: ${score(buffer)}/100`);
```

### CLI Tool

```bash
# From crates.io
cargo install qrcode-ai-scanner-cli

# From GitHub
cargo install --git https://github.com/supernovae-st/qrcode-ai-scanner qrcode-ai-scanner-cli

# Build from source
git clone https://github.com/supernovae-st/qrcode-ai-scanner.git
cd qrcode-ai-scanner
cargo build --release -p qrcode-ai-scanner-cli

# Add to PATH (macOS/Linux)
sudo cp target/release/qrcode-ai /usr/local/bin/
```

Then use it:

```bash
qrcode-ai image.png           # Full validation
qrcode-ai -s image.png        # Score only (for scripts)
qrcode-ai -j image.png        # JSON output
```

### Rust Library

```toml
# From crates.io
[dependencies]
qrcode-ai-scanner-core = "0.2"

# From GitHub
[dependencies]
qrcode-ai-scanner-core = { git = "https://github.com/supernovae-st/qrcode-ai-scanner" }

# From local path
[dependencies]
qrcode-ai-scanner-core = { path = "../qrcode-ai-scanner/crates/qrcode-ai-scanner-core" }
```

Then use it:

```rust
use qrcode_ai_scanner_core::{validate, is_valid, score};

fn main() {
    // Simple check
    if let Some(content) = is_valid("qr.png") {
        println!("QR contains: {}", content);
    }

    // Get score
    let s = score("qr.png");
    println!("Score: {}/100", s);
}
```

### Platform Support

| Platform | Node.js | CLI | Rust |
|----------|---------|-----|------|
| macOS x64 | ✅ | ✅ | ✅ |
| macOS arm64 (M1/M2) | ✅ | ✅ | ✅ |
| Linux x64 | ✅ | ✅ | ✅ |
| Linux arm64 | ✅ | ✅ | ✅ |
| Windows x64 | ✅ | ✅ | ✅ |

---

## Architecture

### Project Structure

```
qrcode-ai-scanner/
├── crates/
│   ├── qrcode-ai-scanner-core/        # Core library (decoder, scorer, types)
│   │   ├── src/
│   │   │   ├── decoder.rs  # Multi-decoder + 4-tier strategy
│   │   │   ├── scorer.rs   # Stress tests + scoring
│   │   │   ├── types.rs    # ValidationResult, QrMetadata
│   │   │   └── error.rs    # Error types
│   │   └── Cargo.toml
│   ├── qrcode-ai-scanner-cli/         # CLI binary
│   └── qrcode-ai-scanner-node/        # Node.js napi-rs bindings
├── test-qr-speed/          # Benchmark images (74 artistic QRs)
├── scripts/                # Benchmark & test scripts
└── docs/                   # Design documents
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

    IMG[Preprocessed Image]:::data --> RXING[rxing<br>ZXing Port]:::primary
    RXING -->|Success| DONE[Decoded]:::success
    RXING -->|Fail| RQRR[rqrr<br>Quirc Port]:::fallback
    RQRR --> DONE
```

| Decoder | Origin | Strength |
|---------|--------|----------|
| [rxing](https://crates.io/crates/rxing) | ZXing (Java) | Better on noisy images |
| [rqrr](https://crates.io/crates/rqrr) | Quirc (C) | Faster on clean images |

### Dependencies

| Crate | Purpose |
|-------|---------|
| [rxing](https://crates.io/crates/rxing) | Primary QR decoder |
| [rqrr](https://crates.io/crates/rqrr) | Fallback decoder |
| [image](https://crates.io/crates/image) | Image loading & transforms |
| [rayon](https://crates.io/crates/rayon) | Parallel processing |
| [napi](https://crates.io/crates/napi) | Node.js bindings |
| [clap](https://crates.io/crates/clap) | CLI argument parsing |

### Developer Journey

```mermaid
%%{init: {'theme': 'base', 'themeVariables': {'lineColor': '#64748b'}}}%%
journey
    accTitle: Developer Experience with QR Code AI Scanner
    accDescr: Steps a developer takes to integrate the scanner
    title Developer Journey
    section Installation
      Add dependency: 5: Dev
      Import library: 5: Dev
    section Quick Validation
      Load image bytes: 5: Dev
      Call isValid(): 5: Dev, Scanner
      Get content: 5: Dev
    section Production Check
      Call score(): 4: Dev, Scanner
      Check threshold: 4: Dev
      Deploy if ready: 5: Dev
    section Advanced
      Full validate(): 3: Dev, Scanner
      Analyze stress tests: 3: Dev
      Optimize QR design: 4: Dev
```

### Score Decision Flow

```mermaid
%%{init: {'theme': 'base', 'themeVariables': {'lineColor': '#64748b'}}}%%
stateDiagram-v2
    accTitle: Score-based Decision Flow
    accDescr: How to decide based on scannability score

    [*] --> Validate
    Validate --> Score

    Score --> Excellent: score >= 80
    Score --> Good: 60-79
    Score --> Fair: 40-59
    Score --> Poor: < 40

    Excellent --> Deploy: Safe for all devices
    Good --> Deploy: Works on most
    Fair --> Review: May fail on some
    Poor --> Regenerate: Too risky

    Deploy --> [*]
    Review --> Regenerate: If critical
    Review --> Deploy: If acceptable
    Regenerate --> Validate: New QR
```

---

## Development

```bash
# Run tests
cargo test --workspace

# Build release
cargo build -p qrcode-ai-scanner-cli --release

# Run benchmarks
cargo bench -p qrcode-ai-scanner-core

# Format & lint
cargo fmt --all && cargo clippy --workspace
```

---

## License

AGPL-3.0

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
