// Story 37-2 (AC-6, U1) — bandeau des corrections de l'AFC : absent pour
// Consultation, en mono-devise et sans notice ; présent sinon (role="status",
// liste, lien) ; « Acquitter » envoie la liste affichée et le bandeau disparaît ;
// un échec de lecture n'affiche rien.

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/svelte';

const { auth, api, settings } = vi.hoisted(() => ({
	auth: { currentUser: { role: 'Admin' } as { role: string } | null },
	api: { getNotices: vi.fn(), acknowledgeNotices: vi.fn() },
	settings: { fetchCompanyCurrent: vi.fn() },
}));

vi.mock('$lib/app/stores/auth.svelte', () => ({ authState: auth }));
vi.mock('./exchange-rates.api', () => api);
vi.mock('$lib/features/settings/settings.api', () => settings);

import ExchangeRateNoticesBanner from './ExchangeRateNoticesBanner.svelte';

const NOTICES = [
	{
		currency: 'USD',
		kind: 'month' as const,
		rateDate: '2024-12-01',
		effectiveRate: '0.8846000000',
		publishedRate: '0.8850000000',
		publishedUnit: 1,
		valueChangedAt: '2024-12-03T09:00:00Z',
	},
	{
		currency: 'EUR',
		kind: 'day' as const,
		rateDate: '2024-06-28',
		effectiveRate: '0.9600000000',
		publishedRate: '0.9612000000',
		publishedUnit: 1,
		valueChangedAt: '2024-07-01T09:00:00Z',
	},
];

function company(multiCurrencyEnabled: boolean) {
	return { company: { multiCurrencyEnabled }, bankAccounts: [] };
}

async function monter(opts: { multi?: boolean; notices?: typeof NOTICES | Error } = {}) {
	settings.fetchCompanyCurrent.mockResolvedValue(company(opts.multi ?? true));
	const n = opts.notices ?? NOTICES;
	if (n instanceof Error) api.getNotices.mockRejectedValue(n);
	else api.getNotices.mockResolvedValue({ notices: n });
	const r = render(ExchangeRateNoticesBanner);
	await new Promise((res) => setTimeout(res, 0));
	return r;
}

describe('ExchangeRateNoticesBanner — Story 37-2', () => {
	beforeEach(() => {
		vi.resetAllMocks();
		auth.currentUser = { role: 'Admin' };
	});

	it('Administrateur, multi-devises, notices : présent, role="status", liste et lien', async () => {
		await monter();
		const bandeau = await screen.findByTestId('exchange-rate-notices-banner');
		expect(bandeau.getAttribute('role')).toBe('status');
		expect(screen.queryByRole('alert')).toBeNull();
		expect(bandeau.textContent).toContain('L’AFC a corrigé 2 cours que votre société a retenus.');
		expect(screen.getByTestId('exchange-rate-notice-USD-month-2024-12-01').textContent).toBe(
			'USD, déc. 2024 : 0.8846 → 0.8850',
		);
		expect(screen.getByTestId('exchange-rate-notice-EUR-day-2024-06-28').textContent).toBe(
			'EUR, 28.06.2024 : 0.9600 → 0.9612',
		);
		expect(screen.getByTestId('exchange-rate-notices-banner-link').getAttribute('href')).toBe(
			'/settings/exchange-rates?date=2024-12-01',
		);
		expect(api.getNotices).toHaveBeenCalledWith({ retry: false });
	});

	it('Comptable : présent', async () => {
		auth.currentUser = { role: 'Comptable' };
		await monter();
		expect(await screen.findByTestId('exchange-rate-notices-banner')).toBeTruthy();
	});

	it('Consultation : absent, rien n’est lu', async () => {
		auth.currentUser = { role: 'Consultation' };
		await monter();
		expect(screen.queryByTestId('exchange-rate-notices-banner')).toBeNull();
		expect(settings.fetchCompanyCurrent).not.toHaveBeenCalled();
		expect(api.getNotices).not.toHaveBeenCalled();
	});

	it('mono-devise : absent, les notices ne sont pas lues', async () => {
		await monter({ multi: false });
		await waitFor(() => expect(settings.fetchCompanyCurrent).toHaveBeenCalled());
		expect(screen.queryByTestId('exchange-rate-notices-banner')).toBeNull();
		expect(api.getNotices).not.toHaveBeenCalled();
	});

	it('sans notice : absent', async () => {
		await monter({ notices: [] });
		await waitFor(() => expect(api.getNotices).toHaveBeenCalled());
		expect(screen.queryByTestId('exchange-rate-notices-banner')).toBeNull();
	});

	it('lecture en échec : absent, sans erreur', async () => {
		await monter({ notices: new Error('réseau') });
		await waitFor(() => expect(api.getNotices).toHaveBeenCalled());
		expect(screen.queryByTestId('exchange-rate-notices-banner')).toBeNull();
	});

	it('« Acquitter » envoie la liste affichée, relit, et le bandeau disparaît', async () => {
		await monter();
		api.acknowledgeNotices.mockResolvedValue(2);
		api.getNotices.mockResolvedValue({ notices: [] });
		await fireEvent.click(await screen.findByTestId('exchange-rate-notices-banner-acknowledge'));
		await waitFor(() =>
			expect(api.acknowledgeNotices).toHaveBeenCalledWith([
				{ currency: 'USD', kind: 'month', rateDate: '2024-12-01', publishedRate: '0.8850000000' },
				{ currency: 'EUR', kind: 'day', rateDate: '2024-06-28', publishedRate: '0.9612000000' },
			]),
		);
		await waitFor(() => expect(screen.queryByTestId('exchange-rate-notices-banner')).toBeNull());
		expect(api.getNotices).toHaveBeenCalledTimes(2);
	});

	it('« Acquitter » en échec : le bandeau reste, avec un message', async () => {
		await monter();
		api.acknowledgeNotices.mockRejectedValue({ code: 'INTERNAL', message: 'x', status: 500 });
		await fireEvent.click(await screen.findByTestId('exchange-rate-notices-banner-acknowledge'));
		expect(await screen.findByTestId('exchange-rate-notices-banner-error')).toBeTruthy();
		expect(screen.getByTestId('exchange-rate-notices-banner')).toBeTruthy();
	});
});
