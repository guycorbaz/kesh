// Story v014-1 — utilitaires d'affichage partagés pour les bank_accounts.
// Story 26-2a (#622) — nom et libellé d'un compte bancaire (description
// optionnelle), source unique de leur forme côté frontend ; même règle que
// `bank_account_display_name` côté serveur (`kesh_db::entities::bank_account`).
import Big from 'big.js';
import type { AccountResponse } from '$lib/features/accounts/accounts.types';
import { admitsLedgerType } from './credit-card';
// F17 Pass 1 code review : factoriser `shortIban` entre `+page.svelte`
// homepage et `bank-accounts/+page.svelte` pour éviter la divergence de
// seuil (4 vs 8 caractères).

/**
 * Tronque un IBAN pour affichage compact : `••••` + 4 derniers chars.
 * Pour les IBAN < 4 chars (cas pathologique de fixture), retourne tel quel.
 *
 * Exemple : `CH4431999123000889012` → `••••9012`.
 *
 * Story 34-11a1 (#972) — gardé : `null` (compte sans IBAN) rend `''`, jamais
 * de `TypeError`.
 */
export function shortIban(iban: string | null): string {
	if (iban === null) return '';
	if (iban.length < 4) return iban;
	return `••••${iban.slice(-4)}`;
}

/** Espace insécable (U+00A0) entre `••••` et les quatre chiffres d'une carte. */
export const CARD_LAST4_SEPARATOR = '\u00a0';

/**
 * Story 34-11a1 (#972, C-e341-21) — **seule** forme d'affichage des quatre
 * chiffres d'une carte : `"•••• 1234"`, séparés par une espace **insécable**
 * ({@link CARD_LAST4_SEPARATOR}, revue P1 A7) pour que la forme ne se coupe
 * jamais en colonne étroite. `null` sans quatre chiffres.
 */
export function cardLast4Label(cardLast4: string | null | undefined): string | null {
	return cardLast4 ? `••••${CARD_LAST4_SEPARATOR}${cardLast4}` : null;
}

/** Ce qui identifie un compte au-delà de son nom : l'IBAN, ou les quatre chiffres d'une carte. */
export type BankAccountIdSource = { iban: string | null; cardLast4?: string | null };

/**
 * Identifiant **court** d'un compte, pour les listes qui le montrent à part
 * (tableau *Comptes bancaires*, accueil, *Paramètres*) — Story 34-11a1 (#972) :
 * `"••••9012"` pour un IBAN, `"•••• 1234"` pour une carte, `''` pour un compte
 * sans IBAN ni quatre chiffres (le nom seul s'affiche alors).
 */
export function bankAccountShortId(a: BankAccountIdSource): string {
	return a.iban !== null ? shortIban(a.iban) : (cardLast4Label(a.cardLast4) ?? '');
}

/**
 * Identifiant **complet** d'un compte, après son nom (sélecteurs, colonne IBAN
 * du tableau) — Story 34-11a1 (#972) : l'IBAN entier, sinon `"•••• 1234"` pour
 * une carte, sinon `null` (le nom seul s'affiche).
 */
export function bankAccountFullId(a: BankAccountIdSource): string | null {
	return a.iban ?? cardLast4Label(a.cardLast4);
}

/**
 * Formate un solde CHF avec 2 décimales (locale `fr-CH`).
 * `null` → message i18n « Solde non disponible » fourni par le caller.
 *
 * Story 25-6-a (#389) — prend une **chaîne décimale ou un `Big`**, jamais un
 * `number` : l'arrondi au centime se fait en `big.js` (demi loin de zéro,
 * l'arrondi du dépôt) ; seul le résultat, déjà au centime, devient un `Number`
 * pour `Intl` — exact jusqu'à ~9·10¹³ CHF.
 */
export function formatChfBalance(balance: string | Big): string {
	return new Intl.NumberFormat('fr-CH', {
		style: 'currency',
		currency: 'CHF',
		minimumFractionDigits: 2,
	}).format(toCentimeNumber(balance));
}

