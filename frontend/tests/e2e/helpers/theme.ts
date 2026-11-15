/**
 * Lecture des jetons de couleur du `@theme` d'`app.css` — fonctions pures.
 *
 * Extraites de la garde `button-destructive.test.ts` (Story 28-1, #640) à la
 * Story 36-1 (#1165, AC8), dont la garde `account-autocomplete-classes.test.ts`
 * fait la même lecture : une seule source pour les deux (DRY).
 *
 * Pourquoi lire le thème : Tailwind v4 n'engendre une classe de couleur que pour
 * un jeton défini. Une classe qui nomme un jeton absent (`bg-accent`) ne produit
 * aucune règle, et rien ne le signale.
 */

/**
 * Bloc `@theme { … }` d'une feuille (le premier), **commentaires retirés** : un
 * jeton écrit en commentaire (`/* --color-card: #fff; *\/`) n'est pas défini.
 */
export function blocTheme(css: string): string {
	const sansCommentaires = css.replace(/\/\*[\s\S]*?\*\//g, '');
	const debut = sansCommentaires.indexOf('@theme');
	if (debut < 0) throw new Error('app.css : bloc @theme introuvable');
	const ouverture = sansCommentaires.indexOf('{', debut);
	const fermeture = sansCommentaires.indexOf('\n}', ouverture);
	return sansCommentaires.slice(ouverture + 1, fermeture);
}

/** Valeur `#rrggbb` du jeton `--color-<nom>` dans le bloc `theme`, ou `undefined`. */
export function jetonDans(nom: string, theme: string): string | undefined {
	const m = new RegExp(`--color-${nom}:\\s*(#[0-9a-f]{6})\\s*;`, 'i').exec(theme);
	return m?.[1];
}

/** Le jeton `--color-<nom>` est-il déclaré dans le bloc `theme` (toute valeur) ? */
export function declareDans(nom: string, theme: string): boolean {
	return new RegExp(`--color-${nom}\\s*:`).test(theme);
}

/** `mot` figure-t-il comme mot entier (séparé par des blancs) dans `classes` ? */
export function motEntier(classes: string, mot: string): boolean {
	const echappe = mot.replace(/[.*+?^${}()|[\]\\/]/g, '\\$&');
	return new RegExp(`(^|\\s)${echappe}(\\s|$)`).test(classes);
}

/**
 * Jetons de couleur nommés par une liste de classes Tailwind, par utilité :
 * `hover:bg-border` → `{ utilite: 'bg', jeton: 'border' }`. Variantes (`hover:`)
 * et opacité (`/10`) retirées ; les classes hors `bg-`, `text-`, `border-`, `ring-`
 * sont ignorées. ⚠️ Une taille (`text-sm`) serait lue comme un jeton `sm` : ne
 * passer que des listes de classes de couleur (un jeton inconnu y rougit alors).
 */
export function jetonsDesClasses(classes: string): { utilite: string; jeton: string }[] {
	return classes
		.split(/\s+/)
		.filter(Boolean)
		.map((c) => c.slice(c.lastIndexOf(':') + 1).replace(/\/\d+$/, ''))
		.map((c) => /^(bg|text|border|ring)-(.+)$/.exec(c))
		.filter((m): m is RegExpExecArray => m !== null)
		.map((m) => ({ utilite: m[1], jeton: m[2] }));
}
