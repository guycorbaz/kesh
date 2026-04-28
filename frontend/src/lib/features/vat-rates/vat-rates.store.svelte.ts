/**
 * Store de session pour les taux TVA (Story 7.2 — KF-003 closure).
 *
 * Pattern « inflight-promise » : si deux composants montent en parallèle,
 * le second attend la promesse en cours plutôt que de relancer un fetch.
 *
 * Cache invalidé au logout via `resetVatRatesCache()` (appelé depuis
 * `auth.svelte.ts::logout()`).
 */

import { listVatRates } from './vat-rates.api';
import type { VatRateResponse } from './vat-rates.types';

let cache: VatRateResponse[] | null = null;
let inflight: Promise<VatRateResponse[]> | null = null;

/**
 * Retourne les taux TVA actifs pour la company de l'user courant. Cache
 * de session — un seul fetch tant que l'user reste connecté.
 *
 * Sous concurrence (deux mounts simultanés), retourne la même promesse
 * pour éviter une double-requête réseau.
 */
export async function getVatRates(): Promise<VatRateResponse[]> {
	if (cache !== null) return cache;
	if (inflight !== null) return inflight;
	inflight = listVatRates()
		.then((rates) => {
			cache = rates;
			inflight = null;
			return rates;
		})
		.catch((err) => {
			// Reset inflight pour autoriser une retry au prochain appel.
			inflight = null;
			throw err;
		});
	return inflight;
}

/**
 * Invalide le cache des taux TVA. À appeler au logout pour éviter qu'un
 * utilisateur suivant sur le même browser hérite des taux du précédent
 * (cross-tenant si la même session web sert plusieurs companies).
 */
export function resetVatRatesCache(): void {
	cache = null;
	inflight = null;
}