/** Arrondi au centime ; un résultat nul rend `0`, jamais `-0` (« -0.00 CHF »). */
function toCentimeNumber(balance: string | Big): number {
	const rounded = new Big(balance).round(2, Big.roundHalfUp);
	return rounded.eq(0) ? 0 : rounded.toNumber();
}

/**
 * Longueur maximale, en caractères, de la description d'un compte bancaire
 * (Story 26-2a, #622). Alimente les `maxlength` des formulaires ; la borne
 * serveur est `BANK_ACCOUNT_DESCRIPTION_MAX_CHARS` (`kesh_db`).
 */
export const BANK_ACCOUNT_DESCRIPTION_MAX = 60;

/** Séparateur banque / description / IBAN : tiret cadratin (U+2014) entouré d'espaces. */
const SEPARATOR = ' \u2014 ';

/**
 * Ce qu'il faut pour nommer un compte bancaire. `description` est
 * **obligatoire** (`null` permis) : un objet qui l'aurait oubliée est refusé
 * par `npm run check`.
 */
export type BankAccountLabelSource = { bankName: string; description: string | null };

/**
 * Ce qu'il faut pour le libellé d'option d'un sélecteur : le nom et l'IBAN
 * (`null` pour un compte sans IBAN, Story 34-11a1), et les quatre chiffres
 * d'une carte s'ils sont connus.
 */
export type BankAccountOptionSource = BankAccountLabelSource & BankAccountIdSource;

/**
 * Nom d'affichage d'un compte bancaire : `"UBS — Personnel"` avec
 * description, `"UBS"` sans. Une description `null`, `undefined`, vide ou
 * faite d'espaces vaut absente.
 *
 * Pour les listes qui montrent l'IBAN à part (écran *Comptes bancaires*,
 * accueil, *Paramètres*).
 */
export function bankAccountName(a: BankAccountLabelSource): string {
	const description = a.description?.trim();
	return description ? `${a.bankName}${SEPARATOR}${description}` : a.bankName;
}

/**
 * Libellé d'option d'un sélecteur de compte bancaire : le nom
 * ({@link bankAccountName}) suivi de l'IBAN complet.
 *
 * - `'dash'` (défaut) : `"UBS — Personnel — CH93…"`, `"UBS — CH93…"` sans
 *   description ;
 * - `'parens'` : `"UBS — Personnel (CH93…)"`, `"UBS (CH93…)"` sans — pour le
 *   **seul** sélecteur de l'import bancaire, qui garde ainsi exactement son
 *   affichage d'avant la description (C-26-2-26).
 *
 * Story 34-11a1 (#972) — un compte sans IBAN montre `"•••• 1234"` à la place de
 * l'IBAN si ses quatre chiffres sont connus, et son **nom seul** sinon.
 */
export function bankAccountOptionLabel(
	a: BankAccountOptionSource,
	ibanStyle: 'dash' | 'parens' = 'dash',
): string {
	const name = bankAccountName(a);
	const id = bankAccountFullId(a);
	if (id === null) return name;
	return ibanStyle === 'parens' ? `${name} (${id})` : `${name}${SEPARATOR}${id}`;
}

/** Options de {@link linkableLedgerAccounts}. */
export interface LinkableLedgerOptions {
	/**
	 * Ne garder que les comptes de passif (`Liability`) — pour le compte de dette
	 * d'une carte de crédit (Story 34-11a2, C-e341-33). Faux par défaut : actif
	 * ou passif.
	 *
	 * ⚠️ `liabilityOnly` est l'autre nom du type de compte bancaire : vrai pour une
	 * carte (`'credit_card'`), faux pour un compte bancaire (`'bank'`). La règle
	 * elle-même vit dans `admitsLedgerType` (`credit-card.ts`), appelée avec ce
	 * type ; un appelant qui connaît le type passe donc
	 * `liabilityOnly: kind === 'credit_card'`, jamais un filtre à lui
	 * (intégration de la 0.17.6, raccord 1, revue P1 B-3).
	 */
	liabilityOnly?: boolean;
}

