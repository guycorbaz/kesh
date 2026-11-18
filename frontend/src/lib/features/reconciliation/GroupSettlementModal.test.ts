// Story 43-3 (#823) — tests Vitest de GroupSettlementModal.svelte, sur le contrat de la 43-2 :
// ouverture sans facture imputée, recherche (`exclude`, `hasMore`, liste vide, `direction:
// null`, chargement), ajout et retrait, total imputé et écart (sens, libellé, arrondi),
// sélecteur du compte de l'écart (apparition, préremplissage, choix gardé, obligation),
// corps envoyé, contrôle de cohérence, refus (D5) et gardes de concurrence (D4).
//
// Le vrai `AccountAutocomplete` est employé : on choisit un compte par `focus` puis
// `mouseDown` sur l'option, comme `ManualMatchModal.test.ts`.

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, fireEvent, waitFor } from '@testing-library/svelte';
import type { AccountResponse } from '$lib/features/accounts/accounts.types';
import type {
	AcceptResponse,
	GroupCandidate,
	GroupCandidatesResponse,
	ReconciliationProposal,
} from './reconciliation.types';

vi.mock('./reconciliation.api', () => ({
	getGroupCandidates: vi.fn(),
	acceptProposals: vi.fn(),
}));

vi.mock('$lib/features/invoices/invoices.api', () => ({
	getInvoiceSettings: vi.fn(),
}));

vi.mock('svelte-sonner', () => ({
	toast: { success: vi.fn(), error: vi.fn(), warning: vi.fn() },
}));

vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string, args?: Record<string, string | number>) =>
		args ? fallback.replace(/\{\s*\$(\w+)\s*\}/g, (_, k) => String(args[k] ?? '')) : fallback,
}));

import * as api from './reconciliation.api';
import * as invoicesApi from '$lib/features/invoices/invoices.api';
import { toast } from 'svelte-sonner';
import GroupSettlementModal from './GroupSettlementModal.svelte';

const mockApi = vi.mocked(api);
const mockSettings = vi.mocked(invoicesApi.getInvoiceSettings);
const mockToast = vi.mocked(toast);

const TX = 41;
const BANK_LEDGER = 1020;

function account(id: number, number: string, name: string, over: Partial<AccountResponse> = {}): AccountResponse {
	return {
		id,
		companyId: 1,
		number,
		name,
		accountType: 'Expense',
		parentId: null,
		active: true,
		role: null,
		postable: true,
		letterable: false,
		version: 1,
		createdAt: '2026-01-01T00:00:00',
		updatedAt: '2026-01-01T00:00:00',
		...over,
	};
}

const ACCOUNTS: AccountResponse[] = [
	account(1020, '1020', 'Banque', { accountType: 'Asset' }),
	account(1100, '1100', 'Débiteurs', { accountType: 'Asset', role: 'Receivable' }),
	account(6940, '6940', 'Frais bancaires'),
	account(6960, '6960', 'Différences d’arrondi'),
	account(6970, '6970', 'Pertes sur débiteurs'),
	account(6941, '6941', 'Frais archivé', { active: false }),
	// Epic 37 — un compte tenu en devise : jamais offert ni prérempli (le rapprochement passe en CHF).
	account(6945, '6945', 'Frais en euros', { currency: 'EUR' }),
];

function proposal(amount = '799.60', id = TX, counterpartyName: string | null = 'Exemple SA'): ReconciliationProposal {
	return {
		bankTransactionId: id,
		transaction: {
			bookingDate: '2026-11-10',
			valueDate: '2026-11-10',
			amount,
			currency: 'CHF',
			counterpartyName,
		},
		candidates: [],
		notice: null,
		alreadyBooked: null,
	};
}

function inv(id: number, raw: string, contact = 'Exemple SA'): GroupCandidate {
	return {
		kind: 'invoice',
		id,
		number: `F-${id}`,
		contactName: contact,
		date: '2026-10-01',
		dueDate: '2026-10-31',
		totalTtc: raw.slice(0, raw.indexOf('.') + 3),
		amountDue: raw.slice(0, raw.indexOf('.') + 3),
		amountDueRaw: raw,
	};
}

function response(
	candidates: GroupCandidate[],
	over: Partial<GroupCandidatesResponse> = {},
): GroupCandidatesResponse {
	return {
		direction: 'client',
		transactionAmount: '799.60',
		maxDifference: '5.00',
		candidates,
		hasMore: false,
		...over,
	};
}

function settings(rounding: number | null = 6960, bankFees: number | null = 6940) {
	return {
		defaultRoundingAccountId: rounding,
		defaultBankFeesAccountId: bankFees,
	} as unknown as Awaited<ReturnType<typeof invoicesApi.getInvoiceSettings>>;
}

/**
 * La recherche simulée : rend `pool` moins les `exclude`, filtrés par `q` sur le numéro si `q`
 * commence par « F- » (la relecture des montants), tous sinon.
 */
