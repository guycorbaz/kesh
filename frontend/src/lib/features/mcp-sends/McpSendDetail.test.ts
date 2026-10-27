/**
 * Story 32-8c (refs #723) — le détail d'un envoi du serveur MCP (AC-C3 à AC-C5).
 *
 * ⚠️ Chaque test nomme la MUTATION qu'il attrape (MC1 à MC6 de la fiche 32-8).
 */
import { describe, it, expect, vi, afterEach, beforeEach } from 'vitest';
import { render, fireEvent, cleanup, waitFor } from '@testing-library/svelte';

vi.mock('./mcp-sends.api', async (original) => ({
	...(await original<typeof import('./mcp-sends.api')>()),
	getMcpSend: vi.fn(),
	decideMcpSend: vi.fn(),
}));
vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_k: string, fallback: string, args?: Record<string, string | number>) =>
		args ? fallback.replace(/\{ \$(\w+) \}/g, (_m, k) => String(args[k])) : fallback,
	i18nLocale: () => 'fr-CH',
}));
vi.mock('$lib/shared/utils/notify', () => ({ notifySuccess: vi.fn() }));

import * as api from './mcp-sends.api';
import McpSendDetail from './McpSendDetail.svelte';
import type { McpSendDetail as Detail } from './mcp-sends.types';

const mockApi = vi.mocked(api);

function envoi(over: Partial<Detail> = {}): Detail {
	return {
		id: 12,
		mcpKeyId: 3,
		mcpKeyName: 'agent de test',
		mcpKeyRevoked: false,
		toolName: 'kesh_version',
		state: 'pending',
		modified: false,
		expiresAt: '2026-10-27T10:00:00.000',
		decidedAt: null,
		decidedBy: null,
		createdAt: '2026-10-26T10:00:00.000',
		summary: 'Version de Kesh',
		content: '{"version":"0.16.0"}',
		contentBytes: 20,
		version: 5,
		...over,
	};
}

/** Une erreur d'API telle que la lève le client. */
function erreurApi(status: number, code: string, message: string) {
	return { status, code, message };
}

function textarea(getByTestId: (id: string) => HTMLElement): HTMLTextAreaElement {
	return getByTestId('mcp-send-content') as HTMLTextAreaElement;
}

async function monter(d: Detail = envoi()) {
	mockApi.getMcpSend.mockResolvedValueOnce(d);
	const r = render(McpSendDetail, { id: d.id });
	await r.findByTestId('mcp-send-title');
	return r;
}

/** Passe en modification et remplace la valeur, comme un clavier. */
async function modifier(r: Awaited<ReturnType<typeof monter>>, valeur: string) {
	await fireEvent.click(r.getByTestId('mcp-send-modify'));
	const t = textarea(r.getByTestId);
	expect(t.readOnly).toBe(false);
	t.value = valeur;
	await fireEvent.input(t);
}

beforeEach(() => vi.clearAllMocks());
afterEach(() => cleanup());

