// Story 40-7d (AC-1) — règles d'écran des comptes Stripe : libellés, ligne de la liste,
// charge du formulaire, comptes de versement admis (miroir de `STRIPE_PAYOUT_ACCOUNT_INVALID`).

import { describe, it, expect } from 'vitest';
import type { AccountResponse } from '$lib/features/accounts/accounts.types';
import type { BankAccountSummary } from './bank-accounts.api';
import {
	isStripe,
	payoutAccountCandidates,
	providerPayload,
	providerSettingsOf,
	providerSummaryLine,
	paypalSummaryLine,
	stripeAccountLabels,
} from './provider';

const plan = (id: number, currency: string | null): AccountResponse =>
	({ id, companyId: 1, number: String(1000 + id), name: `Compte ${id}`, accountType: 'Asset', active: true, role: null, postable: true, currency }) as AccountResponse;

function banque(id: number, overrides: Partial<BankAccountSummary> = {}): BankAccountSummary {
	return {
		id,
		bankName: `Banque ${id}`,
		description: null,
		iban: 'CH9300762011623852957',
		qrIban: null,
		accountKind: 'bank',
		cardLast4: null,
		provider: null,
		paypal: null,
		stripe: null,
		isPrimary: false,
		journalAccountId: id,
		version: 1,
		archived: false,
		closedOn: null,
		currentBalance: null,
		lastTransactionDate: null,
		statementClosingBalance: null,
		statementDate: null,
		ledgerBalanceAtStatement: null,
		statementGap: null,
		...overrides,
	};
}

const stripeSettings = (account: string) => ({ account, feeAccountId: null, payoutBankAccountId: null });

describe('provider — comptes Stripe', () => {
	it('libellés distincts, sans casse ni accents, PayPal écarté', () => {
		const comptes = [
			banque(1, { provider: 'stripe', stripe: stripeSettings('Stripe boutique') }),
			banque(2, { provider: 'stripe', stripe: stripeSettings('stripe BOUTIQUE') }),
			banque(3, { provider: 'stripe', stripe: stripeSettings('Stripe atelier') }),
			banque(4, { provider: 'paypal', paypal: { account: 'PayPal www', feeAccountId: null } }),
		];
		expect(stripeAccountLabels(comptes)).toEqual(['Stripe atelier', 'Stripe boutique']);
		expect(isStripe(comptes[0])).toBe(true);
		expect(isStripe(comptes[3])).toBe(false);
	});

	it('paramètres du prestataire : seulement le prestataire du compte', () => {
		const s = banque(1, { provider: 'stripe', stripe: stripeSettings('Stripe boutique') });
		expect(providerSettingsOf(s, 'stripe')?.account).toBe('Stripe boutique');
		expect(providerSettingsOf(s, 'paypal')).toBeNull();
	});

	it('ligne de la liste : « Stripe · <compte> · <devise> » ; PayPal inchangé', () => {
		const accounts = [plan(1, 'EUR')];
		const s = banque(1, { provider: 'stripe', stripe: stripeSettings('Stripe boutique') });
		expect(providerSummaryLine(s, accounts)).toBe('Stripe · Stripe boutique · EUR');
		expect(paypalSummaryLine(s, accounts)).toBeNull();
		expect(providerSummaryLine(banque(2), accounts)).toBeNull();
	});

	it('charge du formulaire : sous sa clé, le compte de versement pour Stripe seulement', () => {
		expect(providerPayload('stripe', '  Stripe boutique ', 60, 5)).toEqual({
			stripe: { account: 'Stripe boutique', feeAccountId: 60, payoutBankAccountId: 5 },
		});
		expect(providerPayload('paypal', 'PayPal www', null, 5)).toEqual({
			paypal: { account: 'PayPal www', feeAccountId: null },
		});
		expect(providerPayload('stripe', '   ', 60, 5)).toBeNull();
	});
});

describe('provider — comptes de versement d’un compte Stripe', () => {
	const accounts = [plan(1, null), plan(2, 'EUR'), plan(3, null), plan(4, null), plan(5, null), plan(6, null), plan(7, null), plan(8, null)];
	const comptes = [
		banque(1),
		banque(2, { bankName: 'Wise EUR', provider: 'wise' }),
		banque(3, { bankName: 'Wise CHF', provider: 'wise' }),
		banque(4, { provider: 'stripe', stripe: stripeSettings('Stripe boutique') }),
		banque(5, { provider: 'paypal', paypal: { account: 'PayPal www', feeAccountId: null } }),
		banque(6, { archived: true }),
		banque(7, { closedOn: '2020-01-31' }),
		banque(8, { accountKind: 'credit_card' }),
		banque(9, { journalAccountId: null }),
	];

	it('CHF : banques et Wise de la devise, ni Stripe, ni PayPal, ni archivé, ni clos, ni carte, ni compte non lié', () => {
		expect(payoutAccountCandidates(comptes, accounts, null, null).map((b) => b.id)).toEqual([1, 3]);
	});

	it('EUR : le seul compte de la devise ; le compte lui-même jamais', () => {
		expect(payoutAccountCandidates(comptes, accounts, 'EUR', null).map((b) => b.id)).toEqual([2]);
		expect(payoutAccountCandidates(comptes, accounts, null, 1).map((b) => b.id)).toEqual([3]);
	});
});
