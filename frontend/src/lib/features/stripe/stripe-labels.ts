/**
 * Story 40-7d (AC-2, AC-3) — libellés traduits de l'écran Stripe : refus de
 * l'import (une phrase par cause), genres et actions des propositions, écarts
 * et motifs « à examiner », refus de l'acceptation.
 *
 * ⚠️ **Les clés sont écrites en toutes lettres** (jamais construites par
 * gabarit) : `i18n-keys.test.ts` les voit dès qu'elles manquent d'un catalogue.
 * Ce qui est commun aux prestataires vient de `provider/provider-labels.ts`
 * (refus d'une proposition périmée, motifs `ambiguous_sale` et
 * `fee_account_missing`, détails d'un montant différent) ; les autres refus de
 * l'acceptation sont délégués à `failedProposalLabel`, qui n'est pas étendu.
 */
import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
import { isApiError } from '$lib/shared/utils/api-error';
import { errorMessageOf } from '$lib/shared/utils/error-message';
import type { AccountResponse } from '$lib/features/accounts/accounts.types';
import {
	accountNumberName,
	isAlreadyLinked,
	providerFailedLabel,
	providerMismatchDetails,
	providerToReviewLabel,
} from '$lib/features/provider/provider-labels';
import type { ProviderFinding, ProviderLineError } from '$lib/features/provider/provider-review.types';
import type { StripeProposal } from './stripe.types';

/**
 * Les colonnes de l'objet lié que le lecteur exige (l'une ou l'autre), telles
 * que `details.columns` les nomme (`LINKED_OBJECT_COLUMNS`, 40-7a).
 */
export const STRIPE_LINKED_OBJECT_COLUMNS = 'source_id|charge_id';

/** La colonne de l'identifiant du versement (`COL_PAYOUT_ID`, 40-7a). */
export const STRIPE_PAYOUT_ID_COLUMN = 'automatic_payout_id';

/** Libellé d'une erreur de ligne du lecteur Stripe (40-7a) ; code brut si inconnu. */
export function stripeLineErrorLabel(l: Pick<ProviderLineError, 'code' | 'field' | 'value'>): string {
	switch (l.code) {
		case 'FIELD_COUNT':
			return i18nMsg('stripe-csv-line-field-count', 'nombre de cellules différent de l’en-tête');
		case 'ID':
			// Un versement sans identifiant : le rapport n'est pas celui des versements automatiques.
			if (l.field === STRIPE_PAYOUT_ID_COLUMN && (l.value ?? '') === '') {
				return i18nMsg(
					'stripe-csv-line-payout-id-missing',
					'versement sans identifiant : exportez le rapport des versements automatiques (Rapprochement des versements)',
				);
			}
			return i18nMsg('stripe-csv-line-id', 'identifiant ou catégorie vide ou illisible');
		case 'DATE':
			return i18nMsg('stripe-csv-line-date', 'date illisible ou hors format');
		case 'CURRENCY':
			return i18nMsg('stripe-csv-line-currency', 'devise illisible (trois lettres attendues)');
		case 'AMOUNT':
			return i18nMsg('stripe-csv-line-amount', 'montant illisible (point décimal attendu)');
		case 'NET_MISMATCH':
			return i18nMsg('stripe-csv-line-net-mismatch', 'le net ne vaut pas le brut moins les frais');
		case 'PAYOUT_CURRENCY_MIXED':
			return i18nMsg('stripe-csv-line-payout-currency-mixed', 'un même versement porte deux devises');
		default:
			return l.code;
	}
}

function linesOf(details: Record<string, unknown> | undefined): ProviderLineError[] {
	const lines = details?.lines;
	return Array.isArray(lines) ? (lines as ProviderLineError[]) : [];
}

function columnsOf(details: Record<string, unknown> | undefined): string[] {
	const columns = details?.columns;
	return Array.isArray(columns) ? columns.map(String) : [];
}

/**
 * Les phrases d'un refus de l'import : pour `STRIPE_CSV_INVALID`, une phrase par
 * cause (colonnes manquantes nommées, et pour l'objet lié absent la marche à
 * suivre dans Stripe ; une ligne par erreur de ligne) ; pour tout autre refus,
 * le message du serveur (`PROVIDER_CURRENCY_ACCOUNT_MISSING` nomme les devises).
 */
