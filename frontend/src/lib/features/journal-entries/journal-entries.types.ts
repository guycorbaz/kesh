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
	/** Projet analytique de la ligne (Epic 19). `null` = non taguée. */
	projectId: number | null;
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
	/** Écriture que celle-ci contre-passe (Story 24-4a, #380). `null` = ordinaire. */
	reversesEntryId: number | null;
	lines: JournalEntryLineResponse[];
	createdAt: string;
	updatedAt: string;
}

/**
 * Motif pour lequel une écriture ne peut pas être contre-passée (Story 24-4a).
 *
 * ⚠️ Un **code**, jamais une phrase : la traduction se fait ici, dans les quatre
 * locales. Les causes se cumulent côté serveur, qui n'en rend que la première
 * selon une précédence figée.
 */
export type ReversalBlocker =
	| 'IS_A_REVERSAL'
	| 'ALREADY_REVERSED'
	| 'OWNED_BY_INVOICE'
	| 'OWNED_BY_CREDIT_NOTE'
	| 'OWNED_BY_SUPPLIER_INVOICE'
	| 'OWNED_BY_SETTLEMENT'
	| 'MATCHED_BANK_TRANSACTION'
	| 'ACCOUNT_ARCHIVED';

/**
 * Détail d'une écriture — le `GET /{id}` seul, jamais la liste.
 *
 * Sans ces champs l'écran devinerait : il ne pourrait ni masquer le bouton ni
 * dire pourquoi, et se rabattrait sur un 409 découvert **après** le clic.
 */
export interface JournalEntryDetailResponse extends JournalEntryResponse {
	/** Écriture qui contre-passe celle-ci. Dérivé, pas une colonne. */
	reversedByEntryId: number | null;
	reversable: boolean;
	reversalBlockedBy: ReversalBlocker | null;
}

export interface CreateJournalEntryLineRequest {
	accountId: number;
	/** Montant décimal stringifié, point décimal (ex: "100.00"). */
	debit: string;
	credit: string;
	/** Projet analytique optionnel de la ligne (Epic 19, Story 19-2). */
	projectId?: number | null;
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
	/** Ne garder que les écritures touchant ce compte (issue #374). */
	accountId?: number;
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
