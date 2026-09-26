/**
 * Ce qui empêche d'annuler un rapprochement — ses PROPRES textes (Story 25-3-b, #418).
 *
 * ⛔ **Pas la famille `settlement-cancel-blocked-*`**, qui dit « ce règlement » :
 * une écriture d'éclatement, de règle ou de rapprochement manuel n'est pas un
 * règlement. Le dialogue traduit toujours le **code** reçu — par la lecture
 * (`GET …/transactions/{id}`) comme par le refus au clic —, pour les six motifs,
 * `INVOICE_CREDITED` compris.
 *
 * ⚠️ Les replis en dur disent **mot pour mot** le FTL fr-CH — le serveur rend
 * les mêmes clés quand il refuse (`crates/kesh-api/src/errors.rs`,
 * `reconciliation_cancel_blocked_text`).
 */
import type { ApiError } from '$lib/shared/types/api';
import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
import type { ReconciliationCancelCode } from './reconciliation.types';

const MOTIFS: readonly ReconciliationCancelCode[] = [
	'BANK_TRANSACTION_NOT_RECONCILED',
	'INVOICE_CREDITED',
	'FISCAL_YEAR_CLOSED',
	'MATCHED_BANK_TRANSACTION',
	'ACCOUNT_ARCHIVED',
	'FISCAL_YEAR_INVALID',
];

/** Le code est-il l'un des six motifs du dé-rapprochement ? */
export function isReconciliationCancelCode(code: string): code is ReconciliationCancelCode {
	return (MOTIFS as readonly string[]).includes(code);
}

/**
 * Le texte d'un motif. `label` porte le **numéro du compte** archivé — sans
 * lui, « réactivez-le » ne dirait pas lequel.
 */
export function reconciliationCancelMessage(
	code: ReconciliationCancelCode,
	label: string | null,
): string {
	switch (code) {
		case 'BANK_TRANSACTION_NOT_RECONCILED':
			return i18nMsg(
				'reconciliation-cancel-blocked-not-reconciled',
				"Cette transaction bancaire n'est pas rapprochée : il n'y a pas de rapprochement à annuler.",
			);
		case 'INVOICE_CREDITED':
			return i18nMsg(
				'reconciliation-cancel-blocked-credited',
				"La facture de ce rapprochement a été créditée par un avoir : son règlement est un paiement à lettrer, il ne s'annule pas.",
			);
		case 'FISCAL_YEAR_CLOSED':
			return i18nMsg(
				'reconciliation-cancel-blocked-fiscal-year-closed',
				"Ce rapprochement appartient à un exercice clôturé : un administrateur doit rouvrir l'exercice pour pouvoir l'annuler.",
			);
		case 'MATCHED_BANK_TRANSACTION':
			return i18nMsg(
				'reconciliation-cancel-blocked-bank-match',
				"L'écriture de ce rapprochement est aussi rapprochée d'une autre transaction bancaire : annulez d'abord cet autre rapprochement.",
			);
		case 'ACCOUNT_ARCHIVED': {
			const base = i18nMsg(
				'reconciliation-cancel-blocked-account-archived',
				"Un compte de l'écriture de ce rapprochement a été archivé : réactivez-le pour pouvoir annuler le rapprochement.",
			);
			return label ? `${base} (${label})` : base;
		}
		case 'FISCAL_YEAR_INVALID':
			return i18nMsg(
				'reconciliation-cancel-blocked-no-fiscal-year',
				'Aucun exercice ouvert ne couvre la date du jour : créez-le pour pouvoir annuler ce rapprochement.',
			);
		default: {
			// ⛔ C'est l'affectation à `never` qui fait rougir le type-check si un
			// septième motif s'ajoute au type sans s'ajouter ici.
			const exhaustive: never = code;
			return exhaustive;
		}
	}
}

/**
 * Le texte d'un refus **au clic**. Un motif des six → son texte (le 400
 * `ACCOUNT_ARCHIVED` nomme le compte dans `details.rejected[]`). ⛔ **Tout autre
 * code** — `PERIOD_LOCKED`, `OPTIMISTIC_LOCK_CONFLICT`,
 * `RECONCILIATION_ACCOUNT_LOCKED`, `NOT_FOUND`… — affiche le **message du
 * serveur**, déjà traduit et propre à chacun : jamais un texte générique.
 */
export function reconciliationCancelErrorMessage(err: ApiError): string {
	if (!isReconciliationCancelCode(err.code)) return err.message;
	let label: string | null = null;
	const rejected = err.details?.rejected;
	if (err.code === 'ACCOUNT_ARCHIVED' && Array.isArray(rejected)) {
		const numbers = rejected
			.map((r) => (r as { accountNumber?: unknown }).accountNumber)
			.filter((n): n is string => typeof n === 'string');
		label = numbers.length > 0 ? numbers.join(', ') : null;
	}
	return reconciliationCancelMessage(err.code, label);
}
