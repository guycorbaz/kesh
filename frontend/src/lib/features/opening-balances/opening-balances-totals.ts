/**
 * Calculs de l'écran « Soldes de départ » (Story 25-7, #445) : totaux actif /
 * passif, montant à porter au compte de report, avertissement quand la saisie
 * s'équilibre sans lui, et contrepartie du complément.
 *
 * Toute l'arithmétique passe par `big.js` — jamais `Number` (CO art. 957-964).
 *
 * ⚠️ **Le montant à porter aide la saisie, il ne contrôle rien une fois la
 * saisie équilibrée** : équilibrée sans rien sur le compte de report, il vaut 0
 * par construction (le montant mal placé est compté dans les passifs). Le vrai
 * contrôle est de comparer les totaux Actifs et Passifs à ceux de l'ancien
 * bilan — l'écran le dit.
 */

import Big from 'big.js';
import { isValidAmount, parseAmount } from '$lib/features/journal-entries/balance';
import type { AccountResponse } from '$lib/features/accounts/accounts.types';

/** Une ligne de la grille d'ouverture, réduite à ce qu'il faut pour les totaux. */
export interface TotalsRow {
	account: Pick<AccountResponse, 'accountType' | 'role'>;
	debit: string;
	credit: string;
}

export interface OpeningTotals {
	/** Σ (débit − crédit) des comptes `Asset`. */
	assets: Big;
	/** Σ (crédit − débit) des comptes `Liability`, compte de report exclu. */
	liabilities: Big;
	/** Actifs − Passifs : positif → au crédit du report, négatif → au débit. */
	amountToCarry: Big;
	/** (crédit − débit) sur le compte de report. */
	retainedEntered: Big;
	/** Montant à porter − report saisi ; 0 quand la saisie est équilibrée. */
	remainingGap: Big;
}

function amount(raw: string): Big {
	return isValidAmount(raw) ? parseAmount(raw) : new Big(0);
}

/** Totaux de l'AC 2. Un compte saisi contre sa nature compte avec son signe. */
export function computeOpeningTotals(rows: TotalsRow[]): OpeningTotals {
	let assets = new Big(0);
	let liabilities = new Big(0);
	let retainedEntered = new Big(0);
	for (const row of rows) {
		const net = amount(row.debit).minus(amount(row.credit));
		if (row.account.role === 'RetainedEarnings') {
			retainedEntered = retainedEntered.minus(net);
		} else if (row.account.accountType === 'Asset') {
			assets = assets.plus(net);
		} else if (row.account.accountType === 'Liability') {
			liabilities = liabilities.minus(net);
		}
	}
	const amountToCarry = assets.minus(liabilities);
	return {
		assets,
		liabilities,
		amountToCarry,
		retainedEntered,
		remainingGap: amountToCarry.minus(retainedEntered)
	};
}

/** Variante de l'avertissement de l'AC 1, ou `null` s'il n'y a rien à dire. */
export type RetainedEarningsWarning =
	| { kind: 'NO_ROLE' }
	| { kind: 'NOT_POSTABLE'; account: AccountResponse }
	| { kind: 'NO_AMOUNT'; account: AccountResponse };

/**
 * Avertissement de l'AC 1 : seulement sur une saisie **équilibrée**. Il
 * signale, il ne bloque pas (doctrine #301).
 *
 * - aucun compte **actif** ne porte le rôle → `NO_ROLE` ;
 * - le compte qui le porte n'est pas imputable (il n'est pas dans la grille) →
 *   `NOT_POSTABLE` ;
 * - aucun montant sur lui → `NO_AMOUNT`.
 */
export function retainedEarningsWarning(
	accounts: AccountResponse[],
	rows: { account: AccountResponse; debit: string; credit: string }[],
	isBalanced: boolean
): RetainedEarningsWarning | null {
	if (!isBalanced) return null;
	const holder = accounts.find((a) => a.active && a.role === 'RetainedEarnings');
	if (!holder) return { kind: 'NO_ROLE' };
	if (!holder.postable) return { kind: 'NOT_POSTABLE', account: holder };
	const row = rows.find((r) => r.account.id === holder.id);
	const touched = row !== undefined && (amount(row.debit).gt(0) || amount(row.credit).gt(0));
	return touched ? null : { kind: 'NO_AMOUNT', account: holder };
}

/**
 * Contrepartie du complément, telle que le serveur la calculera (arbitrage 3) :
 * écart = Σ débit − Σ crédit des lignes saisies ; > 0 → au crédit du report,
 * < 0 → au débit, = 0 → aucune.
 */
export function complementCounterpart(
	lines: { debit: string; credit: string }[]
): { side: 'credit' | 'debit' | 'none'; amount: Big } {
	const gap = lines.reduce(
		(acc, l) => acc.plus(amount(l.debit)).minus(amount(l.credit)),
		new Big(0)
	);
	if (gap.gt(0)) return { side: 'credit', amount: gap };
	if (gap.lt(0)) return { side: 'debit', amount: gap.abs() };
	return { side: 'none', amount: new Big(0) };
}
