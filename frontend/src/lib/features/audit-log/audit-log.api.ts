/**
 * API du journal d'audit — Story 25-1c-b1 (#378).
 *
 * Consomme les trois routes de la Story 25-1c-a (Comptable et Administrateur,
 * refusées aux clés API) : la liste paginée, le vocabulaire traduit, et
 * l'export CSV. ⚠️ Le frontend ne traduit RIEN du vocabulaire : la route en est
 * la source unique.
 */

import { apiClient } from '$lib/shared/utils/api-client';
import { parseContentDispositionFilename, triggerDownload } from '$lib/shared/utils/download';
import type { AuditLogListResponse, AuditLogQuery, AuditLogVocabulary } from './audit-log.types';
import { filtersOnly, serializeQuery } from './query-helpers';

const BASE = '/api/v1/audit-log';

/** Les paramètres d'une requête ; un paramètre vide ou absent n'est pas envoyé. */
function withParams(url: string, params: URLSearchParams): string {
	const qs = params.toString();
	return qs ? `${url}?${qs}` : url;
}

/** `GET /api/v1/audit-log` — une page de la liste filtrée. */
export function listAuditLog(query: AuditLogQuery): Promise<AuditLogListResponse> {
	const params = serializeQuery(query);
	// La route attend `offset` et `limit` ; `serializeQuery` omet leurs défauts
	// pour l'URL de la page, on les rend explicites ici.
	params.set('offset', String(query.offset ?? 0));
	params.set('limit', String(query.limit ?? 50));
	return apiClient.get<AuditLogListResponse>(withParams(BASE, params));
}

/** `GET /api/v1/audit-log/vocabulary` — types d'entité et actions, traduits. */
export function getAuditLogVocabulary(): Promise<AuditLogVocabulary> {
	return apiClient.get<AuditLogVocabulary>(`${BASE}/vocabulary`);
}

/**
 * `GET /api/v1/audit-log/export.csv` — l'export des filtres affichés
 * (`offset` et `limit` exceptés), traduit par le serveur dans la langue de
 * l'interface. Le nom du fichier vient de `Content-Disposition`.
 */
export async function exportAuditLogCsv(query: AuditLogQuery): Promise<void> {
	const response = await apiClient.getBlob(
		withParams(`${BASE}/export.csv`, serializeQuery(filtersOnly(query))),
	);
	const blob = await response.blob();
	const filename =
		parseContentDispositionFilename(response.headers.get('Content-Disposition')) ??
		'kesh-journal-audit.csv';
	triggerDownload(blob, filename);
}
