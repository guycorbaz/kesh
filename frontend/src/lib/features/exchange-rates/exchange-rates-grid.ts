/**
 * Story 37-2 (AC-6) — règles pures de l'écran *Cours de change* : mentions et ↑
 * d'une case, décision à la validation d'un champ, colonnes du tableau annuel
 * (liaison d'Ouverture et de Clôture), colonnes à récupérer, choix de l'exercice.
 *
 * Aucune constante de douze mois : les colonnes se lisent dans `months` de la
 * réponse du serveur, quelle que soit la durée de l'exercice (S4).
 */

import Big from 'big.js';
import { isValidAmount, type AmountFormatOptions } from '$lib/features/journal-entries/balance';
import { formatSwissDate } from '$lib/features/reports/reports.api';
import type { FiscalYearResponse } from '$lib/features/fiscal-years/fiscal-years.types';
import type {
	Cell,
	FetchRatesRequest,
	FiscalYearRatesResponse,
	FiscalYearRatesRow,
	RateKind,
} from './exchange-rates.types';

/** Bornes de format d'un cours saisi (AC-2, comme 37-3b AC-2). */
export const RATE_FORMAT: AmountFormatOptions = {
	maxDecimals: 10,
	maxIntegerDigits: 7,
	allowEmpty: false,
};

/** Plus grand cours admis (AC-2) : 1 000 000 CHF par unité. */
const MAX_RATE = new Big(1_000_000);

/** Première unité récupérable (bornes de l'AC-5). */
export const FIRST_FETCHABLE_DATE = '2000-01-01';

/** Lit un cours décimal ; `null` s'il n'en est pas un. */
function big(s: string | null | undefined): Big | null {
	if (s === null || s === undefined || s.trim() === '') return null;
	try {
		return new Big(s.replace(',', '.'));
	} catch {
		return null;
	}
}

/** Vrai si deux cours sont numériquement égaux (`0.885` = `0.8850000000`). */
export function sameRate(a: string | null | undefined, b: string | null | undefined): boolean {
	const x = big(a);
	const y = big(b);
	return x !== null && y !== null && x.eq(y);
}

/**
 * Affichage d'un cours : les zéros de fin retirés, quatre décimales au moins
 * (`0.8850000000` → `0.8850`, `0.0056780000` → `0.005678`).
 */
export function formatRate(s: string | null | undefined): string {
	const x = big(s);
	if (x === null) return '';
	const fixed = x.toFixed(10);
	const [entier, fraction = ''] = fixed.split('.');
	let f = fraction.replace(/0+$/, '');
	if (f.length < 4) f = f.padEnd(4, '0');
	return `${entier}.${f}`;
}

/**
 * Vrai si la saisie est un cours admissible : format de l'AC-2 (dix décimales,
 * sept chiffres entiers, virgule ou point), strictement positif, au plus 1 000 000.
 */
export function isValidRateInput(raw: string): boolean {
	const s = raw.trim();
	if (!isValidAmount(s, RATE_FORMAT)) return false;
	const x = big(s);
	return x !== null && x.gt(0) && x.lte(MAX_RATE);
}

/** La saisie normalisée pour le serveur (virgule → point, blancs retirés). */
export function normalizeRateInput(raw: string): string {
	return raw.trim().replace(',', '.');
}

/** Le ↑ d'une case et son libellé accessible. */
export type ArrowKind = 'corrected' | 'published' | 'remove';

/** La mention sous la valeur d'une case (U1, A2, A30). */
export type MentionKind =
	| 'afc-ok'
	| 'afc-corrected'
	| 'afc-unverified'
	| 'saisi-differs'
	| 'saisi-equal'
	| 'saisi-unpublished'
	| 'none-published'
	| 'none';

/** Mention calculée d'une case. */
export interface Mention {
	kind: MentionKind;
	/** La valeur publiée par unité, affichée et renvoyée au ↑. */
	published: string | null;
	/** Date de la correction (`valueChangedAt`) pour `afc-corrected`. */
	correctedAt: string | null;
	/** La source brute d'une ligne qui n'est ni `afc` ni `manual` (A30), entre parenthèses. */
	otherSource: string | null;
	/** Le ↑ offert, ou `null`. */
	arrow: ArrowKind | null;
}

