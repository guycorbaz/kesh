/**
 * Store d'authentification global (Svelte 5 runes) — Story 10-5 refactor.
 *
 * Pattern objet avec getters — l'export direct `$state` est non
 * réassignable depuis un importeur (voir Story 1.9 pattern mode.svelte.ts).
 *
 * Story 10-5 (D5 acté) : `_accessToken` et `_refreshToken` ne sont plus
 * stockés. Les tokens sont en cookies `HttpOnly` + `Secure` + `SameSite=Strict`
 * inaccessibles au JavaScript. `_currentUser` est peuplé depuis le body de
 * la réponse `/login` (D6 — body étendu avec `userId/username/role`) ou via
 * un fetch `/api/v1/auth/me` au boot (`hydrate()` — restaure l'identité
 * sans pouvoir décoder le JWT).
 *
 * `isAuthenticated` getter dépend de `_currentUser !== null` (D5).
 */

import { resetVatRatesCache } from '$lib/features/vat-rates';

export interface CurrentUser {
	/** `sub` du JWT — user_id (i64 sérialisé en string côté backend). */
	userId: string;
	/** Username pour affichage UI (Story 10-5 D6 — récupéré depuis body /login ou /me). */
	username: string;
	/** Rôle RBAC : `Admin`, `Comptable`, `Consultation`. */
	role: string;
}

let _expiresIn = $state<number | null>(null);
let _currentUser = $state<CurrentUser | null>(null);
let _hydrated = false;

/**
 * Story 10-5 : constantes STORAGE_KEY_* conservées comme **defensive cleanup**
 * pour purger les sessions pre-Story 10-5 qui auraient persisté ces clés dans
 * localStorage avant le déploiement de cette version. À retirer dans une
 * release v0.2+ une fois tous les utilisateurs migrés.
 */
export const STORAGE_KEY_ACCESS_TOKEN = 'kesh:auth:accessToken';
export const STORAGE_KEY_REFRESH_TOKEN = 'kesh:auth:refreshToken';
export const STORAGE_KEY_EXPIRES_IN = 'kesh:auth:expiresIn';

/** Body de réponse `POST /api/v1/auth/login` (D6 acté). */
export interface LoginPayload {
	userId: string;
	username: string;
	role: string;
	expiresIn: number;
}

export const authState = {
	get expiresIn(): number | null {
		return _expiresIn;
	},
	get currentUser(): CurrentUser | null {
		return _currentUser;
	},
	get isAuthenticated(): boolean {
		// Story 10-5 D5 : dépend de _currentUser (les tokens sont en cookies HttpOnly).
		return _currentUser !== null;
	},

	/**
	 * Set l'état authentifié depuis le body de réponse `/login` ou `/me`.
	 * Le navigateur a déjà set les cookies HttpOnly via les headers Set-Cookie
	 * du backend — pas besoin de stocker les tokens côté JS.
	 */
	login(payload: LoginPayload) {
		_currentUser = {
			userId: payload.userId,
			username: payload.username,
			role: payload.role,
		};
		_expiresIn = payload.expiresIn;
	},

	/**
	 * Met à jour `expiresIn` après un refresh proactif réussi (T6.7).
	 * Pas d'effet sur `_currentUser` (même utilisateur, juste JWT refreshed).
	 */
	updateExpiresIn(expiresIn: number) {
		_expiresIn = expiresIn;
	},

	/**
	 * Nettoie le state d'authentification SANS appeler l'API logout.
	 * Utilisé quand le refresh a échoué (le cookie est déjà invalidé côté
	 * serveur, inutile d'appeler logout).
	 */
	clearSession() {
		_expiresIn = null;
		_currentUser = null;
		// Defensive cleanup localStorage pour utilisateurs migrant depuis pre-Story 10-5.
		if (typeof window !== 'undefined' && window.localStorage) {
			window.localStorage.removeItem(STORAGE_KEY_ACCESS_TOKEN);
			window.localStorage.removeItem(STORAGE_KEY_REFRESH_TOKEN);
			window.localStorage.removeItem(STORAGE_KEY_EXPIRES_IN);
		}
		resetVatRatesCache();
	},

	async logout() {
		// POST /api/v1/auth/logout avec credentials: 'include' — le browser
		// envoie automatiquement le cookie HttpOnly. Pas de body refresh_token.
		await fetch('/api/v1/auth/logout', {
			method: 'POST',
			credentials: 'include',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({}),
		}).catch(() => {});
		_expiresIn = null;
		_currentUser = null;
		// Defensive cleanup localStorage.
		if (typeof window !== 'undefined' && window.localStorage) {
			window.localStorage.removeItem(STORAGE_KEY_ACCESS_TOKEN);
			window.localStorage.removeItem(STORAGE_KEY_REFRESH_TOKEN);
			window.localStorage.removeItem(STORAGE_KEY_EXPIRES_IN);
		}
		resetVatRatesCache();
	},

	/**
	 * Restaure l'identité utilisateur depuis le cookie HttpOnly via
	 * `GET /api/v1/auth/me` (Story 10-5 T6.3).
	 *
	 * Appelé une seule fois au boot via `hooks.client.ts` (ClientInit async
	 * hook). Garantit que `_currentUser` est peuplé AVANT que les `load()`
	 * functions s'exécutent — sans quoi `(app)/+layout.ts:10` redirige
	 * vers /login pour tous les utilisateurs authentifiés.
	 *
	 * Si 200 → peuple `_currentUser` + `_expiresIn`.
	 * Si 401 (cookie absent ou expiré) → état non-auth (utilisateur doit relog).
	 * Si erreur réseau → swallow silencieux, état non-auth.
	 */
	async hydrate(): Promise<void> {
		// Guard idempotence : éviter double-fetch concurrents /me.
		if (_hydrated) {
			return;
		}
		if (typeof window === 'undefined') {
			return;
		}

		try {
			const res = await fetch('/api/v1/auth/me', { credentials: 'include' });
			if (res.ok) {
				const body = (await res.json()) as {
					userId: number;
					username: string;
					role: string;
					expiresIn: number;
				};
				_currentUser = {
					userId: String(body.userId),
					username: body.username,
					role: body.role,
				};
				_expiresIn = body.expiresIn;
			}
			// res.status === 401 → state non-auth (default null state)
			// res.status autre → swallow, state non-auth
		} catch (error) {
			console.error('[auth] Hydration via /me failed:', error instanceof Error ? error.message : String(error));
		} finally {
			_hydrated = true;
		}
	},
};
