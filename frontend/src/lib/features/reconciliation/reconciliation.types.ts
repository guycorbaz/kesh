// Story 8-4 — Types DTOs réconciliation (miroir kesh-api/routes/reconciliation.rs).

export interface MatchScore {
	total: number;
	amountScore: number;
	referenceScore: number;
	contactScore: number;
}

export interface TransactionSummary {
	bookingDate: string;
	// P-M7 Pass 1 code review : `valueDate` exposé par GET /proposals
	// pour permettre au ManualMatchModal de pré-remplir le datepicker
	// avec la date de valeur (fallback bookingDate si absente).
	valueDate: string | null;
	amount: string;
	currency: string;
	counterpartyName: string | null;
}

// Story 8-5b — discriminator candidate_type. Les fields invoice/rule
// sont Some/None selon ce type. Story 30-1b (#686) : `supplier_invoice`, une
// facture fournisseur proposée pour un débit.
export type CandidateType = 'invoice' | 'rule' | 'supplier_invoice';

export interface ReconciliationCandidate {
	candidateType: CandidateType;
	// Invoice candidate fields
	/** La facture client ; `null` pour une facture fournisseur et une règle. */
	invoiceId: number | null;
	/** Story 30-1b — la facture fournisseur ; `null` sinon. */
	supplierInvoiceId: number | null;
	/** Story 30-1b — le nom du client ou du fournisseur ; `null` pour une règle. */
	contactName: string | null;
	invoiceNumber: string | null;
	/** Montant à régler : le reste dû de la facture (Story 25-4-c, #420). */
	invoiceAmount: string | null;
	/** TTC, présent seulement si la facture est déjà réglée en partie (Story 25-4-c). */
	invoiceTotalTtc: string | null;
	invoiceDate: string | null;
	// Story 8-5b — Rule candidate fields
	ruleId: number | null;
	ruleLabel: string | null;
	ruleMatchType: string | null;
	counterpartyAccountId: number | null;
	counterpartyAccountName: string | null;
	score: MatchScore;
}

/**
 * Story 30-1b (#686, AC-b5) — pourquoi une ligne ne propose rien. Un seul code : une
 * facture fournisseur ouverte du montant exact du débit est dans un lot de paiement en cours
 * sur ce compte.
 */
export interface ProposalNotice {
	code: 'supplier_invoice_in_payment_batch';
	supplierInvoiceId: number;
	supplierInvoiceNumber: string | null;
	paymentBatchId: number;
}

export interface ReconciliationProposal {
	bankTransactionId: number;
	transaction: TransactionSummary;
	candidates: ReconciliationCandidate[];
	/** Story 30-1b — posée, elle va de pair avec `candidates` vide. */
	notice: ProposalNotice | null;
}

export interface GetProposalsResponse {
	proposals: ReconciliationProposal[];
	// H6 Pass 1 code review — pagination indicator. true si la query
	// SQL a renvoyé `limit + 1` lignes côté backend.
	// Story 29-9 (#649) — l'écran le lit : sous la liste, un avis dit qu'il en
	// reste et un bouton demande le plafond (`PROPOSALS_MAX_LIMIT`, 500) ; au
	// plafond, l'avis reste, sans bouton. Pas de pagination au-delà.
	hasMore: boolean;
}

// Story 8-5a-bis Q2 — breaking change : discriminated union sur `type`.
// 'invoice' (8-4 héritée) et 'split' (8-5a-bis FR48). 'manual' réservé v0.2,
// 'rule' réservé 8-5b.
export type AcceptProposalInput =
	| { type: 'invoice'; bankTransactionId: number; invoiceId: number }
	| {
			type: 'split';
			bankTransactionId: number;
			splits: SplitProposalLine[];
			valueDate?: string;
	  }
	| {
			type: 'rule';
			bankTransactionId: number;
			ruleId: number;
			counterpartyAccountId: number;
	  }
	// Story 30-1b (#686) — régler une facture fournisseur par un débit.
	| { type: 'supplier_invoice'; bankTransactionId: number; supplierInvoiceId: number };

export interface SplitProposalLine {
	counterpartyAccountId: number;
	amount: string;
	description: string;
	/** Projet analytique de cette ligne de ventilation (Story 19-5). */
	projectId?: number | null;
}

export interface AcceptedProposal {
	bankTransactionId: number;
	/** `0` hors d'une facture client. */
	invoiceId: number;
	/** Story 30-1b — la facture fournisseur réglée ; `null` sinon. */
	supplierInvoiceId: number | null;
	journalEntryId: number;
	score: MatchScore;
}

export interface FailedProposal {
	bankTransactionId: number;
	errorCode: string;
	details: unknown | null;
}

export interface AcceptResponse {
	accepted: AcceptedProposal[];
	failed: FailedProposal[];
}

export interface RejectedProposal {
	bankTransactionId: number;
	rejectedAt: string;
}

export interface RejectResponse {
	rejected: RejectedProposal[];
	failed: FailedProposal[];
}

// Story 8-5a-base FR45 — réconciliation manuelle.
export interface ManualMatchResponse {
	bankTransactionId: number;
	journalEntryId: number;
}

