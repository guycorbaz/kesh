import { describe, it, expect, vi, beforeEach } from 'vitest';
import { authState } from './auth.svelte';

/** Fabrique un JWT factice en base64url (conforme RFC 7519). */
function fakeJwt(payload: Record<string, unknown>): string {
	const toBase64Url = (obj: unknown) =>
		btoa(JSON.stringify(obj))
			.replace(/\+/g, '-')
			.replace(/\//g, '_')
			.replace(/=+$/, '');
	return `${toBase64Url({ alg: 'HS256', typ: 'JWT' })}.${toBase64Url(payload)}.fake-sig`;
}

describe('authState', () => {
	beforeEach(async () => {
		vi.stubGlobal('fetch', vi.fn().mockResolvedValue({ ok: true }));
		await authState.logout();
		vi.restoreAllMocks();
	});

	it('démarre non authentifié', () => {
		expect(authState.isAuthenticated).toBe(false);
		expect(authState.accessToken).toBeNull();
		expect(authState.refreshToken).toBeNull();
		expect(authState.expiresIn).toBeNull();
		expect(authState.currentUser).toBeNull();
	});

	it('login() stocke les tokens et décode le JWT', () => {
		const token = fakeJwt({ sub: '42', role: 'Admin', exp: 9999999999 });
		authState.login(token, 'refresh-uuid', 900);

		expect(authState.isAuthenticated).toBe(true);
		expect(authState.accessToken).toBe(token);
		expect(authState.refreshToken).toBe('refresh-uuid');
		expect(authState.expiresIn).toBe(900);
		expect(authState.currentUser).toEqual({ userId: '42', role: 'Admin' });
	});

	it('login() extrait correctement les différents rôles', () => {
		for (const role of ['Admin', 'Comptable', 'Consultation']) {
			const token = fakeJwt({ sub: '1', role, exp: 9999999999 });
			authState.login(token, 'r', 900);
			expect(authState.currentUser?.role).toBe(role);
		}
	});

	// --- P7 : tests JWT malformé / claims absents ---

	it('login() avec un token sans 3 segments lève une erreur', () => {
		expect(() => authState.login('not-a-jwt', 'r', 900)).toThrow('JWT malformé');
		expect(authState.isAuthenticated).toBe(false);
		expect(authState.currentUser).toBeNull();
	});

	it('login() avec un token vide lève une erreur', () => {
		expect(() => authState.login('', 'r', 900)).toThrow('JWT malformé');
		expect(authState.isAuthenticated).toBe(false);
	});

	it('login() avec payload base64 invalide lève une erreur', () => {
		expect(() => authState.login('header.!!!invalid!!!.sig', 'r', 900)).toThrow();
		expect(authState.isAuthenticated).toBe(false);
	});

	it('login() avec claims sub/role manquants lève une erreur', () => {
		const token = fakeJwt({ exp: 9999999999 }); // pas de sub ni role
		expect(() => authState.login(token, 'r', 900)).toThrow('Claims JWT manquants');
		expect(authState.isAuthenticated).toBe(false);
		expect(authState.currentUser).toBeNull();
	});

	it('login() avec claims sub/role vides lève une erreur', () => {
		const token = fakeJwt({ sub: '', role: '', exp: 9999999999 });
		expect(() => authState.login(token, 'r', 900)).toThrow('Claims JWT manquants');
		expect(authState.isAuthenticated).toBe(false);
	});

	it('login() ne mute pas le state si le JWT est invalide (atomicité)', () => {
		// D'abord un login valide
		const validToken = fakeJwt({ sub: '1', role: 'Admin', exp: 9999999999 });
		authState.login(validToken, 'valid-refresh', 900);
		expect(authState.isAuthenticated).toBe(true);

		// Puis un login invalide — le state précédent doit rester intact
		expect(() => authState.login('bad', 'r', 900)).toThrow();
		expect(authState.isAuthenticated).toBe(true);
		expect(authState.accessToken).toBe(validToken);
		expect(authState.refreshToken).toBe('valid-refresh');
	});

	// --- Tests logout ---

	it('logout() nettoie tout le state', async () => {
		const mockFetch = vi.fn().mockResolvedValue({ ok: true });
		vi.stubGlobal('fetch', mockFetch);

		const token = fakeJwt({ sub: '1', role: 'Admin', exp: 9999999999 });
		authState.login(token, 'refresh-uuid', 900);
		expect(authState.isAuthenticated).toBe(true);

		await authState.logout();

		expect(authState.isAuthenticated).toBe(false);
		expect(authState.accessToken).toBeNull();
		expect(authState.refreshToken).toBeNull();
		expect(authState.expiresIn).toBeNull();
		expect(authState.currentUser).toBeNull();
	});

	it('logout() envoie POST /api/v1/auth/logout avec refreshToken', async () => {
		const mockFetch = vi.fn().mockResolvedValue({ ok: true });
		vi.stubGlobal('fetch', mockFetch);

		const token = fakeJwt({ sub: '1', role: 'Admin', exp: 9999999999 });
		authState.login(token, 'my-refresh-token', 900);
		await authState.logout();

		expect(mockFetch).toHaveBeenCalledWith('/api/v1/auth/logout', {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({ refreshToken: 'my-refresh-token' }),
		});
	});

	it('logout() ne requiert PAS de header Authorization', async () => {
		const mockFetch = vi.fn().mockResolvedValue({ ok: true });
		vi.stubGlobal('fetch', mockFetch);

		const token = fakeJwt({ sub: '1', role: 'Admin', exp: 9999999999 });
		authState.login(token, 'r', 900);
		await authState.logout();

		const callHeaders = mockFetch.mock.calls[0][1].headers;
		expect(callHeaders).not.toHaveProperty('Authorization');
	});

	it('logout() nettoie le state même si fetch échoue', async () => {
		vi.stubGlobal('fetch', vi.fn().mockRejectedValue(new Error('network error')));

		const token = fakeJwt({ sub: '1', role: 'Admin', exp: 9999999999 });
		authState.login(token, 'r', 900);
		await authState.logout();

		expect(authState.isAuthenticated).toBe(false);
		expect(authState.currentUser).toBeNull();
	});

	it('logout() sans refreshToken ne fait pas de fetch', async () => {
		const mockFetch = vi.fn();
		vi.stubGlobal('fetch', mockFetch);

		await authState.logout();
		expect(mockFetch).not.toHaveBeenCalled();
	});
});