/**
 * Les comptes du plan auxquels un compte bancaire peut être **lié** — source
 * unique du filtre, pour l'écran *Comptes bancaires* et l'étape bancaire de
 * l'assistant (Story 34-1, #966, C-e341-5) :
 *
 * - actifs ;
 * - imputables (`postable`, Story 14-3b : le compte lié est posté à la
 *   réconciliation) ;
 * - de type actif ou passif (seulement passif avec `liabilityOnly`) ;
 * - hors `excludedIds` — les comptes de créance désignés (écran *Comptes
 *   bancaires*, Story 15-6c) ou les comptes de rôle `Receivable` / `Payable`
 *   (assistant, avant que les réglages n'existent, C-e341-3).
 *
 * La liste rendue est **triée par numéro** (`localeCompare`) — revue P1 (B-5) :
 * l'ordre ne dépend plus de celui que rend le serveur, et les trois écrans
 * l'affichent pareil. Le refus du serveur reste l'autorité : ce filtre n'évite
 * qu'un aller-retour.
 */
export function linkableLedgerAccounts(
	accounts: AccountResponse[],
	excludedIds: Set<number>,
	options: LinkableLedgerOptions = {},
): AccountResponse[] {
	return accounts
		.filter(
			(a) =>
				a.active &&
				a.postable &&
				// Intégration de la 0.17.6 (C-e34a2-1, raccord 1) : la règle du type
				// est `admitsLedgerType`, source unique avec l'écran de création.
				admitsLedgerType(options.liabilityOnly ? 'credit_card' : 'bank', a.accountType) &&
				!excludedIds.has(a.id),
		)
		.sort((a, b) => a.number.localeCompare(b.number));
}

/**
 * Numéro du compte que l'assistant **présélectionne** pour le compte bancaire
 * (décision de Guy, #966). ⛔ Une présélection d'écran, rien d'autre : aucun
 * rôle ne se déduit d'un numéro (Story 14-3a), le serveur ne connaît pas cette
 * valeur et l'utilisateur la change ou la retire librement.
 */
export const PRESELECTED_BANK_LEDGER_NUMBER = '1020';

/**
 * L'identifiant à présélectionner dans une liste **déjà filtrée** par
 * {@link linkableLedgerAccounts} : le compte numéroté
 * {@link PRESELECTED_BANK_LEDGER_NUMBER} s'il y figure, `null` (« Non
 * configuré ») sinon — jamais un autre compte à sa place.
 */
export function preselectedBankLedgerId(linkable: AccountResponse[]): number | null {
	return linkable.find((a) => a.number === PRESELECTED_BANK_LEDGER_NUMBER)?.id ?? null;
}

/**
 * Story 34-2 (#533, C-e341-7) — sens présélectionné du solde d'ouverture :
 * *Avoir* (`debit`) pour un compte lié d'actif, *Dette* (`credit`) pour un
 * passif (découvert, carte). Compte inconnu ou absent : *Avoir*. Une
 * présélection seulement : l'utilisateur la change, le serveur ne déduit rien.
 */
export function defaultOpeningSide(
	accounts: AccountResponse[],
	ledgerId: number | null,
): 'debit' | 'credit' {
	const account = ledgerId === null ? undefined : accounts.find((a) => a.id === ledgerId);
	return account?.accountType === 'Liability' ? 'credit' : 'debit';
}

/**
 * Story 35-6 — l'issue du classement d'une saisie de solde d'ouverture :
 * `empty` (aucun solde), `clear` (la seule qui part au serveur, avec sa charge),
 * `ambiguous` (« 1.234 », « 1,234 » : mille ou un franc ?), `unreadable`, `negative`.
 */
