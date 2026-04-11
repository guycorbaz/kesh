/**
 * Validation et calcul d'équilibre des écritures en partie double, côté client.
 *
 * Toute l'arithmétique passe par `big.js` — **jamais `parseFloat`** pour
 * préserver l'intégrité décimale conforme au CO art. 957-964.
 *
 * La regex [`AMOUNT_RE`] applique la limite à 4 décimales imposée par
 * la colonne DB `DECIMAL(19,4)` AVANT tout parse, pour éviter qu'un
 * utilisateur voie un indicateur d'équilibre vert pour une saisie qui
 * sera ensuite rejetée côté backend.
 */

import Big from 'big.js';

/**
 * Format accepté : 1-15 chiffres entiers + optionnellement un séparateur
 * décimal suivi de 1 à 4 chiffres. Accepte virgule ou point ; la
 * normalisation en point se fait avant parse.
 *
 * P8 : exige au moins 1 chiffre après le séparateur pour rejeter les
 * formes ambigües comme `"100,"` (ni entier ni décimal complet).
 */
const AMOUNT_RE = /^\d{1,15}([.,]\d{1,4})?$/;

/** Retourne `true` si `raw` est vide ou conforme à AMOUNT_RE. */
export function isValidAmount(raw: string): boolean {
	if (raw === '') return true;
	return AMOUNT_RE.test(raw);
}

/** Parse un montant (string avec virgule ou point) en `Big`. Vide → 0. */
export function parseAmount(raw: string): Big {
	if (raw === '') return new Big(0);
	return new Big(raw.replace(',', '.'));
}

export interface BalanceResult {
	totalDebit: Big;
	totalCredit: Big;
	diff: Big;
	/** `true` uniquement si débits = crédits ET total > 0 ET tous les montants valides. */
	isBalanced: boolean;
	/** `true` si au moins une ligne contient une valeur hors format. */
	hasInvalidAmount: boolean;
}

export function computeBalance(
	lines: { debit: string; credit: string }[]
): BalanceResult {
	const hasInvalidAmount = lines.some(
		(l) => !isValidAmount(l.debit) || !isValidAmount(l.credit)
	);
	const totalDebit = lines.reduce(
		(acc, l) => acc.plus(isValidAmount(l.debit) ? parseAmount(l.debit) : new Big(0)),
		new Big(0)
	);
	const totalCredit = lines.reduce(
		(acc, l) => acc.plus(isValidAmount(l.credit) ? parseAmount(l.credit) : new Big(0)),
		new Big(0)
	);
	return {
		totalDebit,
		totalCredit,
		diff: totalDebit.minus(totalCredit),
		isBalanced: !hasInvalidAmount && totalDebit.eq(totalCredit) && totalDebit.gt(0),
		hasInvalidAmount
	};
}

/**
 * Classifie une ligne :
 *
 * - `empty` : ligne complètement vide (ignorée au submit).
 * - `partial` : quelque chose est saisi mais l'état n'est pas valide.
 * - `valid` : compte choisi + exactement un des deux montants > 0.
 */
export type LineStatus = 'empty' | 'partial' | 'valid';

/**
 * Formate un montant Big.js en notation suisse avec apostrophe comme
 * séparateur de milliers et point comme séparateur décimal, à 2 décimales.
 *
 * P9 : passer par `Big.toFixed(2)` puis découper manuellement évite la
 * conversion `Number(string)` qui perd de la précision au-delà de
 * `Number.MAX_SAFE_INTEGER ≈ 9×10¹⁵`. Pour une application comptable,
 * l'affichage doit rester exact même pour des montants invraisemblables.
 *
 * Exemples :
 * - `new Big('1234.5')` → `"1'234.50"`
 * - `new Big('99999999999999.99')` → `"99'999'999'999'999.99"` (exact)
 * - `new Big('-1234567.89')` → `"-1'234'567.89"`
 */
export function formatSwissAmount(big: Big): string {
	const fixed = big.toFixed(2);
	const negative = fixed.startsWith('-');
	const unsigned = negative ? fixed.slice(1) : fixed;
	const [intPart, decPart] = unsigned.split('.');
	// Insérer une apostrophe tous les 3 chiffres en partant de la droite.
	const withSeparators = intPart.replace(/\B(?=(\d{3})+(?!\d))/g, '\u2019');
	return `${negative ? '-' : ''}${withSeparators}.${decPart}`;
}

export function classifyLine(line: {
	accountId: number | null;
	debit: string;
	credit: string;
}): LineStatus {
	const emptyDebit = line.debit === '';
	const emptyCredit = line.credit === '';
	if (line.accountId === null && emptyDebit && emptyCredit) return 'empty';

	if (line.accountId === null) return 'partial';
	if (!isValidAmount(line.debit) || !isValidAmount(line.credit)) return 'partial';
	if (emptyDebit && emptyCredit) return 'partial';
	if (!emptyDebit && !emptyCredit) return 'partial';

	// Un seul des deux non vide
	const amount = emptyDebit ? parseAmount(line.credit) : parseAmount(line.debit);
	if (amount.lte(0)) return 'partial';

	return 'valid';
}
