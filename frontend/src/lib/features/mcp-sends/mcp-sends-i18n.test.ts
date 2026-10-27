// @vitest-environment node
/**
 * Story 32-8c (refs #723) — gardes de l'écran des envois du serveur MCP.
 *
 * ## 1. Le repli de chaque clé `mcp-sends-` est sa valeur fr-CH (AC-C8 ; #758)
 *
 * Aucune garde générale du dépôt ne tient un repli égal à sa valeur du catalogue
 * (#758) : `i18n-un-repli-par-cle.test.ts` compare les replis d'une clé **entre eux**,
 * G13 (`i18n-repli-divergent-actif.test.ts`) compare au catalogue sur une **liste
 * fermée** de clés. La fiche 32-7 étend G13 par préfixe (C-32-7-21) ; elle n'a pas
 * fusionné. Cette garde ferme la classe **pour ce préfixe**, dans le dossier de
 * l'écran, sans toucher au fichier de G13 (C-32-8-71) :
 *
 * - pour chaque clé `mcp-sends-*` relevée dans les sources de l'écran (lecteur
 *   commun `findCallSites` / `readFallback`), l'ensemble de ses replis littéraux
 *   est **exactement** `[valeur fr-CH]` ;
 * - l'ensemble des clés relevées est **égal**, dans les deux sens, à celui des clés
 *   du catalogue fr-CH qui commencent par `mcp-sends-` — une clé morte, ou demandée
 *   par un gabarit, rougit ; l'appartenance se juge par `startsWith`, jamais par
 *   une sous-chaîne ;
 * - chaque clé existe dans les **quatre** catalogues ;
 * - `nav-admin-mcp-sends` (résolue par `i18nMsg(item.i18nKey, item.fallback)`, que
 *   le relevé ne voit pas) est lue à part dans `+layout.svelte`.
 *
 * Angle mort écrit : le reste du catalogue reste hors de cette garde (#758).
 *
 * ## 2. Le compteur dit la borne et le message du serveur (point 1 de l'orchestrateur)
 *
 * `MCP_ENVOI_OCTETS_MAX` (frontend) = la constante Rust lue dans sa source ; la valeur
 * de `mcp-sends-content-too-large` = celle de `mcp-send-content-too-large` (le message
 * du serveur), dans chaque langue.
 *
 * ## 3. `ecran_sans_html_brut` (AC-C6)
 *
 * Aucune source de l'écran ne contient d'insertion de HTML brut.
 */
import { describe, it, expect } from 'vitest';
import { readFileSync } from 'node:fs';
// Story 32-10b (refs #723, AC-15 (b)) : le relevé et `ecartsAuCatalogue` vivent dans un
// module partagé, employé aussi par la garde de l'écran des prompts — comportement inchangé.
import {
	catalogueDe,
	ecartsAuCatalogue,
	fichiersDe,
	releveDesReplis as releve,
	sourcesDesDossiers,
} from '$lib/shared/i18n-repli-catalogue';
import { MCP_ENVOI_OCTETS_MAX } from './mcp-sends-format';

const RACINE_FTL = '../crates/kesh-i18n/locales';
const LOCALES = ['fr-CH', 'de-CH', 'it-CH', 'en-CH'] as const;
const PREFIXE = 'mcp-sends-';
const DOSSIERS_ECRAN = ['src/lib/features/mcp-sends', 'src/routes/(app)/admin/mcp/sends'];
const LAYOUT = 'src/routes/(app)/+layout.svelte';
const SOURCE_RUST = '../crates/kesh-api/src/services/envois_mcp.rs';

/** La marque d'une insertion de HTML brut, écrite en deux morceaux pour ne pas se relever ici. */
const HTML_BRUT = '{' + '@html';

/**
 * Le nombre exact de clés `mcp-sends-` : borne anti-test-muet (un relevé qui se
 * viderait et un catalogue vidé du préfixe resteraient égaux). Recompté à la main :
 * `grep -c '^mcp-sends-' crates/kesh-i18n/locales/fr-CH/messages.ftl`.
 */
const NOMBRE_DE_CLES = 48;

/** Valeurs d'un catalogue — clés de tête et valeurs d'une ligne (aucune clé du préfixe n'en a plusieurs). */
function catalogue(locale: string): Map<string, string> {
	return catalogueDe(RACINE_FTL, locale);
}

function sourcesDeLEcran() {
	return sourcesDesDossiers(DOSSIERS_ECRAN);
}

