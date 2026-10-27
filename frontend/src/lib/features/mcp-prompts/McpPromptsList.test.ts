/**
 * Story 32-10b (refs #723) — la liste des prompts du serveur MCP (AC-3) et la
 * création (AC-11, AC-7 « envoi en cours » pour *Créer*).
 */
import { describe, it, expect, vi, afterEach, beforeEach } from 'vitest';
import { render, fireEvent, cleanup, waitFor } from '@testing-library/svelte';

vi.mock('./mcp-prompts.api', () => ({ listMcpPrompts: vi.fn(), createMcpPrompt: vi.fn() }));
vi.mock('$app/navigation', () => ({ goto: vi.fn() }));
vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_k: string, fallback: string, args?: Record<string, string | number>) =>
		args ? fallback.replace(/\{ \$(\w+) \}/g, (_m, k) => String(args[k])) : fallback,
	i18nLocale: () => 'fr-CH',
}));

import * as api from './mcp-prompts.api';
import * as navigation from '$app/navigation';
import McpPromptsList from './McpPromptsList.svelte';
import type { McpPromptSummary, ResumeVersion } from './mcp-prompts.types';

const mockApi = vi.mocked(api);
const mockGoto = vi.mocked(navigation.goto);

const rv = (over: Partial<ResumeVersion> = {}): ResumeVersion => ({
	numero: 1,
	origine: 'standard',
	etat: 'approved',
	nature: 'text',
	...over,
});

function ligne(id: number, over: Partial<McpPromptSummary> = {}): McpPromptSummary {
	return {
		id,
		nom: `prompt-${id}`,
		standard: false,
		version: 1,
		derniere: rv({ origine: 'admin' }),
		enVigueur: rv({ origine: 'admin' }),
		aValider: 0,
		standardDiffere: false,
		...over,
	};
}

/** Les sept cas de la fiche, par `data-testid`. */
const CAS: McpPromptSummary[] = [
	// 1 — standard servi, à jour
	ligne(1, { nom: 'kesh-assistant', standard: true, derniere: rv(), enVigueur: rv() }),
	// 2 — standard personnalisé par l'administrateur (drapeau levé)
	ligne(2, {
		nom: 'kesh-developpement',
		standard: true,
		enVigueur: rv({ numero: 2, origine: 'admin' }),
		standardDiffere: true,
	}),
	// 3 — version d'agent en vigueur
	ligne(3, { enVigueur: rv({ numero: 2, origine: 'agent' }) }),
	// 4 — retiré
	ligne(4, { enVigueur: rv({ numero: 3, origine: 'admin', nature: 'withdrawal' }) }),
	// 5 — non servi (seule une proposition attend)
	ligne(5, { enVigueur: null, derniere: rv({ origine: 'agent', etat: 'pending' }), aValider: 1 }),
	// 6 — deux à valider
	ligne(6, { aValider: 2 }),
	// 7 — standard d'origine `standard` au drapeau levé (non encore repris)
	ligne(7, { nom: 'kesh-x', standard: true, enVigueur: rv(), standardDiffere: true }),
];

async function monter(lignes: McpPromptSummary[]) {
	mockApi.listMcpPrompts.mockResolvedValueOnce(lignes);
	const r = render(McpPromptsList);
	await r.findByTestId('mcp-prompts-title');
	return r;
}

beforeEach(() => vi.clearAllMocks());
afterEach(() => cleanup());

