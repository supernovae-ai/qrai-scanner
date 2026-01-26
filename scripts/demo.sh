#!/bin/bash
# QRCode-AI - Demo Script
# Generates a test QR code and runs validation

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

cd "$PROJECT_DIR"

echo ""
echo "╔════════════════════════════════════════════════════════════════╗"
echo "║            QRCode-AI - Demo                               ║"
echo "╚════════════════════════════════════════════════════════════════╝"
echo ""

# Build if needed
if [ ! -f "target/release/qrcode-ai" ]; then
    echo "🔨 Building release binary..."
    cargo build -p qrcode-ai-scanner-cli --release --quiet
fi

# Generate a test QR code using a small Rust program
echo "🎨 Generating test QR code..."

# Create a temp file for the test QR generator
cat > /tmp/gen_qr.rs << 'EOF'
use image::{DynamicImage, Luma};
use qrcode::QrCode;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let content = args.get(1).map(|s| s.as_str()).unwrap_or("https://qrcode-ai.com");
    let output = args.get(2).map(|s| s.as_str()).unwrap_or("test-qr.png");

    let code = QrCode::new(content.as_bytes()).unwrap();
    let img = code.render::<Luma<u8>>()
        .min_dimensions(400, 400)
        .build();

    let dyn_img = DynamicImage::ImageLuma8(img);
    dyn_img.save(output).unwrap();

    println!("Generated: {} -> {}", content, output);
}
EOF

# Use cargo to run a quick QR generation
cargo run --quiet --example gen_test_qr 2>/dev/null || {
    # Fallback: create example if it doesn't exist
    mkdir -p examples
    cat > examples/gen_test_qr.rs << 'EOF'
use image::{DynamicImage, Luma};

fn main() {
    let content = std::env::args().nth(1).unwrap_or_else(|| "https://qrcode-ai.com".to_string());
    let output = std::env::args().nth(2).unwrap_or_else(|| "test-qr.png".to_string());

    let code = qrcode::QrCode::new(content.as_bytes()).unwrap();
    let img = code.render::<Luma<u8>>()
        .min_dimensions(400, 400)
        .build();

    let dyn_img = DynamicImage::ImageLuma8(img);
    dyn_img.save(&output).unwrap();

    eprintln!("✓ Generated: {} -> {}", content, output);
}
EOF
    cargo run --quiet --example gen_test_qr -- "https://qrcode-ai.com" "test-qr.png"
}

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Run the validator
echo "🔍 Running QRCode-AI..."
echo ""

./target/release/qrcode-ai test-qr.png

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "🚀 Fast mode (-f):"
echo ""

./target/release/qrcode-ai -f test-qr.png

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "⚡ Decode only mode (-d):"
echo ""

./target/release/qrcode-ai -d test-qr.png

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "📊 JSON output (-j):"
echo ""

./target/release/qrcode-ai -j test-qr.png | head -20
echo "  ..."

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "✅ Demo complete!"
echo ""
echo "Try it yourself:"
echo "  ./target/release/qrcode-ai <your-qr.png>"
echo "  ./target/release/qrcode-ai -f <your-qr.png>    # Fast mode"
echo "  ./target/release/qrcode-ai -t <your-qr.png>    # With timing"
echo ""
