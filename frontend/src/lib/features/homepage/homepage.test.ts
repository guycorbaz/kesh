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
		description: null,
		iban: 'CH4431999123000889012',
		qrIban: null,
		isPrimary: true,
		journalAccountId: 10,
		version: 1,
		archived: false,
		closedOn: null,
		currentBalance: '0',
		lastTransactionDate: null,
		statementClosingBalance: null,
		statementDate: null,
		ledgerBalanceAtStatement: null,
		statementGap: null,
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

// Story 29-1a (#232, C-29-1-25, T-a16) — l'écart est calculé par le serveur
// (`bank_accounts::statement_gap`) ; ses cas d'arrondi sont partis en Rust
// (T-a14, `crates/kesh-db/src/repositories/bank_accounts.rs`), à l'identique.
// Ici, seule la LECTURE du champ se vérifie : aucun recalcul.
describe('statementGap', () => {
	it('lit l’écart servi par le serveur, sans le recalculer', () => {
		const g = statementGap(
			// Livres et relevé égaux, mais le serveur dit -200.00 : c'est lui qui fait foi.
			account({
				ledgerBalanceAtStatement: '1000.00',
				statementClosingBalance: '1000.00',
				statementGap: '-200.00',
			}),
		);
		expect(g?.eq(-200)).toBe(true);
	});

	it('sans écart servi, aucun écart — même si livres et relevé diffèrent', () => {
		expect(
			statementGap(
				account({
					ledgerBalanceAtStatement: '800.0000',
					statementClosingBalance: '1000.00',
					statementGap: null,
				}),
			),
		).toBeNull();
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

describe('formatChfBalance — zéro négatif', () => {
	it('un montant négatif sous le centime s’affiche 0.00, pas -0.00', () => {
		expect(formatChfBalance('-0.004')).not.toMatch(/-/);
		expect(formatChfBalance('-0.0000')).not.toMatch(/-/);
	});
});
