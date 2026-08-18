#!/usr/bin/env node
/**
 * Banc de mutations du câblage des sondes — Story 22-2b (#301).
 *
 * Même office que `mutants-22-2a.mjs`, pour la SURFACE.
 *
 * ⚠️ **TROIS preuves de `+page.test.ts` ne prouvaient RIEN avant d'être jouées
 * ici**, et aucune relecture ne les aurait vues : ouvrir une fiche en édition
 * sans retaper l'IDE ne déclenche aucune sonde, donc le test passait pour la
 * mauvaise raison ; le compteur « et N autres » n'est pas rendu quand la liste
 * est vide, donc sa mutation restait invisible ; et une promesse rejetée sans
 * `try/catch` ne fait pas rougir vitest — il fallait afficher AVANT d'échouer.
 *
 * Usage : `node scripts/mutants-22-2b.mjs` depuis `frontend/`.
 */

import { execFileSync } from 'node:child_process';
import { copyFileSync, mkdirSync, readFileSync, unlinkSync, writeFileSync } from 'node:fs';

const MODULE = 'src/routes/(app)/contacts/+page.svelte';
const SPEC = 'src/routes/(app)/contacts/contacts-page.test.ts';

/**
 * ⚠️ La sauvegarde vit **hors de `src/`**, et porte le `pid` dans son nom.
 *
 * Deux raisons, toutes deux payées : un `.orig` déposé dans `src/routes/` est vu
 * par SvelteKit comme un fichier de route et pollue le graphe de Vite ; et deux
 * bancs lancés en même temps sur le même arbre — ce qui arrive dès qu'on met
 * plusieurs relecteurs en parallèle — se détruisaient mutuellement leur
 * sauvegarde, l'un supprimant le `.orig` que l'autre s'apprêtait à restaurer.
 * Le symptôme était un `ENOENT` au premier restore, faussement imputé au script.
 */
const TMP = 'scripts/.tmp';
const BACKUP = `${TMP}/page.svelte.${process.pid}.orig`;
const RAPPORT = `${TMP}/mutants-22-2b.${process.pid}.json`;
mkdirSync(TMP, { recursive: true });

