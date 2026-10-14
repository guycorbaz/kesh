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
import { tick } from 'svelte';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import type {
	GetProposalsResponse,
	AcceptResponse,
	RejectResponse,
} from './reconciliation.types';

// Mock du module API : doit être défini AVANT l'import du composant
// (hoisting Vitest applique vi.mock avant les imports).
// Story 29-9 (#649, AC9 étape 0) — `importOriginal` : le composant importe aussi les
// constantes `PROPOSALS_DEFAULT_LIMIT` / `PROPOSALS_MAX_LIMIT`, qu'une fabrique sans
// l'original n'exporterait pas (Vitest lèverait à l'accès). Les fonctions restent des
// doublures ; seules les constantes sont les vraies.
vi.mock('./reconciliation.api', async (importOriginal) => ({
	...(await importOriginal<typeof import('./reconciliation.api')>()),
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

// Story 15-6d (AC4, test 11) — le composant résout le compte comptable du compte bancaire par
// `listBankAccounts` ; sans ce mock, l'appel partirait sur l'`apiClient` réel.
vi.mock('$lib/features/bank-accounts/bank-accounts.api', () => ({
	listBankAccounts: vi.fn().mockResolvedValue([]),
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
// Story 29-8 (#670) — la modale de rattachement, même doublure.
vi.mock('./LinkSettlementModal.svelte', async () => ({
	default: (await import('./ModalSuccessStub.test.svelte')).default,
}));

// Mock i18n util pour rendre le composant déterministe (renvoie le
// fallback fourni en deuxième argument, variables `{ $x }` interpolées).
vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string, args?: Record<string, string | number>) =>
		args ? fallback.replace(/\{\s*\$(\w+)\s*\}/g, (_, k) => String(args[k] ?? '')) : fallback,
}));

import * as api from './reconciliation.api';
import * as bankAccountsApi from '$lib/features/bank-accounts/bank-accounts.api';
import type { BankAccountSummary } from '$lib/features/bank-accounts/bank-accounts.api';
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

		// Story 29-6 (#665, AC10) — une tx sans candidate porte une case, comme
		// les autres : elle se coche pour être rejetée (le γ refactor la rendait
		// non sélectionnable, ce qui rendait le rejet impraticable).
		const checkboxes = queryAllByTestId('tx-checkbox');
		expect(checkboxes).toHaveLength(1);
		expect(checkboxes[0].getAttribute('data-tx-id')).toBe('2');
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
		expect(item.textContent).toContain(
			'Transaction introuvable : elle a été supprimée avec son import, ou n’appartient pas à ce compte bancaire.'
		);
		expect(item.textContent).not.toContain('ACME GMBH');
	});

	// Story 27-4 (AC8, #634) — l'import d'une transaction cochée a été supprimé avant le lot : le
	// serveur rend `200`, la transaction supprimée sort en `failed[]` et l'autre est acceptée.
	// L'écran le dit ligne par ligne, par la date et le montant RELEVÉS avant le lot, sans
	// bandeau d'erreur.
	it('désigne par sa date et son montant relevés une transaction supprimée avec son import', async () => {
		const kept = makeProposalWithCandidate(1, 101, 1.0);
		const base = makeProposalWithCandidate(2, 102, 1.0);
		const deleted = {
			...base,
			transaction: { ...base.transaction, bookingDate: '2026-05-20', amount: '250.00' },
		};
		mockApi.getProposals
			.mockResolvedValueOnce({ proposals: [kept, deleted], hasMore: false } satisfies GetProposalsResponse)
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
			failed: [{ bankTransactionId: 2, errorCode: 'BANK_TRANSACTION_NOT_FOUND', details: null }],
		} satisfies AcceptResponse);

		const { findAllByTestId, findByTestId, getByTestId, queryByTestId } = render(
			ReconciliationProposals,
			{ bankAccountId: 17 }
		);
		for (const cb of await findAllByTestId('tx-checkbox')) await fireEvent.click(cb);
		await fireEvent.click(getByTestId('reconciliation-accept-btn'));

		expect((await findByTestId('reconciliation-success')).textContent).toContain('1');
		const item = await findByTestId('reconciliation-failed-item');
		expect(item.getAttribute('data-tx-id')).toBe('2');
		expect(item.getAttribute('data-error-code')).toBe('BANK_TRANSACTION_NOT_FOUND');
		const text = item.textContent ?? '';
		expect(text).toContain('2026-05-20');
		expect(text).toContain('250.00 CHF');
		expect(text).not.toContain('TX #');
		expect(text).toContain('supprimée avec son import');
		expect(queryByTestId('reconciliation-error')).toBeNull();
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