describe('McpPromptsList', () => {
	it('rend chacun des cas de la fiche', async () => {
		const r = await monter(CAS);
		await r.findByTestId('mcp-prompts-table');
		// 1 — standard servi
		expect(r.getByTestId('mcp-prompts-standard-1').textContent?.trim()).toBe('Standard');
		expect(r.getByTestId('mcp-prompts-in-force-1').textContent?.trim()).toBe('n° 1');
		expect(r.getByTestId('mcp-prompts-origin-1').textContent?.trim()).toBe('Standard');
		expect(r.queryByTestId('mcp-prompts-customized-1')).toBeNull();
		expect(r.queryByTestId('mcp-prompts-differs-1')).toBeNull();
		// 2 — personnalisé
		expect(r.getByTestId('mcp-prompts-customized-2').textContent?.trim()).toBe('Personnalisé');
		expect(r.getByTestId('mcp-prompts-origin-2').textContent?.trim()).toBe('Administrateur');
		// 3 — agent en vigueur, pastille d'avertissement
		const agent = r.getByTestId('mcp-prompts-origin-3');
		expect(agent.textContent?.trim()).toBe('Agent');
		expect(agent.classList.contains('pastille-avertissement')).toBe(true);
		expect(r.getByTestId('mcp-prompts-origin-2').classList.contains('pastille-avertissement')).toBe(false);
		// 4 — retiré
		expect(r.getByTestId('mcp-prompts-withdrawn-4').textContent?.trim()).toBe('Retiré');
		// 5 — non servi, un à valider
		expect(r.getByTestId('mcp-prompts-not-served-5').textContent?.trim()).toBe('Non servi');
		expect(r.queryByTestId('mcp-prompts-in-force-5')).toBeNull();
		expect(r.getByTestId('mcp-prompts-pending-5').textContent?.trim()).toBe('1 à valider');
		// 6 — deux à valider ; aucune pastille à zéro
		expect(r.getByTestId('mcp-prompts-pending-6').textContent?.trim()).toBe('2 à valider');
		expect(r.queryByTestId('mcp-prompts-pending-1')).toBeNull();
		// 7 — standard d'origine `standard`, drapeau levé
		expect(r.getByTestId('mcp-prompts-differs-7').textContent?.trim()).toBe(
			'Diffère du standard de cette version de Kesh',
		);
		// Lien vers le détail.
		expect(r.getByTestId('mcp-prompts-open-3').getAttribute('href')).toBe('/admin/mcp/prompts/3');
	});

	it('personnalise_sans_avis_de_standard_change', async () => {
		const r = await monter([CAS[1]]);
		await r.findByTestId('mcp-prompts-table');
		expect(r.getByTestId('mcp-prompts-customized-2')).toBeTruthy();
		expect(r.queryByTestId('mcp-prompts-differs-2')).toBeNull();
		expect(r.container.textContent).not.toContain('Diffère du standard');
	});

	it('aucun texte ni description dans la liste (la synthèse seule)', async () => {
		const r = await monter(CAS);
		await r.findByTestId('mcp-prompts-table');
		expect(r.queryAllByRole('textbox')).toHaveLength(0);
		expect(r.container.querySelector('pre')).toBeNull();
	});

	it('liste vide : « Aucun prompt »', async () => {
		const r = await monter([]);
		expect((await r.findByTestId('mcp-prompts-empty')).textContent?.trim()).toBe('Aucun prompt');
	});

	it('403 : « Réservé à l’Administrateur » affiché', async () => {
		mockApi.listMcpPrompts.mockRejectedValueOnce({ status: 403, code: 'FORBIDDEN', message: 'x' });
		const r = render(McpPromptsList);
		expect((await r.findByTestId('mcp-prompts-error')).textContent?.trim()).toBe(
			'Réservé à l’Administrateur.',
		);
	});

	/** Ouvre le formulaire et y tape une saisie. */
	async function remplir(r: Awaited<ReturnType<typeof monter>>, nom: string, d: string, t: string) {
		await fireEvent.click(r.getByTestId('mcp-prompts-new'));
		const champNom = r.getByTestId('mcp-prompts-create-name') as HTMLInputElement;
		const champD = r.getByTestId('mcp-prompts-create-description') as HTMLInputElement;
		const champT = r.getByTestId('mcp-prompts-create-text') as HTMLTextAreaElement;
		champNom.value = nom;
		await fireEvent.input(champNom);
		champD.value = d;
		await fireEvent.input(champD);
		champT.value = t;
		await fireEvent.input(champT);
	}

	it('création : corps exact, puis le détail du prompt créé', async () => {
		const r = await monter([]);
		const t = '  texte\tà tabulation\n\n';
		await remplir(r, 'mon-prompt', ' description ', t);
		mockApi.createMcpPrompt.mockResolvedValueOnce({
			promptId: 42,
			nom: 'mon-prompt',
			numero: 1,
			origine: 'admin',
			nature: 'text',
			etat: 'approved',
			version: 1,
		});
		await fireEvent.click(r.getByTestId('mcp-prompts-create-submit'));
		await waitFor(() => expect(mockGoto).toHaveBeenCalledWith('/admin/mcp/prompts/42'));
		expect(mockApi.createMcpPrompt).toHaveBeenCalledWith({
			nom: 'mon-prompt',
			description: ' description ',
			texte: t,
		});
	});

	it('création : 400 affiché, saisies conservées', async () => {
		const r = await monter([]);
		await remplir(r, 'kesh-moi', 'd', 't');
		mockApi.createMcpPrompt.mockRejectedValueOnce({
			status: 400,
			code: 'VALIDATION_ERROR',
			message: 'Nom de prompt réservé.',
		});
		await fireEvent.click(r.getByTestId('mcp-prompts-create-submit'));
		expect((await r.findByTestId('mcp-prompts-create-error')).textContent?.trim()).toBe(
			'Nom de prompt réservé.',
		);
		expect((r.getByTestId('mcp-prompts-create-name') as HTMLInputElement).value).toBe('kesh-moi');
		expect((r.getByTestId('mcp-prompts-create-text') as HTMLTextAreaElement).value).toBe('t');
		expect(mockGoto).not.toHaveBeenCalled();
	});

	it('gestes_desactives_pendant_l_envoi — cas Créer : disabled, aria-busy, un seul POST', async () => {
		const r = await monter([]);
		await remplir(r, 'mon-prompt', 'd', 't');
		type Ecrite = Awaited<ReturnType<typeof api.createMcpPrompt>>;
		let liberer: (v: Ecrite) => void = () => {};
		mockApi.createMcpPrompt.mockReturnValueOnce(
			new Promise<Ecrite>((res) => {
				liberer = res;
			}),
		);
		const bouton = r.getByTestId('mcp-prompts-create-submit') as HTMLButtonElement;
		await fireEvent.click(bouton);
		await waitFor(() => expect(bouton.disabled).toBe(true));
		expect(bouton.getAttribute('aria-busy')).toBe('true');
		expect((r.getByTestId('mcp-prompts-create-cancel') as HTMLButtonElement).disabled).toBe(true);
		await fireEvent.click(bouton);
		await fireEvent.submit(r.getByTestId('mcp-prompts-create-form'));
		expect(mockApi.createMcpPrompt).toHaveBeenCalledTimes(1);
		liberer({ promptId: 1, nom: 'x', numero: 1, origine: 'admin', nature: 'text', etat: 'approved', version: 1 });
	});
});

