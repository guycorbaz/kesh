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