/** La mention et le ↑ d'une case, selon sa ligne et la valeur publiée (AC-6). */
export function mentionOf(cell: Cell): Mention {
	const published = cell.published?.rate ?? null;
	const base = { published, correctedAt: null, otherSource: null };
	if (cell.effectiveRate === null || cell.source === null) {
		return published !== null
			? { ...base, kind: 'none-published', arrow: 'published' }
			: { ...base, kind: 'none', arrow: null };
	}
	if (cell.source === 'afc') {
		if (published === null) return { ...base, kind: 'afc-unverified', arrow: null };
		if (cell.corrected || !sameRate(cell.effectiveRate, published)) {
			return {
				...base,
				kind: 'afc-corrected',
				correctedAt: cell.published?.valueChangedAt ?? null,
				arrow: 'corrected',
			};
		}
		return { ...base, kind: 'afc-ok', arrow: null };
	}
	const otherSource = cell.source === 'manual' ? null : cell.source;
	if (published === null) return { ...base, otherSource, kind: 'saisi-unpublished', arrow: 'remove' };
	if (sameRate(cell.effectiveRate, published)) {
		return { ...base, otherSource, kind: 'saisi-equal', arrow: 'published' };
	}
	return { ...base, otherSource, kind: 'saisi-differs', arrow: 'published' };
}

/** Ce que fait la validation d'un champ (AC-6, *Cours éditable*). */
export type CommitDecision =
	| { action: 'none' }
	| { action: 'restore' }
	| { action: 'invalid' }
	| { action: 'reset' }
	| { action: 'put'; rate: string };

/**
 * Décision à la validation d'un champ :
 * - champ vidé → la valeur affichée est restaurée, sans appel (seul ↑ retire) ;
 * - saisie illisible ou hors bornes → refusée à l'écran ;
 * - valeur égale au cours affiché → rien ;
 * - valeur égale à la valeur publiée → le ↑ (`reset`), jamais un `PUT` : l'écran ne
 *   crée jamais une ligne `manual` égale à la valeur publiée (A2) ;
 * - sinon → `PUT` du cours saisi.
 */
export function commitDecision(cell: Cell, raw: string): CommitDecision {
	if (raw.trim() === '') return { action: 'restore' };
	if (!isValidRateInput(raw)) return { action: 'invalid' };
	const rate = normalizeRateInput(raw);
	if (sameRate(rate, cell.effectiveRate)) return { action: 'none' };
	if (cell.published && sameRate(rate, cell.published.rate)) return { action: 'reset' };
	return { action: 'put', rate };
}

/** Le mois suivant d'un 1er du mois `AAAA-MM-01`. */
function addMonths(iso: string, n: number): string {
	const [y, m] = iso.split('-').map(Number);
	const d = new Date(Date.UTC(y, m - 1 + n, 1));
	return `${String(d.getUTCFullYear()).padStart(4, '0')}-${String(d.getUTCMonth() + 1).padStart(2, '0')}-01`;
}

/** Le lendemain d'une date pure `AAAA-MM-JJ`, calculé sur le calendrier, sans fuseau. */
export function dayAfter(iso: string): string {
	const [y, m, d] = iso.split('-').map(Number);
	const t = new Date(Date.UTC(y, m - 1, d + 1));
	return `${String(t.getUTCFullYear()).padStart(4, '0')}-${String(t.getUTCMonth() + 1).padStart(2, '0')}-${String(t.getUTCDate()).padStart(2, '0')}`;
}

/** Le 1er du mois d'une date. */
export function monthStart(iso: string): string {
	return `${iso.slice(0, 7)}-01`;
}

/** « juil. 2024 » : mois abrégé et année, dans la locale de l'interface. */
export function monthLabel(iso: string, locale: string): string {
	const [y, m] = iso.split('-').map(Number);
	return new Intl.DateTimeFormat(locale, { month: 'short', year: 'numeric', timeZone: 'UTC' }).format(
		new Date(Date.UTC(y, m - 1, 1)),
	);
}

