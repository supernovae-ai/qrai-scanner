import { readFileSync } from 'fs';
import { isValid, score, summarize, validate } from './index.js';

const file = '../../test-qr-speed/OK_1103ms_100_897e5090.png';
const buffer = readFileSync(file);

console.log('=== Test OK QR (score 100 expected) ===\n');

console.log('isValid():', isValid(buffer)?.slice(0, 60) + '...');
console.log('score():', score(buffer));

const summary = summarize(buffer);
console.log('summarize():', JSON.stringify(summary, null, 2));

console.log('\n=== Full validate() ===\n');
const result = validate(buffer);
console.log('score:', result.score);
console.log('content:', result.content?.slice(0, 60) + '...');
console.log('version:', result.version);
console.log('errorCorrection:', result.errorCorrection);
console.log('decodersSuccess:', result.decodersSuccess);
console.log('stressOriginal:', result.stressOriginal);
console.log('stressDownscale50:', result.stressDownscale50);
console.log('stressBlurLight:', result.stressBlurLight);

console.log('\nNode.js integration verified!');
