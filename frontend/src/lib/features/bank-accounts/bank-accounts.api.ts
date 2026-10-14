// Story 8-5a-zero — Client API pour les bank_accounts (foundation
// `journal_account_id` link). Aucun call de réconciliation manuelle ici.
//
// Story v014-1 — étendu avec CRUD complet post-onboarding :
// - `createBankAccount`, `updateBankAccount`, `archiveBankAccount`.
// - `listBankAccounts(includeArchived?)` avec param optionnel pour toggle UI.
// - `BankAccountSummary` étendu avec `currentBalance` et `archived: boolean`.
//
// Story 25-6-a (#389) — les montants restent des chaînes décimales (`big.js`
// pour tout calcul) ; trois champs du relevé CAMT.053 portant un solde de
// clôture dont la période se termine le plus tard (`period_to`) (et non du dernier importé).
//
// Story 28-4b (#642) — clôture d'un compte bancaire : `closedOn`, second booléen
// `includeClosed` de `listBankAccounts` (C-28-4-20), `closeBankAccount` et
// `reopenBankAccount`.

import { apiClient } from '$lib/shared/utils/api-client';

export interface BankAccountSummary {
	id: number;
	bankName: string;
	/**
	 * Story 26-2a (#622) — description optionnelle (« Personnel »), `null` si
	 * absente. **Obligatoire** dans le type : `npm run check` signale tout objet
	 * qui l'oublierait. S'affiche par `bankAccountName` (`format.ts`).
	 */
	description: string | null;
	iban: string;
	qrIban: string | null;
	isPrimary: boolean;
	journalAccountId: number | null;
	version: number;
	/** Story v014-1 — true si soft-deleted (archivé). */
	archived: boolean;
	/**
	 * Story 28-4b (#642) — date de clôture `AAAA-MM-JJ` d'un compte **clos** (en
	 * lecture seule, historique conservé, réversible), `null` s'il est ouvert. La
	 * date est informative : c'est l'état qui garde (C-28-4-3).
	 */
	closedOn: string | null;
	/**
	 * Story v014-1 T5 — **solde comptable** calculé serveur-side depuis
	 * journal_entry_lines (pas le solde bancaire). `null` si `journal_account_id`
	 * n'est pas configuré sur ce compte.
	 *
	 * Story 25-6-a (#389) — reste une **chaîne décimale** (jamais un `Number`) :
	 * tout calcul passe par `big.js`, tout affichage par `formatChfBalance`.
	 */
	currentBalance: string | null;
	/**
	 * Story v014-1 F13 Pass 1 code review (AC#30) — date `MAX(je.entry_date)`
	 * agrégée sur le `journal_account_id` lié. Format ISO `YYYY-MM-DD`.
	 * `null` si journal_account_id NULL ou aucune écriture.
	 */
	lastTransactionDate: string | null;
	/** Story 25-6-a — solde de clôture du relevé dont la période se termine le plus tard (chaîne décimale). */
	statementClosingBalance: string | null;
	/** Story 25-6-a — date de ce relevé (`YYYY-MM-DD`). */
	statementDate: string | null;
	/**
	 * Story 25-6-a — solde comptable à `statementDate`, corrigé des dates de
	 * valeur. `null` sans relevé, sans compte lié, ou si plusieurs comptes
	 * bancaires partagent le compte de grand livre.
	 */
	ledgerBalanceAtStatement: string | null;
	/**
	 * Story 29-1a (#232) — écart livres − relevé, **calculé par le serveur**
	 * (`bank_accounts::statement_gap`, au centime, demi loin de zéro) : chaîne
	 * décimale, `null` s'il est nul ou ne se calcule pas. Seule définition de
	 * l'écart ; le frontend ne le recalcule pas.
	 */
	statementGap: string | null;
}

/**
 * Type backend raw : les montants sont sérialisés en chaînes par serde (Decimal
 * feature `serde-str`) et le restent.
 */
interface BankAccountSummaryRaw {
	id: number;
	bankName: string;
	description: string | null;
	iban: string;
	qrIban: string | null;
	isPrimary: boolean;
	journalAccountId: number | null;
	version: number;
	archived: boolean;
	// Story 28-4b — absent d'un serveur antérieur à la 0.13.3 : rendu `null`.
	closedOn?: string | null;
	currentBalance: string | null;
	lastTransactionDate: string | null;
	// Absents des réponses de mutation (POST/PUT/PATCH/DELETE rendent un compte nu).
	statementClosingBalance?: string | null;
	statementDate?: string | null;
	ledgerBalanceAtStatement?: string | null;
	statementGap?: string | null;
}

function parseBankAccount(raw: BankAccountSummaryRaw): BankAccountSummary {
	return {
		id: raw.id,
		bankName: raw.bankName,
		description: raw.description ?? null,
		iban: raw.iban,
		qrIban: raw.qrIban,
		isPrimary: raw.isPrimary,
		journalAccountId: raw.journalAccountId,
		version: raw.version,
		archived: raw.archived,
		closedOn: raw.closedOn ?? null,
		currentBalance: raw.currentBalance ?? null,
		lastTransactionDate: raw.lastTransactionDate,
		statementClosingBalance: raw.statementClosingBalance ?? null,
		statementDate: raw.statementDate ?? null,
		ledgerBalanceAtStatement: raw.ledgerBalanceAtStatement ?? null,
		statementGap: raw.statementGap ?? null,
	};
}

