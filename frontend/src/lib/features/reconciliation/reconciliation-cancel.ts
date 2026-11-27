/**
 * Ce qui empêche d'annuler un rapprochement — ses PROPRES textes (Story 25-3-b, #418).
 *
 * ⛔ **Pas la famille `settlement-cancel-blocked-*`**, qui dit « ce règlement » :
 * une écriture d'éclatement, de règle ou de rapprochement manuel n'est pas un
 * règlement. Le dialogue traduit toujours le **code** reçu — par la lecture
 * (`GET …/transactions/{id}`) comme par le refus au clic —, pour les neuf motifs,
 * `INVOICE_CREDITED` compris, et `PERIOD_LOCKED` depuis la Story 53-6a (#1674 : la
 * contre-passation est datée de l'écriture, que la borne du verrou peut couvrir).
 *
 * ⚠️ Les replis en dur disent **mot pour mot** le FTL fr-CH — le serveur rend
 * les mêmes clés quand il refuse (`crates/kesh-api/src/errors.rs`,
 * `reconciliation_cancel_blocked_text`).
 */
import { formatSwissDate } from '$lib/features/reports/reports.api';
import type { ApiError } from '$lib/shared/types/api';
import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
import type {
	CancelReconciliationResponse,
	ReconciliationCancelCode,
} from './reconciliation.types';

const MOTIFS: readonly ReconciliationCancelCode[] = [
	'BANK_TRANSACTION_NOT_RECONCILED',
	'INVOICE_CREDITED',
	'INVOICE_WRITTEN_OFF',
	'FISCAL_YEAR_CLOSED',
	'LETTERING_ALL_LINES_IN_CLOSED_PERIODS',
	'MATCHED_BANK_TRANSACTION',
	'ACCOUNT_ARCHIVED',
	'PERIOD_LOCKED',
	'FISCAL_YEAR_INVALID',
];

/** Le code est-il l'un des neuf motifs du dé-rapprochement ? */
export function isReconciliationCancelCode(code: string): code is ReconciliationCancelCode {
	return (MOTIFS as readonly string[]).includes(code);
}

/**
 * Le texte d'un motif. `label` porte le **numéro du compte** archivé — sans
 * lui, « réactivez-le » ne dirait pas lequel —, ou la **borne** du verrou de
 * période (`AAAA-MM-JJ`, `PERIOD_LOCKED`), ajoutée en date suisse.
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
				"La facture de ce rapprochement a été créditée par un avoir : son règlement reste ouvert au compte débiteurs, il ne s'annule pas.",
			);
		case 'INVOICE_WRITTEN_OFF':
			return i18nMsg(
				'reconciliation-cancel-blocked-written-off',
				"Le reste de la facture de ce rapprochement a été soldé : annulez d'abord le solde.",
			);
		case 'FISCAL_YEAR_CLOSED':
			return i18nMsg(
				'reconciliation-cancel-blocked-fiscal-year-closed',
				"Ce rapprochement appartient à un exercice clôturé : pour pouvoir l'annuler, un administrateur doit rouvrir les exercices clôturés jusqu'à celui-ci, en commençant par le plus récent.",
			);
		case 'LETTERING_ALL_LINES_IN_CLOSED_PERIODS':
			return i18nMsg(
				'reconciliation-cancel-blocked-lettering-closed',
				"Ce rapprochement est lettré avec sa facture, et toutes les lignes de ce lettrage sont dans une période close : il est figé. Pour pouvoir l'annuler, prenez la date la plus récente du lettrage (en général celle du dernier règlement) : si elle est sous le verrou de période, un administrateur doit faire reculer le verrou avant elle ; et si son exercice est suivi d'un exercice clôturé, ou clôturé lui-même, il doit rouvrir les exercices clôturés jusqu'à celui-ci, en commençant par le plus récent.",
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
		case 'PERIOD_LOCKED': {
			// Story 53-6a (#1674, D5) — clé sans variable (le serveur la rend telle quelle) ; la
			// borne s'ajoute comme le numéro du compte archivé, en date suisse.
			const base = i18nMsg(
				'reconciliation-cancel-blocked-period-locked',
				"L'écriture de ce rapprochement est dans une période verrouillée : pour pouvoir l'annuler, un administrateur doit faire reculer le verrou avant sa date.",
			);
			return label ? `${base} (${formatSwissDate(label)})` : base;
		}
		// Inatteignable pour le dé-rapprochement depuis la Story 53-6a ; clé et repli gardés.
		case 'FISCAL_YEAR_INVALID':
			return i18nMsg(
				'reconciliation-cancel-blocked-no-fiscal-year',
				'Aucun exercice ouvert ne couvre la date du jour : créez-le pour pouvoir annuler ce rapprochement.',
			);
		default: {
			// ⛔ C'est l'affectation à `never` qui fait rougir le type-check si un
			// dixième motif s'ajoute au type sans s'ajouter ici.
			const exhaustive: never = code;
			return exhaustive;
		}
	}
}

/**
 * Story 34-11b (#972, AC-4b) — le message de succès d'une annulation : distinct quand seul le
 * lien a été retiré (`reversalJournalEntryId: null`, l'autre côté d'un virement reste
 * rapproché de l'écriture).
 */
