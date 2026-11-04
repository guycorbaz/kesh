/**
 * Les motifs qui empêchent de **repasser en brouillon** une facture fournisseur (route
 * `unvalidate`) — Story v017-4b (#849, AC-25) ; Story v0172-annulees (#868, AC-8) : le
 * geste vaut aussi pour une facture annulée, et chaque refus dit **quoi faire d'abord**,
 * en nommant l'objet à défaire. Patron de `supplierInvoiceCancelMessage`
 * (`invoice-cancel.ts`) : une fonction pure `(code, details)`, un texte par code, un repli
 * générique.
 *
 * Les codes sont ceux de la précédence du serveur (`supplier_invoices::unvalidate_in_tx`) :
 * - cinq empêchements **propres à la pièce** — `SUPPLIER_INVOICE_PAID`,
 *   `SUPPLIER_INVOICE_PAID_RECONCILED`, `SUPPLIER_INVOICE_IN_PAYMENT_BATCH`,
 *   `SUPPLIER_INVOICE_DETACHED_PAYMENT`, `MATCHED_BANK_TRANSACTION` — dont les textes sont
 *   **ceux du serveur**, mêmes clés `error-supplier-invoice-unvalidate-blocked-*`
 *   (`supplier_unvalidation_message`, `errors.rs`), mêmes arguments, tirés de `details` :
 *   le lot (`batchId`), la date de la transaction (`transactionDate`), l'écriture de
 *   paiement (`entryNumber`, `entryDate` — le numéro repart à 1 à chaque exercice) ;
 * - l'état et la version — `ILLEGAL_STATE_TRANSITION`, `OPTIMISTIC_LOCK_CONFLICT` ;
 * - ceux de la suppression des écritures (`journal_entries::delete_in_tx`) —
 *   `FISCAL_YEAR_CLOSED`, `LATER_FISCAL_YEAR_CLOSED`, `ENTRY_IS_REVERSED` (inatteignable :
 *   pour une annulée, la contre-passation part d'abord ; gardé comme au serveur),
 *   `PERIOD_LOCKED`, `ENTRY_LETTERED` — dont le message serveur parle d'« écriture » : ici,
 *   il dit ce qui bloque le geste et qui le lève.
 *
 * ⚠️ Les replis en dur disent **mot pour mot** le FTL fr-CH. Aucun ne nomme l'état validé
 * « ouverte » (décision Q4, C-v017-4-24) ; le vocabulaire est « repasser en brouillon »
 * (C-v0172a-20).
 */
import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
import { formatSwissDate } from '$lib/features/reports/reports.api';

/** Les codes du repassage en brouillon, dans l'ordre de la précédence du serveur. */
export const SUPPLIER_INVOICE_UNVALIDATE_CODES = [
	'SUPPLIER_INVOICE_PAID',
	'SUPPLIER_INVOICE_PAID_RECONCILED',
	'ILLEGAL_STATE_TRANSITION',
	'OPTIMISTIC_LOCK_CONFLICT',
	'SUPPLIER_INVOICE_IN_PAYMENT_BATCH',
	'SUPPLIER_INVOICE_DETACHED_PAYMENT',
	'MATCHED_BANK_TRANSACTION',
	'FISCAL_YEAR_CLOSED',
	'LATER_FISCAL_YEAR_CLOSED',
	'ENTRY_IS_REVERSED',
	'PERIOD_LOCKED',
	'ENTRY_LETTERED',
] as const;

export type SupplierInvoiceUnvalidateCode = (typeof SUPPLIER_INVOICE_UNVALIDATE_CODES)[number];

/**
 * Ce que lit le message dans `details` (AC-8) — les cinq champs neufs, toujours présents
 * dans la réponse du serveur (`null` quand ils ne s'appliquent pas), plus le nom de
 * l'exercice postérieur clos (`LATER_FISCAL_YEAR_CLOSED`, revue P1 E-4).
 */
export interface SupplierInvoiceUnvalidateDetails {
	batchId?: number | null;
	transactionDate?: string | null;
	entryId?: number | null;
	entryNumber?: number | null;
	entryDate?: string | null;
	fiscalYearName?: string | null;
}

/** Une date `AAAA-MM-JJ` au format suisse, ou rien. */
function swissDate(iso: string | null | undefined): string {
	return iso ? formatSwissDate(iso.slice(0, 10)) : '';
}

/**
 * Le message d'un refus du repassage en brouillon. `code` est celui de la réponse
 * (`err.code`) ; `details`, ses `details` (ou `null`). Un code hors de la précédence rend
 * le repli.
 */
