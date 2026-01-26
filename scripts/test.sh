#!/bin/bash
# QRCode-AI - Test Runner Script

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

cd "$PROJECT_DIR"

echo "╔════════════════════════════════════════════════════════════════╗"
echo "║              QRCode-AI - Test Suite                       ║"
echo "╚════════════════════════════════════════════════════════════════╝"
echo ""

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Step 1: Format check
echo -e "${YELLOW}[1/5] Checking code format...${NC}"
if cargo fmt --all -- --check; then
    echo -e "${GREEN}✓ Format OK${NC}"
else
    echo -e "${RED}✗ Format issues found. Run 'cargo fmt --all' to fix.${NC}"
    exit 1
fi
echo ""

# Step 2: Clippy lint
echo -e "${YELLOW}[2/5] Running clippy lints...${NC}"
if cargo clippy --workspace --all-targets -- -D warnings; then
    echo -e "${GREEN}✓ Clippy OK${NC}"
else
    echo -e "${RED}✗ Clippy found issues${NC}"
    exit 1
fi
echo ""

# Step 3: Unit tests
echo -e "${YELLOW}[3/5] Running unit tests...${NC}"
if cargo test --workspace; then
    echo -e "${GREEN}✓ All tests passed${NC}"
else
    echo -e "${RED}✗ Tests failed${NC}"
    exit 1
fi
echo ""

# Step 4: Doc tests
echo -e "${YELLOW}[4/5] Running doc tests...${NC}"
if cargo test --workspace --doc; then
    echo -e "${GREEN}✓ Doc tests passed${NC}"
else
    echo -e "${RED}✗ Doc tests failed${NC}"
    exit 1
fi
echo ""

# Step 5: Build release
echo -e "${YELLOW}[5/5] Building release binary...${NC}"
if cargo build -p qrcode-ai-scanner-cli --release; then
    echo -e "${GREEN}✓ Release build OK${NC}"
else
    echo -e "${RED}✗ Release build failed${NC}"
    exit 1
fi
echo ""

echo "╔════════════════════════════════════════════════════════════════╗"
echo "║                    All checks passed! ✓                        ║"
echo "╚════════════════════════════════════════════════════════════════╝"
echo ""
echo "Binary location: target/release/qrcode-ai"
echo ""
echo "Usage:"
echo "  ./target/release/qrcode-ai <image.png>      # Full validation"
echo "  ./target/release/qrcode-ai -s <image.png>   # Score only"
echo "  ./target/release/qrcode-ai -d <image.png>   # Decode only (fast)"
