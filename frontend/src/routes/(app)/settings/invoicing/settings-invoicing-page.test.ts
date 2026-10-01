// Story 25-4-c3-a1 (#476) — tests Vitest de la page Paramètres → Facturation :
// le sélecteur du compte de différences d'arrondi.
//
// - ne propose que les comptes actifs, imputables, de charge ou de produit ;
// - garde visible le compte déjà choisi, même sorti du filtre (issue #271) ;
// - envoie `defaultRoundingAccountId` à l'enregistrement.
//
// Mocks hoistés avant l'import de la page ; `authState` est le vrai store.

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, fireEvent, waitFor } from '@testing-library/svelte';
import { authState } from '$lib/app/stores/auth.svelte';
import type { AccountResponse } from '$lib/features/accounts/accounts.types';
import type { InvoiceSettingsResponse } from '$lib/features/invoices/invoices.types';

vi.mock('$app/environment', () => ({ browser: true }));
vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string) => fallback,
}));
vi.mock('$lib/shared/utils/notify', () => ({
	notifySuccess: vi.fn(),
	notifyError: vi.fn(),
}));

const getInvoiceSettingsMock = vi.fn<() => Promise<InvoiceSettingsResponse>>();
const updateInvoiceSettingsMock = vi.fn();
vi.mock('$lib/features/invoices/invoices.api', () => ({
	getInvoiceSettings: () => getInvoiceSettingsMock(),
	updateInvoiceSettings: (req: unknown) => updateInvoiceSettingsMock(req),
}));
const fetchAccountsMock = vi.fn<() => Promise<AccountResponse[]>>();
vi.mock('$lib/features/accounts/accounts.api', () => ({
	fetchAccounts: () => fetchAccountsMock(),
}));

import Page from './+page.svelte';

function account(
	id: number,
	number: string,
	accountType: AccountResponse['accountType'],
	overrides: Partial<AccountResponse> = {},
): AccountResponse {
	return {
		id,
		companyId: 1,
		number,
		name: `Compte ${number}`,
		accountType,
		parentId: null,
		active: true,
		postable: true,
		role: null,
		version: 1,
		createdAt: '2026-01-01T00:00:00',
		updatedAt: '2026-01-01T00:00:00',
		...overrides,
	} as AccountResponse;
}

function settings(overrides: Partial<InvoiceSettingsResponse> = {}): InvoiceSettingsResponse {
	return {
		companyId: 1,
		invoiceNumberFormat: 'F-{YEAR}-{SEQ:04}',
		defaultReceivableAccountId: null,
		defaultRevenueAccountId: null,
		defaultVatPayableAccountId: null,
		defaultVatRecoverableAccountId: null,
		defaultVatDecompteAccountId: null,
		defaultSalesJournal: 'Ventes',
		journalEntryDescriptionTemplate: '{YEAR}-{INVOICE_NUMBER}',
		defaultRoundingAccountId: null,
		roundTo5Centimes: true,
		minimumInvoiceAmount: null,
		defaultDiscountAccountId: null,
		defaultBankFeesAccountId: null,
		defaultBadDebtAccountId: null,
		version: 3,
		...overrides,
	};
}

const ACCOUNTS = [
	account(1, '1020', 'Asset'),
	account(2, '2200', 'Liability'),
	account(3, '3000', 'Revenue'),
	account(4, '6940', 'Expense'),
	account(5, '6000', 'Expense', { postable: false }),
	account(6, '6100', 'Expense', { active: false }),
];

function optionValues(select: HTMLSelectElement): string[] {
	return Array.from(select.options).map((o) => o.value);
}

beforeEach(() => {
	authState.login({ userId: '1', username: 'admin', role: 'Admin', expiresIn: 3600 });
	fetchAccountsMock.mockResolvedValue(ACCOUNTS);
	updateInvoiceSettingsMock.mockReset();
});

afterEach(async () => {
	await authState.logout();
	vi.clearAllMocks();
});

