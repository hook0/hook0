#!/usr/bin/env node
/**
 * Image optimization script using Sharp
 * Converts PNG images to WebP format for better performance
 */

const sharp = require('sharp');
const path = require('path');
const fs = require('fs');

const IMAGES_TO_CONVERT = [
  {
    input: 'src/mediakit/logo/400x400.png',
    output: 'src/mediakit/logo/400x400.webp',
    quality: 85
  },
  {
    input: 'src/mediakit/logo/256x256.png',
    output: 'src/mediakit/logo/256x256.webp',
    quality: 85
  },
  {
    input: 'src/mediakit/logo/512x512.png',
    output: 'src/mediakit/logo/512x512.webp',
    quality: 85
  }
];

const rootDir = path.join(__dirname, '..');

console.log('🖼️  Starting image optimization...\n');

const promises = IMAGES_TO_CONVERT.map(({ input, output, quality }) => {
  const inputPath = path.join(rootDir, input);
  const outputPath = path.join(rootDir, output);

  if (!fs.existsSync(inputPath)) {
    console.log(`⚠️  Skipping ${input} (file not found)`);
    return Promise.resolve();
  }

  return sharp(inputPath)
    .webp({ quality })
    .toFile(outputPath)
    .then((info) => {
      const inputSize = fs.statSync(inputPath).size;
      const savings = Math.round((1 - info.size / inputSize) * 100);
      console.log(`✅ ${input} → ${output}`);
      console.log(`   Size: ${Math.round(inputSize / 1024)}KB → ${Math.round(info.size / 1024)}KB (${savings}% smaller)\n`);
    })
    .catch((err) => {
      console.error(`❌ Error converting ${input}:`, err.message);
    });
});

Promise.all(promises)
  .then(() => {
    console.log('🎉 Image optimization complete!');
  })
  .catch((err) => {
    console.error('Error during optimization:', err);
    process.exit(1);
  });
