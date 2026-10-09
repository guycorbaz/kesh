/**
 * Le dialogue de règlement, au centime — Story 25-4-c3-b (#476).
 *
 * ⚠️ Chaque test nomme la MUTATION qu'il attrape.
 */
import { describe, it, expect, vi, afterEach } from 'vitest';
import { render, fireEvent, cleanup, waitFor } from '@testing-library/svelte';

vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_k: string, fallback: string) => fallback,
}));

import SettleInvoiceDialog from './SettleInvoiceDialog.svelte';
import SettleInvoiceDialogHost from './SettleInvoiceDialogHost.test.svelte';
import type { BankAccountSummary } from '$lib/features/bank-accounts/bank-accounts.api';
import type { AccountResponse } from '$lib/features/accounts/accounts.types';

const banque = {
	id: 1,
	bankName: 'Banque',
	iban: 'CH00',
	qrIban: null,
	isPrimary: true,
	journalAccountId: 10,
	version: 1,
	archived: false,
} as unknown as BankAccountSummary;

const SCALE = 'Le montant ne peut avoir plus de deux décimales';
const OVER = 'Le montant dépasse ce qui reste dû sur cette facture';

afterEach(() => cleanup());

function ouvrir(amountDue: string) {
	const onConfirm = vi.fn();
	const r = render(SettleInvoiceDialog, {
		open: true,
		onOpenChange: vi.fn(),
		invoiceDate: '2026-01-01',
		amountDue,
		accounts: [],
		bankAccounts: [banque],
		onConfirm,
	});
	const input = () => document.getElementById('settle-amount') as HTMLInputElement;
	return { ...r, onConfirm, input };
}

async function saisir(input: HTMLInputElement, v: string) {
	await fireEvent.input(input, { target: { value: v } });
}

describe('SettleInvoiceDialog — le reste dû au centime', () => {
	it('le 10.01 pré-rempli sur un reste brut de 10.0050 est accepté (mutation : comparaison au brut)', async () => {
		const { input, getByTestId, onConfirm, queryByText } = ouvrir('10.0050');
		await waitFor(() => expect(input().value).toBe('10.01'));
		expect(queryByText(OVER)).toBeNull();
		await fireEvent.click(getByTestId('settle-confirm'));
		expect(onConfirm).toHaveBeenCalledWith(expect.objectContaining({ amount: '10.01' }));
	});

	it('10.02 reste un trop-perçu', async () => {
		const { input, findByText } = ouvrir('10.0050');
		await saisir(input(), '10.02');
		expect(await findByText(OVER)).toBeTruthy();
	});

	it('10.008 est refusé pour ses décimales, pas accepté comme partiel (mutation : contrôle d’échelle retiré)', async () => {
		const { input, findByText, queryByText } = ouvrir('10.0050');
		await saisir(input(), '10.008');
		expect(await findByText(SCALE)).toBeTruthy();
		expect(queryByText(OVER)).toBeNull();
	});

	it('« 1e-3 » a trois décimales réelles (mutation : décimales comptées sur la saisie brute)', async () => {
		const { input, findByText } = ouvrir('10.0050');
		await saisir(input(), '1e-3');
		expect(await findByText(SCALE)).toBeTruthy();
	});

	it('« 10.000 » est un montant au centime (mutation : zéros de fin comptés)', async () => {
		const { input, getByTestId, queryByText, onConfirm } = ouvrir('10.0050');
		await saisir(input(), '10.000');
		expect(queryByText(SCALE)).toBeNull();
		await fireEvent.click(getByTestId('settle-confirm'));
		expect(onConfirm).toHaveBeenCalledWith(expect.objectContaining({ amount: '10.000' }));
	});

	it('10.00 sur un reste brut de 10.004 est accepté (arrondi au-dessous du brut)', async () => {
		const { input, queryByText } = ouvrir('10.004');
		await waitFor(() => expect(input().value).toBe('10.00'));
		expect(queryByText(OVER)).toBeNull();
		expect(queryByText(SCALE)).toBeNull();
	});
});