function serve(pool: GroupCandidate[], over: Partial<GroupCandidatesResponse> = {}) {
	mockApi.getGroupCandidates.mockImplementation(async (_tx, query = {}) => {
		const exclude = new Set(query.exclude ?? []);
		const q = query.q ?? '';
		const list = pool.filter(
			(c) => !exclude.has(c.id) && (!q.startsWith('F-') || (c.kind === 'invoice' && c.number === q)),
		);
		return response(list, over);
	});
}

function monter(p: ReconciliationProposal = proposal(), accounts = ACCOUNTS) {
	const cb = {
		onOpenChange: vi.fn<(v: boolean) => void>(),
		onSuccess: vi.fn<() => void>(),
		onStale: vi.fn<() => void>(),
	};
	const r = render(GroupSettlementModal, {
		open: true,
		bankAccountId: 17,
		proposal: p,
		accounts,
		bankLedgerAccountId: BANK_LEDGER,
		...cb,
	});
	return { r, cb };
}

type R = ReturnType<typeof monter>['r'];

async function ajouter(r: R, id: number): Promise<void> {
	const boutons = await r.findAllByTestId('group-add');
	const b = boutons.find((x) => x.getAttribute('data-invoice-id') === String(id));
	expect(b, `la facture ${id} doit être proposée`).toBeTruthy();
	await fireEvent.click(b as HTMLElement);
	await waitFor(() =>
		expect(
			r.getAllByTestId('group-imputed-row').some((x) => x.getAttribute('data-invoice-id') === String(id)),
		).toBe(true),
	);
}

function champCompte(): HTMLInputElement | null {
	return document.querySelector('input[aria-autocomplete="list"]');
}

async function choisirCompte(r: R, numero: string): Promise<void> {
	const champ = champCompte() as HTMLInputElement;
	await fireEvent.focus(champ);
	await fireEvent.input(champ, { target: { value: numero } });
	const option = (await r.findAllByRole('option')).find((o) => o.textContent?.includes(numero));
	expect(option, `le compte ${numero} doit être proposé`).toBeTruthy();
	await fireEvent.mouseDown(option as HTMLElement);
}

function texte(r: R, id: string): string {
	return r.getByTestId(id).textContent?.replace(/\s+/g, ' ').trim() ?? '';
}

function submit(r: R): HTMLButtonElement {
	return r.getByTestId('group-submit') as HTMLButtonElement;
}

beforeEach(() => {
	vi.clearAllMocks();
	mockSettings.mockResolvedValue(settings());
});

afterEach(() => {
	vi.useRealTimers();
});

describe('GroupSettlementModal — ouverture et recherche (D2, AC2)', () => {
	it('s’ouvre sans facture imputée ; la première recherche porte le nom de la contrepartie', async () => {
		serve([inv(12, '500.0000'), inv(15, '300.0000', 'Autre SàRL')]);
		const { r } = monter();
		await r.findAllByTestId('group-result-row');
		expect(mockApi.getGroupCandidates).toHaveBeenCalledWith(TX, { q: 'Exemple SA', exclude: [] });
		expect(r.getByTestId('group-imputed-empty')).toBeTruthy();
		expect(r.queryAllByTestId('group-imputed-row')).toHaveLength(0);
		expect(submit(r).disabled).toBe(true);
		expect(texte(r, 'group-block-hint')).toBe('Ajoutez au moins deux factures.');
	});

	it('une facture ajoutée part dans exclude et quitte les résultats ; retirée, elle y revient', async () => {
		serve([inv(12, '500.0000'), inv(15, '300.0000', 'Autre SàRL')]);
		const { r } = monter();
		await ajouter(r, 12);
		await waitFor(() =>
			expect(mockApi.getGroupCandidates).toHaveBeenLastCalledWith(TX, { q: 'Exemple SA', exclude: [12] }),
		);
		await waitFor(() =>
			expect(r.getAllByTestId('group-result-row').map((x) => x.getAttribute('data-invoice-id'))).toEqual([
				'15',
			]),
		);
		await fireEvent.click(r.getByTestId('group-remove'));
		await waitFor(() =>
			expect(mockApi.getGroupCandidates).toHaveBeenLastCalledWith(TX, { q: 'Exemple SA', exclude: [] }),
		);
		await waitFor(() => expect(r.getAllByTestId('group-result-row')).toHaveLength(2));
		expect(r.getByTestId('group-imputed-empty')).toBeTruthy();
	});

	it('une frappe relance la recherche après le délai, avec le terme tapé', async () => {
		serve([inv(12, '500.0000')]);
		const { r } = monter();
		await r.findAllByTestId('group-result-row');
		await fireEvent.input(r.getByTestId('group-search-input'), { target: { value: 'F-12' } });
		await waitFor(() =>
			expect(mockApi.getGroupCandidates).toHaveBeenLastCalledWith(TX, { q: 'F-12', exclude: [] }),
		);
	});

	it('aucun résultat → « aucune facture ne correspond »', async () => {
		serve([]);
		const { r } = monter();
		await r.findByTestId('group-results-empty');
		expect(texte(r, 'group-results-empty')).toBe('Aucune facture ne correspond à cette recherche.');
		expect(r.queryByTestId('group-no-direction')).toBeNull();
	});

	it('direction: null → « aucune facture à proposer pour cette transaction », distinct', async () => {
		serve([], { direction: null });
		const { r } = monter();
		await r.findByTestId('group-no-direction');
		expect(texte(r, 'group-no-direction')).toBe('Aucune facture à proposer pour cette transaction.');
		expect(r.queryByTestId('group-results-empty')).toBeNull();
	});

	it('hasMore → phrase qui invite à préciser la recherche', async () => {
		serve([inv(12, '500.0000')], { hasMore: true });
		const { r } = monter();
		await r.findByTestId('group-has-more');
		expect(texte(r, 'group-has-more')).toBe('D’autres factures correspondent : précisez la recherche.');
	});

	it('chargement → état visible', async () => {
		mockApi.getGroupCandidates.mockReturnValue(new Promise(() => {}));
		const { r } = monter();
		await r.findByTestId('group-loading');
		expect(texte(r, 'group-loading')).toBe('Recherche…');
	});
});