export function stripeImportErrorLines(err: unknown): string[] {
	if (!isApiError(err) || err.code !== 'STRIPE_CSV_INVALID') return [errorMessageOf(err)];
	const details = err.details;
	const reason = typeof details?.reason === 'string' ? details.reason : '';
	switch (reason) {
		case 'STRIPE_CSV_MISSING_COLUMNS': {
			const columns = columnsOf(details);
			const others = columns.filter((c) => c !== STRIPE_LINKED_OBJECT_COLUMNS);
			const out: string[] = [];
			if (others.length > 0) {
				out.push(
					i18nMsg(
						'stripe-import-error-missing-columns',
						'Colonnes manquantes : { $columns }. Exportez le rapport « Rapprochement des versements → Détaillé » de Stripe en CSV.',
						{ columns: others.join(', ') },
					),
				);
			}
			if (columns.includes(STRIPE_LINKED_OBJECT_COLUMNS)) {
				out.push(
					i18nMsg(
						'stripe-import-error-linked-object',
						'La colonne source_id manque : Kesh en a besoin pour retrouver la vente de chaque charge. Dans Stripe, Rapports → Rapprochement des versements → Détaillé, ajoutez la colonne source_id (ou charge_id), puis exportez de nouveau.',
					),
				);
			}
			return out.length > 0 ? out : [errorMessageOf(err)];
		}
		case 'STRIPE_CSV_DUPLICATE_COLUMN':
			return [
				i18nMsg('stripe-import-error-duplicate-column', 'Colonne en double dans l’en-tête : { $columns }.', {
					columns: columnsOf(details).join(', '),
				}),
			];
		case 'STRIPE_CSV_EMPTY':
			return [i18nMsg('stripe-import-error-empty', 'Le fichier est vide : aucune ligne de données.')];
		case 'STRIPE_CSV_ENCODING':
			return [i18nMsg('stripe-import-error-encoding', 'L’encodage du fichier n’est pas reconnu.')];
		case 'STRIPE_CSV_LINES':
			return [
				i18nMsg('stripe-import-error-lines', 'Lignes en erreur, rien n’a été importé :'),
				...linesOf(details).map((l) =>
					i18nMsg('stripe-import-error-line', 'Ligne { $line }, { $field } : { $cause }', {
						line: String(l.line),
						field: l.field ?? '—',
						cause: stripeLineErrorLabel(l),
					}),
				),
			];
		default:
			return [errorMessageOf(err)];
	}
}

/** Genre d'une proposition (colonne « Genre »). */
export function stripeProposalKindLabel(p: StripeProposal): string {
	switch (p.type) {
		case 'link_entry':
			return i18nMsg('stripe-kind-link-entry', 'Rattachement à la vente n° { $number }', {
				number: String(p.entry?.entryNumber ?? p.journalEntryId),
			});
		case 'fee':
			return i18nMsg('stripe-kind-fee', 'Frais');
		case 'payout':
			return i18nMsg('stripe-kind-payout', 'Versement vers { $account }', { account: p.payoutAccountLabel });
	}
}

/** Le nom « numéro — nom » d'un compte (règle commune aux prestataires, 39-11d2). */
const accountName = accountNumberName;

/** Ce que fera une proposition (colonne « Ce qui sera fait »). */
export function stripeProposalActionLabel(p: StripeProposal, accounts: readonly AccountResponse[]): string {
	switch (p.type) {
		case 'link_entry':
			return i18nMsg('stripe-action-link-entry', 'Rattachée à l’écriture de vente, sans nouvelle écriture');
		case 'fee':
			return i18nMsg('stripe-action-fee', 'Écriture au compte de frais { $account }', {
				account: accountName(accounts, p.feeAccountId),
			});
		case 'payout':
			return i18nMsg('stripe-action-payout', 'Écriture du versement vers { $account }', {
				account: p.payoutAccountLabel,
			});
	}
}

/**
 * Les codes d'écart de la revue Stripe : ceux de la revue commune (38-11c) et
 * ceux de Stripe (40-7c). Exportée pour le test, qui la compare aux constantes
 * `GAP_*` des sources Rust.
 */
export const STRIPE_GAP_CODES = [
	'receipt_without_sale',
	'sale_without_receipt',
	'amount_or_currency_mismatch',
	'payout_total_mismatch',
	'payout_without_receipt',
] as const;

