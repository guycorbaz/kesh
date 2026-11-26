/**
 * Story 40-7d (AC-2, AC-3) — DTO **communs** aux pages des prestataires
 * (PayPal, Stripe), miroir de la revue commune de `kesh-reconciliation::
 * provider_review` (38-11c2) : ce que la revue montre d'une transaction et
 * d'une écriture, les écarts et lignes « à examiner », la réponse de
 * l'acceptation (*Pattern batch*) et le compte rendu d'une devise de l'import.
 * Montants en chaînes décimales. Les propositions, propres à chaque
 * prestataire, se déclarent dans sa feature (`paypal.types.ts`,
 * `stripe.types.ts`) sur {@link ProviderProposalBase}.
 */

/** Ce que la revue montre d'une transaction (`TransactionSummary`). */
export interface ProviderReviewTransaction {
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
export interface ProviderReviewEntry {
	journalEntryId: number;
	entryNumber: number;
	entryDate: string;
	description: string;
	/** Identifiants tirés du libellé (« TXN … », `ch_…`). */
	ids: string[];
}

/** Ce que toute proposition porte : son genre, sa transaction, et ce que la revue y ajoute. */
export interface ProviderProposalBase {
	type: string;
	transaction: ProviderReviewTransaction;
	otherTransaction?: ProviderReviewTransaction;
	entry?: ProviderReviewEntry;
}

/** Un écart ou une ligne « à examiner » (`ReviewFinding`). */
export interface ProviderFinding {
	/** Le motif : genre d'écart ou motif « à examiner ». */
	code: string;
	/** La transaction (absente d'une vente sans encaissement). */
	transaction?: ProviderReviewTransaction;
	/** Les écritures en cause. */
	entries?: ProviderReviewEntry[];
	/** Détails propres au motif. */
	details?: Record<string, unknown> | null;
}

/** Réponse de `GET /api/v1/<prestataire>/review`. */
export interface ProviderReview<P extends ProviderProposalBase = ProviderProposalBase> {
	account: string;
	fiscalYearId: number;
	proposals: P[];
	gaps: ProviderFinding[];
	toReview: ProviderFinding[];
	counts?: Record<string, number>;
}

/** Un refus par proposition (*Pattern batch*). */
export interface ProviderFailedProposal {
	bankTransactionId: number;
	errorCode: string;
	details: unknown;
}

/** Réponse de `POST /api/v1/<prestataire>/accept`. */
export interface ProviderAcceptResponse {
	accepted: { type: string; bankTransactionIds: number[]; journalEntryIds: number[] }[];
	failed: ProviderFailedProposal[];
}

/** Le compte rendu d'une devise de l'import d'un prestataire. */
export interface ProviderCurrencyReport {
	currency: string;
	bankAccountId: number;
	/** `null` quand toutes les lignes de la devise étaient déjà connues. */
	importId: number | null;
	/** Transactions insérées (une ligne à frais en compte deux). */
	inserted: number;
	duplicates: number;
}

/**
 * Ce que tout compte rendu d'import porte : le compte et, pour PayPal et
 * Stripe, ses devises. Story 39-11d : Wise rend un compte rendu **par fichier**
 * (`files`) et non par devise ; sa page le montre par `importReportMain`.
 */
export interface ProviderImportResponseBase {
	account: string;
	currencies?: ProviderCurrencyReport[];
}

/** Une erreur de ligne d'un lecteur de prestataire (`details.lines[]` du refus du CSV). */
export interface ProviderLineError {
	line: number;
	field: string | null;
	code: string;
	value: string | null;
}

// ---------------------------------------------------------------------------
// La page commune (`ProviderReviewPage.svelte`)
// ---------------------------------------------------------------------------

/** Les textes propres au prestataire, déjà traduits par sa page. */
export interface ProviderPageTexts {
	title: string;
	subtitle: string;
	noAccount: string;
	accountLabel: string;
	importButton: string;
	/** Story 39-11d — la consigne d'export, sous le bouton d'import (facultative). */
	importHelp?: string;
	/** En-tête de la colonne de l'identifiant chez le prestataire (« TXN », « Identifiant »). */
	idColumn: string;
	/** Rappel affiché sous les sélecteurs, facultatif. */
	hint?: string;
}

/**
 * Le client d'API du prestataire. Story 39-11d : `importFiles` remplace
 * `importFile` quand la page prend plusieurs fichiers d'un geste (Wise) ; sans
 * `getReview` ni `accept`, la page n'a que l'import (pas d'exercice, pas de revue).
 */
export interface ProviderPageApi<P extends ProviderProposalBase, B, R> {
	importFile?(file: File, account: string): Promise<R>;
	importFiles?(files: File[], account: string): Promise<R>;
	getReview?(account: string, fiscalYearId: number): Promise<ProviderReview<P>>;
	accept?(account: string, proposals: B[]): Promise<ProviderAcceptResponse>;
}
