/**
 * Story 32-10b (refs #723) — l'éditeur d'une nouvelle version (AC-9).
 *
 * L'éditeur est un composant de saisie : il remet au détail la valeur EXACTE
 * de ses champs, et dit, quand il est pré-rempli par une version d'agent,
 * qu'enregistrer ôte la marque (P26). L'envoi, le retrait et la relecture sont
 * au détail (`McpPromptDetail.test.ts`).
 *
 * Mutations tenues ici : M9 (en partie : valeur exacte), M14 (l'avis P26 retiré).
 */
import { describe, it, expect, vi, afterEach } from 'vitest';
import { render, fireEvent, cleanup } from '@testing-library/svelte';

vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_k: string, fallback: string, args?: Record<string, string | number>) =>
		args ? fallback.replace(/\{ \$(\w+) \}/g, (_m, k) => String(args[k])) : fallback,
	i18nLocale: () => 'fr-CH',
}));

import McpPromptEditor from './McpPromptEditor.svelte';
import type { Origine } from './mcp-prompts.types';

afterEach(() => cleanup());

function monter(over: { origine?: Origine | null; description?: string; texte?: string } = {}) {
	const onEnregistrer = vi.fn();
	const r = render(McpPromptEditor, {
		description: over.description ?? 'd',
		texte: over.texte ?? 't',
		origine: over.origine ?? null,
		numeroSuivant: 5,
		envoiEnCours: false,
		occupe: false,
		onEnregistrer,
	});
	return { r, onEnregistrer };
}

describe('McpPromptEditor', () => {
	it('envoie_la_valeur_exacte — description et texte tels que saisis', async () => {
		const { r, onEnregistrer } = monter();
		const d = r.getByTestId('mcp-prompt-editor-description') as HTMLInputElement;
		const t = r.getByTestId('mcp-prompt-editor-text') as HTMLTextAreaElement;
		d.value = '  description à espaces ';
		await fireEvent.input(d);
		// ⚠️ Pas de `\r\n` ici : la `value` d'une `<textarea>` normalise elle-même les fins de
		// ligne (`\r\n` → `\n`, norme HTML, jsdom comme les navigateurs) — ce n'est pas
		// l'écran qui le fait. Le corps exact à `\r\n` est figé par `corps_texte_exact`.
		t.value = '\tligne\nautre ‮ ligne\n\n';
		await fireEvent.input(t);
		await fireEvent.click(r.getByTestId('mcp-prompt-editor-save'));
		expect(onEnregistrer).toHaveBeenCalledTimes(1);
		expect(onEnregistrer).toHaveBeenCalledWith('  description à espaces ', '\tligne\nautre ‮ ligne\n\n');
	});

	it('la textarea porte dir="ltr" et spellcheck="false" ; la phrase n° N+1', () => {
		const { r } = monter();
		const t = r.getByTestId('mcp-prompt-editor-text');
		expect(t.tagName).toBe('TEXTAREA');
		expect(t.getAttribute('dir')).toBe('ltr');
		expect(t.getAttribute('spellcheck')).toBe('false');
		expect(r.getByTestId('mcp-prompt-editor-next').textContent?.trim()).toBe(
			'Une nouvelle version n° 5 est créée ; les versions précédentes restent à l’historique.',
		);
	});

	// M14
	it('avis_p26_si_pre_rempli_par_une_version_d_agent — présent pour agent, absent sinon', () => {
		const { r } = monter({ origine: 'agent' });
		const avis = r.getByTestId('mcp-prompt-editor-agent-origin');
		expect(avis.getAttribute('role')).toBe('note');
		expect(avis.textContent?.replace(/\s+/g, ' ').trim()).toBe(
			'Ce texte a été écrit par un agent. L’enregistrer en fait votre version : elle sera servie sans la marque d’un prompt d’agent. Relisez-le comme s’il venait de vous.',
		);
		cleanup();
		for (const origine of ['admin', 'standard', null] as const) {
			const { r: autre } = monter({ origine });
			expect(autre.queryByTestId('mcp-prompt-editor-agent-origin'), String(origine)).toBeNull();
			cleanup();
		}
	});

	it('aucun_controle_bloquant_cote_ecran — 16 001 caractères et un U+202E partent', async () => {
		const { r, onEnregistrer } = monter();
		const t = r.getByTestId('mcp-prompt-editor-text') as HTMLTextAreaElement;
		const long = 'x'.repeat(16_000) + '‮';
		t.value = long;
		await fireEvent.input(t);
		const bouton = r.getByTestId('mcp-prompt-editor-save') as HTMLButtonElement;
		expect(bouton.disabled).toBe(false);
		await fireEvent.click(bouton);
		expect(onEnregistrer).toHaveBeenCalledWith('d', long);
		expect(r.queryByText(/16\s?001/)).toBeNull();
	});

	it('aucun avis du texte servi dans l’éditeur', () => {
		const { r } = monter({ origine: 'agent' });
		expect(r.queryByTestId('mcp-prompt-notice')).toBeNull();
	});
});