export function supplierInvoiceUnvalidateMessage(
	code: string,
	details: SupplierInvoiceUnvalidateDetails | null,
): string {
	const d = details ?? {};
	switch (code) {
		case 'SUPPLIER_INVOICE_PAID':
			return i18nMsg(
				'error-supplier-invoice-unvalidate-blocked-paid',
				"Cette facture fournisseur est payée : annulez d'abord son règlement (bouton « Annuler le règlement » de cette fiche), puis repassez-la en brouillon.",
			);
		case 'SUPPLIER_INVOICE_PAID_RECONCILED':
			return i18nMsg(
				'error-supplier-invoice-unvalidate-blocked-paid-reconciled',
				"Cette facture fournisseur est payée par la transaction bancaire du { $date } : annulez d'abord ce rapprochement, depuis le détail de l'import bancaire — elle redeviendra validée —, puis repassez-la en brouillon.",
				{ date: swissDate(d.transactionDate) },
			);
		case 'SUPPLIER_INVOICE_IN_PAYMENT_BATCH':
			return i18nMsg(
				'error-supplier-invoice-unvalidate-blocked-in-payment-batch',
				"Cette facture figure dans le lot de paiement n° { $batch }, en cours : annulez d'abord ce lot, dans Paiements fournisseurs, puis repassez-la en brouillon.",
				{ batch: d.batchId ?? '' },
			);
		case 'SUPPLIER_INVOICE_DETACHED_PAYMENT':
			// Paiement rapproché : la clé qui nomme la transaction ; sinon, la clé sans date.
			if (d.transactionDate) {
				return i18nMsg(
					'error-supplier-invoice-unvalidate-blocked-detached-payment-reconciled',
					"Cette facture a été annulée alors qu'elle était payée : son paiement (écriture n° { $entry } du { $entryDate }) est rapproché de la transaction bancaire du { $date }. Si ce paiement n'a pas eu lieu, annulez d'abord ce rapprochement, depuis le détail de l'import bancaire — il contre-passera le paiement —, puis repassez la facture en brouillon.",
					{
						entry: d.entryNumber ?? '',
						entryDate: swissDate(d.entryDate),
						date: swissDate(d.transactionDate),
					},
				);
			}
			return i18nMsg(
				'error-supplier-invoice-unvalidate-blocked-detached-payment',
				"Cette facture a été annulée alors qu'elle était payée : son paiement (écriture n° { $entry } du { $entryDate }) reste au grand livre. Si ce paiement n'a pas eu lieu, contre-passez-le d'abord depuis sa fiche, puis repassez la facture en brouillon.",
				{ entry: d.entryNumber ?? '', entryDate: swissDate(d.entryDate) },
			);
		case 'MATCHED_BANK_TRANSACTION':
			return i18nMsg(
				'error-supplier-invoice-unvalidate-blocked-matched',
				"L'écriture d'achat de cette facture est rapprochée de la transaction bancaire du { $date } : annulez d'abord ce rapprochement, depuis le détail de l'import bancaire, puis repassez la facture en brouillon.",
				{ date: swissDate(d.transactionDate) },
			);
		case 'OPTIMISTIC_LOCK_CONFLICT':
			return i18nMsg(
				'supplier-invoices-unvalidate-blocked-stale',
				'La facture a été modifiée entre-temps : rechargez la page, puis réessayez.',
			);
		case 'ILLEGAL_STATE_TRANSITION':
			return i18nMsg(
				'supplier-invoices-unvalidate-blocked-not-validated',
				'Cette facture est déjà un brouillon : rechargez la page pour voir son état.',
			);
		case 'FISCAL_YEAR_CLOSED':
			return i18nMsg(
				'supplier-invoices-unvalidate-blocked-fiscal-year-closed',
				'L’écriture d’achat de cette facture appartient à un exercice clôturé : pour pouvoir la repasser en brouillon, un administrateur doit rouvrir les exercices clôturés jusqu’à celui-ci, en commençant par le plus récent.',
			);
		case 'LATER_FISCAL_YEAR_CLOSED':
			// Revue P1 (E-4) : le serveur nomme l'exercice postérieur clos
			// (`details.fiscalYearName`) ; l'écran le reprend quand il l'a.
			if (d.fiscalYearName) {
				return i18nMsg(
					'supplier-invoices-unvalidate-blocked-later-fiscal-year-closed-named',
					'L’exercice postérieur « { $name } » est clôturé : pour pouvoir repasser cette facture en brouillon, un administrateur doit rouvrir les exercices postérieurs clôturés, en commençant par le plus récent.',
					{ name: d.fiscalYearName },
				);
			}
			return i18nMsg(
				'supplier-invoices-unvalidate-blocked-later-fiscal-year-closed',
				'Un exercice postérieur à celui de cette facture est clôturé : pour pouvoir la repasser en brouillon, un administrateur doit rouvrir les exercices postérieurs clôturés, en commençant par le plus récent.',
			);
		case 'ENTRY_IS_REVERSED':
			return i18nMsg(
				'supplier-invoices-unvalidate-blocked-reversed',
				'L’écriture d’achat de cette facture a été contre-passée : elle ne peut plus être supprimée, et la facture ne repasse pas en brouillon.',
			);
		case 'PERIOD_LOCKED':
			return i18nMsg(
				'supplier-invoices-unvalidate-blocked-period-locked',
				'L’écriture d’achat de cette facture est datée sous le verrou de période : pour pouvoir la repasser en brouillon, un administrateur doit faire reculer le verrou avant sa date.',
			);
		case 'ENTRY_LETTERED':
			return i18nMsg(
				'supplier-invoices-unvalidate-blocked-lettered',
				'L’écriture d’achat de cette facture est lettrée : délettrez-la d’abord, puis repassez la facture en brouillon.',
			);
		default:
			return i18nMsg('supplier-invoices-unvalidate-error', 'La facture n’a pas pu repasser en brouillon.');
	}
}