describe('GroupSettlementModal — écart, arrondi et compte (D2 bis, D3)', () => {
	async function deuxFactures(amount: string, raws: [string, string], over: Partial<GroupCandidatesResponse> = {}) {
		serve([inv(12, raws[0]), inv(15, raws[1], 'Autre SàRL')], over);
		const m = monter(proposal(amount));
		await ajouter(m.r, 12);
		await ajouter(m.r, 15);
		return m;
	}

	it('moins-payé de 0.40 entre deux clients : compte de frais bancaires prérempli, corps envoyé', async () => {
		const { r, cb } = await deuxFactures('799.60', ['500.0000', '300.0000']);
		expect(texte(r, 'group-transaction-amount')).toBe('799.60');
		expect(texte(r, 'group-total-imputed')).toBe('800.00');
		expect(texte(r, 'group-gap')).toBe('Moins-payé de 0.40');
		await waitFor(() => expect(champCompte()?.value).toBe('6940 — Frais bancaires'));
		await waitFor(() => expect(submit(r).disabled).toBe(false));
		mockApi.acceptProposals.mockResolvedValue({
			accepted: [
				{
					bankTransactionId: TX,
					invoiceId: 0,
					supplierInvoiceId: null,
					journalEntryId: 905,
					score: { total: 1, amountScore: 0, referenceScore: 0, contactScore: 0 },
					difference: '-0.4000',
					roundingDifference: '0.0000',
					gap: '-0.40',
					differenceAccountId: 6940,
					settledInvoices: [
						{ invoiceId: 12, amount: '500.0000' },
						{ invoiceId: 15, amount: '300.0000' },
					],
				},
			],
			failed: [],
		} satisfies AcceptResponse);
		await fireEvent.click(submit(r));
		await waitFor(() => expect(cb.onSuccess).toHaveBeenCalledTimes(1));
		expect(mockApi.acceptProposals).toHaveBeenCalledWith(17, [
			{ type: 'invoice_group', bankTransactionId: TX, invoiceIds: [12, 15], differenceAccountId: 6940 },
		]);
		expect(cb.onOpenChange).toHaveBeenCalledWith(false);
		expect(mockToast.success).toHaveBeenCalledTimes(1);
		// Contrôle de cohérence : gap = e, roundingDifference = r → aucun avertissement.
		expect(mockToast.warning).not.toHaveBeenCalled();
	});

	it('trop-perçu de 0.40 : compte d’arrondi prérempli', async () => {
		const { r } = await deuxFactures('800.40', ['500.0000', '300.0000']);
		expect(texte(r, 'group-gap')).toBe('Trop-perçu de 0.40');
		await waitFor(() => expect(champCompte()?.value).toBe('6960 — Différences d’arrondi'));
	});

	it('écart nul : « Aucun écart », aucun sélecteur, corps sans differenceAccountId', async () => {
		const { r } = await deuxFactures('800.00', ['500.0000', '300.0000']);
		expect(texte(r, 'group-gap')).toBe('Aucun écart');
		expect(r.queryByTestId('group-account-section')).toBeNull();
		expect(r.queryByTestId('group-rounding')).toBeNull();
		await waitFor(() => expect(submit(r).disabled).toBe(false));
		mockApi.acceptProposals.mockResolvedValue({ accepted: [], failed: [] });
		await fireEvent.click(submit(r));
		await waitFor(() => expect(mockApi.acceptProposals).toHaveBeenCalledTimes(1));
		expect(mockApi.acceptProposals.mock.calls[0][1]).toEqual([
			{ type: 'invoice_group', bankTransactionId: TX, invoiceIds: [12, 15] },
		]);
	});

	// Le tableau de decision-difference.md, lu à l'écran (texte de l'écart, de l'arrondi, sélecteur).
	it.each([
		['2 × 10.0050 payés 20.01', '20.01', ['10.0050', '10.0050'], 'Aucun écart', null, false],
		['2 × 10.0050 payés 20.02', '20.02', ['10.0050', '10.0050'], 'Trop-perçu de 0.01', null, true],
		['500.00 + 300.00 payés 799.99', '799.99', ['500.0000', '300.0000'], 'Moins-payé de 0.01', null, true],
		['3 × 10.0050 payés 30.06', '30.06', ['10.0050', '10.0050', '10.0050'], 'Trop-perçu de 0.05', '-0.0050', true],
		['10.0049 + 10.0000 payés 20.00', '20.00', ['10.0049', '10.0000'], 'Aucun écart', '-0.0049', false],
		['10.0050 + 10.0000 payés 20.00', '20.00', ['10.0050', '10.0000'], 'Moins-payé de 0.01', '+0.0050', true],
	] as const)('%s → écart, arrondi, compte demandé', async (_cas, amount, raws, gap, rounding, compte) => {
		const pool = raws.map((raw, i) => inv(20 + i, raw, `Tiers ${i}`));
		serve(pool);
		const { r } = monter(proposal(amount));
		for (const c of pool) await ajouter(r, c.id);
		expect(texte(r, 'group-gap')).toBe(gap);
		if (rounding === null) {
			expect(r.queryByTestId('group-rounding')).toBeNull();
		} else {
			expect(texte(r, 'group-rounding')).toBe(`Arrondi de ${rounding} passé au compte d’arrondi`);
		}
		expect(r.queryByTestId('group-account-section') !== null).toBe(compte);
	});

	it('chaque reste dû imputé s’affiche au brut s’il porte une fraction de centime', async () => {
		const { r } = await deuxFactures('20.02', ['10.0050', '10.0050']);
		expect(r.getAllByTestId('group-imputed-due').map((x) => x.textContent?.trim())).toEqual([
			'10.0050',
			'10.0050',
		]);
		// La somme tombe au centime : elle s'affiche au centime.
		expect(texte(r, 'group-total-imputed')).toBe('20.01');
	});

	it('un total imputé à fraction de centime s’affiche au brut', async () => {
		serve([inv(20, '10.0050'), inv(21, '10.0050', 'B'), inv(22, '10.0050', 'C')]);
		const { r } = monter(proposal('30.06'));
		for (const id of [20, 21, 22]) await ajouter(r, id);
		expect(texte(r, 'group-total-imputed')).toBe('30.0150');
	});

	it('arrondi sans compte d’arrondi désigné : dit avant l’envoi, Enregistrer inactif, même si e = 0', async () => {
		mockSettings.mockResolvedValue(settings(null, 6940));
		const { r } = await deuxFactures('20.00', ['10.0049', '10.0000']);
		await r.findByTestId('group-rounding-not-configured');
		expect(texte(r, 'group-rounding-not-configured')).toBe(
			'Aucun compte de différences d’arrondi n’est désigné : demandez à l’administrateur de le régler.',
		);
		expect(submit(r).disabled).toBe(true);
	});

	it('arrondi automatique avec un plafond réglé à 0.00 : Enregistrer actif', async () => {
		const { r } = await deuxFactures('20.00', ['10.0049', '10.0000'], { maxDifference: '0.00' });
		await waitFor(() => expect(submit(r).disabled).toBe(false));
	});

	it('écart au-delà du plafond lu dans la réponse (1.00) : « Écart trop grand », pas de sélecteur, inactif', async () => {
		const { r } = await deuxFactures('798.99', ['500.0000', '300.0000'], { maxDifference: '1.00' });
		expect(texte(r, 'group-gap')).toBe('Écart trop grand (au plus 1.00)');
		expect(r.queryByTestId('group-account-section')).toBeNull();
		expect(submit(r).disabled).toBe(true);
	});

	it('compte désigné archivé : champ vide, compte demandé, Enregistrer inactif', async () => {
		mockSettings.mockResolvedValue(settings(6960, 6941));
		const { r } = await deuxFactures('799.60', ['500.0000', '300.0000']);
		await r.findByTestId('group-account-section');
		expect(champCompte()?.value).toBe('');
		expect(texte(r, 'group-block-hint')).toBe('Choisissez le compte de l’écart.');
		expect(submit(r).disabled).toBe(true);
		await choisirCompte(r, '6970');
		await waitFor(() => expect(submit(r).disabled).toBe(false));
	});

	it('Epic 37 — un compte en devise n’est ni prérempli ni offert dans le sélecteur', async () => {
		mockSettings.mockResolvedValue(settings(6960, 6945));
		const { r } = await deuxFactures('799.60', ['500.0000', '300.0000']);
		await r.findByTestId('group-account-section');
		expect(champCompte()?.value).toBe('');
		const champ = champCompte() as HTMLInputElement;
		await fireEvent.focus(champ);
		await fireEvent.input(champ, { target: { value: '69' } });
		const options = (await r.findAllByRole('option')).map((o) => o.textContent ?? '');
		expect(options.some((t) => t.includes('6940'))).toBe(true);
		expect(options.some((t) => t.includes('6945'))).toBe(false);
	});

	it('choix gardé : le compte choisi reste quand le signe change et quand le sélecteur disparaît puis revient', async () => {
		serve([inv(12, '500.0000'), inv(15, '300.0000', 'Autre SàRL'), inv(16, '0.8000'), inv(17, '0.4000')]);
		const { r } = monter(proposal('800.40'));
		await ajouter(r, 12);
		await ajouter(r, 15);
		await waitFor(() => expect(champCompte()?.value).toBe('6960 — Différences d’arrondi'));
		await choisirCompte(r, '6970');
		await waitFor(() => expect(champCompte()?.value).toBe('6970 — Pertes sur débiteurs'));
		// Le signe change : 800.80 imputés pour 800.40 → moins-payé ; le préremplissage dirait 6940.
		await ajouter(r, 16);
		expect(texte(r, 'group-gap')).toBe('Moins-payé de 0.40');
		await waitFor(() => expect(champCompte()?.value).toBe('6970 — Pertes sur débiteurs'));
		// Retirer 0.80, ajouter 0.40 : écart nul, le sélecteur disparaît.
		const retirer16 = r.getAllByTestId('group-remove').find((x) => x.getAttribute('data-invoice-id') === '16');
		await fireEvent.click(retirer16 as HTMLElement);
		await ajouter(r, 17);
		expect(texte(r, 'group-gap')).toBe('Aucun écart');
		expect(r.queryByTestId('group-account-section')).toBeNull();
		// Retirer 0.40 : le sélecteur revient, avec le choix de l'utilisateur.
		const retirer17 = r.getAllByTestId('group-remove').find((x) => x.getAttribute('data-invoice-id') === '17');
		await fireEvent.click(retirer17 as HTMLElement);
		await r.findByTestId('group-account-section');
		await waitFor(() => expect(champCompte()?.value).toBe('6970 — Pertes sur débiteurs'));
	});

	it('contrôle de cohérence : gap ou arrondi différents de l’écran → un toast dit ce qui a été passé', async () => {
		const { r } = await deuxFactures('799.60', ['500.0000', '300.0000']);
		await waitFor(() => expect(submit(r).disabled).toBe(false));
		mockApi.acceptProposals.mockResolvedValue({
			accepted: [
				{
					bankTransactionId: TX,
					invoiceId: 0,
					supplierInvoiceId: null,
					journalEntryId: 905,
					score: { total: 1, amountScore: 0, referenceScore: 0, contactScore: 0 },
					difference: '-0.5000',
					roundingDifference: '0.0000',
					gap: '-0.50',
					differenceAccountId: 6940,
					settledInvoices: [],
				},
			],
			failed: [],
		});
		await fireEvent.click(submit(r));
		await waitFor(() => expect(mockToast.warning).toHaveBeenCalledTimes(1));
		const message = mockToast.warning.mock.calls[0][0] as string;
		expect(message).toContain('-0.50');
		expect(message).toContain('6940 Frais bancaires');
	});
});

