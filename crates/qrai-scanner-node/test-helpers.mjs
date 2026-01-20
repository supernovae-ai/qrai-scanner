import { readFileSync, readdirSync } from 'fs';
import { join } from 'path';
import {
  validate, decode, isValid, score,
  passesThreshold, isProductionReady, summarize, getRating
} from './index.js';

const testDir = '../../test-qr-speed';
const files = readdirSync(testDir).filter(f => f.endsWith('.png')).slice(0, 5);

console.log('=== QRAISC Node.js Helpers Test ===\n');

for (const file of files) {
  const path = join(testDir, file);
  const buffer = readFileSync(path);

  console.log('File: ' + file);

  // Test isValid
  const content = isValid(buffer);
  console.log('  isValid(): ' + (content ? content.slice(0, 50) + '...' : 'null'));

  // Test score
  const s = score(buffer);
  console.log('  score(): ' + s + '/100');

  // Test getRating
  console.log('  getRating(' + s + '): ' + getRating(s));

  // Test passesThreshold
  console.log('  passesThreshold(70): ' + passesThreshold(buffer, 70));

  // Test isProductionReady
  console.log('  isProductionReady(): ' + isProductionReady(buffer));

  // Test summarize
  const summary = summarize(buffer);
  console.log('  summarize(): valid=' + summary.valid + ', score=' + summary.score + ', rating=' + summary.rating);

  console.log('');
}

console.log('All helpers working!');
