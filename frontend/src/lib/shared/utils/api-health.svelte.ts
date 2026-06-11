/**
 * Store API health (Svelte 5 runes) — Story 10.3.
 *
 * Pilote l'état dégradé global affiché par `<DegradedBanner />` (`+layout.svelte`)
 * quand l'API est injoignable ou que la DB est down (réponses NETWORK_ERROR /
 * TIMEOUT / 503 sur les requêtes idempotentes traversant `api-client.ts`).
 *
 * À l'entrée en état dégradé, démarre un ping périodique `GET /health` toutes
 * les `HEALTH_POLL_INTERVAL_MS` (5s). À la première réponse `{ db: true }`,
 * sort de l'état dégradé et masque le banner.
 *
 * Le fichier est `.svelte.ts` → exécuté uniquement en CSR (`+layout.ts:1-2`
 * fixe `ssr = false` + `prerender = false`), donc `setInterval` est safe.
 */

import { appVersion } from '$lib/shared/utils/app-version.svelte';
import { featureFlags } from '$lib/shared/utils/feature-flags.svelte';

let _isDegraded = $state<boolean>(false);
let _pollTimer: ReturnType<typeof setInterval> | null = null;
// Pass 3 H2 : epoch counter pour invalider les callbacks `pollHealth` en
// vol quand l'état dégradé est cleared (puis éventuellement ré-armé) entre
// le scheduling de la callback et sa résolution. Sans ce compteur, scénario
// observable :
//  T0     : setDegraded() → epoch=1, setInterval(pollHealth) armé
//  T0+5s  : tick pollHealth → fetch('/health') en vol
//  T0+5s2 : un apiClient.get() concurrent renvoie 200 → clearDegraded() externe
//  T0+5s3 : un autre apiClient.get() échoue → setDegraded() ré-armé (epoch=3)
//  T0+5s4 : le pollHealth(1) en vol résout avec db:true (snapshot moment up)
//           → appellerait clearDegraded() à tort, masquant l'état dégradé légitime
// Avec le compteur, pollHealth capture l'epoch au scheduling et vérifie qu'il
// matche encore avant clearDegraded → invalidation déterministe.
let _epoch = 0;

/** Intervalle entre 2 pings `/health` pendant l'état dégradé (5 secondes). */
export const HEALTH_POLL_INTERVAL_MS = 5000;

/**
 * Ping `/health` non-récursif (fetch natif, **pas** `apiClient.get`) pour
 * éviter le retry-during-degraded. Wrappé dans try/catch : toute erreur
 * (network failure, CORS, mixed content, JSON parse, **timeout 2s**) est
 * swallow — le prochain tick réessaiera. Garantit que la `Promise<void>`
 * retournée à `setInterval` ne rejette jamais (pas d'`unhandledrejection`
 * pollution).
 *
 * AC #14ter : `AbortSignal.timeout(2000)` borne le fetch à 2s. Sans ce
 * timeout, un kesh-api qui accepte TCP mais hang HTTP (frozen server) ferait
 * accumuler une Promise pendante par tick `setInterval` (1 par 5s = 720 en
 * 1h) → saturation du pool de connexions HTTP/1.1 du browser (6 par origine)
 * → l'app entière voit ses requêtes API legitime se ralentir.
 *
 * Pass 3 H2 : reçoit `capturedEpoch` du scheduler ; check ground-truth avant
 * `clearDegraded()` que `_epoch` n'a pas changé entre temps (cf. commentaire
 * module ligne 17).
 */
async function pollHealth(capturedEpoch: number): Promise<void> {
	try {
		const res = await fetch('/health', { signal: AbortSignal.timeout(2000) });
		if (!res.ok) return;
		const body = (await res.json()) as {
			db?: unknown;
			version?: unknown;
			forgotPasswordEnabled?: unknown;
		};
		// #159 : capter la version du binaire backend depuis le même ping
		// `/health` (DRY — pas d'endpoint ni de fetch dédié).
		appVersion.set(body.version);
		// Story 17-4d (DD-1) : même DRY pour le flag recovery (DC9). Le refresh
		// ici est INTENTIONNEL (Pass 1 BH#9) : après un redéploiement backend qui
		// change KESH_FEATURE_FORGOT_PASSWORD (et passe par un épisode dégradé),
		// le lien login se met à jour sans reload.
		featureFlags.setForgotPasswordEnabled(body.forgotPasswordEnabled);
		if (body.db !== true) return;
		// Stale check Pass 3 H2 : si l'epoch courant diffère de celui capturé
		// au scheduling, c'est qu'un cycle clearDegraded→setDegraded s'est
		// produit entre temps → l'état dégradé actuel est légitimement nouveau,
		// ne pas le clear avec une réponse vieille.
		if (_epoch !== capturedEpoch) return;
		apiHealth.clearDegraded();
	} catch {
		// swallow — on reste dégradé, prochain tick réessaiera
	}
}

export const apiHealth = {
	get isDegraded(): boolean {
		return _isDegraded;
	},

	/** Bascule en état dégradé. Idempotent (no-op si déjà degraded). */
	setDegraded(): void {
		if (_isDegraded) return;
		_isDegraded = true;
		_epoch += 1;
		// Capture l'epoch au scheduling et passe-le à chaque tick — chaque
		// callback en vol vérifiera la stabilité de l'epoch avant clearDegraded
		// (Pass 3 H2 stale-poll race mitigation).
		const myEpoch = _epoch;
		_pollTimer = setInterval(() => pollHealth(myEpoch), HEALTH_POLL_INTERVAL_MS);
	},

	/** Sort de l'état dégradé. Idempotent (no-op si déjà clean). */
	clearDegraded(): void {
		if (!_isDegraded) return;
		_isDegraded = false;
		_epoch += 1;
		if (_pollTimer !== null) {
			clearInterval(_pollTimer);
			_pollTimer = null;
		}
	},
};
