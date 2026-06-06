/**
 * Copie de texte dans le presse-papiers, **robuste hors secure-context**.
 *
 * Story 17-2b — le secret PAT (`kesh_pat_…`) n'est affiché qu'une fois ;
 * l'utilisateur doit pouvoir le copier de façon fiable, y compris sur un
 * déploiement **HTTP LAN** (NAS Synology) où `navigator.clipboard` est
 * `undefined` (API réservée aux secure-contexts HTTPS / localhost).
 * Cf. feedback projet « pas d'API secure-context-only en runtime » (#145).
 *
 * Stratégie : tenter l'API moderne si disponible, sinon fallback sur
 * `document.execCommand('copy')` via un `<textarea>` hors-écran (déprécié mais
 * universellement supporté, y compris sur HTTP).
 *
 * @returns `true` si la copie a réussi, `false` sinon (le caller affiche alors
 *          un message invitant à copier manuellement).
 */
export async function copyToClipboard(text: string): Promise<boolean> {
	// API moderne (secure-context uniquement : HTTPS ou http://localhost).
	if (typeof navigator !== 'undefined' && navigator.clipboard?.writeText) {
		try {
			await navigator.clipboard.writeText(text);
			return true;
		} catch {
			// Permission refusée ou contexte non-sécurisé malgré la présence de
			// l'API → on tombe sur le fallback ci-dessous.
		}
	}

	// Fallback `execCommand('copy')` — fonctionne sur HTTP LAN.
	if (typeof document === 'undefined') return false;
	const textarea = document.createElement('textarea');
	try {
		textarea.value = text;
		// Hors-écran + non-focusable visuellement, mais sélectionnable.
		textarea.setAttribute('readonly', '');
		textarea.style.position = 'fixed';
		textarea.style.top = '-9999px';
		textarea.style.opacity = '0';
		document.body.appendChild(textarea);
		textarea.select();
		return document.execCommand('copy');
	} catch {
		return false;
	} finally {
		// `finally` : retire le textarea même si `execCommand` throw (sinon
		// élément orphelin accumulé dans le DOM à chaque échec — code-review P1).
		textarea.remove();
	}
}
