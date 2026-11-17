/**
 * Story 37-2 — fabriques de test de l'écran *Cours de change* : cases, lignes et
 * réponses du tableau annuel, construites comme le serveur les rend (colonnes du
 * mois de début au mois de fin, Ouverture = veille du début ou 1er de son mois,
 * Clôture = fin ou 1er de son mois).
 *
 * ⚠️ Module de test : il n'est importé que par des `*.test.ts`.
 */
import type { FiscalYearResponse } from '$lib/features/fiscal-years/fiscal-years.types';
import { dayBefore } from '$lib/shared/utils/local-date';
import { emptyCell, monthStart } from './exchange-rates-grid';
import type { Cell, FiscalYearRatesResponse, FiscalYearRatesRow, RateKind } from './exchange-rates.types';

/** Les mois `AAAA-MM-01` de `start` à `end` inclus. */
export function monthsBetween(start: string, end: string): string[] {
	const out: string[] = [];
	let [y, m] = start.split('-').map(Number);
	const fin = monthStart(end);
	for (;;) {
		const iso = `${String(y).padStart(4, '0')}-${String(m).padStart(2, '0')}-01`;
		out.push(iso);
		if (iso >= fin) break;
		m += 1;
		if (m > 12) {
			m = 1;
			y += 1;
		}
	}
	return out;
}

/** Une case : vide par défaut, puis surchargée. */
export function cell(kind: RateKind, rateDate: string, over: Partial<Cell> = {}): Cell {
	return { ...emptyCell(kind, rateDate), ...over };
}

/** Une ligne `afc` copiée par Kesh. */
export function afcRow(version = 1, id = 10) {
	return { id, version, updatedAt: '2025-01-05T08:00:00Z', updatedByKesh: true };
}

/** Une valeur publiée par unité. */
export function published(rate: string, over: Partial<NonNullable<Cell['published']>> = {}) {
	return {
		rate,
		publishedUnit: 1,
		publishedRate: rate,
		sourceDate: null,
		fetchedAt: '2025-01-05T08:00:00Z',
		valueChangedAt: '2024-12-03T09:00:00Z',
		...over,
	};
}

/** Le tableau annuel d'un exercice, ses lignes construites par `rowOf`. */
export function yearGrid(
	startDate: string,
	endDate: string,
	opening: RateKind = 'day',
	closing: RateKind = 'day',
	rowOf: (g: FiscalYearRatesResponse) => FiscalYearRatesRow[] = () => [],
): FiscalYearRatesResponse {
	const veille = dayBefore(startDate);
	const g: FiscalYearRatesResponse = {
		fiscalYearId: 1,
		startDate,
		endDate,
		openingRateKind: opening,
		closingRateKind: closing,
		openingDate: opening === 'day' ? veille : monthStart(veille),
		closingDate: closing === 'day' ? endDate : monthStart(endDate),
		months: monthsBetween(startDate, endDate),
		rows: [],
	};
	g.rows = rowOf(g);
	return g;
}

/**
 * Une ligne de devise dont les cases sont vides, sauf celles que `cells` surcharge
 * (clé : `month:<date>`, `opening`, `closing`). Une case d'Ouverture ou de Clôture
 * liée à une colonne de mois reprend la case de ce mois, comme au serveur.
 */
export function row(
	g: FiscalYearRatesResponse,
	currency: string,
	cells: Record<string, Partial<Cell>> = {},
	hasAccount = true,
): FiscalYearRatesRow {
	const months = g.months.map((m) => cell('month', m, cells[`month:${m}`] ?? {}));
	const byDate = (kind: RateKind, date: string, slot: string) => {
		if (kind === 'month' && g.months.includes(date)) return months[g.months.indexOf(date)];
		return cell(kind, date, cells[slot] ?? {});
	};
	return {
		currency,
		hasAccount,
		months,
		opening: byDate(g.openingRateKind, g.openingDate, 'opening'),
		closing: byDate(g.closingRateKind, g.closingDate, 'closing'),
	};
}

/** Un exercice comptable. */
export function fy(over: Partial<FiscalYearResponse> = {}): FiscalYearResponse {
	return {
		id: 1,
		companyId: 1,
		name: '2024/25',
		startDate: '2024-07-01',
		endDate: '2025-06-30',
		status: 'Open',
		multiCurrency: true,
		openingRateKind: 'day',
		closingRateKind: 'day',
		createdAt: '2024-07-01T00:00:00Z',
		updatedAt: '2024-07-01T00:00:00Z',
		...over,
	};
}
