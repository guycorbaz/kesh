// Story 37-5 (AC-6, AC-9) — le composant de la réévaluation, en ses deux modes :
// relectures de l'aperçu (sélection et cours modifiés seulement, aucune
// relecture à sélection vide), récupération du cours du jour, passation, liens.

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/svelte';

vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string, args?: Record<string, string | number>) =>
		args
			? fallback.replace(/\{\s*\$(\w+)\s*\}/g, (_, n) => String(args[n] ?? ''))
			: fallback,
	i18nLocale: () => 'fr-CH'
}));

const previewMock = vi.fn();
const postMock = vi.fn();
const fetchMock = vi.fn();
vi.mock('./fiscal-years.api', () => ({
	previewRevaluation: (...a: unknown[]) => previewMock(...a),
	postRevaluation: (...a: unknown[]) => postMock(...a)
}));
vi.mock('$lib/features/exchange-rates/exchange-rates.api', () => ({
	fetchRates: (...a: unknown[]) => fetchMock(...a)
}));

import RevaluationPanel from './RevaluationPanel.svelte';
import { line, preview, rate } from './revaluation.test-fixtures';

const fallback = rate({
	rate: '0.8846000000',
	rateKind: 'month',
	rateDate: '2024-12-01',
	defaultKind: 'month',
	defaultRate: '0.8846000000',
	sourceDate: null
});

function deux() {
	return preview({
		rates: [rate(), rate({ currency: 'EUR', rate: '0.9400000000', defaultRate: '0.9400000000' })],
		lines: [
			line(),
			line({
				accountId: 11,
				accountNumber: '1030',
				accountName: 'Wise EUR',
				currency: 'EUR',
				delta: '-0.60',
				fxAccountsOrigin: 'own',
				counterpartAccountId: 51,
				counterpartAccountNumber: '6842',
				counterpartAccountName: 'Pertes PayPal'
			})
		]
	});
}

beforeEach(() => {
	previewMock.mockReset();
	postMock.mockReset();
	fetchMock.mockReset();
});

