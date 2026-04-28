/**
 * Store de session pour les taux TVA (Story 7.2 — KF-003 closure).
 *
 * Pattern « inflight-promise + generation counter » : si deux composants
 * montent en parallèle, le second attend la promesse en cours plutôt que
 * de relancer un fetch. Le compteur de génération garantit qu'une promesse
 * en vol pendant un `resetVatRatesCache()` (logout, switch tenant) ne
 * pollue pas le cache du tenant suivant.
 *
 * Cache invalidé au logout via `resetVatRatesCache()` (appelé depuis
 * `auth.svelte.ts::clearSession()` et `auth.svelte.ts::logout()`).
 */

import { listVatRates } from './vat-rates.api';
import type { VatRateResponse } from './vat-rates.types';

let cache: VatRateResponse[] | null = null;
let inflight: Promise<VatRateResponse[]> | null = null;
// Compteur incrementé à chaque reset. Une promesse en vol capturera la
// génération courante au moment du fetch ; au resolve, elle ne met à
// jour le cache que si la génération est inchangée — sinon, le résultat
// appartient à un tenant précédent et doit être ignoré.
let generation = 0;

/**
 * Retourne les taux TVA actifs pour la company de l'user courant. Cache
 * de session — un seul fetch tant que l'user reste connecté.
 *
 * Sous concurrence (deux mounts simultanés), retourne la même promesse
 * pour éviter une double-requête réseau. Si un `resetVatRatesCache()`
 * intervient pendant le fetch (logout puis re-login d'un autre user), le
 * résultat est ignoré et un nouveau fetch est déclenché au prochain appel.
 */
export async function getVatRates(): Promise<VatRateResponse[]> {
	if (cache !== null) return cache;
	if (inflight !== null) return inflight;
	const fetchGeneration = generation;
	inflight = listVatRates()
		.then((rates) => {
			// Si la génération a changé pendant le fetch, le user a logout/switché
			// de tenant — le résultat appartient au tenant précédent, on jette.
			if (fetchGeneration === generation) {
				cache = rates;
				inflight = null;
			}
			return rates;
		})
		.catch((err) => {
			// Reset inflight pour autoriser une retry au prochain appel —
			// uniquement si la génération est toujours la nôtre.
			if (fetchGeneration === generation) {
				inflight = null;
			}
			throw err;
		});
	return inflight;
}

/**
 * Invalide le cache des taux TVA. À appeler au logout pour éviter qu'un
 * utilisateur suivant sur le même browser hérite des taux du précédent
 * (cross-tenant si la même session web sert plusieurs companies).
 *
 * Incrémente la génération pour neutraliser toute promesse en vol qui
 * tenterait d'écrire dans le cache après le reset.
 */
export function resetVatRatesCache(): void {
	cache = null;
	inflight = null;
	generation += 1;
}
