// Story 38-12b (AC-2, AC-3) — le dialogue générique : période proposée, refus par
// code, reprise du suivi sur 409, progression (barre et compteur), pause, compte
// rendu et « à jour jusqu'à », échec traduit, suivi perdu (404, erreurs réseau),
// arrêt du suivi à la fermeture et au démontage.

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, fireEvent, waitFor } from '@testing-library/svelte';
import { createRawSnippet } from 'svelte';

vi.mock('$app/environment', () => ({ browser: true }));
vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string, args?: Record<string, string | number>) =>
		args ? fallback.replace(/\{\s*\$(\w+)\s*\}/g, (_, k) => String(args[k] ?? '')) : fallback,
}));
// L'intervalle réel (1 s) ralentirait les tests sans rien prouver de plus.
vi.mock('./provider-download', async (importOriginal) => ({
	...(await importOriginal<typeof import('./provider-download')>()),
	POLL_INTERVAL_MS: 5,
}));

const startMock = vi.fn();
const getMock = vi.fn();
vi.mock('./provider-download.api', async (importOriginal) => ({
	...(await importOriginal<typeof import('./provider-download.api')>()),
	startProviderDownload: (...a: unknown[]) => startMock(...a),
	getProviderDownload: (...a: unknown[]) => getMock(...a),
}));

import ProviderDownloadDialog from './ProviderDownloadDialog.svelte';

// `inserted` facultatif : le composant est générique (`R extends { lastRefreshed? }`), et le
// snippet doit accepter la forme de base au contrôle de types.
interface Result {
	inserted?: number;
	lastRefreshed?: string | null;
}

const report = createRawSnippet((r: () => Result) => ({
	render: () => `<p data-testid="test-report">${r().inserted} insérée(s)</p>`,
}));

const running = (done: number, total: number | null, paused: number | null = null) => ({
	state: 'running',
	steps: { done, total },
	pausedUntilSeconds: paused,
	result: null,
	error: null,
});

function monter(props: Record<string, unknown> = {}) {
	const onDone = vi.fn();
	const onOpenChange = vi.fn();
	const r = render(ProviderDownloadDialog, {
		open: true,
		onOpenChange,
		provider: 'paypal',
		account: 'PayPal www',
		title: 'Télécharger depuis PayPal',
		stepLabel: 'fenêtre',
		notice: 'Texte permanent du prestataire.',
		defaultPeriod: { from: '2026-01-01', to: '2026-11-22' },
		lineLabel: (code: string) => `libellé ${code}`,
		report,
		onDone,
		...props,
	});
	return { ...r, onDone, onOpenChange };
}

const q = (testId: string) => document.querySelector(`[data-testid="${testId}"]`);

beforeEach(() => {
	startMock.mockReset();
	getMock.mockReset();
});

afterEach(() => {
	document.body.innerHTML = '';
});

