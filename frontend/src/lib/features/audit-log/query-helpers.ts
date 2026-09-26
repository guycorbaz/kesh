/**
 * Les filtres du journal d'audit dans l'URL — Story 25-1c-b1 (#378).
 *
 * Patron : `features/journal-entries/query-helpers.ts`. La sérialisation omet
 * les valeurs par défaut ; la lecture ignore les valeurs invalides.
 *
 * ⛔ **`entityId` n'a de sens qu'avec un `entityType`** : la route répond 400
 * sinon. Un `entityId` seul est donc ignoré à la lecture (un lien retouché à la
 * main ne doit pas produire un écran en erreur) et omis à la sérialisation.
 *
 * ⚠️ `entityType` et `action` hors vocabulaire sont CONSERVÉS : un code
 * historique, écrit par une version antérieure, doit rester filtrable.
 */

import type { AuditLogQuery } from './audit-log.types';

/** La taille de page par défaut, omise de l'URL. */
export const DEFAULT_LIMIT = 50;

const DATE_RE = /^\d{4}-\d{2}-\d{2}$/;

/** Les filtres, sans la pagination — ce que l'export transmet. */
export function filtersOnly(query: AuditLogQuery): AuditLogQuery {
	const f: AuditLogQuery = {};
	if (query.dateFrom) f.dateFrom = query.dateFrom;
	if (query.dateTo) f.dateTo = query.dateTo;
	if (query.entityType) {
		f.entityType = query.entityType;
		if (query.entityId !== undefined && query.entityId > 0) f.entityId = query.entityId;
	}
	if (query.action) f.action = query.action;
	return f;
}

/** Sérialise les filtres et la pagination ; les valeurs vides et par défaut sont omises. */
export function serializeQuery(query: AuditLogQuery): URLSearchParams {
	const params = new URLSearchParams();
	const f = filtersOnly(query);
	if (f.dateFrom) params.set('dateFrom', f.dateFrom);
	if (f.dateTo) params.set('dateTo', f.dateTo);
	if (f.entityType) params.set('entityType', f.entityType);
	if (f.entityId !== undefined) params.set('entityId', String(f.entityId));
	if (f.action) params.set('action', f.action);
	if (query.offset !== undefined && query.offset > 0) params.set('offset', String(query.offset));
	if (query.limit !== undefined && query.limit !== DEFAULT_LIMIT) {
		params.set('limit', String(query.limit));
	}
	return params;
}

/** Relit les filtres depuis l'URL ; toute valeur invalide est ignorée. */
export function parseQueryFromUrl(searchParams: URLSearchParams): AuditLogQuery {
	const query: AuditLogQuery = {};

	const dateFrom = searchParams.get('dateFrom');
	if (dateFrom && DATE_RE.test(dateFrom)) query.dateFrom = dateFrom;
	const dateTo = searchParams.get('dateTo');
	if (dateTo && DATE_RE.test(dateTo)) query.dateTo = dateTo;

	const entityType = searchParams.get('entityType');
	if (entityType && entityType.trim() !== '') query.entityType = entityType.trim();

	// ⛔ Sans type, l'identifiant est ignoré.
	const entityId = searchParams.get('entityId');
	if (query.entityType && entityId !== null) {
		const n = Number(entityId);
		if (Number.isInteger(n) && n > 0) query.entityId = n;
	}

	const action = searchParams.get('action');
	if (action && action.trim() !== '') query.action = action.trim();

	const offset = searchParams.get('offset');
	if (offset !== null) {
		const n = Number(offset);
		if (Number.isInteger(n) && n >= 0) query.offset = n;
	}
	const limit = searchParams.get('limit');
	if (limit !== null) {
		const n = Number(limit);
		if (Number.isInteger(n) && n > 0) query.limit = n;
	}
	return query;
}
