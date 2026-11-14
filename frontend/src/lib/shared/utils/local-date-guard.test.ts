/**
 * Story v0174-correctifs-3 (#889, AC-3.9, C-v0174c-17) — la garde de l'inventaire clos des
 * dates du jour et des affichages d'instants.
 *
 * Deux motifs sont relevés **par ligne**, dans toutes les sources `src/**\/*.{ts,svelte}`
 * hors tests (`*.test.ts`, `*.test.svelte`, `*.test-helper*`), **sans masque des
 * commentaires** :
 *
 * - `MOTIF_DATE_UTC` : la date du jour tirée de l'instant UTC — à 00 h 30 à Zurich, elle
 *   rend la veille. La date du jour se prend de `todayLocal` (`local-date.ts`).
 * - `MOTIF_AFFICHAGE_TRONQUE` : un horodatage tronqué à ses dix premiers caractères avant
 *   d'être affiché — la date UTC, non la date suisse. `formatSwissDate` lit l'horodatage
 *   entier (forme parenthésée comprise : `formatSwissDate((x ?? '').slice(0, 10))`).
 *
 * Chaque relevé doit égaler **une** entrée de `ALLOWLIST`, et chaque entrée **un seul**
 * relevé : une entrée est `{ fichier, fragment, motif }`, où `fragment` est un extrait exact
 * de la ligne. Une entrée orpheline, un fragment qui égale deux lignes, un relevé sans
 * entrée rougissent. Un commentaire futur qui citerait un motif mot pour mot est relevé
 * comme du code : il entre à l'allowlist (motif « commentaire ») ou se reformule.
 *
 * Story 35-5c (#929, D4) — **les specs E2E**, par l'ensemble clos : la garde parcourt aussi
 * `tests/e2e/**\/*.ts` (helpers et leurs tests compris) et y relève, sur le texte entier (une
 * forme coupée sur plusieurs lignes est vue, au numéro de la ligne où elle commence) :
 * toute construction de date **sans argument** (l'horloge du processus Node, au fuseau de
 * la machine), toute construction depuis `Date.now`, et toute sérialisation ISO tronquée à
 * dix caractères, sur tout récepteur. **Un seul relevé par ligne**, quel que soit le nombre
 * de motifs qu'elle porte. Le jour d'une spec vient de `todayInZurich()` ; l'arithmétique
 * sur un jour, d'`ajouterJoursIso` (`tests/e2e/helpers/dates.ts`). `ALLOWLIST_SPECS` est
 * close de trois entrées, dont le fragment est le texte exact de la ligne relevée.
 * **Angle mort assumé** : un jour tiré de `Date.now()` sans construction de date
 * (`Intl.DateTimeFormat('sv-SE').format(Date.now())`) n'est pas relevé — aucun
 * aujourd'hui ; `Date.now()` seul sert aux suffixes d'unicité, par centaines.
 */
import { describe, expect, it } from 'vitest';
import { readFileSync, readdirSync } from 'node:fs';
import { join, relative } from 'node:path';

const RACINE = 'src';

