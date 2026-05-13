// Story 8-5b FR47 — Types DTOs reconciliation_rules (miroir
// kesh-api/routes/reconciliation_rules.rs).

export type MatchType =
	| 'counterparty_contains'
	| 'counterparty_exact'
	| 'reference_contains'
	| 'iban_exact';

export interface ReconciliationRule {
	id: number;
	label: string;
	matchType: MatchType;
	matchValue: string;
	counterpartyAccountId: number;
	priority: number;
	active: boolean;
	appliedCount: number;
	lastAppliedAt: string | null;
	version: number;
	createdAt: string;
	updatedAt: string;
}

export interface CreateRuleRequest {
	label: string;
	matchType: MatchType;
	matchValue: string;
	counterpartyAccountId: number;
	priority?: number;
}

export interface UpdateRuleRequest {
	expectedVersion: number;
	label?: string;
	matchValue?: string;
	counterpartyAccountId?: number;
	priority?: number;
	active?: boolean;
}

export interface RulesListResponse {
	items: ReconciliationRule[];
	total: number;
	limit: number;
	offset: number;
}