// Story 8-5a-bis FR48 — éclatement de transaction (split).
export interface SplitResponse {
	bankTransactionId: number;
	journalEntryId: number;
}

// Story 25-3-b (#418) — annuler un rapprochement.

/** Les huit motifs qui refusent l'annulation, dans l'ordre de précédence du serveur. */
export type ReconciliationCancelCode =
	| 'BANK_TRANSACTION_NOT_RECONCILED'
	| 'INVOICE_CREDITED'
	// Story 25-4-d2a (#384) : un solde existe sur la facture.
	| 'INVOICE_WRITTEN_OFF'
	| 'FISCAL_YEAR_CLOSED'
	// Story 15-1a2-0 (#518) : rang 2 bis, le lettrage de pièce figé par la période.
	| 'LETTERING_ALL_LINES_IN_CLOSED_PERIODS'
	| 'MATCHED_BANK_TRANSACTION'
	| 'ACCOUNT_ARCHIVED'
	| 'FISCAL_YEAR_INVALID';

/** `GET /api/v1/reconciliation/transactions/{id}` — lu au clic, pour une transaction. */
export interface ReconciliationTransactionResponse {
	id: number;
	status: string;
	amount: string;
	currency: string;
	bookingDate: string;
	matchedEntryId: number | null;
	/**
	 * `invoice_settlement`, `supplier_settlement` (Story 30-1a, un règlement fournisseur) ou
	 * `entry` ; `null` si la transaction n'est pas rapprochée.
	 */
	kind: 'invoice_settlement' | 'supplier_settlement' | 'entry' | null;
	invoiceId: number | null;
	invoiceNumber: string | null;
	/** Story 30-1a — la facture fournisseur réglée, si `kind = supplier_settlement`. */
	supplierInvoiceId: number | null;
	supplierInvoiceNumber: string | null;
	cancellable: boolean;
	cancelBlockedBy: ReconciliationCancelCode | null;
	/** Numéro du compte archivé (rang 4). */
	cancelBlockedLabel: string | null;
	/** L'AUTRE transaction qui pointe la même écriture (rang 3). */
	cancelBlockedDocumentId: number | null;
}

/** `POST /api/v1/reconciliation/transactions/{id}/cancel`. */
export interface CancelReconciliationResponse {
	bankTransaction: { id: number; status: string; matchedEntryId: number | null };
	reversalJournalEntryId: number;
	invoiceId: number | null;
	/** Story 30-1a — la facture fournisseur dont le paiement a été retiré. */
	supplierInvoiceId: number | null;
}

// Story 29-8 (#670) — rattacher une transaction au règlement saisi depuis la fiche.

/** Un règlement client par virement auquel la transaction se rattache. */
export interface LinkCandidate {
	settlementId: number;
	invoiceId: number;
	invoiceNumber: string | null;
	contactName: string;
	settledOn: string;
	/** Le mouvement du règlement sur le compte de la banque, à deux décimales. */
	amount: string;
	/** `transaction − règlement`, à deux décimales, entre 0.00 et 0.05. */
	gap: string;
	/** Jours entre le règlement et le paiement (toujours positif). */
	daysApart: number;
	journalEntryId: number;
}

/** `GET /api/v1/reconciliation/transactions/{id}/link-candidates`. */
export interface LinkCandidatesResponse {
	bankTransactionId: number;
	candidates: LinkCandidate[];
}

/** `POST /api/v1/reconciliation/transactions/{id}/link`. */
export interface LinkResponse {
	bankTransactionId: number;
	journalEntryId: number;
	invoiceId: number;
	settlementId: number;
}

// Story v017-3a (#671) — rattacher une transaction à une écriture déjà passée.

/** Une écriture déjà passée à laquelle la transaction se rattache. */
export interface EntryCandidate {
	journalEntryId: number;
	entryNumber: number;
	fiscalYearId: number;
	entryDate: string;
	/** Le code du journal (`Achats`, `Ventes`, `Banque`, `Caisse`, `OD`). */
	journal: string;
	description: string;
	/** Le mouvement de l'écriture sur le compte de la banque, signé, à deux décimales. */
	amount: string;
	/** Jours entre l'écriture et le paiement (toujours positif). */
	daysApart: number;
	/**
	 * Le motif qui, AU MOMENT DE LA LECTURE, empêcherait d'annuler ce rattachement — un
	 * instantané —, ou `null`. Les codes fournisseur de tête ne sortent pas pour une candidate.
	 */
	cancelBlockedBy: ReconciliationCancelCode | null;
	/** Numéro du compte archivé (rang 4). */
	cancelBlockedLabel: string | null;
}

/** `GET /api/v1/reconciliation/transactions/{id}/entry-candidates`. */
export interface EntryCandidatesResponse {
	bankTransactionId: number;
	candidates: EntryCandidate[];
}

/** `POST /api/v1/reconciliation/transactions/{id}/link-entry`. */
export interface LinkEntryResponse {
	bankTransactionId: number;
	journalEntryId: number;
}
