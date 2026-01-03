#!/bin/bash
# Quick rename script for assets-qr directory

cd /Users/thibaut/Projects/qrai-validator
CLI="./target/release/qraisc"
DIR="assets-qr"

echo "=== RENAMING QR CODES IN $DIR ==="
echo ""

for f in "$DIR"/qrcode-ai-*.png; do
    # Extract short ID (first 8 chars of UUID)
    name=$(basename "$f" .png | sed 's/qrcode-ai-//' | cut -c1-8)

    # Time the validation
    start=$(perl -MTime::HiRes=time -e 'printf "%.0f\n", time() * 1000')
    result=$("$CLI" "$f" 2>&1)
    end=$(perl -MTime::HiRes=time -e 'printf "%.0f\n", time() * 1000')
    elapsed=$((end - start))

    # Check if success
    if echo "$result" | grep -q "SCANNABILITY"; then
        status="OK"
        score=$(echo "$result" | grep -oE 'SCORE:[[:space:]]+[0-9]+' | head -1 | grep -oE '[0-9]+')
    else
        status="FAIL"
        score="0"
    fi

    # New filename
    new_name="${status}_${elapsed}ms_${name}.png"
    new_path="$DIR/$new_name"

    # Rename
    mv "$f" "$new_path"

    if [ "$status" = "OK" ]; then
        echo "✅ $name -> $new_name (score: $score)"
    else
        echo "❌ $name -> $new_name"
    fi
done

echo ""
echo "=== DONE ==="
ls "$DIR"/*.png | wc -l | xargs -I{} echo "Total: {} images renamed"
