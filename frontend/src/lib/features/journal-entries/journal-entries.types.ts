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
	/**
	 * Lettrage (Story 15-1a-i, #518) — clé du groupe, son code affiché et son
	 * origine (`document`, `reversal`, `manual`). Toujours présents, `null` si la
	 * ligne est ouverte : l'API les rend toujours, les déclarer optionnels
	 * mentirait sur le contrat.
	 */
	letteringKey: number | null;
	letteringCode: string | null;
	letteringOrigin: 'document' | 'reversal' | 'manual' | null;
	/**
	 * Story 37-3a/37-3b — partie en devise ; **omise** pour une ligne CHF (C-37-9).
	 * `currency` est la devise **du compte**, lue par le serveur (T3).
	 */
	foreign?: ForeignLineResponse;
}

/** Story 37-3b (AC-3) — `foreign` d'une ligne rendue par le serveur. */
export interface ForeignLineResponse {
	/** Devise du compte de la ligne (toujours présente, T3). */
	currency: string;
	/** Montant en devise, chaîne décimale sans zéros terminaux. */
	amount: string;
	/** Cours par unité, chaîne décimale sans zéros terminaux. */
	rate: string;
	rateSource: 'afc' | 'manual' | 'revaluation';
}

/**
 * Story 37-3b (AC-2) — `foreign` d'une ligne envoyée. `currency` est facultatif
 * pour l'API (37-3a AC-6) ; l'écran l'envoie **toujours** (la devise affichée) :
 * c'est la garde contre un compte dont la devise a changé depuis le chargement
 * (`FX_CURRENCY_MISMATCH`).
 */
export interface ForeignLineRequest {
	currency?: string;
	amount: string;
	rate?: string;
	chf?: string;
	rateSource?: 'afc' | 'manual';
	side?: 'debit' | 'credit';
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
	/** Numéro de la pièce, ou du compte archivé. `null` quand il n'y en a pas. */
	reversalBlockedLabel: string | null;
	/**
	 * L'écriture se modifie-t-elle (Story 15-8a, #532) ? Motif d'écran, sur
	 * l'état présent : le `PUT` reste seul juge du corps, de la date et de la
	 * version.
	 */
	modifiable: boolean;
	modificationBlockedBy: ModificationBlocker | null;
	/**
	 * Numéro de pièce, nom de l'exercice postérieur clos, borne du verrou
	 * (`AAAA-MM-JJ`), ou code de lettrage ; `null` sinon.
	 */
	modificationBlockedLabel: string | null;
	/**
	 * Story 30-2c (AC21) — le document de la facture fournisseur dont l'écriture est
	 * l'achat ou le règlement ; `null` = aucun document (le détail est le seul
	 * constructeur : jamais « non calculé »).
	 */
	sourceDocument: JournalEntrySourceDocument | null;
}

/** Le document d'une écriture d'achat ou de règlement fournisseur (Story 30-2c). */
export interface JournalEntrySourceDocument {
	supplierInvoiceId: number;
	supplierInvoiceNumber: string | null;
	/** Story 33-1b (#926) — le numéro interne de la facture. */
	internalNumber: string | null;
	originalFilename: string;
}

/**
 * Motif pour lequel une écriture ne peut pas être **modifiée** (Story 15-8a,
 * D8) — les treize codes d'écran, dans l'ordre de précédence du serveur
 * (`ENTRY_IS_FX_REVALUATION`, Story 37-3a, juste après la garde d'écriture,
 * avant `PERIOD_LOCKED` ; `ENTRY_LETTERED`, Story 15-1a-ii, en dernier : il
 * parle après le verrou de période).
 *
 * ⚠️ `ALREADY_REVERSED` est le code d'**écran** (vocabulaire de
 * `ReversalBlocker`) ; le `PUT` rend `ENTRY_IS_REVERSED`. Pas d'`ACCOUNT_ARCHIVED` :
 * un compte archivé ne gèle pas l'écriture, on le remplace.
 */
export type ModificationBlocker =
	| 'FISCAL_YEAR_CLOSED'
	| 'LATER_FISCAL_YEAR_CLOSED'
	| Exclude<ReversalBlocker, 'ACCOUNT_ARCHIVED'>
	| 'DETACHED_SUPPLIER_SETTLEMENT'
	| 'ENTRY_IS_FX_REVALUATION'
	| 'PERIOD_LOCKED'
	| 'ENTRY_LETTERED';

export interface CreateJournalEntryLineRequest {
	accountId: number;
	/** Montant décimal stringifié, point décimal (ex: "100.00"). En devise : le CHF. */
	debit: string;
	credit: string;
	/** Projet analytique optionnel de la ligne (Epic 19, Story 19-2). */
	projectId?: number | null;
	/** Story 37-3b — partie en devise (absente pour une ligne CHF). */
	foreign?: ForeignLineRequest;
}

/**
 * Story 37-3b (AC-2 bis) — corps de `POST /journal-entries/conversions` : deux
 * écritures ; `completeDifference` demande au serveur de calculer la ligne de
 * différence (A31) — l'écran ne l'envoie pas, il porte la ligne qu'il montre.
 */
export interface CreateConversionRequest {
	entries: [CreateJournalEntryRequest, CreateJournalEntryRequest];
	completeDifference?: boolean;
	/**
	 * Story 53-1b (#1365) — la confirmation qui lève le refus
	 * `JOURNAL_ENTRY_PROBABLE_DUPLICATE` de la conversion. Au niveau du **corps** : le
	 * champ n'est **pas** dans `entries[i]` (type {@link CreateJournalEntryRequest}).
	 */
	confirmNotDuplicate?: boolean;
}

/** Réponse 201 de la conversion : les deux écritures, dans l'ordre du corps. */
export interface ConversionResponse {
	entries: [JournalEntryResponse, JournalEntryResponse];
}

export interface CreateJournalEntryRequest {
	entryDate: string;
	journal: Journal;
	description: string;
	lines: CreateJournalEntryLineRequest[];
}

/**
 * Corps de `POST /journal-entries` (Story 53-1a, D5) : celui de
 * {@link CreateJournalEntryRequest}, plus `confirmNotDuplicate` — la confirmation
 * qui lève le refus `JOURNAL_ENTRY_PROBABLE_DUPLICATE`.
 *
 * ⛔ Le champ vit **ici et nulle part ailleurs** : `CreateJournalEntryRequest` est
 * étendu par le corps du `PUT` (qui l'ignore) et employé par les entrées de la
 * conversion (`entries[i]`, qui ne le portent pas : c'est le **corps** de la
 * conversion, {@link CreateConversionRequest}, qui porte le sien, Story 53-1b) ; un
 * test de types le garde.
 */
export type CreateJournalEntryBody = CreateJournalEntryRequest & {
	confirmNotDuplicate?: boolean;
};

/** Corps du `PUT` (Story 15-8a) : celui du `POST`, plus la version lue. */
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
