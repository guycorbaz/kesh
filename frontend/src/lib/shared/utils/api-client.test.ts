import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';

// Mock apiHealth store BEFORE importing api-client (hoisted by vi).
// L'état isDegraded est toggled par les fn mock pour permettre aux tests de
// vérifier la séquence setDegraded → clearDegraded (les anciens tests 401,
// timeout, etc. ne l'observent pas — c'est silencieux).
const { mockApiHealth, resetApiHealthMock } = vi.hoisted(() => {
	const state = { isDegraded: false };
	const setDegraded = vi.fn(() => {
		state.isDegraded = true;
	});
	const clearDegraded = vi.fn(() => {
		state.isDegraded = false;
	});
	return {
		mockApiHealth: {
			get isDegraded(): boolean {
				return state.isDegraded;
			},
			setDegraded,
			clearDegraded,
		},
		resetApiHealthMock: (): void => {
			state.isDegraded = false;
			setDegraded.mockClear();
			clearDegraded.mockClear();
		},
	};
});

vi.mock('$lib/shared/utils/api-health.svelte', () => ({
	apiHealth: mockApiHealth,
	HEALTH_POLL_INTERVAL_MS: 5000,
}));

import { apiClient, isApiError } from './api-client';
import { authState } from '$lib/app/stores/auth.svelte';

