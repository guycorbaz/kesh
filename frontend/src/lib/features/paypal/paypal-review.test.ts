// Story 38-11d (AC-3, AC-4) — règles de la page PayPal : exercice par défaut, clés,
// corps de l'acceptation dans l'ordre des dates, retrait incomplet écarté, paquets de 500.

import { describe, it, expect, vi } from 'vitest';

vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string, args?: Record<string, string | number>) =>
		args ? fallback.replace(/\{\s*\$(\w+)\s*\}/g, (_, k) => String(args[k] ?? '')) : fallback,
}));

import type { FiscalYearResponse } from '$lib/features/fiscal-years/fiscal-years.types';
import type { AccountResponse } from '$lib/features/accounts/accounts.types';
import type { PaypalProposal } from './paypal.types';
import {
	acceptBody,
	defaultFiscalYearId,
	proposalActionLabel,
	proposalKey,
	selectedBodies,
	withdrawalAccountCandidates,
} from './paypal-review';
import { chunkProposals, PAYPAL_ACCEPT_CHUNK } from './paypal.api';

const base = {
	amount: '10.00',
	currency: 'EUR',
	providerType: null,
	name: null,
	counterpartyEmail: null,
	transactionId: null,
};

function year(id: number, startDate: string, endDate: string): FiscalYearResponse {
	return { id, companyId: 1, name: `E${id}`, startDate, endDate, status: 'Open', multiCurrency: true } as FiscalYearResponse;
}

const fee: PaypalProposal = { ...base, type: 'fee', bankTransactionId: 3, date: '2020-03-02', feeAccountId: 60 };
const link: PaypalProposal = {
	...base,
	type: 'link_entry',
	bankTransactionId: 1,
	date: '2020-03-05',
	journalEntryId: 40,
	entryNumber: 12,
};
const conv: PaypalProposal = {
	...base,
	type: 'conversion',
	bankTransactionId: 7,
	date: '2020-03-01',
	debitTransactionId: 7,
	creditTransactionId: 8,
	otherAmount: '11.00',
	otherCurrency: 'USD',
};
const retrait: PaypalProposal = {
	...base,
	currency: 'CHF',
	type: 'withdrawal',
	bankTransactionId: 9,
	date: '2020-03-03',
	counterpartyAccountId: null,
};

describe('paypal-review', () => {
	it('exercice par défaut : celui du jour, sinon le plus récent, sinon null', () => {
		const years = [year(1, '2019-01-01', '2019-12-31'), year(2, '2020-01-01', '2020-12-31')];
		expect(defaultFiscalYearId(years, '2019-06-01')).toBe(1);
		expect(defaultFiscalYearId(years, '2026-06-01')).toBe(2);
		expect(defaultFiscalYearId([], '2026-06-01')).toBeNull();
	});

	it('clé : la transaction débitée d’une conversion', () => {
		expect(proposalKey(conv)).toBe('conversion-7');
		expect(proposalKey(fee)).toBe('fee-3');
	});

	it('corps : chaque genre ; un retrait sans compte est incomplet', () => {
		expect(acceptBody(conv, {})).toEqual({ type: 'conversion', debitTransactionId: 7, creditTransactionId: 8 });
		expect(acceptBody(link, {})).toEqual({ type: 'link_entry', bankTransactionId: 1, journalEntryId: 40 });
		expect(acceptBody(retrait, {})).toBeNull();
		expect(acceptBody(retrait, { 'withdrawal-9': 20 })).toEqual({
			type: 'withdrawal',
			bankTransactionId: 9,
			counterpartyAccountId: 20,
		});
	});

	it('sélection : seulement les cochées, dans l’ordre des dates, incomplètes écartées', () => {
		const bodies = selectedBodies(
			[link, fee, conv, retrait],
			new Set(['link_entry-1', 'fee-3', 'conversion-7', 'withdrawal-9']),
			{},
		);
		expect(bodies.map((b) => b.type)).toEqual(['conversion', 'fee', 'link_entry']);
	});

	it('ce qui sera fait : une écriture si un côté est en CHF, deux sinon', () => {
		expect(proposalActionLabel(conv, [])).toContain('Deux écritures');
		expect(proposalActionLabel({ ...conv, otherCurrency: 'CHF' }, [])).toContain('Une écriture');
		expect(proposalActionLabel(link, [])).toContain('n° 12');
	});

	it('comptes de retrait : actifs imputables en CHF, triés', () => {
		const a = (o: Partial<AccountResponse>) => ({ active: true, postable: true, accountType: 'Asset', ...o }) as AccountResponse;
		const list = [a({ id: 2, number: '1020' }), a({ id: 1, number: '1010' }), a({ id: 3, number: '1031', currency: 'EUR' }), a({ id: 4, number: '2000', accountType: 'Liability' })];
		expect(withdrawalAccountCandidates(list).map((x) => x.id)).toEqual([1, 2]);
	});

	it('paquets de 500 au plus, dans l’ordre', () => {
		const items = Array.from({ length: 1201 }, (_, i) => i);
		const chunks = chunkProposals(items);
		expect(PAYPAL_ACCEPT_CHUNK).toBe(500);
		expect(chunks.map((c) => c.length)).toEqual([500, 500, 201]);
		expect(chunks[1][0]).toBe(500);
	});
});