// Story 15-6d (AC4, test 11, #524, choix C-15-6d-1) — le `journalAccountId` du compte bancaire
// monté est passé à la modale d'affectation manuelle (`bankLedgerAccountId`), qui l'écarte des
// contreparties (test 10, `ManualMatchModal.test.ts`). Repli : échec de l'appel, compte absent
// ou non lié → `null`, aucun filtrage, aucune erreur affichée.
describe('ReconciliationProposals — compte de la banque passé à la modale (Story 15-6d)', () => {
	const mockBankAccounts = vi.mocked(bankAccountsApi);

	function bankAccount(id: number, journalAccountId: number | null): BankAccountSummary {
		return {
			id,
			bankName: 'UBS',
			description: null,
			iban: `CH00${id}`,
			qrIban: null,
			isPrimary: id === 17,
			journalAccountId,
			version: 1,
			archived: false,
			currentBalance: null,
			lastTransactionDate: null,
		} as BankAccountSummary;
	}

	beforeEach(() => {
		vi.clearAllMocks();
		mockApi.getProposals.mockResolvedValue({
			proposals: [makeProposalWithoutCandidate(1)],
			hasMore: false,
		} satisfies GetProposalsResponse);
	});

	async function ledgerPassedToModal(): Promise<{ value: string | null; errorShown: boolean }> {
		const { findByTestId, queryByTestId, unmount } = render(ReconciliationProposals, {
			bankAccountId: 17,
		});
		await fireEvent.click(await findByTestId('manual-match-button'));
		const stub = await findByTestId('stub-modal-success');
		// Laisse se résoudre la promesse de `listBankAccounts` avant de lire la prop.
		await vi.waitFor(() => expect(mockBankAccounts.listBankAccounts).toHaveBeenCalled());
		await new Promise((r) => setTimeout(r, 0));
		const value = stub.getAttribute('data-bank-ledger-account-id');
		const errorShown = queryByTestId('reconciliation-error') !== null;
		unmount();
		return { value, errorShown };
	}

	it('passe le journalAccountId du compte bancaire monté', async () => {
		mockBankAccounts.listBankAccounts.mockResolvedValue([bankAccount(18, 43), bankAccount(17, 42)]);
		const r = await ledgerPassedToModal();
		expect(r.value).toBe('42');
		expect(mockBankAccounts.listBankAccounts).toHaveBeenCalledTimes(1);
	});

	it('listBankAccounts en échec → null (aucun filtrage), aucune erreur affichée', async () => {
		mockBankAccounts.listBankAccounts.mockRejectedValue(new Error('boom'));
		const r = await ledgerPassedToModal();
		expect(r.value).toBe('null');
		expect(r.errorShown).toBe(false);
	});

	it.each([
		['absent de la liste (archivé)', [bankAccount(18, 43)]],
		['non lié', [bankAccount(17, null)]],
	])('compte bancaire %s → null', async (_cas, list) => {
		mockBankAccounts.listBankAccounts.mockResolvedValue(list);
		expect((await ledgerPassedToModal()).value).toBe('null');
	});
});

