/**
 * Le détail d'un import bancaire, côté « Supprimer l'import » — Story 27-1 (#632),
 * revue de code P2 (E-P2-5).
 *
 * Le composant `DeleteBankImportAction` est tenu par son propre Vitest ; ce
 * fichier tient ce que la PAGE lui passe : `canManage` dérivé du rôle. Sans
 * lui, une page qui passerait `canManage={true}` montrerait le bouton au rôle
 * Consultation (le serveur refuserait encore, H4), et rien ne rougirait.
 *
 * ⚠️ Chaque test nomme la MUTATION qu'il attrape. Patron :
 * `supplier-invoices/[id]/supplier-settlement-page.test.ts` — mocks hoistés
 * AVANT l'import du composant.
 */
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, cleanup } from '@testing-library/svelte';
import type { BankImportDetailResponse } from '$lib/features/bank-import/bank-import.types';

vi.mock('$app/navigation', () => ({ goto: vi.fn() }));
vi.mock('$app/state', () => ({
	page: {
		params: { id: '77' },
		url: new URL('http://localhost/bank-import/77'),
	},
}));
vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_k: string, fallback: string, args?: Record<string, string | number>) =>
		args ? fallback.replace(/\{\s*\$(\w+)\s*\}/g, (_, n) => String(args[n] ?? '')) : fallback,
	i18nLocale: () => 'fr-CH',
}));
vi.mock('$lib/shared/utils/notify', () => ({
	notifyError: vi.fn(),
	notifySuccess: vi.fn(),
	notifyWarning: vi.fn(),
	notifyInfo: vi.fn(),
}));
const role = { value: 'Comptable' };
vi.mock('$lib/app/stores/auth.svelte', () => ({
	authState: {
		get currentUser() {
			return { role: role.value, username: 'u', userId: '1' };
		},
	},
}));
const getMock = vi.fn();
vi.mock('$lib/features/bank-import/bank-import.api', () => ({
	getBankImportDetail: (id: number) => getMock(id),
	deleteBankImport: vi.fn(),
}));
const unrejectMock = vi.fn();
vi.mock('$lib/features/reconciliation/reconciliation.api', () => ({
	unrejectTransaction: (id: number) => unrejectMock(id),
	getReconciliationTransaction: vi.fn(),
	cancelReconciliation: vi.fn(),
}));

import { fireEvent, waitFor } from '@testing-library/svelte';
import Page from './+page.svelte';

function detail(): BankImportDetailResponse {
	return {
		id: 77,
		bankAccountId: 3,
		filename: 'releve-mars.csv',
		fileHash: 'a'.repeat(64),
		sourceFormat: 'CSV',
		statementId: null,
		periodFrom: '2026-03-02',
		periodTo: '2026-03-04',
		openingBalance: null,
		closingBalance: null,
		transactionCount: 1,
		importedAt: '2026-03-05T10:00:00',
		transactions: [
			{
				id: 1,
				bookingDate: '2026-03-02',
				valueDate: null,
				amount: '10.00',
				currency: 'CHF',
				reference: null,
				details: 'test',
				endToEndId: null,
				transactionId: null,
				counterpartyIban: null,
				counterpartyName: null,
				status: 'pending',
				matchedEntryId: null,
				autoMatchRejectedAt: null,
			},
		],
	};
}

/**
 * Story 28-5b (#526, V3) — trois lignes : une rejetée, une en attente non
 * rejetée, une rapprochée (marquée, état forgé : la mention ne doit pas y
 * paraître, le serveur refuserait l'annulation).
 */
function detailTroisLignes(): BankImportDetailResponse {
	const base = detail();
	const ligne = base.transactions[0];
	return {
		...base,
		transactionCount: 3,
		transactions: [
			{ ...ligne, id: 1, autoMatchRejectedAt: '2026-10-13T08:30:00Z' },
			{ ...ligne, id: 2, autoMatchRejectedAt: null },
			{
				...ligne,
				id: 3,
				status: 'reconciled',
				matchedEntryId: 9,
				autoMatchRejectedAt: '2026-10-13T08:30:00Z',
			},
		],
	};
}

beforeEach(() => {
	vi.clearAllMocks();
	getMock.mockResolvedValue(detail());
});
afterEach(() => cleanup());

