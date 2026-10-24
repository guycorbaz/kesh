// Story 31-1c (#503) — API frontend de la sauvegarde automatique faite par
// Kesh et des sauvegardes du serveur (Administrateur ; refusées aux clés d'API).
//
// - `GET/PUT /api/v1/admin/backup-schedule` — réglage, état, `nextDueAt` et
//   `overdue` calculés par le serveur (le navigateur ne compare aucune date à
//   sa propre horloge) ;
// - `POST /api/v1/admin/backup-schedule/run` — « Sauvegarder maintenant » ;
// - `GET /api/v1/admin/backups` et `GET /api/v1/admin/backups/{kind}/{name}`.

import { apiClient, LONG_OPERATION_TIMEOUT_MS } from '$lib/shared/utils/api-client';
import { parseContentDispositionFilename, triggerDownload } from '$lib/shared/utils/download';

const BACKUP_SCHEDULE_URL = '/api/v1/admin/backup-schedule';
const SERVER_BACKUPS_URL = '/api/v1/admin/backups';

/** Statut de la dernière tentative (`last_status`). */
export type BackupStatus = 'ok' | 'partial' | 'failed' | 'insufficient_space' | 'disk_full';

/** Les trois statuts d'échec (aucune sauvegarde écrite). */
export const BACKUP_FAILURE_STATUSES: readonly BackupStatus[] = [
	'failed',
	'insufficient_space',
	'disk_full',
];

/** Fréquences permises, en heures (contrainte de la table). */
export const BACKUP_INTERVAL_HOURS = [6, 12, 24, 168] as const;

/** Bornes du nombre de sauvegardes conservées (contrainte de la table). */
export const RETENTION_MIN = 1;
export const RETENTION_MAX = 60;

/** Réglage et état rendus par `GET /api/v1/admin/backup-schedule`. */
export interface BackupSchedule {
	enabled: boolean;
	intervalHours: number;
	retentionCount: number;
	version: number;
	lastAttemptAt: string | null;
	lastSuccessAt: string | null;
	lastSuccessFile: string | null;
	lastSuccessBytes: number | null;
	lastStatus: BackupStatus | null;
	lastFilesNotIncluded: number | null;
	lastCompleteFile: string | null;
	/** Prochaine sauvegarde prévue, calculée par le serveur ; `null` si désactivée. */
	nextDueAt: string | null;
	/** Dernière réussite datant d'au moins deux intervalles — calculé par le serveur. */
	overdue: boolean;
	/**
	 * Story 32-1b (#711 L-B5) — la dernière réussite est datée **après** l'heure
	 * du serveur (horloge reculée) ; calculé par le serveur (Story 32-1d), vrai
	 * seulement si la sauvegarde est activée, et alors `overdue` l'est aussi.
	 */
	lastSuccessInFuture: boolean;
}

/** Corps de `PUT /api/v1/admin/backup-schedule`. */
export interface BackupScheduleSettings {
	enabled: boolean;
	intervalHours: number;
	retentionCount: number;
	version: number;
}

/** Une sauvegarde présente sur le serveur. */
export interface ServerBackup {
	name: string;
	kind: 'auto' | 'pre-import';
	sizeBytes: number;
	modifiedAt: string | null;
}

/** Réponse de `GET /api/v1/admin/backups`. */
export interface ServerBackups {
	/** Taille d'import de l'installation (`KESH_ADMIN_IMPORT_MAX_MB`), en Mio. */
	importMaxMib: number;
	items: ServerBackup[];
}

/** Lit le réglage et l'état de la sauvegarde automatique. */
export function fetchBackupSchedule(): Promise<BackupSchedule> {
	return apiClient.get<BackupSchedule>(BACKUP_SCHEDULE_URL);
}

/** Enregistre le réglage (verrou optimiste : 409 si `version` est périmée). */
export function saveBackupSchedule(settings: BackupScheduleSettings): Promise<BackupSchedule> {
	return apiClient.put<BackupSchedule>(BACKUP_SCHEDULE_URL, {
		enabled: settings.enabled,
		intervalHours: settings.intervalHours,
		retentionCount: settings.retentionCount,
		version: settings.version,
	});
}

/**
 * « Sauvegarder maintenant » : le serveur ne répond qu'une fois la sauvegarde
 * écrite — hors du délai de 30 s du client, jamais rejoué (AC32, AC18b).
 * 409 `BACKUP_IN_PROGRESS` si une autre sauvegarde est en cours.
 */
export function runBackupNow(): Promise<BackupSchedule> {
	return apiClient.post<BackupSchedule>(`${BACKUP_SCHEDULE_URL}/run`, undefined, {
		timeoutMs: LONG_OPERATION_TIMEOUT_MS,
		retry: false,
	});
}

/** Liste les sauvegardes du serveur. */
export function fetchServerBackups(): Promise<ServerBackups> {
	return apiClient.get<ServerBackups>(SERVER_BACKUPS_URL);
}

/** Télécharge une sauvegarde du serveur (appel long, jamais rejoué). */
export async function downloadServerBackup(backup: ServerBackup): Promise<void> {
	const response = await apiClient.getBlob(
		`${SERVER_BACKUPS_URL}/${encodeURIComponent(backup.kind)}/${encodeURIComponent(backup.name)}`,
		{ timeoutMs: LONG_OPERATION_TIMEOUT_MS, retry: false },
	);
	const blob = await response.blob();
	if (blob.size === 0) {
		throw new Error('Réponse vide du serveur (0 octet) — sauvegarde invalide.');
	}
	triggerDownload(
		blob,
		parseContentDispositionFilename(response.headers.get('Content-Disposition')) ?? backup.name,
	);
}

/**
 * C-31-1-52 — `true` si la sauvegarde dépasse la taille d'import de
 * l'installation : comparaison **stricte** ; l'enveloppe de l'envoi n'est pas
 * comptée (une archive à quelques octets du plafond peut être refusée sans
 * mention — écrit au manuel).
 */
export function exceedsImportLimit(sizeBytes: number, importMaxMib: number): boolean {
	return sizeBytes > importMaxMib * 1_048_576;
}

/** Le bandeau de l'application à afficher pour cet état (AC31 (b)). */
export type BannerKind = 'failed' | 'overdue' | null;

/**
 * AC31 (b) — `failed` si la sauvegarde automatique est **activée** et que la
 * dernière tentative a échoué ; sinon `overdue` si le **serveur** dit qu'aucune
 * réussite ne date de moins de deux intervalles ; sinon rien. `partial`
 * n'allume pas le bandeau (C-31-1-40) ; une sauvegarde désactivée non plus
 * (plus aucun tour ne mettrait l'état à jour) ; un état illisible non plus.
 */
export function bannerKind(schedule: BackupSchedule | null): BannerKind {
	if (!schedule || !schedule.enabled) return null;
	if (schedule.lastStatus && BACKUP_FAILURE_STATUSES.includes(schedule.lastStatus)) {
		return 'failed';
	}
	return schedule.overdue ? 'overdue' : null;
}