/** « juillet 2024 » : mois en toutes lettres et année. */
export function monthLongLabel(iso: string, locale: string): string {
	const [y, m] = iso.split('-').map(Number);
	return new Intl.DateTimeFormat(locale, { month: 'long', year: 'numeric', timeZone: 'UTC' }).format(
		new Date(Date.UTC(y, m - 1, 1)),
	);
}

/** L'en-tête d'une colonne : la date du cours (mois abrégé ou JJ.MM.AAAA). */
export function unitLabel(kind: RateKind, rateDate: string, locale: string): string {
	return kind === 'month' ? monthLabel(rateDate, locale) : formatSwissDate(rateDate);
}

/** Une colonne du tableau. */
export interface Column {
	/** Clé de l'unité (`month:2024-07-01`, `day:2024-06-30`) : une colonne par unité. */
	key: string;
	kind: RateKind;
	rateDate: string;
	slot: 'month' | 'opening' | 'closing';
	/** Rang dans `months` pour une colonne de mois. */
	index: number;
}

/** La clé d'une unité. */
export function unitKey(kind: RateKind, rateDate: string): string {
	return `${kind}:${rateDate}`;
}

/**
 * Vrai si la case d'Ouverture ou de Clôture est **la même case** qu'une colonne de
 * mois de l'exercice (genre `month` : Clôture = dernier mois ; Ouverture d'un
 * exercice commencé en cours de mois = premier mois). Elle s'édite alors à un seul
 * endroit, la colonne du mois.
 */
export function isLinkedToMonth(kind: RateKind, rateDate: string, months: readonly string[]): boolean {
	return kind === 'month' && months.includes(rateDate);
}

/** Les colonnes du tableau : les mois, puis Ouverture et Clôture. */
export function gridColumns(grid: FiscalYearRatesResponse): {
	months: Column[];
	opening: Column;
	closing: Column;
} {
	const months = grid.months.map((m, index) => ({
		key: unitKey('month', m),
		kind: 'month' as RateKind,
		rateDate: m,
		slot: 'month' as const,
		index,
	}));
	const opening: Column = {
		key: unitKey(grid.openingRateKind, grid.openingDate),
		kind: grid.openingRateKind,
		rateDate: grid.openingDate,
		slot: 'opening',
		index: -1,
	};
	const closing: Column = {
		key: unitKey(grid.closingRateKind, grid.closingDate),
		kind: grid.closingRateKind,
		rateDate: grid.closingDate,
		slot: 'closing',
		index: -1,
	};
	return { months, opening, closing };
}

/** La case d'une ligne pour une colonne. */
export function cellAt(row: FiscalYearRatesRow, column: Column): Cell {
	if (column.slot === 'opening') return row.opening;
	if (column.slot === 'closing') return row.closing;
	return row.months[column.index];
}

/** Une case vide (ni ligne ni valeur publiée). */
export function emptyCell(kind: RateKind, rateDate: string): Cell {
	return {
		kind,
		rateDate,
		effectiveRate: null,
		source: null,
		row: null,
		published: null,
		corrected: false,
		acknowledged: false,
	};
}

/** La ligne d'une devise ajoutée à l'écran, sans case côté serveur. */
export function emptyRow(currency: string, grid: FiscalYearRatesResponse): FiscalYearRatesRow {
	return {
		currency,
		hasAccount: false,
		months: grid.months.map((m) => emptyCell('month', m)),
		opening: emptyCell(grid.openingRateKind, grid.openingDate),
		closing: emptyCell(grid.closingRateKind, grid.closingDate),
	};
}

/** Pourquoi une colonne n'est pas récupérable. */
export type NotFetchableReason = 'future' | 'before-2000';

/**
 * La récupérabilité d'une unité (bornes de l'AC-5) : un mois au plus tard le mois
 * suivant le mois courant (Europe/Zurich), un jour strictement passé, jamais avant
 * le 1er janvier 2000. `today` = `todayInZurich()`.
 */
export function fetchability(
	kind: RateKind,
	rateDate: string,
	today: string,
): { ok: true } | { ok: false; reason: NotFetchableReason } {
	if (rateDate < FIRST_FETCHABLE_DATE) return { ok: false, reason: 'before-2000' };
	if (kind === 'month') {
		return rateDate <= addMonths(monthStart(today), 1) ? { ok: true } : { ok: false, reason: 'future' };
	}
	return rateDate < today ? { ok: true } : { ok: false, reason: 'future' };
}

