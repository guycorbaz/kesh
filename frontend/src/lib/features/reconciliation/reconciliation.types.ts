// Story 8-4 — Types DTOs réconciliation (miroir kesh-api/routes/reconciliation.rs).

export interface MatchScore {
	total: number;
	amountScore: number;
	referenceScore: number;
	contactScore: number;
}

export interface TransactionSummary {
	bookingDate: string;
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

export interface AcceptProposalInput {
	bankTransactionId: number;
	invoiceId: number;
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
