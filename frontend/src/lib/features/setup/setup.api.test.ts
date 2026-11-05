// Story v0172-session (0.17.3, #876, AC-6) — l'assistant transmet les champs
// d'inactivité de sa réponse à `authState.login` (un des trois constructeurs
// de `LoginPayload`).

import { describe, it, expect, vi } from 'vitest';

const postMock = vi.fn();
vi.mock('$lib/shared/utils/api-client', () => ({
	apiClient: { post: (...a: unknown[]) => postMock(...a) },
}));
const loginMock = vi.fn();
vi.mock('$lib/app/stores/auth.svelte', () => ({
	authState: { login: (...a: unknown[]) => loginMock(...a) },
}));

import { setupAdmin } from './setup.api';

describe('setupAdmin (Story v0172-session)', () => {
	it('transmet idleTimeoutMinutes et idleExpiresIn', async () => {
		postMock.mockResolvedValue({
			accessToken: 'a',
			refreshToken: 'r',
			expiresIn: 900,
			userId: 1,
			username: 'premier',
			role: 'Admin',
			idleTimeoutMinutes: 60,
			idleExpiresIn: 3600,
		});
		await setupAdmin('premier', 'un-mot-de-passe-solide');
		expect(loginMock).toHaveBeenCalledWith(
			expect.objectContaining({ userId: '1', idleTimeoutMinutes: 60, idleExpiresIn: 3600 }),
		);
	});
});
