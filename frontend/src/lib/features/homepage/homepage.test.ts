// Story 25-6-a (#388, #389) — les calculs de l'accueil.
import { describe, it, expect } from 'vitest';
import type { BankAccountSummary } from '$lib/features/bank-accounts/bank-accounts.api';
import type { JournalEntryResponse } from '$lib/features/journal-entries/journal-entries.types';
import { entryAmount, ledgerTotal, statementGap } from './homepage';
import { formatChfBalance } from '$lib/features/bank-accounts/format';

function account(over: Partial<BankAccountSummary> = {}): BankAccountSummary {
	return {
		id: 1,
		bankName: 'UBS',
		iban: 'CH4431999123000889012',
		qrIban: null,
		isPrimary: true,
		journalAccountId: 10,
		version: 1,
		archived: false,
		currentBalance: '0',
		lastTransactionDate: null,
		statementClosingBalance: null,
		statementDate: null,
		ledgerBalanceAtStatement: null,
		...over,
	};
}

describe('ledgerTotal', () => {
	it('somme en big.js — 0.1 + 0.2 vaut 0.3, pas 0.30000000000000004', () => {
		const t = ledgerTotal([
			account({ id: 1, journalAccountId: 10, currentBalance: '0.1' }),
			account({ id: 2, journalAccountId: 11, currentBalance: '0.2' }),
		]);
		expect(t.total.toString()).toBe('0.3');
	});

	it('ne compte qu’une fois un compte de grand livre partagé, et le signale', () => {
		const t = ledgerTotal([
			account({ id: 1, journalAccountId: 10, currentBalance: '500.0000' }),
			account({ id: 2, journalAccountId: 10, currentBalance: '500.0000' }),
		]);
		expect(t.total.toString()).toBe('500');
		expect(t.shared).toBe(true);
	});

	it('signale un total partiel quand un compte n’est pas lié', () => {
		const t = ledgerTotal([
			account({ id: 1, currentBalance: '100' }),
			account({ id: 2, journalAccountId: null, currentBalance: null }),
		]);
		expect(t.partial).toBe(true);
		expect(t.any).toBe(true);
	});
});

describe('statementGap', () => {
	it('rend l’écart solde comptable − relevé', () => {
		const g = statementGap(
			account({ ledgerBalanceAtStatement: '800.0000', statementClosingBalance: '1000.00' }),
		);
		expect(g?.toString()).toBe('-200');
	});

	it('un écart sous le centime n’en est pas un', () => {
		expect(
			statementGap(account({ ledgerBalanceAtStatement: '100.0049', statementClosingBalance: '100.00' })),
		).toBeNull();
	});

	it('arrondit à mi-chemin loin de zéro, pas à l’arrondi bancaire', () => {
		// Arrondi bancaire : 100.005 → 100.00, d'où un faux écart de −0.01.
		expect(
			statementGap(account({ ledgerBalanceAtStatement: '100.005', statementClosingBalance: '100.01' })),
		).toBeNull();
		expect(
			statementGap(account({ ledgerBalanceAtStatement: '-100.005', statementClosingBalance: '-100.01' })),
		).toBeNull();
	});

	it('sans relevé ou sans solde à sa date, pas d’écart', () => {
		expect(statementGap(account({ statementClosingBalance: '10.00' }))).toBeNull();
	});
});

describe('entryAmount et formatChfBalance', () => {
	it('le montant d’une écriture est la somme de ses débits', () => {
		const e = {
			lines: [
				{ debit: '120.50', credit: '0' },
				{ debit: '0', credit: '120.50' },
			],
		} as unknown as JournalEntryResponse;
		expect(entryAmount(e).toString()).toBe('120.5');
	});

	it('formatChfBalance accepte une chaîne à quatre décimales', () => {
		expect(formatChfBalance('1234.5650')).toMatch(/1.234\.57/);
	});
});
