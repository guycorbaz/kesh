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
// sont Some/None selon ce type.
export type CandidateType = 'invoice' | 'rule';

export interface ReconciliationCandidate {
	candidateType: CandidateType;
	// Invoice candidate fields
	invoiceId: number | null;
	invoiceNumber: string | null;
	invoiceAmount: string | null;
	invoiceDate: string | null;
	// Story 8-5b — Rule candidate fields
	ruleId: number | null;
	ruleLabel: string | null;
	ruleMatchType: string | null;
	counterpartyAccountId: number | null;
	counterpartyAccountName: string | null;
	score: MatchScore;
}

export interface ReconciliationProposal {
	bankTransactionId: number;
	transaction: TransactionSummary;
	candidates: ReconciliationCandidate[];
}

export interface GetProposalsResponse {
	proposals: ReconciliationProposal[];
	// H6 Pass 1 code review — pagination indicator. true si la query
	// SQL a renvoyé `limit + 1` lignes côté backend. v0.1 : structure
	// présente mais pas de UI dédiée (bouton « Charger plus » v0.2).
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
	  };

export interface SplitProposalLine {
	counterpartyAccountId: number;
	amount: string;
	description: string;
	/** Projet analytique de cette ligne de ventilation (Story 19-5). */
	projectId?: number | null;
}

export interface AcceptedProposal {
	bankTransactionId: number;
	invoiceId: number;
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

/** Les six motifs qui refusent l'annulation, dans l'ordre de précédence du serveur. */
export type ReconciliationCancelCode =
	| 'BANK_TRANSACTION_NOT_RECONCILED'
	| 'INVOICE_CREDITED'
	| 'FISCAL_YEAR_CLOSED'
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
	/** `invoice_settlement` ou `entry` ; `null` si la transaction n'est pas rapprochée. */
	kind: 'invoice_settlement' | 'entry' | null;
	invoiceId: number | null;
	invoiceNumber: string | null;
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
}