describe('RevaluationPanel — écran complet', () => {
	it('colonnes, lien vers le grand livre, réévaluations déjà passées et leurs écritures', async () => {
		previewMock.mockResolvedValue(
			preview({
				lines: [
					line({
						previousRevaluationTotal: '31.93',
						previousEntryIds: [77],
						delta: '1.19',
						revaluationTotalAfter: '33.12'
					})
				]
			})
		);
		render(RevaluationPanel, { fiscalYearId: 1, mode: 'full', today: '2025-01-15' });
		const link = await screen.findByTestId('revaluation-ledger-10');
		const href = new URL(link.getAttribute('href') ?? '', 'http://x');
		expect(href.pathname).toBe('/reports');
		expect(href.searchParams.get('tab')).toBe('general-ledger');
		expect(href.searchParams.get('accountId')).toBe('10');
		expect(href.searchParams.get('from')).toBe('2024-01-01');
		expect(href.searchParams.get('to')).toBe('2024-12-31');
		expect(screen.getByTestId('revaluation-previous-entry-77').getAttribute('href')).toBe(
			'/journal-entries/77'
		);
		expect(screen.getByTestId('revaluation-after-10').textContent).toMatch(/33\.12/);
		expect(screen.getByTestId('revaluation-delta-10').textContent).toMatch(/1\.19/);
		expect(screen.getByTestId('revaluation-entry-10').textContent).toMatch(
			/Contrepartie : 6950 Gains de change \(réglage de la société\)/
		);
		expect(screen.getByTestId('revaluation-rate-source-USD').textContent).toMatch(/AFC/);
		expect(previewMock).toHaveBeenCalledWith(1, {});
	});

	it('décocher relit avec la sélection, sans perdre les cours modifiés ; la dernière case ne relit pas', async () => {
		previewMock.mockResolvedValueOnce(deux());
		render(RevaluationPanel, { fiscalYearId: 1, mode: 'full', today: '2025-01-15' });
		const champ = (await screen.findByTestId('revaluation-rate-input-USD')) as HTMLInputElement;
		previewMock.mockResolvedValue(deux());
		await fireEvent.change(champ, { target: { value: '0.95' } });
		await waitFor(() =>
			expect(previewMock).toHaveBeenLastCalledWith(1, { accountIds: [10, 11], rates: { USD: '0.95' } })
		);
		await fireEvent.click(screen.getByTestId('revaluation-select-11'));
		await waitFor(() =>
			expect(previewMock).toHaveBeenLastCalledWith(1, { accountIds: [10], rates: { USD: '0.95' } })
		);
		const appels = previewMock.mock.calls.length;
		await fireEvent.click(screen.getByTestId('revaluation-select-10'));
		expect(previewMock.mock.calls.length).toBe(appels);
		expect((screen.getByTestId('revaluation-submit') as HTMLButtonElement).disabled).toBe(true);
		// Le cours modifié est resté au champ, et le cours retenu s'affiche dessous.
		expect((screen.getByTestId('revaluation-rate-input-USD') as HTMLInputElement).value).toBe('0.95');
		expect(screen.getByTestId('revaluation-rate-default-USD').textContent).toMatch(
			/ne vaut que pour cette passation/
		);
	});

	it('un blocker désactive « Passer les écritures » ; décoché et relu sans blocker, il se réactive', async () => {
		previewMock.mockResolvedValueOnce(
			preview({
				...deux(),
				blockers: [{ code: 'FX_ACCOUNT_INVALID', message: 'Compte de différence invalide', details: {} }]
			})
		);
		render(RevaluationPanel, { fiscalYearId: 1, mode: 'full', today: '2025-01-15' });
		expect(await screen.findByTestId('revaluation-blocker-FX_ACCOUNT_INVALID')).toBeTruthy();
		expect((screen.getByTestId('revaluation-submit') as HTMLButtonElement).disabled).toBe(true);
		previewMock.mockResolvedValueOnce(deux());
		await fireEvent.click(screen.getByTestId('revaluation-select-11'));
		await waitFor(() =>
			expect((screen.getByTestId('revaluation-submit') as HTMLButtonElement).disabled).toBe(false)
		);
	});

	it('la passation envoie les cours affichés des devises cochées, puis montre les liens', async () => {
		previewMock.mockResolvedValue(deux());
		postMock.mockResolvedValue({ entryIds: [91, 92] });
		render(RevaluationPanel, { fiscalYearId: 1, mode: 'full', today: '2025-01-15' });
		await fireEvent.click(await screen.findByTestId('revaluation-submit'));
		await waitFor(() =>
			expect(postMock).toHaveBeenCalledWith(1, {
				accountIds: [10, 11],
				rates: { USD: '0.9119300000', EUR: '0.9400000000' }
			})
		);
		expect((await screen.findByTestId('revaluation-posted')).textContent).toMatch(/#91/);
	});

	it('sélection vide (devise sans cours) : le cours récupéré recharge l\u2019aperçu entier (37-5-CR-P2-1)', async () => {
		const sans = rate({ rate: null, rateKind: null, rateDate: null, defaultKind: null, defaultRate: null, defaultSource: null, sourceDate: null });
		previewMock.mockResolvedValueOnce(
			preview({ rates: [sans], lines: [line({ reason: 'RATE_UNAVAILABLE', selectable: false, selected: false, delta: null, rate: null, targetChf: null })] })
		);
		render(RevaluationPanel, { fiscalYearId: 1, mode: 'full', today: '2025-01-15' });
		fetchMock.mockResolvedValue({});
		previewMock.mockResolvedValueOnce(preview());
		await fireEvent.click(await screen.findByTestId('revaluation-fetch-day-USD'));
		await waitFor(() =>
			expect((screen.getByTestId('revaluation-select-10') as HTMLInputElement).disabled).toBe(false)
		);
		expect(previewMock).toHaveBeenLastCalledWith(1, {});
	});

	it('repli mensuel : signalé, « Récupérer le cours du 31.12.2024 », puis aperçu relu au cours du jour', async () => {
		previewMock.mockResolvedValueOnce(preview({ rates: [fallback] }));
		render(RevaluationPanel, { fiscalYearId: 1, mode: 'full', today: '2025-01-15' });
		expect((await screen.findByTestId('revaluation-rate-kind-USD')).textContent).toMatch(
			/cours du jour de clôture indisponible/
		);
		fetchMock.mockResolvedValue({});
		previewMock.mockResolvedValueOnce(preview());
		await fireEvent.click(screen.getByTestId('revaluation-fetch-day-USD'));
		expect(fetchMock).toHaveBeenCalledWith({ date: '2024-12-31' });
		await waitFor(() =>
			expect((screen.getByTestId('revaluation-rate-input-USD') as HTMLInputElement).value).toBe(
				'0.9119300000'
			)
		);
		// La relecture porte la sélection, et aucun cours (le champ n'était pas modifié).
		expect(previewMock).toHaveBeenLastCalledWith(1, { accountIds: [10], rates: {} });
	});

	it('un 404 RATE_NOT_PUBLISHED de la récupération laisse le repli affiché', async () => {
		previewMock.mockResolvedValueOnce(preview({ rates: [fallback] }));
		render(RevaluationPanel, { fiscalYearId: 1, mode: 'full', today: '2025-01-15' });
		fetchMock.mockRejectedValue({ code: 'RATE_NOT_PUBLISHED', status: 404, message: 'x' });
		await fireEvent.click(await screen.findByTestId('revaluation-fetch-day-USD'));
		expect((await screen.findByTestId('revaluation-fetch-error-USD')).textContent).toMatch(
			/Pas encore publié/
		);
		expect(screen.getByTestId('revaluation-rate-kind-USD').textContent).toMatch(/cours mensuel/);
		expect(previewMock).toHaveBeenCalledTimes(1);
	});

	it('date de fin non passée : pas de bouton du jour, le message', async () => {
		previewMock.mockResolvedValueOnce(preview({ rates: [fallback] }));
		render(RevaluationPanel, { fiscalYearId: 1, mode: 'full', today: '2024-12-31' });
		expect(await screen.findByTestId('revaluation-day-not-published-USD')).toBeTruthy();
		expect(screen.queryByTestId('revaluation-fetch-day-USD')).toBeNull();
	});

	it('genre month sans cours mensuel : ni bouton du jour, « Récupérer le mois »', async () => {
		const sans = rate({ rate: null, rateKind: null, rateDate: null, defaultKind: null, defaultRate: null, defaultSource: null, sourceDate: null });
		previewMock.mockResolvedValueOnce(
			preview({ closingRateKind: 'month', rates: [sans], lines: [line({ reason: 'RATE_UNAVAILABLE', selectable: false, selected: false, delta: null, rate: null, targetChf: null })] })
		);
		render(RevaluationPanel, { fiscalYearId: 1, mode: 'full', today: '2025-06-01' });
		expect(await screen.findByTestId('revaluation-fetch-month-USD')).toBeTruthy();
		expect(screen.queryByTestId('revaluation-fetch-day-USD')).toBeNull();
		expect(screen.getByTestId('revaluation-rate-missing-USD')).toBeTruthy();
	});

	it('compte non imputable à solde : avis qui dit quoi faire, case désactivée (37-5-CR-P2-D1)', async () => {
		previewMock.mockResolvedValueOnce(
			preview({ lines: [line({ reason: 'ACCOUNT_NOT_POSTABLE', selectable: false, selected: false })] })
		);
		render(RevaluationPanel, { fiscalYearId: 1, mode: 'full', today: '2025-01-15' });
		const avis = await screen.findByTestId('revaluation-not-postable-10');
		expect(avis.textContent).toMatch(/rendez-le imputable pour le réévaluer/);
		expect((screen.getByTestId('revaluation-select-10') as HTMLInputElement).disabled).toBe(true);
	});

	it('devise sans cours retenu : aucun champ de cours (le cours tapé ne partirait jamais) (37-5-CR-P1-1)', async () => {
		const sans = rate({ currency: 'EUR', rate: null, rateKind: null, rateDate: null, defaultKind: null, defaultRate: null, defaultSource: null, sourceDate: null });
		previewMock.mockResolvedValueOnce(
			preview({
				rates: [rate(), sans],
				lines: [
					line(),
					line({ accountId: 11, accountNumber: '1030', currency: 'EUR', reason: 'RATE_UNAVAILABLE', selectable: false, selected: false, delta: null, rate: null, targetChf: null })
				]
			})
		);
		render(RevaluationPanel, { fiscalYearId: 1, mode: 'full', today: '2025-01-15' });
		expect(await screen.findByTestId('revaluation-rate-input-USD')).toBeTruthy();
		expect(screen.queryByTestId('revaluation-rate-input-EUR')).toBeNull();
		expect(screen.getByTestId('revaluation-rate-value-EUR').textContent).toMatch(/—/);
		expect(screen.getByTestId('revaluation-rate-missing-EUR')).toBeTruthy();
	});

	it('« Passer les écritures » attend la relecture en cours (37-5-CR-P1-2)', async () => {
		previewMock.mockResolvedValueOnce(deux());
		postMock.mockResolvedValue({ entryIds: [1] });
		render(RevaluationPanel, { fiscalYearId: 1, mode: 'full', today: '2025-01-15' });
		const champ = (await screen.findByTestId('revaluation-rate-input-USD')) as HTMLInputElement;
		let relire: (p: unknown) => void = () => {};
		previewMock.mockImplementationOnce(() => new Promise((r) => (relire = r)));
		await fireEvent.change(champ, { target: { value: '0.95' } });
		const bouton = screen.getByTestId('revaluation-submit') as HTMLButtonElement;
		await waitFor(() => expect(bouton.disabled).toBe(true));
		await fireEvent.click(bouton);
		expect(postMock).not.toHaveBeenCalled();
		previewMock.mockResolvedValue(deux());
		relire(deux());
		await waitFor(() => expect(bouton.disabled).toBe(false));
	});

	it('une relecture échouée après la passation se dit, l’aperçu restant affiché (37-5-CR-P1-4)', async () => {
		previewMock.mockResolvedValueOnce(deux());
		postMock.mockResolvedValue({ entryIds: [7] });
		render(RevaluationPanel, { fiscalYearId: 1, mode: 'full', today: '2025-01-15' });
		previewMock.mockRejectedValueOnce(new Error('réseau'));
		await fireEvent.click(await screen.findByTestId('revaluation-submit'));
		expect((await screen.findByTestId('revaluation-error')).textContent?.trim()).not.toBe('');
		expect(screen.getByTestId('revaluation-panel')).toBeTruthy();
	});

	it('source brute d’une autre valeur (A30) : « cours saisi (ecb) »', async () => {
		previewMock.mockResolvedValueOnce(preview({ rates: [rate({ defaultSource: 'ecb', sourceDate: null })] }));
		render(RevaluationPanel, { fiscalYearId: 1, mode: 'full', today: '2025-01-15' });
		expect((await screen.findByTestId('revaluation-rate-source-USD')).textContent).toMatch(
			/cours saisi \(ecb\)/
		);
	});
});

describe('RevaluationPanel — avis de K9', () => {
	it('lecture seule ; « Réévaluer » passe les écritures montrées (présélection, cours affichés)', async () => {
		previewMock.mockResolvedValue(deux());
		postMock.mockResolvedValue({ entryIds: [5] });
		const onPosted = vi.fn();
		render(RevaluationPanel, { fiscalYearId: 1, mode: 'notice', onPosted, today: '2025-01-15' });
		expect(await screen.findByTestId('revaluation-rate-value-USD')).toBeTruthy();
		expect(screen.queryByTestId('revaluation-rate-input-USD')).toBeNull();
		expect(screen.queryByTestId('revaluation-select-10')).toBeNull();
		const bouton = screen.getByTestId('revaluation-submit');
		expect(bouton.textContent).toMatch(/Réévaluer/);
		await fireEvent.click(bouton);
		await waitFor(() =>
			expect(postMock).toHaveBeenCalledWith(1, {
				accountIds: [10, 11],
				rates: { USD: '0.9119300000', EUR: '0.9400000000' }
			})
		);
		expect(onPosted).toHaveBeenCalledWith([5]);
		// L'avis resté monté relit l'aperçu, sans sélection ni cours (37-5-CR-P1-3).
		await waitFor(() => expect(previewMock).toHaveBeenCalledTimes(2));
		expect(previewMock).toHaveBeenLastCalledWith(1, {});
	});

	it('repli mensuel dans l’avis : signalé et récupérable, puis aperçu et contrôles relus', async () => {
		previewMock.mockResolvedValueOnce(preview({ rates: [fallback] }));
		const onRatesFetched = vi.fn();
		render(RevaluationPanel, { fiscalYearId: 1, mode: 'notice', onRatesFetched, today: '2025-01-15' });
		expect((await screen.findByTestId('revaluation-rate-kind-USD')).textContent).toMatch(/indisponible/);
		fetchMock.mockResolvedValue({});
		previewMock.mockResolvedValueOnce(preview());
		await fireEvent.click(screen.getByTestId('revaluation-fetch-day-USD'));
		await waitFor(() => expect(onRatesFetched).toHaveBeenCalled());
		expect(previewMock).toHaveBeenLastCalledWith(1, {});
	});
});
