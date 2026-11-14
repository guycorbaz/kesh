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
 * Options de format d'un montant — Story v016-2b (#816) : le relevé saisi à la
 * main contrôle ses soldes (signés, requis, treize chiffres, deux décimales) et
 * ses montants de ligne (deux décimales, treize chiffres) par la **même**
 * expression que le journal, paramétrée, et non par une seconde expression.
 * Les défauts sont ceux du journal : `DECIMAL(19,4)`, quinze chiffres entiers,
 * sans signe, vide admis.
 */
export interface AmountFormatOptions {
	/** Décimales au plus (défaut 4). */
	maxDecimals?: number;
	/** Chiffres entiers au plus (défaut 15). */
	maxIntegerDigits?: number;
	/** Admet un `-` en tête (défaut non). */
	allowNegative?: boolean;
	/** La chaîne vide est valide (défaut oui, comme le journal). */
	allowEmpty?: boolean;
}

const AMOUNT_FORMAT_DEFAULTS: Required<AmountFormatOptions> = {
	maxDecimals: 4,
	maxIntegerDigits: 15,
	allowNegative: false,
	allowEmpty: true,
};

/**
 * L'expression d'un format, la seule du module, paramétrée par ses options. Par
 * défaut : 1-15 chiffres entiers + optionnellement un séparateur décimal suivi de
 * 1 à 4 chiffres. Accepte virgule ou point ; la normalisation en point se fait
 * avant parse.
 *
 * P8 : exige au moins 1 chiffre après le séparateur pour rejeter les
 * formes ambigües comme `"100,"` (ni entier ni décimal complet).
 */
function amountRegExp(o: Required<AmountFormatOptions>): RegExp {
	const sign = o.allowNegative ? '-?' : '';
	return new RegExp(`^${sign}\\d{1,${o.maxIntegerDigits}}([.,]\\d{1,${o.maxDecimals}})?$`);
}

const AMOUNT_RE = amountRegExp(AMOUNT_FORMAT_DEFAULTS);

/**
 * Retourne `true` si `raw` est conforme au format — par défaut celui du journal
 * (vide admis, 1-15 chiffres, 1-4 décimales, virgule ou point, sans signe).
 */
export function isValidAmount(raw: string, options?: AmountFormatOptions): boolean {
	const o = options ? { ...AMOUNT_FORMAT_DEFAULTS, ...options } : AMOUNT_FORMAT_DEFAULTS;
	if (raw === '') return o.allowEmpty;
	return (options ? amountRegExp(o) : AMOUNT_RE).test(raw);
}

/** Parse un montant (string avec virgule ou point) en `Big`. Vide → 0. */
export function parseAmount(raw: string): Big {
	if (raw === '') return new Big(0);
	return new Big(raw.replace(',', '.'));
}

/** Story 36-3 — le classement d'une saisie du journal. */
export type JournalAmountKind = 'empty' | 'clear' | 'ambiguous' | 'unreadable';

/** Story 36-3 — la lecture d'une saisie du journal ; `value` et `payload` pour une saisie claire. */
export type JournalAmountReading =
	| { kind: 'clear'; value: Big; payload: string }
	| { kind: 'empty' | 'ambiguous' | 'unreadable' };

/** Story 36-3 (stub de la preuve du rouge) — la lecture d'aujourd'hui. */
export function readJournalAmount(raw: string): JournalAmountReading {
	if (raw === '') return { kind: 'empty' };
	if (!isValidAmount(raw)) return { kind: 'unreadable' };
	return { kind: 'clear', value: parseAmount(raw), payload: raw.replace(',', '.') };
}

/** Story 36-3 (stub) — le problème de montant d'une ligne. */
export function lineAmountProblem(
	_debit: string,
	_credit: string
): 'ambiguous' | 'unreadable' | null {
	return null;
}

/** Story 36-3 (D3) — un lecteur de montant : `null` hors format, `Big(0)` pour le vide. */
export type AmountReader = (raw: string) => Big | null;

