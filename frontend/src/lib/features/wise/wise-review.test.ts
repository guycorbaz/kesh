// Story 39-11d2 (AC-2, AC-4) — règles de la page Wise : la clé et la transaction d'une
// proposition (la débitée d'une conversion, la principale d'un groupe), le corps de
// l'acceptation (un groupe avec ses identifiants, sa règle et sa contrepartie, sans `lines`
// ni `merchantAmount`), l'ordre d'envoi, les sous-lignes d'un groupe.

import { describe, it, expect } from 'vitest';
import type { ProviderReviewTransaction } from '$lib/features/provider/provider-review.types';
import {
	wiseAcceptBody,
	wiseFindingLines,
	wiseProposalKey,
	wiseProposalLines,
	wiseProposalTransactionId,
	wiseSelectedBodies,
} from './wise-review';
import type { WiseProposal } from './wise.types';

const tx = (id: number, date = '2020-03-10', more: Partial<ProviderReviewTransaction> = {}): ProviderReviewTransaction => ({
	id,
	bankAccountId: 1,
	date,
	amount: '-10.00',
	currency: 'CHF',
	providerType: 'CARD',
	name: null,
	transactionId: `CARD-${id}`,
	counterpartyEmail: null,
	...more,
});

const groupe = (date = '2020-03-10'): WiseProposal => ({
	type: 'card_group',
	bankTransactionIds: [7, 3, 4],
	ruleId: 2,
	counterpartyAccountId: 40,
	lines: [tx(7, date), tx(3, date, { currency: 'USD' }), tx(4, date, { transactionId: 'FEE-CARD-7' })],
	merchantAmount: '97.95',
	transaction: tx(7, date),
});

const frais = (id: number, date: string): WiseProposal => ({
	type: 'fee',
	bankTransactionId: id,
	feeAccountId: 60,
	transaction: tx(id, date),
});

const conversion: WiseProposal = {
	type: 'conversion',
	debitTransactionId: 11,
	creditTransactionId: 12,
	transaction: tx(11, '2020-02-05'),
	otherTransaction: tx(12, '2020-02-05'),
};

describe('wise-review — clé, transaction, corps (Story 39-11d2)', () => {
	it('la transaction : la débitée d’une conversion, la principale d’un groupe ; la clé en dépend', () => {
		expect(wiseProposalTransactionId(conversion)).toBe(11);
		expect(wiseProposalKey(conversion)).toBe('conversion-11');
		expect(wiseProposalTransactionId(groupe())).toBe(7);
		expect(wiseProposalKey(groupe())).toBe('card_group-7');
		expect(wiseProposalKey(frais(5, '2020-01-01'))).toBe('fee-5');
	});

	it('le corps d’un groupe : identifiants, règle, contrepartie ; sans lines ni merchantAmount', () => {
		const body = wiseAcceptBody(groupe());
		expect(body).toEqual({ type: 'card_group', bankTransactionIds: [7, 3, 4], ruleId: 2, counterpartyAccountId: 40 });
		expect(Object.keys(body)).not.toContain('lines');
		expect(Object.keys(body)).not.toContain('merchantAmount');
	});

	it('les corps des genres communs : leurs seuls champs', () => {
		expect(wiseAcceptBody(conversion)).toEqual({ type: 'conversion', debitTransactionId: 11, creditTransactionId: 12 });
		expect(wiseAcceptBody(frais(5, '2020-01-01'))).toEqual({ type: 'fee', bankTransactionId: 5, feeAccountId: 60 });
		expect(
			wiseAcceptBody({
				type: 'link_entry',
				bankTransactionId: 8,
				journalEntryId: 90,
				transaction: tx(8),
				entry: { journalEntryId: 90, entryNumber: 12, entryDate: '2020-03-10', description: 'x', ids: [] },
			}),
		).toEqual({ type: 'link_entry', bankTransactionId: 8, journalEntryId: 90 });
		expect(
			wiseAcceptBody({ type: 'rule', bankTransactionId: 9, ruleId: 3, counterpartyAccountId: 41, transaction: tx(9) }),
		).toEqual({ type: 'rule', bankTransactionId: 9, ruleId: 3, counterpartyAccountId: 41 });
	});

	it('la sélection seule, dans l’ordre des dates, puis de la transaction', () => {
		const props = [groupe('2020-03-10'), frais(5, '2020-01-20'), conversion, frais(2, '2020-01-20')];
		const bodies = wiseSelectedBodies(props, new Set(['card_group-7', 'fee-5', 'fee-2']));
		expect(bodies.map((b) => b.type)).toEqual(['fee', 'fee', 'card_group']);
		expect(bodies.slice(0, 2)).toEqual([
			{ type: 'fee', bankTransactionId: 2, feeAccountId: 60 },
			{ type: 'fee', bankTransactionId: 5, feeAccountId: 60 },
		]);
	});
});

describe('wise-review — sous-lignes d’un paiement groupé', () => {
	it('proposition : les lignes après la principale ; aucune pour un autre genre', () => {
		expect(wiseProposalLines(groupe()).map((t) => t.id)).toEqual([3, 4]);
		expect(wiseProposalLines(conversion)).toEqual([]);
	});

	it('« à examiner » : details.lines hors la transaction du motif ; aucune hors card_group_*', () => {
		const lines = [tx(7), tx(3), tx(4)];
		expect(
			wiseFindingLines({ code: 'card_group_statements_missing', transaction: tx(7), details: { lines } }).map((t) => t.id),
		).toEqual([3, 4]);
		expect(wiseFindingLines({ code: 'no_rule', transaction: tx(7), details: { lines } })).toEqual([]);
		expect(wiseFindingLines({ code: 'card_group_no_rule', transaction: tx(7), details: {} })).toEqual([]);
		expect(
			wiseFindingLines({ code: 'card_group_unsupported', transaction: tx(7), details: { lines: [null, 'x', tx(5)] } }).map(
				(t) => t.id,
			),
		).toEqual([5]);
	});
});
