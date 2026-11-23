// Story 8-5a-zero — Client API pour les bank_accounts (foundation
// `journal_account_id` link). Aucun appel d'écriture manuelle ici.
//
// Story v014-1 — étendu avec CRUD complet post-onboarding :
// - `createBankAccount`, `updateBankAccount`, `archiveBankAccount`.
// - `listBankAccounts(includeArchived?)` avec param optionnel pour toggle UI.
// - `BankAccountSummary` étendu avec `currentBalance` et `archived: boolean`.
//
// Story 25-6-a (#389) — les montants restent des chaînes décimales (`big.js`
// pour tout calcul) ; trois champs du relevé CAMT.053 portant un solde de
// clôture dont la période se termine le plus tard (`period_to`) (et non du dernier importé).
//
// Story 28-4b (#642) — clôture d'un compte bancaire : `closedOn`, second booléen
// `includeClosed` de `listBankAccounts` (C-28-4-20), `closeBankAccount` et
// `reopenBankAccount`.

import { apiClient } from '$lib/shared/utils/api-client';
import type { ProviderApiStatus } from '$lib/features/provider-download/provider-download.api';

/** Story 34-11a1 (#972, C-e341-13) — type d'un compte : bancaire (IBAN) ou carte (sans IBAN). */
export type BankAccountKind = 'bank' | 'credit_card';

/**
 * Story 38-11d (AC-1) — prestataire de paiement dont un compte bancaire est le
 * solde dans une devise (38-11b) ; `null` pour un compte ordinaire. Wise et
 * Stripe (Epics 39, 40) prennent les deux autres valeurs.
 */
export type BankAccountProvider = 'paypal' | 'wise' | 'stripe';

/**
 * Story 38-11d (AC-1) — paramètres PayPal d'un compte bancaire (38-11b,
 * AC-2) : le libellé du compte PayPal (1 à 120 caractères, « PayPal www ») et
 * le compte de charge des frais (`null` : les frais vont « à examiner »).
 */
export interface PaypalSettings {
	account: string;
	feeAccountId: number | null;
}

/**
 * Story 40-7d (AC-1) — paramètres Stripe d'un compte bancaire (40-7b, AC-2) :
 * ceux de PayPal, plus le **compte bancaire de versement** (`null` : les
 * versements vont « à examiner »), de même devise que le solde, jamais un
 * compte Stripe ni PayPal.
 */
export interface StripeSettings extends PaypalSettings {
	payoutBankAccountId: number | null;
}

/**
 * Story 40-7d (AC-1) — les paramètres d'un compte de prestataire tels que le
 * corps de `POST` ou `PUT /bank-accounts` les porte : **un** prestataire, sous
 * sa clé.
 */
export type ProviderSettingsBody =
	| { paypal: PaypalSettings; stripe?: never }
	| { stripe: StripeSettings; paypal?: never };