/**
 * Les motifs « à examiner » de la revue Stripe (communs servis par Stripe, et
 * propres à Stripe). Exportée pour le test, comparée aux constantes `REVIEW_*`.
 */
export const STRIPE_TO_REVIEW_CODES = [
	'ambiguous_sale',
	'fee_account_missing',
	'payout_account_missing',
	'payout_reversal',
	'payout_not_negative',
	'refund',
	'other_category',
] as const;

/** Une valeur de `details`, en texte (`''` si absente). */
function detail(f: ProviderFinding, key: string): string {
	const v = f.details?.[key];
	return v === null || v === undefined ? '' : String(v);
}

/** Libellé d'un écart de la revue Stripe ; code brut si inconnu. */
export function stripeGapLabel(f: ProviderFinding): string {
	switch (f.code) {
		case 'receipt_without_sale':
			return i18nMsg('stripe-gap-receipt-without-sale', 'Charge sans vente');
		case 'sale_without_receipt':
			return i18nMsg('stripe-gap-sale-without-receipt', 'Vente sans charge');
		case 'amount_or_currency_mismatch':
			return isAlreadyLinked(f)
				? i18nMsg('stripe-gap-already-linked', 'Vente déjà rattachée à une autre charge')
				: i18nMsg('stripe-gap-amount-or-currency-mismatch', 'Montant ou devise différents');
		case 'payout_total_mismatch':
			return i18nMsg(
				'stripe-gap-payout-total-mismatch',
				'Versement incomplet : ses lignes importées font { $found }, pour un versement de { $expected }. Importez le rapport complet du versement.',
				{ found: detail(f, 'found'), expected: detail(f, 'expected') },
			);
		case 'payout_without_receipt':
			return i18nMsg(
				'stripe-gap-payout-without-receipt',
				"Versement sans réception : aucune transaction de la banque ou de Wise n'est rattachée à son écriture. Importez le relevé de réception, puis rattachez-la.",
			);
		default:
			return f.code;
	}
}

/** Détails d'un écart : attendu et trouvé d'un montant ou d'une devise différents ; `null` sinon. */
export function stripeGapDetails(f: ProviderFinding): string | null {
	return providerMismatchDetails(f);
}

/** Libellé d'un motif « à examiner » de la revue Stripe ; code brut si inconnu. */
export function stripeToReviewLabel(f: ProviderFinding): string {
	const common = providerToReviewLabel(f.code);
	if (common !== null) return common;
	switch (f.code) {
		case 'refund':
			return i18nMsg(
				'stripe-review-refund',
				"Remboursement : passez l'écriture à la main (en devise), puis rattachez la transaction.",
			);
		case 'other_category':
			return i18nMsg(
				'stripe-review-other-category',
				"Catégorie Stripe à examiner ({ $category }) : passez l'écriture à la main, puis rattachez la transaction.",
				{ category: detail(f, 'category') },
			);
		case 'payout_reversal':
			return i18nMsg(
				'stripe-review-payout-reversal',
				"Versement qui porte un retour de versement : passez l'écriture à la main, puis rattachez la transaction.",
			);
		case 'payout_not_negative':
			return i18nMsg(
				'stripe-review-payout-not-negative',
				"Versement de montant positif ou nul : passez l'écriture à la main, puis rattachez la transaction.",
			);
		case 'payout_account_missing':
			return i18nMsg(
				'stripe-review-payout-account-missing',
				'Versement sans compte de versement utilisable : réglez le compte de versement du compte Stripe (ni archivé, ni clos, ni un compte Stripe ou PayPal, lié à un compte du plan dans la devise du solde).',
			);
		default:
			return f.code;
	}
}

/**
 * Libellé d'un refus de `POST /stripe/accept`. `STRIPE_PAYOUT_ACCOUNT_INVALID` : le compte de
 * versement réglé, rejugé à l'acceptation, est devenu un compte Stripe ou PayPal, ou d'une autre
 * devise que le solde (même règle et même texte qu'au réglage du compte Stripe).
 */
export function stripeFailedLabel(code: string, details?: unknown): string {
	if (code === 'STRIPE_PAYOUT_ACCOUNT_INVALID') {
		return i18nMsg(
			'error-stripe-payout-account-invalid',
			'Le compte de versement doit être un compte bancaire de la société, ouvert, sans prestataire Stripe ni PayPal, dans la même devise que le solde Stripe.',
		);
	}
	return providerFailedLabel(code, details);
}
