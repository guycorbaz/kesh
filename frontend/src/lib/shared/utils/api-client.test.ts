import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
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
		Object.defineProperty(window, 'location', {
			value: originalLocation,
			writable: true,
			configurable: true,
		});
	});

	// --- HTTP methods with Authorization header ---

	describe('requêtes avec Authorization header', () => {
		beforeEach(() => {
			authState.login(VALID_TOKEN, 'refresh-uuid', 900);
		});

		it('GET ajoute le header Authorization', async () => {
			mockFetch.mockResolvedValue(mockResponse(200, [{ id: 1 }]));

			await apiClient.get('/api/v1/users');

			expect(mockFetch).toHaveBeenCalledWith(
				'/api/v1/users',
				expect.objectContaining({
					method: 'GET',
					headers: expect.objectContaining({
						Authorization: `Bearer ${VALID_TOKEN}`,
					}),
				}),
			);
		});

		it('POST ajoute le header Authorization', async () => {
			mockFetch.mockResolvedValue(mockResponse(200, { id: 2 }));

			await apiClient.post('/api/v1/users', { username: 'test' });

			expect(mockFetch).toHaveBeenCalledWith(
				'/api/v1/users',
				expect.objectContaining({
					method: 'POST',
					headers: expect.objectContaining({
						Authorization: `Bearer ${VALID_TOKEN}`,
					}),
					body: JSON.stringify({ username: 'test' }),
				}),
			);
		});

		it('PUT ajoute le header Authorization', async () => {
			mockFetch.mockResolvedValue(mockResponse(200, { id: 1 }));

			await apiClient.put('/api/v1/users/1', { role: 'Comptable' });

			expect(mockFetch).toHaveBeenCalledWith(
				'/api/v1/users/1',
				expect.objectContaining({
					method: 'PUT',
					headers: expect.objectContaining({
						Authorization: `Bearer ${VALID_TOKEN}`,
					}),
					body: JSON.stringify({ role: 'Comptable' }),
				}),
			);
		});

		it('DELETE ajoute le header Authorization', async () => {
			mockFetch.mockResolvedValue(mockResponse(204));

			await apiClient.delete('/api/v1/users/1');

			expect(mockFetch).toHaveBeenCalledWith(
				'/api/v1/users/1',
				expect.objectContaining({
					method: 'DELETE',
					headers: expect.objectContaining({
						Authorization: `Bearer ${VALID_TOKEN}`,
					}),
				}),
			);
		});

		it('n\'ajoute PAS Authorization sur /api/v1/auth/login', async () => {
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
			authState.login(VALID_TOKEN, 'old-refresh', 900);

			// 1er appel → 401
			// 2e appel → refresh OK
			// 3e appel → retry OK
			mockFetch
				.mockResolvedValueOnce(mockResponse(401, { error: { code: 'UNAUTHENTICATED', message: 'Token expiré' } }))
				.mockResolvedValueOnce(
					mockResponse(200, {
						accessToken: NEW_TOKEN,
						refreshToken: 'new-refresh',
						expiresIn: 900,
					}),
				)
				.mockResolvedValueOnce(mockResponse(200, { data: 'success' }));

			const result = await apiClient.get<{ data: string }>('/api/v1/users');

			expect(result).toEqual({ data: 'success' });
			expect(mockFetch).toHaveBeenCalledTimes(3);

			// Vérifier que le refresh a été appelé
			expect(mockFetch.mock.calls[1][0]).toBe('/api/v1/auth/refresh');
		});

		it('refresh échoué → clearSession + redirect login', async () => {
			authState.login(VALID_TOKEN, 'old-refresh', 900);

			// 1er appel → 401
			// 2e appel → refresh 401 (INVALID_REFRESH_TOKEN)
			mockFetch
				.mockResolvedValueOnce(mockResponse(401, { error: { code: 'UNAUTHENTICATED', message: '' } }))
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

		it('mutex de refresh — 2 requêtes 401 simultanées n\'appellent refresh qu\'une fois', async () => {
			authState.login(VALID_TOKEN, 'old-refresh', 900);

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

			await Promise.all([
				apiClient.get('/api/v1/users'),
				apiClient.get('/api/v1/accounts'),
			]);

			expect(refreshCallCount).toBe(1);
		});
	});

	// --- Refresh 200 avec JSON malformé ---

	describe('refresh réponse malformée', () => {
		it('refresh 200 sans accessToken → clearSession + redirect', async () => {
			authState.login(VALID_TOKEN, 'old-refresh', 900);

			mockFetch
				.mockResolvedValueOnce(mockResponse(401, { error: { code: 'UNAUTHENTICATED', message: '' } }))
				// Refresh retourne 200 mais sans accessToken
				.mockResolvedValueOnce(mockResponse(200, { refreshToken: 'x', expiresIn: 900 }));

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
			authState.login(VALID_TOKEN, 'r', 900);
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
			authState.login(VALID_TOKEN, 'r', 900);
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
			authState.login(VALID_TOKEN, 'r', 900);
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
			authState.login(VALID_TOKEN, 'r', 900);
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
			authState.login(VALID_TOKEN, 'r', 900);
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
			authState.login(VALID_TOKEN, 'r', 900);
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
			authState.login(VALID_TOKEN, 'r', 900);
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
			authState.login(VALID_TOKEN, 'old-refresh', 900);

			mockFetch
				// 1er appel → 401
				.mockResolvedValueOnce(mockResponse(401, { error: { code: 'UNAUTHENTICATED', message: '' } }))
				// Refresh → OK
				.mockResolvedValueOnce(
					mockResponse(200, {
						accessToken: NEW_TOKEN,
						refreshToken: 'new-refresh',
						expiresIn: 900,
					}),
				)
				// Retry → encore 401
				.mockResolvedValueOnce(mockResponse(401, { error: { code: 'UNAUTHENTICATED', message: '' } }));

			await expect(apiClient.get('/api/v1/users')).rejects.toMatchObject({
				code: 'UNAUTHENTICATED',
				status: 401,
			});

			// Doit avoir fait 3 appels : requête initiale, refresh, retry
			// PAS de 2e refresh
			expect(mockFetch).toHaveBeenCalledTimes(3);
			const urls = mockFetch.mock.calls.map((c: unknown[]) => c[0] as string);
			expect(urls.filter((u) => u === '/api/v1/auth/refresh')).toHaveLength(1);
		});
	});

	// --- Réponse non-JSON ---

	describe('réponse non-JSON', () => {
		it('réponse HTML erreur proxy → ApiError UNKNOWN_ERROR', async () => {
			authState.login(VALID_TOKEN, 'r', 900);
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
			authState.login(VALID_TOKEN, 'r', 900);
			expect(authState.isAuthenticated).toBe(true);

			authState.clearSession();

			expect(authState.isAuthenticated).toBe(false);
			expect(authState.accessToken).toBeNull();
			expect(authState.refreshToken).toBeNull();
			expect(mockFetch).not.toHaveBeenCalled();
		});
	});

	// --- Refresh réseau injoignable ---

	describe('refresh réseau injoignable', () => {
		it('refresh fetch échoue (réseau) → clearSession + redirect', async () => {
			authState.login(VALID_TOKEN, 'old-refresh', 900);

			mockFetch
				.mockResolvedValueOnce(mockResponse(401, { error: { code: 'UNAUTHENTICATED', message: '' } }))
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
});
