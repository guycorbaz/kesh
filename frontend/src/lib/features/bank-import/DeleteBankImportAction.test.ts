/**
 * « Supprimer l'import » — Story 27-1 (#632).
 *
 * ⚠️ Chaque test nomme la MUTATION qu'il attrape.
 */
import { describe, it, expect, vi, afterEach, beforeEach } from 'vitest';
import { render, fireEvent, cleanup, waitFor } from '@testing-library/svelte';

vi.mock('./bank-import.api', () => ({
	deleteBankImport: vi.fn(),
}));
vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_k: string, fallback: string, args?: Record<string, string | number>) =>
		args ? fallback.replace(/\{ \$(\w+) \}/g, (_m, k) => String(args[k])) : fallback,
}));

import * as api from './bank-import.api';
import DeleteBankImportAction from './DeleteBankImportAction.svelte';
import type { BankImportDetailResponse, BankTransactionDto } from './bank-import.types';

const mockApi = vi.mocked(api);

function tx(id: number, status = 'pending', matchedEntryId: number | null = null): BankTransactionDto {
	return {
		id,
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
		status,
		matchedEntryId,
	};
}

function detail(transactions: BankTransactionDto[]): BankImportDetailResponse {
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
		transactionCount: transactions.length,
		importedAt: '2026-03-05T10:00:00',
		transactions,
	};
}

function monter(d: BankImportDetailResponse, canManage = true, onDeleted = vi.fn()) {
	return render(DeleteBankImportAction, { detail: d, canManage, onDeleted });
}

beforeEach(() => vi.clearAllMocks());
afterEach(() => cleanup());

describe('DeleteBankImportAction', () => {
	it('canManage faux ⇒ ni bouton ni motif (mutation : canManage ignoré — M16)', () => {
		const { queryByTestId } = monter(detail([tx(1), tx(2, 'reconciled', 5)]), false);
		expect(queryByTestId('bank-import-delete-button')).toBeNull();
		expect(queryByTestId('bank-import-delete-blocked')).toBeNull();
	});

	it('aucune rapprochée ⇒ bouton actif, sans motif (mutation : bouton toujours désactivé)', () => {
		const { getByTestId, queryByTestId } = monter(detail([tx(1), tx(2)]));
		expect((getByTestId('bank-import-delete-button') as HTMLButtonElement).disabled).toBe(false);
		expect(queryByTestId('bank-import-delete-blocked')).toBeNull();
	});

	it('deux rapprochées ⇒ bouton désactivé et motif avec le nombre (mutation : motif sans compte)', () => {
		const { getByTestId } = monter(
			detail([tx(1, 'reconciled', 5), tx(2), tx(3, 'pending', 9)]),
		);
		expect((getByTestId('bank-import-delete-button') as HTMLButtonElement).disabled).toBe(true);
		expect(getByTestId('bank-import-delete-blocked').textContent).toContain(
			'2 transaction(s) de cet import sont rapprochées',
		);
	});

	it('le dialogue nomme le fichier, la période et le nombre (mutation : corps sans arguments)', async () => {
		const { getByTestId, findByTestId } = monter(detail([tx(1), tx(2), tx(3)]));
		await fireEvent.click(getByTestId('bank-import-delete-button'));
		const body = (await findByTestId('bank-import-delete-dialog-body')).textContent ?? '';
		expect(body).toContain('releve-mars.csv');
		expect(body).toContain('du 2026-03-02 au 2026-03-04');
		expect(body).toContain('ses 3 transaction(s)');
	});

	it('Supprimer définitivement ⇒ deleteBankImport(id de l’import) puis onDeleted(filename) (mutation : autre identifiant — M12)', async () => {
		mockApi.deleteBankImport.mockResolvedValue(undefined);
		const onDeleted = vi.fn();
		const { getByTestId, findByTestId } = monter(detail([tx(1)]), true, onDeleted);
		await fireEvent.click(getByTestId('bank-import-delete-button'));
		await fireEvent.click(await findByTestId('bank-import-delete-submit'));
		await waitFor(() => expect(onDeleted).toHaveBeenCalledWith('releve-mars.csv'));
		expect(mockApi.deleteBankImport).toHaveBeenCalledTimes(1);
		expect(mockApi.deleteBankImport).toHaveBeenCalledWith(77);
	});

	it('double clic ⇒ un seul appel (mutation : garde submitting retirée — M13)', async () => {
		let resolve: () => void = () => {};
		mockApi.deleteBankImport.mockReturnValue(new Promise<void>((r) => (resolve = r)));
		const { getByTestId, findByTestId } = monter(detail([tx(1)]));
		await fireEvent.click(getByTestId('bank-import-delete-button'));
		const submit = await findByTestId('bank-import-delete-submit');
		// Deux clics SANS attendre la mise à jour du DOM : l'attribut `disabled`
		// n'est pas encore posé, seule la garde du gestionnaire les distingue.
		void fireEvent.click(submit);
		void fireEvent.click(submit);
		resolve();
		await waitFor(() => expect(mockApi.deleteBankImport).toHaveBeenCalled());
		expect(mockApi.deleteBankImport).toHaveBeenCalledTimes(1);
	});

	it('409 ⇒ message « rapprochée entre-temps » avec reconciledCount, onDeleted non appelé', async () => {
		mockApi.deleteBankImport.mockRejectedValue({
			code: 'BANK_IMPORT_HAS_RECONCILED_TRANSACTIONS',
			message: 'serveur',
			status: 409,
			details: { reconciledCount: 2, transactionCount: 3 },
		});
		const onDeleted = vi.fn();
		const { getByTestId, findByTestId } = monter(detail([tx(1)]), true, onDeleted);
		await fireEvent.click(getByTestId('bank-import-delete-button'));
		await fireEvent.click(await findByTestId('bank-import-delete-submit'));
		const err = (await findByTestId('bank-import-delete-error')).textContent ?? '';
		expect(err).toContain('rapprochée entre-temps (2)');
		expect(onDeleted).not.toHaveBeenCalled();
	});

	it('404 ⇒ message « n’existe plus »', async () => {
		mockApi.deleteBankImport.mockRejectedValue({
			code: 'NOT_FOUND',
			message: 'serveur',
			status: 404,
		});
		const { getByTestId, findByTestId } = monter(detail([tx(1)]));
		await fireEvent.click(getByTestId('bank-import-delete-button'));
		await fireEvent.click(await findByTestId('bank-import-delete-submit'));
		expect((await findByTestId('bank-import-delete-error')).textContent).toContain(
			"Cet import n'existe plus",
		);
	});
});
