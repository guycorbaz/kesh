/**
 * Story 38-11d (AC-3, AC-4) — client des routes PayPal : import du CSV
 * (38-11b), revue et acceptation (38-11c2). Story 40-7d : enveloppe du client
 * commun aux prestataires (`provider/provider-review.api.ts`).
 */
import {
	PROVIDER_ACCEPT_CHUNK,
	acceptProviderProposals,
	chunkProposals,
	getProviderReview,
	importProviderFile,
} from '$lib/features/provider/provider-review.api';
import type {
	PaypalAcceptProposal,
	PaypalAcceptResponse,
	PaypalImportResponse,
	PaypalProposal,
	PaypalReview,
} from './paypal.types';

/**
 * Taille maximale d'un paquet d'acceptation : le plafond du serveur
 * (`MAX_IN_CLAUSE_IDS`, 500 propositions ; un corps plus long est refusé en
 * `400`), commun aux prestataires.
 */
export const PAYPAL_ACCEPT_CHUNK = PROVIDER_ACCEPT_CHUNK;

export { chunkProposals };

/**
 * `POST /api/v1/paypal-imports` (multipart : `file`, `account`) — importe le CSV
 * « Balance affecting transactions » du compte PayPal `account`, ventilé par
 * devise. Refus (`PROVIDER_ACCOUNT_NOT_FOUND`, `PAYPAL_CSV_INVALID`,
 * `PROVIDER_CURRENCY_ACCOUNT_MISSING`, …) : `ApiError`, à traduire par
 * `paypalImportErrorLines` (`paypal-labels.ts`).
 */
export async function importPaypalCsv(file: File, account: string): Promise<PaypalImportResponse> {
	return importProviderFile<PaypalImportResponse>('paypal', file, account);
}

/** `GET /api/v1/paypal/review` — revue d'un compte PayPal sur un exercice (lecture seule). */
export async function getPaypalReview(account: string, fiscalYearId: number): Promise<PaypalReview> {
	return getProviderReview<PaypalProposal>('paypal', account, fiscalYearId);
}

/** `POST /api/v1/paypal/accept` — un paquet de propositions (500 au plus). */
export async function acceptPaypalProposals(
	account: string,
	proposals: PaypalAcceptProposal[],
): Promise<PaypalAcceptResponse> {
	return acceptProviderProposals('paypal', account, proposals);
}