describe('écran des envois — replis égaux au catalogue (AC-C8, #758)', () => {
	const fr = catalogue('fr-CH');

	it('ecartsAuCatalogue : témoin vert, témoins rouges', () => {
		const valeurs = new Map([
			['mcp-sends-a', 'A'],
			['mcp-sends-b', 'B'],
			// `startsWith`, non sous-chaîne : cette clé contient le préfixe sans en relever.
			['audit-log-entity-mcp-sends-x', 'X'],
		]);
		const vert = new Map([
			['mcp-sends-a', new Set(['A'])],
			['mcp-sends-b', new Set(['B'])],
		]);
		expect(ecartsAuCatalogue(vert, valeurs, PREFIXE)).toEqual([]);

		const repliDivergent = new Map([
			['mcp-sends-a', new Set(['A'])],
			['mcp-sends-b', new Set(['B ancien'])],
		]);
		expect(ecartsAuCatalogue(repliDivergent, valeurs, PREFIXE)).toEqual([
			'mcp-sends-b : repli ["B ancien"] ≠ "B"',
		]);
		const deuxReplis = new Map([
			['mcp-sends-a', new Set(['A', 'A bis'])],
			['mcp-sends-b', new Set(['B'])],
		]);
		expect(ecartsAuCatalogue(deuxReplis, valeurs, PREFIXE)).toHaveLength(1);
		const cleMorte = new Map([['mcp-sends-a', new Set(['A'])]]);
		expect(ecartsAuCatalogue(cleMorte, valeurs, PREFIXE)).toEqual([
			'mcp-sends-b : au catalogue, demandée par aucun site littéral',
		]);
		const cleAbsente = new Map([...vert, ['mcp-sends-c', new Set(['C'])]]);
		expect(ecartsAuCatalogue(cleAbsente, valeurs, PREFIXE)).toEqual(['mcp-sends-c : absente du catalogue']);
	});

	it('les_replis_mcp_sends_suivent_le_catalogue', () => {
		const parCle = releve(sourcesDeLEcran());
		const cles = [...parCle.keys()].filter((c) => c.startsWith(PREFIXE));
		expect(cles.length, 'relevé de l’écran').toBe(NOMBRE_DE_CLES);
		expect([...fr.keys()].filter((c) => c.startsWith(PREFIXE)).length, 'catalogue fr-CH').toBe(
			NOMBRE_DE_CLES,
		);
		expect(ecartsAuCatalogue(parCle, fr, PREFIXE)).toEqual([]);
	});

	it('chaque clé de l’écran existe dans les quatre catalogues', () => {
		const cles = [...fr.keys()].filter((c) => c.startsWith(PREFIXE) || c === 'nav-admin-mcp-sends');
		expect(cles.length).toBe(NOMBRE_DE_CLES + 1);
		for (const locale of LOCALES) {
			const valeurs = catalogue(locale);
			const absentes = cles.filter((c) => !valeurs.get(c));
			expect(absentes, locale).toEqual([]);
		}
	});

	it('le repli de l’entrée de menu est sa valeur fr-CH', () => {
		const texte = readFileSync(LAYOUT, 'utf-8');
		const m = /i18nKey:\s*'nav-admin-mcp-sends',\s*fallback:\s*'([^']*)'/.exec(texte);
		expect(m, 'entrée de menu').not.toBeNull();
		expect(m![1]).toBe(fr.get('nav-admin-mcp-sends'));
	});
});

describe('écran des envois — la borne et le message de taille (point 1)', () => {
	it('MCP_ENVOI_OCTETS_MAX est la constante du serveur', () => {
		const rust = readFileSync(SOURCE_RUST, 'utf-8');
		const m = /pub const MCP_ENVOI_OCTETS_MAX: usize = ([\d_]+);/.exec(rust);
		expect(m, 'constante Rust').not.toBeNull();
		expect(Number(m![1].replaceAll('_', ''))).toBe(MCP_ENVOI_OCTETS_MAX);
	});

	it('le message de l’écran est celui du serveur, dans chaque langue', () => {
		const n = MCP_ENVOI_OCTETS_MAX;
		const espace = `${Math.floor(n / 1000)} ${String(n % 1000).padStart(3, '0')}`;
		const virgule = `${Math.floor(n / 1000)},${String(n % 1000).padStart(3, '0')}`;
		for (const locale of LOCALES) {
			const valeurs = catalogue(locale);
			const ecran = valeurs.get('mcp-sends-content-too-large');
			expect(ecran, locale).toBe(valeurs.get('mcp-send-content-too-large'));
			expect(ecran, locale).toContain(locale === 'en-CH' ? virgule : espace);
		}
	});
});

describe('écran des envois — un contenu non fiable se montre comme du texte (AC-C6)', () => {
	/** Les fichiers d'une liste de sources qui portent une insertion de HTML brut. */
	function avecHtmlBrut(sources: { chemin: string; texte: string }[]): string[] {
		return sources.filter((s) => s.texte.includes(HTML_BRUT)).map((s) => s.chemin);
	}

	it('ecran_sans_html_brut', () => {
		const sources = DOSSIERS_ECRAN.flatMap(fichiersDe)
			.filter((f) => f.endsWith('.svelte') || f.endsWith('.ts'))
			.map((chemin) => ({ chemin, texte: readFileSync(chemin, 'utf-8') }));
		expect(sources.length, 'sources relevées').toBeGreaterThanOrEqual(6);
		expect(avecHtmlBrut(sources)).toEqual([]);
	});

	it('témoins : une autre directive passe, une insertion brute rougit', () => {
		const vert = readFileSync('src/lib/features/reminders/ReminderHistory.svelte', 'utf-8');
		expect(vert).toContain('{@const cancelled = r.cancelledAt !== null}');
		expect(avecHtmlBrut([{ chemin: 'vert', texte: vert }])).toEqual([]);
		const rouge = `<p>${HTML_BRUT} envoi.content}</p>`;
		expect(avecHtmlBrut([{ chemin: 'rouge', texte: rouge }])).toEqual(['rouge']);
	});

	it('la textarea porte dir="ltr" et spellcheck="false"', () => {
		const detail = readFileSync('src/lib/features/mcp-sends/McpSendDetail.svelte', 'utf-8');
		const textarea = /<textarea[\s\S]*?><\/textarea>/.exec(detail)?.[0] ?? '';
		expect(textarea).toContain('dir="ltr"');
		expect(textarea).toContain('spellcheck="false"');
		expect(textarea).toContain('bind:value={valeur}');
	});
});
