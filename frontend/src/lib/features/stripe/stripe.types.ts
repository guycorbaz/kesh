/**
 * Story 40-7d (AC-2, AC-3) — DTO de l'écran Stripe, miroir des routes
 * `POST /api/v1/stripe-imports` (40-7b), `GET /api/v1/stripe/review` et
 * `POST /api/v1/stripe/accept` (40-7c). Les formes communes aux prestataires
 * viennent de `provider/provider-review.types.ts`. Montants en chaînes décimales.
 */
import type {
	ProviderImportResponseBase,
	ProviderReview,
	ProviderReviewEntry,
	ProviderReviewTransaction,
} from '$lib/features/provider/provider-review.types';

/** Un versement lu dans le rapport (40-7b) : son identifiant, sa devise, ses lignes et son net. */
export interface StripePayoutReport {
	payoutId: string;
	currency: string;
	lines: number;
	net: string;
}

/** Réponse `201` de `POST /api/v1/stripe-imports`. */
export interface StripeImportResponse extends ProviderImportResponseBase {
	payouts: StripePayoutReport[];
}

/** Les champs d'une proposition Stripe, selon `type` (`StripeReviewProposal`, sérialisée à plat). */
export type StripeProposalFields =
	| { type: 'link_entry'; bankTransactionId: number; journalEntryId: number }
	| { type: 'fee'; bankTransactionId: number; feeAccountId: number }
	| {
			type: 'payout';
			bankTransactionId: number;
			/** Le compte bancaire de versement réglé. */
			payoutBankAccountId: number;
			/** Son libellé (banque, et description). */
			payoutAccountLabel: string;
	  };

/** Une proposition de la revue Stripe : ses champs, sa transaction, l'écriture d'un rattachement. */
export type StripeProposal = StripeProposalFields & {
	transaction: ProviderReviewTransaction;
	otherTransaction?: ProviderReviewTransaction;
	entry?: ProviderReviewEntry;
};

/** Réponse de `GET /api/v1/stripe/review`. */
export type StripeReview = ProviderReview<StripeProposal>;

/**
 * Une proposition envoyée à `POST /api/v1/stripe/accept` : ses seuls champs ;
 * le versement reprend le compte de versement que la revue a montré (une
 * destination changée depuis rend la proposition périmée).
 */
export type StripeAcceptProposal =
	| { type: 'link_entry'; bankTransactionId: number; journalEntryId: number }
	| { type: 'fee'; bankTransactionId: number; feeAccountId: number }
	| { type: 'payout'; bankTransactionId: number; payoutBankAccountId: number };