export interface BankAccountSummary {
	id: number;
	bankName: string;
	/**
	 * Story 26-2a (#622) — description optionnelle (« Personnel »), `null` si
	 * absente. **Obligatoire** dans le type : `npm run check` signale tout objet
	 * qui l'oublierait. S'affiche par `bankAccountName` (`format.ts`).
	 */
	description: string | null;
	/**
	 * Story 34-11a1 (#972) — `null` pour un compte **sans IBAN** (carte de
	 * crédit). Nullable dans le type : `npm run check` signale tout lecteur qui
	 * le traiterait comme une chaîne. S'affiche par `bankAccountShortId` ou
	 * `bankAccountOptionLabel` (`format.ts`).
	 */
	iban: string | null;
	qrIban: string | null;
	/** Story 34-11a1 (#972) — type du compte, déclaré par le serveur. */
	accountKind: BankAccountKind;
	/** Story 34-11a1 (#972) — quatre derniers chiffres d'une carte, `null` sinon. */
	cardLast4: string | null;
	/**
	 * Story 38-11d (AC-1) — prestataire dont ce compte est le solde, `null` pour
	 * un compte ordinaire (une réponse sans le champ en est un).
	 */
	provider: BankAccountProvider | null;
	/** Story 38-11d (AC-1) — paramètres PayPal, `null` hors d'un compte PayPal. */
	paypal: PaypalSettings | null;
	/** Story 40-7d (AC-1) — paramètres Stripe, `null` hors d'un compte Stripe. */
	stripe: StripeSettings | null;
	isPrimary: boolean;
	journalAccountId: number | null;
	version: number;
	/** Story v014-1 — true si soft-deleted (archivé). */
	archived: boolean;
	/**
	 * Story 28-4b (#642) — date de clôture `AAAA-MM-JJ` d'un compte **clos** (en
	 * lecture seule, historique conservé, réversible), `null` s'il est ouvert. La
	 * date est informative : c'est l'état qui garde (C-28-4-3).
	 */
	closedOn: string | null;
	/**
	 * Story v014-1 T5 — **solde comptable** calculé serveur-side depuis
	 * journal_entry_lines (pas le solde bancaire). `null` si `journal_account_id`
	 * n'est pas configuré sur ce compte.
	 *
	 * Story 25-6-a (#389) — reste une **chaîne décimale** (jamais un `Number`) :
	 * tout calcul passe par `big.js`, tout affichage par `formatChfBalance`.
	 */
	currentBalance: string | null;
	/**
	 * Story v014-1 F13 Pass 1 code review (AC#30) — date `MAX(je.entry_date)`
	 * agrégée sur le `journal_account_id` lié. Format ISO `YYYY-MM-DD`.
	 * `null` si journal_account_id NULL ou aucune écriture.
	 */
	lastTransactionDate: string | null;
	/** Story 25-6-a — solde de clôture du relevé dont la période se termine le plus tard (chaîne décimale). */
	statementClosingBalance: string | null;
	/** Story 25-6-a — date de ce relevé (`YYYY-MM-DD`). */
	statementDate: string | null;
	/**
	 * Story 25-6-a — solde comptable à `statementDate`, corrigé des dates de
	 * valeur. `null` sans relevé, sans compte lié, ou si plusieurs comptes
	 * bancaires partagent le compte de grand livre.
	 */
	ledgerBalanceAtStatement: string | null;
	/**
	 * Story 29-1a (#232) — écart livres − relevé, **calculé par le serveur**
	 * (`bank_accounts::statement_gap`, au centime, demi loin de zéro) : chaîne
	 * décimale, `null` s'il est nul ou ne se calcule pas. Seule définition de
	 * l'écart ; le frontend ne le recalcule pas.
	 */
	statementGap: string | null;
	/**
	 * Story 38-12b (AC-1) — pour un compte de prestataire, le téléchargement par
	 * l'API est-il configuré (série d'identifiants au libellé du compte) ? `null`
	 * ou absent pour un compte ordinaire. Jamais un secret.
	 */
	providerApi?: ProviderApiStatus | null;
}

/**
 * Type backend raw : les montants sont sérialisés en chaînes par serde (Decimal
 * feature `serde-str`) et le restent.
 */
interface BankAccountSummaryRaw {
	id: number;
	bankName: string;
	description: string | null;
	iban: string | null;
	qrIban: string | null;
	// Story 34-11a1 — absents d'un serveur antérieur à la 0.17.6 : `bank`, `null`.
	accountKind?: BankAccountKind;
	cardLast4?: string | null;
	// Story 38-11d — absents d'un serveur antérieur à la 38-11b : `null`.
	provider?: BankAccountProvider | null;
	paypal?: { account: string; feeAccountId?: number | null } | null;
	// Story 40-7d — absent d'un serveur antérieur à la 40-7b : `null`.
	stripe?: { account: string; feeAccountId?: number | null; payoutBankAccountId?: number | null } | null;
	isPrimary: boolean;
	journalAccountId: number | null;
	version: number;
	archived: boolean;
	// Story 28-4b — absent d'un serveur antérieur à la 0.13.3 : rendu `null`.
	closedOn?: string | null;
	currentBalance: string | null;
	lastTransactionDate: string | null;
	// Absents des réponses de mutation (POST/PUT/PATCH/DELETE rendent un compte nu).
	statementClosingBalance?: string | null;
	statementDate?: string | null;
	ledgerBalanceAtStatement?: string | null;
	statementGap?: string | null;
	// Story 38-12b — liste seulement ; absent d'un serveur antérieur et des réponses d'un compte.
	providerApi?: ProviderApiStatus | null;
}

