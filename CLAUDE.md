# QRAI Scanner - Claude Instructions

## Project Overview

High-performance QR code scanner and scannability scorer for QR Code AI SaaS.

**Stack**: Rust workspace with napi-rs Node.js bindings
**Target**: <200ms latency, <50ms ideal

## Architecture

```
qrai-scanner/
├── crates/
│   ├── qraisc-core/     # Core library (decoder, scorer, types)
│   ├── qraisc-cli/    # CLI binary (qraisc)
│   └── qraisc-node/     # Node.js napi-rs bindings
├── test-images/       # Test QR codes
└── docs/plans/        # Design documents
```

## Key Modules

- `decoder.rs` - Multi-decoder (rxing primary, rqrr fallback)
- `scorer.rs` - Stress tests (blur, downscale, contrast) → score 0-100
- `types.rs` - ValidationResult, QrMetadata, StressResults

## Commands

```bash
# Development
cargo test --workspace           # Run all tests
cargo test -p qraisc-core          # Test core only
cargo clippy --workspace         # Lint
cargo fmt --all                  # Format

# Build
cargo build -p qraisc-cli --release
cargo build -p qraisc-node --release

# Run CLI
./target/release/qraisc <image.png>

# Node binding
cd crates/qraisc-node && npm run build
```

## Performance Guidelines

- Use `image::load_from_memory` for zero-copy when possible
- Parallel stress tests with rayon when beneficial
- Avoid re-encoding images for stress tests (work on DynamicImage directly)
- Profile with `cargo flamegraph` if needed

## Testing Strategy

- TDD: Tests written first in each module
- Test QR generation: Use `qrcode` crate in dev-dependencies
- Categories: clean, artistic, degraded

## Code Style

- Edition 2021
- No unwrap in library code (use Result)
- Document public APIs
- Serde for all public types

## Dependencies

| Crate | Purpose |
|-------|---------|
| rxing | Primary QR decoder (ZXing port) |
| rqrr | Fallback decoder (Quirc port) |
| image | Image loading/transforms |
| napi/napi-derive | Node.js bindings |
| clap | CLI argument parsing |
| thiserror | Error types |
| serde | Serialization |

## Git Workflow

- Conventional commits: `feat(core):`, `fix(cli):`, `perf(scorer):`
- Run tests before commit
- Keep PRs focused
