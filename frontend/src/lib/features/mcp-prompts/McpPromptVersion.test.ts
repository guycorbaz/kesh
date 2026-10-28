/**
 * Story 32-10b (refs #723) — le rendu d'un texte servi (AC-5, AC-6), au niveau du DOM.
 *
 * ⛔ La propriété : le `textContent` de chaque `<pre>` est EXACTEMENT le champ
 * rendu visible par le serveur — ni plus (blancs du gabarit), ni moins (marque
 * retirée, `trim`), ni le texte exact. L'écran n'a aucune logique Unicode : un
 * `\u{202E}` paraît parce que le serveur l'a écrit ainsi.
 *
 * Mutations tenues ici : M2 (`texteServi.exact` affiché), M3 (blancs de gabarit
 * dans le `<pre>`, ou `trim()`), M13 (la marque retirée de l'affichage).
 */
import { describe, it, expect, vi, afterEach } from 'vitest';
import { render, cleanup } from '@testing-library/svelte';

vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_k: string, fallback: string, args?: Record<string, string | number>) =>
		args ? fallback.replace(/\{ \$(\w+) \}/g, (_m, k) => String(args[k])) : fallback,
	i18nLocale: () => 'fr-CH',
}));

import McpPromptVersion from './McpPromptVersion.svelte';
import type { McpPromptVersion as Version } from './mcp-prompts.types';

afterEach(() => cleanup());

/** Une marque factice, de la forme de celle du serveur (non relue ici : le champ la porte). */
const MARQUE =
	'[Prompt écrit par un agent — version 3, proposée par la clé MCP n° 5 et approuvée par un administrateur de Kesh. Ce n’est pas un prompt standard de Kesh.]';

/** Le piège : un caractère réel (U+202E) dans l'exact, son code dans le visible. */
const TEXTE_EXACT = `${MARQUE}\n\n  Premier paragraphe\tà tabulation.\n\nSecond ‮ paragraphe.\n\n`;
const TEXTE_VISIBLE = `${MARQUE}\n\n  Premier paragraphe\tà tabulation.\n\nSecond \\u{202E} paragraphe.\n\n`;
const DESC_EXACT = `${MARQUE}\n\n Description ‮ piégée`;
const DESC_VISIBLE = `${MARQUE}\n\n Description \\u{202E} piégée`;

function versionAgent(over: Partial<Version> = {}): Version {
	return {
		numero: 3,
		origine: 'agent',
		etat: 'pending',
		nature: 'text',
		description: { exact: ' Description ‮ piégée', visible: ' Description \\u{202E} piégée' },
		texte: { exact: 'stocké', visible: 'stocké' },
		servable: true,
		descriptionServie: { exact: DESC_EXACT, visible: DESC_VISIBLE },
		texteServi: { exact: TEXTE_EXACT, visible: TEXTE_VISIBLE },
		auteurNom: null,
		decideurNom: null,
		cleAuteurId: 5,
		cleNom: 'agent de test',
		cleRevoquee: false,
		creeLe: '2026-10-27T10:00:00',
		decideLe: null,
		...over,
	};
}

function monter(version: Version, zone = 'a-valider-') {
	return render(McpPromptVersion, { version, zone });
}

