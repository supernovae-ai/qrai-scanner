# @supernovae/qrai-scanner

> Node.js bindings for QRAISC - High-performance QR code validation and scannability scoring

Native Rust performance in Node.js via napi-rs. Validate AI-generated and artistic QR codes that break standard scanners.

## Installation

### From npm (recommended)

```bash
npm install @supernovae/qrai-scanner
# or
yarn add @supernovae/qrai-scanner
# or
pnpm add @supernovae/qrai-scanner
```

### From GitHub

```bash
npm install github:supernovae-studio/qrai-scanner
```

### Build from source

```bash
git clone https://github.com/supernovae-studio/qrai-scanner.git
cd qrai-scanner/crates/qraisc-node
npm install
npm run build
```

### Local usage (without npm registry)

After building from source, you can use the package locally in several ways:

**Option 1: npm link (symlink)**

```bash
# In qraisc-node directory
npm link

# In your project
npm link @supernovae/qrai-scanner
```

**Option 2: npm pack (tarball)**

```bash
# In qraisc-node directory
npm pack  # Creates qrcodeai-qrai-scanner-0.1.0.tgz

# In your project
npm install /path/to/qrcodeai-qrai-scanner-0.1.0.tgz
```

**Option 3: Direct path in package.json**

```json
{
  "dependencies": {
    "@supernovae/qrai-scanner": "file:../qrai-scanner/crates/qraisc-node"
  }
}
```

**Option 4: Direct require**

```javascript
const scanner = require('/path/to/qrai-scanner/crates/qraisc-node');
```

## Quick Start

```javascript
import { readFileSync } from 'fs';
import { validate, isValid, score, summarize } from '@supernovae/qrai-scanner';

const buffer = readFileSync('qr.png');

// Simple check - is the QR valid?
const content = isValid(buffer);
if (content) {
  console.log(`QR contains: ${content}`);
}

// Get scannability score
const s = score(buffer);
console.log(`Score: ${s}/100`);

// Get a summary
const summary = summarize(buffer);
console.log(summary);
// { valid: true, score: 85, rating: 'Excellent', productionReady: true, ... }

// Full validation with stress tests
const result = validate(buffer);
console.log(`Score: ${result.score}`);
console.log(`Content: ${result.content}`);
console.log(`Stress tests:`, result.stressOriginal, result.stressDownscale50);
```

## API Reference

### Main Functions

#### `validate(buffer: Buffer): ValidationResult`

Full validation with all stress tests. Returns complete results.

```typescript
const result = validate(buffer);
// result.score: 0-100
// result.decodable: boolean
// result.content: string | null
// result.version: number | null
// result.errorCorrection: 'L' | 'M' | 'Q' | 'H' | null
// result.stressOriginal: boolean
// result.stressDownscale50: boolean
// result.stressBlurLight: boolean
// ...
```

#### `validateFast(buffer: Buffer): ValidationResult`

Fast validation with reduced stress tests. ~2x faster.

```typescript
const result = validateFast(buffer);
// Same return type, but some stress tests skipped
```

#### `decode(buffer: Buffer): DecodeResult`

Decode only, no stress tests. Fastest option.

```typescript
const result = decode(buffer);
// result.content: string
// result.version: number | null
// result.errorCorrection: 'L' | 'M' | 'Q' | 'H' | null
```

### Convenience Helpers

#### `isValid(buffer: Buffer): string | null`

Check if QR is valid. Returns content or null.

```typescript
const content = isValid(buffer);
if (content) {
  console.log(`Valid! Content: ${content}`);
} else {
  console.log('Invalid QR');
}
```

#### `score(buffer: Buffer): number`

Get scannability score (0-100).

```typescript
const s = score(buffer);
console.log(`${s}/100`);
```

#### `passesThreshold(buffer: Buffer, minScore: number): boolean`

Check if QR meets minimum score.

```typescript
if (passesThreshold(buffer, 70)) {
  console.log('Production ready!');
}
```

#### `isProductionReady(buffer: Buffer): boolean`

Check if QR is production-ready (score >= 70).

```typescript
if (isProductionReady(buffer)) {
  await uploadToProduction(buffer);
}
```

#### `summarize(buffer: Buffer): QrSummary`

Get a simple summary object.

```typescript
const summary = summarize(buffer);
// {
//   valid: true,
//   score: 85,
//   content: 'https://example.com',
//   errorCorrection: 'M',
//   rating: 'Excellent',
//   productionReady: true
// }
```

#### `getRating(score: number): string`

Convert score to human-readable rating.

```typescript
getRating(85);  // 'Excellent'
getRating(75);  // 'Good'
getRating(55);  // 'Fair'
getRating(25);  // 'Poor'
```

## TypeScript

Full TypeScript support included:

```typescript
import type {
  ValidationResult,
  DecodeResult,
  QrSummary
} from '@supernovae/qrai-scanner';

const result: ValidationResult = validate(buffer);
```

## Usage Examples

### Express.js API

```typescript
import express from 'express';
import multer from 'multer';
import { validate, isProductionReady } from '@supernovae/qrai-scanner';

const app = express();
const upload = multer();

app.post('/validate', upload.single('qr'), (req, res) => {
  const buffer = req.file.buffer;
  const result = validate(buffer);

  res.json({
    score: result.score,
    content: result.content,
    productionReady: result.score >= 70
  });
});

app.post('/check', upload.single('qr'), (req, res) => {
  const ok = isProductionReady(req.file.buffer);
  res.json({ accepted: ok });
});
```

### Batch Processing

```typescript
import { readFileSync, readdirSync } from 'fs';
import { join } from 'path';
import { score, getRating } from '@supernovae/qrai-scanner';

const qrDir = './qr-codes';
const files = readdirSync(qrDir).filter(f => f.endsWith('.png'));

for (const file of files) {
  const buffer = readFileSync(join(qrDir, file));
  const s = score(buffer);
  console.log(`${file}: ${s}/100 (${getRating(s)})`);
}
```

### Next.js API Route

```typescript
// pages/api/validate.ts
import type { NextApiRequest, NextApiResponse } from 'next';
import { validate } from '@supernovae/qrai-scanner';

export const config = {
  api: { bodyParser: false }
};

export default async function handler(req: NextApiRequest, res: NextApiResponse) {
  const chunks: Buffer[] = [];
  for await (const chunk of req) {
    chunks.push(chunk);
  }
  const buffer = Buffer.concat(chunks);

  const result = validate(buffer);
  res.json(result);
}
```

## Score Interpretation

| Score | Rating | Recommendation |
|-------|--------|----------------|
| 80-100 | Excellent | Safe for all conditions |
| 70-79 | Good | Production ready |
| 60-69 | Acceptable | May fail on older phones |
| 40-59 | Fair | Consider regenerating |
| 0-39 | Poor | Needs redesign |

## Platform Support

Pre-built binaries for:

| Platform | Architecture |
|----------|--------------|
| macOS | x64, arm64 (M1/M2) |
| Linux | x64, arm64 |
| Windows | x64 |

## Performance

| Operation | Clean QR | Artistic QR |
|-----------|----------|-------------|
| `decode()` | ~20ms | ~200ms |
| `validateFast()` | ~50ms | ~500ms |
| `validate()` | ~80ms | ~1000ms |

## License

MIT
