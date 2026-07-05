// Story 9-1 — Client API rapports comptables (4 endpoints GET /api/v1/reports/*).

import Big from 'big.js';
import { apiClient } from '$lib/shared/utils/api-client';
import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
import { formatSwissAmount } from '$lib/features/journal-entries/balance';
import type {
	BalanceSheetDto,
	IncomeStatementDto,
	JournalReportDto,
	JournalReportQuery,
	ProjectExpensesDto,
	ProjectReportQuery,
	ProjectReportType,
	ProjectReturnDto,
	ReportQuery,
	ReportType,
	TrialBalanceDto,
	VatReportDto,
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

export async function getVatReport(query: ReportQuery): Promise<VatReportDto> {
	const qs = buildQuery({
		fiscalYearId: query.fiscalYearId,
		periodStart: query.periodStart,
		periodEnd: query.periodEnd,
	});
	return apiClient.get<VatReportDto>(`/api/v1/reports/vat?${qs}`);
}

// ============================================================================
// Story 19-6a — Rapport « Dépenses par projet »
// ============================================================================

/** Construit la query string commune aux rapports projet (JSON + export). */
function buildProjectQuery(
	query: ProjectReportQuery,
	extra?: Record<string, string | number | undefined>,
): string {
	return buildQuery({
		projectId: query.projectId,
		mode: query.mode,
		fiscalYearId: query.mode === 'fiscal_year' ? query.fiscalYearId : undefined,
		periodStart: query.periodStart,
		periodEnd: query.periodEnd,
		...extra,
	});
}

export async function getProjectExpenses(query: ProjectReportQuery): Promise<ProjectExpensesDto> {
	const qs = buildProjectQuery(query);
	return apiClient.get<ProjectExpensesDto>(`/api/v1/reports/project-expenses?${qs}`);
}

export async function getProjectReturn(query: ProjectReportQuery): Promise<ProjectReturnDto> {
	const qs = buildProjectQuery(query);
	return apiClient.get<ProjectReturnDto>(`/api/v1/reports/project-return?${qs}`);
}

/** URL d'export d'un rapport projet. */
export function getProjectReportExportUrl(
	type: ProjectReportType,
	query: ProjectReportQuery,
	format: 'pdf' | 'csv',
): string {
	const qs = buildProjectQuery(query, { format });
	return `/api/v1/reports/${type}/export?${qs}`;
}

/** Télécharge un export de rapport projet (même mécanique que downloadReport). */
export async function downloadProjectReport(
	type: ProjectReportType,
	query: ProjectReportQuery,
	format: 'pdf' | 'csv',
	filename: string,
): Promise<void> {
	const url = getProjectReportExportUrl(type, query, format);
	const response = await apiClient.getBlob(url);
	const blob = await response.blob();
	triggerDownload(blob, filename);
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
		| VatReportDto
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
		case 'vat': {
			// Story 18-1d : un décompte sans vente (rows vide) mais avec de la TVA
			// récupérable (achats seuls) n'est PAS vide — il doit afficher le pied
			// de tableau (récupérable + solde). Vide = aucune vente ET récupérable 0.
			const vr = dto as VatReportDto;
			return vr.rows.length === 0 && Number(vr.totalVatRecoverable) === 0;
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
 * (AC #22 + Pass 1 ECH-H3 + Pass 1 code-review H10).
 *
 * Pipeline slug ASCII : NFD-strip diacritics + lowercase + non-alphanum → `-` +
 * collapse `-+` → `-` + truncate 20 chars + strip trailing `-`. Fallback
 * `'company'` si vide post-slug (e.g. nom CJK).
 *
 * Pass 1 code-review H10 (AA2-H1 + AA4-F1) : `typeSlug` est résolu via i18n
 * (clé `reports-filename-{type}`) au lieu d'être hardcodé FR. Cohérent avec
 * la résolution backend (cf. `routes/reports.rs::resolve_type_slug`) — en
 * DE-CH on obtient `bilanz`, en IT-CH `bilancio`, etc.
 */
export function buildExportFilename(
	type: ReportType,
	companyName: string,
	period: { start: string; end: string },
	format: 'pdf' | 'csv',
): string {
	const typeSlug = i18nMsg(`reports-filename-${type}`, TYPE_SLUGS_FALLBACK[type]);
	const companySlug = slugify(companyName, 'company');
	return `kesh-${typeSlug}-${companySlug}-${period.start}_${period.end}.${format}`;
}

/**
 * Fallbacks FR-CH si la clé i18n est manquante (cohérent avec les valeurs
 * fr-CH de `reports-filename-*`). Pass 1 code-review M16 : `trial-balance`
 * mis à jour `balance` → `balance-comptes` (clarté vs `bilan`).
 */
const TYPE_SLUGS_FALLBACK: Record<ReportType, string> = {
	'balance-sheet': 'bilan',
	'income-statement': 'compte-resultat',
	'trial-balance': 'balance-comptes',
	journals: 'journaux',
	vat: 'decompte-tva',
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

/**
 * Helper interne : déclenche le download navigateur via un lien éphémère.
 *
 * Pass 1 code-review M11 (BH4-F-01) : `removeChild` + `revokeObjectURL` dans
 * un `finally` pour garantir le cleanup même si `a.click()` jette (e.g. CSP
 * bloque le download, ou popup blocker). Sans ce garde, un crash de click
 * laisserait l'objet URL et le `<a>` éphémère attachés au DOM — fuite mémoire.
 */
function triggerDownload(blob: Blob, filename: string): void {
	const objectUrl = URL.createObjectURL(blob);
	const a = document.createElement('a');
	a.href = objectUrl;
	a.download = filename;
	try {
		document.body.appendChild(a);
		a.click();
	} finally {
		// Cleanup robuste — pas de remove() si appendChild a échoué
		if (a.parentNode) a.parentNode.removeChild(a);
		URL.revokeObjectURL(objectUrl);
	}
}
