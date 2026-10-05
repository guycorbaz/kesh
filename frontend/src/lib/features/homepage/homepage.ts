// Story 25-6-a (#388, #389) — calculs de la page d'accueil, purs et testables.
//
// ⚠️ Tout montant reste une chaîne décimale jusqu'ici et passe par `big.js` :
// jamais de `Number` sur un montant (l'ancien total des liquidités sommait des
// `Number`).

import Big from 'big.js';
import type { BankAccountSummary } from '$lib/features/bank-accounts/bank-accounts.api';
import type { JournalEntryResponse } from '$lib/features/journal-entries/journal-entries.types';

/** Montant d'une écriture : la somme de ses débits (= celle de ses crédits). */
export function entryAmount(entry: JournalEntryResponse): Big {
	return entry.lines.reduce((acc, l) => acc.plus(new Big(l.debit)), new Big(0));
}

/** Total des soldes comptables de la tuile « Comptes bancaires ». */
export interface LedgerTotal {
	total: Big;
	/** Au moins un compte bancaire sans compte lié (donc sans solde). */
	partial: boolean;
	/** Au moins deux comptes bancaires liés au même compte de grand livre. */
	shared: boolean;
	/** Au moins un solde est connu. */
	any: boolean;
}

/**
 * Somme les soldes comptables en ne comptant **chaque compte de grand livre
 * qu'une fois** : deux comptes bancaires liés au même compte ont, par
 * construction, le même solde (la somme porte sur le même compte) — les
 * additionner le compterait deux fois.
 */
export function ledgerTotal(accounts: BankAccountSummary[]): LedgerTotal {
	const seen = new Set<number>();
	let total = new Big(0);
	let shared = false;
	let any = false;
	for (const a of accounts) {
		if (a.currentBalance === null || a.journalAccountId === null) continue;
		any = true;
		if (seen.has(a.journalAccountId)) {
			shared = true;
			continue;
		}
		seen.add(a.journalAccountId);
		total = total.plus(new Big(a.currentBalance));
	}
	const partial = any && accounts.some((a) => a.currentBalance === null);
	return { total, partial, shared, any };
}

/** Arrondi du dépôt : au centime, demi loin de zéro. */
function toCentime(v: string): Big {
	return new Big(v).round(2, Big.roundHalfUp);
}

/**
 * Écart entre le solde comptable à la date du relevé et le relevé, **au
 * centime** (solde comptable − relevé). `null` quand il ne se calcule pas
 * (pas de relevé, pas de compte lié, compte de grand livre partagé) **ou quand
 * il est nul** — seul un écart réel s'affiche.
 */
export function statementGap(account: BankAccountSummary): Big | null {
	if (account.ledgerBalanceAtStatement === null || account.statementClosingBalance === null) {
		return null;
	}
	const gap = toCentime(account.ledgerBalanceAtStatement).minus(
		toCentime(account.statementClosingBalance),
	);
	return gap.eq(0) ? null : gap;
}