export type OpeningAmountKind = 'empty' | 'ambiguous' | 'clear' | 'unreadable' | 'negative';

/** Story 35-6 — le classement d'une saisie, et la charge à envoyer si elle est claire. */
export type OpeningAmountReading = { kind: OpeningAmountKind; payload?: string };

/**
 * Story 35-6 (D1) — séparateurs de groupe : apostrophes droite et typographique et
 * tout blanc Unicode (`\s` : espace, U+00A0, U+202F, U+2009, tabulation…), ceux que
 * l'écran retirait déjà. Permis dans la partie entière seulement.
 */
const GROUP_SEPARATOR = "['’\\s]";
/** Séparateurs de groupe et apostrophes de tête et de queue (L1). */
const EDGES = new RegExp(`^${GROUP_SEPARATOR}+|${GROUP_SEPARATOR}+$`, 'g');
const GROUP_SEPARATORS = new RegExp(GROUP_SEPARATOR, 'g');
/** Une partie entière « groupée » : un séparateur de groupe entre deux chiffres. */
const GROUPED = new RegExp(`[0-9]${GROUP_SEPARATOR}+[0-9]`);
const UNREADABLE: OpeningAmountReading = { kind: 'unreadable' };

/**
 * Story 35-6 (#1026, #1028, #1025) — classe la saisie du solde d'ouverture **avant**
 * tout envoi, par le pipeline ordonné de la fiche (D2) ; la première étape qui conclut
 * l'emporte :
 *
 * 1. **L1** — rognage des blancs et des apostrophes de tête et de queue ; vide → `empty`.
 * 2. **L2** — un `+` ou un `-` en tête est retenu, le reste rogné à nouveau ; reste vide
 *    ou portant encore un signe (`++1`, `1-2`) → `unreadable`.
 * 3. **L3** — le souligné suit le serveur (`rust_decimal`) : avant le premier chiffre,
 *    `unreadable` (`_1`) ; après, retiré (`1_000` → 1000, #1028).
 * 4. **L4** — plus d'un `.` ou `,` → `unreadable` ; le séparateur décimal coupe la
 *    partie entière E et la fraction F.
 * 5. **L5** — F ne porte que des chiffres ASCII (un blanc ou une apostrophe après la
 *    décimale, `1. 234`, n'a pas de lecture sûre).
 * 6. **L6** — E, ses séparateurs de groupe retirés, ne porte que des chiffres ASCII
 *    (un exposant, `1e3`, n'est jamais lu) ; E et F ne sont pas vides ensemble.
 * 7. **L7** — plus de quatre décimales (zéros de queue au-delà de la quatrième
 *    retirés) → `unreadable` ; signe `-` sur une valeur non nulle → `negative` ;
 *    **ambiguë** : un séparateur décimal, F de trois chiffres tels que saisis, E non
 *    groupée et de un à trois chiffres hors zéros de tête
 *    (`^0*[1-9][0-9]{0,2}[.,][0-9]{3}$`) ; sinon `clear`.
 *
 * La charge d'une saisie claire (D5) suit `^[0-9]+(\.[0-9]{1,4})?$` : chiffres de E
 * (`0` si E est vide), puis `.` et F si F n'est pas vide ; aucun signe ; zéros de tête
 * gardés. Le serveur la lit à la valeur que l'aperçu montre (test de parité,
 * `opening_balances.rs`). La table complète est `opening-amount-cases.json`.
 */