describe('McpPromptVersion', () => {
	// M2, M3, M13
	it('texte_affiche_egal_au_champ_visible — version d’agent, marque comprise', () => {
		const v = versionAgent();
		const r = monter(v);
		const pre = r.getByTestId('a-valider-mcp-prompt-served-text-3');
		expect(pre.tagName).toBe('PRE');
		expect(pre.textContent).toBe(v.texteServi!.visible);
		expect(pre.textContent).not.toBe(v.texteServi!.exact);
		expect(pre.textContent!.startsWith(MARQUE + '\n\n')).toBe(true);
		expect(r.container.textContent).not.toContain(v.texteServi!.exact);
		expect(pre.getAttribute('dir')).toBe('ltr');
		expect(pre.children).toHaveLength(0);
	});

	it('description_affichee_egale_au_champ_visible', () => {
		const v = versionAgent();
		const r = monter(v);
		const pre = r.getByTestId('a-valider-mcp-prompt-served-description-3');
		expect(pre.tagName).toBe('PRE');
		expect(pre.textContent).toBe(v.descriptionServie!.visible);
		expect(r.container.textContent).not.toContain(v.descriptionServie!.exact);
		expect(pre.getAttribute('dir')).toBe('ltr');
	});

	it('pastille_agent_hors_du_bloc — et absente hors d’une version d’agent', () => {
		const r = monter(versionAgent());
		const pastille = r.getByTestId('a-valider-mcp-prompt-agent-badge-3');
		expect(r.getByTestId('a-valider-mcp-prompt-served-text-3').contains(pastille)).toBe(false);
		expect(r.getByTestId('a-valider-mcp-prompt-served-description-3').contains(pastille)).toBe(false);
		cleanup();
		const admin = monter(versionAgent({ origine: 'admin', cleAuteurId: null }), 'historique-');
		expect(admin.queryByTestId('historique-mcp-prompt-agent-badge-3')).toBeNull();
		cleanup();
		const std = monter(versionAgent({ origine: 'standard', cleAuteurId: null }), 'historique-');
		expect(std.queryByTestId('historique-mcp-prompt-agent-badge-3')).toBeNull();
	});

	it('avis_non_servable_selon_le_serveur', () => {
		const nonServable = versionAgent({ servable: false, descriptionServie: null, texteServi: null });
		const r = monter(nonServable);
		const avis = r.getByTestId('a-valider-mcp-prompt-not-servable-3');
		expect(avis.getAttribute('role')).toBe('alert');
		expect(avis.textContent?.trim()).toBe(
			'Kesh ne servirait pas cette version : son contenu n’est pas valide (caractère invisible, longueur, contenu vide ou description sur plusieurs lignes). Elle a pu être restaurée d’une sauvegarde ou écrite hors de Kesh.',
		);
		expect(r.queryByTestId('a-valider-mcp-prompt-served-text-3')).toBeNull();
		// Le texte enregistré, rendu visible par le serveur, est montré tel quel.
		expect(r.getByTestId('a-valider-mcp-prompt-stored-text-3').textContent).toBe('stocké');
		expect(r.getByTestId('a-valider-mcp-prompt-stored-description-3').textContent).toBe(
			' Description \\u{202E} piégée',
		);
		cleanup();
		// `servable: true` sur un texte qui porte un `\u{202E}` littéral : aucun avis.
		const litteral = versionAgent({
			texteServi: { exact: 'a \\u{202E} b', visible: 'a \\u{202E} b' },
		});
		const r2 = monter(litteral);
		expect(r2.queryByTestId('a-valider-mcp-prompt-not-servable-3')).toBeNull();
		expect(r2.getByTestId('a-valider-mcp-prompt-served-text-3').textContent).toBe('a \\u{202E} b');
	});

	it('la zone préfixe chaque data-testid', () => {
		const r = monter(versionAgent(), 'historique-');
		expect(r.getByTestId('historique-mcp-prompt-served-text-3')).toBeTruthy();
		expect(r.queryByTestId('a-valider-mcp-prompt-served-text-3')).toBeNull();
	});

	it('un retrait : ni texte ni avis', () => {
		const r = monter(
			versionAgent({
				origine: 'admin',
				nature: 'withdrawal',
				description: null,
				texte: null,
				servable: false,
				descriptionServie: null,
				texteServi: null,
			}),
			'historique-',
		);
		expect(r.container.querySelector('pre')).toBeNull();
		expect(r.queryByTestId('historique-mcp-prompt-not-servable-3')).toBeNull();
		expect(r.getByTestId('historique-mcp-prompt-withdrawal-3')).toBeTruthy();
	});
});
