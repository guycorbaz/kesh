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
	/** Projet analytique par défaut (Story 19-5) — `null` si aucun. */
	defaultProjectId: number | null;
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
	/** Projet analytique par défaut (Story 19-5). */
	defaultProjectId?: number | null;
}

export interface UpdateRuleRequest {
	expectedVersion: number;
	label?: string;
	matchValue?: string;
	counterpartyAccountId?: number;
	priority?: number;
	active?: boolean;
	/**
	 * Projet analytique par défaut (Story 19-5) — sémantique deux niveaux :
	 * champ absent = inchangé ; `null` = effacer ; `<id>` = affecter.
	 */
	defaultProjectId?: number | null;
}

export interface RulesListResponse {
	items: ReconciliationRule[];
	total: number;
	limit: number;
	offset: number;
}
