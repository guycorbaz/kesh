/**
 * Story 38-11d (AC-3 à AC-5) — libellés traduits de l'écran PayPal : refus de
 * l'import (une phrase par cause), genres des propositions, écarts, motifs
 * « à examiner » et refus de l'acceptation.
 *
 * ⚠️ **Les clés sont écrites en toutes lettres** (jamais construites par
 * gabarit) : `i18n-keys.test.ts` les voit dès qu'elles manquent d'un catalogue.
 *
 * Les refus de l'acceptation propres au rapprochement des prestataires ont des
 * clés `paypal-failed-*` ; les autres sont **délégués** à
 * `failedProposalLabel` (rapprochement générique), qui n'est pas étendu (son
 * test compare ses codes aux littéraux de `reconciliation.rs`).
 */
import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
import { isApiError } from '$lib/shared/utils/api-error';
import { errorMessageOf } from '$lib/shared/utils/error-message';
import { failedProposalLabel } from '$lib/features/reconciliation/failed-proposal-label';
import type { PaypalFinding, PaypalLineError, PaypalProposal } from './paypal.types';

/**
 * Codes des refus que l'écran traduit lui-même (`paypal-failed-*`) ; tout autre
 * code passe par `failedProposalLabel`. Exportée pour le test, qui la compare à
 * une liste écrite en dur.
 */
export const PAYPAL_OWN_FAILED_CODES = [
	'PROVIDER_PROPOSAL_STALE',
	'FX_ACCOUNT_INVALID',
	'FX_RATE_INVALID',
	'NOT_FOUND',
	'OPTIMISTIC_LOCK_CONFLICT',
	'RECONCILIATION_ENTRY_NOT_LINKABLE',
] as const;

/** Libellé d'un refus de `POST /paypal/accept`. */
export function paypalFailedLabel(code: string, details?: unknown): string {
	switch (code) {
		case 'PROVIDER_PROPOSAL_STALE':
			return i18nMsg(
				'paypal-failed-proposal-stale',
				'La proposition ne vaut plus : la transaction, l’écriture, la règle ou les paramètres du compte ont changé depuis la revue. Rechargez la page.',
			);
		case 'FX_ACCOUNT_INVALID':
			return i18nMsg(
				'paypal-failed-fx-account-invalid',
				'Aucun compte de conversion utilisable n’est désigné : un administrateur doit en choisir un dans les paramètres du multi-devises.',
			);
		case 'FX_RATE_INVALID':
			return i18nMsg(
				'paypal-failed-fx-rate-invalid',
				'Le cours de la conversion ne se calcule pas à partir des deux montants : passez l’écriture à la main.',
			);
		case 'NOT_FOUND':
			return i18nMsg(
				'paypal-failed-not-found',
				'La transaction ou l’écriture visée est introuvable : elle a été supprimée entre-temps. Rechargez la page.',
			);
		case 'OPTIMISTIC_LOCK_CONFLICT':
			return i18nMsg(
				'paypal-failed-optimistic-lock',
				'La transaction a été modifiée entre-temps. Rechargez la page.',
			);
		case 'RECONCILIATION_ENTRY_NOT_LINKABLE':
			return i18nMsg(
				'paypal-failed-entry-not-linkable',
				'L’écriture de vente n’est plus rattachable à cette transaction (déjà rattachée, contre-passée ou modifiée). Rechargez la page.',
			);
		default:
			return failedProposalLabel(code, details);
	}
}

