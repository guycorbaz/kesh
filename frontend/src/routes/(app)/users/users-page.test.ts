import { describe, it, expect, vi, beforeEach } from 'vitest';
import { authState } from '$lib/app/stores/auth.svelte';

/** Fabrique un JWT factice en base64url. */
function fakeJwt(payload: Record<string, unknown>): string {
	const toBase64Url = (obj: unknown) =>
		btoa(JSON.stringify(obj))
			.replace(/\+/g, '-')
			.replace(/\//g, '_')
			.replace(/=+$/, '');
	return `${toBase64Url({ alg: 'HS256', typ: 'JWT' })}.${toBase64Url(payload)}.fake-sig`;
}

// Mock $app/environment to simulate browser = true
vi.mock('$app/environment', () => ({ browser: true }));

describe('users page auth guard', () => {
	beforeEach(async () => {
		vi.stubGlobal('fetch', vi.fn().mockResolvedValue({ ok: true }));
		await authState.logout();
		vi.restoreAllMocks();
		// Re-stub fetch after restoreAllMocks
		vi.stubGlobal('fetch', vi.fn().mockResolvedValue({ ok: true }));
	});

	it('redirige les non-admins (Comptable) vers /', async () => {
		const token = fakeJwt({ sub: '1', role: 'Comptable', exp: 9999999999 });
		authState.login(token, 'refresh', 3600);

		const { load } = await import('./+page');

		try {
			load();
			expect.unreachable('Should have thrown redirect');
		} catch (err: unknown) {
			const error = err as { status: number; location: string };
			expect(error.status).toBe(302);
			expect(error.location).toBe('/');
		}
	});

	it('redirige les non-admins (Consultation) vers /', async () => {
		const token = fakeJwt({ sub: '1', role: 'Consultation', exp: 9999999999 });
		authState.login(token, 'refresh', 3600);

		const { load } = await import('./+page');

		try {
			load();
			expect.unreachable('Should have thrown redirect');
		} catch (err: unknown) {
			const error = err as { status: number; location: string };
			expect(error.status).toBe(302);
			expect(error.location).toBe('/');
		}
	});

	it('permet l\'accès aux admins (pas de redirect)', async () => {
		const token = fakeJwt({ sub: '1', role: 'Admin', exp: 9999999999 });
		authState.login(token, 'refresh', 3600);

		const { load } = await import('./+page');

		// Should not throw — admin has access
		expect(() => load()).not.toThrow();
	});
});

describe('sidebar conditionnel (logique rôle)', () => {
	beforeEach(async () => {
		vi.stubGlobal('fetch', vi.fn().mockResolvedValue({ ok: true }));
		await authState.logout();
		vi.restoreAllMocks();
		vi.stubGlobal('fetch', vi.fn().mockResolvedValue({ ok: true }));
	});

	it('admin a le rôle Admin dans currentUser', () => {
		const token = fakeJwt({ sub: '1', role: 'Admin', exp: 9999999999 });
		authState.login(token, 'refresh', 3600);
		expect(authState.currentUser?.role).toBe('Admin');
	});

	it('comptable n\'a pas le rôle Admin', () => {
		const token = fakeJwt({ sub: '1', role: 'Comptable', exp: 9999999999 });
		authState.login(token, 'refresh', 3600);
		expect(authState.currentUser?.role).not.toBe('Admin');
	});

	it('consultation n\'a pas le rôle Admin', () => {
		const token = fakeJwt({ sub: '1', role: 'Consultation', exp: 9999999999 });
		authState.login(token, 'refresh', 3600);
		expect(authState.currentUser?.role).not.toBe('Admin');
	});
});
