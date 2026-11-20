// Story 38-11d (AC-1) — règles d'écran des comptes de prestataire.

import { describe, it, expect } from 'vitest';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import type { AccountResponse } from '$lib/features/accounts/accounts.types';
import {
	feeAccountCandidates,
	isPaypal,
	isProvider,
	paypalAccountLabels,
	paypalPayload,
	paypalSummaryLine,
	withoutProviders,
} from './provider';

function account(overrides: Partial<AccountResponse>): AccountResponse {
	return {
		id: 0,
		companyId: 1,
		number: '',
		name: '',
		accountType: 'Expense',
		active: true,
		role: null,
		postable: true,
		...overrides,
	} as AccountResponse;
}

describe('provider — prédicats et filtre', () => {
	it('isPaypal, isProvider : `null` et absent valent compte ordinaire', () => {
		expect(isPaypal({ provider: 'paypal' })).toBe(true);
		expect(isPaypal({ provider: 'wise' })).toBe(false);
		expect(isProvider({ provider: 'wise' })).toBe(true);
		expect(isProvider({ provider: null })).toBe(false);
		expect(isProvider({})).toBe(false);
	});

	it('withoutProviders écarte tout compte de prestataire, garde l’ordre', () => {
		const list = [
			{ id: 1, provider: null },
			{ id: 2, provider: 'paypal' },
			{ id: 3 },
			{ id: 4, provider: 'stripe' },
		];
		expect(withoutProviders(list).map((a) => a.id)).toEqual([1, 3]);
	});

	it('paypalAccountLabels : libellés distincts des seuls comptes PayPal, triés', () => {
		expect(
			paypalAccountLabels([
				{ provider: 'paypal', paypal: { account: 'PayPal www', feeAccountId: null } },
				{ provider: 'paypal', paypal: { account: 'PayPal boutique', feeAccountId: 1 } },
				{ provider: 'paypal', paypal: { account: 'PayPal www', feeAccountId: null } },
				{ provider: null, paypal: null },
			]),
		).toEqual(['PayPal boutique', 'PayPal www']);
	});

	it('feeAccountCandidates : charges actives et imputables, CHF ou devise du compte du plan', () => {
		const accounts = [
			account({ id: 1, number: '6941', currency: 'EUR' }),
			account({ id: 2, number: '6940' }),
			account({ id: 3, number: '6942', currency: 'USD' }),
			account({ id: 4, number: '6943', active: false }),
			account({ id: 5, number: '6944', postable: false }),
			account({ id: 6, number: '3200', accountType: 'Revenue' }),
		];
		expect(feeAccountCandidates(accounts, 'EUR').map((a) => a.id)).toEqual([2, 1]);
		expect(feeAccountCandidates(accounts, null).map((a) => a.id)).toEqual([2]);
	});

	it('paypalSummaryLine : « PayPal · <compte> · <devise> », CHF sans devise, null hors PayPal', () => {
		const accounts = [account({ id: 9, currency: 'USD' }), account({ id: 8 })];
		expect(
			paypalSummaryLine(
				{ provider: 'paypal', paypal: { account: 'PayPal www', feeAccountId: null }, journalAccountId: 9 },
				accounts,
			),
		).toBe('PayPal · PayPal www · USD');
		expect(
			paypalSummaryLine(
				{ provider: 'paypal', paypal: { account: 'PayPal www', feeAccountId: null }, journalAccountId: 8 },
				accounts,
			),
		).toBe('PayPal · PayPal www · CHF');
		expect(paypalSummaryLine({ provider: null, paypal: null, journalAccountId: 8 }, accounts)).toBeNull();
	});

	it('paypalPayload : libellé rogné, null s’il est vide', () => {
		expect(paypalPayload('  PayPal www ', 7)).toEqual({ account: 'PayPal www', feeAccountId: 7 });
		expect(paypalPayload('   ', 7)).toBeNull();
	});
});

/**
 * Inventaire des sites qui listent les comptes bancaires là où l'on importe un
 * relevé, règle une facture, paie un fournisseur, crée un lot ou rapproche par
 * l'écran générique : chacun écarte les comptes de prestataire. Le test lit la
 * source ; les tests de composant (import, saisie) en vérifient le rendu.
 */
const SITES = [
	'lib/features/bank-import/BankImportUpload.svelte',
	'lib/features/bank-import/ManualStatementForm.svelte',
	'lib/features/invoices/SettleInvoiceDialog.svelte',
	'routes/(app)/reconciliation/+page.svelte',
	'routes/(app)/supplier-invoices/[id]/+page.svelte',
	'routes/(app)/payment-batches/+page.svelte',
];

describe('provider — inventaire des sites (Story 38-11d, AC-1)', () => {
	it.each(SITES)('%s appelle withoutProviders', (site) => {
		const source = readFileSync(join(process.cwd(), 'src', site), 'utf8');
		expect(source).toMatch(/withoutProviders\(/);
	});
});
