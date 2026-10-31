/**
 * Calculs et traductions du formulaire de facture fournisseur — Story 30-2b (#685,
 * AC10 à AC12-bis, AC16).
 *
 * Extraits du formulaire de complétion en ligne de la page d'import (12-5d) et du
 * formulaire en ligne de la liste (12-2), qui disparaissent au profit de
 * `SupplierInvoiceForm.svelte` : une définition au lieu de deux (DRY).
 */

import Big from 'big.js';
import { lineVatAmount } from '$lib/features/journal-entries/vat-purchase';
import { isValidAmount } from '$lib/features/journal-entries/balance';
import { isApiError } from '$lib/shared/utils/api-error';
import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
import { normalizeIban, qrIbanHasValidQrIid } from '$lib/shared/utils/iban';
import type {
	CreateSupplierInvoiceLineRequest,
	CreateSupplierInvoiceRequest,
} from './supplier-invoices.types';

/** Une ligne vide, telle que le formulaire l'ajoute. */
export function emptyLine(): CreateSupplierInvoiceLineRequest {
	return { description: '', quantity: '1', unitPrice: '', vatRate: '0', expenseAccountId: 0 };
}

/**
 * Σ TTC pleine précision, en parité avec le serveur (`Σ line_total + Σ
 * line_vat_amount`, `create_in_tx`) ; une ligne incomplète est ignorée.
 */
export function sumTtc(lines: CreateSupplierInvoiceLineRequest[]): Big {
	let total = new Big(0);
	for (const l of lines) {
		let lineTotal: Big;
		try {
			lineTotal = new Big(l.quantity || '0').times(l.unitPrice || '0');
		} catch {
			continue; // saisie incomplète → ignorée pour le total vivant
		}
		total = total.plus(lineTotal);
		try {
			total = total.plus(lineVatAmount(lineTotal.toString(), l.vatRate || '0'));
		} catch {
			// taux non numérique → TVA ignorée pour le total vivant
		}
	}
	return total;
}

/**
 * Indicateur visuel : le total TTC diffère-t-il du montant du QR (pleine précision) ?
 * Le serveur reste juge (`AMOUNT_MISMATCH`) : cet indicateur ne bloque rien.
 */
export function amountMismatch(lines: CreateSupplierInvoiceLineRequest[], target: string | null): boolean {
	if (!target) return false;
	try {
		return !sumTtc(lines).eq(new Big(target));
	} catch {
		return false;
	}
}

/**
 * Le montant d'une ligne qui **n'est pas au centime**, ou `null` — Story v017-3b
 * (#687, AC-12) : le miroir de `kesh_core::accounting::purchase_line_total`, qui
 * reste juge (`SUPPLIER_INVOICE_LINE_NOT_IN_CENTIMES`).
 *
 * `quantité × prix` en `big.js` (jamais `parseFloat`) ; la quantité vide se lit `1`,
 * comme l'envoi ; la virgule décimale, qu'`AMOUNT_RE` admet, se lit comme un point.
 * Une saisie vide ou illisible ne refuse rien (`try/catch`, comme
 * {@link amountMismatch}) : c'est {@link linesInvalid} qui la juge. Le montant rendu
 * l'est par `toFixed()`, jamais en notation exponentielle.
 *
 * ⚠️ La virgule n'est pas normalisée **à l'envoi** : défaut préexistant, #830.
 */
export function lineNotInCentimes(line: CreateSupplierInvoiceLineRequest): string | null {
	const decimal = (raw: string) => raw.trim().replace(',', '.');
	if (!line.unitPrice.trim()) return null;
	try {
		const total = new Big(decimal(line.quantity || '1')).times(decimal(line.unitPrice));
		return total.round(2, Big.roundDown).eq(total) ? null : total.toFixed();
	} catch {
		return null;
	}
}

