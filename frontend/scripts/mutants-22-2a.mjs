#!/usr/bin/env node
/**
 * Banc de mutations du socle d'appariement — Story 22-2a (#301).
 *
 * Applique une à une les mutations que `duplicate-probe.test.ts` déclare
 * attraper, relance vitest, et vérifie que chacune fait rougir **au moins un**
 * test. Une mutation qui laisse la suite verte est une preuve manquante ; une
 * mutation qui fait tomber toute la suite n'isole rien.
 *
 * Pourquoi ce script existe : trois passes de revue en prose de la Story 22-2
 * ont laissé passer des preuves qui ne discriminaient rien — `excludeSelf`
 * supprimée, la garde de soi de `findIdeHolder` retirée, les critères 2 et 3 du
 * classement neutralisés laissaient 17/17 preuves vertes. **Une mutation qui
 * n'a pas été jouée est une mutation dont on ignore le pouvoir.**
 *
 * Usage : `node scripts/mutants-22-2a.mjs` depuis `frontend/`.
 */

import { execFileSync } from 'node:child_process';
import { copyFileSync, readFileSync, unlinkSync, writeFileSync } from 'node:fs';

const MODULE = 'src/lib/features/contacts/duplicate-probe.ts';
const SPEC = 'src/lib/features/contacts/duplicate-probe.test.ts';
const BACKUP = `${MODULE}.orig`;

/** @type {{name: string, from: string, to: string, expect: string}[]} */
const MUTANTS = [
	{
		name: 'supprimer les opérateurs au lieu de les remplacer',
		from: ".replace(BOOLEAN_FT_OPERATORS, ' ')",
		to: ".replace(BOOLEAN_FT_OPERATORS, '')",
		expect: 'normalizeTerm — opérateurs'
	},
	{
		name: "omettre normalize('NFC')",
		from: "\t\t.normalize('NFC')\n",
		to: '',
		expect: 'normalizeTerm — forme Unicode'
	},
	{
		name: 'ne lire que la raison sociale',
		from: "return type === 'Personne' ? `${firstName} ${lastName}`.trim() : name.trim();",
		to: 'return name.trim();',
		expect: 'buildTerm — Personne'
	},
	{
		name: 'mesurer la longueur du terme entier',
		from: 'return Math.max(...normalized.split(/\\s+/).map((t) => t.length)) >= MIN_TOKEN_LENGTH;',
		to: 'return normalized.length >= MIN_TOKEN_LENGTH;',
		expect: 'isArmed — An Li'
	},
	{
		name: 'mesurer le seuil AVANT de normaliser',
		from: '\tconst normalized = normalizeTerm(buildTerm(type, name, firstName, lastName));\n\treturn { normalized, armed: isArmed(normalized) };',
		to: '\tconst brut = buildTerm(type, name, firstName, lastName);\n\treturn { normalized: normalizeTerm(brut), armed: isArmed(brut) };',
		expect: 'probeTerm — Yo-An et C++'
	},
	{
		name: 'replier le nom sans lui appliquer normalizeTerm',
		from: '\treturn normalizeTerm(s)\n\t\t.normalize(',
		to: '\treturn s\n\t\t.normalize(',
		expect: 'fold — symétrie / rank — trait d’union'
	},
	{
		name: 'rank = identité',
		from: '\treturn [...items].sort((a, b) => {',
		to: '\tif (true) return [...items];\n\treturn [...items].sort((a, b) => {',
		expect: 'rank — doublon exact en tête'
	},
	{
		name: 'intervertir les critères 1 et 2',
		from: '\t\t\tstartsWith(fa) - startsWith(fb) ||\n\t\t\tsharedTokens(fb) - sharedTokens(fa) ||',
		to: '\t\t\tsharedTokens(fb) - sharedTokens(fa) ||\n\t\t\tstartsWith(fa) - startsWith(fb) ||',
		expect: 'rank — critère 1 prime'
	},
	{
		name: 'retirer le critère 2',
		from: '\t\t\tsharedTokens(fb) - sharedTokens(fa) ||\n',
		to: '',
		expect: 'rank — critère 2 (doublon réordonné)'
	},
	{
		name: 'retirer le critère 3',
		from: '\t\t\tcommonPrefixLength(fb, term) - commonPrefixLength(fa, term) ||\n',
		to: '',
		expect: 'rank — critère 3'
	},
	{
		name: 'retirer le critère 4',
		from: '\t\t\t(fa < fb ? -1 : fa > fb ? 1 : 0) ||\n',
		to: '',
		expect: 'rank — critère 4'
	},
	{
		name: 'retirer le critère 5',
		from: ' ||\n\t\t\ta.id - b.id\n',
		to: '\n',
		expect: 'rank — critère 5 (homonymes)'
	},
	{
		name: 'faire filtrer rank',
		from: '\treturn [...items].sort((a, b) => {',
		to: '\treturn [...items].filter((x) => fold(x.name).startsWith(term)).sort((a, b) => {',
		expect: 'rank — classe sans filtrer'
	},
	{
		name: 'excludeSelf = identité',
		from: '\tif (editingId === null) return [...items];\n\treturn items.filter((c) => c.id !== editingId);',
		to: '\treturn [...items];',
		expect: 'excludeSelf — retire le contact édité'
	},
	{
		name: 'countOthers : total − affiches.length',
		from: '\treturn Math.max(0, total - self - affiches.length);',
		to: '\treturn Math.max(0, total - affiches.length);',
		expect: 'countOthers — édition solitaire'
	},
	{
		name: 'findIdeHolder sans la garde de soi',
		from: 'return items.find((c) => c.ideNumber === normalized && c.id !== editingId);',
		to: 'return items.find((c) => c.ideNumber === normalized);',
		expect: 'findIdeHolder — exclut le contact édité'
	},
	{
		name: 'findIdeHolder sans la garde de vacuité',
		from: "\tif (normalized === null || normalized === '') return undefined;\n",
		to: '',
		expect: 'findIdeHolder — null ne désigne personne'
	},
	{
		name: 'introduire un import réseau',
		from: "import type { ContactResponse, ContactType } from './contacts.types';",
		to: "import type { ContactResponse, ContactType } from './contacts.types';\nimport { apiClient } from '$lib/shared/utils/api-client';\nvoid apiClient;",
		expect: 'pureté — imports'
	},
	{
		name: 'appeler Date.now() sans import',
		from: 'export function normalizeTerm(raw: string): string {',
		to: 'export function normalizeTerm(raw: string): string {\n\tvoid Date.now();',
		expect: 'pureté — globales'
	}
];