/** Fabrique un JWT factice en base64url. */
function fakeJwt(payload: Record<string, unknown>): string {
	const toBase64Url = (obj: unknown) =>
		btoa(JSON.stringify(obj))
			.replace(/\+/g, '-')
			.replace(/\//g, '_')
			.replace(/=+$/, '');
	return `${toBase64Url({ alg: 'HS256', typ: 'JWT' })}.${toBase64Url(payload)}.fake-sig`;
}

const VALID_TOKEN = fakeJwt({ sub: '1', role: 'Admin', exp: 9999999999 });
const NEW_TOKEN = fakeJwt({ sub: '1', role: 'Admin', exp: 9999999999 });

/** Crée une Response factice. */
function mockResponse(status: number, body?: unknown, ok?: boolean): Response {
	return {
		ok: ok ?? (status >= 200 && status < 300),
		status,
		json: () => Promise.resolve(body),
		headers: new Headers(),
	} as Response;
}

/** Crée une Response factice dont json() échoue (non-JSON). */
function mockNonJsonResponse(status: number): Response {
	return {
		ok: false,
		status,
		json: () => Promise.reject(new Error('not json')),
		headers: new Headers(),
	} as Response;
}

describe('apiClient', () => {
	let mockFetch: ReturnType<typeof vi.fn>;
	let locationReplaceMock: ReturnType<typeof vi.fn>;
	const originalLocation = window.location;

	beforeEach(() => {
		// Reset auth state
		authState.clearSession();

		// Reset apiHealth mock state
		resetApiHealthMock();

		// Désactiver le retry exponentiel par défaut (1 seule tentative). Les
		// tests Story 10.3 qui exercent le retry overrident explicitement.
		(window as unknown as { __KESH_RETRY_DELAYS: readonly number[] }).__KESH_RETRY_DELAYS = [];

		// Mock fetch
		mockFetch = vi.fn();
		vi.stubGlobal('fetch', mockFetch);

		// Mock window.location.replace
		locationReplaceMock = vi.fn();
		Object.defineProperty(window, 'location', {
			value: { ...originalLocation, replace: locationReplaceMock },
			writable: true,
			configurable: true,
		});
	});

	afterEach(() => {
		vi.restoreAllMocks();
		vi.useRealTimers();
		delete (window as unknown as { __KESH_RETRY_DELAYS?: readonly number[] }).__KESH_RETRY_DELAYS;
		Object.defineProperty(window, 'location', {
			value: originalLocation,
			writable: true,
			configurable: true,
		});
	});

	// --- HTTP methods with Authorization header ---

	describe("requêtes avec credentials: 'include' (Story 10-5 — cookie HttpOnly remplace Authorization Bearer)", () => {
		// Pré-Story 10-5 : 4 tests "GET/POST/PUT/DELETE ajoute le header Authorization".
		// Post-Story 10-5 (T7.1/T7.2 D5 acté) : le browser envoie automatiquement
		// le cookie HttpOnly `kesh_access_token` via `credentials: 'include'`.
		// L'header Authorization n'est plus injecté par le frontend.
		beforeEach(() => {
			authState.login({ userId: '1', username: 'test', role: 'Admin', expiresIn: 900 });
		});

		it("GET inclut credentials: 'include' et n'ajoute PAS Authorization", async () => {
			mockFetch.mockResolvedValue(mockResponse(200, [{ id: 1 }]));

			await apiClient.get('/api/v1/users');

			expect(mockFetch).toHaveBeenCalledWith(
				'/api/v1/users',
				expect.objectContaining({
					method: 'GET',
					credentials: 'include',
				}),
			);
			const callOpts = mockFetch.mock.calls[0][1];
			expect(callOpts.headers).not.toHaveProperty('Authorization');
		});

		it("POST inclut credentials: 'include' et n'ajoute PAS Authorization", async () => {
			mockFetch.mockResolvedValue(mockResponse(200, { id: 2 }));

			await apiClient.post('/api/v1/users', { username: 'test' });

			expect(mockFetch).toHaveBeenCalledWith(
				'/api/v1/users',
				expect.objectContaining({
					method: 'POST',
					credentials: 'include',
					body: JSON.stringify({ username: 'test' }),
				}),
			);
			const callOpts = mockFetch.mock.calls[0][1];
			expect(callOpts.headers).not.toHaveProperty('Authorization');
		});

		it("PUT inclut credentials: 'include' et n'ajoute PAS Authorization", async () => {
			mockFetch.mockResolvedValue(mockResponse(200, { id: 1 }));

			await apiClient.put('/api/v1/users/1', { role: 'Comptable' });

			expect(mockFetch).toHaveBeenCalledWith(
				'/api/v1/users/1',
				expect.objectContaining({
					method: 'PUT',
					credentials: 'include',
					body: JSON.stringify({ role: 'Comptable' }),
				}),
			);
			const callOpts = mockFetch.mock.calls[0][1];
			expect(callOpts.headers).not.toHaveProperty('Authorization');
		});

		it("DELETE inclut credentials: 'include' et n'ajoute PAS Authorization", async () => {
			mockFetch.mockResolvedValue(mockResponse(204));

			await apiClient.delete('/api/v1/users/1');

			expect(mockFetch).toHaveBeenCalledWith(
				'/api/v1/users/1',
				expect.objectContaining({
					method: 'DELETE',
					credentials: 'include',
				}),
			);
			const callOpts = mockFetch.mock.calls[0][1];
			expect(callOpts.headers).not.toHaveProperty('Authorization');
		});

		it("n'ajoute PAS Authorization sur /api/v1/auth/login", async () => {
			mockFetch.mockResolvedValue(
				mockResponse(200, { accessToken: VALID_TOKEN, refreshToken: 'r', expiresIn: 900 }),
			);

			await apiClient.post('/api/v1/auth/login', { username: 'u', password: 'p' });

			const headers = mockFetch.mock.calls[0][1].headers;
			expect(headers).not.toHaveProperty('Authorization');
		});
	});

	// --- Refresh automatique sur 401 ---

	describe('refresh automatique sur 401', () => {
		it('refresh et retry sur 401', async () => {
			authState.login({ userId: '1', username: 'test', role: 'Admin', expiresIn: 900 });

			// 1er appel → 401 sur /api/v1/users
			// 2e appel → refresh OK
			// 3e appel → CR Pass 3 BH3-M2 : doRefresh re-fetch /me pour resync
			//            `_currentUser` post-refresh (le cookie HttpOnly bloque le
			//            décodage JWT côté JS, donc re-fetch est le seul moyen
			//            d'obtenir le role courant côté frontend).
			// 4e appel → retry OK sur /api/v1/users
			mockFetch
				.mockResolvedValueOnce(
					mockResponse(401, { error: { code: 'UNAUTHENTICATED', message: 'Token expiré' } }),
				)
				.mockResolvedValueOnce(
					mockResponse(200, {
						accessToken: NEW_TOKEN,
						refreshToken: 'new-refresh',
						expiresIn: 900,
					}),
				)
				.mockResolvedValueOnce(
					mockResponse(200, {
						userId: 1,
						username: 'test',
						role: 'Admin',
						expiresIn: 900,
					}),
				)
				.mockResolvedValueOnce(mockResponse(200, { data: 'success' }));

			const result = await apiClient.get<{ data: string }>('/api/v1/users');

			expect(result).toEqual({ data: 'success' });
			expect(mockFetch).toHaveBeenCalledTimes(4);

			// Vérifier la séquence : initial 401, refresh, /me re-sync, retry.
			expect(mockFetch.mock.calls[1][0]).toBe('/api/v1/auth/refresh');
			expect(mockFetch.mock.calls[2][0]).toBe('/api/v1/auth/me');
		});

		it('refresh échoué → clearSession + redirect login', async () => {
			authState.login({ userId: '1', username: 'test', role: 'Admin', expiresIn: 900 });

			// 1er appel → 401
			// 2e appel → refresh 401 (INVALID_REFRESH_TOKEN)
			mockFetch
				.mockResolvedValueOnce(
					mockResponse(401, { error: { code: 'UNAUTHENTICATED', message: '' } }),
				)
				.mockResolvedValueOnce(
					mockResponse(401, {
						error: { code: 'INVALID_REFRESH_TOKEN', message: 'Session expirée' },
					}),
				);

			await expect(apiClient.get('/api/v1/users')).rejects.toMatchObject({
				code: 'UNAUTHENTICATED',
				status: 401,
			});

			expect(authState.isAuthenticated).toBe(false);
			expect(locationReplaceMock).toHaveBeenCalledWith('/login?reason=session_expired');
		});

		it("mutex de refresh — 2 requêtes 401 simultanées n'appellent refresh qu'une fois", async () => {
			authState.login({ userId: '1', username: 'test', role: 'Admin', expiresIn: 900 });

			let refreshCallCount = 0;

			mockFetch.mockImplementation((url: string) => {
				if (url === '/api/v1/auth/refresh') {
					refreshCallCount++;
					return Promise.resolve(
						mockResponse(200, {
							accessToken: NEW_TOKEN,
							refreshToken: 'new-refresh',
							expiresIn: 900,
						}),
					);
				}
				// Les deux premières requêtes retournent 401, les retry 200
				if (mockFetch.mock.calls.filter((c: string[]) => c[0] === url).length <= 1) {
					return Promise.resolve(
						mockResponse(401, { error: { code: 'UNAUTHENTICATED', message: '' } }),
					);
				}
				return Promise.resolve(mockResponse(200, { ok: true }));
			});

			await Promise.all([apiClient.get('/api/v1/users'), apiClient.get('/api/v1/accounts')]);

			expect(refreshCallCount).toBe(1);
		});
	});

	// --- Refresh 200 avec JSON malformé ---

	describe('refresh réponse malformée (Story 10-5 — valide uniquement expiresIn)', () => {
		// Pré-Story 10-5 : doRefresh validait accessToken + refreshToken + expiresIn.
		// Post-Story 10-5 (T7.3c) : seul `expiresIn` est validé (les tokens sont
		// dans les cookies HttpOnly émis par le backend via Set-Cookie, pas en body
		// JS-accessible). Test adapté pour le nouveau contrat.
		it('refresh 200 sans expiresIn → clearSession + redirect', async () => {
			authState.login({ userId: '1', username: 'test', role: 'Admin', expiresIn: 900 });

			mockFetch
				.mockResolvedValueOnce(
					mockResponse(401, { error: { code: 'UNAUTHENTICATED', message: '' } }),
				)
				// Refresh retourne 200 mais sans expiresIn (body malformé).
				.mockResolvedValueOnce(mockResponse(200, { accessToken: 'x', refreshToken: 'r' }));

			await expect(apiClient.get('/api/v1/users')).rejects.toMatchObject({
				code: 'UNAUTHENTICATED',
				status: 401,
			});

			expect(authState.isAuthenticated).toBe(false);
			expect(locationReplaceMock).toHaveBeenCalledWith('/login?reason=session_expired');
		});
	});

	// --- Timeout ---

	describe('timeout', () => {
		it('requête qui dépasse le timeout → ApiError TIMEOUT', async () => {
			authState.login({ userId: '1', username: 'test', role: 'Admin', expiresIn: 900 });
			mockFetch.mockImplementation(
				() => new Promise(() => {}), // never resolves
			);

			// Simuler l'abort du controller
			const abortError = new DOMException('The operation was aborted.', 'AbortError');
			mockFetch.mockRejectedValue(abortError);

			await expect(apiClient.get('/api/v1/users')).rejects.toMatchObject({
				code: 'TIMEOUT',
				status: 0,
			});
		});
	});

	// --- Parsing erreurs structurées ---

	describe('parsing erreurs structurées', () => {
		it('parse une erreur structurée → ApiError', async () => {
			authState.login({ userId: '1', username: 'test', role: 'Admin', expiresIn: 900 });
			mockFetch.mockResolvedValue(
				mockResponse(403, {
					error: { code: 'FORBIDDEN', message: 'Accès refusé' },
				}),
			);

			await expect(apiClient.get('/api/v1/admin')).rejects.toMatchObject({
				code: 'FORBIDDEN',
				message: 'Accès refusé',
				status: 403,
			});
		});

		it('parse une erreur avec details', async () => {
			authState.login({ userId: '1', username: 'test', role: 'Admin', expiresIn: 900 });
			mockFetch.mockResolvedValue(
				mockResponse(400, {
					error: {
						code: 'VALIDATION_ERROR',
						message: 'Validation échouée',
						details: { username: 'trop court' },
					},
				}),
			);

			await expect(apiClient.post('/api/v1/users', {})).rejects.toMatchObject({
				code: 'VALIDATION_ERROR',
				details: { username: 'trop court' },
				status: 400,
			});
		});
	});

	// --- Réseau injoignable ---

	describe('réseau injoignable', () => {
		it('fetch qui throw → ApiError NETWORK_ERROR', async () => {
			authState.login({ userId: '1', username: 'test', role: 'Admin', expiresIn: 900 });
			mockFetch.mockRejectedValue(new TypeError('Failed to fetch'));

			await expect(apiClient.get('/api/v1/users')).rejects.toMatchObject({
				code: 'NETWORK_ERROR',
				status: 0,
			});
		});
	});

	// --- 503 SERVICE_UNAVAILABLE ---

	describe('503 SERVICE_UNAVAILABLE', () => {
		it('parse 503 avec message DB → ApiError SERVICE_UNAVAILABLE', async () => {
			authState.login({ userId: '1', username: 'test', role: 'Admin', expiresIn: 900 });
			mockFetch.mockResolvedValue(
				mockResponse(503, {
					error: {
						code: 'SERVICE_UNAVAILABLE',
						message: 'Serveur indisponible — vérifiez que la base de données est accessible',
					},
				}),
			);

			await expect(apiClient.get('/api/v1/users')).rejects.toMatchObject({
				code: 'SERVICE_UNAVAILABLE',
				message: 'Serveur indisponible — vérifiez que la base de données est accessible',
				status: 503,
			});
		});
	});

	// --- 401 sur URL exclue (login) — pas de refresh ---

	describe('401 sur URL auth exclue', () => {
		it('401 INVALID_CREDENTIALS sur /auth/login → pas de refresh, erreur retournée', async () => {
			authState.login({ userId: '1', username: 'test', role: 'Admin', expiresIn: 900 });
			mockFetch.mockResolvedValue(
				mockResponse(401, {
					error: { code: 'INVALID_CREDENTIALS', message: 'Identifiants invalides' },
				}),
			);

			await expect(
				apiClient.post('/api/v1/auth/login', { username: 'u', password: 'p' }),
			).rejects.toMatchObject({
				code: 'INVALID_CREDENTIALS',
				status: 401,
			});

			// Un seul appel fetch — pas de refresh
			expect(mockFetch).toHaveBeenCalledTimes(1);
		});
	});

	// --- 429 RATE_LIMITED ---

	describe('429 RATE_LIMITED', () => {
		it('parse 429 RATE_LIMITED → ApiError', async () => {
			authState.login({ userId: '1', username: 'test', role: 'Admin', expiresIn: 900 });
			mockFetch.mockResolvedValue(
				mockResponse(429, {
					error: { code: 'RATE_LIMITED', message: 'Trop de tentatives' },
				}),
			);

			await expect(
				apiClient.post('/api/v1/auth/login', { username: 'u', password: 'p' }),
			).rejects.toMatchObject({
				code: 'RATE_LIMITED',
				message: 'Trop de tentatives',
				status: 429,
			});
		});
	});

	// --- Guard anti-boucle infinie ---

	describe('guard anti-boucle', () => {
		it('retry après refresh retourne 401 → clearSession, pas de 2e refresh', async () => {
			authState.login({ userId: '1', username: 'test', role: 'Admin', expiresIn: 900 });

			mockFetch
				// 1er appel → 401 sur /api/v1/users
				.mockResolvedValueOnce(
					mockResponse(401, { error: { code: 'UNAUTHENTICATED', message: '' } }),
				)
				// Refresh → OK
				.mockResolvedValueOnce(
					mockResponse(200, {
						accessToken: NEW_TOKEN,
						refreshToken: 'new-refresh',
						expiresIn: 900,
					}),
				)
				// CR Pass 3 BH3-M2 : doRefresh re-fetch /me post-refresh pour resync
				// `_currentUser`. Le re-fetch /me est best-effort — log warn si non-OK
				// mais ne fait pas échouer le refresh.
				.mockResolvedValueOnce(
					mockResponse(200, {
						userId: 1,
						username: 'test',
						role: 'Admin',
						expiresIn: 900,
					}),
				)
				// Retry sur /api/v1/users → encore 401
				.mockResolvedValueOnce(
					mockResponse(401, { error: { code: 'UNAUTHENTICATED', message: '' } }),
				);

			await expect(apiClient.get('/api/v1/users')).rejects.toMatchObject({
				code: 'UNAUTHENTICATED',
				status: 401,
			});

			// Doit avoir fait 4 appels : requête initiale, refresh, /me re-sync, retry.
			// PAS de 2e refresh (le guard anti-boucle empêche un nouveau cycle refresh
			// après que le retry ait re-échoué).
			expect(mockFetch).toHaveBeenCalledTimes(4);
			const urls = mockFetch.mock.calls.map((c: unknown[]) => c[0] as string);
			expect(urls.filter((u) => u === '/api/v1/auth/refresh')).toHaveLength(1);
		});
	});

	// --- Réponse non-JSON ---

	describe('réponse non-JSON', () => {
		it('réponse HTML erreur proxy → ApiError UNKNOWN_ERROR', async () => {
			authState.login({ userId: '1', username: 'test', role: 'Admin', expiresIn: 900 });
			mockFetch.mockResolvedValue(mockNonJsonResponse(502));

			await expect(apiClient.get('/api/v1/users')).rejects.toMatchObject({
				code: 'UNKNOWN_ERROR',
				status: 502,
			});
		});
	});

	// --- clearSession ---

	describe('clearSession', () => {
		it('nettoie le state sans appeler fetch', () => {
			authState.login({ userId: '1', username: 'test', role: 'Admin', expiresIn: 900 });
			expect(authState.isAuthenticated).toBe(true);

			authState.clearSession();

			expect(authState.isAuthenticated).toBe(false);
			expect(authState.currentUser).toBeNull();
			expect(authState.expiresIn).toBeNull();
			expect(mockFetch).not.toHaveBeenCalled();
		});
	});

	// --- Refresh réseau injoignable ---

	describe('refresh réseau injoignable', () => {
		it('refresh fetch échoue (réseau) → clearSession + redirect', async () => {
			authState.login({ userId: '1', username: 'test', role: 'Admin', expiresIn: 900 });

			mockFetch
				.mockResolvedValueOnce(
					mockResponse(401, { error: { code: 'UNAUTHENTICATED', message: '' } }),
				)
				.mockRejectedValueOnce(new TypeError('Failed to fetch'));

			await expect(apiClient.get('/api/v1/users')).rejects.toMatchObject({
				code: 'UNAUTHENTICATED',
			});

			expect(authState.isAuthenticated).toBe(false);
			expect(locationReplaceMock).toHaveBeenCalledWith('/login?reason=session_expired');
		});
	});

	// --- isApiError type guard ---

	describe('isApiError', () => {
		it('retourne true pour un ApiError valide', () => {
			expect(isApiError({ code: 'FORBIDDEN', message: 'Non', status: 403 })).toBe(true);
		});

		it('retourne false pour un Error standard', () => {
			expect(isApiError(new Error('boom'))).toBe(false);
		});

		it('retourne false pour null/undefined/string', () => {
			expect(isApiError(null)).toBe(false);
			expect(isApiError(undefined)).toBe(false);
			expect(isApiError('error')).toBe(false);
		});
	});

	// --- Story 10.3 : retry exponentiel + apiHealth degraded ---

	describe('retry exponentiel + apiHealth (Story 10.3)', () => {
		const FAST_DELAYS = [10, 10, 10, 10] as const;

		beforeEach(() => {
			authState.login({ userId: '1', username: 'test', role: 'Admin', expiresIn: 900 });
			// Active le retry exponentiel avec délais accélérés (10ms × 4)
			(window as unknown as { __KESH_RETRY_DELAYS: readonly number[] }).__KESH_RETRY_DELAYS =
				FAST_DELAYS;
			vi.useFakeTimers();
		});

		afterEach(() => {
			vi.useRealTimers();
		});

		it("(a) GET avec NETWORK_ERROR puis 200 au 2e essai → succès retourné, setDegraded puis clearDegraded", async () => {
			mockFetch
				.mockRejectedValueOnce(new TypeError('Failed to fetch'))
				.mockResolvedValueOnce(mockResponse(200, { data: 'ok' }));

			const promise = apiClient.get<{ data: string }>('/api/v1/users');
			// Avance le 1er délai (10ms) pour déclencher la 2e tentative
			await vi.advanceTimersByTimeAsync(FAST_DELAYS[0]);

			const result = await promise;
			expect(result).toEqual({ data: 'ok' });
			expect(mockFetch).toHaveBeenCalledTimes(2);
			expect(mockApiHealth.setDegraded).toHaveBeenCalled();
			expect(mockApiHealth.clearDegraded).toHaveBeenCalled();
		});

		it("(b) GET avec 5× NETWORK_ERROR → throw NETWORK_ERROR, setDegraded, clearDegraded non-appelé", async () => {
			mockFetch.mockRejectedValue(new TypeError('Failed to fetch'));

			const promise = apiClient.get('/api/v1/users');
			// Détache le rejet pour éviter unhandled rejection pendant les advance
			const expectation = expect(promise).rejects.toMatchObject({
				code: 'NETWORK_ERROR',
				status: 0,
			});
			// Avance les 4 délais de retry pour épuiser les 5 tentatives totales
			for (const delay of FAST_DELAYS) {
				await vi.advanceTimersByTimeAsync(delay);
			}
			await expectation;

			expect(mockFetch).toHaveBeenCalledTimes(5);
			expect(mockApiHealth.setDegraded).toHaveBeenCalled();
			expect(mockApiHealth.clearDegraded).not.toHaveBeenCalled();
		});

		it("(c) POST avec NETWORK_ERROR → throw immédiatement (pas de retry), setDegraded appelé", async () => {
			mockFetch.mockRejectedValue(new TypeError('Failed to fetch'));

			await expect(apiClient.post('/api/v1/users', { name: 'x' })).rejects.toMatchObject({
				code: 'NETWORK_ERROR',
				status: 0,
			});

			expect(mockFetch).toHaveBeenCalledTimes(1);
			expect(mockApiHealth.setDegraded).toHaveBeenCalled();
			expect(mockApiHealth.clearDegraded).not.toHaveBeenCalled();
		});

		it("(d) GET avec 503 puis 200 au 2e essai → retry déclenché, succès", async () => {
			mockFetch
				.mockResolvedValueOnce(
					mockResponse(503, {
						error: { code: 'SERVICE_UNAVAILABLE', message: 'DB down' },
					}),
				)
				.mockResolvedValueOnce(mockResponse(200, { data: 'recovered' }));

			const promise = apiClient.get<{ data: string }>('/api/v1/users');
			await vi.advanceTimersByTimeAsync(FAST_DELAYS[0]);

			const result = await promise;
			expect(result).toEqual({ data: 'recovered' });
			expect(mockFetch).toHaveBeenCalledTimes(2);
			expect(mockApiHealth.setDegraded).toHaveBeenCalled();
			expect(mockApiHealth.clearDegraded).toHaveBeenCalled();
		});

		it("(e) GET avec TIMEOUT puis 200 au 2e essai → retry déclenché, setDegraded, clearDegraded", async () => {
			// Pass 1 code review F2 : couvre le path TIMEOUT (AbortError de fetchWithTimeout)
			// du retry exponentiel. Le path NETWORK_ERROR (tests a, b, c) et 503 (test d)
			// étaient déjà couverts, mais TIMEOUT (DOMException AbortError) partage la même
			// branche `catch (err)` dans `fetchWithRetry` ligne 266 — sans test dédié, une
			// régression future qui exclurait AbortError du retry passerait silencieusement.
			mockFetch
				.mockImplementationOnce(
					() =>
						new Promise((_, reject) => {
							reject(new DOMException('The operation was aborted', 'AbortError'));
						}),
				)
				.mockResolvedValueOnce(mockResponse(200, { data: 'recovered-timeout' }));

			const promise = apiClient.get<{ data: string }>('/api/v1/users');
			await vi.advanceTimersByTimeAsync(FAST_DELAYS[0]);

			const result = await promise;
			expect(result).toEqual({ data: 'recovered-timeout' });
			expect(mockFetch).toHaveBeenCalledTimes(2);
			expect(mockApiHealth.setDegraded).toHaveBeenCalled();
			expect(mockApiHealth.clearDegraded).toHaveBeenCalled();
		});
	});
});
