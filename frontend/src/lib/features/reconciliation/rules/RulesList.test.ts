// Story 15-5b (AC14, choix C14) — tests Vitest de RulesList.svelte.
//
// Le client d'API lève un `ApiError` OBJET SIMPLE (`code`, `status`,
// `message`), pas une instance d'`Error`. Le motif `e instanceof Error ?
// e.message : String(e)` l'affichait « [object Object] » — en particulier le
// refus 400 `ACCOUNT_NOT_POSTABLE` de la réactivation d'une règle dont le
// compte n'est plus imputable (AC8 b).
//
// ⛔ `isApiError` n'est PAS mocké : il exige `code` (string) ET `status`
// (number) ; une erreur simulée sans `status` retomberait sur `String(e)` et le
// test passerait à côté de la branche qu'il couvre.

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, fireEvent } from '@testing-library/svelte';
import type { AccountResponse } from '$lib/features/accounts/accounts.types';
import type { ReconciliationRule } from './rules.types';

vi.mock('./rules.api', () => ({
	updateRule: vi.fn(),
	deleteRule: vi.fn(),
}));

vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string) => fallback,
}));

import RulesList from './RulesList.svelte';
import { deleteRule, updateRule } from './rules.api';

const NOT_POSTABLE = {
	code: 'ACCOUNT_NOT_POSTABLE',
	status: 400,
	message: "Le compte 6500 n'est pas imputable.",
};

function rule(overrides: Partial<ReconciliationRule> = {}): ReconciliationRule {
	return {
		id: 1,
		label: 'Loyer',
		matchType: 'counterparty_contains',
		matchValue: 'GERANCE',
		counterpartyAccountId: 10,
		priority: 100,
		active: false,
		defaultProjectId: null,
		appliedCount: 0,
		lastAppliedAt: null,
		version: 3,
		createdAt: '2026-10-08T00:00:00',
		updatedAt: '2026-10-08T00:00:00',
		...overrides,
	};
}

const accounts: AccountResponse[] = [
	{
		id: 10,
		companyId: 1,
		number: '6500',
		name: 'Frais de bureau',
		accountType: 'Expense',
		active: true,
		role: null,
		postable: false,
	} as AccountResponse,
];

describe('RulesList — refus du serveur lisibles (Story 15-5b, AC14)', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});
	afterEach(() => {
		vi.unstubAllGlobals();
	});

	it("affiche le message d'un ApiError à la réactivation, pas « [object Object] »", async () => {
		vi.mocked(updateRule).mockRejectedValueOnce(NOT_POSTABLE);
		const { getByTestId, findByTestId } = render(RulesList, {
			rules: [rule()],
			accounts,
			onEdit: () => {},
			onRefresh: () => {},
		});
		await fireEvent.click(getByTestId('toggle-active-button-1'));
		const error = await findByTestId('rules-list-error');
		expect(error.textContent).toBe(NOT_POSTABLE.message);
		expect(updateRule).toHaveBeenCalledWith(1, { expectedVersion: 3, active: true });
	});

	it("affiche le message d'un ApiError à l'archivage", async () => {
		vi.stubGlobal('confirm', () => true);
		vi.mocked(deleteRule).mockRejectedValueOnce({
			code: 'RECONCILIATION_RULE_NOT_FOUND',
			status: 404,
			message: 'Règle introuvable.',
		});
		const { getByTestId, findByTestId } = render(RulesList, {
			rules: [rule({ active: true })],
			accounts,
			onEdit: () => {},
			onRefresh: () => {},
		});
		await fireEvent.click(getByTestId('delete-button-1'));
		const error = await findByTestId('rules-list-error');
		expect(error.textContent).toBe('Règle introuvable.');
	});
});
