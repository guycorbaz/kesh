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

// Revue de code P1 de la 15-5c (B1, E3) — les deux modales sont remplacées par une doublure
// qui appelle `onSuccess` d'un clic : seul le bilan du parent est en cause ici, et les
// modales ont leurs propres tests.
vi.mock('./ManualMatchModal.svelte', async () => ({
	default: (await import('./ModalSuccessStub.test.svelte')).default,
}));
vi.mock('./TransactionSplitModal.svelte', async () => ({
	default: (await import('./ModalSuccessStub.test.svelte')).default,
}));

// Mock i18n util pour rendre le composant déterministe (renvoie le
// fallback fourni en deuxième argument, variables `{ $x }` interpolées).
vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string, args?: Record<string, string | number>) =>
		args ? fallback.replace(/\{\s*\$(\w+)\s*\}/g, (_, k) => String(args[k] ?? '')) : fallback,
}));

import * as api from './reconciliation.api';
import ReconciliationProposals from './ReconciliationProposals.svelte';

const mockApi = vi.mocked(api);

function makeProposalWithCandidate(
	bankTransactionId: number,
	invoiceId: number,
	scoreTotal: number,
	amounts: { invoiceAmount: string; invoiceTotalTtc: string | null } = {
		invoiceAmount: '100.00',
		invoiceTotalTtc: null,
	},
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
				invoiceAmount: amounts.invoiceAmount,
				invoiceTotalTtc: amounts.invoiceTotalTtc,
				invoiceDate: '2026-05-10',
				ruleId: null,
				ruleLabel: null,
				ruleMatchType: null,
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

	// Story 25-4-c (#420) — la candidate affiche le RESTE DÛ ; le TTC ne
	// suit, en mention, que pour une facture déjà réglée en partie.
	it('shows the amount due, and the TTC only for a partially settled invoice', async () => {
		mockApi.getProposals.mockResolvedValue({
			proposals: [
				makeProposalWithCandidate(1, 101, 1.0, { invoiceAmount: '600', invoiceTotalTtc: '1000' }),
				makeProposalWithCandidate(2, 102, 1.0),
			],
			hasMore: false,
		} satisfies GetProposalsResponse);

		const { findAllByTestId, queryAllByTestId } = render(ReconciliationProposals, {
			bankAccountId: 17,
		});

		const tops = await findAllByTestId('candidate-top-1');
		expect(tops[0].textContent).toContain('(600, reste dû sur 1000)');
		expect(tops[1].textContent).toContain('(100.00)');
		expect(tops[1].textContent).not.toContain('reste dû sur');
		expect(queryAllByTestId('candidate-top-1-amount-due-of')).toHaveLength(1);
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

// Story 15-5b (AC14, choix C14) — le client d'API lève un `ApiError` OBJET
// SIMPLE, pas une instance d'`Error` : les trois `catch` du composant
// l'affichaient « [object Object] ». `isApiError` n'est pas mocké — l'erreur
// simulée porte donc `code` ET `status`.
describe('ReconciliationProposals — refus du serveur lisibles (Story 15-5b, AC14)', () => {
	const apiError = (message: string) => ({ code: 'INTERNAL_ERROR', status: 500, message });

	beforeEach(() => {
		vi.clearAllMocks();
	});

	it("affiche le message d'un ApiError au chargement", async () => {
		mockApi.getProposals.mockRejectedValueOnce(apiError('Chargement impossible.'));
		const { findByTestId } = render(ReconciliationProposals, { bankAccountId: 17 });
		const error = await findByTestId('reconciliation-error');
		expect(error.textContent).toBe('Chargement impossible.');
	});

	it("affiche le message d'un ApiError à l'acceptation", async () => {
		mockApi.getProposals.mockResolvedValue({
			proposals: [makeProposalWithCandidate(1, 101, 1.0)],
			hasMore: false,
		} satisfies GetProposalsResponse);
		mockApi.acceptProposals.mockRejectedValueOnce(apiError('Acceptation impossible.'));
		const { findByTestId, getByTestId } = render(ReconciliationProposals, {
			bankAccountId: 17,
		});
		await fireEvent.click(await findByTestId('tx-checkbox'));
		await fireEvent.click(getByTestId('reconciliation-accept-btn'));
		const error = await findByTestId('reconciliation-error');
		expect(error.textContent).toBe('Acceptation impossible.');
	});

	it("affiche le message d'un ApiError au rejet", async () => {
		mockApi.getProposals.mockResolvedValue({
			proposals: [makeProposalWithCandidate(1, 101, 1.0)],
			hasMore: false,
		} satisfies GetProposalsResponse);
		mockApi.rejectProposals.mockRejectedValueOnce(apiError('Rejet impossible.'));
		const { findByTestId, getByTestId } = render(ReconciliationProposals, {
			bankAccountId: 17,
		});
		await fireEvent.click(await findByTestId('tx-checkbox'));
		await fireEvent.click(getByTestId('reconciliation-reject-btn'));
		const error = await findByTestId('reconciliation-error');
		expect(error.textContent).toBe('Rejet impossible.');
	});
});

// Story 15-5c (AC3, #492, choix C38) — les refus d'un lot s'affichent par leur
// LIBELLÉ et désignent la transaction par sa date, son montant et sa
// contrepartie ; ils restent visibles quand le lot a vidé la liste.
// ⚠️ Dans les tests 2 et 3, le second `getProposals` est explicitement distinct du premier
// (liste vidée) : un mock à valeur constante ne verrait pas une transaction sortie de la
// liste. Le test 1 garde la transaction en liste — il ne mord donc pas sur le relevé avant
// rechargement, que les tests 2 et 3 couvrent.
describe('ReconciliationProposals — refus par lot lisibles (Story 15-5c, AC3)', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	it("affiche le libellé et la transaction d'un refus d'acceptation", async () => {
		mockApi.getProposals
			.mockResolvedValueOnce({
				proposals: [makeProposalWithCandidate(42, 101, 1.0)],
				hasMore: false,
			} satisfies GetProposalsResponse)
			// Après le lot, la transaction refusée est toujours en attente.
			.mockResolvedValueOnce({
				proposals: [makeProposalWithCandidate(42, 101, 1.0)],
				hasMore: false,
			} satisfies GetProposalsResponse);
		mockApi.acceptProposals.mockResolvedValue({
			accepted: [],
			failed: [
				{ bankTransactionId: 42, errorCode: 'ROUNDING_ACCOUNT_NOT_CONFIGURED', details: null },
			],
		} satisfies AcceptResponse);

		const { findByTestId, getByTestId } = render(ReconciliationProposals, { bankAccountId: 17 });
		await fireEvent.click(await findByTestId('tx-checkbox'));
		await fireEvent.click(getByTestId('reconciliation-accept-btn'));

		const item = await findByTestId('reconciliation-failed-item');
		const text = item.textContent ?? '';
		expect(text).toContain('2026-05-15');
		expect(text).toContain('100.00 CHF');
		expect(text).toContain('ACME GMBH');
		expect(text).toContain("aucun compte de différences d'arrondi utilisable");
		// Le code brut ne s'affiche plus dans le texte ; il reste pour le support.
		expect(text).not.toContain('ROUNDING_ACCOUNT_NOT_CONFIGURED');
		expect(text).not.toContain('TX #');
		expect(item.getAttribute('title')).toBe('ROUNDING_ACCOUNT_NOT_CONFIGURED');
	});

	it("affiche le libellé et la transaction d'un refus de rejet", async () => {
		mockApi.getProposals
			.mockResolvedValueOnce({
				proposals: [makeProposalWithCandidate(7, 200, 0.5)],
				hasMore: false,
			} satisfies GetProposalsResponse)
			.mockResolvedValueOnce({ proposals: [], hasMore: false } satisfies GetProposalsResponse);
		mockApi.rejectProposals.mockResolvedValue({
			rejected: [],
			failed: [
				{ bankTransactionId: 7, errorCode: 'RECONCILIATION_ALREADY_RECONCILED', details: null },
			],
		} satisfies RejectResponse);

		const { findByTestId, getByTestId } = render(ReconciliationProposals, { bankAccountId: 17 });
		await fireEvent.click(await findByTestId('tx-checkbox'));
		await fireEvent.click(getByTestId('reconciliation-reject-btn'));

		const item = await findByTestId('reconciliation-failed-item');
		const text = item.textContent ?? '';
		expect(text).toContain('2026-05-15');
		expect(text).toContain('100.00 CHF');
		expect(text).toContain('Cette transaction est déjà réconciliée.');
		expect(text).not.toContain('RECONCILIATION_ALREADY_RECONCILED');
	});

	it('garde le compteur et les échecs partiels quand le lot vide la liste', async () => {
		mockApi.getProposals
			.mockResolvedValueOnce({
				proposals: [makeProposalWithCandidate(1, 101, 1.0), makeProposalWithCandidate(2, 102, 1.0)],
				hasMore: false,
			} satisfies GetProposalsResponse)
			.mockResolvedValueOnce({ proposals: [], hasMore: false } satisfies GetProposalsResponse);
		mockApi.acceptProposals.mockResolvedValue({
			accepted: [
				{
					bankTransactionId: 1,
					invoiceId: 101,
					journalEntryId: 9,
					score: { total: 1.0, amountScore: 1.0, referenceScore: 1.0, contactScore: 0.0 },
				},
			],
			failed: [
				{
					bankTransactionId: 2,
					errorCode: 'ACCOUNT_NOT_POSTABLE',
					details: { rejected: [{ accountId: 5, accountNumber: '3000' }] },
				},
			],
		} satisfies AcceptResponse);

		const { findAllByTestId, findByTestId, getByTestId } = render(ReconciliationProposals, {
			bankAccountId: 17,
		});
		for (const cb of await findAllByTestId('tx-checkbox')) await fireEvent.click(cb);
		await fireEvent.click(getByTestId('reconciliation-accept-btn'));

		// La liste est vide après le lot…
		await findByTestId('reconciliation-empty');
		expect(mockApi.getProposals).toHaveBeenCalledTimes(2);
		// …et le bilan du lot reste affiché.
		expect((await findByTestId('reconciliation-success')).textContent).toContain('1');
		const item = await findByTestId('reconciliation-failed-item');
		expect(item.getAttribute('data-tx-id')).toBe('2');
		expect(item.textContent).toContain('Compte non imputable : 3000');
	});

	// Revue de code P1 (A-3) : une transaction absente du lot relevé garde `TX #<id>` en repli.
	it('désigne par TX #<id> un refus dont la transaction n’a pas été relevée', async () => {
		mockApi.getProposals
			.mockResolvedValueOnce({
				proposals: [makeProposalWithCandidate(42, 101, 1.0)],
				hasMore: false,
			} satisfies GetProposalsResponse)
			.mockResolvedValueOnce({ proposals: [], hasMore: false } satisfies GetProposalsResponse);
		mockApi.acceptProposals.mockResolvedValue({
			accepted: [],
			failed: [
				{ bankTransactionId: 99, errorCode: 'BANK_TRANSACTION_NOT_FOUND', details: null },
			],
		} satisfies AcceptResponse);

		const { findByTestId, getByTestId } = render(ReconciliationProposals, { bankAccountId: 17 });
		await fireEvent.click(await findByTestId('tx-checkbox'));
		await fireEvent.click(getByTestId('reconciliation-accept-btn'));

		const item = await findByTestId('reconciliation-failed-item');
		expect(item.textContent).toContain('TX #99');
		expect(item.textContent).toContain('Transaction bancaire introuvable.');
		expect(item.textContent).not.toContain('ACME GMBH');
	});

	// Revue de code P1 (B1, E3) : une affectation manuelle ou un éclatement réussi ouvre un
	// nouveau bilan ; celui du lot précédent ne doit pas lui survivre.
	it.each([
		['une affectation manuelle', 'manual-match-button'],
		['un éclatement', 'split-button'],
	])('efface le bilan du lot précédent après %s', async (_cas, bouton) => {
		mockApi.getProposals
			.mockResolvedValueOnce({
				proposals: [makeProposalWithCandidate(1, 101, 1.0), makeProposalWithCandidate(2, 102, 1.0)],
				hasMore: false,
			} satisfies GetProposalsResponse)
			.mockResolvedValue({
				proposals: [makeProposalWithCandidate(2, 102, 1.0)],
				hasMore: false,
			} satisfies GetProposalsResponse);
		mockApi.acceptProposals.mockResolvedValue({
			accepted: [
				{
					bankTransactionId: 1,
					invoiceId: 101,
					journalEntryId: 9,
					score: { total: 1.0, amountScore: 1.0, referenceScore: 1.0, contactScore: 0.0 },
				},
			],
			failed: [{ bankTransactionId: 2, errorCode: 'PERIOD_LOCKED', details: null }],
		} satisfies AcceptResponse);

		const { findAllByTestId, findByTestId, getByTestId, queryByTestId } = render(
			ReconciliationProposals,
			{ bankAccountId: 17 },
		);
		for (const cb of await findAllByTestId('tx-checkbox')) await fireEvent.click(cb);
		await fireEvent.click(getByTestId('reconciliation-accept-btn'));
		await findByTestId('reconciliation-failed-item');
		expect(getByTestId('reconciliation-success')).toBeTruthy();

		await fireEvent.click((await findAllByTestId(bouton))[0]);
		await fireEvent.click(await findByTestId('stub-modal-success'));

		await vi.waitFor(() => {
			expect(queryByTestId('reconciliation-failed')).toBeNull();
			expect(queryByTestId('reconciliation-success')).toBeNull();
		});
	});
});
