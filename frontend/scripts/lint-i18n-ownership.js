#!/usr/bin/env node

/**
 * Lint script for i18n key-ownership validation.
 * Enforces that feature-specific keys are only used within their feature folder.
 * Global namespaces (error-*, tooltip-*, common-*, etc.) are allowed everywhere.
 */

import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const GLOBAL_NAMESPACES = ['error', 'tooltip', 'common', 'mode', 'shortcut', 'demo'];
const FEATURES_PATH = path.join(__dirname, '../src/lib/features');

function extractFeatureFromPath(filePath) {
  const match = filePath.match(/src\/lib\/features\/([^\/]+)\//);
  return match ? match[1] : null;
}

function extractI18nKeys(content) {
  const keys = [];
  // Match patterns like: i18nMsg('key-name', 'fallback')
  const regex = /i18nMsg\s*\(\s*['"`]([^'"`]+)['"`]\s*,/g;
  let match;
  while ((match = regex.exec(content)) !== null) {
    keys.push({
      key: match[1],
      position: match.index,
    });
  }
  return keys;
}

function getNamespace(key) {
  return key.split('-')[0];
}

function isGlobalNamespace(namespace) {
  return GLOBAL_NAMESPACES.includes(namespace);
}

function validateKeysInFile(filePath) {
  const content = fs.readFileSync(filePath, 'utf-8');
  const keys = extractI18nKeys(content);
  const feature = extractFeatureFromPath(filePath);

  const violations = [];

  for (const { key } of keys) {
    const namespace = getNamespace(key);

    // Skip global namespaces
    if (isGlobalNamespace(namespace)) {
      continue;
    }

    // For files in features/X, only allow keys with namespace 'X'
    if (feature && namespace !== feature) {
      violations.push({
        file: filePath.replace(process.cwd(), '.'),
        key,
        namespace,
        feature,
        message: `uses key "${key}" (${namespace} namespace) from different feature`,
      });
    }
  }

  return violations;
}

function walkDir(dir, callback) {
  const files = fs.readdirSync(dir);
  for (const file of files) {
    const fullPath = path.join(dir, file);
    const stat = fs.statSync(fullPath);
    if (stat.isDirectory()) {
      if (file !== 'node_modules' && !file.startsWith('.')) {
        walkDir(fullPath, callback);
      }
    } else if ((fullPath.endsWith('.svelte') || fullPath.endsWith('.ts')) && !fullPath.includes('node_modules')) {
      callback(fullPath);
    }
  }
}

function main() {
  const violations = [];

  // Check features folder
  if (fs.existsSync(FEATURES_PATH)) {
    walkDir(FEATURES_PATH, (file) => {
      const fileViolations = validateKeysInFile(file);
      violations.push(...fileViolations);
    });
  }

  // Report results
  if (violations.length === 0) {
    console.log('✅ lint-i18n-ownership: PASS — No cross-feature i18n violations detected');
    process.exit(0);
  } else {
    console.error(`❌ lint-i18n-ownership: FAIL — Found ${violations.length} violation(s):\n`);
    for (const violation of violations) {
      console.error(`  ${violation.file}`);
      console.error(`    ${violation.message}`);
      console.error(`    Recommendation: Move key to global namespace or feature folder\n`);
    }
    process.exit(1);
  }
}

main();
