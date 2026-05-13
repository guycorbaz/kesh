// Story 8-4 (FR44) — H1 Pass 1 code review : tests Vitest pour
// ReconciliationProposals.svelte. Couvre AC #55 (rendu propositions
// + score badges) et AC #58 (état neutre tx sans candidate).
//
// Pattern : mock du module `reconciliation.api` + render via
// `@testing-library/svelte` (Svelte 5 supporté par 5.3.1). On
// vérifie le contrat data-attributes (data-testid) et les flows
// onAccept top-1.

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, fireEvent } from '@testing-library/svelte';
import type {
	GetProposalsResponse,
	AcceptResponse,
	RejectResponse,
} from './reconciliation.types';

// Mock du module API : doit être défini AVANT l'import du composant
// (hoisting Vitest applique vi.mock avant les imports).
vi.mock('./reconciliation.api', () => ({
	getProposals: vi.fn(),
	acceptProposals: vi.fn(),
	rejectProposals: vi.fn(),
	manualMatchTransaction: vi.fn(),
}));

// Story 8-5a-base — accounts API mocked (les comptes sont chargés par
// ReconciliationProposals au mount pour passer au ManualMatchModal).
vi.mock('$lib/features/accounts/accounts.api', () => ({
	fetchAccounts: vi.fn().mockResolvedValue([]),
}));

// Mock i18n util pour rendre le composant déterministe (renvoie le
// fallback fourni en deuxième argument).
vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string) => fallback,
}));

import * as api from './reconciliation.api';
import ReconciliationProposals from './ReconciliationProposals.svelte';

const mockApi = vi.mocked(api);

function makeProposalWithCandidate(
	bankTransactionId: number,
	invoiceId: number,
	scoreTotal: number,
): GetProposalsResponse['proposals'][number] {
	return {
		bankTransactionId,
		transaction: {
			bookingDate: '2026-05-15',
			// P-M7 Pass 1 : valueDate exposé par GET /proposals.
			valueDate: '2026-05-15',
			amount: '100.00',
			currency: 'CHF',
			counterpartyName: 'ACME GMBH',
		},
		candidates: [
			{
				candidateType: 'invoice' as const,
				invoiceId,
				invoiceNumber: `INV-2026-${invoiceId}`,
				invoiceAmount: '100.00',
				invoiceDate: '2026-05-10',
				ruleId: null,
				ruleLabel: null,
				counterpartyAccountId: null,
				counterpartyAccountName: null,
				score: {
					total: scoreTotal,
					amountScore: 1.0,
					referenceScore: scoreTotal >= 0.9 ? 1.0 : 0.5,
					contactScore: 0.0,
				},
			},
		],
	};
}

function makeProposalWithoutCandidate(
	bankTransactionId: number,
): GetProposalsResponse['proposals'][number] {
	return {
		bankTransactionId,
		transaction: {
			bookingDate: '2026-05-15',
			// P-M7 Pass 1 : valueDate exposé par GET /proposals (null
			// fallback testé séparément dans ManualMatchModal.test.ts).
			valueDate: null,
			amount: '50.00',
			currency: 'CHF',
			counterpartyName: 'UNKNOWN',
		},
		candidates: [],
	};
}