describe('GroupSettlementModal — refus (D5, AC4)', () => {
	async function troisFactures() {
		serve([inv(12, '500.0000'), inv(15, '300.0000', 'Autre SàRL'), inv(19, '100.0000')]);
		const m = monter(proposal('900.00'));
		await ajouter(m.r, 12);
		await ajouter(m.r, 15);
		await ajouter(m.r, 19);
		await waitFor(() => expect(submit(m.r).disabled).toBe(false));
		return m;
	}

	it('une facture refusée parmi trois est retirée et nommée, la recherche relue', async () => {
		const { r } = await troisFactures();
		mockApi.acceptProposals.mockResolvedValue({
			accepted: [],
			failed: [
				{
					bankTransactionId: TX,
					errorCode: 'RECONCILIATION_INVOICE_NOT_ELIGIBLE',
					details: { reason: 'invoice_already_paid', invoiceId: 15, invoiceNumber: 'F-15' },
				},
			],
		});
		const avant = mockApi.getGroupCandidates.mock.calls.length;
		await fireEvent.click(submit(r));
		await r.findByTestId('group-refusal');
		expect(texte(r, 'group-refusal')).toBe(
			"Facture F-15 : Cette facture n'est pas éligible à la réconciliation.",
		);
		expect(r.getAllByTestId('group-imputed-row').map((x) => x.getAttribute('data-invoice-id'))).toEqual([
			'12',
			'19',
		]);
		await waitFor(() => expect(mockApi.getGroupCandidates.mock.calls.length).toBeGreaterThan(avant));
	});

	it('sans invoiceNumber, le numéro vient de la liste imputée', async () => {
		const { r } = await troisFactures();
		mockApi.acceptProposals.mockResolvedValue({
			accepted: [],
			failed: [{ bankTransactionId: TX, errorCode: 'INVOICE_NOT_FOUND', details: { invoiceId: 19 } }],
		});
		await fireEvent.click(submit(r));
		await r.findByTestId('group-refusal');
		expect(texte(r, 'group-refusal')).toBe('Facture F-19 : Facture introuvable.');
	});

	it.each([
		['RECONCILIATION_ALREADY_RECONCILED', 'Cette transaction est déjà réconciliée.'],
		['RECONCILIATION_TRANSACTION_NOT_CREDIT', 'Cette transaction n’est pas une entrée d’argent'],
		['RECONCILIATION_CURRENCY_MISMATCH', 'Seules les transactions en CHF'],
		['BANK_TRANSACTION_NOT_FOUND', 'Transaction introuvable'],
	])('refus de transaction %s → toast, puis fermeture et relecture', async (code, phrase) => {
		const { r, cb } = await troisFactures();
		mockApi.acceptProposals.mockResolvedValue({
			accepted: [],
			failed: [{ bankTransactionId: TX, errorCode: code, details: null }],
		});
		await fireEvent.click(submit(r));
		await waitFor(() => expect(cb.onStale).toHaveBeenCalledTimes(1));
		expect(mockToast.error).toHaveBeenCalledTimes(1);
		expect(mockToast.error.mock.calls[0][0]).toContain(phrase);
		expect(cb.onOpenChange).toHaveBeenCalledWith(false);
		expect(cb.onSuccess).not.toHaveBeenCalled();
	});

	it.each([
		[
			'VALIDATION_ERROR',
			{ reason: 'counterparty_equals_bank_ledger' },
			'Le compte choisi pour l’écart est le compte de la banque de cette transaction : choisissez-en un autre.',
		],
		['ACCOUNT_NOT_FOUND', { missingAccountIds: [6940] }, 'Le compte choisi pour l’écart est introuvable ou archivé.'],
		[
			'ACCOUNT_IN_FOREIGN_CURRENCY',
			{ accountId: 6940, currency: 'EUR' },
			'Le compte choisi pour l’écart est tenu en EUR : choisissez un compte en CHF.',
		],
		[
			'ACCOUNT_NOT_POSTABLE',
			{ rejected: [{ accountId: 6940, accountNumber: '6940' }] },
			'Le compte choisi pour l’écart n’est pas imputable (6940) : choisissez un compte imputable.',
		],
		[
			'SETTLEMENT_COUNTERPARTY_IS_CLAIM_ACCOUNT',
			{ accountId: 6940, accountNumber: '6940', claim: 'receivable', role: 'counterparty' },
			'Le compte choisi pour l’écart est le compte de créance (ou de dette) d’une facture du groupe : choisissez-en un autre.',
		],
	])('refus du compte de l’écart %s → sous le sélecteur, qui reste ouvert', async (code, details, phrase) => {
		serve([inv(12, '500.0000'), inv(15, '300.0000', 'Autre SàRL')]);
		const { r, cb } = monter(proposal('799.60'));
		await ajouter(r, 12);
		await ajouter(r, 15);
		await waitFor(() => expect(submit(r).disabled).toBe(false));
		mockApi.acceptProposals.mockResolvedValue({
			accepted: [],
			failed: [{ bankTransactionId: TX, errorCode: code, details }],
		});
		await fireEvent.click(submit(r));
		await r.findByTestId('group-account-error');
		expect(texte(r, 'group-account-error')).toBe(phrase);
		expect(r.queryByTestId('group-refusals')).toBeNull();
		expect(r.getByTestId('group-account-section')).toBeTruthy();
		expect(cb.onOpenChange).not.toHaveBeenCalled();
	});

	it('cas de configuration (bankAccountId présent) → avec les autres refus, pas sous le sélecteur', async () => {
		serve([inv(12, '500.0000'), inv(15, '300.0000', 'Autre SàRL')]);
		const { r } = monter(proposal('799.60'));
		await ajouter(r, 12);
		await ajouter(r, 15);
		await waitFor(() => expect(submit(r).disabled).toBe(false));
		mockApi.acceptProposals.mockResolvedValue({
			accepted: [],
			failed: [
				{
					bankTransactionId: TX,
					errorCode: 'SETTLEMENT_COUNTERPARTY_IS_CLAIM_ACCOUNT',
					details: {
						accountId: 1100,
						accountNumber: '1100',
						claim: 'receivable',
						role: 'counterparty',
						bankAccountId: 3,
						invoiceId: 12,
						invoiceNumber: 'F-12',
					},
				},
			],
		});
		await fireEvent.click(submit(r));
		await r.findByTestId('group-refusal');
		expect(texte(r, 'group-refusal')).toBe(
			'Facture F-12 : Le compte bancaire est lié au compte débiteurs de la facture : reliez-le à son propre compte de banque.',
		);
		expect(r.queryByTestId('group-account-error')).toBeNull();
	});

	it('difference_account_required : les montants sont relus, l’écart recalculé, la phrase sous le sélecteur', async () => {
		const pool = [inv(12, '500.0000'), inv(15, '300.0000', 'Autre SàRL')];
		serve(pool);
		const { r } = monter(proposal('800.00'));
		await ajouter(r, 12);
		await ajouter(r, 15);
		expect(texte(r, 'group-gap')).toBe('Aucun écart');
		await waitFor(() => expect(submit(r).disabled).toBe(false));
		// Entre la recherche et l'envoi, le reste de F-12 a changé (500.40).
		pool[0] = inv(12, '500.4000');
		mockApi.acceptProposals.mockResolvedValue({
			accepted: [],
			failed: [
				{
					bankTransactionId: TX,
					errorCode: 'RECONCILIATION_GROUP_INVALID',
					details: { reason: 'difference_account_required', gap: '-0.40' },
				},
			],
		});
		await fireEvent.click(submit(r));
		await r.findByTestId('group-account-error');
		expect(texte(r, 'group-gap')).toBe('Moins-payé de 0.40');
		expect(texte(r, 'group-account-error')).toBe('Choisissez le compte de l’écart.');
		expect(mockApi.getGroupCandidates).toHaveBeenCalledWith(TX, { q: 'F-12', limit: 100 });
	});

	it('PERIOD_LOCKED laisse la sélection intacte, avec son libellé', async () => {
		const { r } = await troisFactures();
		mockApi.acceptProposals.mockResolvedValue({
			accepted: [],
			failed: [{ bankTransactionId: TX, errorCode: 'PERIOD_LOCKED', details: null }],
		});
		await fireEvent.click(submit(r));
		await r.findByTestId('group-refusal');
		expect(texte(r, 'group-refusal')).toContain('période verrouillée');
		expect(r.getAllByTestId('group-imputed-row')).toHaveLength(3);
	});

	it.each([
		['NOT_FOUND', 404, 'Transaction introuvable'],
		['RECONCILIATION_TRANSACTION_NOT_PENDING', 404, 'n’est plus en attente'],
	])('recherche refusée %s → toast, fermeture, relecture', async (code, status, phrase) => {
		mockApi.getGroupCandidates.mockRejectedValue({ code, message: 'serveur', status });
		const { cb } = monter();
		await waitFor(() => expect(cb.onStale).toHaveBeenCalledTimes(1));
		expect(mockToast.error.mock.calls[0][0]).toContain(phrase);
		expect(cb.onOpenChange).toHaveBeenCalledWith(false);
	});

	it('recherche refusée 409 BANK_ACCOUNT_CLOSED → erreur affichée, modale ouverte', async () => {
		mockApi.getGroupCandidates.mockRejectedValue({
			code: 'BANK_ACCOUNT_CLOSED',
			message: 'Le compte bancaire est clos.',
			status: 409,
		});
		const { r, cb } = monter();
		await r.findByTestId('group-load-error');
		expect(texte(r, 'group-load-error')).toBe('Le compte bancaire est clos.');
		expect(cb.onStale).not.toHaveBeenCalled();
	});
});

