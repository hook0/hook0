#!/usr/bin/env node

/**
 * Documentation validation script for Hook0
 * Checks for broken internal links and other documentation issues
 */

const fs = require('fs');
const path = require('path');
const glob = require('glob');

const DOCS_ROOT = path.join(__dirname, '..');
const LINK_PATTERN = /\[([^\]]+)\]\(([^)]+)\)/g;
const INTERNAL_LINK_PATTERN = /^(?!http|https|mailto|#)/;

let errors = 0;
let warnings = 0;

/**
 * Check if a file exists relative to the source file
 */
function checkFileExists(sourceFile, linkPath) {
  // Remove any hash fragments
  const cleanPath = linkPath.split('#')[0];
  if (!cleanPath) return true; // Hash-only links are OK
  
  const sourcDir = path.dirname(sourceFile);
  const targetPath = path.resolve(sourcDir, cleanPath);
  
  // Check if it's a markdown file
  if (!cleanPath.endsWith('.md')) {
    return true; // Non-markdown links are assumed OK for now
  }
  
  return fs.existsSync(targetPath);
}

/**
 * Validate a single markdown file
 */
function validateFile(filePath) {
  const content = fs.readFileSync(filePath, 'utf8');
  const relativePath = path.relative(DOCS_ROOT, filePath);
  let fileErrors = 0;
  
  // Check for broken internal links
  // Reset lastIndex to avoid skipping matches (global regex persists state)
  LINK_PATTERN.lastIndex = 0;
  let match;
  while ((match = LINK_PATTERN.exec(content)) !== null) {
    const linkText = match[1];
    const linkPath = match[2];
    
    if (INTERNAL_LINK_PATTERN.test(linkPath)) {
      if (!checkFileExists(filePath, linkPath)) {
        console.error(`❌ Broken link in ${relativePath}: [${linkText}](${linkPath})`);
        errors++;
        fileErrors++;
      }
    }
  }
  
  // Check for proper frontmatter
  if (!content.startsWith('---') && !content.startsWith('#')) {
    console.warn(`⚠️  Missing frontmatter in ${relativePath}`);
    warnings++;
  }
  
  // Check for TODO comments
  if (content.includes('TODO') || content.includes('FIXME')) {
    const lines = content.split('\n');
    lines.forEach((line, index) => {
      if (line.includes('TODO') || line.includes('FIXME')) {
        console.warn(`⚠️  ${relativePath}:${index + 1} - Found TODO/FIXME comment`);
        warnings++;
      }
    });
  }
  
  return fileErrors === 0;
}

/**
 * Main validation function
 */
function validateDocumentation() {
  console.log('🔍 Validating Hook0 documentation...\n');
  
  // Find all markdown files
  const files = glob.sync('**/*.md', {
    cwd: DOCS_ROOT,
    absolute: true,
    ignore: ['**/node_modules/**', '**/build/**']
  });
  
  console.log(`Found ${files.length} markdown files to validate\n`);
  
  let validFiles = 0;
  files.forEach(file => {
    if (validateFile(file)) {
      validFiles++;
    }
  });
  
  // Summary
  console.log('\n' + '='.repeat(50));
  console.log('📊 Validation Summary:');
  console.log(`✅ Valid files: ${validFiles}/${files.length}`);
  console.log(`❌ Errors: ${errors}`);
  console.log(`⚠️  Warnings: ${warnings}`);
  
  if (errors > 0) {
    console.log('\n❌ Documentation validation failed!');
    process.exit(1);
  } else if (warnings > 0) {
    console.log('\n⚠️  Documentation validation passed with warnings');
  } else {
    console.log('\n✅ Documentation validation passed!');
  }
}

// Run validation
validateDocumentation();