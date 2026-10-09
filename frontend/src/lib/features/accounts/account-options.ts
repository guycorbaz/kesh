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

import type { AccountResponse, AccountRole } from './accounts.types';

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

// ---------------------------------------------------------------------------
// Story 15-6b (#474) — les écrans de règlement ne proposent plus le compte soldé
// ---------------------------------------------------------------------------

/**
 * Identifiants des comptes portant le rôle `role` (Story 15-6b, AC8 ; signature
 * partagée avec la 15-6c, choix C-15-6-14).
 *
 * **Pourquoi le rôle** : un écran de règlement n'a pas l'écriture de vente sous
 * la main ; le rôle `Receivable` / `Payable` est un singleton par société, dont
 * les réglages de facturation sont dérivés. ⚠️ **La garde serveur, exacte, reste
 * l'autorité** : elle compare la contrepartie au compte que porte l'écriture de
 * vente ou d'achat. Si le réglage a été déplacé hors du compte de rôle, l'écran
 * peut proposer un compte que le serveur refusera, avec un message clair —
 * jamais l'inverse.
 *
 * ⛔ **Calculer les ids sur la liste que l'écran REÇOIT, avant le filtre
 * `active && postable`** : sans quoi un compte bancaire lié à un compte
 * débiteurs devenu **non imputable** ne serait pas écarté.
 *
 * ⚠️ **Les comptes archivés n'y sont pas, et c'est voulu** (choix C-15-6-17) :
 * les écrans chargent le plan sans eux. Un compte bancaire lié à un compte
 * archivé reste donc proposé — mais aucune écriture ne peut viser un compte
 * archivé (garde `active` du serveur) : le serveur suffit.
 */
export function accountIdsWithRole(accounts: AccountResponse[], role: AccountRole): Set<number> {
	return new Set(accounts.filter((a) => a.role === role).map((a) => a.id));
}

/** Les comptes dont l'identifiant n'est pas dans `ids` (Story 15-6b, AC8). */
export function withoutAccountIds(accounts: AccountResponse[], ids: Set<number>): AccountResponse[] {
	return accounts.filter((a) => !ids.has(a.id));
}

/**
 * Les comptes bancaires dont le compte du grand livre (`journalAccountId`)
 * n'est pas dans `ids` (Story 15-6b, AC8). Un compte bancaire sans compte lié
 * est gardé : ce filtre ne juge que le lien au compte soldé.
 */
export function bankAccountsNotLinkedTo<T extends { journalAccountId: number | null }>(
	bankAccounts: T[],
	ids: Set<number>,
): T[] {
	return bankAccounts.filter((b) => b.journalAccountId === null || !ids.has(b.journalAccountId));
}
