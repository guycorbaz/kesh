/**
 * Les motifs qui empêchent de **dévalider** une facture fournisseur — Story v017-4b
 * (#849, AC-25). Patron de `supplierInvoiceCancelMessage` (`invoice-cancel.ts`) : une
 * fonction pure `(code, label)`, un texte par code, un repli générique.
 *
 * Les codes sont ceux de la précédence du serveur
 * (`supplier_invoices::unvalidate_in_tx`, fiche parente § *Précédence*) :
 * - trois empêchements **propres à la pièce** — `SUPPLIER_INVOICE_PAID`,
 *   `SUPPLIER_INVOICE_IN_PAYMENT_BATCH`, `MATCHED_BANK_TRANSACTION` — dont les textes
 *   sont **ceux du serveur**, mêmes clés `error-supplier-invoice-unvalidate-blocked-*`
 *   (`supplier_unvalidation_blocked_text`, `errors.rs`), suffixés comme lui du numéro du
 *   document qui empêche (`details.documentNumber`, le `label`) ;
 * - l'état et la version — `ILLEGAL_STATE_TRANSITION`, `OPTIMISTIC_LOCK_CONFLICT` ;
 * - ceux de la suppression de l'écriture (`journal_entries::delete_in_tx`) —
 *   `FISCAL_YEAR_CLOSED`, `LATER_FISCAL_YEAR_CLOSED`, `ENTRY_IS_REVERSED` (inatteignable,
 *   gardé comme au serveur), `PERIOD_LOCKED`, `ENTRY_LETTERED` — dont le message serveur
 *   parle d'« écriture » : ici, il dit ce qui bloque la **dévalidation** et qui le lève.
 *
 * ⚠️ Les replis en dur disent **mot pour mot** le FTL fr-CH. Aucun ne nomme l'état
 * validé « ouverte » (décision Q4, C-v017-4-24).
 */
import { i18nMsg } from '$lib/shared/utils/i18n.svelte';

/** Les codes de la dévalidation, dans l'ordre de la précédence du serveur. */
export const SUPPLIER_INVOICE_UNVALIDATE_CODES = [
	'SUPPLIER_INVOICE_PAID',
	'ILLEGAL_STATE_TRANSITION',
	'OPTIMISTIC_LOCK_CONFLICT',
	'SUPPLIER_INVOICE_IN_PAYMENT_BATCH',
	'MATCHED_BANK_TRANSACTION',
	'FISCAL_YEAR_CLOSED',
	'LATER_FISCAL_YEAR_CLOSED',
	'ENTRY_IS_REVERSED',
	'PERIOD_LOCKED',
	'ENTRY_LETTERED',
] as const;

export type SupplierInvoiceUnvalidateCode = (typeof SUPPLIER_INVOICE_UNVALIDATE_CODES)[number];

/** Le texte d'un motif propre à la pièce, suffixé comme au serveur. */
function withLabel(base: string, label: string | null): string {
	return label ? `${base} (${label})` : base;
}

/**
 * Le message d'un refus de la dévalidation. `code` est celui de la réponse (`err.code`) ;
 * `label`, le numéro du document qui empêche (`details.documentNumber`), n'est lu que
 * pour les trois motifs propres à la pièce. Un code hors de la précédence rend le repli.
 */
export function supplierInvoiceUnvalidateMessage(code: string, label: string | null): string {
	switch (code) {
		case 'SUPPLIER_INVOICE_PAID':
			return withLabel(
				i18nMsg(
					'error-supplier-invoice-unvalidate-blocked-paid',
					"Cette facture fournisseur est payée : annulez d'abord son règlement.",
				),
				label,
			);
		case 'SUPPLIER_INVOICE_IN_PAYMENT_BATCH':
			return withLabel(
				i18nMsg(
					'error-supplier-invoice-unvalidate-blocked-in-payment-batch',
					"Cette facture figure dans un lot de paiement en cours : annulez d'abord le lot.",
				),
				label,
			);
		case 'MATCHED_BANK_TRANSACTION':
			return withLabel(
				i18nMsg(
					'error-supplier-invoice-unvalidate-blocked-matched',
					"L'écriture d'achat de cette facture est rapprochée d'une transaction bancaire : annulez d'abord le rapprochement.",
				),
				label,
			);
		case 'OPTIMISTIC_LOCK_CONFLICT':
			return i18nMsg(
				'supplier-invoices-unvalidate-blocked-stale',
				'La facture a été modifiée entre-temps : rechargez la page, puis réessayez.',
			);
		case 'ILLEGAL_STATE_TRANSITION':
			return i18nMsg(
				'supplier-invoices-unvalidate-blocked-not-validated',
				'Cette facture n’est plus validée : rechargez la page pour voir son état.',
			);
		case 'FISCAL_YEAR_CLOSED':
			return i18nMsg(
				'supplier-invoices-unvalidate-blocked-fiscal-year-closed',
				'L’écriture d’achat de cette facture appartient à un exercice clôturé : pour pouvoir la dévalider, un administrateur doit rouvrir les exercices clôturés jusqu’à celui-ci, en commençant par le plus récent.',
			);
		case 'LATER_FISCAL_YEAR_CLOSED':
			return i18nMsg(
				'supplier-invoices-unvalidate-blocked-later-fiscal-year-closed',
				'Un exercice postérieur à celui de cette facture est clôturé : pour pouvoir la dévalider, un administrateur doit rouvrir les exercices postérieurs clôturés, en commençant par le plus récent.',
			);
		case 'ENTRY_IS_REVERSED':
			return i18nMsg(
				'supplier-invoices-unvalidate-blocked-reversed',
				'L’écriture d’achat de cette facture a été contre-passée : elle ne peut plus être supprimée, et la facture ne se dévalide pas.',
			);
		case 'PERIOD_LOCKED':
			return i18nMsg(
				'supplier-invoices-unvalidate-blocked-period-locked',
				'L’écriture d’achat de cette facture est datée sous le verrou de période : pour pouvoir la dévalider, un administrateur doit faire reculer le verrou avant sa date.',
			);
		case 'ENTRY_LETTERED':
			return i18nMsg(
				'supplier-invoices-unvalidate-blocked-lettered',
				'L’écriture d’achat de cette facture est lettrée : délettrez-la d’abord, puis dévalidez la facture.',
			);
		default:
			return i18nMsg('supplier-invoices-unvalidate-error', 'La facture n’a pas pu être dévalidée.');
	}
}
