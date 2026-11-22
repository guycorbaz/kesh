/**
 * Story 38-11d (AC-3 à AC-5) — DTO de l'écran PayPal, miroir des routes
 * `POST /api/v1/paypal-imports` (38-11b), `GET /api/v1/paypal/review` et
 * `POST /api/v1/paypal/accept` (38-11c2). Montants en chaînes décimales.
 */

// ---------------------------------------------------------------------------
// Import (38-11b, `routes/paypal_imports.rs`)
// ---------------------------------------------------------------------------

/** Une ligne écartée parce que son statut n'est pas `Completed`. */
export interface PaypalNotCompletedLine {
	transactionId: string;
	type: string;
	status: string;
}

/** Le compte rendu d'une devise de l'import. */
export interface PaypalCurrencyReport {
	currency: string;
	bankAccountId: number;
	/** `null` quand toutes les lignes de la devise étaient déjà connues. */
	importId: number | null;
	/** Transactions insérées (une ligne à frais en compte deux). */
	inserted: number;
	duplicates: number;
}

/** Réponse `201` de `POST /api/v1/paypal-imports`. */
export interface PaypalImportResponse {
	account: string;
	currencies: PaypalCurrencyReport[];
	skipped: {
		notCompleted: PaypalNotCompletedLine[];
		memo: number;
		neutralPairs: number;
	};
}

/** Une erreur de ligne du lecteur PayPal (`details.lines[]` de `PAYPAL_CSV_INVALID`). */
export interface PaypalLineError {
	line: number;
	field: string | null;
	code: string;
	value: string | null;
}

// ---------------------------------------------------------------------------
// Revue et acceptation (38-11c2, AC-5 à AC-9 ; `kesh-reconciliation::provider_review`)
// ---------------------------------------------------------------------------

/** Ce que la revue montre d'une transaction (`TransactionSummary`). */
export interface PaypalReviewTransaction {
	id: number;
	bankAccountId: number;
	/** Date de paiement (`value_date`, à défaut `booking_date`), `AAAA-MM-JJ`. */
	date: string;
	/** Montant signé, deux décimales. */
	amount: string;
	currency: string;
	providerType: string | null;
	name: string | null;
	/** Identifiant chez le prestataire (suffixe `:FEE` des frais compris). */
	transactionId: string | null;
	/** E-mail de la contrepartie (importé du CSV ; revue P1 de la 38-11d, M5). */
	counterpartyEmail: string | null;
}

/** Une écriture que nomme la revue (`EntrySummary`). */
export interface PaypalReviewEntry {
	journalEntryId: number;
	entryNumber: number;
	entryDate: string;
	description: string;
	/** Identifiants tirés du libellé (« TXN … »). */
	ids: string[];
}

/** Les champs d'une proposition, selon `type` (`Proposal`, sérialisé aplati). */
export type PaypalProposalFields =
	| { type: 'link_entry'; bankTransactionId: number; journalEntryId: number }
	| { type: 'fee'; bankTransactionId: number; feeAccountId: number }
	| { type: 'rule'; bankTransactionId: number; ruleId: number; counterpartyAccountId: number }
	| { type: 'conversion'; debitTransactionId: number; creditTransactionId: number }
	| {
			type: 'withdrawal';
			bankTransactionId: number;
			/** Préremplie par le compte du plan du compte bancaire principal ; `null` = à choisir. */
			counterpartyAccountId: number | null;
	  };

/** Une proposition de la revue : ses champs, la transaction (la débitée d'une conversion), l'autre moitié d'une conversion, l'écriture d'un rattachement. */
export type PaypalProposal = PaypalProposalFields & {
	transaction: PaypalReviewTransaction;
	otherTransaction?: PaypalReviewTransaction;
	entry?: PaypalReviewEntry;
};

export type PaypalProposalType = PaypalProposal['type'];

/** Un écart ou une ligne « à examiner » (`ReviewFinding`). */
export interface PaypalFinding {
	/** Le motif : genre d'écart ou motif « à examiner ». */
	code: string;
	/** La transaction (absente d'une vente sans encaissement). */
	transaction?: PaypalReviewTransaction;
	/** Les écritures en cause. */
	entries?: PaypalReviewEntry[];
	/** Détails propres au motif (attendu et trouvé d'un montant ou d'une devise différents). */
	details?: Record<string, unknown> | null;
}

/** Réponse de `GET /api/v1/paypal/review`. */
export interface PaypalReview {
	account: string;
	fiscalYearId: number;
	proposals: PaypalProposal[];
	gaps: PaypalFinding[];
	toReview: PaypalFinding[];
	counts?: Record<string, number>;
}

/** Une proposition envoyée à `POST /api/v1/paypal/accept` (ses seuls champs ; `withdrawal` avec son compte). */
export type PaypalAcceptProposal =
	| Exclude<PaypalProposalFields, { type: 'withdrawal' }>
	| { type: 'withdrawal'; bankTransactionId: number; counterpartyAccountId: number };

/** Un refus par proposition (*Pattern batch*). */
export interface PaypalFailedProposal {
	bankTransactionId: number;
	errorCode: string;
	details: unknown;
}

/** Réponse de `POST /api/v1/paypal/accept`. */
export interface PaypalAcceptResponse {
	accepted: { type: string; bankTransactionIds: number[]; journalEntryIds: number[] }[];
	failed: PaypalFailedProposal[];
}