export function reconciliationCancelDoneMessage(result: CancelReconciliationResponse): string {
	if (result.reversalJournalEntryId === null) {
		return i18nMsg(
			'reconciliation-cancel-done-unlink-only',
			'Le lien est retiré ; l’écriture reste celle de l’autre côté du virement.',
		);
	}
	// Story 43-3 (#823, AC5) — un règlement groupé défait rouvre ses N factures.
	const reopened = result.cancelledInvoices?.length ?? 0;
	// Revue P1 (B-L12) : des factures fournisseur redeviennent validées, à payer, non « à régler ».
	if (reopened > 0 && result.cancelledInvoices?.some((p) => p.kind === 'supplier_invoice')) {
		return i18nMsg(
			'reconciliation-cancel-done-group-supplier',
			'Rapprochement annulé : { $count } factures fournisseur redeviennent validées, à payer.',
			{ count: reopened },
		);
	}
	if (reopened > 0) {
		return i18nMsg(
			'reconciliation-cancel-done-group',
			'Rapprochement annulé : { $count } factures redeviennent à régler.',
			{ count: reopened },
		);
	}
	return i18nMsg('reconciliation-cancel-done', 'Rapprochement annulé.');
}

/**
 * Story 43-3 (#823, AC5) — le motif d'un refus d'annulation **par facture** d'un règlement
 * groupé, au pluriel du groupe, qui nomme la facture en cause (`cancelBlockedInvoice` de la
 * vue) : créditée, soldée, ou lettrage figé. `null` pour les autres motifs, qui gardent leur
 * texte (ils portent sur l'écriture ou l'exercice, non sur une facture).
 */
export function groupedCancelBlockedMessage(
	code: ReconciliationCancelCode,
	invoice: string,
): string | null {
	switch (code) {
		case 'INVOICE_CREDITED':
			return i18nMsg(
				'reconciliation-cancel-blocked-group-credited',
				"Une des factures de ce rapprochement, { $invoice }, a été créditée par un avoir : son règlement reste ouvert au compte débiteurs, le rapprochement ne s'annule pas.",
				{ invoice },
			);
		case 'INVOICE_WRITTEN_OFF':
			return i18nMsg(
				'reconciliation-cancel-blocked-group-written-off',
				"Le reste d'une des factures de ce rapprochement, { $invoice }, a été soldé : annulez d'abord le solde.",
				{ invoice },
			);
		case 'LETTERING_ALL_LINES_IN_CLOSED_PERIODS':
			return i18nMsg(
				'reconciliation-cancel-blocked-group-lettering-closed',
				"Une des factures de ce rapprochement, { $invoice }, est lettrée avec lui, et toutes les lignes de ce lettrage sont dans une période close : il est figé. Pour pouvoir l'annuler, prenez la date la plus récente du lettrage : si elle est sous le verrou de période, un administrateur doit faire reculer le verrou avant elle ; et si son exercice est suivi d'un exercice clôturé, ou clôturé lui-même, il doit rouvrir les exercices clôturés jusqu'à celui-ci, en commençant par le plus récent.",
				{ invoice },
			);
		default:
			return null;
	}
}

/**
 * Le texte d'un refus **au clic**. Un motif des neuf → son texte (le 400
 * `ACCOUNT_ARCHIVED` nomme le compte dans `details.rejected[]` ; le 400
 * `PERIOD_LOCKED` du socle porte la borne dans `details.lockedThrough`, que le
 * 409 d'un règlement groupé n'a pas — le texte est alors rendu sans elle).
 * ⛔ **Tout autre code** — `LATER_FISCAL_YEAR_CLOSED`, `OPTIMISTIC_LOCK_CONFLICT`,
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
	const lockedThrough = err.details?.lockedThrough;
	if (err.code === 'PERIOD_LOCKED' && typeof lockedThrough === 'string') {
		label = lockedThrough;
	}
	return reconciliationCancelMessage(err.code, label);
}