export function openingAmountKind(raw: string): OpeningAmountReading {
	// L1
	let rest = raw.replace(EDGES, '');
	if (rest === '') return { kind: 'empty' };
	// L2
	let negative = false;
	if (rest[0] === '+' || rest[0] === '-') {
		negative = rest[0] === '-';
		rest = rest.slice(1).replace(EDGES, '');
	}
	if (rest === '' || /[+-]/.test(rest)) return UNREADABLE;
	// L3
	const firstUnderscore = rest.indexOf('_');
	if (firstUnderscore !== -1) {
		const firstDigit = rest.search(/[0-9]/);
		if (firstDigit === -1 || firstUnderscore < firstDigit) return UNREADABLE;
		rest = rest.replace(/_/g, '');
	}
	// L4
	const decimalSeparators = rest.match(/[.,]/g) ?? [];
	if (decimalSeparators.length > 1) return UNREADABLE;
	const cut = rest.search(/[.,]/);
	const hasDecimal = cut !== -1;
	const integerPart = hasDecimal ? rest.slice(0, cut) : rest;
	const typedFraction = hasDecimal ? rest.slice(cut + 1) : '';
	// L5
	if (!/^[0-9]*$/.test(typedFraction)) return UNREADABLE;
	// L6
	const grouped = GROUPED.test(integerPart);
	const integerDigits = integerPart.replace(GROUP_SEPARATORS, '');
	if (!/^[0-9]*$/.test(integerDigits)) return UNREADABLE;
	if (integerDigits === '' && typedFraction === '') return UNREADABLE;
	// L7
	const fraction =
		typedFraction.slice(0, 4) + typedFraction.slice(4).replace(/0+$/, '');
	if (fraction.length > 4) return UNREADABLE;
	if (negative && /[1-9]/.test(integerDigits + fraction)) return { kind: 'negative' };
	if (
		hasDecimal &&
		typedFraction.length === 3 &&
		!grouped &&
		/^0*[1-9][0-9]{0,2}$/.test(integerDigits)
	) {
		return { kind: 'ambiguous' };
	}
	const payload = (integerDigits === '' ? '0' : integerDigits) + (fraction === '' ? '' : `.${fraction}`);
	return { kind: 'clear', payload };
}

/**
 * Story 34-2 (#533), adossé au classement par la Story 35-6 (D5) — la charge
 * envoyée au serveur pour une saisie **claire** ; `null` sinon (vide compris). La
 * page classe avant de prendre la charge : un `null` n'y vaut « aucun solde »
 * qu'après un `kind === 'empty'`.
 */
export function openingAmountPayload(raw: string): string | null {
	const reading = openingAmountKind(raw);
	return reading.kind === 'clear' ? (reading.payload ?? null) : null;
}

/**
 * Story 34-2 (revue P1, B-1) — le montant **tel qu'il sera écrit**, au format
 * suisse de `Intl` pour `fr-CH` (`1 234.50 CHF`, le séparateur de milliers est une
 * espace fine insécable, U+202F), exact jusqu'à quatre décimales
 * ({@link formatChfExact}), pour le montrer avant le geste ; `null` si la saisie
 * n'est pas claire ({@link openingAmountKind}, Story 35-6 : jamais « NaN CHF »,
 * #1028). Limites suivies : « 0.00 CHF » pour un montant nul (#1027), moteur sans
 * `Intl.NumberFormat` v3 (#1031).
 */
export function openingAmountPreview(raw: string): string | null {
	const amount = openingAmountPayload(raw);
	return amount === null ? null : formatChfExact(amount);
}

/** Formateur CHF à deux décimales au moins, quatre au plus (la précision de la colonne). */
const CHF_EXACT = new Intl.NumberFormat('fr-CH', {
	style: 'currency',
	currency: 'CHF',
	minimumFractionDigits: 2,
	maximumFractionDigits: 4,
});

/**
 * Story 34-2 — un montant décimal (chaîne) au format suisse, **sans arrondi**
 * jusqu'à quatre décimales : l'aperçu et l'avis disent ce que le serveur écrit
 * (`DECIMAL(19,4)`), là où {@link formatChfBalance} arrondit au centime.
 * La chaîne est passée telle quelle à `Intl.NumberFormat` (format exact des
 * décimaux sous forme de chaîne), sans passer par un `number`.
 */
export function formatChfExact(amount: string): string {
	return (CHF_EXACT.format as (value: string) => string)(amount);
}
