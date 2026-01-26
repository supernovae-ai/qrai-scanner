<div align="center">

# qrcode-ai-scanner-cli

**Command-line QR code validator and scannability scorer**

[![Crates.io](https://img.shields.io/crates/v/qrcode-ai-scanner-cli?style=flat-square&logo=rust&logoColor=white&color=orange)](https://crates.io/crates/qrcode-ai-scanner-cli)
[![License](https://img.shields.io/crates/l/qrcode-ai-scanner-cli?style=flat-square&color=blue)](LICENSE)

🎨 Artistic · 🖼️ Image-embedded · 🎯 Custom styled · 📸 Photo-captured

*Validate QR codes that break standard scanners, from the command line.*

</div>

<br/>

## Installation

### From crates.io (recommended)

```bash
cargo install qrcode-ai-scanner-cli
```

### From GitHub

```bash
cargo install --git https://github.com/supernovae-st/qrcode-ai-scanner qrcode-ai-scanner-cli
```

### Build from source

```bash
git clone https://github.com/supernovae-st/qrcode-ai-scanner.git
cd qrcode-ai-scanner
cargo build --release -p qrcode-ai-scanner-cli
# Binary at: target/release/qrcode-ai
```

### Add to PATH (after building)

```bash
# macOS/Linux
sudo cp target/release/qrcode-ai /usr/local/bin/

# Or add to your shell profile
echo 'export PATH="$PATH:/path/to/qrcode-ai-scanner/target/release"' >> ~/.zshrc
```

## Usage

### Basic Validation

```bash
# Full validation with visual output
qrcode-ai image.png

# JSON output
qrcode-ai -j image.png

# Score only (for scripts)
qrcode-ai -s image.png
# Output: 85
```

### Fast Mode

```bash
# Reduced stress tests (~2x faster)
qrcode-ai -f image.png
```

### Decode Only

```bash
# Skip stress tests, just decode
qrcode-ai -d image.png
```

### Timing Information

```bash
# Show processing time
qrcode-ai -t image.png
```

### Quiet Mode

```bash
# Minimal output
qrcode-ai -q image.png
```

## Options

| Flag | Long | Description |
|------|------|-------------|
| `-s` | `--score-only` | Output only the score (0-100) |
| `-d` | `--decode-only` | Decode without stress tests |
| `-f` | `--fast` | Fast validation (~2x faster) |
| `-j` | `--json` | JSON output |
| `-t` | `--timing` | Show timing info |
| `-q` | `--quiet` | Minimal output |
| `-h` | `--help` | Show help |
| `-V` | `--version` | Show version |

## Examples

### Scripting

```bash
# Check if QR is production-ready (score >= 70)
if [ $(qrcode-ai -s image.png) -ge 70 ]; then
    echo "Production ready!"
fi

# Batch process directory
for f in *.png; do
    score=$(qrcode-ai -s "$f")
    echo "$f: $score"
done
```

### CI/CD Integration

```bash
# GitHub Actions / GitLab CI
- name: Validate QR codes
  run: |
    for qr in assets/qr/*.png; do
      score=$(qrcode-ai -s "$qr")
      if [ $score -lt 70 ]; then
        echo "❌ $qr failed with score $score"
        exit 1
      fi
      echo "✅ $qr passed with score $score"
    done
```

### JSON Processing

```bash
# Extract content with jq
qrcode-ai -j image.png | jq -r '.content'

# Get all stress test results
qrcode-ai -j image.png | jq '.stress_results'
```

## Output Format

### Visual (default)

```
  ╔══════════════════════════════════════════════════════════════════╗
  ║  🌟  SCANNABILITY SCORE: 100                                   ║
  ╚══════════════════════════════════════════════════════════════════╝

  📄 File:    image.png
  ⏱  Time:    54ms

  📝 DECODED CONTENT
  https://example.com

  🧪 STRESS TEST RESULTS
   ✓ Original             [PASS]
   ✓ Downscale 50%        [PASS]
   ✓ Downscale 25%        [PASS]
   ✓ Blur (light)         [PASS]
   ✓ Blur (medium)        [PASS]
   ✓ Low Contrast         [PASS]

  📊 QR METADATA
   Version:          v2   (size complexity)
   Error Correction: M    (~15% recovery)
   Modules:          25x25  (grid size)
   Decoders:         rxing, rqrr
```

### JSON (`-j`)

```json
{
  "score": 100,
  "decodable": true,
  "content": "https://example.com",
  "metadata": {
    "version": 2,
    "error_correction": "M",
    "modules": 25,
    "decoders_success": ["rxing", "rqrr"]
  },
  "stress_results": {
    "original": true,
    "downscale_50": true,
    "downscale_25": true,
    "blur_light": true,
    "blur_medium": true,
    "low_contrast": true
  }
}
```

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Error (file not found, decode failed, etc.) |

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
