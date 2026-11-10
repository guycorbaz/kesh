// Story v014-1 — utilitaires d'affichage partagés pour les bank_accounts.
// Story 26-2a (#622) — nom et libellé d'un compte bancaire (description
// optionnelle), source unique de leur forme côté frontend ; même règle que
// `bank_account_display_name` côté serveur (`kesh_db::entities::bank_account`).
import Big from 'big.js';
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

/**
 * Story 34-11a1 (#972, C-e341-21) — **seule** forme d'affichage des quatre
 * chiffres d'une carte : `"•••• 1234"`. `null` sans quatre chiffres.
 */
export function cardLast4Label(cardLast4: string | null | undefined): string | null {
	return cardLast4 ? `•••• ${cardLast4}` : null;
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