/** Story 36-3 (D3) — la lecture d'avant la story, que gardent les soldes de départ (#1050). */
export const defaultAmountReader: AmountReader = (raw) =>
	isValidAmount(raw) ? parseAmount(raw) : null;

/** Story 36-3 (D3) — la lecture du journal (`readJournalAmount`). */
export const journalAmountReader: AmountReader = (raw) => {
	const r = readJournalAmount(raw);
	if (r.kind === 'empty') return new Big(0);
	return r.kind === 'clear' ? r.value : null;
};

export interface BalanceResult {
	totalDebit: Big;
	totalCredit: Big;
	diff: Big;
	/** `true` uniquement si débits = crédits ET total > 0 ET tous les montants valides. */
	isBalanced: boolean;
	/** `true` si au moins une ligne contient une valeur hors format. */
	hasInvalidAmount: boolean;
}

/**
 * Totaux et équilibre des lignes, chaque montant lu par `read` (Story 36-3, D3) :
 * par défaut `defaultAmountReader`, la lecture d'avant la story (soldes de départ,
 * #1050) ; l'écriture manuelle passe `journalAmountReader`. Un montant que le
 * lecteur rend `null` compte pour zéro et rend `hasInvalidAmount` vrai.
 */
export function computeBalance(
	lines: { debit: string; credit: string }[],
	read: AmountReader = defaultAmountReader
): BalanceResult {
	const read0 = (raw: string) => read(raw) ?? new Big(0);
	const hasInvalidAmount = lines.some((l) => read(l.debit) === null || read(l.credit) === null);
	const totalDebit = lines.reduce((acc, l) => acc.plus(read0(l.debit)), new Big(0));
	const totalCredit = lines.reduce((acc, l) => acc.plus(read0(l.credit)), new Big(0));
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
export function formatSwissAmount(big: Big, decimals = 2): string {
	// Story 25-4-d2b : `decimals` (défaut 2) — un reste à fraction de centime
	// (0.0040) s'affiche aux quatre décimales plutôt qu'en « 0.00 ».
	const fixed = big.toFixed(decimals);
	const negative = fixed.startsWith('-');
	const unsigned = negative ? fixed.slice(1) : fixed;
	const [intPart, decPart] = unsigned.split('.');
	// Insérer une apostrophe tous les 3 chiffres en partant de la droite.
	const withSeparators = intPart.replace(/\B(?=(\d{3})+(?!\d))/g, '\u2019');
	// Story 33-3b : `toFixed(0)` n'a pas de point, donc pas de partie décimale.
	const fraction = decPart === undefined ? '' : `.${decPart}`;
	return `${negative ? '-' : ''}${withSeparators}${fraction}`;
}

/**
 * Classe une ligne, ses montants lus par `read` (Story 36-3, D3 ; défaut :
 * `defaultAmountReader`). ⚠️ Ne pas passer cette fonction telle quelle à
 * `Array.prototype.map` : `map` donnerait l'indice pour lecteur. Écrire
 * `lines.map((l) => classifyLine(l, journalAmountReader))`.
 */
export function classifyLine(
	line: {
		accountId: number | null;
		debit: string;
		credit: string;
	},
	read: AmountReader = defaultAmountReader
): LineStatus {
	const emptyDebit = line.debit === '';
	const emptyCredit = line.credit === '';
	if (line.accountId === null && emptyDebit && emptyCredit) return 'empty';

	if (line.accountId === null) return 'partial';
	const debit = read(line.debit);
	const credit = read(line.credit);
	if (debit === null || credit === null) return 'partial';
	if (emptyDebit && emptyCredit) return 'partial';
	if (!emptyDebit && !emptyCredit) return 'partial';

	// Un seul des deux non vide
	const amount = emptyDebit ? credit : debit;
	if (amount.lte(0)) return 'partial';

	return 'valid';
}
