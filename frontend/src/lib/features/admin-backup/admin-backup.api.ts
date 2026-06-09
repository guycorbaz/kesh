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
// sont **dupliqués localement** depuis `exports.api.ts` (~30 lignes) plutôt
// qu'importés cross-feature (anti-pattern) ou extraits maintenant. L'extraction
// vers `lib/shared/utils/download.ts` (3ᵉ feature à dupliquer) est tracée comme
// cleanup v0.2 / Epic 15 (note umbrella 9-2b §triggerDownload-reuse).

import { apiClient } from '$lib/shared/utils/api-client';

const FULL_EXPORT_URL = '/api/v1/admin/full-export';
const FALLBACK_FILENAME = 'kesh-installation.keshbackup';

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
 */
export async function downloadFullExport(): Promise<void> {
	const response = await apiClient.getBlob(FULL_EXPORT_URL);
	const blob = await response.blob();
	const filename =
		parseContentDispositionFilename(response.headers.get('Content-Disposition')) ??
		FALLBACK_FILENAME;
	triggerDownload(blob, filename);
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
