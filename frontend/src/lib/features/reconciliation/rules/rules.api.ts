// Story 8-5b FR47 — Client API CRUD pour reconciliation_rules.

import { apiClient } from '$lib/shared/utils/api-client';
import type {
	CreateRuleRequest,
	ReconciliationRule,
	RulesListResponse,
	UpdateRuleRequest,
} from './rules.types';

/**
 * Story 38-11d (AC-2) — une règle lue : les filtres absents d'une réponse
 * (serveur antérieur à la 38-11c1) valent `null`.
 */
function parseRule(raw: ReconciliationRule): ReconciliationRule {
	return {
		...raw,
		providerType: raw.providerType ?? null,
		counterpartyEmail: raw.counterpartyEmail ?? null,
		direction: raw.direction ?? null,
	};
}

/** GET /api/v1/reconciliation/rules — liste paginée. */
export async function listRules(
	page = 1,
	perPage = 50,
	active?: boolean,
): Promise<RulesListResponse> {
	const params = new URLSearchParams({ page: String(page), perPage: String(perPage) });
	if (active !== undefined) params.set('active', String(active));
	const res = await apiClient.get<RulesListResponse>(
		`/api/v1/reconciliation/rules?${params.toString()}`,
	);
	return { ...res, items: res.items.map(parseRule) };
}

/** GET /api/v1/reconciliation/rules/{id} — détail. */
export async function getRule(id: number): Promise<ReconciliationRule> {
	return parseRule(await apiClient.get<ReconciliationRule>(`/api/v1/reconciliation/rules/${id}`));
}

/** POST /api/v1/reconciliation/rules — création. */
export async function createRule(req: CreateRuleRequest): Promise<ReconciliationRule> {
	return parseRule(await apiClient.post<ReconciliationRule>('/api/v1/reconciliation/rules', req));
}

/** PATCH /api/v1/reconciliation/rules/{id} — patch partiel + optimistic lock. */
export async function updateRule(
	id: number,
	req: UpdateRuleRequest,
): Promise<ReconciliationRule> {
	return parseRule(
		await apiClient.patch<ReconciliationRule>(`/api/v1/reconciliation/rules/${id}`, req),
	);
}

/** DELETE /api/v1/reconciliation/rules/{id} — soft-delete idempotent (204). */
export async function deleteRule(id: number): Promise<void> {
	await apiClient.delete(`/api/v1/reconciliation/rules/${id}`);
}
