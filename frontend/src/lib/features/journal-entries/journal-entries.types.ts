/**
 * Types TypeScript pour les écritures comptables en partie double.
 *
 * Miroir des DTOs Rust définis dans `crates/kesh-api/src/routes/journal_entries.rs`.
 * Les montants sont transportés en string décimale pour éviter les
 * erreurs d'arrondi JSON (JavaScript n'a que des f64).
 */

export type Journal = 'Achats' | 'Ventes' | 'Banque' | 'Caisse' | 'OD';

export interface JournalEntryLineResponse {
	id: number;
	accountId: number;
	lineOrder: number;
	/** Montant décimal stringifié (ex: "100.00"). Parser avec big.js. */
	debit: string;
	credit: string;
}

export interface JournalEntryResponse {
	id: number;
	companyId: number;
	fiscalYearId: number;
	entryNumber: number;
	/** Date ISO YYYY-MM-DD. */
	entryDate: string;
	journal: Journal;
	description: string;
	version: number;
	lines: JournalEntryLineResponse[];
	createdAt: string;
	updatedAt: string;
}

export interface CreateJournalEntryLineRequest {
	accountId: number;
	/** Montant décimal stringifié, point décimal (ex: "100.00"). */
	debit: string;
	credit: string;
}

export interface CreateJournalEntryRequest {
	entryDate: string;
	journal: Journal;
	description: string;
	lines: CreateJournalEntryLineRequest[];
}

export interface UpdateJournalEntryRequest extends CreateJournalEntryRequest {
	version: number;
}

// ---------------------------------------------------------------------------
// Listing / pagination / tri (Story 3.4)
// ---------------------------------------------------------------------------

/**
 * Colonne de tri. Les valeurs sérialisées en PascalCase matchent exactement
 * les variants de l'enum `SortBy` côté Rust (`kesh-core/listing`).
 */
export type SortBy = 'EntryDate' | 'EntryNumber' | 'Journal' | 'Description';

export type SortDirection = 'Asc' | 'Desc';

/**
 * Paramètres de requête pour `GET /api/v1/journal-entries`.
 * Tous optionnels. Le backend applique des défauts et des clamps.
 */
export interface JournalEntryListQuery {
	description?: string;
	amountMin?: string;
	amountMax?: string;
	dateFrom?: string;
	dateTo?: string;
	journal?: Journal;
	sortBy?: SortBy;
	sortDir?: SortDirection;
	offset?: number;
	limit?: number;
}

/**
 * Envelope standard pour les listes paginées (Story 3.4).
 * Cohérent avec `crates/kesh-api/src/routes/mod.rs::ListResponse<T>`.
 */
export interface ListResponse<T> {
	items: T[];
	total: number;
	offset: number;
	limit: number;
}
