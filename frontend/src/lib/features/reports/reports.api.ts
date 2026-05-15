// Story 9-1 — Client API rapports comptables (4 endpoints GET /api/v1/reports/*).

import Big from 'big.js';
import { apiClient } from '$lib/shared/utils/api-client';
import { formatSwissAmount } from '$lib/features/journal-entries/balance';
import type {
	BalanceSheetDto,
	IncomeStatementDto,
	JournalReportDto,
	JournalReportQuery,
	ReportQuery,
	ReportType,
	TrialBalanceDto,
} from './reports.types';

function buildQuery(params: Record<string, string | number | undefined>): string {
	const sp = new URLSearchParams();
	for (const [key, val] of Object.entries(params)) {
		if (val !== undefined && val !== null && val !== '') {
			sp.set(key, String(val));
		}
	}
	return sp.toString();
}

export async function getBalanceSheet(query: ReportQuery): Promise<BalanceSheetDto> {
	const qs = buildQuery({
		fiscalYearId: query.fiscalYearId,
		periodStart: query.periodStart,
		periodEnd: query.periodEnd,
	});
	return apiClient.get<BalanceSheetDto>(`/api/v1/reports/balance-sheet?${qs}`);
}

export async function getIncomeStatement(query: ReportQuery): Promise<IncomeStatementDto> {
	const qs = buildQuery({
		fiscalYearId: query.fiscalYearId,
		periodStart: query.periodStart,
		periodEnd: query.periodEnd,
	});
	return apiClient.get<IncomeStatementDto>(`/api/v1/reports/income-statement?${qs}`);
}

export async function getTrialBalance(query: ReportQuery): Promise<TrialBalanceDto> {
	const qs = buildQuery({
		fiscalYearId: query.fiscalYearId,
		periodStart: query.periodStart,
		periodEnd: query.periodEnd,
	});
	return apiClient.get<TrialBalanceDto>(`/api/v1/reports/trial-balance?${qs}`);
}

export async function getJournalReport(query: JournalReportQuery): Promise<JournalReportDto> {
	const qs = buildQuery({
		fiscalYearId: query.fiscalYearId,
		periodStart: query.periodStart,
		periodEnd: query.periodEnd,
		journal: query.journal,
	});
	return apiClient.get<JournalReportDto>(`/api/v1/reports/journals?${qs}`);
}

/**
 * Helper Pass 1 AA-11 / BH-11 : détecte si un rapport est "vide" pour afficher
 * le message UX `reports-error-no-entries-in-period` (UX-DR38).
 */
export function isReportEmpty(
	type: ReportType,
	dto:
		| BalanceSheetDto
		| IncomeStatementDto
		| TrialBalanceDto
		| JournalReportDto
		| null
		| undefined,
): boolean {
	if (!dto) return true;
	switch (type) {
		case 'balance-sheet': {
			const bs = dto as BalanceSheetDto;
			return bs.assets.length === 0 && bs.liabilities.length === 0;
		}
		case 'income-statement': {
			const is_ = dto as IncomeStatementDto;
			return is_.revenues.length === 0 && is_.expenses.length === 0;
		}
		case 'trial-balance': {
			const tb = dto as TrialBalanceDto;
			return tb.rows.length === 0;
		}
		case 'journals': {
			const jr = dto as JournalReportDto;
			return jr.journals.every((s) => s.entries.length === 0);
		}
	}
}

/** Formate une date ISO YYYY-MM-DD en suisse dd.mm.yyyy. */
export function formatSwissDate(iso: string): string {
	const parts = iso.split('-');
	if (parts.length !== 3) return iso;
	return `${parts[2]}.${parts[1]}.${parts[0]}`;
}

/**
 * Formate un montant décimal suisse — wrapper sûr autour de `formatSwissAmount`.
 * Si la valeur ne peut être parsée (NaN, vide, malformée), retourne la chaîne brute.
 * Pass 1 code review patch P22 — DRY (anciennement dupliqué dans les 4 vues).
 */
export function formatReportAmount(v: string): string {
	try {
		return formatSwissAmount(new Big(v));
	} catch {
		return v;
	}
}

// ============================================================================
// Story 9-2a — Export PDF & CSV
// ============================================================================

