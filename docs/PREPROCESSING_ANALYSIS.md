# Preprocessing Parameter Analysis for Slow Artistic QR Codes

## Test Images Visual Analysis

| ID | Description | Background | Finder Patterns | Difficulty |
|----|-------------|------------|-----------------|------------|
| 3eb25154 | Space cat astronaut | Multi-color gradient, stars | Gradient/transparent | VERY HARD (2/2240) |
| ff06edb3 | Iron Man | Dark/black | Orange/red colored | MEDIUM (596/2240) |
| d56ef35e | F1 Racing cars | Dark with motion blur | White + orange mixed | MEDIUM (743/2240) |
| 14f79efe | Colorful bird | Light/cream | Strong black | EASIER (720/2240) |

## Executive Summary

Benchmark testing of 2240 parameter combinations on 4 slow artistic QR codes (1500-1800ms decode time) reveals **dramatically suboptimal parameters** in the current decoder.

### Key Finding: Current Parameters Are Wrong

The decoder's `known_good_params` use:
- **Size: 300-500** (suboptimal - 200-250 is 5-10x faster)
- **Contrast: 2.0-4.0** (suboptimal - 1.0-1.5 has higher success rate)
- **Blur: 0-1.5** (correct - but 1.0-1.5 much better than 0)

## Benchmark Results

### Optimal Parameters Per Image

| Image ID | Size | Contrast | Brightness | Blur | Grayscale | Decode Time |
|----------|------|----------|------------|------|-----------|-------------|
| ff06edb3 | 200 | 1.0 | 0.8 | 1.0 | Y | **1ms** |
| d56ef35e | 200 | 1.0 | 0.8 | 0.0 | N | **1ms** |
| 3eb25154 | 250 | 1.0 | 0.9 | 0.0 | N | **10ms** |
| 14f79efe | 200 | 1.0 | 0.8 | 1.0 | Y | **7ms** |

Compare to current decoder: **1500-1800ms** --> optimal: **1-10ms**

### Cross-Image Success Rates

#### Size (smaller is better!)
| Size | Success Rate | Avg Decode Time |
|------|--------------|-----------------|
| 200 | 38.7% | 3ms |
| 250 | 37.6% | 3ms |
| 300 | 29.5% | 6ms |
| 350 | 21.4% | 9ms |
| 400 | 22.2% | 10ms |
| 450 | 17.0% | 18ms |
| 500 | 14.9% | 29ms |
| none | 2.8% | 93ms |

**Insight**: Size=200 has the highest success rate AND fastest decode time. The current decoder uses 300-500 which is significantly worse.

#### Contrast (lower is better!)
| Contrast | Success Rate | Avg Decode Time |
|----------|--------------|-----------------|
| 1.0 | 31.1% | 6ms |
| 1.5 | 28.8% | 8ms |
| 2.0 | 22.8% | 10ms |
| 2.5 | 21.6% | 11ms |
| 3.0 | 19.7% | 12ms |
| 3.5 | 19.0% | 13ms |
| 4.0 | 18.0% | 13ms |

**Insight**: Lower contrast (1.0-1.5) is MORE effective than high contrast. The current decoder uses 2.0-4.0 which is counterproductive.

#### Blur (more is better!)
| Blur | Success Rate |
|------|--------------|
| 0.0 | 5.1% (116/2240) |
| 0.5 | 10.3% (229/2240) |
| 1.0 | 34.5% (716/2240) |
| 1.5 | 50.0% (999/2240) |

**Insight**: Higher blur (1.0-1.5) dramatically improves success rate. Artistic QRs have noise/artifacts that blur helps smooth out.

#### Brightness
| Brightness | Success Rate |
|------------|--------------|
| 0.8 | 7.1% |
| 0.9 | 11.9% |
| 1.0 | 19.1% |
| 1.1 | 25.3% |
| 1.2 | 28.5% |

**Insight**: Slightly brighter (1.1-1.2) helps. Artistic QRs often have darker foregrounds.

## Recommended New Parameters

### Tier 1: Fast Path (should be tried first)
```rust
PreprocessParams { resize: 200, contrast: 1.0, brightness: 0.8, blur: 1.0, grayscale: true },
PreprocessParams { resize: 200, contrast: 1.0, brightness: 1.0, blur: 1.5, grayscale: true },
PreprocessParams { resize: 250, contrast: 1.0, brightness: 1.1, blur: 1.0, grayscale: true },
PreprocessParams { resize: 200, contrast: 1.5, brightness: 1.2, blur: 1.0, grayscale: false },
```

### Tier 2: Medium (if Tier 1 fails)
```rust
PreprocessParams { resize: 300, contrast: 1.5, brightness: 1.1, blur: 1.5, grayscale: true },
PreprocessParams { resize: 250, contrast: 2.0, brightness: 1.2, blur: 1.0, grayscale: true },
PreprocessParams { resize: 300, contrast: 1.0, brightness: 1.0, blur: 1.0, grayscale: false },
PreprocessParams { resize: 350, contrast: 1.5, brightness: 1.1, blur: 1.5, grayscale: true },
```

### Tier 3: Stubborn (for difficult images like 3eb25154)
```rust
PreprocessParams { resize: 250, contrast: 1.0, brightness: 0.9, blur: 0.0, grayscale: false },
PreprocessParams { resize: 400, contrast: 4.0, brightness: 1.2, blur: 0.0, grayscale: true },
```

## Why Current Parameters Are Slow

1. **Over-processing**: Large resize (400-500) creates more pixels to decode
2. **Wrong contrast strategy**: High contrast (2.0-4.0) destroys subtle gradients in artistic QRs
3. **Insufficient blur**: Low blur (0-0.5) doesn't smooth noise/artifacts
4. **Wrong size priority**: Current params test larger sizes first when smaller is better

## Performance Impact

| Approach | Decode Time |
|----------|-------------|
| Current decoder | 1500-1800ms |
| Optimal params | 1-10ms |
| Speedup | **150-1800x** |

## Special Case: 3eb25154

This image only decoded with 2/2240 combinations:
1. size=250, contrast=1.0, brightness=0.9, blur=0.0, grayscale=false (10ms)
2. size=400, contrast=4.0, brightness=1.2, blur=0.0, grayscale=true (48ms)

This is a particularly difficult artistic QR that requires specific preprocessing. The image likely has:
- Fine detail that blur destroys
- Color information critical for decode (grayscale=false works)
- Very specific luminance balance

## Recommendations

1. **Reorder known_good_params**: Put size=200 combos FIRST
2. **Add blur variants**: Include blur=1.0 and blur=1.5 in early tiers
3. **Lower contrast defaults**: Start with contrast=1.0, escalate to higher values
4. **Keep color option early**: Some artistic QRs decode better with grayscale=false
5. **Add specific fallback for stubborn images**: The 3eb25154 pattern (size=250, no blur, no grayscale)