/** Libellé d'une erreur de ligne du lecteur PayPal (38-11a) ; code brut si inconnu. */
export function paypalLineErrorLabel(code: string): string {
	switch (code) {
		case 'FIELD_COUNT':
			return i18nMsg('paypal-csv-line-field-count', 'nombre de cellules différent de l’en-tête');
		case 'DATE':
			return i18nMsg('paypal-csv-line-date', 'date illisible ou d’un autre format que le fichier');
		case 'AMOUNT':
			return i18nMsg('paypal-csv-line-amount', 'montant illisible');
		case 'NET_MISMATCH':
			return i18nMsg('paypal-csv-line-net-mismatch', 'le net ne vaut pas le brut plus les frais');
		case 'SIGN_MISMATCH':
			return i18nMsg('paypal-csv-line-sign-mismatch', 'signe du brut contraire à Balance Impact');
		case 'CURRENCY':
			return i18nMsg('paypal-csv-line-currency', 'devise illisible (trois lettres attendues)');
		case 'TRANSACTION_ID':
			return i18nMsg('paypal-csv-line-transaction-id', 'identifiant de transaction vide ou illisible');
		case 'BALANCE_IMPACT':
			return i18nMsg('paypal-csv-line-balance-impact', 'Balance Impact autre que Credit, Debit ou Memo');
		case 'REQUIRED':
			return i18nMsg('paypal-csv-line-required', 'cellule obligatoire vide');
		default:
			return code;
	}
}

function linesOf(details: Record<string, unknown> | undefined): PaypalLineError[] {
	const lines = details?.lines;
	return Array.isArray(lines) ? (lines as PaypalLineError[]) : [];
}

function columnsOf(details: Record<string, unknown> | undefined): string[] {
	const columns = details?.columns;
	return Array.isArray(columns) ? columns.map(String) : [];
}

/**
 * Les phrases d'un refus de l'import : pour `PAYPAL_CSV_INVALID`, une phrase par
 * cause (colonnes manquantes nommées, une ligne par erreur de ligne, remède de
 * l'ordre des dates) ; pour tout autre refus, le message du serveur
 * (`PROVIDER_CURRENCY_ACCOUNT_MISSING` nomme les devises).
 */
export function paypalImportErrorLines(err: unknown): string[] {
	if (!isApiError(err) || err.code !== 'PAYPAL_CSV_INVALID') return [errorMessageOf(err)];
	const details = err.details;
	const reason = typeof details?.reason === 'string' ? details.reason : '';
	switch (reason) {
		case 'PAYPAL_CSV_MISSING_COLUMNS':
			return [
				i18nMsg('paypal-import-error-missing-columns', 'Colonnes manquantes : { $columns }. Exportez le rapport « Balance affecting transactions » en CSV, colonnes en anglais.', {
					columns: columnsOf(details).join(', '),
				}),
			];
		case 'PAYPAL_CSV_DUPLICATE_COLUMN':
			return [
				i18nMsg('paypal-import-error-duplicate-column', 'Colonne en double dans l’en-tête : { $columns }.', {
					columns: columnsOf(details).join(', '),
				}),
			];
		case 'PAYPAL_CSV_DATE_ORDER_AMBIGUOUS':
			return [
				i18nMsg(
					'paypal-import-error-date-order-ambiguous',
					'L’ordre du jour et du mois des dates ne se décide pas : aucun jour ne dépasse 12. Exportez une période plus longue, ou changez le format de date dans PayPal.',
				),
			];
		case 'PAYPAL_CSV_DATE_ORDER_CONFLICT':
			return [
				i18nMsg(
					'paypal-import-error-date-order-conflict',
					'Les dates du fichier se contredisent : certaines ne se lisent que jour en tête, d’autres que mois en tête.',
				),
			];
		case 'PAYPAL_CSV_EMPTY':
			return [i18nMsg('paypal-import-error-empty', 'Le fichier est vide : aucune ligne de données.')];
		case 'PAYPAL_CSV_ENCODING':
			return [i18nMsg('paypal-import-error-encoding', 'L’encodage du fichier n’est pas reconnu.')];
		case 'PAYPAL_CSV_LINES':
			return [
				i18nMsg('paypal-import-error-lines', 'Lignes en erreur, rien n’a été importé :'),
				...linesOf(details).map((l) =>
					i18nMsg('paypal-import-error-line', 'Ligne { $line }, { $field } : { $cause }', {
						line: String(l.line),
						field: l.field ?? '—',
						cause: paypalLineErrorLabel(l.code),
					}),
				),
			];
		default:
			return [errorMessageOf(err)];
	}
}

/**
 * Genre d'une proposition (colonne « Genre ») ; `ruleLabels` : le libellé de
 * chaque règle par identifiant (la revue ne sert que l'identifiant).
 */
