/**
 * Tests unitaires Vitest pour le helper `authedApiContext` de `test-state.ts`.
 *
 * Story 10-5 refactor : le helper utilise désormais `storageState clone` du
 * browser context (pour propager les cookies HttpOnly) au lieu de lire
 * localStorage + injecter un Bearer header. Tests adaptés en conséquence.
 *
 * Le helper réel orchestre Playwright (`page.context().storageState()` +
 * `playwrightRequest.newContext`), donc on mock entièrement `@playwright/test`
 * pour valider le contrat de l'API (clone storageState + pas de Bearer header
 * post-Story-10-5).
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';

// Mock du module `@playwright/test` AVANT l'import du helper sous test.
const { newContextMock } = vi.hoisted(() => ({ newContextMock: vi.fn() }));
vi.mock('@playwright/test', () => ({
	request: {
		newContext: newContextMock,
	},
}));

// Import APRÈS le mock (le module test-state.ts importera la version mockée).
import { authedApiContext } from './test-state';

type FakeBrowserContext = {
	storageState: ReturnType<typeof vi.fn>;
};
type FakePage = {
	context: ReturnType<typeof vi.fn>;
};

function makePage(storageStateValue: object): FakePage {
	const browserContext: FakeBrowserContext = {
		storageState: vi.fn().mockResolvedValue(storageStateValue),
	};
	return {
		context: vi.fn().mockReturnValue(browserContext),
	};
}

beforeEach(() => {
	newContextMock.mockReset();
	newContextMock.mockResolvedValue({ dispose: vi.fn().mockResolvedValue(undefined) });
	vi.stubEnv('KESH_BACKEND_URL', 'http://test.example:3000');
});

afterEach(() => {
	vi.unstubAllEnvs();
});

describe('authedApiContext (Story 10-5 storageState clone)', () => {
	it('clone le storageState du browser context (incluant cookies HttpOnly)', async () => {
		// Simule un storageState Playwright contenant le cookie HttpOnly d'auth.
		const fakeStorageState = {
			cookies: [
				{
					name: 'kesh_access_token',
					value: 'jwt-token-xxx',
					domain: '127.0.0.1',
					path: '/',
					httpOnly: true,
					sameSite: 'Strict',
				},
			],
			origins: [],
		};
		const page = makePage(fakeStorageState);

		await authedApiContext(page as never);

		// storageState a été lu sur le browser context (clone).
		const ctxResult = page.context();
		expect(ctxResult.storageState).toHaveBeenCalledTimes(1);

		// newContext a reçu le storageState cloné + le baseURL.
		expect(newContextMock).toHaveBeenCalledTimes(1);
		const call = newContextMock.mock.calls[0][0] as {
			baseURL: string;
			storageState: object;
		};
		expect(call.storageState).toEqual(fakeStorageState);
		expect(call.baseURL).toMatch(/^https?:\/\//);
	});

	it("n'injecte PAS d'header Authorization Bearer (Story 10-5 — cookies HttpOnly remplacent Bearer)", async () => {
		const page = makePage({ cookies: [], origins: [] });

		await authedApiContext(page as never);

		const call = newContextMock.mock.calls[0][0] as {
			baseURL: string;
			storageState: object;
			extraHTTPHeaders?: Record<string, string>;
		};
		// Post-Story 10-5 : pas d'Authorization Bearer (cookies HttpOnly via storageState).
		expect(call.extraHTTPHeaders).toBeUndefined();
	});

	it('avec un storageState vide (pas de cookies), retourne quand même un context (login() pas appelé avant)', async () => {
		// Différence vs pre-Story-10-5 : pas de throw « no accessToken in localStorage »
		// — le contexte est créé même sans cookies. Les appels API authentifiés
		// échoueront en 401 si le user n'est pas authentifié (sémantique
		// browser HTTP standard).
		const emptyStorageState = { cookies: [], origins: [] };
		const page = makePage(emptyStorageState);

		await authedApiContext(page as never);

		expect(newContextMock).toHaveBeenCalledTimes(1);
	});
});