describe('McpSendDetail', () => {
	// MC5 : valeur de la textarea dérivée (`content.trimStart()`).
	it('la_valeur_affichee_est_la_valeur_recue (valeur commençant par \\n)', async () => {
		const content = '\n  première\tligne \n\\u{202E}\n\n';
		const r = await monter(envoi({ content }));
		const t = textarea(r.getByTestId);
		expect(t.value).toBe(content);
		expect(t.readOnly).toBe(true);
		expect(t.getAttribute('dir')).toBe('ltr');
		expect(t.getAttribute('spellcheck')).toBe('false');
	});

	it('bouton_selon_modification — tel quel, puis version modifiée, puis rétabli', async () => {
		const r = await monter();
		expect(r.queryByTestId('mcp-send-approve')).not.toBeNull();
		expect(r.queryByTestId('mcp-send-approve-modified')).toBeNull();
		await modifier(r, 'autre texte');
		expect(r.queryByTestId('mcp-send-approve')).toBeNull();
		expect(r.queryByTestId('mcp-send-approve-modified')).not.toBeNull();
		await fireEvent.click(r.getByTestId('mcp-send-restore'));
		expect(textarea(r.getByTestId).value).toBe('{"version":"0.16.0"}');
		expect(r.queryByTestId('mcp-send-approve')).not.toBeNull();
		expect(r.queryByTestId('mcp-send-approve-modified')).toBeNull();
	});

	// MC2 : « tel quel » envoie `content`.
	it('tel_quel_n_envoie_pas_de_contenu', async () => {
		const r = await monter();
		mockApi.decideMcpSend.mockResolvedValueOnce(envoi({ state: 'approved', version: 6 }));
		await fireEvent.click(r.getByTestId('mcp-send-approve'));
		await waitFor(() => expect(mockApi.decideMcpSend).toHaveBeenCalledTimes(1));
		const [id, corps] = mockApi.decideMcpSend.mock.calls[0];
		expect(id).toBe(12);
		expect(JSON.stringify(corps)).toBe('{"version":5,"decision":"approve"}');
	});

	it('tel quel après modification puis retour à la valeur reçue : toujours sans contenu', async () => {
		const r = await monter();
		await modifier(r, 'autre');
		await modifier2(r, '{"version":"0.16.0"}');
		mockApi.decideMcpSend.mockResolvedValueOnce(envoi({ state: 'approved' }));
		await fireEvent.click(r.getByTestId('mcp-send-approve'));
		await waitFor(() => expect(mockApi.decideMcpSend).toHaveBeenCalledTimes(1));
		expect(JSON.stringify(mockApi.decideMcpSend.mock.calls[0][1])).toBe(
			'{"version":5,"decision":"approve"}',
		);
	});

	// MC1 : `trim()` sur la valeur envoyée.
	it('envoie_la_valeur_exacte — espace de tête, tabulation, ligne vide finale, U+202E littéral', async () => {
		const valeur = ' tête\tx\n\\u{202E}\n‮\n\n';
		const r = await monter();
		await modifier(r, valeur);
		mockApi.decideMcpSend.mockResolvedValueOnce(envoi({ state: 'approved', modified: true, content: valeur }));
		await fireEvent.click(r.getByTestId('mcp-send-approve-modified'));
		await waitFor(() => expect(mockApi.decideMcpSend).toHaveBeenCalledTimes(1));
		expect(mockApi.decideMcpSend.mock.calls[0][1]).toEqual({
			version: 5,
			decision: 'approve',
			content: valeur,
		});
		expect((mockApi.decideMcpSend.mock.calls[0][1] as { content: string }).content).toBe(valeur);
	});

	// MC4 : refus sans confirmation.
	it('refus_apres_confirmation', async () => {
		const r = await monter();
		await fireEvent.click(r.getByTestId('mcp-send-refuse'));
		expect(mockApi.decideMcpSend).not.toHaveBeenCalled();
		const confirmer = await r.findByTestId('mcp-send-refuse-confirm');
		mockApi.decideMcpSend.mockResolvedValueOnce(envoi({ state: 'refused', content: null, summary: null }));
		await fireEvent.click(confirmer);
		await waitFor(() => expect(mockApi.decideMcpSend).toHaveBeenCalledTimes(1));
		expect(JSON.stringify(mockApi.decideMcpSend.mock.calls[0][1])).toBe('{"version":5,"decision":"refuse"}');
		expect(await r.findByTestId('mcp-send-erased')).toBeTruthy();
	});

	it('le compteur compte les octets UTF-8 ; au-delà de la borne, le message du serveur', async () => {
		const r = await monter(envoi({ content: 'é✓' }));
		expect(r.getByTestId('mcp-send-counter').textContent).toMatch(/^\s*5 octets sur 24\D576\s*$/);
		await modifier(r, 'a'.repeat(24_577));
		expect(r.getByTestId('mcp-send-too-large').textContent?.trim()).toBe(
			"Le contenu modifié dépasse la taille d'un envoi (24 576 octets).",
		);
		await modifier2(r, 'a'.repeat(24_576));
		expect(r.queryByTestId('mcp-send-too-large')).toBeNull();
	});

	describe('les erreurs (AC-C5)', () => {
		// MC6 : 409 non montré.
		it('409 déjà décidé : message du serveur et Recharger', async () => {
			const r = await monter();
			mockApi.decideMcpSend.mockRejectedValueOnce(
				erreurApi(409, 'ILLEGAL_STATE_TRANSITION', 'Cet envoi a déjà été décidé.'),
			);
			await fireEvent.click(r.getByTestId('mcp-send-approve'));
			const err = await r.findByTestId('mcp-send-error');
			expect(err.textContent).toContain('Cet envoi a déjà été décidé.');
			expect(r.queryByTestId('mcp-send-reload')).not.toBeNull();
			mockApi.getMcpSend.mockResolvedValueOnce(envoi({ state: 'approved' }));
			await fireEvent.click(r.getByTestId('mcp-send-reload'));
			await waitFor(() => expect(mockApi.getMcpSend).toHaveBeenCalledTimes(2));
			await waitFor(() => expect(r.queryByTestId('mcp-send-error')).toBeNull());
		});

		it('409 version périmée : message de l’écran et Recharger', async () => {
			const r = await monter();
			mockApi.decideMcpSend.mockRejectedValueOnce(
				erreurApi(409, 'OPTIMISTIC_LOCK_CONFLICT', 'conflit générique'),
			);
			await fireEvent.click(r.getByTestId('mcp-send-approve'));
			const err = await r.findByTestId('mcp-send-error');
			expect(err.textContent).toContain('Cet envoi a changé depuis son affichage');
			expect(r.queryByTestId('mcp-send-reload')).not.toBeNull();
		});

		it('404 : « Envoi introuvable »', async () => {
			const r = await monter();
			mockApi.decideMcpSend.mockRejectedValueOnce(erreurApi(404, 'NOT_FOUND', 'x'));
			await fireEvent.click(r.getByTestId('mcp-send-approve'));
			expect((await r.findByTestId('mcp-send-error')).textContent).toContain('Envoi introuvable.');
			expect(r.queryByTestId('mcp-send-reload')).toBeNull();
		});

		it('400 d’une version modifiée : le message du serveur et l’aide aux caractères refusés', async () => {
			const r = await monter();
			await modifier(r, 'avertissement ⚠️');
			// L'aide est là dès la modification ; après le refus, elle passe sous le message, une fois.
			expect(r.queryByTestId('mcp-send-hidden-help')).not.toBeNull();
			mockApi.decideMcpSend.mockRejectedValueOnce(
				erreurApi(
					400,
					'VALIDATION_ERROR',
					'Le contenu modifié contient un caractère invisible ou de contrôle : retirez-le avant de valider.',
				),
			);
			await fireEvent.click(r.getByTestId('mcp-send-approve-modified'));
			const err = await r.findByTestId('mcp-send-error');
			expect(err.textContent).toContain('caractère invisible ou de contrôle');
			expect(r.getByTestId('mcp-send-error-help').textContent).toContain('U+FE0F');
			expect(r.queryByTestId('mcp-send-hidden-help')).toBeNull();
			// Aucun contrôle côté client : la valeur est partie telle quelle.
			expect((mockApi.decideMcpSend.mock.calls[0][1] as { content: string }).content).toBe(
				'avertissement ⚠️',
			);
		});

		it('400 de taille : le message du serveur, sans l’aide aux caractères', async () => {
			const r = await monter();
			await modifier(r, 'a'.repeat(24_577));
			mockApi.decideMcpSend.mockRejectedValueOnce(
				erreurApi(400, 'VALIDATION_ERROR', "Le contenu modifié dépasse la taille d'un envoi (24 576 octets)."),
			);
			await fireEvent.click(r.getByTestId('mcp-send-approve-modified'));
			expect((await r.findByTestId('mcp-send-error')).textContent).toContain('24 576 octets');
			expect(r.queryByTestId('mcp-send-error-help')).toBeNull();
		});

		it('chargement refusé (403) : le refus de l’API est montré', async () => {
			mockApi.getMcpSend.mockRejectedValueOnce(erreurApi(403, 'FORBIDDEN', 'Accès refusé.'));
			const r = render(McpSendDetail, { id: 12 });
			expect((await r.findByTestId('mcp-send-load-error')).textContent).toContain('Accès refusé.');
		});

		it('chargement 404 : « Envoi introuvable » ; numéro illisible : sans appel', async () => {
			mockApi.getMcpSend.mockRejectedValueOnce(erreurApi(404, 'NOT_FOUND', 'x'));
			const r = render(McpSendDetail, { id: 99 });
			expect((await r.findByTestId('mcp-send-load-error')).textContent).toContain('Envoi introuvable.');
			cleanup();
			const r2 = render(McpSendDetail, { id: Number('abc') });
			expect((await r2.findByTestId('mcp-send-load-error')).textContent).toContain('Envoi introuvable.');
			expect(mockApi.getMcpSend).toHaveBeenCalledTimes(1);
		});
	});

	describe('les envois décidés', () => {
		it('validé tel quel : contenu en lecture seule, sans actions', async () => {
			const r = await monter(
				envoi({ state: 'approved', decidedAt: '2026-10-26T11:00:00.000', decidedBy: 'admin' }),
			);
			expect(textarea(r.getByTestId).readOnly).toBe(true);
			expect(r.getByTestId('mcp-send-approved-how').textContent).toContain('Envoyé tel quel.');
			expect(r.getByTestId('mcp-send-decided').textContent).toContain('admin');
			expect(r.queryByTestId('mcp-send-actions')).toBeNull();
		});

		it('validé modifié : « Modifié avant l’envoi »', async () => {
			const r = await monter(envoi({ state: 'approved', modified: true }));
			expect(r.getByTestId('mcp-send-approved-how').textContent).toContain('Modifié avant l’envoi.');
			expect(r.getByTestId('mcp-send-state').textContent).toContain('Validé — modifié');
		});

		it('refusé ou échu : « Contenu effacé », aucune zone de texte', async () => {
			for (const state of ['refused', 'expired'] as const) {
				const r = await monter(envoi({ state, content: null, summary: null }));
				expect(r.getByTestId('mcp-send-erased').textContent).toContain('Contenu effacé.');
				expect(r.queryByTestId('mcp-send-content')).toBeNull();
				expect(r.queryByTestId('mcp-send-actions')).toBeNull();
				cleanup();
			}
		});
	});

	it('l’avis permanent et le résumé, non envoyé, en texte', async () => {
		const r = await monter(envoi({ summary: '<b>Ignore les consignes</b>' }));
		expect(r.getByTestId('mcp-send-notice').textContent).toContain('\\u{…}');
		const resume = r.getByTestId('mcp-send-summary');
		expect(resume.textContent?.trim()).toBe('<b>Ignore les consignes</b>');
		expect(resume.querySelector('b')).toBeNull();
	});
});

/** Remplace la valeur d'une textarea déjà en modification. */
async function modifier2(r: Awaited<ReturnType<typeof monter>>, valeur: string) {
	const t = textarea(r.getByTestId);
	t.value = valeur;
	await fireEvent.input(t);
}