/** La première ligne qui n'est pas au centime (rang 0-based) et son montant, ou `null`. */
export function firstLineNotInCentimes(
	lines: CreateSupplierInvoiceLineRequest[],
): { index: number; lineTotal: string } | null {
	for (const [index, line] of lines.entries()) {
		const lineTotal = lineNotInCentimes(line);
		if (lineTotal !== null) return { index, lineTotal };
	}
	return null;
}

/** Lignes **structurellement** invalides (description, compte, quantité, prix). */
export function linesInvalid(lines: CreateSupplierInvoiceLineRequest[]): boolean {
	if (lines.length === 0) return true;
	return lines.some(
		(l) =>
			!l.description.trim() ||
			!l.expenseAccountId ||
			!isValidAmount(l.quantity) ||
			!isValidAmount(l.unitPrice),
	);
}

/** Les coordonnées de paiement saisies à la main (document sans QR, ou sans document). */
export interface PaymentInput {
	iban: string;
	reference: string;
	expectedAmount: string;
}

/**
 * Les champs de paiement du corps, depuis la saisie (AC12) : l'IBAN, normalisé, est
 * **routé** vers `creditorQrIban` s'il a la forme d'un QR-IBAN
 * ({@link qrIbanHasValidQrIid}), vers `creditorIban` sinon ; une saisie vide vaut
 * `null`. La validation est celle du serveur (`INVALID_PAYMENT_DETAILS`).
 */
export function paymentFields(
	input: PaymentInput,
): Pick<
	CreateSupplierInvoiceRequest,
	'creditorIban' | 'creditorQrIban' | 'paymentReference' | 'expectedPaymentAmount'
> {
	const iban = normalizeIban(input.iban);
	const isQr = iban !== '' && qrIbanHasValidQrIid(iban);
	return {
		creditorIban: iban !== '' && !isQr ? iban : null,
		creditorQrIban: isQr ? iban : null,
		paymentReference: input.reference.trim() || null,
		expectedPaymentAmount: input.expectedAmount.trim() || null,
	};
}

/** Le libellé d'un motif de refus `INVALID_PAYMENT_DETAILS` (AC12-bis, sept motifs). */
export function paymentReasonLabel(reason: string): string {
	switch (reason) {
		case 'INVALID_IBAN':
			return i18nMsg(
				'supplier-invoices-payment-error-invalid-iban',
				'IBAN invalide : vérifiez le code pays, la longueur et les chiffres de contrôle.',
			);
		case 'QR_IBAN_IN_IBAN_FIELD':
			return i18nMsg(
				'supplier-invoices-payment-error-qr-iban-in-iban-field',
				'Cet IBAN est un QR-IBAN : il s’emploie avec une référence QRR.',
			);
		case 'NOT_A_QR_IBAN':
			return i18nMsg('supplier-invoices-payment-error-not-a-qr-iban', 'Cet IBAN n’est pas un QR-IBAN.');
		case 'BOTH_IBANS':
			return i18nMsg(
				'supplier-invoices-payment-error-both-ibans',
				'Indiquez un IBAN ou un QR-IBAN, pas les deux.',
			);
		case 'QRR_REQUIRED':
			return i18nMsg(
				'supplier-invoices-payment-error-qrr-required',
				'Un QR-IBAN exige une référence QRR (27 chiffres).',
			);
		case 'INVALID_QRR':
			return i18nMsg(
				'supplier-invoices-payment-error-invalid-qrr',
				'Référence QRR invalide : 27 chiffres, dont un chiffre de contrôle.',
			);
		case 'REFERENCE_TOO_LONG':
			return i18nMsg(
				'supplier-invoices-payment-error-reference-too-long',
				'Référence trop longue : 64 caractères au plus.',
			);
		default:
			return i18nMsg('supplier-invoices-payment-error-generic', 'Coordonnées de paiement invalides.');
	}
}

