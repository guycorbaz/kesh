/**
 * Story 29-1a (#232) — fabrique de réponses `closing-checks` pour les tests
 * Vitest du panneau et de la page des exercices. Fixtures synthétiques.
 */

import type {
	ClosingCheck,
	ClosingCheckCode,
	ClosingChecksResponse
} from './fiscal-years.types';

const ORDER: [ClosingCheckCode, ClosingCheck['severity']][] = [
	['EARLIER_FISCAL_YEAR_OPEN', 'blocking'],
	['PENDING_BANK_TRANSACTIONS', 'warning'],
	['BANK_STATEMENT_AT_YEAR_END', 'warning'],
	['DRAFT_INVOICES', 'info'],
	['IMPORTED_SUPPLIER_INVOICES_TO_COMPLETE', 'info'],
	['INCONSISTENT_ENTRIES', 'anomaly'],
	['NEXT_FISCAL_YEAR', 'info']
];

/** Les sept contrôles `ok` (K7 avec un exercice suivant), surchargés par code. */
export function closingChecks(
	over: Partial<Record<ClosingCheckCode, Partial<ClosingCheck>>> = {}
): ClosingChecksResponse {
	const base: Record<ClosingCheckCode, ClosingCheck['details']> = {
		EARLIER_FISCAL_YEAR_OPEN: { fiscalYear: null },
		PENDING_BANK_TRANSACTIONS: { accounts: [] },
		BANK_STATEMENT_AT_YEAR_END: { accounts: [], notCheckable: [] },
		DRAFT_INVOICES: { invoices: [] },
		IMPORTED_SUPPLIER_INVOICES_TO_COMPLETE: {},
		INCONSISTENT_ENTRIES: {
			countByReason: {
				UNBALANCED: 0,
				TOO_FEW_LINES: 0,
				DATE_OUTSIDE_ITS_YEAR: 0,
				OTHER_YEAR_DATED_INSIDE: 0
			},
			entries: []
		},
		NEXT_FISCAL_YEAR: {
			fiscalYear: { id: 2, name: 'Exercice 2027', startDate: '2027-01-01', status: 'Open' }
		}
	};
	return {
		fiscalYearId: 1,
		fiscalYearName: 'Exercice 2026',
		startDate: '2026-01-01',
		endDate: '2026-12-31',
		checks: ORDER.map(([code, severity]) => ({
			code,
			severity,
			status: 'ok',
			count: 0,
			details: base[code],
			...over[code]
		}))
	};
}
