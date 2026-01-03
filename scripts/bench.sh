#!/bin/bash
# QRAI Validator - Benchmark Runner

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

cd "$PROJECT_DIR"

echo "╔════════════════════════════════════════════════════════════════╗"
echo "║            QRAI Validator - Performance Benchmarks             ║"
echo "╚════════════════════════════════════════════════════════════════╝"
echo ""

# Run benchmarks
cargo bench -p qrai-core

echo ""
echo "Benchmark results saved to: target/criterion/"
echo "Open target/criterion/report/index.html for detailed HTML reports"