describe('GroupSettlementModal — gardes de concurrence (D4, AC3)', () => {
	it('une réponse de recherche hors d’ordre est ignorée', async () => {
		let resolveFirst!: (v: GroupCandidatesResponse) => void;
		mockApi.getGroupCandidates
			.mockImplementationOnce(() => new Promise((res) => (resolveFirst = res)))
			.mockImplementationOnce(async () => response([inv(15, '300.0000', 'Récente')]));
		const { r } = monter();
		await fireEvent.input(r.getByTestId('group-search-input'), { target: { value: 'Récente' } });
		await r.findAllByTestId('group-result-row');
		resolveFirst(response([inv(12, '500.0000', 'Ancienne')]));
		await new Promise((res) => setTimeout(res, 20));
		expect(r.getAllByTestId('group-result-row').map((x) => x.getAttribute('data-invoice-id'))).toEqual(['15']);
	});

	it('fermée pendant l’envoi : rouverte sur une autre transaction, la modale est propre et n’est pas touchée', async () => {
		serve([inv(12, '500.0000'), inv(15, '300.0000', 'Autre SàRL')]);
		const { r, cb } = monter(proposal('800.00'));
		await ajouter(r, 12);
		await ajouter(r, 15);
		await waitFor(() => expect(submit(r).disabled).toBe(false));
		let resolveAccept!: (v: AcceptResponse) => void;
		mockApi.acceptProposals.mockImplementation(() => new Promise((res) => (resolveAccept = res)));
		await fireEvent.click(submit(r));
		await r.rerender({ open: false });
		await r.rerender({ open: true, proposal: proposal('50.00', 77, 'Autre') });
		await waitFor(() =>
			expect(mockApi.getGroupCandidates).toHaveBeenLastCalledWith(77, { q: 'Autre', exclude: [] }),
		);
		expect(r.queryAllByTestId('group-imputed-row')).toHaveLength(0);
		resolveAccept({
			accepted: [],
			failed: [{ bankTransactionId: TX, errorCode: 'PERIOD_LOCKED', details: null }],
		});
		await new Promise((res) => setTimeout(res, 20));
		// Le refus d'une session précédente n'agit pas sur la modale rouverte.
		expect(r.queryByTestId('group-refusal')).toBeNull();
		expect(cb.onSuccess).not.toHaveBeenCalled();
	});

	it('fermée pendant l’envoi, le règlement passé : le parent relit sa liste', async () => {
		serve([inv(12, '500.0000'), inv(15, '300.0000', 'Autre SàRL')]);
		const { r, cb } = monter(proposal('800.00'));
		await ajouter(r, 12);
		await ajouter(r, 15);
		await waitFor(() => expect(submit(r).disabled).toBe(false));
		let resolveAccept!: (v: AcceptResponse) => void;
		mockApi.acceptProposals.mockImplementation(() => new Promise((res) => (resolveAccept = res)));
		await fireEvent.click(submit(r));
		await r.rerender({ open: false });
		resolveAccept({
			accepted: [
				{
					bankTransactionId: TX,
					invoiceId: 0,
					supplierInvoiceId: null,
					journalEntryId: 9,
					score: { total: 1, amountScore: 0, referenceScore: 0, contactScore: 0 },
				},
			],
			failed: [],
		});
		await waitFor(() => expect(cb.onSuccess).toHaveBeenCalledTimes(1));
		expect(mockToast.success).not.toHaveBeenCalled();
	});
});

