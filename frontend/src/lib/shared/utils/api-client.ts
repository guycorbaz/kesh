/**
 * Client API – wrapper autour de `fetch()` natif.
 *
 * Fonctionnalités :
 * 1. Injection automatique du header Authorization (sauf login/logout/refresh)
 * 2. Interception 401 → refresh token → retry transparent
 * 3. Mutex de refresh (une seule tentative simultanée)
 * 4. Parsing erreurs structurées → ApiError
 */

import type { ApiError } from '$lib/shared/types/api';
import { authState } from '$lib/app/stores/auth.svelte';

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

/** Promise partagée pour le mutex de refresh. */
let refreshPromise: Promise<boolean> | null = null;

/**
 * Tente un refresh des tokens via POST /api/v1/auth/refresh.
 * Retourne `true` si le refresh a réussi, `false` sinon.
 * En cas d'échec : clearSession() + redirect login.
 */
async function doRefresh(): Promise<boolean> {
	const currentRefreshToken = authState.refreshToken;
	if (!currentRefreshToken) {
		authState.clearSession();
		window.location.replace('/login?reason=session_expired');
		return false;
	}

	let res: Response;
	try {
		res = await fetch('/api/v1/auth/refresh', {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({ refreshToken: currentRefreshToken }),
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
		if (
			typeof data?.accessToken !== 'string' ||
			typeof data?.refreshToken !== 'string' ||
			typeof data?.expiresIn !== 'number'
		) {
			authState.clearSession();
			window.location.replace('/login?reason=session_expired');
			return false;
		}
		authState.login(data.accessToken, data.refreshToken, data.expiresIn);
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
 * Ajoute Authorization sauf pour les URLs d'auth exclues.
 */
function buildHeaders(url: string, customHeaders?: Record<string, string>): Record<string, string> {
	const headers: Record<string, string> = {
		'Content-Type': 'application/json',
		...customHeaders,
	};

	if (!AUTH_EXCLUDED_URLS.some((excluded) => url.startsWith(excluded)) && authState.accessToken) {
		headers['Authorization'] = `Bearer ${authState.accessToken}`;
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

/**
 * Exécute une requête fetch avec gestion des erreurs et du refresh token.
 *
 * @param url - URL relative (ex: `/api/v1/users`)
 * @param options - Options fetch (method, body, etc.)
 * @param isRetry - Guard anti-boucle : si `true`, ne pas retenter le refresh
 * @returns La réponse parsée en JSON, typée `T`
 * @throws {ApiError} En cas d'erreur HTTP ou réseau
 */
/** Timeout par défaut pour les requêtes fetch (30 secondes). */
const REQUEST_TIMEOUT_MS = 30_000;

async function request<T>(url: string, options: RequestInit = {}, isRetry = false): Promise<T> {
	const headers = buildHeaders(url, options.headers as Record<string, string> | undefined);

	const controller = new AbortController();
	const timeout = setTimeout(() => controller.abort(), REQUEST_TIMEOUT_MS);

	let res: Response;
	try {
		res = await fetch(url, { ...options, headers, signal: controller.signal });
	} catch (err) {
		const isTimeout = err instanceof DOMException && err.name === 'AbortError';
		const error: ApiError = {
			code: isTimeout ? 'TIMEOUT' : 'NETWORK_ERROR',
			message: isTimeout
				? 'Le serveur ne répond pas. Réessayez ultérieurement.'
				: 'Impossible de contacter le serveur. Vérifiez votre connexion.',
			status: 0,
		};
		throw error;
	} finally {
		clearTimeout(timeout);
	}

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

export const apiClient = {
	get<T>(url: string): Promise<T> {
		return request<T>(url, { method: 'GET' });
	},

	post<T>(url: string, body?: unknown): Promise<T> {
		return request<T>(url, {
			method: 'POST',
			body: body !== undefined ? JSON.stringify(body) : undefined,
		});
	},

	put<T>(url: string, body?: unknown): Promise<T> {
		return request<T>(url, {
			method: 'PUT',
			body: body !== undefined ? JSON.stringify(body) : undefined,
		});
	},

	delete(url: string): Promise<void> {
		return request<void>(url, { method: 'DELETE' });
	},
};
