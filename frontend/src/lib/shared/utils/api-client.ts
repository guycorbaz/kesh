/**
 * Client API – wrapper autour de `fetch()` natif.
 *
 * Fonctionnalités :
 * 1. Injection automatique du header Authorization (sauf login/logout/refresh)
 * 2. Interception 401 → refresh token → retry transparent
 * 3. Mutex de refresh (une seule tentative simultanée)
 * 4. Parsing erreurs structurées → ApiError
 * 5. Retry exponentiel sur NETWORK_ERROR / TIMEOUT / 503 pour les méthodes
 *    idempotentes (GET, HEAD) — Story 10.3. Pilote l'état dégradé global via
 *    `apiHealth.setDegraded()` / `clearDegraded()` (banner DegradedBanner).
 */

import type { ApiError } from '$lib/shared/types/api';
import { authState } from '$lib/app/stores/auth.svelte';
import { apiHealth } from '$lib/shared/utils/api-health.svelte';

/** Type guard pour vérifier qu'une erreur est un ApiError. */
export function isApiError(err: unknown): err is ApiError {
	return (
		typeof err === 'object' &&
		err !== null &&
		'code' in err &&
		'status' in err &&
		typeof (err as ApiError).code === 'string' &&
		typeof (err as ApiError).status === 'number'
	);
}

/** URLs exclues de l'injection du header Authorization. */
const AUTH_EXCLUDED_URLS = ['/api/v1/auth/login', '/api/v1/auth/logout', '/api/v1/auth/refresh'];

/**
 * Délais de retry exponentiels pour les fetch échouant en NETWORK_ERROR /
 * TIMEOUT / 503 sur méthodes idempotentes (Story 10.3). Total : ~14.3s avant
 * give-up. 4 délais ⇒ 5 tentatives totales = 1 initiale + 4 retries.
 *
 * Exporté pour permettre aux tests Vitest de l'override (le mock global
 * `vi.mock` n'est pas nécessaire si on remplace le module ou si on l'override
 * via `window.__KESH_RETRY_DELAYS` pour les tests E2E Playwright).
 */
export const DEGRADED_RETRY_DELAYS_MS = [300, 1000, 3000, 10000] as const;

/** Timeout par défaut pour les requêtes fetch (30 secondes). */
const REQUEST_TIMEOUT_MS = 30_000;

const sleep = (ms: number) => new Promise<void>((resolve) => setTimeout(resolve, ms));

/** Promise partagée pour le mutex de refresh. */
let refreshPromise: Promise<boolean> | null = null;

/**
 * Tente un refresh des tokens via POST /api/v1/auth/refresh.
 * Retourne `true` si le refresh a réussi, `false` sinon.
 * En cas d'échec : clearSession() + redirect login.
 *
 * Story 10-5 (T7.3) :
 * - Plus de guard `!currentRefreshToken` (Pass 1 F-AH-P1-2) — le refresh
 *   passe par le cookie HttpOnly, pas accessible côté JS. Le browser envoie
 *   automatiquement le cookie via `credentials: 'include'`.
 * - Plus de body JSON `{refreshToken: ...}` — le backend lit le cookie.
 * - Pas d'appel à `authState.login(...)` (qui voulait stocker les tokens en
 *   localStorage et décoder le JWT côté JS — anti-pattern D5). On utilise
 *   `authState.updateExpiresIn()` (T6.7) pour bumper `_expiresIn` sans
 *   toucher à `_currentUser` (qui reste inchangé, même utilisateur, juste
 *   nouveau JWT refreshed côté cookie).
 */
async function doRefresh(): Promise<boolean> {
	let res: Response;
	try {
		res = await fetch('/api/v1/auth/refresh', {
			method: 'POST',
			credentials: 'include',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({}),
		});
	} catch {
		authState.clearSession();
		window.location.replace('/login?reason=session_expired');
		return false;
	}

	if (res.ok) {
		let data: Record<string, unknown>;
		try {
			data = await res.json();
		} catch {
			authState.clearSession();
			window.location.replace('/login?reason=session_expired');
			return false;
		}
		// D1 Option A acté : body conserve `accessToken/refreshToken/expiresIn`
		// pour rétro-compat tests 19+ `*_e2e.rs` historiques. On valide
		// uniquement `expiresIn` (les tokens sont ignorés côté JS — ils sont
		// dans les cookies HttpOnly).
		if (typeof data?.expiresIn !== 'number') {
			authState.clearSession();
			window.location.replace('/login?reason=session_expired');
			return false;
		}
		authState.updateExpiresIn(data.expiresIn);
		return true;
	}

	// Refresh échoué (401 INVALID_REFRESH_TOKEN ou autre erreur)
	authState.clearSession();
	window.location.replace('/login?reason=session_expired');
	return false;
}

