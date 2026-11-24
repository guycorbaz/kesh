// Story 8-5b FR47 — Types DTOs reconciliation_rules (miroir
// kesh-api/routes/reconciliation_rules.rs).

export type MatchType =
	| 'counterparty_contains'
	| 'counterparty_exact'
	| 'reference_contains'
	| 'iban_exact';

/**
 * Story 38-11d (AC-2) — sens facultatif d'une règle (filtre de 38-11c1, AC-3) :
 * `credit` (entrées), `debit` (sorties) ; `null` = les deux.
 */
export type RuleDirection = 'credit' | 'debit';

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
	/**
	 * Story 38-11d (AC-2) — filtres facultatifs de 38-11c1 (AC-2, AC-3),
	 * combinés au critère : type de la transaction chez le prestataire (exact,
	 * sans casse), e-mail exact (normalisé), sens. `null` = filtre absent. Une
	 * réponse d'un serveur antérieur, sans ces champs, les lit `null`.
	 */
	providerType: string | null;
	counterpartyEmail: string | null;
	direction: RuleDirection | null;
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
	/** Story 38-11d (AC-2) — filtres facultatifs ; absents ou `null` = aucun. */
	providerType?: string | null;
	counterpartyEmail?: string | null;
	direction?: RuleDirection | null;
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
	/**
	 * Story 38-11d (AC-2) — filtres, même sémantique que `defaultProjectId` :
	 * absent = inchangé ; `null` = effacé ; valeur = posée.
	 */
	providerType?: string | null;
	counterpartyEmail?: string | null;
	direction?: RuleDirection | null;
}

export interface RulesListResponse {
	items: ReconciliationRule[];
	total: number;
	limit: number;
	offset: number;
}
