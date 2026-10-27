/**
 * Story 32-7 (refs #723) — la garde de route de `/admin/mcp` (AC-1).
 *
 * Le `load` ne redirige que si `browser` est vrai (`admin/backup/+page.ts`) : il est
 * simulé ici. Témoin : le même `load` lève bien la redirection pour un Comptable —
 * sans quoi le cas Administrateur passerait à vide sous un `browser` resté faux. Le
 * serveur reste l'autorité : la garde est une garde d'écran.
 *
 * Le nom du fichier n'est pas `+page.test.ts` : SvelteKit réserve le préfixe `+`.
 */
import { describe, it, expect, vi, afterEach } from 'vitest';

vi.mock('$app/environment', () => ({ browser: true }));

import { authState } from '$lib/app/stores/auth.svelte';
import { load, ssr } from './+page';

function redirection(): { status: number; location: string } | null {
	try {
		load();
		return null;
	} catch (err: unknown) {
		return err as { status: number; location: string };
	}
}

describe('/admin/mcp — la garde', () => {
	afterEach(async () => {
		await authState.logout();
	});

	it('pas de rendu serveur', () => {
		expect(ssr).toBe(false);
	});

	it('témoin : un Comptable est redirigé vers / (302)', () => {
		authState.login({ userId: '1', username: 'c', role: 'Comptable', expiresIn: 3600 });
		const r = redirection();
		expect(r?.status).toBe(302);
		expect(r?.location).toBe('/');
	});

	it('Consultation : redirigé vers /', () => {
		authState.login({ userId: '1', username: 'v', role: 'Consultation', expiresIn: 3600 });
		expect(redirection()?.location).toBe('/');
	});

	it('Administrateur : aucune redirection', () => {
		authState.login({ userId: '1', username: 'a', role: 'Admin', expiresIn: 3600 });
		expect(redirection()).toBeNull();
	});
});