function parseBankAccount(raw: BankAccountSummaryRaw): BankAccountSummary {
	return {
		id: raw.id,
		bankName: raw.bankName,
		description: raw.description ?? null,
		iban: raw.iban ?? null,
		qrIban: raw.qrIban,
		accountKind: raw.accountKind ?? 'bank',
		cardLast4: raw.cardLast4 ?? null,
		provider: raw.provider ?? null,
		paypal: raw.paypal
			? { account: raw.paypal.account, feeAccountId: raw.paypal.feeAccountId ?? null }
			: null,
		stripe: raw.stripe
			? {
					account: raw.stripe.account,
					feeAccountId: raw.stripe.feeAccountId ?? null,
					payoutBankAccountId: raw.stripe.payoutBankAccountId ?? null,
				}
			: null,
		isPrimary: raw.isPrimary,
		journalAccountId: raw.journalAccountId,
		version: raw.version,
		archived: raw.archived,
		closedOn: raw.closedOn ?? null,
		currentBalance: raw.currentBalance ?? null,
		lastTransactionDate: raw.lastTransactionDate,
		statementClosingBalance: raw.statementClosingBalance ?? null,
		statementDate: raw.statementDate ?? null,
		ledgerBalanceAtStatement: raw.ledgerBalanceAtStatement ?? null,
		statementGap: raw.statementGap ?? null,
		providerApi: raw.providerApi ?? null,
	};
}

/**
 * GET /api/v1/bank-accounts — liste les bank_accounts de la company
 * courante (multi-tenant scoping serveur-side via JWT).
 *
 * Par défaut, ni archivés ni clos : c'est la liste des comptes où l'on peut
 * **écrire** (sélecteurs d'import, de règlement, de lot ; tuile de l'accueil).
 *
 * @param includeArchived Si `true`, inclut aussi les comptes archivés (toggle UI).
 * @param includeClosed Story 28-4b (#642, C-28-4-20) — si `true`, inclut aussi
 *   les comptes clos (bascule de l'écran *Comptes bancaires* ; réglages de
 *   facturation, alignés sur le serveur qui compte un compte clos comme lié à
 *   son compte comptable, C-28-4-7). Second booléen et non objet d'options :
 *   les appelants existants (`listBankAccounts()`, `listBankAccounts(false)`)
 *   gardent leur sens et leur code.
 */
export async function listBankAccounts(
	includeArchived = false,
	includeClosed = false,
): Promise<BankAccountSummary[]> {
	const params = new URLSearchParams();
	if (includeArchived) params.set('includeArchived', 'true');
	if (includeClosed) params.set('includeClosed', 'true');
	const query = params.toString();
	const path = query === '' ? '/api/v1/bank-accounts' : `/api/v1/bank-accounts?${query}`;
	const raw = await apiClient.get<BankAccountSummaryRaw[]>(path);
	return raw.map(parseBankAccount);
}

/**
 * POST /api/v1/bank-accounts/{id}/close — **clôt** un compte bancaire
 * (Story 28-4a/28-4b, #642 ; Comptable+). Le compte passe en lecture seule :
 * plus rien ne s'y enregistre, son historique reste lisible ; réversible par
 * {@link reopenBankAccount}.
 *
 * Refus (message traduit par le serveur, à afficher par `errorMessageOf`) :
 * `412 BANK_ACCOUNT_CANNOT_CLOSE_PRIMARY`, `412 BANK_ACCOUNT_HAS_PENDING_TRANSACTIONS`,
 * `412 BANK_ACCOUNT_HAS_OPEN_PAYMENT_BATCHES`, `409 OPTIMISTIC_LOCK_CONFLICT`,
 * `409 BANK_ACCOUNT_CLOSED` (déjà clos), `404` (archivé, inexistant).
 *
 * @param closedOn Date de clôture `AAAA-MM-JJ`, informative ; l'écran l'envoie
 *   toujours (préremplie à la date **locale** du jour).
 */
export async function closeBankAccount(
	id: number,
	version: number,
	closedOn: string,
): Promise<BankAccountSummary> {
	const raw = await apiClient.post<BankAccountSummaryRaw>(`/api/v1/bank-accounts/${id}/close`, {
		version,
		closedOn,
	});
	return parseBankAccount(raw);
}

/**
 * POST /api/v1/bank-accounts/{id}/reopen — **rouvre** un compte bancaire clos
 * (Story 28-4a/28-4b, #642 ; Comptable+). Le compte ne redevient pas principal.
 * Refus : `409 BANK_ACCOUNT_NOT_CLOSED`, `409 OPTIMISTIC_LOCK_CONFLICT`, `404`.
 */
export async function reopenBankAccount(id: number, version: number): Promise<BankAccountSummary> {
	const raw = await apiClient.post<BankAccountSummaryRaw>(`/api/v1/bank-accounts/${id}/reopen`, {
		version,
	});
	return parseBankAccount(raw);
}

