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
import { copyFileSync, mkdirSync, readFileSync, unlinkSync, writeFileSync } from 'node:fs';

const MODULE = 'src/lib/features/contacts/duplicate-probe.ts';
const SPEC = 'src/lib/features/contacts/duplicate-probe.test.ts';

/** Hors de `src/`, et nommée par le `pid` — cf. l'en-tête de `mutants-22-2b.mjs`. */
const TMP = 'scripts/.tmp';
const BACKUP = `${TMP}/duplicate-probe.${process.pid}.orig`;
const RAPPORT = `${TMP}/mutants-22-2a.${process.pid}.json`;
mkdirSync(TMP, { recursive: true });

/** @type {{name: string, from: string, to: string, expect: string}[]} */
const MUTANTS = [
	{
		name: 'supprimer les opérateurs au lieu de les remplacer',
		from: ".replace(BOOLEAN_FT_OPERATORS, ' ')",
		to: ".replace(BOOLEAN_FT_OPERATORS, '')",
		expect: 'remplace les opérateurs par une espace'
	},
	{
		name: 'ne pas retirer les invisibles de largeur nulle',
		from: "\t\t.replace(ZERO_WIDTH, '')\n",
		to: '',
		expect: 'retire les invisibles de LARGEUR NULLE'
	},
	{
		name: 'replier sans passer par NFKC',
		from: "\t\t.normalize('NFKC')\n",
		to: '',
		expect: 'replie les formes de COMPATIBILITÉ'
	},
	{
		name: 'buildTerm : interpoler sans garde',
		from: '\tconst sain = (v: string) => (typeof v === \'string\' ? v : \'\');',
		to: '\tconst sain = (v: string) => v;',
		expect: 'ne fabrique JAMAIS le mot'
	},
	{
		name: 'isArmed : Math.max(...spread)',
		from: "\treturn normalized.split(/\\s+/).some((t) => t.length >= MIN_TOKEN_LENGTH);",
		to: '\treturn Math.max(...normalized.split(/\\s+/).map((t) => t.length)) >= MIN_TOKEN_LENGTH;',
		expect: 'DÉBORDER LA PILE'
	},
	{
		name: 'countOthers : retirer la garde Number.isFinite',
		from: '\tif (!Number.isFinite(total)) return 0;\n',
		to: '',
		expect: 'total non fini'
	},
	{
		name: "omettre normalize('NFC')",
		from: "\t\t.normalize('NFC')\n",
		to: '',
		expect: 'stabilise la forme Unicode'
	},
	{
		name: 'ne lire que la raison sociale',
		from: '\t\t? `${sain(firstName)} ${sain(lastName)}`.trim()\n\t\t: sain(name).trim();',
		to: '\t\t? sain(name).trim()\n\t\t: sain(name).trim();',
		expect: 'Personne : prénom + nom'
	},
	{
		name: 'mesurer la longueur du terme entier',
		from: 'return normalized.split(/\\s+/).some((t) => t.length >= MIN_TOKEN_LENGTH);',
		to: 'return normalized.length >= MIN_TOKEN_LENGTH;',
		expect: 'An Li'
	},
	{
		name: 'mesurer le seuil AVANT de normaliser',
		from: '\tconst normalized = normalizeTerm(buildTerm(type, name, firstName, lastName));\n\treturn { normalized, armed: isArmed(normalized) };',
		to: '\tconst brut = buildTerm(type, name, firstName, lastName);\n\treturn { normalized: normalizeTerm(brut), armed: isArmed(brut) };',
		expect: 'Yo-An'
	},
	{
		name: 'replier le nom sans lui appliquer normalizeTerm',
		from: '\treturn normalizeTerm(s)\n\t\t.normalize(',
		to: '\treturn s\n\t\t.normalize(',
		expect: 'TRAIT D’UNION'
	},
	{
		name: 'rank = identité',
		from: '\treturn [...items].sort((a, b) => {',
		to: '\tif (true) return [...items];\n\treturn [...items].sort((a, b) => {',
		expect: 'le doublon exact sort EN TÊTE'
	},
	{
		name: 'intervertir les critères 1 et 2',
		from: '\t\t\tstartsWith(fa) - startsWith(fb) ||\n\t\t\tsharedTokens(fb) - sharedTokens(fa) ||',
		to: '\t\t\tsharedTokens(fb) - sharedTokens(fa) ||\n\t\t\tstartsWith(fa) - startsWith(fb) ||',
		expect: 'critère 1 : commencer par le terme'
	},
	{
		name: 'retirer le critère 2',
		from: '\t\t\tsharedTokens(fb) - sharedTokens(fa) ||\n',
		to: '',
		expect: 'critère 2 : ramène le doublon RÉORDONNÉ'
	},
	{
		name: 'retirer le critère 3',
		from: '\t\t\tcommonPrefixLength(fb, term) - commonPrefixLength(fa, term) ||\n',
		to: '',
		expect: 'critère 3 : le plus long préfixe commun'
	},
	{
		name: 'retirer le critère 4',
		from: '\t\t\t(fa < fb ? -1 : fa > fb ? 1 : 0) ||\n',
		to: '',
		expect: 'critère 4 : l’alphabétique tranche'
	},
	{
		name: 'retirer le critère 5',
		from: ' ||\n\t\t\ta.id - b.id\n',
		to: '\n',
		expect: 'critère 5 : deux HOMONYMES STRICTS'
	},
	{
		name: "retirer la garde ?? '' de rank",
		from: "\t\tconst fa = fold(a.name ?? '');\n\t\tconst fb = fold(b.name ?? '');",
		to: '\t\tconst fa = fold(a.name);\n\t\tconst fb = fold(b.name);',
		expect: 'ne fait pas exploser le classement'
	},
	{
		name: 'faire filtrer rank',
		from: '\treturn [...items].sort((a, b) => {',
		to: '\treturn [...items].filter((x) => fold(x.name).startsWith(term)).sort((a, b) => {',
		expect: 'classe sans filtrer ni dupliquer'
	},
	{
		name: 'excludeSelf = identité',
		from: '\tif (editingId === null) return [...items];\n\treturn items.filter((c) => c.id !== editingId);',
		to: '\treturn [...items];',
		expect: 'retire le contact édité, et lui seul'
	},
	{
		name: 'countOthers : total − affiches.length',
		from: '\treturn Math.max(0, total - self - affiches.length);',
		to: '\treturn Math.max(0, total - affiches.length);',
		expect: 'édition solitaire'
	},
	{
		name: 'findIdeHolder sans la garde de soi',
		from: 'return items.find((c) => c.ideNumber === normalized && c.id !== editingId);',
		to: 'return items.find((c) => c.ideNumber === normalized);',
		expect: 'exclut le contact édité — le signal FRANC'
	},
	{
		name: 'findIdeHolder sans la garde de vacuité',
		from: "\tif (normalized === null || normalized === '') return undefined;\n",
		to: '',
		expect: 'null ne désigne personne'
	},
	{
		name: 'introduire un import réseau',
		from: "import type { ContactResponse, ContactType } from './contacts.types';",
		to: "import type { ContactResponse, ContactType } from './contacts.types';\nimport { apiClient } from '$lib/shared/utils/api-client';\nvoid apiClient;",
		expect: 'n’importe rien qui touche au réseau'
	},
	{
		name: 'appeler Date.now() sans import',
		from: 'export function normalizeTerm(raw: string): string {',
		to: 'export function normalizeTerm(raw: string): string {\n\tvoid Date.now();',
		expect: 'n’appelle aucune globale'
	},
	{
		name: 'describeProches : cascade au lieu d’invariant (le code LIVRÉ)',
		from: '\t\treturn (effectif.get(lignes[i]) ?? 0) > 1\n\t\t\t? ` — ${bases[i]} · #${c.id}`\n\t\t\t: ` — ${bases[i]}`;',
		to: '\t\treturn ` — ${bases[i]}`;',
		expect: 'partagent une VILLE NON VIDE'
	},
	{
		name: 'describeProches : coller l’id systématiquement',
		from: '\t\treturn (effectif.get(lignes[i]) ?? 0) > 1\n\t\t\t? ` — ${bases[i]} · #${c.id}`\n\t\t\t: ` — ${bases[i]}`;',
		to: '\t\treturn ` — ${bases[i]} · #${c.id}`;',
		expect: 'ne colle PAS d’id'
	},
	{
		name: 'describeProches : ne comparer que le descripteur, sans le nom',
		from: "JSON.stringify([c.name ?? '', bases[i]])",
		to: 'bases[i]',
		expect: 'LIGNE ENTIÈRE'
	},
	{
		name: 'describeProches : pousser l’email même quand la ville est là',
		from: 'if (bouts.length === 0 && c.email) bouts.push(c.email);',
		to: 'if (c.email) bouts.push(c.email);',
		expect: 'la cascade reste celle d’avant'
	},
	{
		name: 'describeProches : juger la vacuité sans rogner les espaces',
		from: "(v): v is string => typeof v === 'string' && v.trim() !== ''",
		to: "(v): v is string => typeof v === 'string' && v !== ''",
		expect: 'ESPACES ne compte pas'
	},
	{
		name: 'BH-3 : seuil d’armement porté de 3 à 4',
		from: 't.length >= MIN_TOKEN_LENGTH',
		to: 't.length > MIN_TOKEN_LENGTH',
		expect: 'mesure le plus long token'
	},
	{
		name: 'BH-3 : retirer le Set de déduplication des tokens du terme',
		from: 'const termTokens = [...new Set(term.split(/\\s+/).filter(Boolean))];',
		to: 'const termTokens = term.split(/\\s+/).filter(Boolean);',
		expect: 'un token RÉPÉTÉ'
	},
	{
		name: 'BH-6 : retirer le strip des diacritiques combinants',
		from: "\t\t.replace(/[\\u0300-\\u036f]/g, '')",
		to: '',
		expect: 'quel que soit l’ordre'
	}
];