/**
 * Mutex de refresh : si un refresh est déjà en cours, retourne la
 * même Promise. Sinon, lance un nouveau refresh.
 */
async function refreshTokens(): Promise<boolean> {
	if (refreshPromise) return refreshPromise;
	refreshPromise = doRefresh();
	try {
		return await refreshPromise;
	} finally {
		refreshPromise = null;
	}
}

/**
 * Construit les headers pour une requête.
 *
 * Story 10-5 (T7.2) : ne plus injecter le header `Authorization: Bearer ...`
 * (le browser envoie automatiquement le cookie HttpOnly via `credentials: 'include'`
 * dans `fetchWithTimeout`). La constante `AUTH_EXCLUDED_URLS` est conservée
 * pour traçabilité mais n'a plus de rôle actif post-Story 10-5.
 */
function buildHeaders(
	_url: string,
	customHeaders?: Record<string, string>,
	body?: BodyInit | null,
): Record<string, string> {
	const headers: Record<string, string> = {
		...customHeaders,
	};

	// Story 8-1b §api-client : pour FormData, le browser pose son propre
	// `Content-Type: multipart/form-data; boundary=...` avec le delimiter
	// correct. Forcer `application/json` casserait la requête. On
	// n'ajoute le default JSON que si le body n'est PAS un FormData.
	if (!(body instanceof FormData) && !headers['Content-Type']) {
		headers['Content-Type'] = 'application/json';
	}

	return headers;
}

/**
 * Parse une réponse d'erreur en `ApiError`.
 * Gère les cas : JSON structuré, JSON non structuré, réponse non-JSON.
 */
async function parseErrorResponse(res: Response): Promise<ApiError> {
	let body: unknown;
	try {
		body = await res.json();
	} catch {
		return {
			code: 'UNKNOWN_ERROR',
			message: `Erreur serveur (${res.status})`,
			status: res.status,
		};
	}

	if (
		typeof body === 'object' &&
		body !== null &&
		'error' in body &&
		typeof (body as Record<string, unknown>).error === 'object'
	) {
		const err = (body as { error: Record<string, unknown> }).error;
		return {
			code: typeof err.code === 'string' ? err.code : 'UNKNOWN_ERROR',
			message: typeof err.message === 'string' ? err.message : `Erreur serveur (${res.status})`,
			details: typeof err.details === 'object' ? (err.details as Record<string, unknown>) : undefined,
			status: res.status,
		};
	}

	return {
		code: 'UNKNOWN_ERROR',
		message: `Erreur serveur (${res.status})`,
		status: res.status,
	};
}

/** True si la méthode HTTP est idempotente — éligible au retry exponentiel. */
function isIdempotentMethod(method: string | undefined): boolean {
	return method === undefined || method === 'GET' || method === 'HEAD';
}

/**
 * Un fetch unique avec timeout. Throw en cas de NETWORK_ERROR / TIMEOUT.
 *
 * **⚠️ Pass 3 M2** : si `init.signal` est passé par le caller, il est
 * **silencieusement overridé** par `controller.signal` (spread ordre :
 * `{ ...init, signal: controller.signal }` → l'override gagne). Aucun caller
 * du `apiClient.*` actuel ne passe `signal`, donc OK aujourd'hui. Si un futur
 * caller a besoin d'une cancellation externe (e.g. "abort upload on route
 * change") il faudra composer les signaux via `AbortSignal.any([init.signal,
 * controller.signal])` (API stable Chrome 116+ / Safari 17.4+) plutôt que
 * d'écraser.
 */
async function fetchWithTimeout(url: string, init: RequestInit): Promise<Response> {
	const controller = new AbortController();
	const timeout = setTimeout(() => controller.abort(), REQUEST_TIMEOUT_MS);
	try {
		// Story 10-5 (T7.1) : `credentials: 'include'` pour permettre l'envoi
		// automatique du cookie HttpOnly `kesh_access_token` par le browser.
		// Le cookie n'est pas envoyé par défaut sur cross-origin, et Vite
		// dev `:5173` vs API `:3000` est same-site mais credentials default
		// 'same-origin' peut bloquer selon le browser — `'include'` explicite.
		return await fetch(url, { ...init, credentials: 'include', signal: controller.signal });
	} finally {
		clearTimeout(timeout);
	}
}

