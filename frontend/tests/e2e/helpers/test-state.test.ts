/**
 * Tests unitaires Vitest pour le helper `authedApiContext` de `test-state.ts`.
 *
 * Le helper réel orchestre Playwright (`page.evaluate` + `playwrightRequest.newContext`),
 * donc on mock entièrement `@playwright/test` pour valider le contrat de l'API
 * (résolution du token + injection du Bearer header + garde-fou null/empty) sans
 * démarrer un vrai browser.
 *
 * Cf. story 9-5-1b AC #4 — 3 cas obligatoires :
 *  1. Cas nominal (token présent) → context retourné avec Authorization Bearer
 *  2. Cas erreur (token null) → throw avec message exact
 *  3. Cas erreur (token vide) → throw (validation `!token` couvre les 3 cas)
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';

// Mock du module `@playwright/test` AVANT l'import du helper sous test.
// `vi.mock` est hoisted, donc on utilise `vi.hoisted` pour déclarer la mock
// fonction au même niveau (sinon ReferenceError « Cannot access X before
// initialization »).
const { newContextMock } = vi.hoisted(() => ({ newContextMock: vi.fn() }));
vi.mock('@playwright/test', () => ({
	request: {
		newContext: newContextMock,
	},
}));

// Import APRÈS le mock (le module test-state.ts importera la version mockée).
import { authedApiContext } from './test-state';

type FakePage = { evaluate: ReturnType<typeof vi.fn> };

function makePage(tokenValue: string | null): FakePage {
	return {
		evaluate: vi.fn().mockResolvedValue(tokenValue),
	};
}

beforeEach(() => {
	newContextMock.mockReset();
	// Fake APIRequestContext minimal — inclut `dispose` pour rester compatible
	// avec un caller qui appellerait `await ctx.dispose()` (anti-régression Pass 1
	// code-review BH-L3).
	newContextMock.mockResolvedValue({ dispose: vi.fn().mockResolvedValue(undefined) });
	// Stub explicite `KESH_BACKEND_URL` pour rendre le test hermétique vs CI env
	// (Pass 1 code-review ECH-L2). Sans ce stub, `resolveBackendUrl()` lit
	// `process.env.KESH_BACKEND_URL` qui peut être inattendu sur certains runners.
	vi.stubEnv('KESH_BACKEND_URL', 'http://test.example:3000');
});

afterEach(() => {
	vi.unstubAllEnvs();
});

describe('authedApiContext', () => {
	it('cas nominal : token présent en localStorage → context retourné avec Bearer header', async () => {
		const page = makePage('tok-123');

		await authedApiContext(page as never);

		expect(page.evaluate).toHaveBeenCalledTimes(1);
		expect(newContextMock).toHaveBeenCalledTimes(1);
		const call = newContextMock.mock.calls[0][0] as {
			baseURL: string;
			extraHTTPHeaders: Record<string, string>;
		};
		expect(call.extraHTTPHeaders).toEqual({ Authorization: 'Bearer tok-123' });
		expect(call.baseURL).toMatch(/^https?:\/\//);
	});

	it('cas erreur : token null en localStorage → throw avec message explicite', async () => {
		const page = makePage(null);

		await expect(authedApiContext(page as never)).rejects.toThrow(
			'authedApiContext: no accessToken in localStorage — call login(page) before this helper',
		);
		expect(newContextMock).not.toHaveBeenCalled();
	});

	it('cas erreur : token vide en localStorage → throw avec même message (validation !token)', async () => {
		const page = makePage('');

		await expect(authedApiContext(page as never)).rejects.toThrow(
			'authedApiContext: no accessToken in localStorage — call login(page) before this helper',
		);
		expect(newContextMock).not.toHaveBeenCalled();
	});
});