// Story 28-5b (#526, AC6, V4) — l'indication « où retrouver une transaction rejetée » est rendue
// en fin de section, HORS des branches de la liste : un lot qui rejette la dernière ligne vide la
// liste, et c'est précisément là qu'elle sert.
describe('ReconciliationProposals — indication des transactions rejetées (Story 28-5b)', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	const HINT = 'Une transaction rejetée quitte cette liste.';

	it('liste vide ⇒ l’indication est rendue (mutation M12 : indication dans la branche « liste non vide »)', async () => {
		mockApi.getProposals.mockResolvedValue({ proposals: [], hasMore: false } satisfies GetProposalsResponse);
		const { findByTestId } = render(ReconciliationProposals, { bankAccountId: 17 });
		await findByTestId('reconciliation-empty');
		expect((await findByTestId('reconciliation-rejected-hint')).textContent).toContain(HINT);
	});

	it('liste non vide ⇒ l’indication est rendue, et son repli égale le catalogue fr-CH (revue P1, E-4)', async () => {
		mockApi.getProposals.mockResolvedValue({
			proposals: [makeProposalWithCandidate(1, 101, 1.0)],
			hasMore: false,
		} satisfies GetProposalsResponse);
		const { findAllByTestId, getByTestId } = render(ReconciliationProposals, { bankAccountId: 17 });
		await findAllByTestId('reconciliation-row');
		const texte = (getByTestId('reconciliation-rejected-hint').textContent ?? '').trim();
		expect(texte).toContain(HINT);
		// Le mock d'`i18nMsg` rend le repli : on le compare mot pour mot à la valeur fr-CH.
		const prefixe = 'reconciliation-labels-rejected-hint = ';
		const attendu = readFileSync(join('../crates/kesh-i18n/locales', 'fr-CH', 'messages.ftl'), 'utf-8')
			.split('\n')
			.find((l) => l.startsWith(prefixe))
			?.slice(prefixe.length);
		expect(attendu).toBeDefined();
		expect(texte).toBe(attendu);
	});

	it('liste en erreur ⇒ l’indication est rendue', async () => {
		mockApi.getProposals.mockRejectedValue({ code: 'INTERNAL_ERROR', message: 'Panne.', status: 500 });
		const { findByTestId } = render(ReconciliationProposals, { bankAccountId: 17 });
		await findByTestId('reconciliation-error');
		expect((await findByTestId('reconciliation-rejected-hint')).textContent).toContain(HINT);
	});

	it('avec des échecs partiels ⇒ l’indication vient APRÈS leur bloc', async () => {
		mockApi.getProposals
			.mockResolvedValueOnce({
				proposals: [makeProposalWithCandidate(7, 200, 0.5)],
				hasMore: false,
			} satisfies GetProposalsResponse)
			.mockResolvedValueOnce({ proposals: [], hasMore: false } satisfies GetProposalsResponse);
		mockApi.rejectProposals.mockResolvedValue({
			rejected: [],
			failed: [{ bankTransactionId: 7, errorCode: 'RECONCILIATION_ALREADY_REJECTED', details: null }],
		} satisfies RejectResponse);
		const { findByTestId, getByTestId } = render(ReconciliationProposals, { bankAccountId: 17 });
		await fireEvent.click(await findByTestId('tx-checkbox'));
		await fireEvent.click(getByTestId('reconciliation-reject-btn'));
		const bloc = await findByTestId('reconciliation-failed');
		const hint = getByTestId('reconciliation-rejected-hint');
		expect(bloc.compareDocumentPosition(hint) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
		// Le refus d'un lot dit désormais où annuler le rejet (AC7 b).
		expect((await findByTestId('reconciliation-failed-item')).textContent).toContain(
			'Son rejet s’annule depuis le détail de son import',
		);
	});
});

// Story 29-3 (#527) — l'écran obéit à l'ordre reçu (`candidates[0]`), sans le refaire : c'est le
// serveur qui l'écrit (`order_candidates`). Aucun test ne construisait jusqu'ici une candidate
// RÈGLE : la branche règle d'`onAccept` et le badge n'étaient éprouvés nulle part (AC4).
type Candidate = GetProposalsResponse['proposals'][number]['candidates'][number];

function invoiceCandidate(invoiceId: number): Candidate {
	return makeProposalWithCandidate(0, invoiceId, 0.1).candidates[0];
}

function ruleCandidate(ruleId: number, counterpartyAccountId: number): Candidate {
	return {
		candidateType: 'rule' as const,
		invoiceId: null,
		invoiceNumber: null,
		invoiceAmount: null,
		invoiceTotalTtc: null,
		invoiceDate: null,
		ruleId,
		ruleLabel: 'Dubois',
		ruleMatchType: 'counterparty_contains',
		counterpartyAccountId,
		counterpartyAccountName: '3000 Ventes',
		score: { total: 1.0, amountScore: 0.0, referenceScore: 0.0, contactScore: 0.0 },
	};
}

function makeProposalWithCandidates(
	bankTransactionId: number,
	candidates: Candidate[],
): GetProposalsResponse['proposals'][number] {
	return { ...makeProposalWithCandidate(bankTransactionId, 0, 0.1), candidates };
}

