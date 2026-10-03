// Story v014-1 — utilitaires d'affichage partagés pour les bank_accounts.
import Big from 'big.js';
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
