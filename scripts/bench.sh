#!/bin/bash
# QRCode-AI - Benchmark Runner

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

cd "$PROJECT_DIR"

echo "╔════════════════════════════════════════════════════════════════╗"
echo "║            QRCode-AI - Performance Benchmarks             ║"
echo "╚════════════════════════════════════════════════════════════════╝"
echo ""

# Run benchmarks
cargo bench -p qrcode-ai-scanner-core

echo ""
echo "Benchmark results saved to: target/criterion/"
echo "Open target/criterion/report/index.html for detailed HTML reports"