/**
 * PATCH /api/v1/bank-accounts/{id} — met à jour `journalAccountId` uniquement
 * (legacy 8-5a-zero, scope strict). Pour édition complète, utiliser PUT.
 */
export async function updateBankAccountJournalLink(
	id: number,
	journalAccountId: number | null,
	version: number,
): Promise<BankAccountSummary> {
	const raw = await apiClient.patch<BankAccountSummaryRaw>(`/api/v1/bank-accounts/${id}`, {
		journalAccountId,
		version,
	});
	return parseBankAccount({ ...raw, currentBalance: raw.currentBalance ?? null, lastTransactionDate: raw.lastTransactionDate ?? null });
}

/**
 * Story 34-2 (#533, C-e341-7) — sens d'un solde d'ouverture, **lu sur le
 * relevé** : `debit` (*Avoir*) le porte au débit du compte lié, `credit`
 * (*Dette* : un découvert, une carte) au crédit.
 */
export type OpeningBalanceSide = 'debit' | 'credit';

/** Story 34-2 (#533) — un solde d'ouverture : montant décimal positif (chaîne) et sens. */
export interface OpeningBalancePayload {
	amount: string;
	side: OpeningBalanceSide;
}

/**
 * Story 34-2 (#533) — l'écriture d'ouverture passée avec un compte bancaire :
 * de quoi la nommer (numéro, date `AAAA-MM-JJ`).
 */
export interface OpeningEntrySummary {
	id: number;
	entryNumber: number;
	entryDate: string;
	fiscalYearId: number;
}

/**
 * Payload de création — POST /api/v1/bank-accounts (Story v014-1).
 */
export type NewBankAccountPayload = NewBankAccountFields & BankOrCardFields;

/** Champs communs aux deux types de compte. */
interface NewBankAccountFields {
	bankName: string;
	/** Story 26-2a — optionnel : l'omettre crée un compte sans description. */
	description?: string | null;
	journalAccountId?: number | null;
	/**
	 * Story 34-2 (#533) — solde d'ouverture, facultatif ; exige
	 * `journalAccountId`. L'écran ne l'envoie que si un montant est saisi.
	 */
	openingBalance?: OpeningBalancePayload;
}

/** Story 34-2 (#533) — le compte créé, et l'écriture d'ouverture passée avec lui (`null` sans solde). */
export interface CreatedBankAccount {
	account: BankAccountSummary;
	openingEntry: OpeningEntrySummary | null;
}

/**
 * Story 34-11a2 (#972 ; revue P1, B10) — union discriminée par le type : un
 * compte bancaire porte **obligatoirement** son IBAN (type absent = compte
 * bancaire, payload de la 0.17.5) ; une carte n'a ni IBAN, ni QR-IBAN, ni
 * « principal », et peut porter ses quatre derniers chiffres.
 */
type BankOrCardFields =
	| {
			accountKind?: 'bank';
			iban: string;
			qrIban?: string | null;
			isPrimary?: boolean;
			cardLast4?: never;
			paypal?: never;
			stripe?: never;
	  }
	| {
			accountKind: 'credit_card';
			cardLast4?: string | null;
			iban?: never;
			qrIban?: never;
			isPrimary?: never;
			paypal?: never;
			stripe?: never;
	  }
	| ({
			/**
			 * Story 38-11d (AC-1) — un compte PayPal (compte bancaire, 38-11b) :
			 * IBAN facultatif (`null`), ni QR-IBAN ni « principal », compte du
			 * plan obligatoire (admis en devise). Story 40-7d : de même un compte
			 * Stripe, sous `stripe`.
			 */
			accountKind?: 'bank';
			iban: string | null;
			qrIban: null;
			isPrimary: false;
			cardLast4?: never;
	  } & ProviderSettingsBody);

/**
 * POST /api/v1/bank-accounts — crée un nouveau compte bancaire post-onboarding
 * (Comptable+). Transition primary silencieuse symétrique au PUT — si
 * `isPrimary=true` et un autre primary existe, l'ancien est démoté
 * silencieusement dans la même transaction.
 *
 * Story 34-2 (#533) : avec `openingBalance`, le serveur passe l'écriture
 * d'ouverture dans la même transaction ; un refus (`OPENING_COMPLEMENT_*`) ne
 * crée rien. La réponse porte alors `openingEntry`.
 */
