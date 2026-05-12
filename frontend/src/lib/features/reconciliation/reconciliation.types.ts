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

export interface ReconciliationCandidate {
	invoiceId: number;
	invoiceNumber: string | null;
	invoiceAmount: string;
	invoiceDate: string;
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
	  };

export interface SplitProposalLine {
	counterpartyAccountId: number;
	amount: string;
	description: string;
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
