/**
 * Helpers purs pour les produits (Story 4.2).
 *
 * - `formatPrice(d)` : string décimal (ex: `"1500.0000"`) → `"1’500.00"`
 *   (apostrophe typographique U+2019, norme Swiss SN01 / BFS).
 * - `formatVatRate(d)` : `"8.10"` → `"8.10%"`.
 *
 * Arithmétique et formatage via `big.js` (pattern `journal-entries/balance.ts`).
 * **Ne pas** utiliser `Intl.NumberFormat('de-CH', ...)` : il passe par `parseFloat()`
 * et perd la précision au-delà de `Number.MAX_SAFE_INTEGER` — incompatible avec
 * une appli comptable.
 */

import Big from 'big.js';
import { formatSwissAmount } from '$lib/features/journal-entries/balance';

/**
 * Formate un prix (string décimal backend, ex: `"1500.0000"`) avec le
 * séparateur suisse : `"1’500.00"` (2 décimales, apostrophe typographique
 * U+2019 `’` comme séparateur milliers).
 *
 * Réutilise `formatSwissAmount` de `balance.ts` pour garder **une seule**
 * implémentation du pattern comptable. Retourne `""` si l'input est falsy
 * ou non parsable (défensif).
 */
export function formatPrice(d: string | null | undefined): string {
	if (!d) return '';
	try {
		return formatSwissAmount(new Big(d));
	} catch {
		return String(d);
	}
}

/**
 * Formate un taux TVA (string décimal, ex: `"8.10"`) en `"8.10%"`.
 * Retourne `""` si input vide ou invalide.
 */
export function formatVatRate(d: string | null | undefined): string {
	if (!d) return '';
	try {
		return `${new Big(d).toFixed(2)}%`;
	} catch {
		return String(d);
	}
}

/**
 * Regex de validation client du prix : 1-15 chiffres entiers (sans zéro
 * en tête superflu) + optionnellement 1 à 4 décimales. Accepte `0`,
 * rejette `007.50`, `01`, `.5`, `1.23456`.
 */
const PRICE_RE = /^(0|[1-9][0-9]{0,14})(\.[0-9]{1,4})?$/;

/**
 * Retourne `true` si `raw` est un prix valide. La chaîne est d'abord trimmée
 * (tolère paste avec espaces). Empty string ⇒ `false` : un prix est requis ;
 * ce choix évite la sémantique surprenante d'une "valeur manquante = valide"
 * et supprime le double-garde `!isValidPrice() || trim() === ''` côté caller.
 */
export function isValidPrice(raw: string): boolean {
	const trimmed = raw.trim();
	if (trimmed === '') return false;
	return PRICE_RE.test(trimmed);
}

/**
 * Normalise un prix saisi par l'utilisateur : virgule → point, trim. Gère les
 * claviers mobiles suisses qui produisent `,` sur la touche décimale.
 */
export function normalizePriceInput(raw: string): string {
	return raw.trim().replace(',', '.');
}

/**
 * Classifie le contenu du champ prix pour offrir un message d'erreur précis.
 * - `empty` : champ vide (avant toute saisie ou après effacement)
 * - `negative` : valeur numérique strictement négative (syntaxe `-N`)
 * - `invalid` : tout autre format non parsable ou hors grammaire
 * - `ok` : prix valide selon `isValidPrice`
 */
export function classifyPriceInput(raw: string): 'empty' | 'negative' | 'invalid' | 'ok' {
	const normalized = normalizePriceInput(raw);
	if (normalized === '') return 'empty';
	if (/^-\s*\d/.test(normalized)) return 'negative';
	return PRICE_RE.test(normalized) ? 'ok' : 'invalid';
}
