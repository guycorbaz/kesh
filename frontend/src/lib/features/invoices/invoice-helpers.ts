/**
 * Helpers purs pour les factures (Story 5.1).
 *
 * Arithmétique via `big.js` — ne JAMAIS utiliser `Number`/`parseFloat`
 * pour les montants (perte de précision au-delà de `MAX_SAFE_INTEGER`).
 */

import Big from 'big.js';
import { formatSwissAmount } from '$lib/features/journal-entries/balance';
import type { CreateInvoiceLineRequest } from './invoices.types';

// Mode d'arrondi local, appliqué par appel à `.round(dp, mode)` plutôt que via
// `Big.RM` global, pour ne pas affecter d'autres features (journal-entries,
// products) qui peuvent dépendre du mode par défaut.
//
// ROUND_HALF_UP (1) arrondit 0.5 vers +∞. Pour les montants **positifs** uniquement
// (Story 5.1 : `unit_price ≥ 0`, `quantity > 0`), ce mode est équivalent à
// `rust_decimal::Decimal::round_dp` (MidpointAwayFromZero) côté backend.
// Attention : ces modes divergent sur les valeurs négatives (Rust: -0.5 → -1,
// big.js RHU: -0.5 → 0). Les avoirs/notes de crédit (Epic 10) devront revoir
// cette équivalence si des lignes négatives sont introduites.
const ROUND_HALF_UP = 1;

/**
 * Calcule `line_total = quantity × unit_price`, arrondi à 4 décimales
 * (cohérent avec `DECIMAL(19,4)` côté DB).
 * Retourne `"0.0000"` si l'un des inputs est invalide.
 */
export function computeLineTotal(qty: string, unitPrice: string): string {
	try {
		return new Big(qty).times(unitPrice).round(4, ROUND_HALF_UP).toFixed(4);
	} catch {
		return '0.0000';
	}
}

/**
 * Somme `line_total` pour toutes les lignes, arrondie à 4 décimales.
 */
export function computeInvoiceTotal(lines: Pick<CreateInvoiceLineRequest, 'quantity' | 'unitPrice'>[]): string {
	// Arrondit chaque line_total à 4 décimales avant sommation — miroir du
	// backend (`compute_line_total` applique `.round_dp(4)` avant Σ).
	return lines
		.reduce((acc, l) => {
			try {
				return acc.plus(new Big(l.quantity).times(l.unitPrice).round(4, ROUND_HALF_UP));
			} catch {
				return acc;
			}
		}, new Big(0))
		.toFixed(4);
}

/**
 * Formate un montant facture (string décimal 4 décimales) au format suisse :
 * `"1500.0000"` → `"1’500.00"`. Délègue à `formatSwissAmount` (DRY).
 */
export function formatInvoiceTotal(d: string | null | undefined): string {
	if (!d) return '';
	try {
		return formatSwissAmount(new Big(d));
	} catch {
		return String(d);
	}
}
