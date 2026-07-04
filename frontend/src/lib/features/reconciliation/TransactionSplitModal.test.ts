// Story 8-5a-bis FR48 — tests Vitest pour TransactionSplitModal.svelte.
// Couvre AC #93 / #95 (balance live indicator, submit disabled tant que
// balance ≠ exact, ≥ 2 lignes obligatoires).
//
// Pattern : mock du module `reconciliation.api` + render via
// `@testing-library/svelte`. Mocks i18n + API definis AVANT l'import
// du composant (hoisting Vitest).

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, fireEvent } from '@testing-library/svelte';
import type { ReconciliationProposal } from './reconciliation.types';

vi.mock('./reconciliation.api', () => ({
	splitTransaction: vi.fn(),
}));

vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string) => fallback,
}));

// Story 19-5 — mock listProjects (chargé via $effect dans le composant).
const listProjectsMock = vi.fn(async () => [] as unknown[]);
vi.mock('$lib/features/projects/projects.api', () => ({
	listProjects: () => listProjectsMock(),
}));

import * as api from './reconciliation.api';
import TransactionSplitModal from './TransactionSplitModal.svelte';
import type { AccountResponse } from '$lib/features/accounts/accounts.types';

const mockApi = vi.mocked(api);

function makeProposal(amount: string = '-10700.00'): ReconciliationProposal {
	return {
		bankTransactionId: 42,
		transaction: {
			bookingDate: '2026-05-31',
			valueDate: '2026-05-31',
			amount,
			currency: 'CHF',
			counterpartyName: 'BATCH SALARIES',
		},
		candidates: [],
	};
}

function makeAccount(
	id: number,
	number: string,
	name: string,
	accountType: AccountResponse['accountType'],
): AccountResponse {
	return {
		id,
		companyId: 1,
		number,
		name,
		accountType,
		parentId: null,
		active: true,
		version: 1,
		createdAt: '2026-01-01T00:00:00Z',
		updatedAt: '2026-01-01T00:00:00Z',
	};
}

function makeAccounts(): AccountResponse[] {
	return [
		makeAccount(5000, '5000', 'Salaires', 'Expense'),
		makeAccount(5700, '5700', 'Charges sociales', 'Expense'),
		// Classe 1 (banque) — doit être filtré côté UI (5/6/7 only).
		makeAccount(1020, '1020', 'Banque', 'Asset'),
	];
}

describe('TransactionSplitModal', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	afterEach(() => {
		vi.restoreAllMocks();
	});

	it('renders modal with 2 initial lines + balance indicator when open=true', async () => {
		const { findByTestId } = render(TransactionSplitModal, {
			open: true,
			onOpenChange: () => {},
			bankAccountId: 17,
			proposal: makeProposal(),
			accounts: makeAccounts(),
			onSuccess: () => {},
		});

		// Au moins 2 lignes initialement (MIN_SPLITS).
		await findByTestId('split-row-0');
		await findByTestId('split-row-1');
		// Balance indicator + boutons.
		await findByTestId('split-balance-indicator');
		await findByTestId('split-submit');
		await findByTestId('split-cancel');
		await findByTestId('split-add-line');
	});

	// Story 19-5 — colonne projet par ligne visible seulement si projets présents.
	it('shows a per-line project selector when projects exist', async () => {
		listProjectsMock.mockResolvedValueOnce([
			{ id: 5, parentId: null, code: 'CHALET', name: 'Chalet', archived: false },
		]);
		const { findByTestId } = render(TransactionSplitModal, {
			open: true,
			onOpenChange: () => {},
			bankAccountId: 17,
			proposal: makeProposal(),
			accounts: makeAccounts(),
			onSuccess: () => {},
		});
		// Chaque ligne a son sélecteur projet.
		await findByTestId('split-line-project-0');
		await findByTestId('split-line-project-1');
	});

	it('hides the per-line project selector when no project exists', async () => {
		listProjectsMock.mockResolvedValueOnce([]);
		const { findByTestId, queryByTestId } = render(TransactionSplitModal, {
			open: true,
			onOpenChange: () => {},
			bankAccountId: 17,
			proposal: makeProposal(),
			accounts: makeAccounts(),
			onSuccess: () => {},
		});
		await findByTestId('split-row-0');
		expect(queryByTestId('split-line-project-0')).toBeNull();
	});

	it('disables submit until balance is exact', async () => {
		const { findByTestId } = render(TransactionSplitModal, {
			open: true,
			onOpenChange: () => {},
			bankAccountId: 17,
			proposal: makeProposal('-100.00'),
			accounts: makeAccounts(),
			onSuccess: () => {},
		});

		const submit = (await findByTestId('split-submit')) as HTMLButtonElement;
		// Initialement balance = 0 ≠ |-100| → submit disabled.
		expect(submit.disabled).toBe(true);
	});

	it('shows imbalance message when sum ≠ |tx.amount|', async () => {
		const { findByTestId } = render(TransactionSplitModal, {
			open: true,
			onOpenChange: () => {},
			bankAccountId: 17,
			proposal: makeProposal('-100.00'),
			accounts: makeAccounts(),
			onSuccess: () => {},
		});

		// Remplir un montant qui ne balance pas.
		const amount0 = (await findByTestId('split-amount-0')) as HTMLInputElement;
		await fireEvent.input(amount0, { target: { value: '40' } });
		const amount1 = (await findByTestId('split-amount-1')) as HTMLInputElement;
		await fireEvent.input(amount1, { target: { value: '30' } });

		const indicator = await findByTestId('split-balance-indicator');
		// 40 + 30 = 70 ≠ 100 → écart -30.00.
		expect(indicator.textContent).toContain('non équilibrée');
	});

	it('add-line and remove-line update split rows count (min 2, max 50)', async () => {
		const { findByTestId, queryByTestId } = render(TransactionSplitModal, {
			open: true,
			onOpenChange: () => {},
			bankAccountId: 17,
			proposal: makeProposal('-100.00'),
			accounts: makeAccounts(),
			onSuccess: () => {},
		});

		// Add une ligne.
		const addBtn = await findByTestId('split-add-line');
		await fireEvent.click(addBtn);
		await findByTestId('split-row-2');

		// Remove la 3e ligne.
		const removeBtn = await findByTestId('split-remove-2');
		await fireEvent.click(removeBtn);
		expect(queryByTestId('split-row-2')).toBeNull();

		// Tenter de retirer encore (min 2) — le bouton doit être disabled.
		const remove0 = (await findByTestId('split-remove-0')) as HTMLButtonElement;
		expect(remove0.disabled).toBe(true);
	});
});
