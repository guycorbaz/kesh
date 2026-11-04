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
import { apiHealth } from '$lib/shared/utils/api-health.svelte';

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

		// Story v0172-session (C-v0172s-26) : `/me` en 401, puis UN essai de
		// renouvellement (refusé ici) — deux appels au lieu d'un.
		expect(mockFetch).toHaveBeenCalledTimes(2);
		expect(mockFetch.mock.calls[1][0]).toBe('/api/v1/auth/refresh');
		expect(authState.isAuthenticated).toBe(false);
		expect(authState.currentUser).toBeNull();
		expect(authState.expiresIn).toBeNull();
	});

	// --- Story v0172-session (0.17.3, #876) : renouvellement au démarrage (AC-7) ---

	function reponse(ok: boolean, status: number, body: unknown) {
		return { ok, status, json: () => Promise.resolve(body) };
	}
	const ME = {
		userId: 7,
		username: 'bob',
		role: 'Comptable',
		expiresIn: 900,
		idleTimeoutMinutes: 60,
		idleExpiresIn: 3_000,
	};

	it('hydrate() : /me 401, renouvellement 200, /me 200 ⇒ identité et champs d’inactivité (AC-7 1)', async () => {
		const mockFetch = vi
			.fn()
			.mockResolvedValueOnce(reponse(false, 401, {}))
			.mockResolvedValueOnce(reponse(true, 200, { expiresIn: 900 }))
			.mockResolvedValueOnce(reponse(true, 200, ME));
		vi.stubGlobal('fetch', mockFetch);

		await authState.hydrate();

		expect(mockFetch.mock.calls.map((c) => c[0])).toEqual([
			'/api/v1/auth/me',
			'/api/v1/auth/refresh',
			'/api/v1/auth/me',
		]);
		expect(authState.currentUser).toEqual({ userId: '7', username: 'bob', role: 'Comptable' });
		expect(authState.idleTimeoutMinutes).toBe(60);
		expect(authState.idleExpiresIn).toBe(3_000);
	});

	it('hydrate() : renouvellement refusé ⇒ état nul, sans redirection (AC-7 2)', async () => {
		const replaceMock = vi.fn();
		const original = window.location;
		Object.defineProperty(window, 'location', {
			value: { ...original, replace: replaceMock },
			writable: true,
			configurable: true,
		});
		vi.stubGlobal(
			'fetch',
			vi.fn().mockResolvedValueOnce(reponse(false, 401, {})).mockResolvedValueOnce(reponse(false, 401, {})),
		);

		await authState.hydrate();

		expect(authState.currentUser).toBeNull();
		expect(replaceMock).not.toHaveBeenCalled();
		Object.defineProperty(window, 'location', { value: original, writable: true, configurable: true });
	});

	it('hydrate() et un chemin 401 en vol ⇒ un seul POST /auth/refresh (AC-7 3)', async () => {
		const { apiClient } = await import('$lib/shared/utils/api-client');
		let lacher: (v: unknown) => void = () => {};
		const refreshEnVol = new Promise((r) => (lacher = r));
		const mockFetch = vi.fn((url: string) => {
			if (url === '/api/v1/auth/refresh') return refreshEnVol;
			if (url === '/api/v1/some/data') return Promise.resolve(reponse(false, 401, {}));
			return Promise.resolve(reponse(true, 200, ME));
		});
		vi.stubGlobal('fetch', mockFetch);
		// Le chemin 401 lance le renouvellement…
		const requete = apiClient.get('/api/v1/some/data').catch(() => undefined);
		await vi.waitFor(() =>
			expect(mockFetch.mock.calls.some((c) => c[0] === '/api/v1/auth/refresh')).toBe(true),
		);
		// … pendant que /me du démarrage rend 401 (ici : la seconde lecture).
		mockFetch.mockImplementationOnce(() => Promise.resolve(reponse(false, 401, {})));
		const hydratation = authState.hydrate();
		lacher(reponse(true, 200, { expiresIn: 900 }));
		await Promise.all([requete, hydratation]);
		expect(mockFetch.mock.calls.filter((c) => c[0] === '/api/v1/auth/refresh')).toHaveLength(1);
	});

	it('hydrate() sur /login?reason=idle : aucune requête, état nul (AC-7 4)', async () => {
		const original = window.location;
		Object.defineProperty(window, 'location', {
			value: { ...original, pathname: '/login', search: '?reason=idle&minutes=60' },
			writable: true,
			configurable: true,
		});
		const mockFetch = vi.fn();
		vi.stubGlobal('fetch', mockFetch);

		await authState.hydrate();

		expect(mockFetch).not.toHaveBeenCalled();
		expect(authState.currentUser).toBeNull();
		Object.defineProperty(window, 'location', { value: original, writable: true, configurable: true });
	});

	it('hydrate() sur /login?reason=session_expired : comportement ordinaire (/me)', async () => {
		const original = window.location;
		Object.defineProperty(window, 'location', {
			value: { ...original, pathname: '/login', search: '?reason=session_expired' },
			writable: true,
			configurable: true,
		});
		const mockFetch = vi.fn().mockResolvedValue(reponse(true, 200, ME));
		vi.stubGlobal('fetch', mockFetch);

		await authState.hydrate();

		expect(mockFetch.mock.calls[0][0]).toBe('/api/v1/auth/me');
		Object.defineProperty(window, 'location', { value: original, writable: true, configurable: true });
	});

	// --- Issue #347 : un backend indisponible n'invalide PAS la session ---
	//
	// ⚠️ La version précédente de ce test assertait `currentUser === null` après
	// un 5xx — mais le `beforeEach` appelle `logout()`, donc l'état était DÉJÀ
	// null avant l'appel. Il passait quel que soit le comportement, y compris
	// après inversion : il ne discriminait rien. Les assertions portent
	// désormais sur ce qui distingue réellement les deux branches.
	//
	// ⚠️ **Ce que ces tests ne peuvent PAS couvrir, et pourquoi** : `hydrate()`
	// fait un early-return si l'état est déjà hydraté, et `login()` le marque
	// tel. Le seul chemin où `hydrate()` s'exécute sur une identité peuplée est
	// le listener cross-tab (`auth-change`), qui remet `_hydrated = false` — il
	// dépend de `BroadcastChannel`, que jsdom ne garantit pas. La clause « ne
	// pas effacer l'identité » de #347 sert donc ce cas-là ; au rechargement de
	// page, `_currentUser` est de toute façon nul et c'est `setDegraded()` qui
	// porte la correction.

	it("hydrate() avec 5xx signale l'indisponibilité au lieu de la traiter comme une session invalide", async () => {
		const warnSpy = vi.spyOn(console, 'warn').mockImplementation(() => {});
		vi.stubGlobal(
			'fetch',
			vi.fn().mockResolvedValue({
				ok: false,
				status: 503,
				json: () => Promise.resolve({ error: { code: 'SERVICE_UNAVAILABLE', message: '' } }),
			}),
		);

		await authState.hydrate();

		expect(apiHealth.isDegraded).toBe(true);
		expect(warnSpy).toHaveBeenCalledWith(
			expect.stringContaining('Hydration via /me returned non-OK status 503'),
		);
		warnSpy.mockRestore();
		apiHealth.clearDegraded();
	});

	it("hydrate() sur échec RÉSEAU signale l'indisponibilité — sans quoi la bannière reste invisible", async () => {
		// `hydrate()` appelle `fetch` directement, pas `apiClient` : le
		// signalement que ce dernier pose dans son propre catch ne s'applique
		// pas ici. C'est ce qui laissait l'utilisateur devant un écran de
		// connexion muet pendant une panne (issue #347).
		const errSpy = vi.spyOn(console, 'error').mockImplementation(() => {});
		vi.stubGlobal('fetch', vi.fn().mockRejectedValue(new TypeError('Failed to fetch')));

		await authState.hydrate();

		expect(apiHealth.isDegraded).toBe(true);
		errSpy.mockRestore();
		apiHealth.clearDegraded();
	});

	it("hydrate() avec 401 EFFACE l'identité et ne signale AUCUNE indisponibilité", async () => {
		// Le contraste est tout l'objet du correctif : 401 est une réponse du
		// serveur SUR la session ; 503 et l'échec réseau n'en sont pas.
		vi.stubGlobal(
			'fetch',
			vi.fn().mockResolvedValue({
				ok: false,
				status: 401,
				json: () => Promise.resolve({ error: { code: 'UNAUTHENTICATED', message: '' } }),
			}),
		);

		await authState.hydrate();

		expect(authState.isAuthenticated).toBe(false);
		expect(authState.currentUser).toBeNull();
		expect(apiHealth.isDegraded).toBe(false);
	});

	it("au démarrage à froid, un backend absent ne fabrique PAS d'authentification", async () => {
		// Préserver « rien » doit rester « rien » : un visiteur non connecté est
		// toujours redirigé vers /login pendant une panne — il y voit désormais
		// la bannière, au lieu d'un écran muet.
		vi.spyOn(console, 'error').mockImplementation(() => {});
		vi.stubGlobal('fetch', vi.fn().mockRejectedValue(new TypeError('Failed to fetch')));

		await authState.hydrate();

		expect(authState.isAuthenticated).toBe(false);
		expect(authState.currentUser).toBeNull();
		apiHealth.clearDegraded();
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
