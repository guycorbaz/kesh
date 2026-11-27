/**
 * Story 39-11d (AC-2) — DTO de l'écran Wise, miroir de la réponse `201` de
 * `POST /api/v1/wise-imports` (39-11b ; zip de Wise, 39-11e) ; Story 39-11d2
 * (AC-2) — et de la revue et de l'acceptation (`GET /api/v1/wise/review`,
 * `POST /api/v1/wise/accept`, 39-11c). À la différence
 * de PayPal et de Stripe, le compte rendu se tient **par fichier** (Wise exporte
 * un relevé par solde, donc par devise), avec les soldes du relevé, son
 * avertissement de continuité et, pour un zip, l'archive d'où il vient.
 * Montants en chaînes décimales.
 */
import type {
	ProviderImportResponseBase,
	ProviderReview,
	ProviderReviewEntry,
	ProviderReviewTransaction,
} from '$lib/features/provider/provider-review.types';

/**
 * Avertissement d'un fichier (39-11b, AC-5) : `gap`, le relevé ne commence pas
 * où finissait le précédent du compte (`expected` = sa clôture, `found` =
 * l'ouverture du fichier). L'import passe.
 */
export interface WiseFileWarning {
	code: 'gap' | string;
	expected: string;
	found: string;
	previousImportId: number;
}

/** Le compte rendu d'un fichier importé. */
export interface WiseFileReport {
	/** Nom du fichier (pour un zip : le nom de l'entrée CSV). */
	filename: string;
	/** Le zip d'où vient le fichier, `null` pour un fichier envoyé à part (39-11e). */
	archive: string | null;
	currency: string;
	bankAccountId: number;
	/** `null` quand toutes les lignes du fichier étaient déjà connues. */
	importId: number | null;
	/** Lignes lues (`inserted + duplicates`). */
	rows: number;
	inserted: number;
	duplicates: number;
	openingBalance: string;
	closingBalance: string;
	warnings: WiseFileWarning[];
}

/** Une entrée d'un zip que l'import n'a pas lue (dossier, autre extension : 39-11e, D3). */
export interface WiseIgnoredEntry {
	archive: string;
	entry: string;
}

/** Réponse `201` de `POST /api/v1/wise-imports`. */
export interface WiseImportResponse extends ProviderImportResponseBase {
	files: WiseFileReport[];
	ignoredEntries: WiseIgnoredEntry[];
}

/**
 * Story 39-11d2 (AC-2) — les champs d'une proposition de la revue Wise, selon
 * `type` (`WiseReviewProposal`, sérialisée à plat) : les genres communs, et le
 * paiement par carte groupé (`card_group`), avec ses lignes (`lines`, la
 * principale en tête, dans l'ordre de `bankTransactionIds`) et le montant du
 * marchand en CHF (`merchantAmount`), champs d'affichage seuls.
 */
export type WiseProposalFields =
	| { type: 'fee'; bankTransactionId: number; feeAccountId: number }
	| { type: 'conversion'; debitTransactionId: number; creditTransactionId: number }
	| { type: 'link_entry'; bankTransactionId: number; journalEntryId: number }
	| { type: 'rule'; bankTransactionId: number; ruleId: number; counterpartyAccountId: number }
	| {
			type: 'card_group';
			bankTransactionIds: number[];
			ruleId: number;
			counterpartyAccountId: number;
			lines: ProviderReviewTransaction[];
			merchantAmount: string;
	  };

/** Une proposition de la revue Wise : ses champs, sa transaction (la débitée d'une conversion, la principale d'un groupe). */
export type WiseProposal = WiseProposalFields & {
	transaction: ProviderReviewTransaction;
	otherTransaction?: ProviderReviewTransaction;
	entry?: ProviderReviewEntry;
};

/** Réponse de `GET /api/v1/wise/review`. */
export type WiseReview = ProviderReview<WiseProposal>;

/**
 * Une proposition envoyée à `POST /api/v1/wise/accept` : les champs du corps
 * seuls (un groupe sans `lines` ni `merchantAmount`).
 */
export type WiseAcceptProposal =
	| Exclude<WiseProposalFields, { type: 'card_group' }>
	| { type: 'card_group'; bankTransactionIds: number[]; ruleId: number; counterpartyAccountId: number };
