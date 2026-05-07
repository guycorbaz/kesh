// Story 8-5a-zero — Client API pour les bank_accounts (foundation
// `journal_account_id` link). Aucun call de réconciliation manuelle ici
// (8-5a-base / 8-5a-bis livreront la suite).

import { apiClient } from '$lib/shared/utils/api-client';

export interface BankAccountSummary {
	id: number;
	bankName: string;
	iban: string;
	qrIban: string | null;
	isPrimary: boolean;
	journalAccountId: number | null;
	version: number;
}

/**
 * GET /api/v1/bank-accounts — liste les bank_accounts de la company
 * courante (multi-tenant scoping serveur-side via JWT).
 */
export async function listBankAccounts(): Promise<BankAccountSummary[]> {
	return apiClient.get<BankAccountSummary[]>('/api/v1/bank-accounts');
}

/**
 * PATCH /api/v1/bank-accounts/{id} — met à jour `journalAccountId`.
 *
 * @param id Bank account id.
 * @param journalAccountId Compte du plan comptable à lier, ou `null` pour délier.
 * @param version Version courante (optimistic lock).
 */
export async function updateBankAccountJournalLink(
	id: number,
	journalAccountId: number | null,
	version: number,
): Promise<BankAccountSummary> {
	return apiClient.patch<BankAccountSummary>(`/api/v1/bank-accounts/${id}`, {
		journalAccountId,
		version,
	});
}