const MOTIF_DATE_UTC = /toISOString\(\)\.slice\(0, ?10\)/;
const MOTIF_AFFICHAGE_TRONQUE = /formatSwissDate\(.*\.slice\(0, ?10\)/;

interface Releve {
	fichier: string;
	ligne: number;
	texte: string;
}

interface Entree {
	fichier: string;
	fragment: string;
	motif: string;
}

/**
 * Les sites conformes (fiche v0174-correctifs-3, § 5 et § 6) : sept entrées. La borne du
 * verrou de période (`settings/+page.svelte`, site 13) en est sortie à la Story 35-5a
 * (#907) : elle prend la veille de `todayLocal`, le serveur jugeant au jour de Zurich.
 */
const ALLOWLIST: Entree[] = [
	{
		fichier: 'lib/features/invoices/invoice-helpers.ts',
		fragment: 'return d.toISOString().slice(0, 10);',
		motif: 'site 4 — arithmétique de jours sur les composants UTC d’une date pure (addDaysIso)',
	},
	{
		fichier: 'lib/features/journal-entries/form-helpers.ts',
		fragment: 'return d.toISOString().slice(0, 10);',
		motif: 'site 5 — arithmétique de jours sur les composants UTC d’une date pure (nextDay)',
	},
	{
		fichier: 'routes/(app)/invoices/[id]/+page.svelte',
		fragment: 'formatSwissDate(invoice.paidAt.slice(0, 10))',
		motif: 'a — date du règlement à minuit, non un instant',
	},
	{
		fichier: 'routes/(app)/invoices/due-dates/+page.svelte',
		fragment: 'formatSwissDate(inv.paidAt.slice(0, 10))',
		motif: 'f — date du règlement à minuit, non un instant',
	},
	{
		fichier: 'routes/(app)/supplier-invoices/[id]/+page.svelte',
		fragment: 'formatSwissDate(paidAt.slice(0, 10))',
		motif: 'i — date du règlement à minuit, non un instant',
	},
	{
		fichier: 'routes/(app)/settings/api-keys/+page.svelte',
		fragment: 'formatSwissDate(value.slice(0, 10))',
		motif: 'expiresAt — date de calendrier envoyée T23:59:59Z (exception de la 0.17.2)',
	},
	{
		fichier: 'lib/features/supplier-invoices/supplier-invoice-unvalidate.ts',
		fragment: 'formatSwissDate(iso.slice(0, 10))',
		motif: 'dates pures AAAA-MM-JJ des refus de dévalidation',
	},
];

function estSource(nom: string): boolean {
	if (!/\.(ts|svelte)$/.test(nom)) return false;
	return !/\.test\.(ts|svelte)$/.test(nom) && !nom.includes('.test-helper');
}

/** Les lignes qui portent l'un des deux motifs, dans `sources` (`chemin → texte`). */
function relever(sources: Map<string, string>): Releve[] {
	const out: Releve[] = [];
	for (const [fichier, texte] of sources) {
		texte.split('\n').forEach((l, i) => {
			if (MOTIF_DATE_UTC.test(l) || MOTIF_AFFICHAGE_TRONQUE.test(l)) {
				out.push({ fichier, ligne: i + 1, texte: l });
			}
		});
	}
	return out;
}

/** Les écarts entre les relevés et l'allowlist ; vide si l'inventaire est clos. */
function auditer(releves: Releve[], allowlist: Entree[]): string[] {
	const erreurs: string[] = [];
	const usages = new Map<Entree, Releve[]>(allowlist.map((e) => [e, []]));
	for (const r of releves) {
		const entrees = allowlist.filter((e) => e.fichier === r.fichier && r.texte.includes(e.fragment));
		if (entrees.length === 0) {
			erreurs.push(`relevé sans entrée : ${r.fichier}:${r.ligne} — ${r.texte.trim()}`);
		} else if (entrees.length > 1) {
			erreurs.push(`relevé à plusieurs entrées : ${r.fichier}:${r.ligne}`);
		} else {
			usages.get(entrees[0])!.push(r);
		}
	}
	for (const [e, rs] of usages) {
		if (rs.length === 0) erreurs.push(`entrée orpheline : ${e.fichier} — ${e.fragment}`);
		if (rs.length > 1) {
			erreurs.push(
				`fragment à plusieurs lignes : ${e.fichier} — ${e.fragment} (lignes ${rs.map((r) => r.ligne).join(', ')})`,
			);
		}
	}
	return erreurs;
}

function lireSources(): Map<string, string> {
	const sources = new Map<string, string>();
	const parcourir = (rep: string) => {
		for (const e of readdirSync(rep, { withFileTypes: true })) {
			const chemin = join(rep, e.name);
			if (e.isDirectory()) parcourir(chemin);
			else if (estSource(e.name)) sources.set(relative(RACINE, chemin), readFileSync(chemin, 'utf-8'));
		}
	};
	parcourir(RACINE);
	return sources;
}

describe('garde des dates du jour et des affichages d’instants (#889)', () => {
	it('les sources ne portent que les sept sites conformes de l’allowlist', () => {
		const sources = lireSources();
		expect(sources.size).toBeGreaterThan(100);
		expect(auditer(relever(sources), ALLOWLIST)).toEqual([]);
	});

	// MC-889-g — la sonde : la garde rougit sur chacun des quatre écarts.
	const base = new Map([
		[
			'routes/(app)/invoices/[id]/+page.svelte',
			'{formatSwissDate(invoice.paidAt.slice(0, 10))}\n{formatSwissDate(invoice.pdfFrozenAt)}',
		],
	]);
	// Story 35-5a : l'entrée se désigne par son fichier, non par sa position (le retrait de
	// l'entrée du verrou avait décalé l'index et fait viser la sonde à côté).
	const une: Entree[] = ALLOWLIST.filter(
		(e) => e.fichier === 'routes/(app)/invoices/[id]/+page.svelte',
	);

	it('sonde : le décor de la sonde est propre', () => {
		expect(auditer(relever(base), une)).toEqual([]);
	});

	it('sonde : un `.slice(0, 10)` rétabli dans un fichier autorisé rougit (site b)', () => {
		const s = new Map(base);
		s.set(
			'routes/(app)/invoices/[id]/+page.svelte',
			'{formatSwissDate(invoice.paidAt.slice(0, 10))}\n{formatSwissDate(invoice.pdfFrozenAt.slice(0, 10))}',
		);
		expect(auditer(relever(s), une)).toEqual([
			expect.stringContaining('relevé sans entrée : routes/(app)/invoices/[id]/+page.svelte:2'),
		]);
	});

	it('sonde : une date UTC ajoutée hors allowlist rougit', () => {
		const s = new Map(base);
		s.set('lib/neuf.ts', 'const t = new Date().toISOString().slice(0, 10);');
		expect(auditer(relever(s), une)).toEqual([expect.stringContaining('lib/neuf.ts:1')]);
	});

	it('sonde : une entrée qui ne correspond plus à rien rougit', () => {
		expect(auditer(relever(new Map()), une)).toEqual([
			expect.stringContaining('entrée orpheline'),
		]);
	});

	it('sonde : un fragment qui égale deux lignes rougit', () => {
		const s = new Map([
			[
				'routes/(app)/invoices/[id]/+page.svelte',
				'{formatSwissDate(invoice.paidAt.slice(0, 10))}\n{formatSwissDate(invoice.paidAt.slice(0, 10))}',
			],
		]);
		expect(auditer(relever(s), une)).toEqual([
			expect.stringContaining('fragment à plusieurs lignes'),
		]);
	});

	it('sonde : la forme parenthésée est relevée (site j)', () => {
		const s = new Map([['x.svelte', "formatSwissDate((batch.confirmedAt ?? '').slice(0, 10))"]]);
		expect(relever(s)).toHaveLength(1);
	});
});

// ---------------------------------------------------------------------------
// Story 35-5c (#929, D4) — les specs E2E
// ---------------------------------------------------------------------------

const RACINE_SPECS = 'tests/e2e';

/**
 * Les motifs des specs, appliqués au texte entier (drapeau `g`) : les blancs, retours à la
 * ligne compris, sont admis entre les jetons.
 */
const MOTIFS_SPECS: RegExp[] = [
	/new\s+Date\s*\(\s*\)/g,
	/new\s+Date\s*\(\s*Date\s*\.\s*now/g,
	/toISOString\s*\(\s*\)\s*\.\s*slice\s*\(\s*0\s*,\s*10\s*\)/g,
];

/**
 * Les trois sites admis (fiche 35-5c, D4). Le fragment est le texte exact de la ligne.
 */
const ALLOWLIST_SPECS: Entree[] = [
	{
		fichier: 'helpers/api-fixtures.ts',
		fragment: "return new Date().toLocaleDateString('sv-SE', { timeZone: 'Europe/Zurich' });",
		motif: 'todayInZurich — le jour suisse, seule lecture de l’horloge des specs',
	},
	{
		fichier: 'audit-log.spec.ts',
		fragment: 'return new Date().toISOString().slice(0, 10);',
		motif: 'todayUtc — le filtre du journal d’audit compare des horodatages UTC (#669)',
	},
	{
		fichier: 'invoices.spec.ts',
		fragment: 'const sentAt = new Date(Date.now() - 3_600_000).toISOString().slice(0, 19);',
		motif: 'un instant (il y a une heure), non un jour',
	},
];

/** Les lignes des specs qui portent au moins un motif — une fois par ligne. */
function releverSpecs(sources: Map<string, string>): Releve[] {
	const out: Releve[] = [];
	for (const [fichier, texte] of sources) {
		const lignes = texte.split('\n');
		const vues = new Set<number>();
		for (const motif of MOTIFS_SPECS) {
			for (const m of texte.matchAll(motif)) {
				const ligne = texte.slice(0, m.index).split('\n').length;
				if (!vues.has(ligne)) {
					vues.add(ligne);
					out.push({ fichier, ligne, texte: lignes[ligne - 1] });
				}
			}
		}
	}
	return out.sort((a, b) => a.fichier.localeCompare(b.fichier) || a.ligne - b.ligne);
}

function lireSpecs(): Map<string, string> {
	const sources = new Map<string, string>();
	const parcourir = (rep: string) => {
		for (const e of readdirSync(rep, { withFileTypes: true })) {
			const chemin = join(rep, e.name);
			if (e.isDirectory()) parcourir(chemin);
			else if (/\.ts$/.test(e.name)) {
				sources.set(relative(RACINE_SPECS, chemin), readFileSync(chemin, 'utf-8'));
			}
		}
	};
	parcourir(RACINE_SPECS);
	return sources;
}

describe('garde des dates du jour des specs E2E (#929, Story 35-5c)', () => {
	it('les specs ne portent que les trois sites de l’allowlist', () => {
		const sources = lireSpecs();
		expect(sources.size).toBeGreaterThan(100);
		expect(sources.has('helpers/api-fixtures.ts')).toBe(true);
		expect(auditer(releverSpecs(sources), ALLOWLIST_SPECS)).toEqual([]);
		expect(sources.has('helpers/dates.ts')).toBe(true);
	});

	const releves = (texte: string) => releverSpecs(new Map([['x.spec.ts', texte]]));

	it('sonde : le jour UTC d’une spec rougit, entier ou coupé', () => {
		expect(releves("const t = new Date().toISOString().slice(0, 10);")).toHaveLength(1);
		expect(
			releves('const t = new Date()\n\t.toISOString()\n\t.slice(0, 10);').map((r) => r.ligne),
		).toEqual([1, 2]);
		expect(releves('const t = d\n\t.toISOString()\n\t.slice(0, 10);').map((r) => r.ligne)).toEqual([
			2,
		]);
	});

	it('sonde : un jour par une variable et une date depuis Date.now rougissent', () => {
		expect(releves('const d = new Date();\nconst j = d.getDate();').map((r) => r.ligne)).toEqual([
			1,
		]);
		expect(releves('const d = new Date(Date.now() + 1);')).toHaveLength(1);
		expect(releves('const d = new Date(\n);')).toHaveLength(1);
	});

	it('sonde : une ligne à deux motifs ne donne qu’un relevé, que son entrée apparie', () => {
		const s = new Map([['audit-log.spec.ts', '\treturn new Date().toISOString().slice(0, 10);']]);
		const r = releverSpecs(s);
		expect(r).toHaveLength(1);
		expect(
			auditer(
				r,
				ALLOWLIST_SPECS.filter((e) => e.fichier === 'audit-log.spec.ts'),
			),
		).toEqual([]);
	});

	it('sonde : l’arithmétique sur un jour donné ne rougit pas', () => {
		expect(releves("const j = ajouterJoursIso('2026-03-10', 1);")).toEqual([]);
		expect(releves('const d = new Date(Date.UTC(2026, 0, 1 + n));')).toEqual([]);
		expect(releves('const id = `x-${Date.now()}`;')).toEqual([]);
	});

	it('sonde : un relevé hors allowlist rougit, nommé', () => {
		const s = new Map([['neuf.spec.ts', 'const t = new Date();']]);
		expect(auditer(releverSpecs(s), [])).toEqual([
			expect.stringContaining('relevé sans entrée : neuf.spec.ts:1'),
		]);
	});
});
