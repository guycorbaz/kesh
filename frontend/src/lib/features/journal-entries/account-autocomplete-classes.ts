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

/** Option désignée au clavier — la seule qu'*Entrée* choisit. */
export const CLASSE_DESIGNEE = 'bg-primary text-primary-foreground';

/**
 * Survol à la souris d'une option **non désignée** — un retour de pointeur, non
 * un choix (*Entrée* ne choisit pas l'option survolée). Jamais posé sur
 * l'option désignée : son texte blanc passerait sur un gris clair.
 */
export const CLASSE_SURVOL = 'hover:bg-border';