export async function createBankAccount(
	payload: NewBankAccountPayload,
): Promise<CreatedBankAccount> {
	const raw = await apiClient.post<BankAccountSummaryRaw & { openingEntry?: OpeningEntrySummary }>(
		'/api/v1/bank-accounts',
		payload,
	);
	return {
		account: parseBankAccount({ ...raw, currentBalance: raw.currentBalance ?? null, lastTransactionDate: raw.lastTransactionDate ?? null }),
		openingEntry: raw.openingEntry ?? null,
	};
}

/**
 * POST /api/v1/bank-accounts/{id}/opening-balance — Story 34-2 (#533, AC-6) :
 * saisit le solde d'ouverture d'un compte existant, lié (Comptable+). Refus :
 * `404`, `409 BANK_ACCOUNT_CLOSED`, `412 BANK_ACCOUNT_NOT_CONFIGURED`, puis
 * ceux du complément (`OPENING_COMPLEMENT_*`, dont `ACCOUNT_MOVED`).
 */
export async function enterOpeningBalance(
	id: number,
	payload: OpeningBalancePayload,
): Promise<OpeningEntrySummary> {
	return apiClient.post<OpeningEntrySummary>(`/api/v1/bank-accounts/${id}/opening-balance`, payload);
}

/**
 * Payload d'édition complète — PUT /api/v1/bank-accounts/{id} (Story v014-1).
 */
export type UpdateBankAccountPayload = UpdateBankAccountFields &
	(
		| {
				/** IBAN obligatoire d'un compte bancaire. */
				iban: string;
				qrIban?: string | null;
				isPrimary?: boolean;
				cardLast4?: never;
				/**
				 * Story 38-11d (AC-1) — `null` retire les paramètres PayPal d'un
				 * compte qui en portait ; **absent** = inchangé (compte non PayPal).
				 */
				paypal?: null;
				/** Story 40-7d (AC-1) — de même pour les paramètres Stripe. */
				stripe?: null;
		  }
		| ({
				/**
				 * Story 38-11d (AC-1) — un compte PayPal : IBAN facultatif, ni QR-IBAN ni « principal ».
				 * Story 40-7d : de même un compte Stripe, sous `stripe`.
				 */
				iban: string | null;
				qrIban: null;
				isPrimary: false;
				cardLast4?: never;
		  } & ProviderSettingsBody)
		| {
				/**
				 * Story 34-11a2 (#972) — une carte : ni IBAN (le serveur le refuse :
				 * revue P1 de la 34-11a1, E5/B7), ni QR-IBAN, ni « principal » ; ses
				 * quatre chiffres, **absents ⇒ effacés** (remplacement complet). Le
				 * type n'est pas envoyé : absent, il vaut le type courant.
				 */
				cardLast4: string | null;
				iban?: never;
				qrIban?: never;
				isPrimary?: never;
				paypal?: never;
				stripe?: never;
		  }
	);

/** Champs communs du `PUT`. */
interface UpdateBankAccountFields {
	bankName: string;
	/**
	 * Story 26-2a — **obligatoire** : le `PUT` remplace tout, l'omettre
	 * **efface** la description (C-26-2-19). `null` pour l'effacer exprès.
	 */
	description: string | null;
	journalAccountId?: number | null;
	version: number;
}

/**
 * PUT /api/v1/bank-accounts/{id} — édition complète d'un compte bancaire
 * (Comptable+). Optimistic lock via `version`.
 */
export async function updateBankAccount(
	id: number,
	payload: UpdateBankAccountPayload,
): Promise<BankAccountSummary> {
	const raw = await apiClient.put<BankAccountSummaryRaw>(`/api/v1/bank-accounts/${id}`, payload);
	return parseBankAccount({ ...raw, currentBalance: raw.currentBalance ?? null, lastTransactionDate: raw.lastTransactionDate ?? null });
}

/**
 * DELETE /api/v1/bank-accounts/{id} — soft-delete (archive) un compte bancaire
 * (Comptable+). Refus 412 si transactions existent OU si primary + autres
 * comptes ni archivés ni clos (Story 28-4a) ; refus 409 BANK_ACCOUNT_CLOSED
 * d'un compte clos.
 */
export async function archiveBankAccount(
	id: number,
	version: number,
): Promise<BankAccountSummary> {
	const raw = await apiClient.delete<BankAccountSummaryRaw>(`/api/v1/bank-accounts/${id}`, {
		version,
	});
	return parseBankAccount({ ...raw, currentBalance: raw.currentBalance ?? null, lastTransactionDate: raw.lastTransactionDate ?? null });
}