describe('McpPromptsList — remédiation de la revue de code P1', () => {
	// M25 (B7) : une navigation en échec n'est pas une création en échec.
	it('creation_reussie_navigation_en_echec — aucune erreur de création, un lien vers le prompt créé', async () => {
		mockApi.listMcpPrompts.mockResolvedValueOnce([]);
		const r = render(McpPromptsList);
		await r.findByTestId('mcp-prompts-title');
		await fireEvent.click(r.getByTestId('mcp-prompts-new'));
		const nom = r.getByTestId('mcp-prompts-create-name') as HTMLInputElement;
		nom.value = 'mon-prompt';
		await fireEvent.input(nom);
		mockApi.createMcpPrompt.mockResolvedValueOnce({
			promptId: 42,
			nom: 'mon-prompt',
			numero: 1,
			origine: 'admin',
			nature: 'text',
			etat: 'approved',
			version: 1,
		});
		mockGoto.mockRejectedValueOnce(new Error('navigation interrompue'));
		await fireEvent.click(r.getByTestId('mcp-prompts-create-submit'));
		const cree = await r.findByTestId('mcp-prompts-created');
		expect(cree.getAttribute('href')).toBe('/admin/mcp/prompts/42');
		expect(r.queryByTestId('mcp-prompts-create-error')).toBeNull();
	});
});
