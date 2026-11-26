/**
 * Story 55-1a — un `page` de `$app/state` dont l'`url` est **réactive**, et un historique simulé, pour
 * les seuls tests de l'URL des Rapports (`reports-page-url.test.ts`). Patron de
 * `journal-entries/[id]/page-state.test-helper.svelte.ts`, qui ne porte que `params`.
 *
 * SvelteKit ne remonte pas la page quand seuls les paramètres de requête changent : seul `page.url`
 * change, et l'effet de la page l'applique. Un objet simple dans `vi.mock` ne déclencherait rien.
 *
 * ⚠️ Module de test : il n'est importé que par `reports-page-url.test.ts`.
 */

/** L'état réactif lu par la page à la place de `page` de `$app/state`. */
export const pageState = $state({ url: new URL('http://localhost/reports') });

/** L'historique du navigateur simulé : les entrées et la position courante. */
export const historique = { entrees: [] as string[], position: 0 };

/** Remet l'historique à une seule entrée, l'adresse d'arrivée (chargement de la page). */
export function arriverSur(href: string): void {
	const url = new URL(href, 'http://localhost');
	historique.entrees = [url.href];
	historique.position = 0;
	pageState.url = url;
}

/** Une navigation : empile une entrée (lien, `goto` sans `replaceState`) ou remplace la courante. */
export function naviguer(href: string, remplacer = false): void {
	const url = new URL(href, pageState.url);
	if (remplacer) {
		historique.entrees[historique.position] = url.href;
	} else {
		historique.entrees = historique.entrees.slice(0, historique.position + 1);
		historique.entrees.push(url.href);
		historique.position += 1;
	}
	pageState.url = url;
}

/** *Retour* du navigateur (`popstate`). */
export function retour(): void {
	historique.position -= 1;
	pageState.url = new URL(historique.entrees[historique.position]);
}

/** *Suivant* du navigateur. */
export function suivant(): void {
	historique.position += 1;
	pageState.url = new URL(historique.entrees[historique.position]);
}

/** La recherche courante, sans le `?`. */
export function rechercheCourante(): string {
	return pageState.url.search.replace(/^\?/, '');
}
