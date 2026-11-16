/**
 * Story 37-3b (AC-1, AC-2) — calculs des lignes en devise côté écran.
 *
 * Jumeaux de `kesh-core` (`crates/kesh-core/src/accounting/fx.rs`, 37-3a AC-3) :
 * même arrondi (au centime, l'équidistant loin de zéro, `Big.roundHalfUp`
 * **explicite**, jamais `Big.RM` global), mêmes bornes, mêmes codes d'erreur.
 * Les deux côtés sont tenus égaux par les fixtures **partagées**
 * `crates/kesh-core/tests/fixtures/fx_rounding.json` et
 * `fx_rate_from_chf.json`, que lit le Vitest de ce module (aucune copie).
 *
 * - {@link computeChf} — `round(montant × cours, 2)` (`fx_chf` + étapes 2 et 3
 *   de `check_foreign_line`) ;
 * - {@link rateFromChf} — le cours déduit d'un CHF (`rate_from_chf`) ;
 * - {@link openingRateDate} — la date du cours d'ouverture proposé, jumelle de
 *   la fonction pure de 37-2 (fixture `fx_opening_dates.json`).
 *
 * Aucune I/O.
 */

import Big from 'big.js';
import { isValidAmount, readJournalAmount } from './balance';

/** Plus grand montant en devise, et plus grand CHF d'une ligne en devise : 10¹³. */
export const FX_AMOUNT_MAX = new Big('10000000000000');
/** Plus grand CHF d'une ligne en devise (37-3a AC-3). */
export const FX_CHF_MAX = FX_AMOUNT_MAX;
/** Plus grand cours admis (par unité) : 10⁶. */
export const FX_RATE_MAX = new Big('1000000');
/** Décimales au plus d'un montant en devise. */
export const FX_AMOUNT_MAX_DECIMALS = 4;
/** Décimales au plus d'un cours. */
export const FX_RATE_MAX_DECIMALS = 10;
/**
 * Pas d'ajustement de {@link rateFromChf} de part et d'autre du cours arrondi —
 * la même borne que `RATE_FROM_CHF_MAX_STEPS` de `kesh-core`.
 */
export const RATE_FROM_CHF_MAX_STEPS = 10;

/** Codes d'erreur des calculs en devise, ceux du serveur (37-3a AC-3). */
export type FxErrorCode = 'FX_VALUE_OUT_OF_RANGE' | 'FX_LINE_ROUNDS_TO_ZERO' | 'FX_AMOUNT_MISMATCH';

/** Résultat d'un calcul en devise. */
export type FxResult<T> = { ok: true; value: T } | { ok: false; error: FxErrorCode; expected?: Big };

/**
 * Constructeur `Big` propre au module pour la division de {@link rateFromChf} :
 * trente décimales, arrondi `roundHalfUp` — sans toucher aux réglages globaux.
 */
const BigDiv = Big();
BigDiv.DP = 30;
BigDiv.RM = Big.roundHalfUp;

/** Nombre de décimales d'un `Big` normalisé (zéros terminaux ignorés). */
function decimals(b: Big): number {
	// `c` : chiffres sans zéros terminaux ; `e` : exposant décimal du premier.
	return Math.max(0, b.c.length - b.e - 1);
}

/** Lit un décimal en `Big`, `null` s'il n'est pas lisible. */
function toBig(v: string | Big): Big | null {
	if (v instanceof Big) return v;
	try {
		return new Big(v.trim().replace(',', '.'));
	} catch {
		return null;
	}
}

/** Bornes d'un montant en devise : au plus 4 décimales, au plus 10¹³ en valeur absolue. */
function amountInBounds(a: Big): boolean {
	return decimals(a) <= FX_AMOUNT_MAX_DECIMALS && a.abs().lte(FX_AMOUNT_MAX);
}

/** Bornes d'un cours (`check_rate_bounds`) : > 0, au plus 10 décimales, au plus 10⁶. */
export function rateInBounds(r: Big): boolean {
	return r.gt(0) && decimals(r) <= FX_RATE_MAX_DECIMALS && r.lte(FX_RATE_MAX);
}

/**
 * Le CHF d'une ligne en devise : `round(montant × cours, 2)`, l'équidistant
 * loin de zéro. Refus, dans l'ordre de `check_foreign_line` (37-3a AC-3) :
 * `FX_VALUE_OUT_OF_RANGE` pour un montant hors bornes (4 décimales, 10¹³) ou
 * ≤ 0, un cours hors bornes (> 0, 10 décimales, 10⁶) ou un CHF arrondi > 10¹³ ;
 * `FX_LINE_ROUNDS_TO_ZERO` pour un CHF arrondi nul.
 */
export function computeChf(amount: string | Big, rate: string | Big): FxResult<Big> {
	const a = toBig(amount);
	const r = toBig(rate);
	if (a === null || r === null) return { ok: false, error: 'FX_VALUE_OUT_OF_RANGE' };
	if (!amountInBounds(a) || !rateInBounds(r)) return { ok: false, error: 'FX_VALUE_OUT_OF_RANGE' };
	// Le produit de `Big` est exact (aucune limite de chiffres significatifs) :
	// l'arrondi explicite au centime est le seul.
	const chf = a.times(r).round(2, Big.roundHalfUp);
	if (chf.abs().gt(FX_CHF_MAX)) return { ok: false, error: 'FX_VALUE_OUT_OF_RANGE' };
	if (a.lte(0)) return { ok: false, error: 'FX_VALUE_OUT_OF_RANGE' };
	if (chf.eq(0)) return { ok: false, error: 'FX_LINE_ROUNDS_TO_ZERO' };
	return { ok: true, value: chf };
}