describe('page détail d’import — bouton « Supprimer l’import » selon le rôle', () => {
	for (const r of ['Admin', 'Comptable']) {
		it(`${r} ⇒ le bouton est rendu (mutation : canManage toujours faux)`, async () => {
			role.value = r;
			const { findByTestId } = render(Page);
			expect(await findByTestId('bank-import-delete-button')).not.toBeNull();
		});
	}

	it('Consultation ⇒ aucun bouton (mutation : canManage={true} passé par la page — E-P2-5)', async () => {
		role.value = 'Consultation';
		const { findByTestId, queryByTestId } = render(Page);
		// Le détail est chargé : l'absence du bouton n'est pas celle d'une page vide.
		expect((await findByTestId('detail-filename')).textContent).toBe('releve-mars.csv');
		expect(queryByTestId('bank-import-delete-button')).toBeNull();
	});
});

describe('page détail d’import — la transaction rejetée (Story 28-5b, #526, V3)', () => {
	it('seule la ligne rejetée porte la mention « Rejetée le » (ni l’attente, ni la rapprochée)', async () => {
		role.value = 'Comptable';
		getMock.mockResolvedValue(detailTroisLignes());
		const { findAllByTestId, container } = render(Page);
		const mentions = await findAllByTestId('detail-tx-rejected-on');
		expect(mentions).toHaveLength(1);
		const ligne = (id: number) => container.querySelector(`[data-testid="detail-tx-row"][data-tx-id="${id}"]`);
		expect(ligne(1)?.querySelector('[data-testid="detail-tx-rejected-on"]')).not.toBeNull();
		expect(ligne(1)?.querySelector('[data-testid="detail-tx-unreject"]')).not.toBeNull();
		for (const id of [2, 3]) {
			expect(ligne(id)?.querySelector('[data-testid="detail-tx-rejected-on"]')).toBeNull();
			expect(ligne(id)?.querySelector('[data-testid="detail-tx-unreject"]')).toBeNull();
		}
	});

	it('Consultation ⇒ la mention, sans le bouton (la page passe canManage au composant)', async () => {
		role.value = 'Consultation';
		getMock.mockResolvedValue(detailTroisLignes());
		const { findAllByTestId, queryByTestId } = render(Page);
		expect(await findAllByTestId('detail-tx-rejected-on')).toHaveLength(1);
		expect(queryByTestId('detail-tx-unreject')).toBeNull();
	});

	// Revue de code P2 ciblée (P2-1) — la bannière d'erreur masque les lignes : une relecture en
	// échec ne doit pas condamner la page. Deux annulations lancées coup sur coup ; la relecture de
	// la première échoue, celle de la seconde réussit : les lignes reviennent, sans bannière
	// (mutation M24 : `errorMessage = null` retiré de `load()`).
	it('une relecture réussie après une relecture en échec efface la bannière et rend les lignes', async () => {
		role.value = 'Comptable';
		const base = detailTroisLignes();
		const deuxRejetees: BankImportDetailResponse = {
			...base,
			transactions: [base.transactions[0], { ...base.transactions[0], id: 2 }],
		};
		const apres: BankImportDetailResponse = {
			...deuxRejetees,
			transactions: deuxRejetees.transactions.map((t) => ({ ...t, autoMatchRejectedAt: null })),
		};
		getMock
			.mockResolvedValueOnce(deuxRejetees)
			.mockRejectedValueOnce({ code: 'INTERNAL_ERROR', message: 'Panne passagère.', status: 500 })
			.mockResolvedValueOnce(apres);
		const resolvers: Array<() => void> = [];
		unrejectMock.mockImplementation(
			() => new Promise<void>((r) => resolvers.push(r)),
		);

		const { findAllByTestId, findByTestId, queryByTestId } = render(Page);
		const boutons = await findAllByTestId('detail-tx-unreject');
		expect(boutons).toHaveLength(2);
		await fireEvent.click(boutons[0]);
		await fireEvent.click(boutons[1]);
		await waitFor(() => expect(resolvers).toHaveLength(2));

		resolvers[0]();
		expect((await findByTestId('bank-import-detail-error')).textContent).toContain('Panne passagère.');

		resolvers[1]();
		await waitFor(() => expect(getMock).toHaveBeenCalledTimes(3));
		expect(await findAllByTestId('detail-tx-row')).toHaveLength(2);
		expect(queryByTestId('bank-import-detail-error')).toBeNull();
		expect(queryByTestId('detail-tx-rejected-on')).toBeNull();
	});
});
