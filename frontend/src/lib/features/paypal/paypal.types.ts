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
// Revue et acceptation (38-11c2, AC-5 à AC-9)
// ---------------------------------------------------------------------------

/** La transaction d'un élément de la revue. */
export interface PaypalReviewTransaction {
	bankTransactionId: number;
	date: string;
	amount: string;
	currency: string;
	providerType: string | null;
	name: string | null;
	counterpartyEmail: string | null;
	/** Identifiant PayPal (`transaction_id`, suffixe `:FEE` des frais compris). */
	transactionId: string | null;
}

/** Une proposition de la revue, selon `type`. */
export type PaypalProposal =
	| (PaypalReviewTransaction & {
			type: 'link_entry';
			journalEntryId: number;
			entryNumber: number | null;
	  })
	| (PaypalReviewTransaction & { type: 'fee'; feeAccountId: number })
	| (PaypalReviewTransaction & {
			type: 'rule';
			ruleId: number;
			ruleLabel: string | null;
			counterpartyAccountId: number;
	  })
	| (PaypalReviewTransaction & {
			type: 'conversion';
			debitTransactionId: number;
			creditTransactionId: number;
			/** L'autre côté de la conversion (montant et devise). */
			otherAmount: string | null;
			otherCurrency: string | null;
	  })
	| (PaypalReviewTransaction & {
			type: 'withdrawal';
			/** Préremplie par le compte du plan du compte bancaire principal ; `null` = à choisir. */
			counterpartyAccountId: number | null;
	  });

export type PaypalProposalType = PaypalProposal['type'];

/** Un écart de la revue. */
export interface PaypalGap extends Partial<PaypalReviewTransaction> {
	kind: 'receipt_without_sale' | 'sale_without_receipt' | 'amount_or_currency_mismatch' | string;
	journalEntryId?: number | null;
	entryNumber?: number | null;
	entryDate?: string | null;
	saleTransactionId?: string | null;
	details?: Record<string, unknown> | null;
}

/** Une ligne « à examiner ». */
export interface PaypalToReview extends PaypalReviewTransaction {
	reason: string;
	journalEntryIds?: number[];
}

/** Réponse de `GET /api/v1/paypal/review`. */
export interface PaypalReview {
	account: string;
	fiscalYearId: number;
	proposals: PaypalProposal[];
	gaps: PaypalGap[];
	toReview: PaypalToReview[];
	counts?: Record<string, number>;
}

/** Une proposition envoyée à `POST /api/v1/paypal/accept`. */
export type PaypalAcceptProposal =
	| { type: 'link_entry'; bankTransactionId: number; journalEntryId: number }
	| { type: 'fee'; bankTransactionId: number; feeAccountId: number }
	| { type: 'rule'; bankTransactionId: number; ruleId: number; counterpartyAccountId: number }
	| { type: 'conversion'; debitTransactionId: number; creditTransactionId: number }
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
