// Story 33-3b (#918) — la synthèse des graphiques de l'accueil, miroir du contrat
// de `GET /api/v1/reports/homepage-summary` (Story 33-3a).
//
// Tout montant reste une chaîne décimale (cf. `homepage.ts`).

import { apiClient } from '$lib/shared/utils/api-client';

export interface MonthSummary {
	/** `AAAA-MM`. */
	month: string;
	/** Premier jour du mois, borné à l'exercice. */
	from: string;
	/** Dernier jour du mois, borné à l'exercice. */
	to: string;
	revenues: string;
	expenses: string;
}

export interface ExpensePost {
	accountId: number;
	number: string;
	name: string;
	amount: string;
	/** Pourcentage de `expensePostsBase`, une décimale écrite. */
	share: string;
}

export interface OtherExpenses {
	amount: string;
	share: string;
	accountCount: number;
}

export interface SummaryFiscalYear {
	id: number;
	name: string;
	startDate: string;
	endDate: string;
}

interface SummaryCommon {
	asOf: string;
	months: MonthSummary[];
	totalRevenues: string;
	totalExpenses: string;
	expensePosts: ExpensePost[];
	otherExpenses: OtherExpenses | null;
	expensePostsBase: string;
	creditBalanceExpenseAccounts: number;
}

/** `fiscalYear` n'est nul qu'avec `noOpenFiscalYear`. */
export type HomepageSummary =
	| (SummaryCommon & { state: 'open'; fiscalYear: SummaryFiscalYear })
	| (SummaryCommon & { state: 'noOpenFiscalYear'; fiscalYear: null });

export async function fetchHomepageSummary(): Promise<HomepageSummary> {
	return apiClient.get<HomepageSummary>('/api/v1/reports/homepage-summary');
}
