/**
 * Story 38-11d (AC-3, AC-4) — client des routes PayPal : import du CSV
 * (38-11b), revue et acceptation (38-11c2).
 */
import { apiClient } from '$lib/shared/utils/api-client';
import type {
	PaypalAcceptProposal,
	PaypalAcceptResponse,
	PaypalImportResponse,
	PaypalReview,
} from './paypal.types';

/**
 * Taille maximale d'un paquet d'acceptation : le plafond du serveur
 * (`MAX_IN_CLAUSE_IDS`, 500 propositions ; un corps plus long est refusé en
 * `400`).
 */
export const PAYPAL_ACCEPT_CHUNK = 500;

/**
 * `POST /api/v1/paypal-imports` (multipart : `file`, `account`) — importe le CSV
 * « Balance affecting transactions » du compte PayPal `account`, ventilé par
 * devise. Refus (`PROVIDER_ACCOUNT_NOT_FOUND`, `PAYPAL_CSV_INVALID`,
 * `PROVIDER_CURRENCY_ACCOUNT_MISSING`, …) : `ApiError`, à traduire par
 * `paypalImportErrorLines` (`paypal-labels.ts`).
 */
export async function importPaypalCsv(file: File, account: string): Promise<PaypalImportResponse> {
	const form = new FormData();
	form.append('file', file);
	form.append('account', account);
	return apiClient.postFormData<PaypalImportResponse>('/api/v1/paypal-imports', form);
}

/** `GET /api/v1/paypal/review` — revue d'un compte PayPal sur un exercice (lecture seule). */
export async function getPaypalReview(account: string, fiscalYearId: number): Promise<PaypalReview> {
	const params = new URLSearchParams({ account, fiscalYearId: String(fiscalYearId) });
	return apiClient.get<PaypalReview>(`/api/v1/paypal/review?${params.toString()}`);
}

/** `POST /api/v1/paypal/accept` — un paquet de propositions (500 au plus). */
export async function acceptPaypalProposals(
	account: string,
	proposals: PaypalAcceptProposal[],
): Promise<PaypalAcceptResponse> {
	return apiClient.post<PaypalAcceptResponse>('/api/v1/paypal/accept', { account, proposals });
}

/**
 * Découpe une sélection en paquets de {@link PAYPAL_ACCEPT_CHUNK} au plus, dans
 * l'ordre reçu (l'écran trie par date avant).
 */
export function chunkProposals<T>(items: readonly T[], size = PAYPAL_ACCEPT_CHUNK): T[][] {
	const out: T[][] = [];
	for (let i = 0; i < items.length; i += size) out.push(items.slice(i, i + size));
	return out;
}
