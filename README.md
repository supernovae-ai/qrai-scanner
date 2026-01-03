# QRAI Validator

High-performance QR code validation and scannability scoring, written in Rust.

## Features

- **Multi-decoder**: Uses rxing (ZXing port) and rqrr for robust decoding
- **Scannability Score**: 0-100 score based on stress tests (blur, downscale, contrast)
- **Fast**: Optimized for real-time validation (<200ms target)
- **Node.js Bindings**: napi-rs integration for direct use in Node.js
- **CLI Tool**: Test QR codes from the command line

## Installation

### CLI

```bash
cargo install --path crates/qrai-cli
```

### Node.js

```bash
cd crates/qrai-node
npm install
npm run build
```

### As Rust Library

```toml
[dependencies]
qrai-core = { git = "https://github.com/SuperNovae-studio/qrai-validator" }
```

## Usage

### CLI

```bash
# Full validation with JSON output
qrai-validator image.png

# Pretty printed JSON
qrai-validator -p image.png

# Score only (for scripts)
qrai-validator -s image.png
# Output: 85

# Decode only (fast, no stress tests)
qrai-validator -d image.png
```

### Rust

```rust
use qrai_core::validate;

let image_bytes = std::fs::read("qr.png")?;
let result = validate(&image_bytes)?;

println!("Score: {}", result.score);
println!("Content: {:?}", result.content);
println!("Stress tests: {:?}", result.stress_results);
```

### Node.js

```typescript
import { validate, decode, validateScoreOnly } from '@qrcodeai/qrai-validator';
import { readFileSync } from 'fs';

const imageBuffer = readFileSync('qr.png');

// Full validation
const result = validate(imageBuffer);
console.log(`Score: ${result.score}`);
console.log(`Content: ${result.content}`);

// Fast decode (no stress tests)
const decoded = decode(imageBuffer);
console.log(`Content: ${decoded.content}`);

// Score only
const score = validateScoreOnly(imageBuffer);
console.log(`Score: ${score}`);
```

## Score Calculation

The scannability score is computed by running the QR through stress tests:

| Test | Weight | Description |
|------|--------|-------------|
| Original | 20 | Decode at original resolution |
| Downscale 50% | 15 | Decode at half size |
| Downscale 25% | 10 | Decode at quarter size |
| Blur (light) | 15 | Decode with σ=1.0 blur |
| Blur (medium) | 10 | Decode with σ=2.0 blur |
| Low Contrast | 15 | Decode with 50% contrast |
| Multi-decoder | 15 | Bonus if both decoders succeed |

**Score = (passed tests × weights) / total weight × 100**

## Development

```bash
# Run tests
cargo test --workspace

# Build CLI
cargo build -p qrai-cli --release

# Build Node binding
cd crates/qrai-node
npm run build

# Format code
cargo fmt --all

# Lint
cargo clippy --workspace
```

## Architecture

```
qrai-validator/
├── crates/
│   ├── qrai-core/     # Core library (decoding, scoring)
│   ├── qrai-cli/      # Command-line interface
│   └── qrai-node/     # Node.js bindings (napi-rs)
└── test-images/       # Test QR codes
```

## License

MIT License - see [LICENSE](LICENSE) for details.

## Author

Thibaut @ [SuperNovae Studio](https://supernovae.studio)