/** Construit l'ApiError correspondant à un fetch échoué (network ou timeout). */
function toFetchApiError(err: unknown): ApiError {
	const isTimeout = err instanceof DOMException && err.name === 'AbortError';
	return {
		code: isTimeout ? 'TIMEOUT' : 'NETWORK_ERROR',
		message: isTimeout
			? 'Le serveur ne répond pas. Réessayez ultérieurement.'
			: 'Impossible de contacter le serveur. Vérifiez votre connexion.',
		status: 0,
	};
}

/**
 * Délais de retry actifs — lit `window.__KESH_RETRY_DELAYS` si défini (hook
 * E2E Playwright pour accélérer les tests, cf. Story 10.3 AC #21). Fallback
 * sur `DEGRADED_RETRY_DELAYS_MS` en production. Lecture à chaque appel pour
 * que les tests qui mutent window juste avant le call voient le changement.
 */
function getRetryDelays(): readonly number[] {
	if (typeof window !== 'undefined') {
		const override = (window as unknown as { __KESH_RETRY_DELAYS?: readonly number[] })
			.__KESH_RETRY_DELAYS;
		if (Array.isArray(override)) return override;
	}
	return DEGRADED_RETRY_DELAYS_MS;
}

/**
 * Helper retry partagé entre `request<T>` et `requestRaw`. Enveloppe
 * uniquement la tentative `fetch + timeout` (le refresh 401, le parsing
 * d'erreur, et la résolution du body restent dans le caller).
 *
 * - **Méthodes idempotentes (GET, HEAD)** : retry sur NETWORK_ERROR / TIMEOUT
 *   / response 503 jusqu'à épuisement des délais ; à l'épuisement, throw
 *   l'ApiError de la dernière tentative (ou retourne la 503 pour que le
 *   caller la traite via `parseErrorResponse`).
 * - **Méthodes non-idempotentes (POST/PUT/PATCH/DELETE)** : aucun retry —
 *   throw l'ApiError immédiatement (ou retourne la 503 pour le caller).
 * - **Pilotage banner** : `apiHealth.setDegraded()` au 1er échec retry-eligible
 *   (même pour les non-retryables) ; `clearDegraded()` au 1er succès de fetch
 *   ne retournant pas 503 (réponse 200, 4xx, 5xx-non-503 → la DB répond).
 */
async function fetchWithRetry(url: string, init: RequestInit): Promise<Response> {
	const idempotent = isIdempotentMethod(init.method);
	const delays = getRetryDelays();
	const maxAttempts = idempotent ? delays.length + 1 : 1;

	for (let attempt = 0; attempt < maxAttempts; attempt++) {
		try {
			const res = await fetchWithTimeout(url, init);
			if (res.status === 503) {
				apiHealth.setDegraded();
				if (attempt < maxAttempts - 1) {
					// Story 10.3 F7 (Pass 1 code review) : drain le body 503 abandonné
					// avant le retry pour libérer la connexion HTTP/1.1 (sinon le browser
					// la garde réservée jusqu'au TCP idle → réduit le pool effectif de 6
					// connexions/origine pendant la fenêtre de sleep + retry suivants).
					await res.body?.cancel().catch(() => {
						// body déjà consommé ou pas de body — no-op
					});
					await sleep(delays[attempt]);
					continue;
				}
				// Give-up : retourne la 503 → caller la traitera via parseErrorResponse
				return res;
			}
			// Succès end-to-end (res.ok = 2xx) → la DB a forcément répondu pour
			// produire ce statut → masquer le banner si actif. Pass 3 H1 : on
			// **NE PAS** clearDegraded sur les non-2xx (401, 403, 4xx, 5xx-non-503)
			// car ces statuts peuvent être produits AVANT tout accès DB (JWT
			// middleware in-memory pour 401, RBAC claims pour 403, body parse
			// pour 422, sqlx::Error::Protocol/WorkerCrashed mappé vers 500 etc.) —
			// ne prouvent rien sur la santé DB. `clearDegraded()` est idempotent
			// (cf. `api-health.svelte.ts:63` early-return si !_isDegraded).
			if (res.ok) {
				apiHealth.clearDegraded();
			}
			return res;
		} catch (err) {
			apiHealth.setDegraded();
			if (attempt < maxAttempts - 1) {
				await sleep(delays[attempt]);
				continue;
			}
			throw toFetchApiError(err);
		}
	}

	// Pass 2 BH2-L1 + ECH2-1 / Pass 4 L1 : la boucle for-loop a un invariant :
	// chaque itération `return` OU `throw` avant l'incrément naturel. Ce
	// `throw` est requis pour le type-narrowing TypeScript (TS2366 sinon —
	// la boucle for-loop n'est pas exhaustive aux yeux du compiler même si
	// elle l'est à l'exécution). Jamais exécuté en pratique.
	throw toFetchApiError(new Error('unreachable'));
}

