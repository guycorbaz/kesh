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
import { todayInZurich } from '$lib/shared/utils/local-date';

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

	it('409 d’une AUTRE période : l’avis dit la période suivie, jusqu’au compte rendu (revue P1, M1)', async () => {
		startMock.mockRejectedValue({
			code: 'PROVIDER_DOWNLOAD_RUNNING',
			message: 'déjà en cours',
			status: 409,
			details: { provider: 'paypal', jobId: 32, from: '2026-03-01', to: '2026-03-30' },
		});
		getMock
			.mockResolvedValueOnce(running(1, 2))
			.mockResolvedValue({ ...running(2, 2), state: 'done', result: { inserted: 1, lastRefreshed: null } });
		monter();
		await waitFor(() => expect(q('provider-download-start')).not.toBeNull());
		expect(q('provider-download-resumed')).toBeNull();
		await fireEvent.click(q('provider-download-start') as HTMLElement);
		await waitFor(() =>
			expect(q('provider-download-resumed')?.textContent).toContain(
				'Un téléchargement du 01.03.2026 au 30.03.2026 est déjà en cours pour ce compte : son suivi est repris.',
			),
		);
		await waitFor(() => expect(q('test-report')).not.toBeNull());
		// Compte rendu affiché : l'avis passe au passé (revue P2, L6).
		expect(q('provider-download-resumed')?.textContent).toContain(
			'Le téléchargement du 01.03.2026 au 30.03.2026, déjà en cours pour ce compte, a été suivi à la place du vôtre : lancez maintenant votre période.',
		);
		expect(q('provider-download-resumed')?.textContent).not.toContain('quand il sera terminé');
		// « Nouvelle période » : l'avis s'efface avec le formulaire.
		await fireEvent.click(q('provider-download-again') as HTMLElement);
		await waitFor(() => expect(q('provider-download-resumed')).toBeNull());
	});

	it('409 de la MÊME période : le suivi reprend, sans avis', async () => {
		startMock.mockRejectedValue({
			code: 'PROVIDER_DOWNLOAD_RUNNING',
			message: 'déjà en cours',
			status: 409,
			details: { provider: 'paypal', jobId: 33, from: '2026-01-01', to: '2026-11-22' },
		});
		getMock.mockResolvedValue(running(1, 2));
		monter();
		await waitFor(() => expect(q('provider-download-start')).not.toBeNull());
		await fireEvent.click(q('provider-download-start') as HTMLElement);
		await waitFor(() => expect(getMock).toHaveBeenCalledWith('paypal', 33));
		expect(q('provider-download-running')).not.toBeNull();
		expect(q('provider-download-resumed')).toBeNull();
	});

	it('409 sur une fin future : la fin lue (aujourd’hui) est la même période, sans avis ; le champ de fin borné à aujourd’hui (revue P2, L1)', async () => {
		const today = todayInZurich();
		startMock.mockRejectedValue({
			code: 'PROVIDER_DOWNLOAD_RUNNING',
			message: 'déjà en cours',
			status: 409,
			details: { provider: 'paypal', jobId: 34, from: '2026-01-01', to: today },
		});
		getMock.mockResolvedValue(running(1, 2));
		monter({ defaultPeriod: { from: '2026-01-01', to: '2026-11-22' } });
		await waitFor(() => expect(q('provider-download-to')).not.toBeNull());
		expect((q('provider-download-to') as HTMLInputElement).max).toBe(today);
		await fireEvent.input(q('provider-download-to') as HTMLInputElement, { target: { value: '2099-12-31' } });
		await fireEvent.click(q('provider-download-start') as HTMLElement);
		await waitFor(() => expect(getMock).toHaveBeenCalledWith('paypal', 34));
		expect(startMock).toHaveBeenCalledWith('paypal', { account: 'PayPal www', from: '2026-01-01', to: '2099-12-31' });
		expect(q('provider-download-running')).not.toBeNull();
		expect(q('provider-download-resumed')).toBeNull();
	});

	it('409 d’une autre période puis suivi perdu : « suivi perdu » nomme la période suivie, sans l’avis au futur (revue P2, L6)', async () => {
		startMock.mockRejectedValue({
			code: 'PROVIDER_DOWNLOAD_RUNNING',
			message: 'déjà en cours',
			status: 409,
			details: { provider: 'paypal', jobId: 35, from: '2026-03-01', to: '2026-03-30' },
		});
		getMock.mockRejectedValue({ code: 'NOT_FOUND', message: 'x', status: 404 });
		monter();
		await waitFor(() => expect(q('provider-download-start')).not.toBeNull());
		await fireEvent.click(q('provider-download-start') as HTMLElement);
		await waitFor(() => expect(q('provider-download-lost')).not.toBeNull());
		expect(q('provider-download-lost')?.textContent).toContain(
			'Suivi perdu du téléchargement du 01.03.2026 au 30.03.2026 : relancez votre période, rien n’est doublé.',
		);
		expect(q('provider-download-resumed')).toBeNull();
	});

	it('ancien POST qui répond après une réouverture : le bouton reste désactivé pendant le POST suivant (revue P2, L3)', async () => {
		const pending: Array<(v: { jobId: number }) => void> = [];
		startMock.mockImplementation(() => new Promise((r) => pending.push(r)));
		getMock.mockResolvedValue(running(1, 3));
		const r = monter();
		await waitFor(() => expect(q('provider-download-start')).not.toBeNull());
		await fireEvent.click(q('provider-download-start') as HTMLElement);
		await waitFor(() => expect(startMock).toHaveBeenCalledTimes(1));
		await r.rerender({ open: false });
		await r.rerender({ open: true });
		await waitFor(() => expect((q('provider-download-start') as HTMLButtonElement)?.disabled).toBe(false));
		await fireEvent.click(q('provider-download-start') as HTMLElement);
		await waitFor(() => expect(startMock).toHaveBeenCalledTimes(2));
		expect((q('provider-download-start') as HTMLButtonElement).disabled).toBe(true);
		// L'ancien POST répond : sa génération est passée, le bouton reste désactivé.
		pending[0]({ jobId: 60 });
		await new Promise((res) => setTimeout(res, 40));
		expect((q('provider-download-start') as HTMLButtonElement).disabled).toBe(true);
		expect(getMock).not.toHaveBeenCalled();
		pending[1]({ jobId: 61 });
		await waitFor(() => expect(getMock).toHaveBeenCalledWith('paypal', 61));
		expect(getMock).not.toHaveBeenCalledWith('paypal', 60);
	});

	it('lancement réussi puis « Nouvelle période » : le bouton redevient actif (revue P3, HIGH-1)', async () => {
		startMock.mockResolvedValue({ jobId: 70 });
		getMock.mockResolvedValue({ ...running(1, 1), state: 'done', result: { inserted: 2, lastRefreshed: null } });
		monter();
		await waitFor(() => expect(q('provider-download-start')).not.toBeNull());
		await fireEvent.click(q('provider-download-start') as HTMLElement);
		await waitFor(() => expect(q('test-report')).not.toBeNull());
		await fireEvent.click(q('provider-download-again') as HTMLElement);
		await waitFor(() => expect(q('provider-download-start')).not.toBeNull());
		const bouton = q('provider-download-start') as HTMLButtonElement;
		expect(bouton.disabled, 'bouton actif après un lancement réussi').toBe(false);
		expect(bouton.textContent).toContain('Télécharger');
		await fireEvent.click(bouton);
		await waitFor(() => expect(startMock).toHaveBeenCalledTimes(2));
	});

	it('409 repris puis « Nouvelle période » : le bouton redevient actif (revue P3, HIGH-1)', async () => {
		startMock.mockRejectedValue({
			code: 'PROVIDER_DOWNLOAD_RUNNING',
			message: 'déjà en cours',
			status: 409,
			details: { provider: 'paypal', jobId: 71, from: '2026-03-01', to: '2026-03-30' },
		});
		getMock.mockResolvedValue({ ...running(1, 1), state: 'done', result: { inserted: 1, lastRefreshed: null } });
		monter();
		await waitFor(() => expect(q('provider-download-start')).not.toBeNull());
		await fireEvent.click(q('provider-download-start') as HTMLElement);
		await waitFor(() => expect(q('test-report')).not.toBeNull());
		await fireEvent.click(q('provider-download-again') as HTMLElement);
		await waitFor(() => expect(q('provider-download-start')).not.toBeNull());
		expect((q('provider-download-start') as HTMLButtonElement).disabled, 'bouton actif après un 409 repris').toBe(
			false,
		);
	});

	it('fermeture pendant le POST : sa réponse ne lance aucun suivi (revue P1, L1)', async () => {
		let resolveStart: (v: { jobId: number }) => void = () => {};
		startMock.mockImplementation(() => new Promise((r) => (resolveStart = r)));
		getMock.mockResolvedValue(running(1, 3));
		const r = monter();
		await waitFor(() => expect(q('provider-download-start')).not.toBeNull());
		await fireEvent.click(q('provider-download-start') as HTMLElement);
		await waitFor(() => expect(startMock).toHaveBeenCalled());
		await r.rerender({ open: false });
		resolveStart({ jobId: 40 });
		await new Promise((res) => setTimeout(res, 40));
		expect(getMock).not.toHaveBeenCalled();
	});

	it('même chose pour une reprise sur 409 arrivée après la fermeture (revue P1, L1)', async () => {
		let rejectStart: (e: unknown) => void = () => {};
		startMock.mockImplementation(() => new Promise((_, rej) => (rejectStart = rej)));
		getMock.mockResolvedValue(running(1, 3));
		const r = monter();
		await waitFor(() => expect(q('provider-download-start')).not.toBeNull());
		await fireEvent.click(q('provider-download-start') as HTMLElement);
		await waitFor(() => expect(startMock).toHaveBeenCalled());
		await r.rerender({ open: false });
		rejectStart({ code: 'PROVIDER_DOWNLOAD_RUNNING', message: 'x', status: 409, details: { jobId: 41 } });
		await new Promise((res) => setTimeout(res, 40));
		expect(getMock).not.toHaveBeenCalled();
	});

	it('`done` sans compte rendu : « suivi perdu », le suivi s’arrête (revue P1, L2)', async () => {
		startMock.mockResolvedValue({ jobId: 42 });
		getMock.mockResolvedValue({ ...running(3, 3), state: 'done', result: null });
		const { onDone } = monter();
		await waitFor(() => expect(q('provider-download-start')).not.toBeNull());
		await fireEvent.click(q('provider-download-start') as HTMLElement);
		await waitFor(() => expect(q('provider-download-lost')).not.toBeNull());
		await new Promise((res) => setTimeout(res, 40));
		expect(getMock).toHaveBeenCalledTimes(1);
		expect(onDone).not.toHaveBeenCalled();
	});

	it('`onDone` qui lève : appelé une seule fois, le suivi s’arrête (revue P1, L2)', async () => {
		const error = vi.spyOn(console, 'error').mockImplementation(() => {});
		startMock.mockResolvedValue({ jobId: 43 });
		getMock.mockResolvedValue({ ...running(1, 1), state: 'done', result: { inserted: 3, lastRefreshed: null } });
		const onDone = vi.fn(() => {
			throw new Error('rechargement en échec');
		});
		monter({ onDone });
		await waitFor(() => expect(q('provider-download-start')).not.toBeNull());
		await fireEvent.click(q('provider-download-start') as HTMLElement);
		await waitFor(() => expect(q('test-report')?.textContent).toBe('3 insérée(s)'));
		await new Promise((res) => setTimeout(res, 40));
		expect(onDone).toHaveBeenCalledTimes(1);
		expect(getMock).toHaveBeenCalledTimes(1);
		error.mockRestore();
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
