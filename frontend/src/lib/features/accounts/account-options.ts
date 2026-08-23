/**
 * Options d'un `<select>` de compte : la liste filtrée, **plus la valeur déjà
 * enregistrée** quand celle-ci n'y figure plus (issue #271).
 *
 * # Le défaut que ce fichier ferme
 *
 * Les `<select>` **natifs** de configuration filtrent leurs options sur
 * `postable` depuis la Story 14-3b — le compte y sera posté à la génération
 * d'écriture, et proposer un compte non-postable ferait rejeter l'écriture en
 * aval. Le filtre est juste ; ce qu'il ne prévoyait pas, c'est qu'un compte
 * **déjà configuré** devienne non-postable **après coup**. Le cas n'est pas
 * théorique : lui ajouter un sous-compte bascule le parent à
 * `postable = FALSE` (règle 14-3a).
 *
 * L'enchaînement est alors muet de bout en bout :
 *
 * 1. la valeur enregistrée n'a plus d'`<option>` correspondante, donc le
 *    navigateur met `selectedIndex = -1` et **le champ s'affiche vide** — alors
 *    que la base porte bien la valeur ;
 * 2. l'administrateur, devant un champ requis vide, y touche — réflexe
 *    naturel — et l'état bascule **silencieusement** sur `null` ;
 * 3. l'enregistrement écrit `NULL` : le backend accepte `None` sans broncher ;
 * 4. la facturation suivante échoue en `ConfigurationRequired`, **loin de la
 *    cause**, sur un écran qui n'a rien à voir.
 *
 * Rien ne rougit à aucune des quatre étapes. C'est le mode d'échec que ce dépôt
 * appelle le défaut muet, appliqué à une configuration.
 *
 * # Ce que la fonction fait, et ce qu'elle ne fait PAS
 *
 * Elle **n'ouvre pas** le filtre : un compte non-postable ne devient jamais
 * sélectionnable. Elle réintroduit exclusivement la valeur **courante**, pour
 * qu'elle reste lisible et qu'un `change` involontaire ne puisse pas l'effacer.
 * Choisir sciemment un autre compte reste possible ; perdre le sien par
 * accident ne l'est plus.
 *
 * ⚠️ `AccountAutocomplete.svelte` — le composant des quatre écrans de saisie
 * d'écriture — n'est **pas** concerné, et ne doit pas être « aligné » sur ce
 * helper : son `$effect` résout déjà le libellé de la valeur courante sur la
 * liste **complète** des comptes. Le défaut est propre aux `<select>` natifs,
 * dont les options sont l'unique source de vérité de ce qui est affichable.
 */

import type { AccountResponse } from './accounts.types';

/**
 * Rend `filtered`, en y ajoutant le compte d'identifiant `currentId` s'il
 * existe dans `all` sans figurer dans `filtered`.
 *
 * Le compte réintroduit est placé **en tête** : il est la valeur courante, et
 * une liste de comptes est ordonnée par numéro, non par pertinence — l'insérer
 * à sa place numérique le rendrait indiscernable des options légitimes.
 *
 * @param filtered liste déjà filtrée (typiquement sur `active` et `postable`)
 * @param currentId identifiant enregistré, ou `null` si le champ est vide
 * @param all liste complète, seule capable de résoudre un compte écarté par le filtre
 */
export function withCurrentAccount(
	filtered: AccountResponse[],
	currentId: number | null | undefined,
	all: AccountResponse[],
): AccountResponse[] {
	if (currentId === null || currentId === undefined) return filtered;
	if (filtered.some((a) => a.id === currentId)) return filtered;

	const current = all.find((a) => a.id === currentId);
	// Un identifiant que la liste complète ne résout pas — compte supprimé, ou
	// liste pas encore chargée — ne donne aucune option à fabriquer. Rendre
	// `filtered` tel quel laisse le champ vide, ce qui est le comportement
	// d'avant ce helper : on ne peut pas afficher un compte qu'on ne connaît pas.
	if (!current) return filtered;

	return [current, ...filtered];
}
