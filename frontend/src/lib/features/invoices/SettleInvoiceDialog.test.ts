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
import type { BankAccountSummary } from '$lib/features/bank-accounts/bank-accounts.api';

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
