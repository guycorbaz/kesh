// Story 31-1b (AC15, #503) — garde de rôle de `/onboarding/restore`, même règle
// que `(app)/admin/restore/+page.ts` : la page ne s'affiche pas non plus par URL
// directe pour un non-Administrateur (L7, P2). Le serveur refuse de toute façon (403).
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { authState } from '$lib/app/stores/auth.svelte';

vi.mock('$app/environment', () => ({ browser: true }));

describe('/onboarding/restore — garde de rôle', () => {
	beforeEach(async () => {
		vi.stubGlobal('fetch', vi.fn().mockResolvedValue({ ok: true }));
		await authState.logout();
	});

	for (const role of ['Comptable', 'Consultation']) {
		it(`${role} → redirection vers /onboarding`, async () => {
			authState.login({ userId: '2', username: 'u', role, expiresIn: 3600 });
			const { load } = await import('./+page');
			try {
				load();
				expect.unreachable('redirection attendue');
			} catch (err: unknown) {
				const e = err as { status: number; location: string };
				expect(e.status).toBe(302);
				expect(e.location).toBe('/onboarding');
			}
		});
	}

	it('Administrateur → la page se charge', async () => {
		authState.login({ userId: '1', username: 'admin', role: 'Admin', expiresIn: 3600 });
		const { load } = await import('./+page');
		expect(() => load()).not.toThrow();
	});
});