describe('ReconciliationProposals — la candidate affichée et acceptée est candidates[0] (Story 29-3)', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		mockApi.acceptProposals.mockResolvedValue({ accepted: [], failed: [] } satisfies AcceptResponse);
	});

	async function acceptTheOnlyRow(candidates: Candidate[]) {
		mockApi.getProposals.mockResolvedValue({
			proposals: [makeProposalWithCandidates(42, candidates)],
			hasMore: false,
		} satisfies GetProposalsResponse);
		const view = render(ReconciliationProposals, { bankAccountId: 17 });
		await fireEvent.click(await view.findByTestId('tx-checkbox'));
		const badge = view.queryByTestId('candidate-rule-badge');
		await fireEvent.click(view.getByTestId('reconciliation-accept-btn'));
		expect(mockApi.acceptProposals).toHaveBeenCalledTimes(1);
		return { badge, items: mockApi.acceptProposals.mock.calls[0][1] };
	}

	it('[règle, facture] ⇒ badge Règle, et « Accepter » envoie la règle, jamais la facture', async () => {
		const { badge, items } = await acceptTheOnlyRow([ruleCandidate(9, 3000), invoiceCandidate(101)]);
		expect(badge).not.toBeNull();
		expect(items).toEqual([
			{ type: 'rule', bankTransactionId: 42, ruleId: 9, counterpartyAccountId: 3000 },
		]);
	});

	it('[facture, règle] ⇒ pas de badge Règle, et « Accepter » envoie la facture', async () => {
		const { badge, items } = await acceptTheOnlyRow([invoiceCandidate(101), ruleCandidate(9, 3000)]);
		expect(badge).toBeNull();
		expect(items).toEqual([{ type: 'invoice', bankTransactionId: 42, invoiceId: 101 }]);
	});
});

// Story 29-3 (#527, AC5) — la page dit, au-dessus de la liste, quelle proposition elle affiche ;
// seulement si une ligne au moins a une candidate (C-29-3-14).
describe('ReconciliationProposals — la phrase de priorité (Story 29-3)', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	const PRIORITY = 'Une facture proposée passe toujours devant une règle.';

	it('une ligne a une candidate ⇒ la phrase est rendue, avant la table, et son repli égale le catalogue fr-CH', async () => {
		mockApi.getProposals.mockResolvedValue({
			proposals: [makeProposalWithoutCandidate(2), makeProposalWithCandidate(1, 101, 0.1)],
			hasMore: false,
		} satisfies GetProposalsResponse);
		const { findAllByTestId, getByTestId } = render(ReconciliationProposals, { bankAccountId: 17 });
		await findAllByTestId('reconciliation-row');
		const hint = getByTestId('reconciliation-priority-hint');
		const texte = (hint.textContent ?? '').trim();
		expect(texte).toContain(PRIORITY);
		expect(
			hint.compareDocumentPosition(getByTestId('reconciliation-table')) &
				Node.DOCUMENT_POSITION_FOLLOWING,
		).toBeTruthy();
		const prefixe = 'reconciliation-labels-priority-hint = ';
		const attendu = readFileSync(join('../crates/kesh-i18n/locales', 'fr-CH', 'messages.ftl'), 'utf-8')
			.split('\n')
			.find((l) => l.startsWith(prefixe))
			?.slice(prefixe.length);
		expect(attendu).toBeDefined();
		expect(texte).toBe(attendu);
	});

	it('liste vide ⇒ pas de phrase (mutation M9)', async () => {
		mockApi.getProposals.mockResolvedValue({ proposals: [], hasMore: false } satisfies GetProposalsResponse);
		const { findByTestId, queryByTestId } = render(ReconciliationProposals, { bankAccountId: 17 });
		await findByTestId('reconciliation-empty');
		expect(queryByTestId('reconciliation-priority-hint')).toBeNull();
	});

	it('liste en erreur ⇒ pas de phrase', async () => {
		mockApi.getProposals.mockRejectedValue({ code: 'INTERNAL_ERROR', message: 'Panne.', status: 500 });
		const { findByTestId, queryByTestId } = render(ReconciliationProposals, { bankAccountId: 17 });
		await findByTestId('reconciliation-error');
		expect(queryByTestId('reconciliation-priority-hint')).toBeNull();
	});

	it('rechargement en erreur après une liste à candidates ⇒ pas de phrase (mutation M9b : phrase hors de la branche)', async () => {
		// Le rechargement qui suit « Accepter » échoue : `proposals` garde les lignes du premier
		// chargement, et seule la branche « liste non vide » retient la phrase.
		mockApi.getProposals
			.mockResolvedValueOnce({
				proposals: [makeProposalWithCandidate(1, 101, 0.1)],
				hasMore: false,
			} satisfies GetProposalsResponse)
			.mockRejectedValueOnce({ code: 'INTERNAL_ERROR', message: 'Panne.', status: 500 });
		mockApi.acceptProposals.mockResolvedValue({ accepted: [], failed: [] } satisfies AcceptResponse);
		const { findByTestId, getByTestId, queryByTestId } = render(ReconciliationProposals, {
			bankAccountId: 17,
		});
		await fireEvent.click(await findByTestId('tx-checkbox'));
		expect(getByTestId('reconciliation-priority-hint')).not.toBeNull();
		await fireEvent.click(getByTestId('reconciliation-accept-btn'));
		await findByTestId('reconciliation-error');
		expect(queryByTestId('reconciliation-priority-hint')).toBeNull();
	});

	it('aucune ligne n’a de candidate ⇒ pas de phrase (mutation M9)', async () => {
		mockApi.getProposals.mockResolvedValue({
			proposals: [makeProposalWithoutCandidate(1), makeProposalWithoutCandidate(2)],
			hasMore: false,
		} satisfies GetProposalsResponse);
		const { findAllByTestId, queryByTestId } = render(ReconciliationProposals, { bankAccountId: 17 });
		expect(await findAllByTestId('reconciliation-row')).toHaveLength(2);
		expect(queryByTestId('reconciliation-priority-hint')).toBeNull();
	});
});

