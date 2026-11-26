// Story 40-7d (AC-3) — règles de la page Stripe : clé, corps de l'acceptation (le versement
// avec le compte de versement de la revue), ordre des dates.

import { describe, it, expect } from 'vitest';
import { stripeAcceptBody, stripeProposalKey, stripeSelectedBodies } from './stripe-review';
import type { StripeProposal } from './stripe.types';

const tx = (id: number, date: string) => ({
	id,
	bankAccountId: 2,
	date,
	amount: '1.00',
	currency: 'CHF',
	providerType: 'charge',
	name: null,
	transactionId: `txn_${id}`,
	counterpartyEmail: null,
});

const link: StripeProposal = { type: 'link_entry', bankTransactionId: 3, journalEntryId: 40, transaction: tx(3, '2020-02-01') };
const fee: StripeProposal = { type: 'fee', bankTransactionId: 1, feeAccountId: 60, transaction: tx(1, '2020-02-01') };
const payout: StripeProposal = {
	type: 'payout',
	bankTransactionId: 2,
	payoutBankAccountId: 5,
	payoutAccountLabel: 'Banque',
	transaction: tx(2, '2020-01-15'),
};

describe('stripe-review', () => {
	it('clé : genre et transaction', () => {
		expect(stripeProposalKey(payout)).toBe('payout-2');
	});

	it('corps : les seuls champs ; le versement reprend son compte de versement, sans libellé', () => {
		expect(stripeAcceptBody(link)).toEqual({ type: 'link_entry', bankTransactionId: 3, journalEntryId: 40 });
		expect(stripeAcceptBody(fee)).toEqual({ type: 'fee', bankTransactionId: 1, feeAccountId: 60 });
		expect(stripeAcceptBody(payout)).toEqual({ type: 'payout', bankTransactionId: 2, payoutBankAccountId: 5 });
	});

	it('sélection : cochées seulement, dans l’ordre des dates puis des transactions', () => {
		const bodies = stripeSelectedBodies([link, fee, payout], new Set(['link_entry-3', 'fee-1', 'payout-2']));
		expect(bodies.map((b) => b.bankTransactionId)).toEqual([2, 1, 3]);
		expect(stripeSelectedBodies([link, fee, payout], new Set(['fee-1'])).map((b) => b.type)).toEqual(['fee']);
	});
});