export function proposalKindLabel(
	p: PaypalProposal,
	ruleLabels: ReadonlyMap<number, string> = new Map(),
): string {
	switch (p.type) {
		case 'link_entry':
			return i18nMsg('paypal-kind-link-entry', 'Vente rattachée');
		case 'fee':
			return i18nMsg('paypal-kind-fee', 'Frais');
		case 'rule':
			return i18nMsg('paypal-kind-rule', 'Règle { $label }', { label: ruleLabels.get(p.ruleId) ?? `#${p.ruleId}` });
		case 'conversion':
			return i18nMsg('paypal-kind-conversion', 'Conversion');
		case 'withdrawal':
			return i18nMsg('paypal-kind-withdrawal', 'Retrait');
	}
}

/** Libellé d'un genre d'écart. */
export function gapKindLabel(kind: string): string {
	switch (kind) {
		case 'receipt_without_sale':
			return i18nMsg('paypal-gap-receipt-without-sale', 'Encaissement sans vente');
		case 'sale_without_receipt':
			return i18nMsg('paypal-gap-sale-without-receipt', 'Vente sans encaissement');
		case 'amount_or_currency_mismatch':
			return i18nMsg('paypal-gap-amount-or-currency-mismatch', 'Montant ou devise différents');
		default:
			return kind;
	}
}

/**
 * Story 38-11d (revue P1, M3) — l'écriture d'un écart « montant ou devise différents » est-elle
 * déjà rattachée à une autre transaction (`details.alreadyLinked`) ? Le serveur range ce cas
 * sous le même code ; l'écran lui donne son propre libellé.
 */
function isAlreadyLinked(f: PaypalFinding): boolean {
	return f.code === 'amount_or_currency_mismatch' && f.details?.alreadyLinked === true;
}

/** Libellé d'un écart : son genre, ou « vente déjà rattachée » pour une écriture déjà rattachée. */
export function findingGapLabel(f: PaypalFinding): string {
	if (isAlreadyLinked(f)) {
		return i18nMsg('paypal-gap-already-linked', 'Vente déjà rattachée à une autre transaction');
	}
	return gapKindLabel(f.code);
}

/**
 * Ce qu'un écart « montant ou devise différents » attend et trouve ; `null` pour un autre écart,
 * des détails absents, ou une écriture déjà rattachée de même montant et devise (le libellé dit
 * tout). Écriture sans débit au compte : « aucun débit trouvé », jamais « trouvé » suivi de rien.
 */
export function findingGapDetails(f: PaypalFinding): string | null {
	const d = f.details;
	if (!d || f.code !== 'amount_or_currency_mismatch') return null;
	const s = (k: string) => (d[k] === null || d[k] === undefined ? '' : String(d[k]));
	const expected = `${s('expectedAmount')} ${s('expectedCurrency')}`.trim();
	if (s('foundAmount') === '') {
		return i18nMsg('paypal-gap-mismatch-details-none', 'Attendu { $expected }, aucun débit trouvé dans l’écriture', {
			expected,
		});
	}
	const found = `${s('foundAmount')} ${s('foundCurrency')}`.trim();
	if (isAlreadyLinked(f) && found === expected) return null;
	return i18nMsg('paypal-gap-mismatch-details', 'Attendu { $expected }, trouvé { $found }', { expected, found });
}

/** Libellé d'un motif « à examiner ». */
export function toReviewReasonLabel(reason: string): string {
	switch (reason) {
		case 'ambiguous_sale':
			return i18nMsg('paypal-to-review-ambiguous-sale', 'Plusieurs ventes portent cette TXN');
		case 'fee_account_missing':
			return i18nMsg('paypal-to-review-fee-account-missing', 'Compte de frais non réglé');
		case 'conversion_unpaired':
			return i18nMsg('paypal-to-review-conversion-unpaired', 'Conversion sans sa paire');
		case 'withdrawal_foreign_currency':
			return i18nMsg(
				'paypal-to-review-withdrawal-foreign-currency',
				'Retrait d’un solde en devise : écriture à la main',
			);
		case 'no_rule':
			return i18nMsg('paypal-to-review-no-rule', 'Aucune règle');
		default:
			return reason;
	}
}
