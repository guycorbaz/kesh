/**
 * Store de la version applicative (Svelte 5 runes) — #159.
 *
 * La version provient du **backend au runtime** : champ `version` de la réponse
 * `GET /health`, résolu côté Rust via `env!("CARGO_PKG_VERSION")` (donc figé à
 * la compilation du binaire `kesh-api`, source unique = `crates/kesh-api/Cargo.toml`
 * bumpé par `scripts/prepare-release.sh`). Sourcer depuis le backend garantit
 * que le numéro affiché correspond toujours au binaire qui tourne, sans dépendre
 * d'un `frontend/package.json` à bumper manuellement à chaque release.
 *
 * Peuplé au boot par le ping `/health` du root layout (`+layout.svelte`), et
 * par les pings de recovery `pollHealth` (`api-health.svelte.ts`). Affiché dans
 * les pieds de page (app + onboarding) et sur login/setup. Fallback `'—'` tant
 * que la réponse backend n'est pas arrivée.
 */

let _version = $state<string>('—');

export const appVersion = {
	/** Version courante (`'—'` avant la 1ère réponse `/health`). */
	get value(): string {
		return _version;
	},

	/** Mémorise la version renvoyée par le backend. No-op si vide/non-string. */
	set(version: unknown): void {
		if (typeof version === 'string' && version.length > 0) {
			_version = version;
		}
	}
};