function runSuite() {
	try {
		execFileSync('npx', ['vitest', 'run', SPEC, '--reporter=json', '--outputFile=.mutants.json'], {
			stdio: 'pipe'
		});
	} catch {
		/* une suite rouge fait sortir vitest en code non nul — c'est attendu */
	}
	const report = JSON.parse(readFileSync('.mutants.json', 'utf-8'));
	unlinkSync('.mutants.json');
	const failed = [];
	for (const file of report.testResults ?? []) {
		for (const t of file.assertionResults ?? []) {
			if (t.status === 'failed') failed.push(t.fullName ?? t.title);
		}
	}
	return { total: report.numTotalTests ?? 0, failed };
}

copyFileSync(MODULE, BACKUP);
const original = readFileSync(MODULE, 'utf-8');

console.log('Banc de mutations — socle d’appariement (Story 22-2a)\n');
const base = runSuite();
console.log(`Référence : ${base.total} preuves, ${base.failed.length} rouge(s).\n`);
if (base.failed.length > 0) {
	console.error('⛔ La suite n’est pas verte AVANT mutation. Rien à mesurer.');
	copyFileSync(BACKUP, MODULE);
	unlinkSync(BACKUP);
	process.exit(1);
}

let survivants = 0;
let indiscriminees = 0;

for (const m of MUTANTS) {
	if (!original.includes(m.from)) {
		console.log(`⛔ ${m.name.padEnd(48)} MOTIF INTROUVABLE — mutation non appliquée`);
		survivants++;
		continue;
	}
	writeFileSync(MODULE, original.replace(m.from, m.to));
	const { total, failed } = runSuite();
	copyFileSync(BACKUP, MODULE);

	const n = failed.length;
	const verdict = n === 0 ? '⛔ SURVIT' : n >= total ? '⚠️  n’isole rien' : `✓ ${n} rouge(s)`;
	if (n === 0) survivants++;
	if (n >= total) indiscriminees++;
	console.log(`${verdict.padEnd(18)} ${m.name}`);
	if (n > 0 && n < total) console.log(`${' '.repeat(19)}↳ ${failed.slice(0, 3).join(' · ')}`);
}

copyFileSync(BACKUP, MODULE);
unlinkSync(BACKUP);

console.log(
	`\n${MUTANTS.length} mutations jouées · ${survivants} survivante(s) · ${indiscriminees} non discriminante(s)`
);
process.exit(survivants === 0 && indiscriminees === 0 ? 0 : 1);