/**
 * Exécute une requête fetch avec gestion des erreurs et du refresh token.
 *
 * @param url - URL relative (ex: `/api/v1/users`)
 * @param options - Options fetch (method, body, etc.)
 * @param isRetry - Guard anti-boucle 401-refresh : si `true`, ne pas retenter le refresh
 * @returns La réponse parsée en JSON, typée `T`
 * @throws {ApiError} En cas d'erreur HTTP ou réseau
 */
async function request<T>(url: string, options: RequestInit = {}, isRetry = false): Promise<T> {
	const headers = buildHeaders(
		url,
		options.headers as Record<string, string> | undefined,
		options.body ?? null,
	);

	const res = await fetchWithRetry(url, { ...options, headers });

	// 401 sur une URL non-auth → tenter un refresh
	if (
		res.status === 401 &&
		!isRetry &&
		!AUTH_EXCLUDED_URLS.some((excluded) => url.startsWith(excluded))
	) {
		const refreshed = await refreshTokens();
		if (refreshed) {
			// Retry avec le nouveau token (isRetry = true pour le guard anti-boucle)
			return request<T>(url, options, true);
		}
		// refreshTokens() a déjà fait clearSession + redirect si échec
		const error: ApiError = {
			code: 'UNAUTHENTICATED',
			message: 'Session expirée',
			status: 401,
		};
		throw error;
	}

	if (!res.ok) {
		throw await parseErrorResponse(res);
	}

	// 204 No Content — pas de body
	if (res.status === 204) {
		return undefined as T;
	}

	try {
		return (await res.json()) as T;
	} catch {
		return undefined as T;
	}
}

/**
 * Variante binaire de `request()` — retourne la `Response` brute.
 *
 * Utilisée pour télécharger des blobs (PDF, exports CSV) : applique la même
 * logique JWT + refresh 401 + retry exponentiel + parsing erreurs 4xx/5xx en
 * `ApiError`, mais ne parse pas le body en JSON. Le caller appelle `.blob()`
 * / `.arrayBuffer()`.
 *
 * @throws {ApiError} En cas d'erreur HTTP ou réseau.
 */
async function requestRaw(
	url: string,
	options: RequestInit = {},
	isRetry = false,
): Promise<Response> {
	const headers = buildHeaders(url, options.headers as Record<string, string> | undefined);

	const res = await fetchWithRetry(url, { ...options, headers });

	if (
		res.status === 401 &&
		!isRetry &&
		!AUTH_EXCLUDED_URLS.some((excluded) => url.startsWith(excluded))
	) {
		const refreshed = await refreshTokens();
		if (refreshed) {
			return requestRaw(url, options, true);
		}
		const error: ApiError = {
			code: 'UNAUTHENTICATED',
			message: 'Session expirée',
			status: 401,
		};
		throw error;
	}

	if (!res.ok) {
		throw await parseErrorResponse(res);
	}

	return res;
}

export const apiClient = {
	get<T>(url: string): Promise<T> {
		return request<T>(url, { method: 'GET' });
	},

	/** Téléchargement binaire : retourne la `Response` brute (appelez `.blob()`). */
	getBlob(url: string): Promise<Response> {
		return requestRaw(url, { method: 'GET' });
	},

	post<T>(url: string, body?: unknown): Promise<T> {
		return request<T>(url, {
			method: 'POST',
			body: body !== undefined ? JSON.stringify(body) : undefined,
		});
	},

	/**
	 * POST avec body `FormData` (Story 8-1b §api-client) — le browser pose
	 * automatiquement `Content-Type: multipart/form-data; boundary=...`.
	 * Utilisé pour l'upload bank-import (XML CAMT.053).
	 */
	postFormData<T>(url: string, form: FormData): Promise<T> {
		return request<T>(url, { method: 'POST', body: form });
	},

	put<T>(url: string, body?: unknown): Promise<T> {
		return request<T>(url, {
			method: 'PUT',
			body: body !== undefined ? JSON.stringify(body) : undefined,
		});
	},

	/**
	 * PATCH JSON — Story 8-5a-zero (`PATCH /api/v1/bank-accounts/{id}`).
	 * Aligné sur `put` et `post` : Content-Type application/json, JWT auto-injecté.
	 */
	patch<T>(url: string, body?: unknown): Promise<T> {
		return request<T>(url, {
			method: 'PATCH',
			body: body !== undefined ? JSON.stringify(body) : undefined,
		});
	},

	delete(url: string): Promise<void> {
		return request<void>(url, { method: 'DELETE' });
	},
};
