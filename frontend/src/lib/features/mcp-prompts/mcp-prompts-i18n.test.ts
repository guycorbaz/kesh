// @vitest-environment node
/**
 * Story 32-10b (refs #723) — gardes de l'écran des prompts du serveur MCP.
 *
 * ## 1. Le repli de chaque clé `mcp-prompts-` est sa valeur fr-CH (AC-15 ; #758)
 *
 * Même garde que celle de l'écran des envois (32-8c), par le module partagé
 * `$lib/shared/i18n-repli-catalogue` (AC-15 (b) : la 32-7 n'a pas fusionné) :
 * - pour chaque clé `mcp-prompts-*` relevée dans les sources de l'écran, l'ensemble
 *   de ses replis littéraux est **exactement** `[valeur fr-CH]` ;
 * - l'ensemble des clés relevées est **égal**, dans les deux sens, à celui des clés
 *   du catalogue fr-CH qui commencent par `mcp-prompts-` — appartenance par
 *   `startsWith`. Les messages du serveur (32-10a-iii) portent le préfixe distinct
 *   `mcp-prompt-` (singulier), qui ne commence pas par `mcp-prompts-` ;
 * - chaque clé existe dans les **quatre** catalogues ;
 * - `nav-admin-mcp-prompts` (résolue par `i18nMsg(item.i18nKey, item.fallback)`, que
 *   le relevé ne voit pas) est lue à part dans `+layout.svelte`.
 *
 * Angle mort écrit : le reste du catalogue reste hors de cette garde (#758).
 *
 * ## 2. `ecran_sans_html_brut` (AC-13) — et le sens d'écriture des textes
 *
 * Aucune source de l'écran ne contient d'insertion de HTML brut ; chaque `<pre>` et
 * chaque `<textarea>` porte `dir="ltr"`, chaque `<textarea>` `spellcheck="false"`.
 */
import { describe, it, expect } from 'vitest';
import { readFileSync } from 'node:fs';
import {
	catalogueDe,
	ecartsAuCatalogue,
	fichiersDe,
	releveDesReplis,
	sourcesDesDossiers,
	valeursDuCatalogue,
} from '$lib/shared/i18n-repli-catalogue';

const RACINE_FTL = '../crates/kesh-i18n/locales';
const LOCALES = ['fr-CH', 'de-CH', 'it-CH', 'en-CH'] as const;
const PREFIXE = 'mcp-prompts-';
const ENTREE_DE_MENU = 'nav-admin-mcp-prompts';
const DOSSIERS_ECRAN = ['src/lib/features/mcp-prompts', 'src/routes/(app)/admin/mcp/prompts'];
const LAYOUT = 'src/routes/(app)/+layout.svelte';

/** La marque d'une insertion de HTML brut, composée par une constante pour ne pas se relever ici. */
const HTML_BRUT = '{' + '@html';

/**
 * Le nombre exact de clés `mcp-prompts-` : borne anti-test-muet (un relevé qui se
 * viderait et un catalogue vidé du préfixe resteraient égaux). Recompté à la main :
 * `grep -c '^mcp-prompts-' crates/kesh-i18n/locales/fr-CH/messages.ftl`.
 * 91 → 94 à la remédiation de la revue de code P1 (`-error-reread`, `-retry`, `-created-open`).
 */
const NOMBRE_DE_CLES = 94;

describe('écran des prompts — replis égaux au catalogue (AC-15, #758)', () => {
	const fr = catalogueDe(RACINE_FTL, 'fr-CH');

	it('ecartsAuCatalogue : témoin vert, témoins rouges (préfixe des prompts)', () => {
		const valeurs = valeursDuCatalogue(
			[
				'mcp-prompts-a = A',
				'mcp-prompts-b = B',
				// Factice, dit tel : contient le préfixe sans commencer par lui — `startsWith`,
				// non sous-chaîne, ne l'ajoute pas à l'ensemble du préfixe.
				'audit-log-entity-mcp-prompts-x = X',
				// Un message du serveur, au préfixe singulier : hors de l'ensemble.
				'mcp-prompt-name-invalid = N',
			].join('\n'),
		);
		const vert = new Map([
			['mcp-prompts-a', new Set(['A'])],
			['mcp-prompts-b', new Set(['B'])],
		]);
		expect(ecartsAuCatalogue(vert, valeurs, PREFIXE)).toEqual([]);
		const repliDivergent = new Map([
			['mcp-prompts-a', new Set(['A'])],
			['mcp-prompts-b', new Set(['B ancien'])],
		]);
		expect(ecartsAuCatalogue(repliDivergent, valeurs, PREFIXE)).toEqual([
			'mcp-prompts-b : repli ["B ancien"] ≠ "B"',
		]);
		const cleMorte = new Map([['mcp-prompts-a', new Set(['A'])]]);
		expect(ecartsAuCatalogue(cleMorte, valeurs, PREFIXE)).toEqual([
			'mcp-prompts-b : au catalogue, demandée par aucun site littéral',
		]);
	});

	it('les_replis_mcp_prompts_suivent_le_catalogue', () => {
		const parCle = releveDesReplis(sourcesDesDossiers(DOSSIERS_ECRAN));
		const cles = [...parCle.keys()].filter((c) => c.startsWith(PREFIXE));
		expect(cles.length, 'relevé de l’écran').toBe(NOMBRE_DE_CLES);
		expect([...fr.keys()].filter((c) => c.startsWith(PREFIXE)).length, 'catalogue fr-CH').toBe(
			NOMBRE_DE_CLES,
		);
		expect(ecartsAuCatalogue(parCle, fr, PREFIXE)).toEqual([]);
	});

	it('chaque clé de l’écran existe dans les quatre catalogues', () => {
		const cles = [...fr.keys()].filter((c) => c.startsWith(PREFIXE) || c === ENTREE_DE_MENU);
		expect(cles.length).toBe(NOMBRE_DE_CLES + 1);
		for (const locale of LOCALES) {
			const valeurs = catalogueDe(RACINE_FTL, locale);
			const absentes = cles.filter((c) => !valeurs.get(c));
			expect(absentes, locale).toEqual([]);
		}
	});

	it('le repli de l’entrée de menu est sa valeur fr-CH', () => {
		const texte = readFileSync(LAYOUT, 'utf-8');
		const m = /i18nKey:\s*'nav-admin-mcp-prompts',\s*fallback:\s*'([^']*)'/.exec(texte);
		expect(m, 'entrée de menu').not.toBeNull();
		expect(m![1]).toBe(fr.get(ENTREE_DE_MENU));
	});

	it('l’entrée de menu suit immédiatement celle des envois', () => {
		const texte = readFileSync(LAYOUT, 'utf-8');
		const envois = texte.indexOf("'nav-admin-mcp-sends'");
		const prompts = texte.indexOf("'nav-admin-mcp-prompts'");
		expect(envois).toBeGreaterThan(0);
		expect(prompts).toBeGreaterThan(envois);
		// Entre les deux clés, la seule entrée est celle des prompts elle-même.
		expect(texte.slice(envois, prompts).match(/i18nKey:/g) ?? []).toHaveLength(1);
	});
});

