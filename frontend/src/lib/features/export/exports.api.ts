// Story 9-2b — API frontend pour l'export global ZIP de souveraineté.
//
// Cohérent pattern Story 9-2a `reports.api.ts::downloadReport` +
// `triggerDownload` cleanup (Pass 1 code-review M11 — try/finally pour
// `removeChild` + `revokeObjectURL` même si `a.click()` jette).

import { apiClient } from '$lib/shared/utils/api-client';

const GLOBAL_EXPORT_URL = '/api/v1/exports/global.zip';
const FALLBACK_FILENAME = 'kesh-export.zip';

/**
 * Télécharge l'export global ZIP via `fetch` + Blob + lien `<a download>` éphémère.
 *
 * Le filename est extrait du header `Content-Disposition` retourné par le
 * backend (Story 9-2b §filename + AC #28) — le frontend ne le calcule pas.
 *
 * `apiClient.getBlob` jette déjà une `ApiError` sur HTTP non-2xx via
 * `parseErrorResponse` (Pass 3 ECH3-M2 — pas de `response.ok` redondant ici).
 * L'exception remonte à `+page.svelte` qui formate l'`errorMsg` (Pass 1 M13).
 *
 * Pass 1 ECH-H2 + AC #25/26 : le caller (`+page.svelte`) gère le flag
 * `exporting` + guard re-entrancy first-line.
 */
export async function downloadGlobalExport(): Promise<void> {
	const response = await apiClient.getBlob(GLOBAL_EXPORT_URL);
	const blob = await response.blob();
	const filename =
		parseContentDispositionFilename(response.headers.get('Content-Disposition')) ??
		FALLBACK_FILENAME;
	triggerDownload(blob, filename);
}

/**
 * Parse un header `Content-Disposition` au format RFC 6266 / RFC 5987.
 *
 * Reconnait deux formes :
 *
 * 1. **RFC 5987 préféré** (`filename*=UTF-8''<percent-encoded>`) — supporte
 *    les caractères non-ASCII via `decodeURIComponent`. La forme avec tag
 *    langue (`filename*=UTF-8'fr-CH'…`) est aussi acceptée — le tag est
 *    skip après le second `'`.
 * 2. **RFC 6266 fallback** (`filename="…"`) — ASCII strict, retour as-is.
 *
 * Retourne `null` si :
 * - `header` est `null` ou chaîne vide
 * - aucune des deux formes ne matche
 *
 * Pass 3 ECH3-H4 : implémenté localement (pas un pattern réutilisé de 9-2a,
 * qui passait le filename par paramètre côté caller).
 */
export function parseContentDispositionFilename(header: string | null): string | null {
	if (!header) return null;

	// RFC 5987 form (priority) : `filename*=UTF-8''<percent>` or `filename*=UTF-8'<lang>'<percent>`.
	const rfc5987 = header.match(/filename\*\s*=\s*UTF-8'[^']*'([^;\r\n]+)/i);
	if (rfc5987 && rfc5987[1]) {
		try {
			const decoded = decodeURIComponent(rfc5987[1].trim());
			if (decoded.length > 0) return decoded;
		} catch {
			// Percent-decode failure → fall through to RFC 6266 form below.
		}
	}

	// RFC 6266 fallback : `filename="…"` (ASCII strict).
	const rfc6266 = header.match(/filename\s*=\s*"([^"]+)"/i);
	if (rfc6266 && rfc6266[1]) {
		return rfc6266[1];
	}

	// RFC 6266 unquoted : `filename=…` (rare, accepté défensivement).
	// Pass 1 code-review H6 (C4 Blind F02/F09 + C4-ECH-H2) — lookahead négatif
	// `(?!\*)` pour exclure les tokens `filename*=...` : header `filename*=UTF-8''`
	// (RFC 5987 valeur vide) ne doit PAS être matché par la regex unquoted, sinon
	// retour `"UTF-8''"` au lieu de `null`. La regex `\s*=` matche zéro espace, donc
	// le lookahead vérifie le caractère immédiatement après `filename` lui-même.
	const rfc6266Unq = header.match(/filename(?!\*)\s*=\s*([^;\s]+)/i);
	if (rfc6266Unq && rfc6266Unq[1]) {
		return rfc6266Unq[1];
	}

	return null;
}

/**
 * Déclenche le download navigateur via un lien `<a download>` éphémère.
 *
 * **Decision §triggerDownload-reuse** (Pass 1 BH-MEDIUM-02) : duplication
 * locale de la fn `triggerDownload` de `reports.api.ts:237` (~10 lignes) —
 * refactor d'extraction vers `lib/shared/utils/download.ts` reporté Epic 15
 * v0.2 si > 2 features dupliquent.
 *
 * Cleanup robuste (Pass 1 code-review M11) : `removeChild` + `revokeObjectURL`
 * dans un `finally` pour éviter une fuite mémoire si `a.click()` jette
 * (CSP, popup blocker, etc.).
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
