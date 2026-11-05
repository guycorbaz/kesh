// Story v0174-correctifs-3 (#889, AC-3.8, site `users`) — la date de création d'un
// utilisateur, horodatage UTC sans décalage, s'affiche à sa date suisse : créé à 23 h 30
// UTC le 4 novembre, il l'est le 5 à Zurich. Avant la 0.17.4, `new Date(iso)` le lisait
// comme une heure LOCALE.
//
// Le nom du fichier n'est pas `+page.test.ts` : SvelteKit réserve le préfixe `+`.

import { describe, it, expect, vi, afterEach } from 'vitest';
import { render, cleanup } from '@testing-library/svelte';

vi.mock('$app/environment', () => ({ browser: true }));
vi.mock('svelte-sonner', () => ({ toast: { success: vi.fn(), error: vi.fn() } }));
vi.mock('$lib/app/stores/auth.svelte', () => ({
	authState: {
		isAuthenticated: true,
		currentUser: { role: 'Admin', username: 'admin', userId: '1' },
	},
}));
const getMock = vi.hoisted(() => vi.fn());
vi.mock('$lib/shared/utils/api-client', async (importOriginal) => {
	const real = await importOriginal<typeof import('$lib/shared/utils/api-client')>();
	return { ...real, apiClient: { ...real.apiClient, get: (url: string) => getMock(url) } };
});

import Page from './+page.svelte';
import { pinTimeZone } from '$lib/shared/utils/pin-time-zone.test-helper';

afterEach(() => {
	cleanup();
	vi.clearAllMocks();
});

describe('utilisateurs — la date de création à l’heure suisse (#889)', () => {
	pinTimeZone('Europe/Zurich');
	it('créé le 4 novembre à 23 h 30 UTC : `05.11.2026`', async () => {
		getMock.mockResolvedValue({
			items: [
				{
					id: 2,
					username: 'compta',
					role: 'Comptable',
					active: true,
					email: null,
					version: 1,
					createdAt: '2026-11-04T23:30:00',
					updatedAt: '2026-11-04T23:30:00',
				},
			],
			total: 1,
			offset: 0,
			limit: 20,
		});
		const { findByText } = render(Page);
		expect(await findByText('05.11.2026')).toBeTruthy();
	});
});
