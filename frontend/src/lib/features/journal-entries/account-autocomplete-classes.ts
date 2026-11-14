/**
 * Classes d'une option de la liste d'`AccountAutocomplete` — Story 36-1 (#1165, D1).
 *
 * Une seule source, lue par le composant ET par la garde de jetons
 * `account-autocomplete-classes.test.ts` (AC8).
 *
 * Pourquoi des jetons du thème, et lesquels : l'ancienne `bg-accent` nommait un
 * jeton que le `@theme` d'`app.css` ne définit pas — Tailwind v4 n'en tirait
 * aucune règle, et la désignation ne se voyait nulle part. La désignation se
 * distingue du fond de la liste (blanc) et du survol (gris clair) par sa
 * luminance même : fond `primary` plein, texte `primary-foreground` (contrastes
 * calculés par la garde AC8).
 */

/**
 * La liste d'options (`<ul role="listbox">`). Son fond (`bg-popover`) est celui
 * contre lequel la garde AC8 mesure la désignation : il se lit ICI, non dans un
 * nom écrit dans le test (revue P1, B4).
 */
export const CLASSE_LISTE =
	'absolute z-20 mt-1 max-h-60 w-full overflow-auto rounded-md border border-border bg-popover shadow-md';

/** Option désignée au clavier — la seule qu'*Entrée* choisit. */
export const CLASSE_DESIGNEE = 'bg-primary text-primary-foreground';

/**
 * Survol à la souris d'une option **non désignée** — un retour de pointeur, non
 * un choix (*Entrée* ne choisit pas l'option survolée). Jamais posé sur
 * l'option désignée : son texte blanc passerait sur un gris clair.
 */
export const CLASSE_SURVOL = 'hover:bg-border';
