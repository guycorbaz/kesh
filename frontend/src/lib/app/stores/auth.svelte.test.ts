/**
 * Tests Vitest du store `authState` — Story 10-5 refactor complet.
 *
 * Pré-Story 10-5 : tests basés sur localStorage + décodage JWT côté JS.
 * Post-Story 10-5 (D5/D6 actés) : tokens en cookies HttpOnly inaccessibles
 * JS → tests basés sur `fetch /api/v1/auth/me` mocké + nouvelle signature
 * `login({userId, username, role, expiresIn})`.
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { authState } from './auth.svelte';

describe('authState (Story 10-5)', () => {
	beforeEach(async () => {
		// Reset state via logout (fetch mocked succeed).
		vi.stubGlobal('fetch', vi.fn().mockResolvedValue({ ok: true }));
		await authState.logout();
		vi.restoreAllMocks();
	});

	it('démarre non authentifié', () => {
		expect(authState.isAuthenticated).toBe(false);
		expect(authState.expiresIn).toBeNull();
		expect(authState.currentUser).toBeNull();
	});

	it('login() peuple `_currentUser` et `_expiresIn` depuis le payload (D6 acté)', () => {
		authState.login({
			userId: '42',
			username: 'alice',
			role: 'Admin',
			expiresIn: 900,
		});

		expect(authState.isAuthenticated).toBe(true);
		expect(authState.expiresIn).toBe(900);
		expect(authState.currentUser).toEqual({
			userId: '42',
			username: 'alice',
			role: 'Admin',
		});
	});

	it('login() extrait correctement les différents rôles depuis le payload', () => {
		for (const role of ['Admin', 'Comptable', 'Consultation']) {
			authState.login({ userId: '1', username: 'u', role, expiresIn: 900 });
			expect(authState.currentUser?.role).toBe(role);
		}
	});

	it('updateExpiresIn() bumpe `_expiresIn` sans toucher `_currentUser` (T6.7)', () => {
		authState.login({ userId: '1', username: 'alice', role: 'Admin', expiresIn: 900 });
		expect(authState.expiresIn).toBe(900);
		const userBefore = authState.currentUser;

		authState.updateExpiresIn(1800);

		expect(authState.expiresIn).toBe(1800);
		expect(authState.currentUser).toEqual(userBefore); // inchangé
	});

	// --- Tests logout ---

	it('logout() nettoie tout le state', async () => {
		vi.stubGlobal('fetch', vi.fn().mockResolvedValue({ ok: true }));

		authState.login({ userId: '1', username: 'alice', role: 'Admin', expiresIn: 900 });
		expect(authState.isAuthenticated).toBe(true);

		await authState.logout();

		expect(authState.isAuthenticated).toBe(false);
		expect(authState.expiresIn).toBeNull();
		expect(authState.currentUser).toBeNull();
	});

	it("logout() envoie POST /api/v1/auth/logout avec credentials: 'include' et sans body refreshToken (cookie HttpOnly)", async () => {
		const mockFetch = vi.fn().mockResolvedValue({ ok: true });
		vi.stubGlobal('fetch', mockFetch);

		authState.login({ userId: '1', username: 'alice', role: 'Admin', expiresIn: 900 });
		await authState.logout();

		expect(mockFetch).toHaveBeenCalledWith('/api/v1/auth/logout', {
			method: 'POST',
			credentials: 'include',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({}),
		});
	});

	it('logout() ne requiert PAS de header Authorization', async () => {
		const mockFetch = vi.fn().mockResolvedValue({ ok: true });
		vi.stubGlobal('fetch', mockFetch);

		authState.login({ userId: '1', username: 'alice', role: 'Admin', expiresIn: 900 });
		await authState.logout();

		const callHeaders = mockFetch.mock.calls[0][1].headers;
		expect(callHeaders).not.toHaveProperty('Authorization');
	});

	it('logout() nettoie le state même si fetch échoue', async () => {
		vi.stubGlobal('fetch', vi.fn().mockRejectedValue(new Error('network error')));

		authState.login({ userId: '1', username: 'alice', role: 'Admin', expiresIn: 900 });
		await authState.logout();

		expect(authState.isAuthenticated).toBe(false);
		expect(authState.currentUser).toBeNull();
	});

	// --- Tests hydrate (Story 10-5 T6.3) ---

	it('hydrate() fetch /me, peuple `_currentUser` + `_expiresIn` si 200', async () => {
		// CR Pass 1 H2 — assertions complètes + reset _hydrated via logout
		// (M2 patch : logout() réinitialise maintenant _hydrated = false).
		// Le beforeEach appelle logout(), donc _hydrated est false ici.
		const mockFetch = vi.fn().mockResolvedValue({
			ok: true,
			json: () =>
				Promise.resolve({
					userId: 42,
					username: 'alice',
					role: 'Admin',
					expiresIn: 900,
				}),
		});
		vi.stubGlobal('fetch', mockFetch);

		await authState.hydrate();

		// CR Pass 1 H2 — assertions explicites sur le state post-hydrate.
		expect(mockFetch).toHaveBeenCalledWith('/api/v1/auth/me', {
			credentials: 'include',
		});
		expect(authState.isAuthenticated).toBe(true);
		expect(authState.currentUser).toEqual({
			userId: '42', // body.userId (number) converti en String
			username: 'alice',
			role: 'Admin',
		});
		expect(authState.expiresIn).toBe(900);
	});

	it('hydrate() avec 401 laisse l\'état non-auth (cookie absent ou expiré)', async () => {
		// CR Pass 1 H2 complémentaire — couvre le branch 401 explicite.
		const mockFetch = vi.fn().mockResolvedValue({
			ok: false,
			status: 401,
			json: () => Promise.resolve({ error: { code: 'UNAUTHENTICATED', message: '' } }),
		});
		vi.stubGlobal('fetch', mockFetch);

		await authState.hydrate();

		expect(mockFetch).toHaveBeenCalledOnce();
		expect(authState.isAuthenticated).toBe(false);
		expect(authState.currentUser).toBeNull();
		expect(authState.expiresIn).toBeNull();
	});

	it('hydrate() avec 5xx (backend KO transitoire) reset state à null + console.warn (CR Pass 3 BH3-M3 / Pass 4 ECH4-L1)', async () => {
		// CR Pass 3 BH3-M3 a ajouté la branche else (non-OK et non-401) qui
		// log warn + reset state à null pour discriminer panne backend
		// (5xx) vs session expirée légitime (401). Couvre Pass 4 ECH4-L1.
		const warnSpy = vi.spyOn(console, 'warn').mockImplementation(() => {});
		const mockFetch = vi.fn().mockResolvedValue({
			ok: false,
			status: 503,
			json: () => Promise.resolve({ error: { code: 'SERVICE_UNAVAILABLE', message: '' } }),
		});
		vi.stubGlobal('fetch', mockFetch);

		await authState.hydrate();

		expect(mockFetch).toHaveBeenCalledOnce();
		expect(authState.isAuthenticated).toBe(false);
		expect(authState.currentUser).toBeNull();
		expect(authState.expiresIn).toBeNull();
		// Le warn doit signaler la non-OK status pour observabilité.
		expect(warnSpy).toHaveBeenCalledWith(
			expect.stringContaining('Hydration via /me returned non-OK status 503'),
		);
		warnSpy.mockRestore();
	});

	it('hydrate() avec body /me malformé (CR Pass 4 BH4-M1↓) — guards runtime reset state à null', async () => {
		// CR Pass 4 BH4-M1↓ defense-in-depth : si /me 200 retourne un body
		// malformé (e.g. {userId: null} via proxy/WAF/migration), les guards
		// runtime typeof empêchent de pourrir `_currentUser` avec des undefined.
		const errorSpy = vi.spyOn(console, 'error').mockImplementation(() => {});
		const mockFetch = vi.fn().mockResolvedValue({
			ok: true,
			json: () =>
				Promise.resolve({
					userId: null, // shape violation
					username: 'alice',
					role: 'Admin',
					expiresIn: 900,
				}),
		});
		vi.stubGlobal('fetch', mockFetch);

		await authState.hydrate();

		expect(authState.isAuthenticated).toBe(false);
		expect(authState.currentUser).toBeNull();
		expect(authState.expiresIn).toBeNull();
		expect(errorSpy).toHaveBeenCalledWith(
			expect.stringContaining('/me returned malformed body'),
			expect.anything(),
		);
		errorSpy.mockRestore();
	});

	it('hydrate() guard idempotence — 2e appel = no-op (fetch appelé 1 seule fois)', async () => {
		const mockFetch = vi.fn().mockResolvedValue({
			ok: true,
			json: () =>
				Promise.resolve({
					userId: 1,
					username: 'u',
					role: 'Admin',
					expiresIn: 900,
				}),
		});
		vi.stubGlobal('fetch', mockFetch);

		await authState.hydrate();
		await authState.hydrate(); // 2e appel — guard _hydrated empêche le re-fetch

		expect(mockFetch).toHaveBeenCalledOnce();
	});
});
