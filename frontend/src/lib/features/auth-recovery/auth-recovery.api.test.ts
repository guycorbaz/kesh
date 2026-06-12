// Story 17-4d T-D1 — Tests unitaires Vitest du client API auth-recovery.
//
// Vérifie le contrat figé 17-4c :
// - `requestPasswordReset` : POST /api/v1/auth/forgot-password, corps JSON
//   `{ identifier }`, et tolère le `200` à CORPS VIDE (anti-énum DC4 — le
//   backend retourne `StatusCode::OK` sans JSON).
// - `resetPassword` : POST /api/v1/auth/reset-password, corps camelCase
//   `{ token, newPassword }`.
// - Les erreurs structurées backend (`INVALID_OR_EXPIRED_TOKEN`,
//   `VALIDATION_ERROR`, `RATE_LIMITED`) sont propagées en `ApiError` typé.

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { requestPasswordReset, resetPassword } from './auth-recovery.api';
import { isApiError } from '$lib/shared/utils/api-client';

/** Réponse 200 à corps vide : `res.json()` rejette, comme un vrai fetch. */
function emptyOkResponse(): Partial<Response> {
	return {
		ok: true,
		status: 200,
		json: () => Promise.reject(new SyntaxError('Unexpected end of JSON input')),
		headers: new Headers(),
	};
}

/** Réponse d'erreur structurée `{ error: { code, message } }` (errors.rs). */
function errorResponse(status: number, code: string, message: string): Partial<Response> {
	return {
		ok: false,
		status,
		json: () => Promise.resolve({ error: { code, message } }),
		headers: new Headers(),
	};
}

describe('auth-recovery.api', () => {
	let mockFetch: ReturnType<typeof vi.fn>;

	beforeEach(() => {
		mockFetch = vi.fn().mockResolvedValue(emptyOkResponse() as Response);
		vi.stubGlobal('fetch', mockFetch);
	});

	afterEach(() => {
		vi.unstubAllGlobals();
	});

	it('requestPasswordReset POSTe { identifier } sur /api/v1/auth/forgot-password', async () => {
		await requestPasswordReset('jdupont');
		expect(mockFetch).toHaveBeenCalledTimes(1);
		const [url, init] = mockFetch.mock.calls[0] as [string, RequestInit];
		expect(url).toBe('/api/v1/auth/forgot-password');
		expect(init.method).toBe('POST');
		expect(JSON.parse(init.body as string)).toEqual({ identifier: 'jdupont' });
		const headers = init.headers as Record<string, string>;
		expect(headers['Content-Type']).toBe('application/json');
	});

	it('requestPasswordReset résout sur un 200 à corps vide (anti-énum DC4)', async () => {
		// Le mock par défaut rejette `json()` — exactement le comportement d'un
		// `Response` sans corps. La promesse doit résoudre sans throw.
		await expect(requestPasswordReset('inconnu@example.ch')).resolves.toBeUndefined();
	});

	it('requestPasswordReset propage le 429 RATE_LIMITED en ApiError typé', async () => {
		mockFetch.mockResolvedValue(
			errorResponse(429, 'RATE_LIMITED', 'Trop de tentatives') as Response,
		);
		try {
			await requestPasswordReset('jdupont');
			expect.unreachable('aurait dû throw');
		} catch (err) {
			expect(isApiError(err)).toBe(true);
			if (isApiError(err)) {
				expect(err.code).toBe('RATE_LIMITED');
				expect(err.status).toBe(429);
			}
		}
	});

	it('resetPassword POSTe { token, newPassword } camelCase sur /api/v1/auth/reset-password', async () => {
		mockFetch.mockResolvedValue({
			ok: true,
			status: 200,
			json: () => Promise.resolve({ status: 'ok' }),
			headers: new Headers(),
		} as Response);
		await resetPassword('aB3xY9…token27chars', 'nouveau-mot-de-passe-12');
		const [url, init] = mockFetch.mock.calls[0] as [string, RequestInit];
		expect(url).toBe('/api/v1/auth/reset-password');
		expect(init.method).toBe('POST');
		// camelCase exigé par serde 17-4c (`new_password` serait silencieusement
		// ignoré → 422/400 backend).
		expect(JSON.parse(init.body as string)).toEqual({
			token: 'aB3xY9…token27chars',
			newPassword: 'nouveau-mot-de-passe-12',
		});
	});

	it('resetPassword propage le 400 INVALID_OR_EXPIRED_TOKEN générique', async () => {
		mockFetch.mockResolvedValue(
			errorResponse(400, 'INVALID_OR_EXPIRED_TOKEN', 'Lien invalide ou expiré') as Response,
		);
		try {
			await resetPassword('token-perime', 'nouveau-mot-de-passe-12');
			expect.unreachable('aurait dû throw');
		} catch (err) {
			expect(isApiError(err)).toBe(true);
			if (isApiError(err)) {
				expect(err.code).toBe('INVALID_OR_EXPIRED_TOKEN');
				expect(err.status).toBe(400);
			}
		}
	});

	it('resetPassword propage le 400 VALIDATION_ERROR (politique mdp)', async () => {
		mockFetch.mockResolvedValue(
			errorResponse(400, 'VALIDATION_ERROR', 'Mot de passe trop court') as Response,
		);
		try {
			await resetPassword('token-valide', 'court');
			expect.unreachable('aurait dû throw');
		} catch (err) {
			expect(isApiError(err)).toBe(true);
			if (isApiError(err)) {
				expect(err.code).toBe('VALIDATION_ERROR');
			}
		}
	});
});