describe('écran des prompts — un texte non fiable se montre comme du texte (AC-13)', () => {
	function sourcesSvelteEtTs() {
		return DOSSIERS_ECRAN.flatMap(fichiersDe)
			.filter((f) => f.endsWith('.svelte') || f.endsWith('.ts'))
			.map((chemin) => ({ chemin, texte: readFileSync(chemin, 'utf-8') }));
	}

	/** Les gabarits Svelte de l'écran, commentaires HTML retirés (ils nomment `<pre>`). */
	function gabarits() {
		return sourcesSvelteEtTs()
			.filter((s) => s.chemin.endsWith('.svelte'))
			.map(({ chemin, texte }) => ({ chemin, texte: texte.replace(/<!--[\s\S]*?-->/g, '') }));
	}

	/** Les fichiers d'une liste de sources qui portent une insertion de HTML brut. */
	function avecHtmlBrut(sources: { chemin: string; texte: string }[]): string[] {
		return sources.filter((s) => s.texte.includes(HTML_BRUT)).map((s) => s.chemin);
	}

	it('ecran_sans_html_brut', () => {
		const sources = sourcesSvelteEtTs();
		expect(sources.length, 'sources relevées').toBeGreaterThanOrEqual(10);
		expect(avecHtmlBrut(sources)).toEqual([]);
	});

	it('témoins : une autre directive passe, une insertion brute rougit', () => {
		const vert = readFileSync('src/lib/features/reminders/ReminderHistory.svelte', 'utf-8');
		expect(vert).toContain('{@const cancelled = r.cancelledAt !== null}');
		expect(avecHtmlBrut([{ chemin: 'vert', texte: vert }])).toEqual([]);
		const rouge = `<pre>${HTML_BRUT} servi.texteVisible}</pre>`;
		expect(avecHtmlBrut([{ chemin: 'rouge', texte: rouge }])).toEqual(['rouge']);
	});

	it('chaque <pre> et <textarea> porte dir="ltr" ; chaque <textarea> spellcheck="false"', () => {
		const svelte = gabarits();
		let pres = 0;
		let textareas = 0;
		for (const { chemin, texte } of svelte) {
			for (const balise of texte.match(/<pre\b[^>]*>/g) ?? []) {
				pres += 1;
				expect(balise, chemin).toContain('dir="ltr"');
			}
			for (const balise of texte.match(/<textarea\b[^>]*>/g) ?? []) {
				textareas += 1;
				expect(balise, chemin).toContain('dir="ltr"');
				expect(balise, chemin).toContain('spellcheck="false"');
			}
		}
		// Borne anti-test-muet : 4 `<pre>` de la version, 2 du dialogue du standard ;
		// 2 `<textarea>` (éditeur, création).
		expect(pres).toBe(6);
		expect(textareas).toBe(2);
	});

	it('règle des blancs de gabarit : chaque <pre> d’un tenant, interpolation seule', () => {
		const svelte = gabarits();
		let vus = 0;
		for (const { chemin, texte } of svelte) {
			for (const pre of texte.match(/<pre\b[^>]*>[\s\S]*?<\/pre>/g) ?? []) {
				vus += 1;
				expect(pre, chemin).toMatch(/^<pre\b[^>]*>\{[^{}\n]+\}<\/pre>$/);
			}
		}
		expect(vus).toBe(6);
	});
});
