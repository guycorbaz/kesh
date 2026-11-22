/**
 * Story 38-11d (AC-3, AC-4) — règles de la page PayPal, hors composant pour
 * être testées seules : exercice par défaut, clé et sélection des
 * propositions, corps de l'acceptation, ce que fera chaque proposition.
 */
import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
import type { AccountResponse } from '$lib/features/accounts/accounts.types';
import type { FiscalYearResponse } from '$lib/features/fiscal-years/fiscal-years.types';
import type { PaypalAcceptProposal, PaypalProposal } from './paypal.types';

/**
 * L'exercice proposé par défaut : celui qui contient `today` (`AAAA-MM-JJ`),
 * sinon le plus récent ; `null` sans exercice.
 */
export function defaultFiscalYearId(years: readonly FiscalYearResponse[], today: string): number | null {
	const current = years.find((y) => y.startDate <= today && today <= y.endDate);
	if (current) return current.id;
	const sorted = [...years].sort((a, b) => b.startDate.localeCompare(a.startDate));
	return sorted[0]?.id ?? null;
}

/** La transaction qui identifie une proposition (la débitée d'une conversion). */
export function proposalTransactionId(p: PaypalProposal): number {
	return p.type === 'conversion' ? p.debitTransactionId : p.bankTransactionId;
}

/** Clé stable d'une proposition (cases à cocher, sélecteurs). */
export function proposalKey(p: PaypalProposal): string {
	return `${p.type}-${proposalTransactionId(p)}`;
}

/**
 * Le corps d'acceptation d'une proposition ; `null` si elle est incomplète (un
 * retrait sans compte de destination). `withdrawalAccounts` : le compte choisi
 * à l'écran, par clé de proposition.
 */
export function acceptBody(
	p: PaypalProposal,
	withdrawalAccounts: Readonly<Record<string, number | null>>,
): PaypalAcceptProposal | null {
	switch (p.type) {
		case 'link_entry':
			return { type: 'link_entry', bankTransactionId: p.bankTransactionId, journalEntryId: p.journalEntryId };
		case 'fee':
			return { type: 'fee', bankTransactionId: p.bankTransactionId, feeAccountId: p.feeAccountId };
		case 'rule':
			return {
				type: 'rule',
				bankTransactionId: p.bankTransactionId,
				ruleId: p.ruleId,
				counterpartyAccountId: p.counterpartyAccountId,
			};
		case 'conversion':
			return {
				type: 'conversion',
				debitTransactionId: p.debitTransactionId,
				creditTransactionId: p.creditTransactionId,
			};
		case 'withdrawal': {
			const key = proposalKey(p);
			const chosen = key in withdrawalAccounts ? withdrawalAccounts[key] : p.counterpartyAccountId;
			return chosen === null || chosen === undefined
				? null
				: { type: 'withdrawal', bankTransactionId: p.bankTransactionId, counterpartyAccountId: chosen };
		}
	}
}

/**
 * Les corps à envoyer pour la sélection, **dans l'ordre des dates** (puis de la
 * transaction) ; les propositions incomplètes sont écartées.
 */
export function selectedBodies(
	proposals: readonly PaypalProposal[],
	selected: ReadonlySet<string>,
	withdrawalAccounts: Readonly<Record<string, number | null>>,
): PaypalAcceptProposal[] {
	return [...proposals]
		.filter((p) => selected.has(proposalKey(p)))
		.sort(
			(a, b) =>
				a.transaction.date.localeCompare(b.transaction.date) ||
				proposalTransactionId(a) - proposalTransactionId(b),
		)
		.map((p) => acceptBody(p, withdrawalAccounts))
		.filter((b): b is PaypalAcceptProposal => b !== null);
}

/**
 * Les comptes offerts comme destination d'un retrait : actifs, imputables, de
 * type actif, en CHF (le retrait part d'un solde CHF, 38-11c2 AC-5), **hors** les
 * comptes de `excludedIds` : les comptes du plan du compte PayPal lui-même, que
 * le serveur refuse comme contrepartie (revue P3, L1). Le compte de conversion
 * n'est pas refusé par le serveur : il reste offert.
 */
export function withdrawalAccountCandidates(
	accounts: readonly AccountResponse[],
	excludedIds: readonly number[] = [],
): AccountResponse[] {
	return accounts
		.filter(
			(a) =>
				a.active &&
				a.postable &&
				a.accountType === 'Asset' &&
				!a.currency &&
				!excludedIds.includes(a.id),
		)
		.sort((a, b) => a.number.localeCompare(b.number));
}

function accountName(accounts: readonly AccountResponse[], id: number): string {
	const a = accounts.find((x) => x.id === id);
	return a ? `${a.number} — ${a.name}` : `#${id}`;
}

/** Ce que fera une proposition (colonne « Ce qui sera fait »), hors retrait sans compte. */
export function proposalActionLabel(p: PaypalProposal, accounts: readonly AccountResponse[]): string {
	switch (p.type) {
		case 'link_entry':
			return i18nMsg('paypal-action-link-entry', 'Rattachée à l’écriture n° { $number }', {
				number: String(p.entry?.entryNumber ?? p.journalEntryId),
			});
		case 'fee':
			return i18nMsg('paypal-action-fee', 'Écriture au compte de frais { $account }', {
				account: accountName(accounts, p.feeAccountId),
			});
		case 'rule':
			return i18nMsg('paypal-action-rule', 'Écriture à la contrepartie { $account }', {
				account: accountName(accounts, p.counterpartyAccountId),
			});
		case 'conversion':
			return p.transaction.currency === 'CHF' || p.otherTransaction?.currency === 'CHF'
				? i18nMsg('paypal-action-conversion-one', 'Une écriture, au CHF réel')
				: i18nMsg('paypal-action-conversion-two', 'Deux écritures, par le compte de conversion');
		case 'withdrawal':
			return p.counterpartyAccountId === null
				? i18nMsg('paypal-action-withdrawal-choose', 'Virement vers le compte choisi')
				: i18nMsg('paypal-action-withdrawal', 'Virement vers { $account }', {
						account: accountName(accounts, p.counterpartyAccountId),
					});
	}
}
