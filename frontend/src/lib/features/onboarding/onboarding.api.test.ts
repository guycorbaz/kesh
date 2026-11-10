/**
 * Story 34-1 (#966, AC-1, M6) — `setBankAccount` envoie TOUJOURS `description` et
 * `journalAccountId`, `null` compris : côté serveur, un champ absent vaut
 * « inchangé » et `null` vaut « effacé » (C-e341-34). Un `undefined` serait omis
 * par `JSON.stringify` et changerait le sens de l'envoi.
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';

const { postMock } = vi.hoisted(() => ({ postMock: vi.fn() }));
vi.mock('$lib/shared/utils/api-client', () => ({
	apiClient: { post: postMock, get: vi.fn() },
}));

import { setBankAccount } from './onboarding.api';

beforeEach(() => {
	postMock.mockReset();
	postMock.mockResolvedValue({ stepCompleted: 7 });
});

describe('setBankAccount (Story 34-1)', () => {
	it('envoie la description et le compte lié', async () => {
		await setBankAccount({
			bankName: 'UBS',
			iban: 'CH93 0076 2011 6238 5295 7',
			qrIban: null,
			description: 'Personnel',
			journalAccountId: 12,
		});
		expect(postMock).toHaveBeenCalledWith('/api/v1/onboarding/bank-account', {
			bankName: 'UBS',
			iban: 'CH93 0076 2011 6238 5295 7',
			qrIban: null,
			description: 'Personnel',
			journalAccountId: 12,
		});
	});

	it('envoie les deux null explicites, qui survivent à la sérialisation', async () => {
		await setBankAccount({
			bankName: 'UBS',
			iban: 'CH93 0076 2011 6238 5295 7',
			qrIban: null,
			description: null,
			journalAccountId: null,
		});
		const corps = JSON.parse(JSON.stringify(postMock.mock.calls[0][1]));
		expect(corps).toHaveProperty('description', null);
		expect(corps).toHaveProperty('journalAccountId', null);
	});
});
