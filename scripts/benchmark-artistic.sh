#!/bin/bash
echo "=== ARTISTIC QR BATTERY TEST ==="
ok=0
fail=0
total_time=0

for f in test-images/*.png; do
    name=$(basename "$f" .png | sed 's/qrcode-ai-//' | cut -c1-8)
    start=$(python3 -c 'import time; print(int(time.time()*1000))')
    result=$(./target/release/qrcode-ai "$f" 2>&1)
    end=$(python3 -c 'import time; print(int(time.time()*1000))')
    elapsed=$((end - start))
    total_time=$((total_time + elapsed))

    if echo "$result" | grep -q "SCANNABILITY"; then
        score=$(echo "$result" | grep -oE 'SCORE:[[:space:]]+[0-9]+' | head -1 | grep -oE '[0-9]+')
        echo "OK $name: ${elapsed}ms (Score: $score)"
        ok=$((ok + 1))
    else
        echo "FAIL $name: ${elapsed}ms"
        fail=$((fail + 1))
    fi
done

echo ""
echo "=== SUMMARY ==="
echo "Success: $ok / $((ok + fail))"
echo "Total time: ${total_time}ms"
echo "Average: $((total_time / (ok + fail)))ms per image"