// Story 29-6 (#665, AC10) — une transaction sans proposition se coche et se rejette ;
// « Accepter » ne la traite pas : il est grisé, et la rangée des boutons le dit.
//
// ⚠️ Un état grisé ou actif se prouve par l'ATTRIBUT `disabled` : `fireEvent.click` livre
// l'événement à un bouton désactivé (jsdom ne bloque que `click()`, et Svelte 5 délègue
// `onclick`) — précédent `DeleteBankImportAction.test.ts`.
describe('ReconciliationProposals — une ligne sans proposition se rejette (Story 29-6)', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	const HINT_TESTID = 'reconciliation-unmatched-not-acceptable-hint';

	function checkboxOf(container: HTMLElement, txId: number): HTMLElement {
		const box = container.querySelector<HTMLElement>(
			`[data-testid="tx-checkbox"][data-tx-id="${txId}"]`,
		);
		if (!box) throw new Error(`pas de case pour la transaction ${txId}`);
		return box;
	}

	it('cochée seule ⇒ Rejeter actif et l’envoie, Accepter grisé, indication rendue (M10, M13)', async () => {
		mockApi.getProposals.mockResolvedValue({
			proposals: [makeProposalWithoutCandidate(2)],
			hasMore: false,
		} satisfies GetProposalsResponse);
		mockApi.rejectProposals.mockResolvedValue({
			rejected: [{ bankTransactionId: 2, rejectedAt: '2026-05-15T00:00:00Z' }],
			failed: [],
		} satisfies RejectResponse);
		const { findAllByTestId, getByTestId, container } = render(ReconciliationProposals, {
			bankAccountId: 17,
		});
		await findAllByTestId('reconciliation-row');
		await fireEvent.click(checkboxOf(container, 2));

		const accept = getByTestId('reconciliation-accept-btn') as HTMLButtonElement;
		const reject = getByTestId('reconciliation-reject-btn') as HTMLButtonElement;
		expect(accept.disabled).toBe(true);
		expect(reject.disabled).toBe(false);
		expect(getByTestId(HINT_TESTID)).not.toBeNull();

		await fireEvent.click(reject);
		expect(mockApi.rejectProposals).toHaveBeenCalledTimes(1);
		expect(mockApi.rejectProposals).toHaveBeenCalledWith(17, [2]);
	});

	it('clic sur Accepter grisé ⇒ ni erreur, ni appel, la table reste (M12 : garde de tête)', async () => {
		mockApi.getProposals.mockResolvedValue({
			proposals: [makeProposalWithoutCandidate(2)],
			hasMore: false,
		} satisfies GetProposalsResponse);
		const { findAllByTestId, getByTestId, queryByTestId, queryAllByTestId, container } = render(
			ReconciliationProposals,
			{ bankAccountId: 17 },
		);
		await findAllByTestId('reconciliation-row');
		await fireEvent.click(checkboxOf(container, 2));
		await fireEvent.click(getByTestId('reconciliation-accept-btn'));

		expect(queryByTestId('reconciliation-error')).toBeNull();
		expect(mockApi.acceptProposals).not.toHaveBeenCalled();
		expect(queryAllByTestId('tx-checkbox')).toHaveLength(1);
	});

	it('avec et sans proposition cochées ⇒ Accepter grisé ; décocher la seconde ⇒ actif, n’envoie que la première', async () => {
		mockApi.getProposals.mockResolvedValue({
			proposals: [makeProposalWithCandidate(1, 101, 1.0), makeProposalWithoutCandidate(2)],
			hasMore: false,
		} satisfies GetProposalsResponse);
		mockApi.acceptProposals.mockResolvedValue({ accepted: [], failed: [] } satisfies AcceptResponse);
		const { findAllByTestId, getByTestId, queryByTestId, container } = render(
			ReconciliationProposals,
			{ bankAccountId: 17 },
		);
		await findAllByTestId('reconciliation-row');
		await fireEvent.click(checkboxOf(container, 1));
		await fireEvent.click(checkboxOf(container, 2));

		const accept = getByTestId('reconciliation-accept-btn') as HTMLButtonElement;
		expect(accept.disabled).toBe(true);
		expect(getByTestId(HINT_TESTID)).not.toBeNull();

		await fireEvent.click(checkboxOf(container, 2));
		expect((getByTestId('reconciliation-accept-btn') as HTMLButtonElement).disabled).toBe(false);
		expect(queryByTestId(HINT_TESTID)).toBeNull();

		await fireEvent.click(getByTestId('reconciliation-accept-btn'));
		expect(mockApi.acceptProposals).toHaveBeenCalledWith(17, [
			{ type: 'invoice', bankTransactionId: 1, invoiceId: 101 },
		]);
	});

	it('seules des lignes avec proposition cochées ⇒ pas d’indication (M11)', async () => {
		mockApi.getProposals.mockResolvedValue({
			proposals: [makeProposalWithCandidate(1, 101, 1.0), makeProposalWithoutCandidate(2)],
			hasMore: false,
		} satisfies GetProposalsResponse);
		const { findAllByTestId, queryByTestId, container } = render(ReconciliationProposals, {
			bankAccountId: 17,
		});
		await findAllByTestId('reconciliation-row');
		expect(queryByTestId(HINT_TESTID)).toBeNull();
		await fireEvent.click(checkboxOf(container, 1));
		expect(queryByTestId(HINT_TESTID)).toBeNull();
	});

	it('l’indication est dans la rangée des boutons, et son repli égale le catalogue fr-CH', async () => {
		mockApi.getProposals.mockResolvedValue({
			proposals: [makeProposalWithoutCandidate(2)],
			hasMore: false,
		} satisfies GetProposalsResponse);
		const { findAllByTestId, getByTestId, container } = render(ReconciliationProposals, {
			bankAccountId: 17,
		});
		await findAllByTestId('reconciliation-row');
		await fireEvent.click(checkboxOf(container, 2));
		const hint = getByTestId(HINT_TESTID);
		// C-29-6-18 : dernier enfant de la rangée, après « Rejeter ».
		expect(hint.parentElement).toBe(getByTestId('reconciliation-reject-btn').parentElement);
		expect(hint.parentElement?.lastElementChild).toBe(hint);
		const prefixe = 'reconciliation-labels-unmatched-not-acceptable-hint = ';
		const attendu = readFileSync(join('../crates/kesh-i18n/locales', 'fr-CH', 'messages.ftl'), 'utf-8')
			.split('\n')
			.find((l) => l.startsWith(prefixe))
			?.slice(prefixe.length);
		expect(attendu).toBeDefined();
		expect((hint.textContent ?? '').trim()).toBe(attendu);
	});
});

