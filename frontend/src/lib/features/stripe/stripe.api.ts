/**
 * Story 40-7d (AC-2, AC-3) — client des routes Stripe : import du rapport
 * (40-7b), revue et acceptation (40-7c), sur le client commun aux prestataires.
 */
import {
	acceptProviderProposals,
	getProviderReview,
	importProviderFile,
} from '$lib/features/provider/provider-review.api';
import type { ProviderAcceptResponse } from '$lib/features/provider/provider-review.types';
import type { StripeAcceptProposal, StripeImportResponse, StripeProposal, StripeReview } from './stripe.types';

/**
 * `POST /api/v1/stripe-imports` (multipart : `file`, `account`) — importe le
 * rapport « Itemized payout reconciliation » du compte Stripe `account`,
 * ventilé par devise. Refus (`STRIPE_CSV_INVALID`, …) : `ApiError`, à traduire
 * par `stripeImportErrorLines` (`stripe-labels.ts`).
 */
export async function importStripeReport(file: File, account: string): Promise<StripeImportResponse> {
	return importProviderFile<StripeImportResponse>('stripe', file, account);
}

/** `GET /api/v1/stripe/review` — revue d'un compte Stripe sur un exercice (lecture seule). */
export async function getStripeReview(account: string, fiscalYearId: number): Promise<StripeReview> {
	return getProviderReview<StripeProposal>('stripe', account, fiscalYearId);
}

/** `POST /api/v1/stripe/accept` — un paquet de propositions (500 au plus). */
export async function acceptStripeProposals(
	account: string,
	proposals: StripeAcceptProposal[],
): Promise<ProviderAcceptResponse> {
	return acceptProviderProposals('stripe', account, proposals);
}
