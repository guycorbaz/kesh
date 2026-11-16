// Story 37-3b (AC-2) — le formulaire d'écriture en société multi-devises : ligne
// en devise (cours proposé, CHF dérivé, corps envoyé), bouton d'écart (une ligne
// ajoutée, aucun appel réseau), une devise par écriture (R7), refus du serveur
// sur la bonne ligne (rang du corps sans lignes vides), « Garder mon cours ».

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, fireEvent, screen, waitFor } from '@testing-library/svelte';

const { createMock, toastError } = vi.hoisted(() => ({
	createMock: vi.fn(),
	toastError: vi.fn()
}));

vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string, args?: Record<string, unknown>) =>
		fallback.replace(/\{ \$(\w+) \}/g, (_m: string, k: string) => String(args?.[k] ?? ''))
}));
vi.mock('$lib/features/onboarding/onboarding.svelte', () => ({
	i18nMsg: (_key: string, fallback: string, args?: Record<string, unknown>) =>
		fallback.replace(/\{ \$(\w+) \}/g, (_m: string, k: string) => String(args?.[k] ?? ''))
}));
vi.mock('./journal-entries.api', () => ({
	createJournalEntry: createMock,
	updateJournalEntry: vi.fn()
}));
vi.mock('svelte-sonner', () => ({
	toast: { error: toastError, success: vi.fn() }
}));
vi.mock('$lib/shared/utils/notify', () => ({
	notifyMissingFiscalYearOrFallback: vi.fn(() => false)
}));
vi.mock('$lib/shared/utils/local-date', () => ({ todayLocal: () => '2026-03-15' }));

import JournalEntryForm from './JournalEntryForm.svelte';
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
		createdAt: '2026-01-01T00:00:00',
		updatedAt: '2026-01-01T00:00:00',
		...extra
	};
}

const CHF = account(1, '1000');
const USD = account(10, '1022', { currency: 'USD' });
const EUR = account(12, '1021', { currency: 'EUR' });
const GAIN = account(31, '6840', { accountType: 'Revenue' });
const LOSS = account(32, '6940', { accountType: 'Expense' });
const ACCOUNTS = [CHF, USD, EUR, GAIN, LOSS];

let lookup: ReturnType<typeof vi.fn>;
function cache(rate = '0.8846', source = 'afc') {
	lookup = vi.fn(async (currency: string, date: string) => ({
		currency,
		kind: 'month',
		rateDate: date,
		effectiveRate: rate,
		source,
		publishedRate: null,
		publishedUnit: null,
		sourceDate: null
	}));
	return createRateLookupCache({
		lookup: lookup as unknown as Parameters<typeof createRateLookupCache>[0]['lookup'],
		fetch: vi.fn(),
		canFetch: () => true,
		today: () => '2026-03-15'
	});
}

async function pick(index: number, number: string) {
	const input = screen.getAllByRole('textbox', { name: 'Compte' })[index];
	await fireEvent.focus(input);
	const option = screen.getAllByRole('option').find((o) => (o.textContent ?? '').includes(number));
	if (!option) throw new Error(`option ${number} introuvable`);
	await fireEvent.mouseDown(option);
	await fireEvent.blur(input);
}

function optionsOf(index: number): string[] {
	const input = screen.getAllByRole('textbox', { name: 'Compte' })[index];
	void fireEvent.focus(input);
	return screen.queryAllByRole('option').map((o) => o.textContent ?? '');
}

function mount(rateLookup = cache()) {
	return render(JournalEntryForm, {
		props: {
			accounts: ACCOUNTS,
			accountsLoadError: false,
			multiCurrency: true,
			companyFx: { fxGainAccountId: 31, fxLossAccountId: 32 },
			rateLookup,
			onSuccess: vi.fn(),
			onCancel: vi.fn()
		}
	});
}

const tick = () => new Promise((r) => setTimeout(r, 0));
/** La valeur d'un champ désigné par son `data-testid`. */
const val = (id: string) => (screen.getByTestId(id) as HTMLInputElement).value;

