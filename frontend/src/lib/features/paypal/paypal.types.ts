/**
 * Story 38-11d (AC-3 à AC-5) — DTO de l'écran PayPal, miroir des routes
 * `POST /api/v1/paypal-imports` (38-11b), `GET /api/v1/paypal/review` et
 * `POST /api/v1/paypal/accept` (38-11c2). Montants en chaînes décimales.
 * Story 40-7d : les formes communes aux prestataires vivent dans
 * `provider/provider-review.types.ts` ; les noms PayPal en sont des alias.
 */
import type {
	ProviderAcceptResponse,
	ProviderCurrencyReport,
	ProviderFailedProposal,
	ProviderFinding,
	ProviderImportResponseBase,
	ProviderLineError,
	ProviderReview,
	ProviderReviewEntry,
	ProviderReviewTransaction,
} from '$lib/features/provider/provider-review.types';

// ---------------------------------------------------------------------------
// Import (38-11b, `routes/paypal_imports.rs`)
// ---------------------------------------------------------------------------

/** Une ligne écartée parce que son statut n'est pas `Completed`. */
export interface PaypalNotCompletedLine {
	transactionId: string;
	type: string;
	status: string;
}

/** Le compte rendu d'une devise de l'import (commun aux prestataires, 40-7d). */
export type PaypalCurrencyReport = ProviderCurrencyReport;

/** Réponse `201` de `POST /api/v1/paypal-imports`. */
export interface PaypalImportResponse extends ProviderImportResponseBase {
	skipped: {
		notCompleted: PaypalNotCompletedLine[];
		memo: number;
		neutralPairs: number;
	};
}

/** Une erreur de ligne du lecteur PayPal (`details.lines[]` de `PAYPAL_CSV_INVALID`). */
export type PaypalLineError = ProviderLineError;

// ---------------------------------------------------------------------------
// Revue et acceptation (38-11c2, AC-5 à AC-9 ; `kesh-reconciliation::provider_review`)
// ---------------------------------------------------------------------------

/** Ce que la revue montre d'une transaction (commun aux prestataires, 40-7d). */
export type PaypalReviewTransaction = ProviderReviewTransaction;

/** Une écriture que nomme la revue (commun aux prestataires, 40-7d). */
export type PaypalReviewEntry = ProviderReviewEntry;

/** Les champs d'une proposition, selon `type` (`Proposal`, sérialisé aplati). */
export type PaypalProposalFields =
	| { type: 'link_entry'; bankTransactionId: number; journalEntryId: number }
	| { type: 'fee'; bankTransactionId: number; feeAccountId: number }
	| { type: 'rule'; bankTransactionId: number; ruleId: number; counterpartyAccountId: number }
	| { type: 'conversion'; debitTransactionId: number; creditTransactionId: number }
	| {
			type: 'withdrawal';
			bankTransactionId: number;
			/** Préremplie par le compte du plan du compte bancaire principal ; `null` = à choisir. */
			counterpartyAccountId: number | null;
	  };

/** Une proposition de la revue : ses champs, la transaction (la débitée d'une conversion), l'autre moitié d'une conversion, l'écriture d'un rattachement. */
export type PaypalProposal = PaypalProposalFields & {
	transaction: PaypalReviewTransaction;
	otherTransaction?: PaypalReviewTransaction;
	entry?: PaypalReviewEntry;
};

export type PaypalProposalType = PaypalProposal['type'];

/** Un écart ou une ligne « à examiner » (commun aux prestataires, 40-7d). */
export type PaypalFinding = ProviderFinding;

/** Réponse de `GET /api/v1/paypal/review`. */
export type PaypalReview = ProviderReview<PaypalProposal>;

/** Une proposition envoyée à `POST /api/v1/paypal/accept` (ses seuls champs ; `withdrawal` avec son compte). */
export type PaypalAcceptProposal =
	| Exclude<PaypalProposalFields, { type: 'withdrawal' }>
	| { type: 'withdrawal'; bankTransactionId: number; counterpartyAccountId: number };

/** Un refus par proposition (*Pattern batch*, commun aux prestataires, 40-7d). */
export type PaypalFailedProposal = ProviderFailedProposal;

/** Réponse de `POST /api/v1/paypal/accept`. */
export type PaypalAcceptResponse = ProviderAcceptResponse;