// Story 29-8 (#670, AC4, C-29-8-14) — le bouton « Rattacher à un règlement », sur les seules
// lignes de crédit en CHF (miroir de la condition 1), et le bilan après un rattachement.
describe('ReconciliationProposals — rattacher à un règlement (Story 29-8)', () => {
	function ligne(
		id: number,
		amount: string,
		currency = 'CHF',
	): GetProposalsResponse['proposals'][number] {
		const p = makeProposalWithoutCandidate(id);
		return { ...p, transaction: { ...p.transaction, amount, currency } };
	}

	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('le bouton est sur une ligne de crédit en CHF, et absent sur un débit, un montant nul ou une autre devise (M12)', async () => {
		mockApi.getProposals.mockResolvedValue({
			proposals: [ligne(1, '100.03'), ligne(2, '-50.00'), ligne(3, '0.00'), ligne(4, '20.00', 'EUR')],
			hasMore: false,
		} satisfies GetProposalsResponse);
		const { findAllByTestId } = render(ReconciliationProposals, { bankAccountId: 17 });
		await findAllByTestId('reconciliation-row');
		const boutons = await findAllByTestId('link-settlement-button');
		expect(boutons.map((b) => b.getAttribute('data-tx-id'))).toEqual(['1']);
	});

	it('un rattachement réussi efface le bilan du lot précédent, recharge et affiche « 1 opération(s) réussie(s). »', async () => {
		mockApi.getProposals
			.mockResolvedValueOnce({
				proposals: [makeProposalWithCandidate(1, 101, 1.0), ligne(2, '100.03')],
				hasMore: false,
			} satisfies GetProposalsResponse)
			.mockResolvedValue({ proposals: [ligne(2, '100.03')], hasMore: false } satisfies GetProposalsResponse);
		mockApi.rejectProposals.mockResolvedValue({
			rejected: [],
			failed: [{ bankTransactionId: 1, errorCode: 'PERIOD_LOCKED', details: null }],
		} satisfies RejectResponse);
		const { findAllByTestId, findByTestId, getByTestId, queryByTestId } = render(
			ReconciliationProposals,
			{ bankAccountId: 17 },
		);
		await fireEvent.click((await findAllByTestId('tx-checkbox'))[0]);
		await fireEvent.click(getByTestId('reconciliation-reject-btn'));
		await findByTestId('reconciliation-failed-item');

		const appelsAvant = mockApi.getProposals.mock.calls.length;
		await fireEvent.click(await findByTestId('link-settlement-button'));
		await fireEvent.click(await findByTestId('stub-modal-success'));
		await vi.waitFor(() => {
			expect(queryByTestId('reconciliation-failed')).toBeNull();
			expect(getByTestId('reconciliation-success').textContent).toContain('1');
			expect(getByTestId('reconciliation-success').textContent).toContain(
				'opération(s) réussie(s).',
			);
		});
		expect(mockApi.getProposals.mock.calls.length).toBeGreaterThan(appelsAvant);
	});
});