// Story 15-6b (#474, AC8, AC9 ; test 18) — le compte débiteurs n'est pas une contrepartie.
describe('SettleInvoiceDialog — la contrepartie n’est pas le compte débiteurs', () => {
	const DEBITEURS = {
		id: 1100,
		number: '1100',
		name: 'Débiteurs',
		accountType: 'Asset',
		active: true,
		postable: true,
		role: 'Receivable',
	} as unknown as AccountResponse;
	const CAISSE = {
		id: 1000,
		number: '1000',
		name: 'Caisse',
		accountType: 'Asset',
		active: true,
		postable: true,
		role: null,
	} as unknown as AccountResponse;
	function compte(id: number, journalAccountId: number | null, isPrimary = false) {
		return {
			...banque,
			id,
			bankName: `Banque ${id}`,
			isPrimary,
			journalAccountId,
		} as BankAccountSummary;
	}
	const EMPTY =
		'Aucun compte bancaire utilisable : le seul compte lié est le compte débiteurs de cette facture. Reliez un compte bancaire à son propre compte de banque, ou réglez par un compte interne.';

	function monter(accounts: AccountResponse[], bankAccounts: BankAccountSummary[], errorMsg = '') {
		return render(SettleInvoiceDialog, {
			open: true,
			onOpenChange: vi.fn(),
			invoiceDate: '2026-01-01',
			amountDue: '100.00',
			accounts,
			bankAccounts,
			errorMsg,
			onConfirm: vi.fn(),
		});
	}
	const options = (testId: string) =>
		Array.from(
			(document.querySelector(`[data-testid="${testId}"]`) as HTMLSelectElement).options,
		).map((o) => o.value);

	it('1100 (rôle Receivable) est absent du menu « Compte interne » (mutation : filtre retiré)', async () => {
		const { getByTestId } = monter([CAISSE, DEBITEURS], []);
		await fireEvent.change(getByTestId('settle-type'), { target: { value: 'internal_account' } });
		await waitFor(() => expect(options('settle-account')).toContain('1000'));
		expect(options('settle-account')).not.toContain('1100');
	});

	it('un compte bancaire lié au 1100 est absent, et la présélection tombe sur un autre', async () => {
		monter([CAISSE, DEBITEURS], [compte(1, 1100, true), compte(2, 1020)]);
		await waitFor(() => expect(options('settle-bank')).toEqual(['2']));
		expect((document.getElementById('settle-bank') as HTMLSelectElement).value).toBe('2');
	});

	it('`accounts` arrivé APRÈS l’ouverture ne réinitialise pas la saisie (mutation : présélection dans l’effet de réinitialisation)', async () => {
		// Hôte qui passe les props une à une (cf. son en-tête : `rerender` les changerait toutes).
		const { getByTestId } = render(SettleInvoiceDialogHost, {
			lateAccounts: [CAISSE, DEBITEURS],
			bankAccounts: [compte(1, 1100, true), compte(2, 1020)],
		});
		const input = document.getElementById('settle-amount') as HTMLInputElement;
		await waitFor(() => expect(input.value).toBe('100.00'));
		await waitFor(() =>
			expect((document.getElementById('settle-bank') as HTMLSelectElement).value).toBe('1'),
		);
		await fireEvent.input(input, { target: { value: '40.00' } });
		await fireEvent.click(getByTestId('host-load-accounts'));
		await waitFor(() => expect(options('settle-bank')).toEqual(['2']));
		expect((document.getElementById('settle-bank') as HTMLSelectElement).value).toBe('2');
		expect((document.getElementById('settle-amount') as HTMLInputElement).value).toBe('40.00');
	});

	// Revue de code P1 (finding L6) — les ids se calculent AVANT le filtre
	// `active && postable` : un 1100 devenu non imputable écarte toujours la
	// banque qui y est liée.
	it('1100 non imputable : la banque qui y est liée reste absente (mutation : ids calculés après le filtre `active && postable`)', async () => {
		monter(
			[CAISSE, { ...DEBITEURS, postable: false } as AccountResponse],
			[compte(1, 1100, true), compte(2, 1020)],
		);
		await waitFor(() => expect(options('settle-bank')).toEqual(['2']));
	});

	// Revue de code P1 (finding L7) — la réouverture : la réinitialisation remet
	// le compte bancaire à `null`, puis l'effet distinct présélectionne DANS la
	// liste filtrée ; le montant revient au reste dû.
	it('réouverture : saisie réinitialisée, présélection refaite dans la liste filtrée', async () => {
		const { getByTestId } = render(SettleInvoiceDialogHost, {
			lateAccounts: [CAISSE, DEBITEURS],
			bankAccounts: [compte(1, 1100, true), compte(2, 1020), compte(3, 1030)],
		});
		await fireEvent.click(getByTestId('host-load-accounts'));
		const bank = () => document.getElementById('settle-bank') as HTMLSelectElement;
		const amount = () => document.getElementById('settle-amount') as HTMLInputElement;
		await waitFor(() => expect(options('settle-bank')).toEqual(['2', '3']));
		await waitFor(() => expect(bank().value).toBe('2'));
		await fireEvent.change(bank(), { target: { value: '3' } });
		await fireEvent.input(amount(), { target: { value: '40.00' } });
		expect(bank().value).toBe('3');

		await fireEvent.click(getByTestId('host-toggle-open')); // fermer
		await fireEvent.click(getByTestId('host-toggle-open')); // rouvrir
		await waitFor(() => expect(amount().value).toBe('100.00'));
		await waitFor(() => expect(bank().value).toBe('2'));
	});

	// Revue de code P1 (finding L7) — un choix de l'utilisateur, encore éligible,
	// survit à l'arrivée tardive de `accounts`.
	it('un choix de l’utilisateur encore éligible survit à l’arrivée tardive de `accounts`', async () => {
		const { getByTestId } = render(SettleInvoiceDialogHost, {
			lateAccounts: [CAISSE, DEBITEURS],
			bankAccounts: [compte(1, 1100, true), compte(2, 1020), compte(3, 1030)],
		});
		const bank = () => document.getElementById('settle-bank') as HTMLSelectElement;
		await waitFor(() => expect(bank().value).toBe('1'));
		await fireEvent.change(bank(), { target: { value: '3' } });
		await fireEvent.click(getByTestId('host-load-accounts'));
		await waitFor(() => expect(options('settle-bank')).toEqual(['2', '3']));
		expect(bank().value).toBe('3');
	});

	it('le seul compte bancaire lié au 1100 → le message de liste vide', async () => {
		const { findByText } = monter([CAISSE, DEBITEURS], [compte(1, 1100, true)]);
		expect(await findByText(EMPTY)).toBeTruthy();
	});

	it('aucun compte bancaire → PAS de message de liste vide (rien n’a été écarté)', async () => {
		const { queryByTestId } = monter([CAISSE, DEBITEURS], []);
		await waitFor(() => expect(document.getElementById('settle-bank')).not.toBeNull());
		expect(queryByTestId('settle-no-eligible-bank')).toBeNull();
	});

	it('la prop `errorMsg` s’affiche (AC9 — rendu seulement)', async () => {
		const refus = 'Le compte 1100 est le compte débiteurs de cette facture';
		const { findByText, getByTestId } = monter([CAISSE], [compte(2, 1020, true)], refus);
		await waitFor(() =>
			expect((getByTestId('settle-bank') as HTMLSelectElement).value).toBe('2'),
		);
		expect(await findByText(refus)).toBeTruthy();
	});
});
