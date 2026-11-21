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
import {
	amountEntryPayload,
	formatSwissAmount,
	readJournalAmount,
} from '$lib/features/journal-entries/balance';

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
 * Story 48-10 (#1173, D1) — la charge à envoyer pour un prix saisi : la saisie rognée, lue
 * comme le montant d'une écriture (`amountEntryPayload`, grammaire de `readJournalAmount`) —
 * virgule ou point décimal, apostrophe des milliers retirée (« 1'250 » → `1250`, « 10,50 » →
 * `10.50`), zéros de tête et de fin gardés. `null` pour le vide, l'ambigu (« 1.250 ») et
 * l'illisible : l'appelant n'envoie qu'après `classifyPriceInput` = `ok`.
 *
 * Avant la story, la fonction remplaçait la virgule par un point et rien d'autre : « 1.250 »
 * partait et le serveur le lisait un franc vingt-cinq. `PRICE_RE` et `isValidPrice`, une
 * seconde grammaire sans appelant de production, sont retirés.
 */
export function normalizePriceInput(raw: string): string | null {
	return amountEntryPayload(raw.trim());
}

/**
 * Classifie le contenu du champ prix pour offrir un message d'erreur précis.
 * - `empty` : champ vide (avant toute saisie ou après effacement)
 * - `negative` : valeur numérique strictement négative (syntaxe `-N`), jugée avant la lecture
 * - `ambiguous` : un point ou une virgule suivis de trois chiffres, qui peuvent séparer les
 *   milliers (« 1.250 », « 12,125 ») — Story 48-10 (#1173) ; un prix de 1 à 999 francs, non
 *   groupé, à trois décimales s'écrit avec la quatrième (« 12.1250 »)
 * - `invalid` : hors de la grammaire d'un montant d'écriture (exposant, `.5`, `5.`, blanc
 *   intérieur, plus de quatre décimales…)
 * - `ok` : lecture claire (`normalizePriceInput` rend sa charge)
 */
export function classifyPriceInput(
	raw: string,
): 'empty' | 'negative' | 'ambiguous' | 'invalid' | 'ok' {
	const trimmed = raw.trim();
	if (trimmed === '') return 'empty';
	if (/^-\s*\d/.test(trimmed)) return 'negative';
	const kind = readJournalAmount(trimmed).kind;
	if (kind === 'ambiguous') return 'ambiguous';
	return kind === 'clear' ? 'ok' : 'invalid';
}