describe('ProviderDownloadDialog', () => {
	it('propose la période, montre le texte permanent, lance la tâche du compte', async () => {
		startMock.mockResolvedValue({ jobId: 4 });
		getMock.mockResolvedValue({ ...running(0, 3), state: 'done', result: { inserted: 2, lastRefreshed: null } });
		monter();
		await waitFor(() => expect(q('provider-download-from')).not.toBeNull());
		expect((q('provider-download-from') as HTMLInputElement).value).toBe('2026-01-01');
		expect((q('provider-download-to') as HTMLInputElement).value).toBe('2026-11-22');
		expect(q('provider-download-notice')?.textContent).toContain('Texte permanent');
		expect(q('provider-download-sandbox')).toBeNull();
		await fireEvent.input(q('provider-download-from') as HTMLInputElement, { target: { value: '2026-09-01' } });
		await fireEvent.click(q('provider-download-start') as HTMLElement);
		await waitFor(() => expect(startMock).toHaveBeenCalled());
		expect(startMock).toHaveBeenCalledWith('paypal', { account: 'PayPal www', from: '2026-09-01', to: '2026-11-22' });
		await waitFor(() => expect(getMock).toHaveBeenCalledWith('paypal', 4));
	});

	it('mention « bac à sable »', async () => {
		monter({ sandbox: true });
		await waitFor(() => expect(q('provider-download-sandbox')?.textContent).toContain('bac à sable'));
	});

	it('suit la tâche : « fenêtre n/N », barre, pause, puis compte rendu et « à jour jusqu’à »', async () => {
		startMock.mockResolvedValue({ jobId: 9 });
		let resolveDone: () => void = () => {};
		const gate = new Promise<void>((r) => (resolveDone = r));
		getMock
			.mockResolvedValueOnce(running(1, 3))
			.mockResolvedValueOnce(running(2, 3, 7))
			.mockImplementation(async () => {
				await gate;
				return {
					state: 'done',
					steps: { done: 3, total: 3 },
					pausedUntilSeconds: null,
					result: { inserted: 12, lastRefreshed: '2026-11-22T06:30:00+00:00' },
					error: null,
				};
			});
		const { onDone } = monter();
		await waitFor(() => expect(q('provider-download-start')).not.toBeNull());
		await fireEvent.click(q('provider-download-start') as HTMLElement);
		await waitFor(() => expect(q('provider-download-paused')?.textContent).toContain('reprise dans 7 s'));
		expect(q('provider-download-progress')?.textContent).toBe('fenêtre 2/3');
		expect(q('provider-download-bar')?.getAttribute('aria-valuenow')).toBe('67');
		resolveDone();
		await waitFor(() => expect(q('test-report')?.textContent).toBe('12 insérée(s)'));
		// 06:30 UTC = 07:30 à Zurich (heure d'hiver).
		expect(q('provider-download-last-refreshed')?.textContent).toContain('22.11.2026 07:30');
		expect(onDone).toHaveBeenCalledWith({ inserted: 12, lastRefreshed: '2026-11-22T06:30:00+00:00' });
		const calls = getMock.mock.calls.length;
		await new Promise((r) => setTimeout(r, 40));
		expect(getMock.mock.calls.length).toBe(calls);
	});

	it('total inconnu : compteur sans barre', async () => {
		startMock.mockResolvedValue({ jobId: 1 });
		getMock.mockResolvedValue(running(4, null));
		monter();
		await waitFor(() => expect(q('provider-download-start')).not.toBeNull());
		await fireEvent.click(q('provider-download-start') as HTMLElement);
		await waitFor(() => expect(q('provider-download-progress')?.textContent).toBe('fenêtre 4'));
		expect(q('provider-download-bar')).toBeNull();
	});

	it('409 PROVIDER_DOWNLOAD_RUNNING : reprend le suivi de la tâche en cours, sans erreur', async () => {
		startMock.mockRejectedValue({
			code: 'PROVIDER_DOWNLOAD_RUNNING',
			message: 'déjà en cours',
			status: 409,
			details: { provider: 'paypal', jobId: 31 },
		});
		getMock.mockResolvedValue(running(1, 2));
		monter();
		await waitFor(() => expect(q('provider-download-start')).not.toBeNull());
		await fireEvent.click(q('provider-download-start') as HTMLElement);
		await waitFor(() => expect(getMock).toHaveBeenCalledWith('paypal', 31));
		expect(q('provider-download-refusal')).toBeNull();
		expect(q('provider-download-running')).not.toBeNull();
	});

	it('refus par code : le motif, le formulaire reste', async () => {
		startMock.mockRejectedValue({
			code: 'PROVIDER_PERIOD_INVALID',
			message: 'x',
			status: 400,
			details: { provider: 'paypal', reason: 'too_old' },
		});
		monter();
		await waitFor(() => expect(q('provider-download-start')).not.toBeNull());
		await fireEvent.click(q('provider-download-start') as HTMLElement);
		await waitFor(() => expect(q('provider-download-refusal')?.textContent).toContain('plus de trois ans'));
		expect(getMock).not.toHaveBeenCalled();
		expect(q('provider-download-start')).not.toBeNull();
	});

	it('échec : le motif traduit, la fenêtre, les lignes, « rien n’a été enregistré »', async () => {
		startMock.mockResolvedValue({ jobId: 2 });
		getMock.mockResolvedValue({
			state: 'failed',
			steps: { done: 1, total: 3 },
			pausedUntilSeconds: null,
			result: null,
			error: {
				code: 'PROVIDER_RESPONSE_INVALID',
				message: 'serveur',
				details: { provider: 'paypal', window: '2026-01-31…2026-03-01', lines: [{ line: 2, field: 'transaction_status', code: 'STATUS', value: 'X' }] },
			},
		});
		const { onDone } = monter();
		await waitFor(() => expect(q('provider-download-start')).not.toBeNull());
		await fireEvent.click(q('provider-download-start') as HTMLElement);
		await waitFor(() => expect(q('provider-download-failed')).not.toBeNull());
		const lines = [...document.querySelectorAll('[data-testid="provider-download-failed-line"]')].map((e) => e.textContent);
		expect(lines).toEqual([
			'La réponse du prestataire n’a pas pu être lue.',
			'Transaction 2, transaction_status : libellé STATUS',
			'Période 2026-01-31…2026-03-01.',
			'Rien n’a été enregistré : relancez le téléchargement.',
		]);
		expect(onDone).not.toHaveBeenCalled();
	});

	it('404 au suivi : « suivi perdu »', async () => {
		startMock.mockResolvedValue({ jobId: 3 });
		getMock.mockRejectedValue({ code: 'NOT_FOUND', message: 'x', status: 404 });
		monter();
		await waitFor(() => expect(q('provider-download-start')).not.toBeNull());
		await fireEvent.click(q('provider-download-start') as HTMLElement);
		await waitFor(() => expect(q('provider-download-lost')?.textContent).toContain('rien n’est doublé'));
		expect(getMock).toHaveBeenCalledTimes(1);
	});

	it('erreurs réseau : réessaie, puis « suivi perdu » au troisième échec consécutif', async () => {
		startMock.mockResolvedValue({ jobId: 3 });
		getMock
			.mockRejectedValueOnce(new TypeError('Failed to fetch'))
			.mockResolvedValueOnce(running(1, 3))
			.mockRejectedValue(new TypeError('Failed to fetch'));
		monter();
		await waitFor(() => expect(q('provider-download-start')).not.toBeNull());
		await fireEvent.click(q('provider-download-start') as HTMLElement);
		await waitFor(() => expect(q('provider-download-lost')).not.toBeNull());
		// Une erreur, une réponse (compte remis à zéro), puis trois erreurs.
		expect(getMock).toHaveBeenCalledTimes(5);
	});

	it('la fermeture du dialogue arrête le suivi (aucune interrogation orpheline)', async () => {
		startMock.mockResolvedValue({ jobId: 5 });
		getMock.mockResolvedValue(running(1, 3));
		const r = monter();
		await waitFor(() => expect(q('provider-download-start')).not.toBeNull());
		await fireEvent.click(q('provider-download-start') as HTMLElement);
		await waitFor(() => expect(getMock.mock.calls.length).toBeGreaterThanOrEqual(2));
		await r.rerender({ open: false });
		await new Promise((res) => setTimeout(res, 20));
		const calls = getMock.mock.calls.length;
		await new Promise((res) => setTimeout(res, 40));
		expect(getMock.mock.calls.length).toBe(calls);
	});

	it('le démontage arrête le suivi', async () => {
		startMock.mockResolvedValue({ jobId: 6 });
		getMock.mockResolvedValue(running(1, 3));
		const r = monter();
		await waitFor(() => expect(q('provider-download-start')).not.toBeNull());
		await fireEvent.click(q('provider-download-start') as HTMLElement);
		await waitFor(() => expect(getMock.mock.calls.length).toBeGreaterThanOrEqual(2));
		r.unmount();
		await new Promise((res) => setTimeout(res, 20));
		const calls = getMock.mock.calls.length;
		await new Promise((res) => setTimeout(res, 40));
		expect(getMock.mock.calls.length).toBe(calls);
	});
});
