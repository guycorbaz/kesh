// Story 15-5b (AC14, choix C14) — test Vitest de la page des règles
// d'affectation : un échec de chargement rejeté en `ApiError` (objet simple,
// pas une instance d'`Error`) s'affiche par son message, non par
// « [object Object] ».
//
// ⛔ `isApiError` n'est PAS mocké — il exige `code` ET `status`.

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render } from '@testing-library/svelte';

vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string) => fallback,
}));

const listRulesMock = vi.fn();
vi.mock('$lib/features/reconciliation/rules/rules.api', () => ({
	listRules: (...args: unknown[]) => listRulesMock(...args),
	createRule: vi.fn(),
	updateRule: vi.fn(),
	deleteRule: vi.fn(),
}));

const fetchAccountsMock = vi.fn();
vi.mock('$lib/features/accounts/accounts.api', () => ({
	fetchAccounts: () => fetchAccountsMock(),
}));

import Page from './+page.svelte';

describe('page des règles — échec de chargement lisible (Story 15-5b, AC14)', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	it("affiche le message d'un ApiError, pas « [object Object] »", async () => {
		listRulesMock.mockRejectedValueOnce({
			code: 'FORBIDDEN',
			status: 403,
			message: 'Accès refusé.',
		});
		fetchAccountsMock.mockResolvedValueOnce([]);
		const { findByTestId } = render(Page);
		const error = await findByTestId('rules-page-error');
		expect(error.textContent).toBe('Accès refusé.');
	});
});