/** Ligne 0 : USD 100 au débit (cours proposé 0.8846) ; ligne 1 : CHF au crédit. */
async function usdLine(credit: string) {
	await fireEvent.input(document.getElementById('entry-description')!, {
		target: { value: 'Écriture USD' }
	});
	await pick(0, '1022');
	await waitFor(() => expect(['0.8846', '0.88']).toContain(val('journal-entry-line-fx-rate-0')));
	await fireEvent.input(screen.getByTestId('journal-entry-line-fx-debit-0'), { target: { value: '100' } });
	await pick(1, '1000');
	const montants = screen.getAllByPlaceholderText('0.00');
	await fireEvent.input(montants[montants.length - 1], { target: { value: credit } });
}

describe('JournalEntryForm — multi-devises', () => {
	beforeEach(() => {
		createMock.mockReset();
		toastError.mockReset();
	});

	it('ligne USD : cours proposé AFC, CHF dérivé, corps avec la devise du compte et afc', async () => {
		mount();
		await usdLine('88.46');
		expect(val('journal-entry-line-fx-chf-0')).toBe('88.46');
		expect(screen.getByTestId('journal-entry-line-fx-indicator-0').textContent).toContain('AFC');
		createMock.mockResolvedValueOnce({});
		await fireEvent.click(screen.getByRole('button', { name: 'Valider' }));
		await tick();
		expect(createMock).toHaveBeenCalledTimes(1);
		const body = createMock.mock.calls[0][0];
		expect(body.lines[0]).toEqual({
			accountId: 10,
			debit: '88.46',
			credit: '0',
			projectId: null,
			foreign: { currency: 'USD', amount: '100', rate: '0.8846', rateSource: 'afc' }
		});
		expect(body.lines[1].foreign).toBeUndefined();
	});

	it('cours saisi par la société (source manual) : proposé, indicateur « cours saisi », envoyé manual', async () => {
		mount(cache('0.88', 'manual'));
		await usdLine('88.00');
		expect(screen.getByTestId('journal-entry-line-fx-indicator-0').textContent).toContain('cours saisi');
		createMock.mockResolvedValueOnce({});
		await fireEvent.click(screen.getByRole('button', { name: 'Valider' }));
		await tick();
		expect(createMock.mock.calls[0][0].lines[0].foreign.rateSource).toBe('manual');
	});

	it('bouton d’écart : un clic ajoute exactement une ligne, aucun appel réseau', async () => {
		mount();
		await usdLine('88.40');
		const before = screen.getAllByRole('textbox', { name: 'Compte' }).length;
		const calls = lookup.mock.calls.length;
		await fireEvent.click(screen.getByTestId('journal-entry-add-fx-difference'));
		expect(screen.getAllByRole('textbox', { name: 'Compte' })).toHaveLength(before + 1);
		expect(createMock).not.toHaveBeenCalled();
		expect(lookup.mock.calls.length).toBe(calls);
		// La ligne ajoutée équilibre : gain 0.06 au crédit.
		expect((screen.getByRole('button', { name: 'Valider' }) as HTMLButtonElement).disabled).toBe(false);
	});

	it('R7 : avec une ligne USD, l’autre ligne n’offre pas de compte EUR ; le lien Conversion apparaît', async () => {
		mount();
		await pick(0, '1022');
		const opts = optionsOf(1);
		expect(opts.some((o) => o.includes('1021'))).toBe(false);
		expect(opts.some((o) => o.includes('1000'))).toBe(true);
		expect(screen.getByTestId('journal-entry-conversion-link')).toBeTruthy();
	});

	it('refus sur la ligne fautive, une ligne vide précédant (rang du corps envoyé)', async () => {
		mount();
		// Ligne 0 laissée vide ; ligne 1 : USD ; ligne 2 : CHF.
		await fireEvent.input(document.getElementById('entry-description')!, {
			target: { value: 'Écriture USD' }
		});
		await fireEvent.click(screen.getByRole('button', { name: '+ Ajouter une ligne' }));
		await pick(1, '1022');
		await waitFor(() => expect(val('journal-entry-line-fx-rate-1')).toBe('0.8846'));
		await fireEvent.input(screen.getByTestId('journal-entry-line-fx-debit-1'), { target: { value: '100' } });
		await pick(2, '1000');
		const montants = screen.getAllByPlaceholderText('0.00');
		await fireEvent.input(montants[montants.length - 1], { target: { value: '88.46' } });
		createMock.mockRejectedValueOnce({
			code: 'FX_CURRENCY_MISMATCH',
			status: 400,
			message: 'serveur',
			details: { line: 1, expected: 'EUR', provided: 'USD' }
		});
		await fireEvent.click(screen.getByRole('button', { name: 'Valider' }));
		await tick();
		// Le corps omet la ligne vide : son rang 1 est la ligne 1 du formulaire.
		expect(createMock.mock.calls[0][0].lines).toHaveLength(2);
		expect(screen.getByTestId('journal-entry-line-refusal-1').textContent).toContain('Rechargez la page');
		expect(screen.queryByTestId('journal-entry-line-refusal-0')).toBeNull();
	});

	it('FX_RATE_NOT_AFC à expectedRate null : seule « Garder mon cours » ; la ligne repart manual', async () => {
		mount();
		await usdLine('88.46');
		createMock.mockRejectedValueOnce({
			code: 'FX_RATE_NOT_AFC',
			status: 400,
			message: 'serveur',
			details: { line: 1, expectedRate: null }
		});
		await fireEvent.click(screen.getByRole('button', { name: 'Valider' }));
		await tick();
		expect(screen.queryByTestId('journal-entry-line-take-expected-0')).toBeNull();
		await fireEvent.click(screen.getByTestId('journal-entry-line-keep-rate-0'));
		createMock.mockResolvedValueOnce({});
		await fireEvent.click(screen.getByRole('button', { name: 'Valider' }));
		await tick();
		expect(createMock.mock.calls[1][0].lines[0].foreign.rateSource).toBe('manual');
		expect(createMock.mock.calls[1][0].lines[0].debit).toBe('88.46');
	});

	it('FX_RATE_NOT_AFC avec cours attendu : « Reprendre le cours attendu » passe la ligne en afc', async () => {
		mount();
		await usdLine('88.46');
		createMock.mockRejectedValueOnce({
			code: 'FX_RATE_NOT_AFC',
			status: 400,
			message: 'serveur',
			details: { line: 1, expectedRate: '0.885' }
		});
		await fireEvent.click(screen.getByRole('button', { name: 'Valider' }));
		await tick();
		await fireEvent.click(screen.getByTestId('journal-entry-line-take-expected-0'));
		expect(val('journal-entry-line-fx-chf-0')).toBe('88.50');
		const montants = screen.getAllByPlaceholderText('0.00');
		await fireEvent.input(montants[montants.length - 1], { target: { value: '88.50' } });
		createMock.mockResolvedValueOnce({});
		await fireEvent.click(screen.getByRole('button', { name: 'Valider' }));
		await tick();
		expect(createMock.mock.calls[1][0].lines[0].foreign).toMatchObject({ rate: '0.885', rateSource: 'afc' });
	});

	it('MULTI_CURRENCY_DISABLED (exercice mono-devise) : message traduit, saisie conservée', async () => {
		mount();
		await usdLine('88.46');
		createMock.mockRejectedValueOnce({
			code: 'MULTI_CURRENCY_DISABLED',
			status: 409,
			message: 'serveur',
			details: { fiscalYearId: 3 }
		});
		await fireEvent.click(screen.getByRole('button', { name: 'Valider' }));
		await tick();
		expect(screen.getByTestId('journal-entry-form-refusal').textContent).toContain('mono-devise');
		expect(val('journal-entry-line-fx-debit-0')).toBe('100');
	});
});
