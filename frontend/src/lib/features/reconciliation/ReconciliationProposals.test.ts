// Story 8-4 (FR44) — H1 Pass 1 code review : tests Vitest pour
// ReconciliationProposals.svelte. Couvre AC #55 (rendu propositions
// + score badges) et AC #58 (état neutre tx sans candidate).
//
// Pattern : mock du module `reconciliation.api` + render via
// `@testing-library/svelte` (Svelte 5 supporté par 5.3.1). On
// vérifie le contrat data-attributes (data-testid) et les flows
// onAccept top-1.

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, fireEvent, cleanup } from '@testing-library/svelte';
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
// Story 34-4, revue P1 (E1-3) : une enveloppe par modale, qui marque la doublure
// (`data-modal` = `manual`, `split`, `link`).
vi.mock('./ManualMatchModal.svelte', async () => ({
	default: (await import('./ModalSuccessStubManual.test.svelte')).default,
}));
vi.mock('./TransactionSplitModal.svelte', async () => ({
	default: (await import('./ModalSuccessStubSplit.test.svelte')).default,
}));
// Story 29-8 (#670) — la modale de rattachement, même doublure.
vi.mock('./LinkSettlementModal.svelte', async () => ({
	default: (await import('./ModalSuccessStubLink.test.svelte')).default,
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
				supplierInvoiceId: null,
				contactName: null,
				invoiceNumber: `INV-2026-${invoiceId}`,
				internalNumber: null,
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
		notice: null,
		alreadyBooked: null,
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
		notice: null,
		alreadyBooked: null,
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
					supplierInvoiceId: null,
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

	// Story 8-5a-base FR45 — bouton « Écriture manuelle » (geste rebaptisé par la 34-3) présent
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
		// 2 rows → 2 boutons « Écriture manuelle » (override workflow,
		// même quand candidate auto-proposée existe — cf. L66 Pass 1).
		expect(buttons).toHaveLength(2);
		// Story 34-3 (#967, AC-1) — le nom dit qu'il passe une écriture.
		expect(buttons[0].textContent?.trim()).toBe('Écriture manuelle');
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
		expect(text).toContain('15.05.2026'); // v0172 (#880) : forme suisse (était l’ISO)
		expect(text).not.toContain('2026-05-15');
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
		expect(text).toContain('15.05.2026'); // v0172 (#880) : forme suisse (était l’ISO)
		expect(text).not.toContain('2026-05-15');
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
					supplierInvoiceId: null,
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
					supplierInvoiceId: null,
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
		expect(text).toContain('20.05.2026'); // v0172 (#880) : forme suisse (était l’ISO)
		expect(text).not.toContain('2026-05-20');
		expect(text).toContain('250.00 CHF');
		expect(text).not.toContain('TX #');
		expect(text).toContain('supprimée avec son import');
		expect(queryByTestId('reconciliation-error')).toBeNull();
	});

	// Revue de code P1 (B1, E3) : une écriture manuelle ou un éclatement réussi ouvre un
	// nouveau bilan ; celui du lot précédent ne doit pas lui survivre.
	it.each([
		['une écriture manuelle', 'manual-match-button'],
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
					supplierInvoiceId: null,
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
// monté est passé au dialogue d'écriture manuelle (`bankLedgerAccountId`), qui l'écarte des
// contreparties (test 10, `ManualMatchModal.test.ts`). Repli : échec de l'appel, compte absent
// ou non lié → `null`, aucun filtrage, aucune erreur affichée.
// Story 34-3 (#967, AC-3, C-e342-2) — *Éclater* le reçoit aussi : le filtre des classes 5, 6 et
// 7, qui l'écartait par accident, est retiré.
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

	async function ledgerPassedToModal(
		bouton: 'manual-match-button' | 'split-button' = 'manual-match-button',
	): Promise<{ value: string | null; errorShown: boolean }> {
		const { findByTestId, queryByTestId, unmount } = render(ReconciliationProposals, {
			bankAccountId: 17,
		});
		await fireEvent.click(await findByTestId(bouton));
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

	it('Story 34-3 (AC-3) — Éclater reçoit le compte de la banque', async () => {
		mockBankAccounts.listBankAccounts.mockResolvedValue([bankAccount(18, 43), bankAccount(17, 42)]);
		const r = await ledgerPassedToModal('split-button');
		expect(r.value).toBe('42');
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
		supplierInvoiceId: null,
		contactName: null,
		invoiceNumber: null,
		internalNumber: null,
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
		// Story 29-9 (#678, V22) : l'indication est désormais DANS la région d'annonce, qui est
		// elle-même le dernier enfant de la rangée.
		const region = getByTestId('reconciliation-selection-status');
		const rangee = getByTestId('reconciliation-reject-btn').parentElement;
		expect(hint.parentElement).toBe(region);
		expect(region.parentElement).toBe(rangee);
		expect(rangee?.lastElementChild).toBe(region);
		const prefixe = 'reconciliation-labels-unmatched-not-acceptable-hint = ';
		const attendu = readFileSync(join('../crates/kesh-i18n/locales', 'fr-CH', 'messages.ftl'), 'utf-8')
			.split('\n')
			.find((l) => l.startsWith(prefixe))
			?.slice(prefixe.length);
		expect(attendu).toBeDefined();
		expect((hint.textContent ?? '').trim()).toBe(attendu);
	});
});

// Story 29-8 (#670, AC4, C-29-8-14) — le bouton de rattachement, et le bilan après un
// rattachement. Story v017-3a (#671, AC-13) : le bouton, renommé « Rattacher à une écriture
// existante », est sur toute ligne en CHF de montant non nul (miroir de
// `transaction_can_be_linked_to_entry`), débits compris.
describe('ReconciliationProposals — rattacher à une écriture existante (Stories 29-8, v017-3a)', () => {
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

	it('le bouton est sur une ligne en CHF non nulle, débit compris, et absent sur un montant nul ou une autre devise (v017-3a M14)', async () => {
		mockApi.getProposals.mockResolvedValue({
			proposals: [ligne(1, '100.03'), ligne(2, '-50.00'), ligne(3, '0.00'), ligne(4, '20.00', 'EUR')],
			hasMore: false,
		} satisfies GetProposalsResponse);
		const { findAllByTestId } = render(ReconciliationProposals, { bankAccountId: 17 });
		await findAllByTestId('reconciliation-row');
		const boutons = await findAllByTestId('link-settlement-button');
		expect(boutons.map((b) => b.getAttribute('data-tx-id'))).toEqual(['1', '2']);
		expect(boutons[1].textContent?.trim()).toBe('Rattacher à une écriture existante');
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

// Story 29-9 (#678, AC11) — chaque case nomme sa transaction pour un lecteur d'écran, et
// l'indication « ne peut pas être acceptée » (Story 29-6) vit dans une région d'annonce
// présente DÈS le rendu de la liste : une région insérée avec son contenu n'est pas annoncée.
describe('ReconciliationProposals — cases nommées et indication annoncée (Story 29-9, #678)', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	const HINT_TESTID = 'reconciliation-unmatched-not-acceptable-hint';
	const REGION_TESTID = 'reconciliation-selection-status';

	/** La valeur fr-CH d'une clé, lue au catalogue (C-29-9-25) et remplie de `args`. */
	function libelleFr(cle: string, args: Record<string, string>): string {
		const prefixe = `${cle} = `;
		const brut = readFileSync(join('../crates/kesh-i18n/locales', 'fr-CH', 'messages.ftl'), 'utf-8')
			.split('\n')
			.find((l) => l.startsWith(prefixe))
			?.slice(prefixe.length);
		if (brut === undefined) throw new Error(`clé absente du catalogue fr-CH : ${cle}`);
		return brut.replace(/\{\s*\$(\w+)\s*\}/g, (_, k) => args[k] ?? '');
	}

	function sansContrepartie(id: number, counterpartyName: string | null) {
		const p = makeProposalWithoutCandidate(id);
		return { ...p, transaction: { ...p.transaction, counterpartyName } };
	}

	it('V19 — chaque case porte le nom de sa transaction : date, montant et devise, contrepartie (M22)', async () => {
		mockApi.getProposals.mockResolvedValue({
			proposals: [makeProposalWithCandidate(1, 101, 1.0), makeProposalWithoutCandidate(2)],
			hasMore: false,
		} satisfies GetProposalsResponse);
		const { findAllByTestId, getByRole } = render(ReconciliationProposals, { bankAccountId: 17 });
		await findAllByTestId('reconciliation-row');
		const avec = getByRole('checkbox', {
			name: libelleFr('reconciliation-labels-select-transaction', {
				date: '15.05.2026' /* v0172 (#880, C-v0172b-11) */,
				amount: '100.00',
				currency: 'CHF',
				counterparty: 'ACME GMBH',
			}),
		});
		expect(avec.getAttribute('data-tx-id')).toBe('1');
		const sans = getByRole('checkbox', {
			name: libelleFr('reconciliation-labels-select-transaction', {
				date: '15.05.2026' /* v0172 (#880, C-v0172b-11) */,
				amount: '50.00',
				currency: 'CHF',
				counterparty: 'UNKNOWN',
			}),
		});
		expect(sans.getAttribute('data-tx-id')).toBe('2');
	});

	it('V20 — sans contrepartie (null ou vide), le nom ne la mentionne pas, sans virgule finale (M23, M29)', async () => {
		mockApi.getProposals.mockResolvedValue({
			proposals: [sansContrepartie(3, null), sansContrepartie(4, '')],
			hasMore: false,
		} satisfies GetProposalsResponse);
		const { findAllByTestId, container } = render(ReconciliationProposals, { bankAccountId: 17 });
		await findAllByTestId('reconciliation-row');
		const attendu = libelleFr('reconciliation-labels-select-transaction-no-counterparty', {
			date: '15.05.2026' /* v0172 (#880, C-v0172b-11) */,
			amount: '50.00',
			currency: 'CHF',
		});
		expect(attendu.trim().endsWith(',')).toBe(false);
		for (const id of [3, 4]) {
			const box = container.querySelector(`[data-testid="tx-checkbox"][data-tx-id="${id}"]`);
			expect(box?.getAttribute('aria-label')).toBe(attendu);
		}
	});

	it('V21 — la région d’annonce précède le changement : même nœud, vide, puis l’indication en elle (M24-M28)', async () => {
		mockApi.getProposals.mockResolvedValue({
			proposals: [makeProposalWithCandidate(1, 101, 1.0), makeProposalWithoutCandidate(2)],
			hasMore: false,
		} satisfies GetProposalsResponse);
		const { findAllByTestId, getByTestId, queryByTestId, container } = render(ReconciliationProposals, {
			bankAccountId: 17,
		});
		await findAllByTestId('reconciliation-row');
		const region = getByTestId(REGION_TESTID);
		expect(region.getAttribute('role')).toBe('status');
		expect(region.getAttribute('aria-live')).toBe('polite');
		expect(region.getAttribute('aria-atomic')).toBe('true');
		expect((region.textContent ?? '').trim()).toBe('');
		expect(queryByTestId(HINT_TESTID)).toBeNull();

		const box = container.querySelector<HTMLElement>('[data-testid="tx-checkbox"][data-tx-id="2"]');
		if (!box) throw new Error('pas de case pour la transaction 2');
		await fireEvent.click(box);
		const hint = getByTestId(HINT_TESTID);
		expect(getByTestId(REGION_TESTID)).toBe(region);
		expect(region.contains(hint)).toBe(true);

		await fireEvent.click(box);
		expect(getByTestId(REGION_TESTID)).toBe(region);
		expect((region.textContent ?? '').trim()).toBe('');
		expect(queryByTestId(HINT_TESTID)).toBeNull();
	});
});

// ============================================================
// Story 30-1b (#686, AC-b11) — les factures fournisseur d'un débit
// ============================================================

/** Une candidate facture fournisseur. */
function supplierCandidate(
	supplierInvoiceId: number,
	invoiceNumber: string | null,
	contactName: string | null,
	internalNumber: string | null = null,
): Candidate {
	return {
		candidateType: 'supplier_invoice' as const,
		invoiceId: null,
		supplierInvoiceId,
		contactName,
		invoiceNumber,
		internalNumber,
		invoiceAmount: '480',
		invoiceTotalTtc: null,
		invoiceDate: '2026-03-01',
		ruleId: null,
		ruleLabel: null,
		ruleMatchType: null,
		counterpartyAccountId: null,
		counterpartyAccountName: null,
		score: { total: 1.0, amountScore: 1.0, referenceScore: 1.0, contactScore: 1.0 },
	};
}

/** Une ligne de débit qui ne propose rien et porte la `notice` du lot du compte. */
function noticeRow(
	bankTransactionId: number,
	supplierInvoiceNumber: string | null,
	internalNumber: string | null = null,
): GetProposalsResponse['proposals'][number] {
	const p = makeProposalWithoutCandidate(bankTransactionId);
	return {
		...p,
		transaction: { ...p.transaction, amount: '-300' },
		notice: {
			code: 'supplier_invoice_in_payment_batch',
			supplierInvoiceId: 77,
			supplierInvoiceNumber,
			internalNumber,
			paymentBatchId: 12,
		},
	};
}

describe('ReconciliationProposals — factures fournisseur (Story 30-1b)', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		mockApi.acceptProposals.mockResolvedValue({ accepted: [], failed: [] } satisfies AcceptResponse);
	});

	// T-b26 — la candidate fournisseur : badge, fournisseur, numéro (à défaut #id), montant ;
	// la candidate client reste rendue comme avant.
	it('T-b26 — rend la candidate fournisseur (badge, fournisseur, numéro, montant) et la client inchangée', async () => {
		mockApi.getProposals.mockResolvedValue({
			proposals: [
				makeProposalWithCandidates(1, [supplierCandidate(55, 'FF-2026-17', 'Garage Bel-Air SA')]),
				makeProposalWithCandidates(2, [supplierCandidate(56, null, null)]),
				makeProposalWithCandidate(3, 101, 1.0),
			],
			hasMore: false,
		} satisfies GetProposalsResponse);
		const { findAllByTestId, container } = render(ReconciliationProposals, { bankAccountId: 17 });
		await findAllByTestId('reconciliation-row');
		const row = (id: number) =>
			container.querySelector<HTMLElement>(`[data-testid="reconciliation-row"][data-tx-id="${id}"]`)!;

		const first = row(1).querySelector('[data-testid="candidate-top-1"]')!;
		expect(first.getAttribute('data-candidate-type')).toBe('supplier_invoice');
		expect(first.getAttribute('data-supplier-invoice-id')).toBe('55');
		expect(first.textContent).toContain('Garage Bel-Air SA');
		expect(first.textContent).toContain('FF-2026-17');
		expect(first.textContent).toContain('480');
		expect(row(1).querySelector('[data-testid="candidate-supplier-invoice-badge"]')?.textContent).toContain(
			'Facture fournisseur',
		);

		const second = row(2).querySelector('[data-testid="candidate-top-1"]')!;
		expect(second.textContent).toContain('#56');

		const client = row(3).querySelector('[data-testid="candidate-top-1"]')!;
		expect(client.getAttribute('data-candidate-type')).toBe('invoice');
		expect(client.getAttribute('data-invoice-id')).toBe('101');
		expect(client.textContent).toContain('INV-2026-101');
		expect(row(3).querySelector('[data-testid="candidate-supplier-invoice-badge"]')).toBeNull();
	});

	// T-b27 — le corps d'« Accepter » pour chaque type de candidate (facture client, règle,
	// facture fournisseur), dans un même lot, dans l'ordre de la sélection.
	it('T-b27 — « Accepter » envoie le type de chaque candidate, fournisseur compris', async () => {
		mockApi.getProposals.mockResolvedValue({
			proposals: [
				makeProposalWithCandidates(1, [supplierCandidate(55, 'FF-1', 'Garage')]),
				makeProposalWithCandidates(2, [invoiceCandidate(101)]),
				makeProposalWithCandidates(3, [ruleCandidate(9, 3000)]),
			],
			hasMore: false,
		} satisfies GetProposalsResponse);
		const view = render(ReconciliationProposals, { bankAccountId: 17 });
		const boxes = await view.findAllByTestId('tx-checkbox');
		for (const b of boxes) await fireEvent.click(b);
		await fireEvent.click(view.getByTestId('reconciliation-accept-btn'));
		expect(mockApi.acceptProposals).toHaveBeenCalledTimes(1);
		expect(mockApi.acceptProposals.mock.calls[0][0]).toBe(17);
		expect(mockApi.acceptProposals.mock.calls[0][1]).toEqual([
			{ type: 'supplier_invoice', bankTransactionId: 1, supplierInvoiceId: 55 },
			{ type: 'invoice', bankTransactionId: 2, invoiceId: 101 },
			{ type: 'rule', bankTransactionId: 3, ruleId: 9, counterpartyAccountId: 3000 },
		]);
	});

	// T-b39 — la `notice` est affichée À LA PLACE d'une proposition ; « Accepter » est grisé ;
	// le texte ne conseille pas de confirmer le lot et ne cite aucun numéro d'issue ; le numéro
	// de facture manquant est remplacé par #id.
	it('T-b39 — la notice remplace la proposition, Accepter grisé, sans « confirmez » ni numéro d’issue', async () => {
		mockApi.getProposals.mockResolvedValue({
			proposals: [noticeRow(5, 'FF-LOT-1'), noticeRow(6, null)],
			hasMore: false,
		} satisfies GetProposalsResponse);
		const view = render(ReconciliationProposals, { bankAccountId: 17 });
		const notices = await view.findAllByTestId('proposal-notice');
		expect(notices).toHaveLength(2);
		expect(notices[0].getAttribute('data-notice-code')).toBe('supplier_invoice_in_payment_batch');
		expect(notices[0].textContent).toContain('FF-LOT-1');
		expect(notices[0].textContent).toContain('n° 12');
		expect(notices[1].textContent).toContain('#77');
		for (const n of notices) {
			const texte = (n.textContent ?? '').toLowerCase();
			expect(texte).not.toContain('confirm');
			expect(texte).not.toMatch(/#\d{3,}/);
			expect(texte).toContain('annulez le lot');
		}
		expect(view.queryAllByTestId('candidate-top-1')).toHaveLength(0);
		expect(view.queryByText('Aucune correspondance')).toBeNull();
		await fireEvent.click(view.getAllByTestId('tx-checkbox')[0]);
		expect(view.getByTestId('reconciliation-accept-btn').hasAttribute('disabled')).toBe(true);
		expect(view.getByTestId('reconciliation-reject-btn').hasAttribute('disabled')).toBe(false);
	});

	// Story 33-1b (#926) — l'avis nomme par le numéro interne ; la candidate montre les deux.
	it('33-1b — l’avis dit le numéro interne, la candidate les deux numéros', async () => {
		mockApi.getProposals.mockResolvedValue({
			proposals: [
				noticeRow(5, 'ABC-LOT', 'FF-2026-005'),
				makeProposalWithCandidates(1, [supplierCandidate(55, 'ABC-17', 'Garage', 'FF-2026-017')]),
			],
			hasMore: false,
		} satisfies GetProposalsResponse);
		const view = render(ReconciliationProposals, { bankAccountId: 17 });
		const [notice] = await view.findAllByTestId('proposal-notice');
		expect(notice.textContent).toContain('FF-2026-005');
		expect(notice.textContent).not.toContain('ABC-LOT');
		const [candidate] = await view.findAllByTestId('candidate-top-1');
		expect(candidate.textContent).toContain('FF-2026-017 (ABC-17)');
	});

	it('T-b39 — le repli de la notice et celui de la priorité égalent le catalogue fr-CH', () => {
		const texte = readFileSync(
			join('../crates/kesh-i18n/locales', 'fr-CH', 'messages.ftl'),
			'utf-8',
		);
		const source = readFileSync(
			join('src/lib/features/reconciliation', 'ReconciliationProposals.svelte'),
			'utf-8',
		);
		for (const cle of [
			'reconciliation-labels-notice-supplier-invoice-in-batch',
			'reconciliation-labels-priority-hint',
			'reconciliation-labels-supplier-invoice-badge',
		]) {
			const valeur = texte
				.split('\n')
				.find((l) => l.startsWith(`${cle} = `))
				?.slice(cle.length + 3);
			expect(valeur, cle).toBeDefined();
			expect(source).toContain(`'${valeur}'`);
		}
	});
});

// v0172-correctifs (#880, AC-C2, C-v0172b-11) — la date de la transaction en forme suisse,
// dans la cellule et dans le nom de sa case (V19, V20 ci-dessus).
describe('ReconciliationProposals — date de la transaction en forme suisse (v0172, #880)', () => {
	it('la cellule de date dit 15.05.2026, jamais 2026-05-15', async () => {
		mockApi.getProposals.mockResolvedValue({
			proposals: [makeProposalWithCandidate(1, 101, 1.0)],
			hasMore: false,
		} satisfies GetProposalsResponse);
		const { findAllByTestId } = render(ReconciliationProposals, { bankAccountId: 17 });
		const [row] = await findAllByTestId('reconciliation-row');
		const cells = Array.from(row.querySelectorAll('td')).map((td) => td.textContent?.trim());
		expect(cells).toContain('15.05.2026');
		expect(row.textContent).not.toContain('2026-05-15');
	});
});

// Story 34-4 (#828, AC-5, AC-6) — la transaction qui semble déjà comptabilisée : l'avertissement
// sous la ligne, puis la confirmation avant Accepter, Écriture manuelle et Éclater.
describe('ReconciliationProposals — transaction déjà comptabilisée (Story 34-4, #828)', () => {
	const TEXTE_UNE =
		'Cette transaction semble déjà comptabilisée : une écriture non rapprochée du compte de la banque lui correspond. Rattachez-la (« Rattacher à une écriture existante ») plutôt que de passer une écriture : elle serait comptée deux fois.';
	const texteDes = (n: number) =>
		`Cette transaction semble déjà comptabilisée : ${n} écritures non rapprochées du compte de la banque lui correspondent. Rattachez-la (« Rattacher à une écriture existante ») plutôt que de passer une écriture : elle serait comptée deux fois.`;
	// Revue P1 (E1-1, B1-5) : le second côté se dit par la transaction de sens contraire, et la
	// phrase du cas combiné vaut au singulier comme au pluriel.
	const TEXTE_AILLEURS =
		'Une transaction de sens contraire, sur un autre de vos comptes bancaires, est déjà rapprochée d’une écriture du même montant : c’est probablement l’autre côté d’un virement entre vos comptes. Ne passez pas d’écriture pour elle : rejetez-la.';
	const TEXTE_COMBINE =
		'Si rien de ce que propose « Rattacher à une écriture existante » ne lui correspond, elle est peut-être l’autre côté d’un virement entre vos comptes : une transaction de sens contraire, sur un autre de vos comptes bancaires, est rapprochée d’une écriture du même montant.';

	/** Une ligne avec candidate, et ses trois nombres (`null` : pas de champ). */
	function signalee(
		id: number,
		compte: [number, number, number] | null,
		counterpartyName = 'ACME GMBH',
	): GetProposalsResponse['proposals'][number] {
		const p = makeProposalWithCandidate(id, 100 + id, 1.0);
		return {
			...p,
			transaction: { ...p.transaction, counterpartyName },
			alreadyBooked: compte
				? { settlementCount: compte[0], entryCount: compte[1], matchedElsewhereCount: compte[2] }
				: null,
		};
	}

	async function monter(proposals: GetProposalsResponse['proposals']) {
		mockApi.getProposals.mockResolvedValue({ proposals, hasMore: false } satisfies GetProposalsResponse);
		const view = render(ReconciliationProposals, { bankAccountId: 17 });
		await view.findAllByTestId('reconciliation-row');
		return view;
	}

	/** Le texte de l'avertissement de la transaction `id`, ou `null`. */
	function avertissement(view: Awaited<ReturnType<typeof monter>>, id: number): string | null {
		const el = view
			.queryAllByTestId('already-booked-warning')
			.find((e) => e.getAttribute('data-tx-id') === String(id));
		return el ? (el.textContent ?? '').trim() : null;
	}

	beforeEach(() => {
		vi.clearAllMocks();
		mockApi.acceptProposals.mockResolvedValue({ accepted: [], failed: [] } satisfies AcceptResponse);
		mockApi.rejectProposals.mockResolvedValue({ rejected: [], failed: [] } satisfies RejectResponse);
	});

	afterEach(() => cleanup());

	// ---------- AC-5 : l'avertissement, quatre cas ----------

	it('AC-5 aucun : alreadyBooked null ou trois zéros ⇒ aucun avertissement, jamais « 0 écriture »', async () => {
		const view = await monter([signalee(1, null), signalee(2, [0, 0, 0])]);
		expect(view.queryAllByTestId('already-booked-warning')).toHaveLength(0);
		expect(view.queryAllByTestId('already-booked-row')).toHaveLength(0);
		expect(document.body.textContent).not.toContain('0 écriture');
	});

	it('AC-5 (a) une écriture ⇒ la clé -one, texte complet, sous la ligne (M8)', async () => {
		const view = await monter([signalee(1, [0, 1, 0])]);
		expect(avertissement(view, 1)).toBe(TEXTE_UNE);
		// Sous la ligne : la rangée d'avertissement suit immédiatement celle de la transaction.
		const [ligne] = view.getAllByTestId('reconciliation-row');
		expect(ligne.nextElementSibling?.getAttribute('data-testid')).toBe('already-booked-row');
		expect(avertissement(view, 1)).not.toMatch(/#\d/);
	});

	it('AC-5 (a) plusieurs ⇒ la clé -other, avec le total règlements + écritures', async () => {
		const view = await monter([signalee(1, [1, 2, 0]), signalee(2, [1, 0, 0])]);
		expect(avertissement(view, 1)).toBe(texteDes(3));
		// Un seul règlement compte comme une seule écriture.
		expect(avertissement(view, 2)).toBe(TEXTE_UNE);
	});

	it('AC-5 (b) le second côté seul ⇒ la phrase du virement, sans la phrase (a) (M7)', async () => {
		const view = await monter([signalee(1, [0, 0, 2])]);
		expect(avertissement(view, 1)).toBe(TEXTE_AILLEURS);
	});

	it('AC-5 (c) les deux ⇒ (a) puis la phrase combinée, sans « rejetez-la »', async () => {
		const view = await monter([signalee(1, [0, 1, 1]), signalee(2, [2, 0, 3])]);
		expect(avertissement(view, 1)).toBe(`${TEXTE_UNE} ${TEXTE_COMBINE}`);
		expect(avertissement(view, 2)).toBe(`${texteDes(2)} ${TEXTE_COMBINE}`);
		expect(avertissement(view, 1)).not.toContain('rejetez-la');
	});

	// ---------- AC-6 : la confirmation ----------

	it('AC-6 Accepter, sélection avec une ligne signalée ⇒ confirmation qui la nomme ; Annuler n’appelle rien', async () => {
		const view = await monter([signalee(1, [0, 1, 0], 'Dubarde SàRL'), signalee(2, null)]);
		const cases = view.getAllByTestId('tx-checkbox');
		await fireEvent.click(cases[0]);
		await fireEvent.click(cases[1]);
		await fireEvent.click(view.getByTestId('reconciliation-accept-btn'));
		await view.findByTestId('already-booked-confirm');
		const items = view.getAllByTestId('already-booked-confirm-item');
		expect(items.map((i) => i.getAttribute('data-tx-id'))).toEqual(['1']);
		expect(items[0].textContent?.trim()).toBe('15.05.2026 · 100.00 CHF · Dubarde SàRL');
		expect(mockApi.acceptProposals).not.toHaveBeenCalled();

		await fireEvent.click(view.getByTestId('already-booked-confirm-cancel'));
		await vi.waitFor(() => expect(view.queryByTestId('already-booked-confirm')).toBeNull());
		expect(mockApi.acceptProposals).not.toHaveBeenCalled();
		expect(mockApi.rejectProposals).not.toHaveBeenCalled();
		expect(mockApi.manualMatchTransaction).not.toHaveBeenCalled();
	});

	it('AC-6 Accepter, Continuer quand même ⇒ le lot entier part, la ligne non signalée comprise', async () => {
		const view = await monter([signalee(1, [0, 1, 0]), signalee(2, null)]);
		const cases = view.getAllByTestId('tx-checkbox');
		await fireEvent.click(cases[0]);
		await fireEvent.click(cases[1]);
		await fireEvent.click(view.getByTestId('reconciliation-accept-btn'));
		await fireEvent.click(await view.findByTestId('already-booked-confirm-continue'));
		await vi.waitFor(() => expect(mockApi.acceptProposals).toHaveBeenCalledTimes(1));
		const [, items] = mockApi.acceptProposals.mock.calls[0];
		expect(items.map((i) => i.bankTransactionId).sort()).toEqual([1, 2]);
	});

	it('AC-6 Accepter sans ligne signalée ⇒ aucun dialogue, le lot part aussitôt', async () => {
		const view = await monter([signalee(1, [0, 0, 0]), signalee(2, null)]);
		await fireEvent.click(view.getAllByTestId('tx-checkbox')[0]);
		await fireEvent.click(view.getByTestId('reconciliation-accept-btn'));
		await vi.waitFor(() => expect(mockApi.acceptProposals).toHaveBeenCalledTimes(1));
		expect(view.queryByTestId('already-booked-confirm')).toBeNull();
	});

	// Revue P1 (E1-3, A1-2) : chaque doublure porte le nom de sa modale (`data-modal`), si bien
	// que *Continuer quand même* doit rouvrir **le** geste demandé (mutation M9 : `manual` et
	// `split` intervertis dans `runGesture`).
	it.each([
		['manual-match-button', 'manual'],
		['split-button', 'split'],
	])(
		'AC-6 %s sur une ligne signalée ⇒ confirmation ; Annuler n’ouvre rien ; Continuer ouvre la modale « %s » (M5, M9)',
		async (bouton, modale) => {
			const view = await monter([signalee(1, [0, 0, 1])]);
			await fireEvent.click(view.getByTestId(bouton));
			await view.findByTestId('already-booked-confirm');
			expect(view.queryByTestId('stub-modal-success')).toBeNull();
			await fireEvent.click(view.getByTestId('already-booked-confirm-cancel'));
			await vi.waitFor(() => expect(view.queryByTestId('already-booked-confirm')).toBeNull());
			expect(view.queryByTestId('stub-modal-success')).toBeNull();
			expect(mockApi.manualMatchTransaction).not.toHaveBeenCalled();

			await fireEvent.click(view.getByTestId(bouton));
			await fireEvent.click(await view.findByTestId('already-booked-confirm-continue'));
			const ouverte = await view.findByTestId('stub-modal-success');
			expect(ouverte.getAttribute('data-modal')).toBe(modale);
			expect(view.queryByTestId('already-booked-confirm')).toBeNull();
		},
	);

	it.each([
		['manual-match-button', 'manual'],
		['split-button', 'split'],
	])('AC-6 %s sur une ligne non signalée ⇒ la modale « %s » s’ouvre aussitôt', async (bouton, modale) => {
		const view = await monter([signalee(1, null)]);
		await fireEvent.click(view.getByTestId(bouton));
		const ouverte = await view.findByTestId('stub-modal-success');
		expect(ouverte.getAttribute('data-modal')).toBe(modale);
		expect(view.queryByTestId('already-booked-confirm')).toBeNull();
	});

	// Revue P1 (E1-6) : fermer la confirmation par Échap vaut *Annuler* — rien n'est appelé, et
	// le geste la rouvre ensuite (le geste en attente a bien été oublié).
	it('AC-6 Échap ferme la confirmation comme Annuler, et le geste la rouvre ensuite', async () => {
		const view = await monter([signalee(1, [0, 1, 0])]);
		await fireEvent.click(view.getByTestId('manual-match-button'));
		const dialogue = await view.findByTestId('already-booked-confirm');
		await fireEvent.keyDown(dialogue, { key: 'Escape' });
		await vi.waitFor(() => expect(view.queryByTestId('already-booked-confirm')).toBeNull());
		expect(view.queryByTestId('stub-modal-success')).toBeNull();
		expect(mockApi.manualMatchTransaction).not.toHaveBeenCalled();
		await fireEvent.click(view.getByTestId('manual-match-button'));
		await view.findByTestId('already-booked-confirm');
	});

	it('AC-6 Rejeter une ligne signalée ⇒ aucune confirmation (M6)', async () => {
		const view = await monter([signalee(1, [0, 1, 1])]);
		await fireEvent.click(view.getAllByTestId('tx-checkbox')[0]);
		await fireEvent.click(view.getByTestId('reconciliation-reject-btn'));
		await vi.waitFor(() => expect(mockApi.rejectProposals).toHaveBeenCalledWith(17, [1]));
		expect(view.queryByTestId('already-booked-confirm')).toBeNull();
	});

	it('AC-6 Rattacher à une écriture existante sur une ligne signalée ⇒ aucune confirmation', async () => {
		const view = await monter([signalee(1, [0, 1, 0])]);
		await fireEvent.click(view.getByTestId('link-settlement-button'));
		const ouverte = await view.findByTestId('stub-modal-success');
		expect(ouverte.getAttribute('data-modal')).toBe('link');
		expect(view.queryByTestId('already-booked-confirm')).toBeNull();
	});

	// Revue P2 (B2-5) : la liste est relue pendant la confirmation (ici, « Afficher aussi les plus
	// anciennes ») et la ligne en a disparu — traitée dans un autre onglet, par exemple. *Continuer
	// quand même* relit la ligne dans la liste courante : absente, rien ne s'ouvre (jamais la
	// modale d'une ligne périmée).
	it('AC-6 Continuer ne rouvre pas le geste d’une ligne sortie de la liste pendant la confirmation', async () => {
		mockApi.getProposals
			.mockResolvedValueOnce({
				proposals: [signalee(1, [0, 1, 0])],
				hasMore: true,
			} satisfies GetProposalsResponse)
			.mockResolvedValue({ proposals: [signalee(2, null)], hasMore: false } satisfies GetProposalsResponse);
		const view = render(ReconciliationProposals, { bankAccountId: 17 });
		await view.findAllByTestId('reconciliation-row');
		await fireEvent.click(view.getByTestId('manual-match-button'));
		await view.findByTestId('already-booked-confirm');
		await fireEvent.click(view.getByTestId('reconciliation-show-older-btn'));
		await vi.waitFor(() =>
			expect(view.getAllByTestId('reconciliation-row').map((r) => r.getAttribute('data-tx-id'))).toEqual([
				'2',
			]),
		);
		await fireEvent.click(view.getByTestId('already-booked-confirm-continue'));
		await vi.waitFor(() => expect(view.queryByTestId('already-booked-confirm')).toBeNull());
		expect(view.queryByTestId('stub-modal-success')).toBeNull();
	});
});
