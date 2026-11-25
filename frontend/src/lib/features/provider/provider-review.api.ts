/**
 * Story 40-7d (AC-2, AC-3) — client **commun** des routes des prestataires
 * (PayPal 38-11b/38-11c2, Stripe 40-7b/40-7c) : import du fichier, revue,
 * acceptation par paquets. Chaque feature l'enveloppe sous ses noms
 * (`paypal.api.ts`, `stripe.api.ts`).
 */
import { apiClient } from '$lib/shared/utils/api-client';
import type { ProviderAcceptResponse, ProviderProposalBase, ProviderReview } from './provider-review.types';

/** Les prestataires servis par les routes `/<prestataire>-imports`, `/<prestataire>/review`, `/<prestataire>/accept`. */
export type ReviewedProvider = 'paypal' | 'stripe';

/**
 * Taille maximale d'un paquet d'acceptation : le plafond du serveur
 * (`MAX_IN_CLAUSE_IDS`, 500 propositions ; un corps plus long est refusé en
 * `400`).
 */
export const PROVIDER_ACCEPT_CHUNK = 500;

/**
 * `POST /api/v1/<prestataire>-imports` (multipart : `file`, `account`) —
 * importe le fichier du compte `account`, ventilé par devise. Refus : `ApiError`.
 */
export async function importProviderFile<R>(
	provider: ReviewedProvider,
	file: File,
	account: string,
): Promise<R> {
	const form = new FormData();
	form.append('file', file);
	form.append('account', account);
	return apiClient.postFormData<R>(`/api/v1/${provider}-imports`, form);
}

/** `GET /api/v1/<prestataire>/review` — revue d'un compte sur un exercice (lecture seule). */
export async function getProviderReview<P extends ProviderProposalBase>(
	provider: ReviewedProvider,
	account: string,
	fiscalYearId: number,
): Promise<ProviderReview<P>> {
	const params = new URLSearchParams({ account, fiscalYearId: String(fiscalYearId) });
	return apiClient.get<ProviderReview<P>>(`/api/v1/${provider}/review?${params.toString()}`);
}

/** `POST /api/v1/<prestataire>/accept` — un paquet de propositions (500 au plus). */
export async function acceptProviderProposals<B>(
	provider: ReviewedProvider,
	account: string,
	proposals: B[],
): Promise<ProviderAcceptResponse> {
	return apiClient.post<ProviderAcceptResponse>(`/api/v1/${provider}/accept`, { account, proposals });
}

/**
 * Découpe une sélection en paquets de {@link PROVIDER_ACCEPT_CHUNK} au plus,
 * dans l'ordre reçu (l'écran trie par date avant).
 */
export function chunkProposals<T>(items: readonly T[], size = PROVIDER_ACCEPT_CHUNK): T[][] {
	const out: T[][] = [];
	for (let i = 0; i < items.length; i += size) out.push(items.slice(i, i + size));
	return out;
}