/** @type {{name: string, from: string, to: string, expect: string}[]} */
const MUTANTS = [
	{
		name: "terme BRUT au lieu de normalisé",
		from: "await listContacts({ search: normalized, limit: 20, includeArchived: false })",
		to: "await listContacts({ search: formName, limit: 20, includeArchived: false })",
		expect: "l’argument de LA SONDE"
	},
	{
		name: "sonde nom sur les ARCHIVÉS",
		from: "search: normalized, limit: 20, includeArchived: false",
		to: "search: normalized, limit: 20, includeArchived: true",
		expect: "l’argument de LA SONDE"
	},
	{
		name: "retirer rank(...)",
		from: "const retenus = rank(excludeSelf(rn.items, soi), normalized);",
		to: "const retenus = excludeSelf(rn.items, soi);",
		expect: "classé, coupé à CINQ, et compté"
	},
	{
		name: "retirer .slice(0, 5)",
		from: "proches = retenus.slice(0, 5);",
		to: "proches = retenus;",
		expect: "classé, coupé à CINQ, et compté"
	},
	{
		name: "désarmement du nom sans effacement",
		from: "\t\t\tnameSeq++;\n\t\t\tproches = [];\n\t\t\tautres = 0;\n\t\t\treturn;",
		to: "\t\t\treturn;",
		expect: "la bascule de type RÉARME"
	},
	{
		name: "désarmement de l’IDE sans effacement",
		from: "\t\t\tideSeq++;\n\t\t\tideHolder = null;\n\t\t\treturn;",
		to: "\t\t\treturn;",
		expect: "vider le champ IDE"
	},
	{
		name: "excludeSelf sans le contact édité",
		from: "const retenus = rank(excludeSelf(rn.items, soi), normalized);",
		to: "const retenus = rank(rn.items, normalized);",
		expect: "le contact édité ne figure pas"
	},
	{
		name: "findIdeHolder sans la garde de soi",
		from: "ideHolder = findIdeHolder(ri.items, ide, soi) ?? null;",
		to: "ideHolder = ri.items.find((x) => x.ideNumber === ide) ?? null;",
		expect: "RETAPER son propre IDE"
	},
	{
		name: "countOthers sans le contact édité",
		from: "autres = countOthers(rn.total, rn.items, proches, soi);",
		to: "autres = Math.max(0, rn.total - proches.length);",
		expect: "le compteur « et N autres » SOUSTRAIT"
	},
	{
		name: "sonde branchée sur le clavier",
		from: "oninput={scheduleNameProbe}\n\t\t\t\t\t\tmaxlength={255}",
		to: "onkeydown={scheduleNameProbe}\n\t\t\t\t\t\tmaxlength={255}",
		expect: "l’argument de LA SONDE"
	},
	{
		name: "le <select> de type ne réarme plus",
		from: "\t\t\t\t\tonchange={scheduleNameProbe}\n",
		to: "",
		expect: "la bascule de type RÉARME"
	},
	{
		name: "try/catch inopérant sur la sonde IDE",
		from: "\t\t\tif (seq === ideSeq) ideHolder = null;",
		to: "\t\t\tvoid seq;",
		expect: "une sonde qui ÉCHOUE efface"
	},
	{
		name: "AC-b4 : ne pas annuler la minuterie précédente",
		from: "\t\tif (nameProbeHandle) clearTimeout(nameProbeHandle);\n\t\tnameProbeHandle = setTimeout(runNameProbe, PROBE_DELAY_MS);",
		to: "\t\tnameProbeHandle = setTimeout(runNameProbe, PROBE_DELAY_MS);",
		expect: "preuve 1 : vingt caractères"
	},
	{
		name: "AC-b4 : retirer la garde d’ordre de la sonde nom",
		from: "\t\t\tif (seq !== nameSeq) return;",
		to: "",
		expect: "preuve 2 : deux réponses résolues"
	},
	{
		name: "AC-b4 : une seule paire (minuterie, compteur) pour les deux sondes",
		from: "\t\tconst seq = ++ideSeq;",
		to: "\t\tconst seq = ++nameSeq;",
		expect: "preuve 3 : le test CROISÉ"
	},
	{
		name: "AC-b4 : sonder l’IDE sans vérifier son format",
		from: "if (!ide || !validateIdeFormat(formIde)) {",
		to: "if (!ide) {",
		expect: "preuve 4 : valide → invalide → valide"
	},
	{
		name: "ECH-3 : fermer le formulaire ne fait plus taire les sondes",
		from: "\t$effect(() => {\n\t\tif (!formOpen) resetProbes();\n\t});",
		to: "",
		expect: "rouvrir en CRÉATION"
	},
	{
		name: "BH-1 : try/catch inopérant sur la sonde NOM",
		from: "\t\t\tif (seq === nameSeq) {\n\t\t\t\tproches = [];\n\t\t\t\tautres = 0;\n\t\t\t}",
		to: "\t\t\tvoid seq;",
		expect: "la sonde NOM qui échoue"
	},
	{
		name: "BH-2 : le champ PRÉNOM ne réarme plus la sonde",
		from: "bind:value={formFirstName}\n\t\t\t\t\t\t\toninput={scheduleNameProbe}",
		to: "bind:value={formFirstName}",
		expect: "preuve 2-bis : le champ PRÉNOM"
	},
	{
		name: "BH-2 : le champ NOM DE FAMILLE ne réarme plus la sonde",
		from: "bind:value={formLastName}\n\t\t\t\t\t\t\toninput={scheduleNameProbe}",
		to: "bind:value={formLastName}",
		expect: "preuve 2-ter : le champ NOM DE FAMILLE"
	},
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

copyFileSync(MODULE, BACKUP);
const original = readFileSync(MODULE, 'utf-8');

console.log('Banc de mutations — câblage des sondes (Story 22-2b)\n');
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
