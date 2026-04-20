/**
 * Store d'authentification global (Svelte 5 runes).
 *
 * Pattern objet avec getters — l'export direct `$state` est non
 * réassignable depuis un importeur (voir Story 1.9 pattern mode.svelte.ts).
 *
 * Le JWT est décodé côté client (base64url, sans vérification de
 * signature) pour extraire `sub` (userId) et `role`.
 * `username` n'est PAS dans le JWT — absent de `CurrentUser`.
 */

export interface CurrentUser {
	/** `sub` du JWT — user_id (i64 sérialisé en string côté backend). */
	userId: string;
	/** Rôle RBAC : `Admin`, `Comptable`, `Consultation`. */
	role: string;
}

let _accessToken = $state<string | null>(null);
let _refreshToken = $state<string | null>(null);
let _expiresIn = $state<number | null>(null);
let _currentUser = $state<CurrentUser | null>(null);
let _hydrated = false;

/**
 * Décode le payload JWT (segment central, base64url) sans
 * vérification de signature. Ajoute le padding `=` manquant
 * pour compatibilité `atob()`.
 */
function decodeJwtPayload(token: string): { sub: string; role: string; exp: number } {
	const parts = token.split('.');
	if (parts.length !== 3) {
		throw new Error(`JWT malformé : ${parts.length} segment(s) au lieu de 3`);
	}
	const segment = parts[1];
	// Restaurer le base64 standard + padding
	const base64 = segment.replace(/-/g, '+').replace(/_/g, '/');
	const padded = base64.padEnd(base64.length + ((4 - (base64.length % 4)) % 4), '=');
	let payload: Record<string, unknown>;
	try {
		payload = JSON.parse(atob(padded));
	} catch {
		throw new Error('Impossible de décoder le payload JWT');
	}
	if (typeof payload.sub !== 'string' || !payload.sub || typeof payload.role !== 'string' || !payload.role) {
		throw new Error(`Claims JWT manquants ou vides : sub=${payload.sub}, role=${payload.role}`);
	}
	return payload as { sub: string; role: string; exp: number };
}

export const STORAGE_KEY_ACCESS_TOKEN = 'kesh:auth:accessToken';
export const STORAGE_KEY_REFRESH_TOKEN = 'kesh:auth:refreshToken';
export const STORAGE_KEY_EXPIRES_IN = 'kesh:auth:expiresIn';

export const authState = {
	get accessToken(): string | null {
		return _accessToken;
	},
	get refreshToken(): string | null {
		return _refreshToken;
	},
	get expiresIn(): number | null {
		return _expiresIn;
	},
	get currentUser(): CurrentUser | null {
		return _currentUser;
	},
	get isAuthenticated(): boolean {
		return _accessToken !== null;
	},

	login(accessToken: string, refreshToken: string, expiresIn: number) {
		// Valider AVANT d'affecter — garantir l'atomicité du state
		const claims = decodeJwtPayload(accessToken);
		_accessToken = accessToken;
		_refreshToken = refreshToken;
		_expiresIn = expiresIn;
		_currentUser = { userId: claims.sub, role: claims.role };
		// Persister à localStorage pour survire aux navigations de page
		if (typeof window !== 'undefined' && window.localStorage) {
			window.localStorage.setItem(STORAGE_KEY_ACCESS_TOKEN, accessToken);
			window.localStorage.setItem(STORAGE_KEY_REFRESH_TOKEN, refreshToken);
			window.localStorage.setItem(STORAGE_KEY_EXPIRES_IN, String(expiresIn));
		}
	},

	/**
	 * Nettoie le state d'authentification SANS appeler l'API logout.
	 * Utilisé quand le refresh token a échoué (le token est déjà
	 * invalide côté serveur, inutile d'appeler logout).
	 */
	clearSession() {
		_accessToken = null;
		_refreshToken = null;
		_expiresIn = null;
		_currentUser = null;
		// Nettoyer localStorage aussi
		if (typeof window !== 'undefined' && window.localStorage) {
			window.localStorage.removeItem(STORAGE_KEY_ACCESS_TOKEN);
			window.localStorage.removeItem(STORAGE_KEY_REFRESH_TOKEN);
			window.localStorage.removeItem(STORAGE_KEY_EXPIRES_IN);
		}
	},

	async logout() {
		// POST /api/v1/auth/logout — PAS de header Authorization requis
		// (design intentionnel backend : accepte logout même avec JWT expiré)
		if (_refreshToken) {
			await fetch('/api/v1/auth/logout', {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({ refreshToken: _refreshToken }),
			}).catch(() => {});
		}
		_accessToken = null;
		_refreshToken = null;
		_expiresIn = null;
		_currentUser = null;
		// Nettoyer localStorage aussi
		if (typeof window !== 'undefined' && window.localStorage) {
			window.localStorage.removeItem(STORAGE_KEY_ACCESS_TOKEN);
			window.localStorage.removeItem(STORAGE_KEY_REFRESH_TOKEN);
			window.localStorage.removeItem(STORAGE_KEY_EXPIRES_IN);
		}
	},

	/**
	 * Restaure les tokens depuis localStorage (appelé au démarrage de l'app).
	 * Safe pour SSR : vérifie typeof window avant d'accéder à localStorage.
	 * Valide l'expiration du token avant restauration.
	 */
	hydrate() {
		// Guard: Only hydrate once (prevent concurrent load() functions from restoring multiple times)
		if (_hydrated) {
			return;
		}
		if (typeof window === 'undefined' || !window.localStorage) {
			return;
		}
		const accessToken = window.localStorage.getItem(STORAGE_KEY_ACCESS_TOKEN);
		const refreshToken = window.localStorage.getItem(STORAGE_KEY_REFRESH_TOKEN);
		const expiresInStr = window.localStorage.getItem(STORAGE_KEY_EXPIRES_IN);

		try {
			if (accessToken && refreshToken && expiresInStr) {
				const expiresIn = parseInt(expiresInStr, 10);
				if (isNaN(expiresIn)) {
					throw new Error('expiresIn is not a valid number');
				}
				// Valider le token AVANT de l'affecter
				const claims = decodeJwtPayload(accessToken);

				// Vérifier que le token n'est pas expiré (exp en secondes, Date.now() en ms)
				const nowSeconds = Math.floor(Date.now() / 1000);
				if (claims.exp < nowSeconds) {
					console.warn('[auth] Token expired during hydration, clearing session');
					window.localStorage.removeItem(STORAGE_KEY_ACCESS_TOKEN);
					window.localStorage.removeItem(STORAGE_KEY_REFRESH_TOKEN);
					window.localStorage.removeItem(STORAGE_KEY_EXPIRES_IN);
				} else {
					_accessToken = accessToken;
					_refreshToken = refreshToken;
					_expiresIn = expiresIn;
					_currentUser = { userId: claims.sub, role: claims.role };
				}
			}
		} catch (error) {
			// Token invalide ou décodage échoué — nettoyer localStorage
			console.error('[auth] Hydration failed, clearing session:', error instanceof Error ? error.message : String(error));
			window.localStorage.removeItem(STORAGE_KEY_ACCESS_TOKEN);
			window.localStorage.removeItem(STORAGE_KEY_REFRESH_TOKEN);
			window.localStorage.removeItem(STORAGE_KEY_EXPIRES_IN);
		} finally {
			// Mark hydration as complete regardless of outcome (prevents duplicate attempts)
			_hydrated = true;
		}
	},
};
