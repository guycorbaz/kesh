/**
 * Types de l'écran des postes ouverts (Story 15-1c-i, #518).
 *
 * Miroirs des DTO livrés par la 15-1b et la 15-1c-0
 * (`crates/kesh-api/src/routes/letterings.rs`) : `OpenItemsResponse`,
 * `ProposalsResponse`, `LetteringDetailResponse`. Les montants sont des
 * **chaînes décimales** (jamais des nombres JSON) : toute arithmétique passe par
 * `big.js`.
 */

/** Type de pièce, sérialisé par `DocumentKind::as_str()` côté serveur. */
export type DocumentType =
	| 'invoice'
	| 'creditNote'
	| 'supplierInvoice'
	| 'settlement'
	| 'bankTransaction';

/** La pièce qui possède l'écriture d'une ligne (`DocumentResponse`). */
export interface DocumentRef {
	type: DocumentType;
	id: number;
	number: string | null;
	/** Pour `settlement` : la facture réglée ; `null` sinon. */
	invoiceId: number | null;
	invoiceNumber: string | null;
}

/** Origine d'un groupe de lettrage (`letterings::Origin`). */
export type LetteringOrigin = 'document' | 'reversal' | 'manual';

/** Pourquoi une ligne est ouverte à `asOf` (15-1b AC4). */
export type OpenItemReason = 'unlettered' | 'letteredAfterAsOf';

/** Où en est la pièce de la ligne, **aujourd'hui** (15-1b AC4). */
export type DocumentState =
	| 'unpaid'
	| 'partiallySettled'
	| 'nothingDue'
	| 'paidWithoutSettlementEntry';

/** Une ligne ouverte à `asOf` (`OpenItemResponse`). */
export interface OpenItem {
	lineId: number;
	entryId: number;
	/** ⚠️ Repart à 1 à chaque exercice : se lit avec `fiscalYearName`. */
	entryNumber: number;
	fiscalYearName: string;
	date: string;
	journal: string;
	description: string;
	debit: string;
	credit: string;
	document: DocumentRef | null;
	letteringCode: string | null;
	/** Non lu par l'écran : l'origine s'affiche au panneau du groupe. */
	letteringOrigin: LetteringOrigin | null;
	letteredOn: string | null;
	reason: OpenItemReason;
	documentState: DocumentState | null;
	amountDue: string | null;
	manuallyLetterable: boolean;
	inOpenPeriod: boolean;
}

/** Réponse de `GET /accounts/{id}/open-items`. */
export interface OpenItemsResponse {
	accountId: number;
	accountNumber: string;
	asOf: string;
	/** Sens débit, `Σ(débit − crédit)` — sur tout l'ensemble, non sur la page. */
	balance: string;
	/** Sens débit — sur tout l'ensemble. */
	openTotal: string;
	total: number;
	offset: number;
	limit: number;
	items: OpenItem[];
}

/** Une ligne d'une paire proposée (`ProposalLineResponse`). */
export interface ProposalLine {
	lineId: number;
	entryId: number;
	entryNumber: number;
	fiscalYearName: string;
	date: string;
	journal: string;
	description: string;
	document: DocumentRef | null;
	inOpenPeriod: boolean;
}

/** Une paire proposée (`ProposalResponse`). */
export interface Proposal {
	amount: string;
	daysApart: number;
	reversalPair: boolean;
	debit: ProposalLine;
	credit: ProposalLine;
}

/** Réponse de `GET /accounts/{id}/lettering-proposals` — **pas d'`offset`**. */
export interface ProposalsResponse {
	accountId: number;
	candidateCount: number;
	total: number;
	limit: number;
	items: Proposal[];
}

/** Code de la prévision du délettrage (`ManualDissolutionBlocker::code()`). */
export type ManualDissolutionBlocker =
	| 'LETTERING_IS_DOCUMENT'
	| 'LETTERING_LINE_OWNED_BY_DOCUMENT'
	| 'LETTERING_ALL_LINES_IN_CLOSED_PERIODS';

/** Une ligne d'un groupe lu en détail (`LetteringDetailLineResponse`). */
export interface LetteringDetailLine {
	id: number;
	entryId: number;
	entryNumber: number;
	fiscalYearId: number;
	fiscalYearName: string;
	date: string;
	debit: string;
	credit: string;
	journal: string;
	description: string;
	document: DocumentRef | null;
	ownedByDocument: boolean;
	inOpenPeriod: boolean;
}

/** Réponse du `GET /letterings/{key}` (15-1c-0 AC15). */
export interface LetteringDetail {
	key: number;
	code: string;
	origin: LetteringOrigin;
	accountId: number;
	accountNumber: string;
	accountName: string;
	/** Indicatif (lu sans verrou) ; le `DELETE` fait autorité. */
	manualDissolutionBlockedBy: ManualDissolutionBlocker | null;
	lines: LetteringDetailLine[];
}

/** Réponse 201 du `POST /letterings` (forme de la 15-1a-i, non enrichie). */
export interface LetteringCreated {
	key: number;
	code: string;
	origin: LetteringOrigin;
	accountId: number;
}

/** Ce que la sélection retient d'une ligne, pour survivre au changement de page. */
export interface SelectedLine {
	lineId: number;
	debit: string;
	credit: string;
	inOpenPeriod: boolean;
}

/** L'état du panneau des propositions — chargé et échoué **à part** de la liste (AC5). */
export type ProposalsState =
	| { status: 'loading' }
	| { status: 'error'; message: string }
	| { status: 'ready'; data: ProposalsResponse };

/** L'état du panneau d'un groupe — affiché **indépendamment** de la liste (AC6). */
export type GroupState =
	| { status: 'loading'; code: string }
	| { status: 'notFound'; code: string }
	| { status: 'error'; code: string; message: string }
	| { status: 'ready'; data: LetteringDetail };
