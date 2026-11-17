// Story 37-3b (AC-2 bis) — l'assistant Conversion : aperçu avec la ligne de
// différence, corps envoyé (devise de chaque compte, ligne de différence portée,
// sans `completeDifference`), refus affichés sur la colonne de `details.entry`
// ou au-dessus de l'aperçu, bouton désactivé avec son motif.

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, fireEvent, screen, waitFor } from '@testing-library/svelte';

const { conversionMock } = vi.hoisted(() => ({ conversionMock: vi.fn() }));

vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string, args?: Record<string, unknown>) =>
		fallback.replace(/\{ \$(\w+) \}/g, (_m: string, k: string) => String(args?.[k] ?? ''))
}));
vi.mock('./journal-entries.api', () => ({ createConversion: conversionMock }));
vi.mock('svelte-sonner', () => ({ toast: { error: vi.fn(), success: vi.fn() } }));
vi.mock('$lib/shared/utils/local-date', () => ({ todayLocal: () => '2026-03-05' }));

import ConversionAssistant from './ConversionAssistant.svelte';
import type { AccountResponse } from '$lib/features/accounts/accounts.types';
import { createRateLookupCache } from './rate-lookup';

function account(id: number, number: string, extra: Partial<AccountResponse> = {}): AccountResponse {
	return {
		id,
		companyId: 1,
		number,
		name: `Compte ${number}`,
		accountType: 'Asset',
		parentId: null,
		active: true,
		role: null,
		postable: true,
		letterable: false,
		version: 1,
		createdAt: '',
		updatedAt: '',
		...extra
	};
}

const ACCOUNTS = [
	account(21, '1021', { currency: 'EUR' }),
	account(22, '1022', { currency: 'USD' }),
	account(30, '1090'),
	account(31, '6840', { accountType: 'Revenue' }),
	account(32, '6940', { accountType: 'Expense' })
];

function cache() {
	const rates: Record<string, string> = { EUR: '0.947', USD: '0.8846' };
	return createRateLookupCache({
		lookup: async (currency, date, kind) => ({
			currency,
			kind,
			rateDate: date,
			effectiveRate: rates[currency],
			source: 'afc',
			publishedRate: null,
			publishedUnit: null,
			sourceDate: null
		}),
		fetch: vi.fn(),
		canFetch: () => true,
		today: () => '2026-03-05'
	});
}

async function pickIn(side: 'debit' | 'credit', number: string) {
	const col = screen.getByTestId(`conversion-column-${side}`);
	const input = col.querySelector('input[aria-label="Compte"]') as HTMLInputElement;
	await fireEvent.focus(input);
	const option = screen.getAllByRole('option').find((o) => (o.textContent ?? '').includes(number));
	if (!option) throw new Error(`option ${number} introuvable`);
	await fireEvent.mouseDown(option);
	await fireEvent.blur(input);
}

function mount() {
	return render(ConversionAssistant, {
		props: {
			accounts: ACCOUNTS,
			company: { fxGainAccountId: 31, fxLossAccountId: 32, fxTransitAccountId: 30 },
			openFiscalYears: [{ startDate: '2026-01-01', endDate: '2026-12-31' }],
			cache: cache(),
			onDone: vi.fn()
		}
	});
}

async function fill() {
	mount();
	await pickIn('debit', '1021');
	await pickIn('credit', '1022');
	await waitFor(() =>
		expect((screen.getByTestId('conversion-credit-rate') as HTMLInputElement).value).toBe('0.8846')
	);
	await fireEvent.input(screen.getByTestId('conversion-debit-amount'), { target: { value: '1000' } });
	await fireEvent.input(screen.getByTestId('conversion-credit-amount'), { target: { value: '1078.40' } });
}

describe('ConversionAssistant', () => {
	beforeEach(() => conversionMock.mockReset());

	it('désactivé avec son motif tant que la saisie est incomplète', () => {
		mount();
		expect((screen.getByTestId('conversion-submit') as HTMLButtonElement).disabled).toBe(true);
		expect(screen.getByTestId('conversion-blocker-incomplete')).toBeTruthy();
	});

	it('aperçu : gain 6.95 sur le compte de la société ; corps sans completeDifference', async () => {
		await fill();
		expect(screen.getByTestId('conversion-preview-2-difference').textContent).toContain('6.95');
		conversionMock.mockResolvedValueOnce({ entries: [] });
		await fireEvent.click(screen.getByTestId('conversion-submit'));
		await waitFor(() => expect(conversionMock).toHaveBeenCalled());
		const body = conversionMock.mock.calls[0][0];
		expect(body).not.toHaveProperty('completeDifference');
		expect(body.entries[0].lines[0].foreign).toEqual({
			currency: 'EUR',
			amount: '1000',
			rate: '0.947',
			rateSource: 'afc'
		});
		expect(body.entries[1].lines[2]).toEqual({ accountId: 31, debit: '0', credit: '6.95' });
	});

	it('refus portant details.entry : affiché sur la colonne Crédit', async () => {
		await fill();
		conversionMock.mockRejectedValueOnce({
			code: 'FX_CURRENCY_MISMATCH',
			status: 400,
			message: 'serveur',
			details: { entry: 2, line: 1, expected: 'GBP' }
		});
		await fireEvent.click(screen.getByTestId('conversion-submit'));
		const err = await screen.findByTestId('conversion-credit-error');
		expect(err.textContent).toContain('tenu en GBP');
		expect(screen.queryByTestId('conversion-debit-error')).toBeNull();
	});

	it('refus sans details.entry (transit) : au-dessus de l’aperçu, saisie conservée', async () => {
		await fill();
		conversionMock.mockRejectedValueOnce({
			code: 'FX_CONVERSION_TRANSIT_UNBALANCED',
			status: 400,
			message: 'Le compte de conversion ne revient pas à zéro',
			details: { entry1: '947.00', entry2: '-900.00' }
		});
		await fireEvent.click(screen.getByTestId('conversion-submit'));
		const err = await screen.findByTestId('conversion-global-error');
		expect(err.textContent).toContain('ne revient pas à zéro');
		expect((screen.getByTestId('conversion-debit-amount') as HTMLInputElement).value).toBe('1000');
	});

	it('CHF saisi au lieu du cours : le cours s’en déduit', async () => {
		await fill();
		await fireEvent.input(screen.getByTestId('conversion-credit-chf'), { target: { value: '950.00' } });
		expect((screen.getByTestId('conversion-credit-rate') as HTMLInputElement).value).toBe('0.8809347181');
	});
});