// Story 29-9 (#649, AC1-AC3) — s'il reste des transactions au-delà de la liste (`hasMore`),
// l'écran le dit et offre de la porter au plafond (500) ; au plafond, un avis sans bouton.
describe('ReconciliationProposals — la suite de la liste (Story 29-9, #649)', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	/** `n` lignes sans proposition, identifiants `from`, `from + 1`, … */
	function lignes(n: number, from = 1): GetProposalsResponse['proposals'] {
		return Array.from({ length: n }, (_, i) => makeProposalWithoutCandidate(from + i));
	}

	it('V1 — hasMore faux ⇒ ni avis, ni bouton, ni avis plafonné', async () => {
		mockApi.getProposals.mockResolvedValue({ proposals: lignes(3), hasMore: false } satisfies GetProposalsResponse);
		const { findAllByTestId, queryByTestId } = render(ReconciliationProposals, { bankAccountId: 17 });
		await findAllByTestId('reconciliation-row');
		expect(queryByTestId('reconciliation-more')).toBeNull();
		expect(queryByTestId('reconciliation-show-older-btn')).toBeNull();
		expect(queryByTestId('reconciliation-more-capped')).toBeNull();
	});

	it('V2 — hasMore vrai ⇒ l’avis dit « 100 », le bouton est là, et la liste a été demandée à 100 (M1)', async () => {
		mockApi.getProposals.mockResolvedValue({ proposals: lignes(100), hasMore: true } satisfies GetProposalsResponse);
		const { findByTestId, getByTestId } = render(ReconciliationProposals, { bankAccountId: 17 });
		const avis = await findByTestId('reconciliation-more');
		expect(avis.textContent).toContain('100');
		expect(getByTestId('reconciliation-show-older-btn').textContent).toContain('500');
		expect(mockApi.getProposals).toHaveBeenCalledWith(17, 100);
	});

	it('V3 — un clic recharge au plafond, en UNE requête (M2, M3, M5)', async () => {
		mockApi.getProposals
			.mockResolvedValueOnce({ proposals: lignes(100), hasMore: true } satisfies GetProposalsResponse)
			.mockResolvedValue({ proposals: lignes(150), hasMore: false } satisfies GetProposalsResponse);
		const { findByTestId, findAllByTestId } = render(ReconciliationProposals, { bankAccountId: 17 });
		await fireEvent.click(await findByTestId('reconciliation-show-older-btn'));
		await vi.waitFor(async () => expect(await findAllByTestId('reconciliation-row')).toHaveLength(150));
		// Une requête de trop (M3 : le bouton appelle aussi `load()`) part au tick suivant : compter après lui.
		await tick();
		expect(mockApi.getProposals).toHaveBeenCalledTimes(2);
		expect(mockApi.getProposals.mock.calls[1]).toEqual([17, 500]);
	});

	it('V4 — au plafond, il en reste encore ⇒ avis plafonné, aucun bouton, et il ne conseille pas le rejet (M4)', async () => {
		mockApi.getProposals
			.mockResolvedValueOnce({ proposals: lignes(100), hasMore: true } satisfies GetProposalsResponse)
			.mockResolvedValue({ proposals: lignes(500), hasMore: true } satisfies GetProposalsResponse);
		const { findByTestId, queryByTestId } = render(ReconciliationProposals, { bankAccountId: 17 });
		await fireEvent.click(await findByTestId('reconciliation-show-older-btn'));
		const capped = await findByTestId('reconciliation-more-capped');
		expect(capped.textContent).toContain('500');
		expect(capped.textContent?.toLowerCase()).not.toContain('rejet');
		expect(queryByTestId('reconciliation-show-older-btn')).toBeNull();
		expect(queryByTestId('reconciliation-more')).toBeNull();
	});

	it('V5 — après le clic, un « Accepter » réussi relit la liste au plafond, pas à 100 (M5)', async () => {
		mockApi.getProposals
			.mockResolvedValueOnce({
				proposals: [makeProposalWithCandidate(1, 101, 1.0), ...lignes(99, 2)],
				hasMore: true,
			} satisfies GetProposalsResponse)
			.mockResolvedValue({
				proposals: [makeProposalWithCandidate(1, 101, 1.0), ...lignes(149, 2)],
				hasMore: false,
			} satisfies GetProposalsResponse);
		mockApi.acceptProposals.mockResolvedValue({ accepted: [], failed: [] } satisfies AcceptResponse);
		const { findByTestId, findAllByTestId, getByTestId, container } = render(ReconciliationProposals, {
			bankAccountId: 17,
		});
		await fireEvent.click(await findByTestId('reconciliation-show-older-btn'));
		await vi.waitFor(async () => expect(await findAllByTestId('reconciliation-row')).toHaveLength(150));
		const box = container.querySelector<HTMLElement>('[data-testid="tx-checkbox"][data-tx-id="1"]');
		if (!box) throw new Error('pas de case pour la transaction 1');
		await fireEvent.click(box);
		await fireEvent.click(getByTestId('reconciliation-accept-btn'));
		await vi.waitFor(() => expect(mockApi.getProposals).toHaveBeenCalledTimes(3));
		expect(mockApi.getProposals.mock.calls[2]).toEqual([17, 500]);
	});

	it('V12 — hasMore retombe après un lot ⇒ ni avis, ni bouton, ni avis plafonné (M14)', async () => {
		mockApi.getProposals
			.mockResolvedValueOnce({
				proposals: [makeProposalWithCandidate(1, 101, 1.0), ...lignes(99, 2)],
				hasMore: true,
			} satisfies GetProposalsResponse)
			.mockResolvedValue({ proposals: lignes(99, 2), hasMore: false } satisfies GetProposalsResponse);
		mockApi.acceptProposals.mockResolvedValue({ accepted: [], failed: [] } satisfies AcceptResponse);
		const { findByTestId, findAllByTestId, getByTestId, queryByTestId, container } = render(
			ReconciliationProposals,
			{ bankAccountId: 17 },
		);
		await findByTestId('reconciliation-more');
		const box = container.querySelector<HTMLElement>('[data-testid="tx-checkbox"][data-tx-id="1"]');
		if (!box) throw new Error('pas de case pour la transaction 1');
		await fireEvent.click(box);
		await fireEvent.click(getByTestId('reconciliation-accept-btn'));
		await vi.waitFor(async () => expect(await findAllByTestId('reconciliation-row')).toHaveLength(99));
		expect(queryByTestId('reconciliation-more')).toBeNull();
		expect(queryByTestId('reconciliation-show-older-btn')).toBeNull();
		expect(queryByTestId('reconciliation-more-capped')).toBeNull();
	});

	it('V18 — pendant un lot, le bouton est grisé et ne relance rien (M21)', async () => {
		mockApi.getProposals.mockResolvedValue({
			proposals: [makeProposalWithCandidate(1, 101, 1.0), ...lignes(99, 2)],
			hasMore: true,
		} satisfies GetProposalsResponse);
		let resolveAccept!: (r: AcceptResponse) => void;
		mockApi.acceptProposals.mockReturnValue(
			new Promise<AcceptResponse>((resolve) => {
				resolveAccept = resolve;
			}),
		);
		const { findByTestId, getByTestId, container } = render(ReconciliationProposals, {
			bankAccountId: 17,
		});
		await findByTestId('reconciliation-more');
		const box = container.querySelector<HTMLElement>('[data-testid="tx-checkbox"][data-tx-id="1"]');
		if (!box) throw new Error('pas de case pour la transaction 1');
		await fireEvent.click(box);
		await fireEvent.click(getByTestId('reconciliation-accept-btn'));
		const btn = getByTestId('reconciliation-show-older-btn') as HTMLButtonElement;
		expect(btn.disabled).toBe(true);
		expect(mockApi.getProposals).toHaveBeenCalledTimes(1);
		resolveAccept({ accepted: [], failed: [] });
		await vi.waitFor(() => expect(mockApi.getProposals).toHaveBeenCalledTimes(2));
	});
});