/** Le corps du `POST /fetch` d'une colonne. */
export function fetchRequestOf(column: Pick<Column, 'kind' | 'rateDate'>): FetchRatesRequest {
	if (column.kind === 'month') {
		const [y, m] = column.rateDate.split('-').map(Number);
		return { year: y, month: m };
	}
	return { date: column.rateDate };
}

/**
 * Les colonnes distinctes du tableau, sans les cases liées (une Ouverture liée à la
 * première colonne, une Clôture à la dernière ne comptent qu'une fois).
 */
export function distinctColumns(grid: FiscalYearRatesResponse): Column[] {
	const { months, opening, closing } = gridColumns(grid);
	const seen = new Set<string>();
	const out: Column[] = [];
	for (const c of [...months, opening, closing]) {
		if (seen.has(c.key)) continue;
		seen.add(c.key);
		out.push(c);
	}
	return out;
}

/**
 * Vrai si une colonne a un cours manquant : une ligne affichée sans valeur publiée,
 * ou, pour une devise portée par un compte (`hasAccount`), sans cours retenu (A24).
 */
export function columnHasMissing(rows: readonly FiscalYearRatesRow[], column: Column): boolean {
	return rows.some((r) => {
		const c = cellAt(r, column);
		return c.published === null || (r.hasAccount && c.effectiveRate === null);
	});
}

/** « Récupérer les cours manquants » : un `POST` par colonne récupérable qui a un manque. */
export function columnsToFetch(
	grid: FiscalYearRatesResponse,
	rows: readonly FiscalYearRatesRow[],
	today: string,
): Column[] {
	return distinctColumns(grid).filter(
		(c) => fetchability(c.kind, c.rateDate, today).ok && columnHasMissing(rows, c),
	);
}

/** « Revérifier les cours publiés » : un `POST` par colonne récupérable déjà présente. */
export function columnsToRecheck(
	grid: FiscalYearRatesResponse,
	rows: readonly FiscalYearRatesRow[],
	today: string,
): Column[] {
	return distinctColumns(grid).filter(
		(c) =>
			fetchability(c.kind, c.rateDate, today).ok &&
			rows.some((r) => cellAt(r, c).published !== null),
	);
}

/** Vrai si l'exercice couvre la date. */
function covers(fy: FiscalYearResponse, date: string): boolean {
	return fy.startDate <= date && date <= fy.endDate;
}

/**
 * L'exercice affiché par défaut parmi les exercices multi-devises : celui qui
 * couvre `date` (lien du bandeau) s'il est donné, sinon celui qui couvre
 * aujourd'hui, sinon le plus récent. `null` sans exercice multi-devises.
 */
export function defaultFiscalYear(
	fiscalYears: readonly FiscalYearResponse[],
	today: string,
	date: string | null = null,
): FiscalYearResponse | null {
	const multi = fiscalYears.filter((f) => f.multiCurrency);
	if (multi.length === 0) return null;
	if (date) {
		const d = multi.find((f) => covers(f, date));
		if (d) return d;
	}
	const now = multi.find((f) => covers(f, today));
	if (now) return now;
	return [...multi].sort((a, b) => (a.startDate < b.startDate ? 1 : -1))[0];
}

/** L'exercice suivant : celui qui commence le lendemain de la clôture (un trou rompt le lien). */
export function nextFiscalYearOf(
	fy: Pick<FiscalYearResponse, 'endDate'>,
	fiscalYears: readonly FiscalYearResponse[],
): FiscalYearResponse | null {
	const start = dayAfter(fy.endDate);
	return fiscalYears.find((f) => f.startDate === start) ?? null;
}

/** Les exercices multi-devises, du plus récent au plus ancien (sélecteur). */
export function multiCurrencyFiscalYears(fiscalYears: readonly FiscalYearResponse[]): FiscalYearResponse[] {
	return fiscalYears
		.filter((f) => f.multiCurrency)
		.sort((a, b) => (a.startDate < b.startDate ? 1 : -1));
}
