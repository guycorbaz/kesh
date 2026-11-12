// Story 34-11a2 (#972, AC-7) — la carte de crédit à l'écran des comptes
// bancaires : choix du type à la création, champs d'une carte, liste réduite
// aux passifs, payloads sans IBAN (création et modification — points hérités
// E5/B7 de la revue P1 de la 34-11a1), type affiché et figé en édition.
//
// Patron et mocks de `bank-accounts-page.test.ts` ; un fichier à part pour ne
// pas mêler les deux histoires.

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, fireEvent, waitFor } from '@testing-library/svelte';
import type { AccountResponse } from '$lib/features/accounts/accounts.types';
import type { BankAccountSummary } from '$lib/features/bank-accounts/bank-accounts.api';

vi.mock('$app/environment', () => ({ browser: true }));

vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string, args?: Record<string, string | number>) =>
		args ? fallback.replace(/\{\s*\$(\w+)\s*\}/g, (_, k) => String(args[k] ?? '')) : fallback,
}));

vi.mock('svelte-sonner', () => ({
	toast: { success: vi.fn(), error: vi.fn() },
}));

const fetchAccountsMock = vi.fn<() => Promise<AccountResponse[]>>();
vi.mock('$lib/features/accounts/accounts.api', () => ({
	fetchAccounts: () => fetchAccountsMock(),
}));

const listBankAccountsMock = vi.fn<() => Promise<BankAccountSummary[]>>();
const createBankAccountMock = vi.fn<(payload: unknown) => Promise<unknown>>();
const updateBankAccountMock = vi.fn<(id: number, payload: unknown) => Promise<unknown>>();
vi.mock('$lib/features/bank-accounts/bank-accounts.api', () => ({
	listBankAccounts: () => listBankAccountsMock(),
	createBankAccount: (payload: unknown) => createBankAccountMock(payload),
	updateBankAccount: (id: number, payload: unknown) => updateBankAccountMock(id, payload),
	archiveBankAccount: vi.fn(),
	closeBankAccount: vi.fn(),
	reopenBankAccount: vi.fn(),
	updateBankAccountJournalLink: vi.fn(),
}));

const getInvoiceSettingsMock = vi.fn<() => Promise<Record<string, unknown>>>();
vi.mock('$lib/features/invoices/invoices.api', () => ({
	getInvoiceSettings: () => getInvoiceSettingsMock(),
}));

import Page from './+page.svelte';

function account(overrides: Partial<AccountResponse>): AccountResponse {
	return {
		id: 0,
		companyId: 1,
		number: '',
		name: '',
		accountType: 'Asset',
		parentId: null,
		active: true,
		role: null,
		postable: true,
		letterable: false,
		version: 1,
		createdAt: '2026-01-01T00:00:00',
		updatedAt: '2026-01-01T00:00:00',
		...overrides,
	};
}

const ACTIF = account({ id: 20, number: '1020', name: 'Banque' });
const PASSIF = account({ id: 30, number: '2030', name: 'Carte de crédit', accountType: 'Liability' });

const carte: BankAccountSummary = {
	id: 7,
	bankName: 'Visa',
	description: 'Personnel',
	iban: null,
	qrIban: null,
	accountKind: 'credit_card',
	cardLast4: '4821',
	isPrimary: false,
	journalAccountId: PASSIF.id,
	version: 3,
	archived: false,
	closedOn: null,
	currentBalance: '0.00',
	lastTransactionDate: null,
	statementClosingBalance: null,
	statementDate: null,
	ledgerBalanceAtStatement: null,
	statementGap: null,
};

const options = (el: HTMLElement) =>
	Array.from((el as HTMLSelectElement).options).map((o) => o.value);

async function fill(el: HTMLElement, value: string) {
	await fireEvent.input(el, { target: { value } });
}

