// Story 25-7 (#445) — calculs de l'écran des soldes de départ.

import { describe, it, expect } from 'vitest';
import {
	complementCounterpart,
	computeOpeningTotals,
	retainedEarningsWarning,
} from './opening-balances-totals';
import type { AccountResponse } from '$lib/features/accounts/accounts.types';

function acc(overrides: Partial<AccountResponse>): AccountResponse {
	return {
		id: 1,
		companyId: 1,
		number: '1000',
		name: 'Banque',
		accountType: 'Asset',
		parentId: null,
		active: true,
		role: null,
		postable: true,
		version: 1,
		createdAt: '2026-01-01T00:00:00',
		updatedAt: '2026-01-01T00:00:00',
		...overrides,
	};
}

const ASSET = acc({ id: 1 });
const LIAB = acc({ id: 2, number: '2000', accountType: 'Liability' });
const RE = acc({ id: 3, number: '2970', accountType: 'Liability', role: 'RetainedEarnings' });

describe('computeOpeningTotals', () => {
	it('un compte contre sa nature compte avec son signe', () => {
		const t = computeOpeningTotals([
			{ account: ASSET, debit: '', credit: '50' },
			{ account: LIAB, debit: '20', credit: '' },
		]);
		expect(t.assets.toString()).toBe('-50');
		expect(t.liabilities.toString()).toBe('-20');
		expect(t.amountToCarry.toString()).toBe('-30');
	});

	it('le report est exclu des passifs et compté à part', () => {
		const t = computeOpeningTotals([
			{ account: ASSET, debit: '100', credit: '' },
			{ account: RE, debit: '', credit: '100' },
		]);
		expect(t.liabilities.toString()).toBe('0');
		expect(t.retainedEntered.toString()).toBe('100');
		expect(t.remainingGap.toString()).toBe('0');
	});

	it('montants invalides ignorés, jamais NaN', () => {
		const t = computeOpeningTotals([{ account: ASSET, debit: 'abc', credit: '' }]);
		expect(t.assets.toString()).toBe('0');
	});
});

describe('retainedEarningsWarning', () => {
	const rows = (re: string) => [
		{ account: ASSET, debit: '100', credit: '' },
		{ account: RE, debit: '', credit: re },
	];
	it('rien si la saisie n’est pas équilibrée', () => {
		expect(retainedEarningsWarning([ASSET, RE], rows(''), false)).toBeNull();
	});
	it('NO_AMOUNT / rien / NO_ROLE / NOT_POSTABLE', () => {
		expect(retainedEarningsWarning([ASSET, RE], rows(''), true)?.kind).toBe('NO_AMOUNT');
		expect(retainedEarningsWarning([ASSET, RE], rows('100'), true)).toBeNull();
		expect(retainedEarningsWarning([ASSET, LIAB], rows(''), true)?.kind).toBe('NO_ROLE');
		expect(
			retainedEarningsWarning([ASSET, { ...RE, postable: false }], rows(''), true)?.kind
		).toBe('NOT_POSTABLE');
		// Un compte de report archivé ne compte pas.
		expect(retainedEarningsWarning([ASSET, { ...RE, active: false }], rows(''), true)?.kind).toBe(
			'NO_ROLE'
		);
	});
});

describe('complementCounterpart', () => {
	it('sens et montant', () => {
		expect(complementCounterpart([{ debit: '10', credit: '' }])).toMatchObject({ side: 'credit' });
		const d = complementCounterpart([{ debit: '', credit: '10.5' }]);
		expect([d.side, d.amount.toString()]).toEqual(['debit', '10.5']);
		expect(complementCounterpart([{ debit: '5', credit: '' }, { debit: '', credit: '5' }]).side).toBe(
			'none'
		);
	});
});