describe('ReconciliationProposals', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	afterEach(() => {
		vi.restoreAllMocks();
	});

	it('renders proposals with score badges', async () => {
		mockApi.getProposals.mockResolvedValue({
			proposals: [
				makeProposalWithCandidate(1, 101, 1.0),
				makeProposalWithoutCandidate(2),
			],
			hasMore: false,
		} satisfies GetProposalsResponse);

		const { findAllByTestId } = render(ReconciliationProposals, {
			bankAccountId: 17,
		});

		const rows = await findAllByTestId('reconciliation-row');
		expect(rows).toHaveLength(2);

		const badges = await findAllByTestId('score-badge');
		expect(badges.length).toBeGreaterThanOrEqual(1);
		// La proposal 1 a score=1.0 → tier high.
		expect(badges[0].getAttribute('data-score-tier')).toBe('high');
	});

	it('shows neutral state for tx without candidates', async () => {
		mockApi.getProposals.mockResolvedValue({
			proposals: [makeProposalWithoutCandidate(2)],
			hasMore: false,
		} satisfies GetProposalsResponse);

		const { findByTestId, queryAllByTestId, findByText } = render(
			ReconciliationProposals,
			{ bankAccountId: 17 },
		);

		// La row existe, le texte « Aucune correspondance » est rendu.
		await findByTestId('reconciliation-row');
		await findByText(/Aucune correspondance/i);

		// γ refactor : pas de checkbox tx-level pour une tx sans candidate.
		const checkboxes = queryAllByTestId('tx-checkbox');
		expect(checkboxes).toHaveLength(0);
	});

	it('selects tx via checkbox and triggers accept with top-1 invoice', async () => {
		mockApi.getProposals.mockResolvedValue({
			proposals: [makeProposalWithCandidate(42, 101, 1.0)],
			hasMore: false,
		} satisfies GetProposalsResponse);
		mockApi.acceptProposals.mockResolvedValue({
			accepted: [
				{
					bankTransactionId: 42,
					invoiceId: 101,
					journalEntryId: 999,
					score: {
						total: 1.0,
						amountScore: 1.0,
						referenceScore: 1.0,
						contactScore: 0.0,
					},
				},
			],
			failed: [],
		} satisfies AcceptResponse);

		const { findByTestId } = render(ReconciliationProposals, {
			bankAccountId: 17,
		});

		const checkbox = await findByTestId('tx-checkbox');
		await fireEvent.click(checkbox);

		const acceptBtn = await findByTestId('reconciliation-accept-btn');
		await fireEvent.click(acceptBtn);

		expect(mockApi.acceptProposals).toHaveBeenCalledTimes(1);
		const [bankAccountId, items] = mockApi.acceptProposals.mock.calls[0];
		expect(bankAccountId).toBe(17);
		// Story 8-5a-bis Q2 breaking — chaque proposal porte un discriminator
		// `type` ('invoice' par défaut depuis ReconciliationProposals).
		expect(items).toEqual([
			{ type: 'invoice', bankTransactionId: 42, invoiceId: 101 },
		]);
	});

	// Story 8-5a-base FR45 — bouton « Affecter manuellement » présent
	// sur toutes les rows pending (avec ou sans candidate). Décision
	// Pass 1 code review P-M2 : override workflow utile pour frais
	// bancaires (tx sans candidate) ET correction d'un match auto
	// incorrect (tx avec candidate). Cf. L66.
	it('shows manual match button for all pending rows (with and without candidate)', async () => {
		mockApi.getProposals.mockResolvedValue({
			proposals: [
				makeProposalWithoutCandidate(2),
				makeProposalWithCandidate(3, 102, 0.6),
			],
			hasMore: false,
		} satisfies GetProposalsResponse);

		const { findAllByTestId } = render(ReconciliationProposals, {
			bankAccountId: 17,
		});

		const buttons = await findAllByTestId('manual-match-button');
		// 2 rows → 2 boutons « Affecter manuellement » (override workflow,
		// même quand candidate auto-proposée existe — cf. L66 Pass 1).
		expect(buttons).toHaveLength(2);
		// Présent sur la tx sans candidate ET sur la tx avec candidate.
		expect(buttons[0].getAttribute('data-tx-id')).toBe('2');
		expect(buttons[1].getAttribute('data-tx-id')).toBe('3');
	});

	it('triggers reject with selected txIds only', async () => {
		mockApi.getProposals.mockResolvedValue({
			proposals: [makeProposalWithCandidate(7, 200, 0.5)],
			hasMore: false,
		} satisfies GetProposalsResponse);
		mockApi.rejectProposals.mockResolvedValue({
			rejected: [{ bankTransactionId: 7, rejectedAt: '2026-05-15T00:00:00Z' }],
			failed: [],
		} satisfies RejectResponse);

		const { findByTestId } = render(ReconciliationProposals, {
			bankAccountId: 17,
		});

		const checkbox = await findByTestId('tx-checkbox');
		await fireEvent.click(checkbox);

		const rejectBtn = await findByTestId('reconciliation-reject-btn');
		await fireEvent.click(rejectBtn);

		expect(mockApi.rejectProposals).toHaveBeenCalledTimes(1);
		const [bankAccountId, ids] = mockApi.rejectProposals.mock.calls[0];
		expect(bankAccountId).toBe(17);
		expect(ids).toEqual([7]);
	});
});
