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

/** Les sites conformes (fiche v0174-correctifs-3, § 5 et § 6) : huit entrées. */
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
		fichier: 'routes/(app)/settings/+page.svelte',
		fragment: 'return d.toISOString().slice(0, 10);',
		motif: 'site 13 — veille UTC, borne du verrou jugée par le serveur à sa date UTC (C-v0174c-14, #907)',
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
	it('les sources ne portent que les huit sites conformes de l’allowlist', () => {
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
	const une: Entree[] = [ALLOWLIST[3]];

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
