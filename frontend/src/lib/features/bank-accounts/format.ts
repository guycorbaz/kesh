// Story v014-1 — utilitaires d'affichage partagés pour les bank_accounts.
// Story 26-2a (#622) — nom et libellé d'un compte bancaire (description
// optionnelle), source unique de leur forme côté frontend ; même règle que
// `bank_account_display_name` côté serveur (`kesh_db::entities::bank_account`).
import Big from 'big.js';
import type { AccountResponse } from '$lib/features/accounts/accounts.types';
// F17 Pass 1 code review : factoriser `shortIban` entre `+page.svelte`
// homepage et `bank-accounts/+page.svelte` pour éviter la divergence de
// seuil (4 vs 8 caractères).

/**
 * Tronque un IBAN pour affichage compact : `••••` + 4 derniers chars.
 * Pour les IBAN < 4 chars (cas pathologique de fixture), retourne tel quel.
 *
 * Exemple : `CH4431999123000889012` → `••••9012`.
 */
export function shortIban(iban: string): string {
	if (iban.length < 4) return iban;
	return `••••${iban.slice(-4)}`;
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

/** Ce qu'il faut pour le libellé d'option d'un sélecteur : le nom et l'IBAN. */
export type BankAccountOptionSource = BankAccountLabelSource & { iban: string };

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
 */
export function bankAccountOptionLabel(
	a: BankAccountOptionSource,
	ibanStyle: 'dash' | 'parens' = 'dash',
): string {
	const name = bankAccountName(a);
	return ibanStyle === 'parens' ? `${name} (${a.iban})` : `${name}${SEPARATOR}${a.iban}`;
}

/** Options de {@link linkableLedgerAccounts}. */
export interface LinkableLedgerOptions {
	/**
	 * Ne garder que les comptes de passif (`Liability`) — pour le compte de dette
	 * d'une carte de crédit (Story 34-11a2, C-e341-33). Faux par défaut : actif
	 * ou passif.
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
				(options.liabilityOnly
					? a.accountType === 'Liability'
					: a.accountType === 'Asset' || a.accountType === 'Liability') &&
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
 * Story 34-2 (#533) — le montant saisi, tel qu'envoyé : les séparateurs de
 * milliers suisses (apostrophes droite ou typographique, espaces) retirés, la
 * virgule décimale lue comme un point. `null` si le champ est vide : aucun solde.
 * Le contrôle de forme reste celui du serveur (positif, quatre décimales).
 */
export function openingAmountPayload(raw: string): string | null {
	const cleaned = raw.replace(/['’\s  ]/g, '').replace(',', '.');
	return cleaned === '' ? null : cleaned;
}
