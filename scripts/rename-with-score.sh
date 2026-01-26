#!/bin/bash
# Don't use set -e since some QR codes fail to validate

DIR="test-qr-speed"
CLI="./target/release/qrcode-ai"

echo "=== RENAMING WITH SCORE ==="
echo ""

for f in "$DIR"/*.png; do
    base=$(basename "$f" .png)
    id=$(echo "$base" | grep -oE '[a-f0-9]{8}$')

    start=$(perl -MTime::HiRes=time -e 'printf "%.0f\n", time() * 1000')
    result=$("$CLI" "$f" 2>&1)
    end=$(perl -MTime::HiRes=time -e 'printf "%.0f\n", time() * 1000')
    elapsed=$((end - start))

    if echo "$result" | grep -q "SCANNABILITY"; then
        stat="OK"
        score=$(echo "$result" | grep -oE 'SCORE:[[:space:]]+[0-9]+' | head -1 | grep -oE '[0-9]+')
    else
        stat="FAIL"
        score="0"
    fi

    new_name="${stat}_${elapsed}ms_${score}_${id}.png"
    mv "$f" "$DIR/$new_name"

    if [ "$stat" = "OK" ]; then
        echo "✅ $id -> $new_name"
    else
        echo "❌ $id -> $new_name"
    fi
done

echo ""
echo "=== DONE ==="
ls "$DIR"/*.png | wc -l | xargs echo "Total images:"
