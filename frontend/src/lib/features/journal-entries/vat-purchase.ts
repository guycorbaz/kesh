/**
 * Helpers de l'assistant de saisie d'achat avec TVA récupérable (Story 18-1c).
 *
 * L'assistant pré-remplit une écriture comptable d'achat en 3 lignes
 * (`D charge / D impôt préalable / C contrepartie`) à partir d'un montant HT et
 * d'un taux TVA, postée via l'endpoint existant `POST /journal-entries`. Aucun
 * appel réseau ici : pure logique de génération de lignes.
 *
 * **Parité backend (DC-c2)** : `lineVatAmount` réplique EXACTEMENT
 * `kesh_core::accounting::line_vat_amount(base_ht, rate_percent)` —
 * `round(base × taux ÷ 100, 2)` avec arrondi **half-up away-from-zero**
 * (`MidpointAwayFromZero` backend ≡ `Big.roundHalfUp` côté front car les
 * montants d'achat HT sont positifs). Tout écart d'un centime déséquilibrerait
 * l'écriture → rejet `ENTRY_UNBALANCED` (le backend compare en égalité décimale
 * exacte, sans epsilon).
 *
 * **Verrou d'équilibre (F-OPUS-C1)** : les 3 lignes émettent toutes des montants
 * arrondis à 2 décimales (`débit charge = round₂(HT)`, PAS le HT brut). Comme
 * `vat` a exactement 2 décimales, `round₂(HT) + vat == round₂(HT + vat)` —
 * l'équilibre tient par construction même pour un HT saisi à 3-4 décimales
 * (autorisé par `isValidAmount`).
 */

import Big from 'big.js';
import { parseAmount } from './balance';
import type { LineDraft } from './form-helpers';

/**
 * Montant de TVA pour une base HT et un taux en pourcent, arrondi au centime
 * (half-up away-from-zero), en parité exacte avec le backend `line_vat_amount`.
 *
 * @param ht base HT (string, virgule ou point acceptés)
 * @param ratePercent taux en pourcent (string, ex. `"8.10"`)
 * @returns montant TVA à 2 décimales (string, ex. `"81.00"`)
 */
export function lineVatAmount(ht: string, ratePercent: string): string {
	// `parseAmount` normalise virgule → point avant `new Big(...)` (DRY, H1).
	const base = parseAmount(ht);
	const rate = parseAmount(ratePercent);
	return base.times(rate).div(100).round(2, Big.roundHalfUp).toFixed(2);
}

/** Arrondit un montant string (virgule ou point) à 2 décimales half-up. */
function round2(amount: string): string {
	return parseAmount(amount).round(2, Big.roundHalfUp).toFixed(2);
}

/** Paramètres de génération d'une écriture d'achat avec TVA récupérable. */
export interface PurchaseVatParams {
	/** Compte de charge (Expense, débit HT). */
	chargeAccountId: number;
	/** Montant HT saisi (string, virgule ou point). */
	htAmount: string;
	/** Taux TVA en pourcent (string, ex. `"8.10"`). */
	ratePercent: string;
	/** Compte de contrepartie (crédit TTC : fournisseur, banque, caisse…). */
	counterpartyAccountId: number;
	/** Compte d'impôt préalable (`default_vat_recoverable_account_id`). */
	recoverableAccountId: number;
}

/**
 * Génère les lignes équilibrées d'une écriture d'achat avec TVA récupérable.
 *
 * - `vat > 0` → 3 lignes : `D charge round₂(HT)`, `D impôt préalable vat`,
 *   `C contrepartie round₂(HT) + vat`.
 * - `vat == 0` (taux exempt/0) → 2 lignes : `D charge round₂(HT)`,
 *   `C contrepartie round₂(HT)` (pas de ligne d'impôt préalable — une ligne à
 *   `debit = 0` serait rejetée par `chk_jel_debit_credit_exclusive`).
 *
 * Équilibre garanti par construction : `Σdebit == Σcredit` exactement (DC-c7).
 */
export function buildPurchaseVatLines(params: PurchaseVatParams): LineDraft[] {
	const {
		chargeAccountId,
		htAmount,
		ratePercent,
		counterpartyAccountId,
		recoverableAccountId
	} = params;

	const charge = round2(htAmount);
	const vat = lineVatAmount(htAmount, ratePercent);
	const ttc = parseAmount(charge).plus(vat).toFixed(2);

	const lines: LineDraft[] = [
		{ accountId: chargeAccountId, debit: charge, credit: '', projectId: null }
	];

	// Ligne d'impôt préalable émise seulement si la TVA est strictement > 0
	// (F-OPUS-1 / contrainte DB `chk_jel_debit_credit_exclusive` : `debit > 0 XOR credit > 0`).
	if (parseAmount(vat).gt(0)) {
		lines.push({ accountId: recoverableAccountId, debit: vat, credit: '', projectId: null });
	}

	lines.push({ accountId: counterpartyAccountId, debit: '', credit: ttc, projectId: null });

	return lines;
}

/**
 * `true` si une ligne de brouillon n'est pas vierge (compte choisi OU montant
 * saisi). Une `LineDraft` initiale `{ accountId: null, debit: '', credit: '' }`
 * est vierge. Sert à décider la confirmation avant remplacement (AC6/DC-c9).
 */
export function isDraftLineNonEmpty(line: LineDraft): boolean {
	return line.accountId !== null || line.debit.trim() !== '' || line.credit.trim() !== '';
}
