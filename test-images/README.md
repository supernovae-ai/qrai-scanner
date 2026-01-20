# Test Images

Test QR codes for validation and benchmarking.

## Structure

```
test-images/
├── clean/      # Standard QR codes (high contrast, no styling)
├── artistic/   # AI-generated artistic QR codes
└── degraded/   # Intentionally degraded (blur, low contrast)
```

## Adding Test Images

Place QR code images (PNG/JPEG) in the appropriate folder:

- **clean/**: Standard black-on-white QR codes
- **artistic/**: Stylized QR codes from QR Code AI
- **degraded/**: Blurred, low-contrast, or damaged QR codes

## Naming Convention

```
<category>_<expected-score>_<content-hash>.png
```

Examples:
- `clean_100_abc123.png` - Clean QR, expects score ~100
- `artistic_85_def456.png` - Artistic QR, expects score ~85
- `degraded_40_ghi789.png` - Degraded QR, expects score ~40

## Usage

```bash
# Run CLI on test images
qraisc test-images/clean/*.png

# Batch benchmark
for f in test-images/**/*.png; do
    echo "$f: $(qraisc -s "$f")"
done
```
