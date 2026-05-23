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

let _isDegraded = $state<boolean>(false);
let _pollTimer: ReturnType<typeof setInterval> | null = null;

/** Intervalle entre 2 pings `/health` pendant l'état dégradé (5 secondes). */
export const HEALTH_POLL_INTERVAL_MS = 5000;

/**
 * Ping `/health` non-récursif (fetch natif, **pas** `apiClient.get`) pour
 * éviter le retry-during-degraded. Wrappé dans try/catch : toute erreur
 * (network failure, CORS, mixed content, JSON parse) est swallow — le prochain
 * tick réessaiera. Garantit que la `Promise<void>` retournée à `setInterval`
 * ne rejette jamais (pas d'`unhandledrejection` pollution).
 */
async function pollHealth(): Promise<void> {
	try {
		const res = await fetch('/health');
		if (!res.ok) return;
		const body = (await res.json()) as { db?: unknown };
		if (body.db === true) {
			apiHealth.clearDegraded();
		}
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
		_pollTimer = setInterval(pollHealth, HEALTH_POLL_INTERVAL_MS);
	},

	/** Sort de l'état dégradé. Idempotent (no-op si déjà clean). */
	clearDegraded(): void {
		if (!_isDegraded) return;
		_isDegraded = false;
		if (_pollTimer !== null) {
			clearInterval(_pollTimer);
			_pollTimer = null;
		}
	},
};
