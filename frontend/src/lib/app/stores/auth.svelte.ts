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

/**
 * CR Pass 3 ECH3-M4 — `BroadcastChannel` singleton pour la synchronisation
 * cross-tab du state d'authentification. Tab1 émet `{ type: 'auth-change' }`
 * sur login/logout/clearSession ; les autres tabs reçoivent l'event, resettent
 * `_hydrated = false`, puis `hydrate()` se re-déclenche pour resync le state
 * depuis le serveur (cookie HttpOnly partagé pour le domaine, le state JS
 * peut diverger sans cette synchronisation).
 *
 * Fallback `null` si l'API n'est pas disponible (SSR / browsers sans
 * BroadcastChannel — Chrome 54+, Firefox 38+, Safari 15.4+, sufficient v0.1).
 * Tests Vitest jsdom : `BroadcastChannel` peut être indéfini selon la version
 * — le code tolère.
 */
const AUTH_BROADCAST_CHANNEL_NAME = 'kesh:auth';
const authBroadcast: BroadcastChannel | null =
	typeof window !== 'undefined' && typeof BroadcastChannel !== 'undefined'
		? new BroadcastChannel(AUTH_BROADCAST_CHANNEL_NAME)
		: null;

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
 * Story v011-5 — flag « setup-required » set à `true` quand `/me` ou tout autre
 * route protégée retourne `423 Locked` (table `users` vide côté backend, le
 * 1er admin n'a pas encore été créé via `/setup`). Le `+layout.ts` racine
 * inspecte ce flag au boot et redirige vers `/setup` ; `clearSession()` et
 * `login()` le resettent à `false` (un login réussi implique que les users
 * existent → state cohérent).
 */
let _setupRequired = $state<boolean>(false);

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
	/** Story v011-5 — true si `/me` ou autre route protégée retourne 423 Locked. */
	get isSetupRequired(): boolean {
		return _setupRequired;
	},

	/**
	 * Set l'état authentifié depuis le body de réponse `/login` ou `/me`.
	 * Le navigateur a déjà set les cookies HttpOnly via les headers Set-Cookie
	 * du backend — pas besoin de stocker les tokens côté JS.
	 *
	 * CR Pass 3 ECH3-M2 : `_hydrated = false` au début pour symétrie avec
	 * `clearSession()` / `logout()` (qui resettent aussi `_hydrated`). Sans
	 * ça, un boot 401 → login intra-SPA → futur `hydrate()` aurait early-
	 * return malgré un state à re-syncer côté serveur. Le re-set à `true`
	 * en fin signale « state synchronisé via login payload, pas besoin de /me ».
	 *
	 * CR Pass 3 ECH3-M4 : broadcast `auth-change` cross-tab pour que les
	 * autres tabs resync via `hydrate()`.
	 */
	login(payload: LoginPayload, options?: { broadcast?: boolean }) {
		_hydrated = false;
		_currentUser = {
			userId: payload.userId,
			username: payload.username,
			role: payload.role,
		};
		_expiresIn = payload.expiresIn;
		// Story v011-5 — un login réussi implique que les users existent côté
		// backend → state cohérent. Reset le flag (cas use : setup réussit dans
		// une autre tab → broadcast → cette tab re-hydrate via login()).
		_setupRequired = false;
		_hydrated = true;
		if (options?.broadcast !== false) {
			authBroadcast?.postMessage({ type: 'auth-change' });
		}
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
	 *
	 * CR Pass 1 M2 : réinitialise `_hydrated = false` pour permettre un
	 * éventuel ré-hydrate dans la même SPA session (post-logout puis
	 * re-login intra-SPA). Pré-Story-10-5, `_hydrated` était set au boot
	 * via `hooks.client.ts` synchrone et restait true — pas re-utilisé.
	 * Post-Story 10-5, `hydrate()` est async via /me et pourrait être
	 * appelée à nouveau si l'app détecte un état desynchronisé.
	 */
	clearSession(options?: { broadcast?: boolean }) {
		_expiresIn = null;
		_currentUser = null;
		_setupRequired = false;
		_hydrated = false;
		// Defensive cleanup localStorage pour utilisateurs migrant depuis pre-Story 10-5.
		if (typeof window !== 'undefined' && window.localStorage) {
			window.localStorage.removeItem(STORAGE_KEY_ACCESS_TOKEN);
			window.localStorage.removeItem(STORAGE_KEY_REFRESH_TOKEN);
			window.localStorage.removeItem(STORAGE_KEY_EXPIRES_IN);
		}
		resetVatRatesCache();
		// CR Pass 3 ECH3-M4 : broadcast cross-tab pour resync les autres tabs.
		if (options?.broadcast !== false) {
			authBroadcast?.postMessage({ type: 'auth-change' });
		}
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
		_hydrated = false; // CR Pass 1 M2 — permet re-hydrate intra-SPA session
		// Defensive cleanup localStorage.
		if (typeof window !== 'undefined' && window.localStorage) {
			window.localStorage.removeItem(STORAGE_KEY_ACCESS_TOKEN);
			window.localStorage.removeItem(STORAGE_KEY_REFRESH_TOKEN);
			window.localStorage.removeItem(STORAGE_KEY_EXPIRES_IN);
		}
		resetVatRatesCache();
		// CR Pass 3 ECH3-M4 : broadcast cross-tab pour resync les autres tabs.
		authBroadcast?.postMessage({ type: 'auth-change' });
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
	 * Si 401 (cookie absent ou expiré) → reset state à null (CR Pass 3 ECH3-M1).
	 * Si 5xx (backend KO transitoire) → reset state à null + console.warn
	 *   explicite, mais NE redirige PAS vers /login en silence (CR Pass 3 BH3-M3).
	 * Si erreur réseau → swallow silencieux, état non-auth.
	 *
	 * CR Pass 3 ECH3-H1 : purge defensive `STORAGE_KEY_*` AVANT le fetch /me
	 * pour éliminer la fenêtre XSS de migration pre-Story-10-5 → post-Story-10-5
	 * (anciens tokens en localStorage volables jusqu'au prochain logout). 3 keys
	 * purgées au boot de chaque session est negligible côté perf.
	 */
	async hydrate(): Promise<void> {
		// Guard idempotence : éviter double-fetch concurrents /me.
		if (_hydrated) {
			return;
		}
		if (typeof window === 'undefined') {
			return;
		}

		// CR Pass 3 ECH3-H1 — defensive cleanup pré-fetch : purge les 3 keys
		// localStorage pre-Story-10-5 au boot de chaque session. Élimine la
		// fenêtre XSS migration où un refresh JWT legacy (TTL 30j) resterait
		// volable jusqu'au prochain logout/clearSession.
		if (window.localStorage) {
			window.localStorage.removeItem(STORAGE_KEY_ACCESS_TOKEN);
			window.localStorage.removeItem(STORAGE_KEY_REFRESH_TOKEN);
			window.localStorage.removeItem(STORAGE_KEY_EXPIRES_IN);
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
				// CR Pass 4 BH4-M1↓ : validation runtime de la shape /me — endpoint
				// est server-controlled typé strict (MeResponse Rust), mais defense-
				// in-depth contre proxy/WAF/migration retournant `{userId: null}` ou
				// fields manquants. Cast TS `as` ne fournit AUCUNE protection runtime.
				if (
					typeof body.userId !== 'number' ||
					typeof body.username !== 'string' ||
					typeof body.role !== 'string' ||
					typeof body.expiresIn !== 'number'
				) {
					console.error(
						'[auth] /me returned malformed body — expected { userId: number, username: string, role: string, expiresIn: number }, got:',
						body,
					);
					_currentUser = null;
					_expiresIn = null;
				} else {
					_currentUser = {
						userId: String(body.userId),
						username: body.username,
						role: body.role,
					};
					_expiresIn = body.expiresIn;
				}
			} else if (res.status === 401) {
				// CR Pass 3 ECH3-M1 — reset state si /me retourne 401 : couvre le
				// cas où `hydrate()` est appelé sur un state déjà populé (e.g.
				// cookie révoqué côté serveur entre 2 hydrates intra-SPA).
				_currentUser = null;
				_expiresIn = null;
				_setupRequired = false;
			} else if (res.status === 423) {
				// Story v011-5 (AC #18) — `/me` retourne 423 Locked : la table
				// `users` est vide côté backend, le 1er admin n'a pas encore été
				// créé via `/setup`. Set `_setupRequired = true` pour que le
				// layout racine redirige vers `/setup`. **Sans `console.warn`** :
				// 423 est un état légal documenté, pas un signal d'indisponibilité
				// backend (distingué du else 5xx ci-dessous).
				_currentUser = null;
				_expiresIn = null;
				_setupRequired = true;
			} else {
				// CR Pass 3 BH3-M3 — discriminer 5xx (backend KO transitoire) de
				// 401 (non-auth légitime). Reset state à null pour cohérence
				// d'isAuthenticated, mais console.warn explicite pour ne pas
				// perdre le signal d'indisponibilité backend (ex: cold start,
				// DB down 5s, restart Axum) — l'observabilité externe (Sentry-
				// like) peut capturer ce warn pour distinguer une vraie panne
				// d'une session expirée normale.
				console.warn(
					`[auth] Hydration via /me returned non-OK status ${res.status} — backend may be temporarily unavailable`,
				);
				_currentUser = null;
				_expiresIn = null;
			}
		} catch (error) {
			console.error('[auth] Hydration via /me failed:', error instanceof Error ? error.message : String(error));
			_currentUser = null;
			_expiresIn = null;
		} finally {
			_hydrated = true;
		}
	},
};

/**
 * CR Pass 3 ECH3-M4 — Listener cross-tab : si une autre tab émet `auth-change`
 * (login/logout/clearSession), reset `_hydrated = false` puis re-déclenche
 * `hydrate()` pour resync le state depuis le serveur (cookie HttpOnly partagé
 * pour le domaine, le state JS de cette tab doit refléter la réalité serveur).
 *
 * Le broadcast est `same-origin` et `same-browser-profile` — pas de fuite
 * d'info cross-domain. Le code ignore les events que cette tab elle-même a
 * émis (BroadcastChannel ne livre PAS au sender — comportement spec WHATWG).
 */
authBroadcast?.addEventListener('message', (event) => {
	if (event.data?.type === 'auth-change') {
		_hydrated = false;
		// Re-déclenche hydrate asynchroniquement — pas besoin d'await ici,
		// l'API reactive Svelte propagera la mise à jour du state aux callers.
		void authState.hydrate();
	}
});
