/**
 * Store des feature-flags backend (Svelte 5 runes) — Story 17-4d (DD-1).
 *
 * Les flags proviennent du **backend au runtime** : champs de la réponse
 * `GET /health` (présents dans les DEUX branches 200/503, indépendants de
 * l'état DB — DC9 Story 17-4c). Peuplé au boot par le ping `/health` du root
 * layout (`+layout.svelte`) et par les pings de recovery `pollHealth`
 * (`api-health.svelte.ts`) — pattern identique à `app-version.svelte.ts`.
 *
 * `forgotPasswordEnabled` : défaut **`false`** tant que `/health` n'a pas
 * répondu — le lien « Mot de passe oublié ? » du login reste masqué sur une
 * installation feature-off plutôt que d'apparaître puis disparaître (anti-flash
 * et anti-faux-affordance, cf. story 17-4d « Pièges connus »).
 *
 * ⚠️ État `$state` module-level : sûr UNIQUEMENT parce que l'app est CSR pure
 * (`ssr = false` + `prerender = false`, `routes/+layout.ts`) — en SSR ce serait
 * un état partagé inter-requêtes. Même contrainte (et même pattern) que
 * `app-version.svelte.ts` et `api-health.svelte.ts` (Pass 1 17-4d BH#4).
 *
 * Limitation connue (Pass 1 ECH#2, acceptée v0.2) : le flag est figé à la
 * valeur du boot tant qu'aucun épisode dégradé ne déclenche `pollHealth` — un
 * admin qui active `KESH_FEATURE_FORGOT_PASSWORD` à chaud n'est reflété dans
 * les onglets déjà ouverts qu'au prochain rechargement de page.
 */

let _forgotPasswordEnabled = $state<boolean>(false);

export const featureFlags = {
	/** `true` si le backend expose le recovery self-service (KESH_FEATURE_FORGOT_PASSWORD). */
	get forgotPasswordEnabled(): boolean {
		return _forgotPasswordEnabled;
	},

	/** Mémorise le flag renvoyé par `/health`. No-op si la valeur n'est pas un booléen. */
	setForgotPasswordEnabled(value: unknown): void {
		if (typeof value === 'boolean') {
			_forgotPasswordEnabled = value;
		}
	},
};