/**
 * Construit l'URL d'un endpoint d'export `/api/v1/reports/{type}/export?format=...`.
 * Pour le rapport `journals`, le paramètre optionnel `journal` est inclus si fourni.
 */
export function getReportExportUrl(
	type: ReportType,
	query: ReportQuery | JournalReportQuery,
	format: 'pdf' | 'csv',
): string {
	const params: Record<string, string | number | undefined> = {
		fiscalYearId: query.fiscalYearId,
		periodStart: query.periodStart,
		periodEnd: query.periodEnd,
		format,
	};
	if (type === 'journals') {
		params.journal = (query as JournalReportQuery).journal;
	}
	const qs = buildQuery(params);
	return `/api/v1/reports/${type}/export?${qs}`;
}

/**
 * Construit le filename `kesh-{typeSlug}-{companySlug}-{periodStart}_{periodEnd}.{ext}`
 * (AC #22 + Pass 1 ECH-H3).
 *
 * Pipeline slug ASCII : NFD-strip diacritics + lowercase + non-alphanum → `-` +
 * collapse `-+` → `-` + truncate 20 chars + strip trailing `-`. Fallback
 * `'company'` si vide post-slug (e.g. nom CJK).
 */
export function buildExportFilename(
	type: ReportType,
	companyName: string,
	period: { start: string; end: string },
	format: 'pdf' | 'csv',
): string {
	const typeSlug = TYPE_SLUGS[type];
	const companySlug = slugify(companyName, 'company');
	return `kesh-${typeSlug}-${companySlug}-${period.start}_${period.end}.${format}`;
}

/** Slugs FR-CH par défaut (cohérent avec les clés i18n `reports-filename-*`). */
const TYPE_SLUGS: Record<ReportType, string> = {
	'balance-sheet': 'bilan',
	'income-statement': 'compte-resultat',
	'trial-balance': 'balance',
	journals: 'journaux',
};

/**
 * Slug ASCII strict. Pipeline (cf. AC #22) :
 * 1. NFD-decompose + strip diacritics (`/[̀-ͯ]/g`).
 * 2. lowercase.
 * 3. `[^a-z0-9-]` → `-`.
 * 4. Collapse `-+` → `-`.
 * 5. Truncate 20 chars.
 * 6. Strip trailing `-` (Pass 1 ECH-H3).
 * 7. Fallback si vide.
 */
export function slugify(input: string, fallback: string): string {
	const normalised = input
		.normalize('NFD')
		.replace(/[̀-ͯ]/g, '')
		.toLowerCase()
		.replace(/[^a-z0-9-]/g, '-')
		.replace(/-+/g, '-')
		.slice(0, 20)
		.replace(/-+$/, '')
		.replace(/^-+/, '');
	return normalised.length === 0 ? fallback : normalised;
}

/**
 * Télécharge un export PDF/CSV via `fetch` + Blob + lien `<a download>` éphémère.
 *
 * Le 401 est géré par `apiClient` (redirect login). Autres erreurs (400, 500)
 * sont relayées via une exception formatée que `+page.svelte` capture dans
 * `errorMsg` (AC #23 + Pass 1 ECH-H3).
 *
 * @param type — Type du rapport (utilisé dans le URL path).
 * @param query — Query params (`fiscalYearId`, période, journal optionnel).
 * @param format — `'pdf'` ou `'csv'`.
 * @param filename — Filename suggéré au browser (construit via `buildExportFilename`).
 */
export async function downloadReport(
	type: ReportType,
	query: ReportQuery | JournalReportQuery,
	format: 'pdf' | 'csv',
	filename: string,
): Promise<void> {
	const url = getReportExportUrl(type, query, format);
	// On utilise `apiClient.getBlob` pour bénéficier de la gestion 401 + ApiError centralisée.
	const response = await apiClient.getBlob(url);
	const blob = await response.blob();
	triggerDownload(blob, filename);
}

/** Helper interne : déclenche le download navigateur via un lien éphémère. */
function triggerDownload(blob: Blob, filename: string): void {
	const objectUrl = URL.createObjectURL(blob);
	const a = document.createElement('a');
	a.href = objectUrl;
	a.download = filename;
	document.body.appendChild(a);
	a.click();
	document.body.removeChild(a);
	URL.revokeObjectURL(objectUrl);
}