/**
 * Où afficher un refus : sous l'IBAN, sous la référence, sous une **ligne** (Story
 * v017-3b, #687 : `line`, rang 1-based du serveur), ou en tête du formulaire.
 */
export interface SubmitError {
	field: 'iban' | 'reference' | 'line' | null;
	/** Le rang 1-based de la ligne désignée, quand `field === 'line'`. */
	line?: number;
	message: string;
}

/**
 * Traduit le refus d'un enregistrement (AC16). `INVALID_PAYMENT_DETAILS` va **sous le
 * champ** nommé par `details.field` ; les autres refus en tête du formulaire. Dans
 * tous les cas, la saisie reste intacte : c'est l'appelant qui ne la vide pas.
 */
export function describeSubmitError(err: unknown): SubmitError {
	if (!isApiError(err)) {
		return {
			field: null,
			message: i18nMsg('supplier-invoices-err-generic', 'Erreur inattendue.'),
		};
	}
	switch (err.code) {
		case 'INVALID_PAYMENT_DETAILS': {
			const d = err.details as { field?: string; reason?: string } | undefined;
			const field = d?.field === 'paymentReference' ? 'reference' : 'iban';
			return { field, message: paymentReasonLabel(d?.reason ?? '') };
		}
		case 'SUPPLIER_INVOICE_LINE_NOT_IN_CENTIMES': {
			// Story v017-3b (#687) — la clé et le texte du repli d'`errors.rs` : un texte
			// pour un refus (validation P3, F3).
			const d = err.details as { line?: number; lineTotal?: string } | undefined;
			return {
				field: 'line',
				line: d?.line,
				message: i18nMsg(
					'supplier-invoices-errors-line-not-in-centimes',
					'La ligne { $line } donne un montant de { $lineTotal }, qui n’est pas au centime. Saisissez la ligne au montant facturé par le fournisseur, au centime.',
					{ line: d?.line ?? '?', lineTotal: d?.lineTotal ?? '?' },
				),
			};
		}
		case 'QR_FIELDS_FROM_DOCUMENT':
			return {
				field: null,
				message: i18nMsg(
					'supplier-invoices-err-qr-fields',
					'Les coordonnées de paiement viennent du QR-code du document : elles ne se saisissent pas.',
				),
			};
		case 'AMOUNT_MISMATCH': {
			const d = err.details as { actual?: string; expected?: string } | undefined;
			return {
				field: null,
				message: i18nMsg(
					'supplier-invoices-err-amount',
					'Le total des lignes ({$actual}) ne correspond pas au montant du QR ({$expected}).',
					{ actual: d?.actual ?? '?', expected: d?.expected ?? '?' },
				),
			};
		}
		case 'CURRENCY_NOT_SUPPORTED':
			return {
				field: null,
				message: i18nMsg(
					'supplier-invoices-err-currency',
					'Devise non supportée (CHF uniquement). Retirez le document de la liste, ou créez la facture sans document puis joignez-le.',
				),
			};
		case 'IBAN_REFERENCE_MISMATCH':
			return {
				field: null,
				message: i18nMsg(
					'supplier-invoices-err-iban-ref',
					'Incohérence entre l’IBAN et la référence QRR du QR-code.',
				),
			};
		case 'IMPORT_NOT_PENDING_COMPLETION':
			return {
				field: null,
				message: i18nMsg(
					'supplier-invoices-err-not-pending',
					'Ce document a déjà été traité ou retiré de la liste.',
				),
			};
		case 'IMPORTED_INVOICE_NOT_FOUND':
			return {
				field: null,
				message: i18nMsg('supplier-invoices-err-doc-not-found', 'Document introuvable.'),
			};
		case 'FISCAL_YEAR_INVALID':
			return {
				field: null,
				message: i18nMsg(
					'supplier-invoices-err-fiscal-year',
					'Aucun exercice ouvert ne couvre cette date.',
				),
			};
		default:
			return { field: null, message: err.message };
	}
}
