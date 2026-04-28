/**
 * Types TS miroir des DTOs API pour les exercices comptables (Story 3.7).
 *
 * Shape identique au backend `crates/kesh-api/src/routes/fiscal_years.rs`
 * (serde `rename_all = "camelCase"`).
 */

export type FiscalYearStatus = 'Open' | 'Closed';

export interface FiscalYearResponse {
	id: number;
	companyId: number;
	name: string;
	/** Format ISO `YYYY-MM-DD`. */
	startDate: string;
	/** Format ISO `YYYY-MM-DD`. */
	endDate: string;
	status: FiscalYearStatus;
	createdAt: string;
	updatedAt: string;
}

export interface CreateFiscalYearRequest {
	name: string;
	/** Format ISO `YYYY-MM-DD`. */
	startDate: string;
	/** Format ISO `YYYY-MM-DD`. */
	endDate: string;
}

export interface UpdateFiscalYearRequest {
	name: string;
}
