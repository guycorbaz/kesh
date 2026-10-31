/**
 * Le texte à afficher pour une valeur attrapée — Story 28-2 (#520).
 *
 * # Le défaut qu'il ferme
 *
 * `ApiError` est une **interface** (`$lib/shared/types/api`), pas une
 * sous-classe d'`Error` : le client HTTP lève des objets simples
 * `{ code, message, status }`. Le motif
 *
 * ```ts
 * e instanceof Error ? e.message : String(e)
 * ```
 *
 * rendait donc **« [object Object] »** pour toute erreur de l'API. Tout `catch`
 * qui affiche une erreur passe par `errorMessageOf` ; la garde
 * `erreurs-en-texte.test.ts` interdit le motif dans les sources.
 *
 * # Règles, dans cet ordre
 *
 * 1. Une `ApiError` dont le `message` est une chaîne non vide → ce message, tel
 *    quel (le serveur l'a traduit, sauf les refus de validation écrits en dur
 *    dans les routes, rendus tels quels (#710) ; les messages que fabrique le
 *    client sont traduits à la source, `api-client.ts`).
 * 2. Une `ApiError` dont le `message` est vide, blanc ou n'est pas une chaîne
 *    → le repli de l'appelant s'il en donne un, sinon le message générique
 *    traduit.
 * 3. Toute autre valeur — `Error`, `TypeError`, `DOMException`, chaîne, nombre,
 *    `null`, objet quelconque — → le repli de l'appelant s'il en donne un, sinon
 *    le message générique traduit, **et**, dans les deux cas, la valeur est
 *    journalisée (`console.error`) pour le diagnostic. Le `message` d'une
 *    `Error` n'est pas affiché : il est technique et non traduit
 *    (« Failed to fetch », « No candidate for txId=5 ») — C-28-2-3.
 *
 * # Le repli (Story v017-2, #809)
 *
 * Le second paramètre, optionnel, est un message **propre à l'appelant** — déjà
 * traduit, typiquement `msg('onboarding-address-save-error', …)` — qui remplace
 * le générique aux règles 2 et 3. Un repli vide ou blanc est traité comme absent.
 * Sans repli, le comportement est celui de la Story 28-2.
 *
 * Le rendu n'est jamais `'[object Object]'`, ni la chaîne vide.
 */

import { isApiError } from './api-error';
import { i18nMsg } from './i18n.svelte';

/** Le message générique — repli identique aux autres sites de la clé. */
function messageGenerique(): string {
	return i18nMsg('error-unexpected', 'Erreur inattendue.');
}

/**
 * Texte lisible et traduit d'une valeur attrapée dans un `catch`.
 *
 * @param err la valeur attrapée
 * @param repli message de l'appelant, à la place du générique quand le serveur ne
 *   donne pas de motif lisible (règles 2 et 3) ; vide ou blanc = absent
 */
export function errorMessageOf(err: unknown, repli?: string): string {
	const defaut = (): string =>
		typeof repli === 'string' && repli.trim() !== '' ? repli : messageGenerique();
	if (isApiError(err)) {
		const message: unknown = err.message;
		return typeof message === 'string' && message.trim() !== '' ? message : defaut();
	}
	console.error('[errorMessageOf]', err);
	return defaut();
}
