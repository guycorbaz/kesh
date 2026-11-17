/**
 * Story 37-5 — fabrique d'aperçus de réévaluation pour les tests Vitest du
 * composant et de l'avis de K9. Fixtures synthétiques (cours fictifs).
 */

import type {
	RevaluationLine,
	RevaluationPreview,
	RevaluationRate
} from './fiscal-years.types';

export function rate(over: Partial<RevaluationRate> = {}): RevaluationRate {
	return {
		currency: 'USD',
		rate: '0.9119300000',
		rateKind: 'day',
		rateDate: '2024-12-31',
		defaultKind: 'day',
		defaultRate: '0.9119300000',
		defaultSource: 'afc',
		overridden: false,
		sourceDate: '2024-12-31',
		...over
	};
}

export function line(over: Partial<RevaluationLine> = {}): RevaluationLine {
	return {
		accountId: 10,
		accountNumber: '1020',
		accountName: 'PayPal USD',
		accountType: 'Asset',
		currency: 'USD',
		archived: false,
		selectable: true,
		selected: true,
		reason: null,
		deviseNet: '1000.0000',
		chfNet: '880.0000',
		rate: '0.9119300000',
		targetChf: '911.93',
		delta: '31.93',
		counterpartAccountId: 50,
		counterpartAccountNumber: '6950',
		counterpartAccountName: 'Gains de change',
		fxAccountsOrigin: 'company',
		label: 'Gain de change PayPal USD USD 0.91193 au 31.12.2024 (non réalisé)',
		previousRevaluationTotal: '0.00',
		previousEntryIds: [],
		revaluationTotalAfter: '31.93',
		...over
	};
}

export function preview(over: Partial<RevaluationPreview> = {}): RevaluationPreview {
	return {
		fiscalYearId: 1,
		startDate: '2024-01-01',
		endDate: '2024-12-31',
		closingRateKind: 'day',
		rates: [rate()],
		lines: [line()],
		totals: { gains: '31.93', losses: '0.00' },
		preselectionTruncated: false,
		blockers: [],
		...over
	};
}
