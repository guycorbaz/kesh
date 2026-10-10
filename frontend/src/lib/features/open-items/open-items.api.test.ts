// Story 15-1c-i, revue P1 (E, LOW) — `asOf` part TOUJOURS au serveur (C-15-1c-5),
// et les deux `lineId` d'un lettrage partent tels quels.
import { describe, it, expect, vi } from 'vitest';

const get = vi.fn().mockResolvedValue({});
const post = vi.fn().mockResolvedValue({});
const del = vi.fn().mockResolvedValue(undefined);
vi.mock('$lib/shared/utils/api-client', () => ({
	apiClient: { get: (u: string) => get(u), post: (u: string, b: unknown) => post(u, b), delete: (u: string) => del(u) },
}));

import { createLettering, deleteLettering, fetchLettering, fetchOpenItems, fetchProposals } from './open-items.api';

describe('open-items.api', () => {
	it('la vue reçoit asOf, limit et offset (mutation : asOf omis)', async () => {
		await fetchOpenItems(7, '2026-03-31', 50);
		const u = new URL(get.mock.calls[0][0], 'http://x');
		expect(u.pathname).toBe('/api/v1/accounts/7/open-items');
		expect(u.searchParams.get('asOf')).toBe('2026-03-31');
		expect(u.searchParams.get('offset')).toBe('50');
		expect(u.searchParams.get('limit')).toBe('50');
	});

	it('propositions sans asOf ni offset ; lettrage, lecture et délettrage par le code', async () => {
		await fetchProposals(7);
		expect(get).toHaveBeenLastCalledWith('/api/v1/accounts/7/lettering-proposals');
		await createLettering([4, 9]);
		expect(post).toHaveBeenCalledWith('/api/v1/letterings', { lineIds: [4, 9] });
		await fetchLettering('AB');
		expect(get).toHaveBeenLastCalledWith('/api/v1/letterings/AB');
		await deleteLettering('AB');
		expect(del).toHaveBeenCalledWith('/api/v1/letterings/AB');
	});
});
