#!/bin/bash
echo "=== FINAL BENCHMARK ==="
for f in examples/qrcodes/*.png; do
    name=$(basename "$f" .png | sed 's/qrcode-ai-//' | cut -c1-8)
    start=$(python3 -c 'import time; print(int(time.time()*1000))')
    result=$(./target/release/qraisc "$f" 2>&1)
    end=$(python3 -c 'import time; print(int(time.time()*1000))')
    elapsed=$((end - start))
    if echo "$result" | grep -q "SCANNABILITY"; then
        score=$(echo "$result" | grep -oE 'SCORE:[[:space:]]+[0-9]+' | head -1 | grep -oE '[0-9]+')
        echo "OK $name: ${elapsed}ms (Score: $score)"
    else
        echo "FAIL $name: ${elapsed}ms"
    fi
done