/**
 * GET /api/v1/bank-accounts — liste les bank_accounts de la company
 * courante (multi-tenant scoping serveur-side via JWT).
 *
 * Par défaut, ni archivés ni clos : c'est la liste des comptes où l'on peut
 * **écrire** (sélecteurs d'import, de règlement, de lot ; tuile de l'accueil).
 *
 * @param includeArchived Si `true`, inclut aussi les comptes archivés (toggle UI).
 * @param includeClosed Story 28-4b (#642, C-28-4-20) — si `true`, inclut aussi
 *   les comptes clos (bascule de l'écran *Comptes bancaires* ; réglages de
 *   facturation, alignés sur le serveur qui compte un compte clos comme lié à
 *   son compte comptable, C-28-4-7). Second booléen et non objet d'options :
 *   les appelants existants (`listBankAccounts()`, `listBankAccounts(false)`)
 *   gardent leur sens et leur code.
 */
export async function listBankAccounts(
	includeArchived = false,
	includeClosed = false,
): Promise<BankAccountSummary[]> {
	const params = new URLSearchParams();
	if (includeArchived) params.set('includeArchived', 'true');
	if (includeClosed) params.set('includeClosed', 'true');
	const query = params.toString();
	const path = query === '' ? '/api/v1/bank-accounts' : `/api/v1/bank-accounts?${query}`;
	const raw = await apiClient.get<BankAccountSummaryRaw[]>(path);
	return raw.map(parseBankAccount);
}

/**
 * POST /api/v1/bank-accounts/{id}/close — **clôt** un compte bancaire
 * (Story 28-4a/28-4b, #642 ; Comptable+). Le compte passe en lecture seule :
 * plus rien ne s'y enregistre, son historique reste lisible ; réversible par
 * {@link reopenBankAccount}.
 *
 * Refus (message traduit par le serveur, à afficher par `errorMessageOf`) :
 * `412 BANK_ACCOUNT_CANNOT_CLOSE_PRIMARY`, `412 BANK_ACCOUNT_HAS_PENDING_TRANSACTIONS`,
 * `412 BANK_ACCOUNT_HAS_OPEN_PAYMENT_BATCHES`, `409 OPTIMISTIC_LOCK_CONFLICT`,
 * `409 BANK_ACCOUNT_CLOSED` (déjà clos), `404` (archivé, inexistant).
 *
 * @param closedOn Date de clôture `AAAA-MM-JJ`, informative ; l'écran l'envoie
 *   toujours (préremplie à la date **locale** du jour).
 */
export async function closeBankAccount(
	id: number,
	version: number,
	closedOn: string,
): Promise<BankAccountSummary> {
	const raw = await apiClient.post<BankAccountSummaryRaw>(`/api/v1/bank-accounts/${id}/close`, {
		version,
		closedOn,
	});
	return parseBankAccount(raw);
}

/**
 * POST /api/v1/bank-accounts/{id}/reopen — **rouvre** un compte bancaire clos
 * (Story 28-4a/28-4b, #642 ; Comptable+). Le compte ne redevient pas principal.
 * Refus : `409 BANK_ACCOUNT_NOT_CLOSED`, `409 OPTIMISTIC_LOCK_CONFLICT`, `404`.
 */
export async function reopenBankAccount(id: number, version: number): Promise<BankAccountSummary> {
	const raw = await apiClient.post<BankAccountSummaryRaw>(`/api/v1/bank-accounts/${id}/reopen`, {
		version,
	});
	return parseBankAccount(raw);
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
	return parseBankAccount({ ...raw, currentBalance: raw.currentBalance ?? null, lastTransactionDate: raw.lastTransactionDate ?? null });
}

/**
 * Payload de création — POST /api/v1/bank-accounts (Story v014-1).
 */
export interface NewBankAccountPayload {
	bankName: string;
	/** Story 26-2a — optionnel : l'omettre crée un compte sans description. */
	description?: string | null;
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
	return parseBankAccount({ ...raw, currentBalance: raw.currentBalance ?? null, lastTransactionDate: raw.lastTransactionDate ?? null });
}

/**
 * Payload d'édition complète — PUT /api/v1/bank-accounts/{id} (Story v014-1).
 */
export interface UpdateBankAccountPayload {
	bankName: string;
	/**
	 * Story 26-2a — **obligatoire** : le `PUT` remplace tout, l'omettre
	 * **efface** la description (C-26-2-19). `null` pour l'effacer exprès.
	 */
	description: string | null;
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
	return parseBankAccount({ ...raw, currentBalance: raw.currentBalance ?? null, lastTransactionDate: raw.lastTransactionDate ?? null });
}

/**
 * DELETE /api/v1/bank-accounts/{id} — soft-delete (archive) un compte bancaire
 * (Comptable+). Refus 412 si transactions existent OU si primary + autres
 * comptes ni archivés ni clos (Story 28-4a) ; refus 409 BANK_ACCOUNT_CLOSED
 * d'un compte clos.
 */
export async function archiveBankAccount(
	id: number,
	version: number,
): Promise<BankAccountSummary> {
	const raw = await apiClient.delete<BankAccountSummaryRaw>(`/api/v1/bank-accounts/${id}`, {
		version,
	});
	return parseBankAccount({ ...raw, currentBalance: raw.currentBalance ?? null, lastTransactionDate: raw.lastTransactionDate ?? null });
}