/**
 * Le cours déduit d'un CHF (V1, CR-6), jumeau de `rate_from_chf` : CHF ÷
 * montant arrondi à 10 décimales ; s'il ne redonne pas le CHF
 * (`round(montant × cours, 2)`), ajusté au dernier chiffre, jusqu'à
 * {@link RATE_FROM_CHF_MAX_STEPS} pas de 10⁻¹⁰ de part et d'autre, le plus
 * proche d'abord, vers le haut avant le bas. Refus : `FX_VALUE_OUT_OF_RANGE`
 * (montant > 0 et dans ses bornes, CHF > 0 et ≤ 10¹³, cours obtenu dans ses
 * bornes), `FX_AMOUNT_MISMATCH` quand aucun cours ne convient (`expected` : le
 * CHF du cours arrondi).
 */
export function rateFromChf(amount: string | Big, chf: string | Big): FxResult<Big> {
	const a = toBig(amount);
	const c = toBig(chf);
	if (a === null || c === null) return { ok: false, error: 'FX_VALUE_OUT_OF_RANGE' };
	if (a.lte(0) || !amountInBounds(a)) return { ok: false, error: 'FX_VALUE_OUT_OF_RANGE' };
	if (c.lte(0) || c.gt(FX_CHF_MAX)) return { ok: false, error: 'FX_VALUE_OUT_OF_RANGE' };
	const base = new Big(new BigDiv(c).div(new BigDiv(a)).round(10, Big.roundHalfUp));
	if (!rateInBounds(base)) return { ok: false, error: 'FX_VALUE_OUT_OF_RANGE' };
	const first = computeChf(a, base);
	if (!first.ok && first.error === 'FX_VALUE_OUT_OF_RANGE') {
		return { ok: false, error: 'FX_VALUE_OUT_OF_RANGE' };
	}
	const expected = first.ok ? first.value : new Big(0);
	if (first.ok && first.value.eq(c)) return { ok: true, value: base };
	const step = new Big('0.0000000001');
	for (let k = 1; k <= RATE_FROM_CHF_MAX_STEPS; k++) {
		const delta = step.times(k);
		for (const candidate of [base.plus(delta), base.minus(delta)]) {
			if (!rateInBounds(candidate)) continue;
			const got = computeChf(a, candidate);
			if (got.ok && got.value.eq(c)) return { ok: true, value: candidate };
		}
	}
	return { ok: false, error: 'FX_AMOUNT_MISMATCH', expected };
}

/** Lecture d'un cours saisi : `null` hors format ou hors bornes, `'empty'` pour le vide. */
export function readRate(raw: string): Big | null | 'empty' {
	const t = raw.trim();
	if (t === '') return 'empty';
	if (!isValidAmount(t, { maxDecimals: FX_RATE_MAX_DECIMALS, maxIntegerDigits: 7, allowEmpty: false })) {
		return null;
	}
	const r = new Big(t.replace(',', '.'));
	return rateInBounds(r) ? r : null;
}

/**
 * Lecture d'un montant en devise saisi, par la lecture du journal
 * (`readJournalAmount` : apostrophe des milliers admise, « 1.234 » ambigu,
 * quatre décimales) puis la borne de 10¹³ : `null` hors format, ambigu ou hors
 * bornes, `'empty'` pour le vide.
 */
export function readForeignAmount(raw: string): Big | null | 'empty' {
	const r = readJournalAmount(raw);
	if (r.kind === 'empty') return 'empty';
	if (r.kind !== 'clear') return null;
	return r.value.lte(FX_AMOUNT_MAX) ? r.value : null;
}

/** Le jour précédent d'une date `AAAA-MM-JJ`. */
function previousDay(iso: string): string {
	// Arithmétique de calendrier pure, en UTC (aucune date du jour, aucun fuseau).
	const d = new Date(Date.UTC(Number(iso.slice(0, 4)), Number(iso.slice(5, 7)) - 1, Number(iso.slice(8, 10)) - 1));
	const p = (n: number) => String(n).padStart(2, '0');
	return `${d.getUTCFullYear()}-${p(d.getUTCMonth() + 1)}-${p(d.getUTCDate())}`;
}

/** Le 1er du mois d'une date `AAAA-MM-JJ`. */
export function firstOfMonth(iso: string): string {
	return `${iso.slice(0, 7)}-01`;
}

/** Genre d'un cours : mensuel AFC ou journalier. */
export type RateKind = 'day' | 'month';

/**
 * Date du cours d'**ouverture** proposé d'un exercice qui commence le
 * `startDate` (T4, CR-4 — le cours d'ouverture est le cours de clôture de
 * l'exercice précédent), jumelle de la fonction pure de 37-2 :
 * - genre `day` : la veille de l'ouverture ;
 * - genre `month` : le 1er du mois de cette veille (le dernier mois de
 *   l'exercice précédent ; pour une ouverture en cours de mois, le mois de
 *   l'ouverture lui-même — 15.03 → mars).
 *
 * La règle se calcule sur la seule date d'ouverture.
 */
export function openingRateDate(startDate: string, kind: RateKind): string {
	const eve = previousDay(startDate);
	return kind === 'day' ? eve : firstOfMonth(eve);
}
