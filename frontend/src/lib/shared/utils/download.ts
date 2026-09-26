/**
 * Téléchargement d'un fichier servi par l'API — module partagé (Story 25-1c-b1, #378).
 *
 * Les deux fonctions sont reprises **à l'identique** de
 * `features/export/exports.api.ts`, y compris la forme `try/finally` de
 * `triggerDownload` : la libération de l'URL objet a lieu même si `click()`
 * jette.
 *
 * ⚠️ **Aucune des copies existantes n'est encore migrée ici** — il en existe
 * sept (quatre fonctions dans des `.api.ts`, trois écrites en ligne dans des
 * pages, au comportement différent) et deux définitions de
 * `parseContentDispositionFilename`. Leur regroupement est suivi par l'issue
 * #438. Tout NOUVEL écran importe d'ici, et n'en écrit pas une huitième.
 */

/**
 * Extrait le nom de fichier d'un en-tête `Content-Disposition`.
 *
 * 1. **RFC 5987** (`filename*=UTF-8''<percent>`, prioritaire) — décode les
 *    caractères non ASCII ; la forme avec tag de langue
 *    (`filename*=UTF-8'fr-CH'…`) est acceptée.
 * 2. **RFC 6266** (`filename="…"`), puis sa forme sans guillemets.
 *
 * Rend `null` pour un en-tête absent ou vide, ou sans nom de fichier.
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
 * Déclenche le téléchargement navigateur d'un `Blob` par un lien `<a download>`
 * éphémère. L'ancre est retirée et l'URL objet libérée dans un `finally`, même
 * si `click()` jette (CSP, bloqueur de fenêtres).
 */
export function triggerDownload(blob: Blob, filename: string): void {
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