describe('Paramètres → Facturation — compte de différences d’arrondi', () => {
	it('ne propose que les charges et produits actifs et imputables', async () => {
		getInvoiceSettingsMock.mockResolvedValue(settings());
		const { findByTestId } = render(Page);
		const select = (await findByTestId('settings-rounding-account')) as HTMLSelectElement;
		const values = optionValues(select);
		expect(values).toContain('3'); // produit
		expect(values).toContain('4'); // charge
		expect(values).not.toContain('1'); // actif
		expect(values).not.toContain('2'); // passif
		expect(values).not.toContain('5'); // non imputable
		expect(values).not.toContain('6'); // archivé
	});

	it('garde visible le compte déjà choisi, même sorti du filtre', async () => {
		getInvoiceSettingsMock.mockResolvedValue(settings({ defaultRoundingAccountId: 6 }));
		const { findByTestId } = render(Page);
		const select = (await findByTestId('settings-rounding-account')) as HTMLSelectElement;
		expect(optionValues(select)).toContain('6');
	});

	it('envoie le compte choisi à l’enregistrement', async () => {
		getInvoiceSettingsMock.mockResolvedValue(settings());
		updateInvoiceSettingsMock.mockResolvedValue(settings({ defaultRoundingAccountId: 4, version: 4 }));
		const { findByTestId, container } = render(Page);
		const select = (await findByTestId('settings-rounding-account')) as HTMLSelectElement;
		const option = Array.from(select.options).find((o) => o.textContent?.includes('6940'))!;
		option.selected = true;
		await fireEvent.change(select);
		await fireEvent.submit(container.querySelector('form')!);
		await waitFor(() => expect(updateInvoiceSettingsMock).toHaveBeenCalledTimes(1));
		expect(updateInvoiceSettingsMock.mock.calls[0][0]).toMatchObject({
			defaultRoundingAccountId: 4,
		});
	});

	// Story 25-4-c4-b (#494) — l'arrondi à 5 centimes, réglable.
	it('la case reflète le réglage et son changement part à l’enregistrement (mutation : champ non envoyé)', async () => {
		getInvoiceSettingsMock.mockResolvedValue(settings({ roundTo5Centimes: true }));
		updateInvoiceSettingsMock.mockResolvedValue(settings({ roundTo5Centimes: false, version: 4 }));
		const { findByTestId, container } = render(Page);
		const box = (await findByTestId('settings-round-to-5-centimes')) as HTMLInputElement;
		await waitFor(() => expect(box.checked).toBe(true));
		await fireEvent.click(box);
		await fireEvent.submit(container.querySelector('form')!);
		await waitFor(() => expect(updateInvoiceSettingsMock).toHaveBeenCalledTimes(1));
		expect(updateInvoiceSettingsMock.mock.calls[0][0]).toMatchObject({
			roundTo5Centimes: false,
		});
	});

	// Story 25-4-e (#495) — le montant minimum : chargé, envoyé, vide = aucun seuil.
	it('le montant minimum est chargé et envoyé ; vide, il part à null (mutation : chaîne vide envoyée)', async () => {
		getInvoiceSettingsMock.mockResolvedValue(settings({ minimumInvoiceAmount: '5.00' }));
		updateInvoiceSettingsMock.mockResolvedValue(settings({ minimumInvoiceAmount: null, version: 4 }));
		const { findByTestId, container } = render(Page);
		const input = (await findByTestId('settings-minimum-invoice-amount')) as HTMLInputElement;
		await waitFor(() => expect(input.value).toBe('5.00'));
		await fireEvent.input(input, { target: { value: '  ' } });
		await fireEvent.submit(container.querySelector('form')!);
		await waitFor(() => expect(updateInvoiceSettingsMock).toHaveBeenCalledTimes(1));
		expect(updateInvoiceSettingsMock.mock.calls[0][0]).toMatchObject({ minimumInvoiceAmount: null });
	});
});

// Story 25-4-d1 (#384) — les comptes des natures d'écart soldé.
describe('Paramètres → Facturation — solde du reste', () => {
	const FIELDS = [
		{ testid: 'settings-discount-account', key: 'defaultDiscountAccountId' },
		{ testid: 'settings-bank-fees-account', key: 'defaultBankFeesAccountId' },
		{ testid: 'settings-bad-debt-account', key: 'defaultBadDebtAccountId' },
	] as const;

	it('chaque sélecteur ne propose que les charges et produits actifs et imputables, et affiche le compte chargé', async () => {
		getInvoiceSettingsMock.mockResolvedValue(
			settings({ defaultDiscountAccountId: 3, defaultBankFeesAccountId: 4, defaultBadDebtAccountId: 6 }),
		);
		const { findByTestId } = render(Page);
		for (const [field, loaded] of [
			[FIELDS[0], '3'],
			[FIELDS[1], '4'],
			[FIELDS[2], '6'], // archivé, mais déjà choisi : gardé visible (#271)
		] as const) {
			const select = (await findByTestId(field.testid)) as HTMLSelectElement;
			await waitFor(() => expect(select.value).toBe(loaded));
			const values = optionValues(select);
			expect(values).toContain('3');
			expect(values).toContain('4');
			expect(values).not.toContain('1');
			expect(values).not.toContain('2');
			expect(values).not.toContain('5');
		}
	});

	it('envoie les trois comptes à l’enregistrement (mutation : un champ non envoyé)', async () => {
		getInvoiceSettingsMock.mockResolvedValue(settings({ defaultBadDebtAccountId: 3 }));
		updateInvoiceSettingsMock.mockResolvedValue(settings({ version: 4 }));
		const { findByTestId, container } = render(Page);
		for (const [testid, number] of [
			['settings-discount-account', '3000'],
			['settings-bank-fees-account', '6940'],
		] as const) {
			const select = (await findByTestId(testid)) as HTMLSelectElement;
			const option = Array.from(select.options).find((o) => o.textContent?.includes(number))!;
			option.selected = true;
			await fireEvent.change(select);
		}
		const badDebt = (await findByTestId('settings-bad-debt-account')) as HTMLSelectElement;
		await waitFor(() => expect(badDebt.value).toBe('3'));
		badDebt.value = '';
		Array.from(badDebt.options)[0].selected = true;
		await fireEvent.change(badDebt);
		await fireEvent.submit(container.querySelector('form')!);
		await waitFor(() => expect(updateInvoiceSettingsMock).toHaveBeenCalledTimes(1));
		expect(updateInvoiceSettingsMock.mock.calls[0][0]).toMatchObject({
			defaultDiscountAccountId: 3,
			defaultBankFeesAccountId: 4,
			defaultBadDebtAccountId: null,
		});
	});
});
