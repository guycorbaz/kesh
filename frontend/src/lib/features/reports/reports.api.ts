// Story 9-1 — Client API rapports comptables (4 endpoints GET /api/v1/reports/*).

import { apiClient } from '$lib/shared/utils/api-client';
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
