#!/usr/bin/env node

/**
 * Audit E2E selectors for brittleness
 * Scans for getByText(), getByLabel(), getByRole() without data-testid fallback
 * 
 * Usage: node scripts/audit-e2e-selectors.js
 */

import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const SPEC_DIR = path.join(__dirname, '../tests/e2e');

function findSpecFiles(dir) {
	const files = [];
	const items = fs.readdirSync(dir);
	items.forEach(item => {
		const fullPath = path.join(dir, item);
		if (fs.statSync(fullPath).isFile() && item.endsWith('.spec.ts')) {
			files.push(fullPath);
		}
	});
	return files;
}

const findings = [];

// Scan all .spec.ts files
const specFiles = findSpecFiles(SPEC_DIR);

specFiles.forEach(file => {
	const content = fs.readFileSync(file, 'utf-8');
	const lines = content.split('\n');

	lines.forEach((line, idx) => {
		const lineNum = idx + 1;

		// Skip if already has data-testid fallback on same line
		if (line.includes('data-testid')) {
			return;
		}

		// Check for getByText pattern
		const getByTextMatches = line.match(/getByText\(['"`][^'"`]+['"`]\)/g) || [];
		getByTextMatches.forEach(match => {
			findings.push({
				file: path.relative(SPEC_DIR, file),
				line: lineNum,
				selector: match,
				type: 'getByText',
				priority: 'HIGH',
			});
		});

		// Check for getByLabel pattern
		const getByLabelMatches = line.match(/getByLabel\(['"`][^'"`]+['"`]\)/g) || [];
		getByLabelMatches.forEach(match => {
			findings.push({
				file: path.relative(SPEC_DIR, file),
				line: lineNum,
				selector: match,
				type: 'getByLabel',
				priority: 'MEDIUM',
			});
		});

		// Check for getByRole pattern
		const getByRoleMatches = line.match(/getByRole\(['"`][^'"`]+['"`][^)]*\)/g) || [];
		getByRoleMatches.forEach(match => {
			findings.push({
				file: path.relative(SPEC_DIR, file),
				line: lineNum,
				selector: match.substring(0, 60) + (match.length > 60 ? '...' : ''),
				type: 'getByRole',
				priority: 'MEDIUM',
			});
		});
	});
});

// Sort by priority and file
const priorityOrder = { HIGH: 0, MEDIUM: 1, LOW: 2 };
findings.sort((a, b) => {
	if (priorityOrder[a.priority] !== priorityOrder[b.priority]) {
		return priorityOrder[a.priority] - priorityOrder[b.priority];
	}
	if (a.file !== b.file) {
		return a.file.localeCompare(b.file);
	}
	return a.line - b.line;
});

// Report
console.log('\n📋 E2E Selector Brittleness Audit\n');
console.log(`Total brittle selectors found: ${findings.length}\n`);

if (findings.length === 0) {
	console.log('✅ No brittle selectors detected!\n');
	process.exit(0);
}

// Group by type
const byType = {};
findings.forEach(f => {
	if (!byType[f.type]) byType[f.type] = [];
	byType[f.type].push(f);
});

Object.entries(byType).forEach(([type, items]) => {
	if (items.length === 0) return;
	const icons = { getByText: '🔴', getByLabel: '🟡', getByRole: '🟡' };
	console.log(`\n${icons[type] || '•'} ${type.toUpperCase()} (${items.length})`);
	items.forEach(f => {
		console.log(`  ${f.file}:${f.line}`);
		console.log(`    ${f.selector}`);
	});
});

console.log('\n📊 Refactoring Plan:\n');
console.log('1. Start with getByText patterns (brittle, impact on copy changes)');
console.log('2. Move to getByLabel and getByRole (medium impact)');
console.log('3. Validate with: npx playwright test --reporter=line\n');

process.exit(findings.length > 0 ? 1 : 0);
