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
