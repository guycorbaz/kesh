// Story 8-5a-zero — Client API pour les bank_accounts (foundation
// `journal_account_id` link). Aucun call de réconciliation manuelle ici.
//
// Story v014-1 — étendu avec CRUD complet post-onboarding :
// - `createBankAccount`, `updateBankAccount`, `archiveBankAccount`.
// - `listBankAccounts(includeArchived?)` avec param optionnel pour toggle UI.
// - `BankAccountSummary` étendu avec `currentBalance: number | null` (Decimal
//   sérialisé en string par serde, converti en `number` côté TS) et
//   `archived: boolean`.

import { apiClient } from '$lib/shared/utils/api-client';

export interface BankAccountSummary {
	id: number;
	bankName: string;
	iban: string;
	qrIban: string | null;
	isPrimary: boolean;
	journalAccountId: number | null;
	version: number;
	/** Story v014-1 — true si soft-deleted (archivé). */
	archived: boolean;
	/**
	 * Story v014-1 T5 — solde calculé serveur-side depuis journal_entry_lines.
	 * `null` si `journal_account_id` n'est pas configuré sur ce compte (L4
	 * limitation v0.1 — lien plan comptable obligatoire pour calcul du solde).
	 *
	 * Décimal sérialisé en string par serde (feature `serde-str`), converti
	 * en `number` ici. La précision CHF (2 décimales en pratique) ne pose pas
	 * de problème via Number ; si v0.2 nécessite plus de précision, migrer
	 * vers `decimal.js`.
	 */
	currentBalance: number | null;
}

/**
 * Type backend raw : `currentBalance` est sérialisé en string par serde
 * (Decimal feature `serde-str`). Convertir en number via `Number(...)`.
 */
interface BankAccountSummaryRaw {
	id: number;
	bankName: string;
	iban: string;
	qrIban: string | null;
	isPrimary: boolean;
	journalAccountId: number | null;
	version: number;
	archived: boolean;
	currentBalance: string | null;
}

function parseBankAccount(raw: BankAccountSummaryRaw): BankAccountSummary {
	return {
		id: raw.id,
		bankName: raw.bankName,
		iban: raw.iban,
		qrIban: raw.qrIban,
		isPrimary: raw.isPrimary,
		journalAccountId: raw.journalAccountId,
		version: raw.version,
		archived: raw.archived,
		currentBalance: raw.currentBalance == null ? null : Number(raw.currentBalance),
	};
}

/**
 * GET /api/v1/bank-accounts — liste les bank_accounts de la company
 * courante (multi-tenant scoping serveur-side via JWT).
 *
 * @param includeArchived Si `true`, inclut aussi les comptes archivés (toggle UI).
 */
export async function listBankAccounts(includeArchived = false): Promise<BankAccountSummary[]> {
	const path = includeArchived
		? '/api/v1/bank-accounts?includeArchived=true'
		: '/api/v1/bank-accounts';
	const raw = await apiClient.get<BankAccountSummaryRaw[]>(path);
	return raw.map(parseBankAccount);
}

/**
 * PATCH /api/v1/bank-accounts/{id} — met à jour `journalAccountId` uniquement
 * (legacy 8-5a-zero, scope strict). Pour édition complète, utiliser PUT.
 */
export async function updateBankAccountJournalLink(
	id: number,
	journalAccountId: number | null,
	version: number,
): Promise<BankAccountSummary> {
	const raw = await apiClient.patch<BankAccountSummaryRaw>(`/api/v1/bank-accounts/${id}`, {
		journalAccountId,
		version,
	});
	return parseBankAccount({ ...raw, currentBalance: raw.currentBalance ?? null });
}

/**
 * Payload de création — POST /api/v1/bank-accounts (Story v014-1).
 */
export interface NewBankAccountPayload {
	bankName: string;
	iban: string;
	qrIban?: string | null;
	isPrimary?: boolean;
	journalAccountId?: number | null;
}

/**
 * POST /api/v1/bank-accounts — crée un nouveau compte bancaire post-onboarding
 * (Comptable+). Transition primary silencieuse symétrique au PUT — si
 * `isPrimary=true` et un autre primary existe, l'ancien est démoté
 * silencieusement dans la même transaction.
 */
export async function createBankAccount(
	payload: NewBankAccountPayload,
): Promise<BankAccountSummary> {
	const raw = await apiClient.post<BankAccountSummaryRaw>('/api/v1/bank-accounts', payload);
	return parseBankAccount({ ...raw, currentBalance: raw.currentBalance ?? null });
}

/**
 * Payload d'édition complète — PUT /api/v1/bank-accounts/{id} (Story v014-1).
 */
export interface UpdateBankAccountPayload {
	bankName: string;
	iban: string;
	qrIban?: string | null;
	isPrimary?: boolean;
	journalAccountId?: number | null;
	version: number;
}

/**
 * PUT /api/v1/bank-accounts/{id} — édition complète d'un compte bancaire
 * (Comptable+). Optimistic lock via `version`.
 */
export async function updateBankAccount(
	id: number,
	payload: UpdateBankAccountPayload,
): Promise<BankAccountSummary> {
	const raw = await apiClient.put<BankAccountSummaryRaw>(`/api/v1/bank-accounts/${id}`, payload);
	return parseBankAccount({ ...raw, currentBalance: raw.currentBalance ?? null });
}

/**
 * DELETE /api/v1/bank-accounts/{id} — soft-delete (archive) un compte bancaire
 * (Comptable+). Refus 412 si transactions existent OU si primary + autres
 * comptes non-archivés.
 */
export async function archiveBankAccount(
	id: number,
	version: number,
): Promise<BankAccountSummary> {
	const raw = await apiClient.delete<BankAccountSummaryRaw>(`/api/v1/bank-accounts/${id}`, {
		version,
	});
	return parseBankAccount({ ...raw, currentBalance: raw.currentBalance ?? null });
}
