// Story 17-3b — API frontend pour l'export complet d'installation (.keshbackup).
//
// Consomme `GET /api/v1/admin/full-export` (Story 17-3a : Admin strict +
// anti-PAT, `Content-Type: application/octet-stream`, `Content-Disposition`
// `kesh-installation-{YYYY-MM-DD}.keshbackup`).
//
// **Distinct** de `exports.api.ts::downloadGlobalExport` (Story 9-2b) qui est
// l'export per-company CSV (`/exports/global.zip`). Ici = installation complète.
//
// DC-B1 (cf. story 17-3b) : `parseContentDispositionFilename` + `triggerDownload`
// sont **dupliqués localement** depuis `exports.api.ts` (~30 lignes). Le module
// partagé `lib/shared/utils/download.ts` existe désormais (Story 25-1c-b1) ; la
// migration de cette copie et des six autres est suivie par l'issue #438.

import { apiClient, LONG_OPERATION_TIMEOUT_MS } from '$lib/shared/utils/api-client';

const FULL_EXPORT_URL = '/api/v1/admin/full-export';
const FALLBACK_FILENAME = 'kesh-installation.keshbackup';

/** Nombre de pièces référencées que l'export n'a pas emportées (Story 31-1a, AC5). */
const HEADER_FILES_NOT_INCLUDED = 'x-kesh-backup-files-not-included';
/** Posé quand l'archive dépasse le plafond d'import ; valeur : ce plafond en Mio (31-1a, AC12). */
const HEADER_EXCEEDS_IMPORT_LIMIT = 'x-kesh-backup-exceeds-import-limit';

/**
 * Story 31-1b (AC18) — ce que l'écran doit dire après un export réussi. `null` :
 * en-tête absent (serveur antérieur à la 0.15.0) ou illisible — rien à dire.
 */
export interface FullExportOutcome {
	/** Pièces référencées non emportées (`0` : toutes emportées). */
	filesNotIncluded: number | null;
	/** Plafond d'import dépassé, en Mio ; `null` si l'archive le respecte. */
	exceedsImportLimitMib: number | null;
}

/** Lit un en-tête entier positif ou nul ; `null` s'il manque ou ne se lit pas. */
function headerCount(headers: Headers, name: string): number | null {
	const raw = headers.get(name)?.trim();
	if (!raw || !/^\d+$/.test(raw)) return null;
	const value = Number(raw);
	return Number.isSafeInteger(value) ? value : null;
}

/**
 * Télécharge l'export complet d'installation `.keshbackup` via `fetch` + Blob +
 * lien `<a download>` éphémère.
 *
 * Le filename provient du header `Content-Disposition` retourné par le backend
 * 17-3a ; fallback `kesh-installation.keshbackup` si absent/illisible.
 *
 * `apiClient.getBlob` jette déjà une `ApiError` sur HTTP non-2xx — l'exception
 * remonte au composant appelant (`AdminBackupPanel`) qui formate le message.
 * Le guard de ré-entrance (`exporting`) vit côté composant (AC9).
 *
 * Story 31-1b : l'appel échappe au délai de 30 s du client et n'est jamais
 * rejoué (AC18b — le serveur n'envoie ses en-têtes qu'une fois l'archive
 * assemblée) ; la fonction rend ce que les en-têtes disent des pièces omises et
 * du plafond d'import (AC18).
 */
export async function downloadFullExport(): Promise<FullExportOutcome> {
	const response = await apiClient.getBlob(FULL_EXPORT_URL, {
		timeoutMs: LONG_OPERATION_TIMEOUT_MS,
		retry: false,
	});
	const blob = await response.blob();
	// Garde intégrité (review Pass 1) : un `.keshbackup` est un ZIP, jamais
	// 0 octet. Une réponse 200 vide (troncature stream, disque plein côté
	// serveur) ne doit pas produire un fichier silencieusement inutilisable —
	// on jette pour que l'UI affiche une erreur plutôt qu'un faux succès.
	if (blob.size === 0) {
		throw new Error('Réponse vide du serveur (0 octet) — sauvegarde invalide.');
	}
	const filename =
		parseContentDispositionFilename(response.headers.get('Content-Disposition')) ??
		FALLBACK_FILENAME;
	triggerDownload(blob, filename);
	return {
		filesNotIncluded: headerCount(response.headers, HEADER_FILES_NOT_INCLUDED),
		exceedsImportLimitMib: headerCount(response.headers, HEADER_EXCEEDS_IMPORT_LIMIT),
	};
}

/**
 * Parse un header `Content-Disposition` (RFC 6266 / RFC 5987).
 * Reconnait `filename*=UTF-8''<percent>` (priorité, supporte non-ASCII) et
 * `filename="…"` (fallback ASCII). Retourne `null` si rien ne matche.
 *
 * Dupliqué de `exports.api.ts` (DC-B1) — logique identique, déjà revue 9-2b.
 */
export function parseContentDispositionFilename(header: string | null): string | null {
	if (!header) return null;

	// RFC 5987 (priorité) : `filename*=UTF-8''<percent>` ou `…'<lang>'<percent>`.
	const rfc5987 = header.match(/filename\*\s*=\s*UTF-8'[^']*'([^;\r\n]+)/i);
	if (rfc5987 && rfc5987[1]) {
		try {
			const decoded = decodeURIComponent(rfc5987[1].trim());
			if (decoded.length > 0) return decoded;
		} catch {
			// Percent-decode échoué → fallback RFC 6266 ci-dessous.
		}
	}

	// RFC 6266 : `filename="…"` (ASCII strict).
	const rfc6266 = header.match(/filename\s*=\s*"([^"]+)"/i);
	if (rfc6266 && rfc6266[1]) {
		return rfc6266[1];
	}

	// RFC 6266 unquoted : `filename=…` (lookahead négatif `(?!\*)` pour ne pas
	// matcher `filename*=…` à valeur vide).
	const rfc6266Unq = header.match(/filename(?!\*)\s*=\s*([^;\s]+)/i);
	if (rfc6266Unq && rfc6266Unq[1]) {
		return rfc6266Unq[1];
	}

	return null;
}

/**
 * Déclenche le download navigateur via un lien `<a download>` éphémère.
 * `URL.createObjectURL` est **HTTP-LAN safe** (pas de secure-context requis,
 * cf. AC28). Cleanup robuste dans un `finally` (fuite mémoire si `click()` jette).
 *
 * Dupliqué de `exports.api.ts` (DC-B1).
 */
function triggerDownload(blob: Blob, filename: string): void {
	const objectUrl = URL.createObjectURL(blob);
	const a = document.createElement('a');
	a.href = objectUrl;
	a.download = filename;
	try {
		document.body.appendChild(a);
		a.click();
	} finally {
		if (a.parentNode) a.parentNode.removeChild(a);
		URL.revokeObjectURL(objectUrl);
	}
}
