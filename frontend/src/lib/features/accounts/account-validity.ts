/**
 * Prédicat d'« utilisabilité » d'un compte pour une valeur **déjà persistée**
 * (Story 16-1b, D7 / AC2 / AC8).
 *
 * # Pourquoi ce fichier existe
 *
 * Deux consommateurs doivent rendre le **même** verdict :
 *
 * - `AccountAutocomplete` — pour afficher le marqueur textuel sur le champ ;
 * - `InvoiceForm` — pour bloquer l'enregistrement et nommer les lignes fautives.
 *
 * Si la règle était écrite deux fois, elle divergerait : un champ marqué sans
 * blocage (l'utilisateur croit pouvoir enregistrer), ou un blocage sans marqueur
 * (l'utilisateur ne sait pas quelle ligne corriger). Les deux sont pires que
 * l'absence de fonctionnalité. D'où une source unique.
 *
 * # Ce que ce prédicat ne fait PAS
 *
 * Il ne filtre pas le dropdown. Les propositions restent
 * `accounts.filter((a) => a.active && a.postable)` dans le composant : charger
 * la liste complète (`fetchAccounts(true)`, D11) sert **uniquement** à résoudre
 * le libellé d'une valeur persistée, jamais à la rendre re-sélectionnable.
 */

import type { AccountResponse, AccountType } from './accounts.types';

export interface AccountValidityOptions {
	/** Type attendu ; `undefined` = aucun contrôle de type. */
	requiredAccountType?: AccountType;
	/**
	 * Compte exempté du **seul** critère `postable`.
	 *
	 * Miroir de 16-1a D3-bis : le compte de produit par défaut de la société peut
	 * devenir non-imputable sans intention, et le backend l'accepte quand même.
	 * Si le frontend le marquait invalide et bloquait l'enregistrement,
	 * l'utilisateur serait **enfermé** — incapable de sauver son brouillon.
	 * **Le frontend ne doit jamais bloquer ce que le backend accepte.**
	 *
	 * Les critères `active` et `requiredAccountType` restent appliqués : le seul
	 * relâchement porte sur `postable`.
	 */
	postableExemptAccountId?: number | null;
}

/**
 * `true` si la valeur persistée n'est plus utilisable et doit être signalée.
 *
 * Un compte **non résolu** (`undefined`, absent de la liste fournie) renvoie
 * `false` : il n'y a rien à qualifier, et bloquer sur une liste incomplète
 * enfermerait l'utilisateur pour une raison qui ne le concerne pas. C'est
 * cohérent avec le mode dégradé de D10 — l'indisponibilité de la liste des
 * comptes ne doit pas empêcher de saisir une facture.
 */
export function isAccountUnusable(
	account: AccountResponse | undefined,
	options: AccountValidityOptions = {}
): boolean {
	if (!account) return false;
	if (!account.active) return true;
	if (
		options.requiredAccountType !== undefined &&
		account.accountType !== options.requiredAccountType
	) {
		return true;
	}
	return !account.postable && account.id !== (options.postableExemptAccountId ?? null);
}
