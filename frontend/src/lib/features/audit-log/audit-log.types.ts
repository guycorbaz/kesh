/**
 * Types du journal d'audit — Story 25-1c-b1 (#378).
 *
 * Miroir **exact** des DTO de la route (Story 25-1c-a,
 * `crates/kesh-api/src/routes/audit_log.rs`). ⚠️ Pas de `companyId` : la route
 * filtre strictement par société et ne le rend pas.
 */

/** Une entrée du journal, telle que `GET /api/v1/audit-log` la rend. */
export interface AuditLogEntry {
	id: number;
	/** Horodatage ISO 8601 en UTC, finissant par `Z`. */
	createdAt: string;
	/** Le nom de l'auteur AU MOMENT de l'écriture (instantané, pas une jointure). */
	actorLabel: string;
	actorType: 'user' | 'api_key';
	actorApiKeyId: number | null;
	userId: number;
	/** Le code technique de l'action (`contact.created`) — sert aux sélecteurs, jamais affiché. */
	action: string;
	/** L'action traduite dans la langue de l'interface — c'est elle qui s'affiche. */
	actionLabel: string;
	entityType: string;
	entityTypeLabel: string;
	/** `0` quand l'entrée ne vise pas une entité précise. */
	entityId: number;
	details: unknown;
}

/** Une entrée du vocabulaire : un code et son libellé traduit. */
export interface AuditLogVocabularyItem {
	code: string;
	label: string;
}

/** `GET /api/v1/audit-log/vocabulary` — rendu dans l'ordre des CODES. */
export interface AuditLogVocabulary {
	entityTypes: AuditLogVocabularyItem[];
	actions: AuditLogVocabularyItem[];
}

/** Les filtres et la pagination de la liste. */
export interface AuditLogQuery {
	/** Jour UTC, `AAAA-MM-JJ`. */
	dateFrom?: string;
	/** Jour UTC, `AAAA-MM-JJ`, inclus. */
	dateTo?: string;
	entityType?: string;
	/** N'a de sens qu'avec `entityType` : la route répond 400 sinon. */
	entityId?: number;
	action?: string;
	offset?: number;
	limit?: number;
}

/** L'enveloppe paginée de la liste. */
export interface AuditLogListResponse {
	items: AuditLogEntry[];
	total: number;
	offset: number;
	limit: number;
}