describe('page des comptes bancaires — la carte de crédit (Story 34-11a2, AC-7)', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		fetchAccountsMock.mockResolvedValue([ACTIF, PASSIF]);
		listBankAccountsMock.mockResolvedValue([carte]);
		getInvoiceSettingsMock.mockResolvedValue({
			defaultReceivableAccountId: null,
			defaultPayableAccountId: null,
		});
		createBankAccountMock.mockResolvedValue({});
		updateBankAccountMock.mockResolvedValue({});
	});

	it('création d’une carte : IBAN, QR-IBAN et principal masqués, quatre chiffres et leur aide, passifs seuls', async () => {
		const r = render(Page);
		await fireEvent.click(await r.findByTestId('create-bank-account-button'));
		// Compte bancaire par défaut : l'IBAN est là, l'actif proposé.
		expect(r.queryByTestId('form-iban')).not.toBeNull();
		expect(options(await r.findByTestId('form-journal-account'))).toContain('20');

		await fireEvent.click(await r.findByTestId('form-kind-credit-card'));
		expect(r.queryByTestId('form-iban')).toBeNull();
		expect(r.queryByTestId('form-qr-iban')).toBeNull();
		expect(r.queryByTestId('form-is-primary')).toBeNull();
		expect((await r.findByTestId('form-card-last4-help')).textContent).toContain(
			'Kesh ne garde jamais le numéro complet',
		);
		const select = await r.findByTestId('form-journal-account');
		expect(options(select)).toContain('30');
		expect(options(select)).not.toContain('20');
	});

	it('création d’une carte : le payload porte le type et les quatre chiffres, jamais d’IBAN', async () => {
		const r = render(Page);
		await fireEvent.click(await r.findByTestId('create-bank-account-button'));
		await fireEvent.click(await r.findByTestId('form-kind-credit-card'));
		await fill(await r.findByTestId('form-bank-name'), 'Visa');
		await fill(await r.findByTestId('form-card-last4'), '4821');
		await fireEvent.change(await r.findByTestId('form-journal-account'), {
			target: { value: '30' },
		});
		await fireEvent.click(await r.findByTestId('form-submit'));
		await waitFor(() => expect(createBankAccountMock).toHaveBeenCalledTimes(1));
		const payload = createBankAccountMock.mock.calls[0][0] as Record<string, unknown>;
		expect(payload.accountKind).toBe('credit_card');
		expect(payload.cardLast4).toBe('4821');
		expect(payload.journalAccountId).toBe(30);
		expect('iban' in payload).toBe(false);
		expect('qrIban' in payload).toBe(false);
		expect('isPrimary' in payload).toBe(false);
	});

	it('création d’une carte : quatre chiffres invalides ou compte lié absent, refus à l’écran sans appel', async () => {
		const r = render(Page);
		await fireEvent.click(await r.findByTestId('create-bank-account-button'));
		await fireEvent.click(await r.findByTestId('form-kind-credit-card'));
		await fill(await r.findByTestId('form-bank-name'), 'Visa');
		await fill(await r.findByTestId('form-card-last4'), '12a');
		await fireEvent.click(await r.findByTestId('form-submit'));
		expect((await r.findByTestId('form-error')).textContent).toContain('quatre chiffres');
		await fill(await r.findByTestId('form-card-last4'), '');
		await fireEvent.click(await r.findByTestId('form-submit'));
		await waitFor(() =>
			expect(r.getByTestId('form-error').textContent).toContain('compte de passif'),
		);
		expect(createBankAccountMock).not.toHaveBeenCalled();
	});

	// Revue P1 (E5) — sans `maxlength`, un numéro complet collé n'est pas tronqué à ses quatre
	// premiers chiffres : il est refusé à l'écran, et rien ne part au serveur.
	it('un numéro complet collé dans les quatre chiffres est refusé, jamais tronqué ni envoyé', async () => {
		const r = render(Page);
		await fireEvent.click(await r.findByTestId('create-bank-account-button'));
		await fireEvent.click(await r.findByTestId('form-kind-credit-card'));
		const champ = (await r.findByTestId('form-card-last4')) as HTMLInputElement;
		expect(champ.getAttribute('maxlength')).toBeNull();
		await fill(await r.findByTestId('form-bank-name'), 'Visa');
		await fill(champ, '4111 1111 1111 1111');
		await fireEvent.change(await r.findByTestId('form-journal-account'), {
			target: { value: '30' },
		});
		await fireEvent.click(await r.findByTestId('form-submit'));
		expect((await r.findByTestId('form-error')).textContent).toContain('quatre chiffres');
		expect(createBankAccountMock).not.toHaveBeenCalled();
	});

	// Revue P1 (B9, M14) — un actif choisi pour un compte bancaire est retiré au passage à
	// « Carte de crédit » : l'écran ne l'envoie pas, il demande un passif.
	it('passer à « Carte de crédit » retire un actif déjà choisi', async () => {
		const r = render(Page);
		await fireEvent.click(await r.findByTestId('create-bank-account-button'));
		await fireEvent.change(await r.findByTestId('form-journal-account'), {
			target: { value: '20' },
		});
		await fireEvent.click(await r.findByTestId('form-kind-credit-card'));
		await fill(await r.findByTestId('form-bank-name'), 'Visa');
		await fireEvent.click(await r.findByTestId('form-submit'));
		expect((await r.findByTestId('form-error')).textContent).toContain('compte de passif');
		expect(createBankAccountMock).not.toHaveBeenCalled();
	});

	it('modification d’une carte (E5/B7) : type affiché, champs de carte, payload sans IBAN ni type', async () => {
		const r = render(Page);
		await fireEvent.click(await r.findByTestId('edit-button-7'));
		expect((await r.findByTestId('edit-kind')).textContent).toContain('Carte de crédit');
		expect(r.queryByTestId('edit-iban')).toBeNull();
		expect(r.queryByTestId('edit-is-primary')).toBeNull();
		const last4 = (await r.findByTestId('edit-card-last4')) as HTMLInputElement;
		expect(last4.value).toBe('4821');
		expect(options(await r.findByTestId('edit-journal-account'))).not.toContain('20');
		await fill(last4, '');
		await fireEvent.click(await r.findByTestId('edit-form-submit'));
		await waitFor(() => expect(updateBankAccountMock).toHaveBeenCalledTimes(1));
		const [id, payload] = updateBankAccountMock.mock.calls[0] as [number, Record<string, unknown>];
		expect(id).toBe(7);
		expect('iban' in payload).toBe(false);
		expect('accountKind' in payload).toBe(false);
		expect('cardLast4' in payload).toBe(true);
		expect(payload.cardLast4).toBeNull();
		expect(payload.journalAccountId).toBe(30);
	});

	it('la liste montre « •••• 4821 » pour la carte', async () => {
		const r = render(Page);
		expect((await r.findByTestId('iban-cell-7')).textContent).toContain('4821');
		expect((await r.findByTestId('iban-cell-7')).textContent).toContain('••••');
	});
});
