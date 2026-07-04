// Story 19-5 — tests Vitest pour RuleFormModal.svelte, focalisés sur le
// sélecteur « projet par défaut » (AC13). Pattern : mock rules.api + i18n +
// projects.api AVANT l'import du composant (hoisting Vitest), render via
// @testing-library/svelte (Svelte 5).

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render } from '@testing-library/svelte';
import type { AccountResponse } from '$lib/features/accounts/accounts.types';

vi.mock('./rules.api', () => ({
	createRule: vi.fn(),
	updateRule: vi.fn(),
}));

vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string) => fallback,
}));

const listProjectsMock = vi.fn(async () => [] as unknown[]);
vi.mock('$lib/features/projects/projects.api', () => ({
	listProjects: () => listProjectsMock(),
}));

import RuleFormModal from './RuleFormModal.svelte';
import type { ReconciliationRule } from './rules.types';

function makeAccounts(): AccountResponse[] {
	return [
		{
			id: 10,
			companyId: 1,
			number: '6510',
			name: 'Télécom',
			accountType: 'Expense',
			active: true,
		} as AccountResponse,
	];
}

describe('RuleFormModal — sélecteur projet par défaut (Story 19-5)', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('affiche le sélecteur projet par défaut quand des projets existent', async () => {
		listProjectsMock.mockResolvedValueOnce([
			{ id: 7, parentId: null, code: 'RENOV', name: 'Rénovation', archived: false },
		]);
		const { findByTestId } = render(RuleFormModal, {
			rule: null,
			accounts: makeAccounts(),
			onSuccess: () => {},
			onCancel: () => {},
		});
		await findByTestId('rule-form-default-project');
	});

	it('masque le sélecteur projet quand aucun projet et aucun tag existant', async () => {
		listProjectsMock.mockResolvedValueOnce([]);
		const { findByTestId, queryByTestId } = render(RuleFormModal, {
			rule: null,
			accounts: makeAccounts(),
			onSuccess: () => {},
			onCancel: () => {},
		});
		await findByTestId('rule-form-modal');
		expect(queryByTestId('rule-form-default-project')).toBeNull();
	});

	it("affiche le sélecteur en édition même sans projets chargés si la règle porte déjà un tag", async () => {
		listProjectsMock.mockResolvedValueOnce([]);
		const rule: ReconciliationRule = {
			id: 1,
			label: 'R1',
			matchType: 'counterparty_contains',
			matchValue: 'X',
			counterpartyAccountId: 10,
			priority: 100,
			active: true,
			defaultProjectId: 42,
			appliedCount: 0,
			lastAppliedAt: null,
			version: 1,
			createdAt: '2026-07-04T00:00:00',
			updatedAt: '2026-07-04T00:00:00',
		};
		const { findByTestId } = render(RuleFormModal, {
			rule,
			accounts: makeAccounts(),
			onSuccess: () => {},
			onCancel: () => {},
		});
		// Le tag existant force l'affichage (option ad-hoc « Projet archivé »).
		await findByTestId('rule-form-default-project');
	});
});