function runSuite() {
	try {
		execFileSync('npx', ['vitest', 'run', SPEC, '--reporter=json', `--outputFile=${RAPPORT}`], {
			stdio: 'pipe'
		});
	} catch {
		/* une suite rouge fait sortir vitest en code non nul — c'est attendu */
	}
	const report = JSON.parse(readFileSync(RAPPORT, 'utf-8'));
	unlinkSync(RAPPORT);
	const failed = [];
	for (const file of report.testResults ?? []) {
		for (const t of file.assertionResults ?? []) {
			if (t.status === 'failed') failed.push(t.fullName ?? t.title);
		}
	}
	return { total: report.numTotalTests ?? 0, failed };
}

// ⚠️ **Un trou dans le tableau plante le banc À MI-PARCOURS**, après avoir laissé
// le fichier muté — c'est arrivé, par une virgule en double qu'aucune relecture
// n'attrape (`},,` crée un élément `undefined`). Ce contrôle est le premier geste
// du script : il échoue AVANT la première copie de sauvegarde, donc sans rien
// laisser derrière lui.
for (const [i, m] of MUTANTS.entries()) {
	if (!m || !m.name || m.from === undefined || m.to === undefined || !m.expect) {
		console.error(`⛔ Entrée ${i} du tableau MUTANTS malformée ou absente (virgule en double ?).`);
		process.exit(2);
	}
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
let horsCible = 0;

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
	// ⚠️ Le champ `expect` est ASSERTÉ, pas affiché. Une mutation qui fait
	// rougir une preuve n'établit rien si ce n'est pas CELLE qu'elle annonce :
	// c'est la différence entre « la suite réagit » et « cette preuve-ci garde
	// ce défaut-là ».
	const viseJuste = failed.some((t) => t.includes(m.expect));
	const verdict =
		n === 0
			? '⛔ SURVIT'
			: n >= total
				? '⚠️  n’isole rien'
				: viseJuste
					? `✓ ${n} rouge(s)`
					: '⛔ RATE SA CIBLE';
	if (n === 0) survivants++;
	if (n >= total) indiscriminees++;
	if (n > 0 && n < total && !viseJuste) horsCible++;
	console.log(`${verdict.padEnd(18)} ${m.name}`);
	if (!viseJuste && n > 0) console.log(`${' '.repeat(19)}↳ attendait « ${m.expect} »`);
	if (n > 0 && n < total) console.log(`${' '.repeat(19)}↳ ${failed.slice(0, 3).join(' · ')}`);
}

copyFileSync(BACKUP, MODULE);
unlinkSync(BACKUP);

console.log(
	`\n${MUTANTS.length} mutations jouées · ${survivants} survivante(s) · ${indiscriminees} non discriminante(s) · ${horsCible} hors cible`
);
process.exit(survivants === 0 && indiscriminees === 0 && horsCible === 0 ? 0 : 1);
