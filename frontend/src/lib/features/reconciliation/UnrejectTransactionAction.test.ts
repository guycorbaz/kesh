/**
 * Story 28-5b (#526) — V2 : la mention « Rejetée le … » et le bouton
 * « Annuler le rejet » d'une ligne du détail d'import.
 *
 * ⚠️ Chaque test nomme la MUTATION qu'il attrape. Patron :
 * `CancelReconciliationDialog.test.ts` — mocks hoistés AVANT l'import du
 * composant.
 */
import { describe, it, expect, vi, afterEach, beforeEach } from 'vitest';
import { render, fireEvent, cleanup, waitFor } from '@testing-library/svelte';
import type { BankTransactionDto } from '$lib/features/bank-import/bank-import.types';

vi.mock('./reconciliation.api', () => ({
	unrejectTransaction: vi.fn(),
}));
vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_k: string, fallback: string, args?: Record<string, string | number>) =>
		args ? fallback.replace(/\{ \$(\w+) \}/g, (_m, k) => String(args[k])) : fallback,
	i18nLocale: () => 'fr-CH',
}));
vi.mock('$lib/shared/utils/notify', () => ({
	notifyError: vi.fn(),
	notifySuccess: vi.fn(),
	notifyWarning: vi.fn(),
	notifyInfo: vi.fn(),
}));

import * as api from './reconciliation.api';
import * as notify from '$lib/shared/utils/notify';
import UnrejectTransactionAction from './UnrejectTransactionAction.svelte';

const mockApi = vi.mocked(api);
const mockNotify = vi.mocked(notify);

function rejected(partial: Partial<BankTransactionDto> = {}): BankTransactionDto {
	return {
		id: 42,
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
		autoMatchRejectedAt: '2026-10-13T08:30:00Z',
		...partial,
	};
}

function monter(canManage: boolean, tx = rejected(), onUnrejected = vi.fn()) {
	return { onUnrejected, ...render(UnrejectTransactionAction, { tx, canManage, onUnrejected }) };
}

beforeEach(() => vi.clearAllMocks());
afterEach(() => cleanup());

describe('UnrejectTransactionAction', () => {
	it('canManage=false ⇒ la mention, datée, et aucun bouton (mutation M10 : bouton sans condition)', () => {
		const { getByTestId, queryByTestId } = monter(false);
		const mention = getByTestId('detail-tx-rejected-on').textContent ?? '';
		expect(mention).toContain('Rejetée le');
		// La date est formatée (dateStyle medium, fr-CH) : ni la chaîne ISO brute, ni l'heure.
		expect(mention).toContain('2026');
		expect(mention).not.toContain('T08:30');
		expect(queryByTestId('detail-tx-unreject')).toBeNull();
	});

	it('canManage=true ⇒ la mention et le bouton', () => {
		const { getByTestId } = monter(true);
		expect(getByTestId('detail-tx-rejected-on')).not.toBeNull();
		expect(getByTestId('detail-tx-unreject').textContent).toContain('Annuler le rejet');
	});

	it('transaction non rejetée ⇒ rien n’est rendu', () => {
		const { queryByTestId } = monter(true, rejected({ autoMatchRejectedAt: null }));
		expect(queryByTestId('detail-tx-rejected-on')).toBeNull();
		expect(queryByTestId('detail-tx-unreject')).toBeNull();
	});

	it('clic ⇒ UN appel, bouton désactivé pendant ; succès ⇒ notifySuccess puis onUnrejected', async () => {
		let resolve!: () => void;
		mockApi.unrejectTransaction.mockReturnValue(
			new Promise<void>((r) => {
				resolve = r;
			}),
		);
		const { getByTestId, onUnrejected } = monter(true);
		const bouton = getByTestId('detail-tx-unreject') as HTMLButtonElement;
		await fireEvent.click(bouton);
		await waitFor(() => expect(bouton.disabled).toBe(true));
		// Un second clic pendant l'appel n'envoie rien.
		await fireEvent.click(bouton);
		expect(mockApi.unrejectTransaction).toHaveBeenCalledTimes(1);
		expect(mockApi.unrejectTransaction).toHaveBeenCalledWith(42);
		expect(onUnrejected).not.toHaveBeenCalled();
		resolve();
		await waitFor(() => expect(onUnrejected).toHaveBeenCalledTimes(1));
		expect(mockNotify.notifySuccess).toHaveBeenCalledTimes(1);
		expect(mockNotify.notifySuccess.mock.calls[0][0]).toContain('Rejet annulé');
		expect(mockNotify.notifyError).not.toHaveBeenCalled();
	});

	it('échec ⇒ notifyError(message du serveur) ET onUnrejected (mutation M11 : pas de relecture sur échec)', async () => {
		mockApi.unrejectTransaction.mockRejectedValue({
			code: 'RECONCILIATION_TRANSACTION_NOT_REJECTED',
			message: 'Message du serveur, déjà traduit.',
			status: 409,
		});
		const { getByTestId, onUnrejected } = monter(true);
		await fireEvent.click(getByTestId('detail-tx-unreject'));
		await waitFor(() => expect(onUnrejected).toHaveBeenCalledTimes(1));
		expect(mockNotify.notifyError).toHaveBeenCalledWith('Message du serveur, déjà traduit.');
		expect(mockNotify.notifySuccess).not.toHaveBeenCalled();
	});
});