// Story 43-3 (#823, D6, C-43-18) — le côté fournisseur, dans un commit retirable si la 43-1b sort.
describe('GroupSettlementModal — un débit règle plusieurs factures fournisseur (D6)', () => {
	function sup(id: number, raw: string, contact = 'Fournisseur SA'): GroupCandidate {
		return {
			kind: 'supplier_invoice',
			id,
			internalNumber: `FF-${id}`,
			supplierInvoiceNumber: `R-${id}`,
			contactName: contact,
			date: '2026-10-01',
			dueDate: null,
			paymentReference: null,
			amountDue: raw.slice(0, raw.indexOf('.') + 3),
			amountDueRaw: raw,
		};
	}

	async function deuxFournisseurs(amount: string) {
		serve([sup(7, '500.0000'), sup(8, '300.0000', 'Autre fournisseur')], {
			direction: 'supplier',
			transactionAmount: amount,
		});
		const m = monter(proposal(amount));
		await ajouter(m.r, 7);
		await ajouter(m.r, 8);
		return m;
	}

	it('débit supérieur aux factures : frais bancaires préremplis, corps supplier_invoice_group', async () => {
		const { r } = await deuxFournisseurs('-800.40');
		expect(texte(r, 'group-gap')).toBe('Débit supérieur aux factures de 0.40');
		await waitFor(() => expect(champCompte()?.value).toBe('6940 — Frais bancaires'));
		await waitFor(() => expect(submit(r).disabled).toBe(false));
		mockApi.acceptProposals.mockResolvedValue({ accepted: [], failed: [] });
		await fireEvent.click(submit(r));
		await waitFor(() => expect(mockApi.acceptProposals).toHaveBeenCalledTimes(1));
		expect(mockApi.acceptProposals.mock.calls[0][1]).toEqual([
			{
				type: 'supplier_invoice_group',
				bankTransactionId: TX,
				supplierInvoiceIds: [7, 8],
				differenceAccountId: 6940,
			},
		]);
	});

	it('débit inférieur aux factures : compte d’arrondi prérempli ; colonne « Fournisseur »', async () => {
		const { r } = await deuxFournisseurs('-799.60');
		expect(texte(r, 'group-gap')).toBe('Débit inférieur aux factures de 0.40');
		await waitFor(() => expect(champCompte()?.value).toBe('6960 — Différences d’arrondi'));
		expect(r.getAllByTestId('group-imputed-row')[0].textContent).toContain('FF-7');
	});

	it('refus d’une facture fournisseur : supplierInvoiceId lu, la facture retirée et nommée', async () => {
		const { r } = await deuxFournisseurs('-800.00');
		await waitFor(() => expect(submit(r).disabled).toBe(false));
		mockApi.acceptProposals.mockResolvedValue({
			accepted: [],
			failed: [
				{
					bankTransactionId: TX,
					errorCode: 'RECONCILIATION_SUPPLIER_PAYMENT_HOLD',
					details: { reason: 'other_invoice_in_payment_batch', supplierInvoiceId: 8, invoiceNumber: 'FF-8' },
				},
			],
		});
		await fireEvent.click(submit(r));
		await r.findByTestId('group-refusal');
		expect(texte(r, 'group-refusal')).toContain('Facture FF-8 : Une autre facture fournisseur');
		expect(r.getAllByTestId('group-imputed-row').map((x) => x.getAttribute('data-invoice-id'))).toEqual(['7']);
	});
});
