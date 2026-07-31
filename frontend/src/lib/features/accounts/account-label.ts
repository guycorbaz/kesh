/**
 * Libellé d'affichage d'un compte de produit porté par une ligne de pièce
 * (Story 16-1b, AC6-bis).
 *
 * # Pourquoi ce fichier existe
 *
 * Les écrans de détail **facture** et **avoir** résolvent le même identifiant de
 * compte en « numéro — libellé », avec une seule différence de règle sur le cas
 * `null`. Écrite deux fois, la partie commune divergerait (un écran affichant
 * `#3900` là où l'autre affiche le nom du compte), et la différence de règle
 * cesserait d'être lisible comme une décision — elle passerait pour un oubli.
 */

import type { AccountResponse } from './accounts.types';
import { i18nMsg } from '$lib/shared/utils/i18n.svelte';

/** `3200 — Honoraires`, ou `#3200` si le compte n'est pas résoluble. */
function label(accounts: AccountResponse[], id: number): string {
	const a = accounts.find((x) => x.id === id);
	return a ? `${a.number} — ${a.name}` : `#${id}`;
}

/**
 * Libellé côté **facture** : une ligne à `null` nomme le compte par défaut de la
 * société, suivi de la mention « (défaut) ».
 *
 * La mention distingue « ce compte a été choisi » de « ce compte s'applique par
 * défaut ». Sans elle, un vide laisserait croire à un oubli — or `null` veut dire
 * « suivre le défaut », pas « aucune imputation ».
 *
 * Tiret si le défaut société n'est pas configuré, **et** sur toute pièce qui
 * n'est plus un brouillon : l'écriture y est déjà passée, il n'y a plus de repli
 * à annoncer.
 */
export function invoiceRevenueAccountLabel(
	accounts: AccountResponse[],
	revenueAccountId: number | null,
	defaultRevenueAccountId: number | null,
	isDraft: boolean
): string {
	if (revenueAccountId !== null) return label(accounts, revenueAccountId);
	// Passe 2 de revue : la mention n'a de sens qu'au FUTUR. Sur une facture
	// validée ou annulée, l'écriture est déjà passée — annoncer un repli qui
	// n'aura pas lieu serait le même mensonge que celui qu'on refuse côté avoir.
	// La population existe : le backfill de 16-1a-bis laisse délibérément à
	// `NULL` les pièces dont l'écriture a été retouchée, et exclut les
	// `cancelled`.
	if (!isDraft) return '—';
	if (defaultRevenueAccountId === null) return '—';
	return `${label(accounts, defaultRevenueAccountId)} ${i18nMsg('account-label-default-suffix', '(défaut)')}`;
}

/**
 * Libellé côté **avoir** : une ligne à `null` affiche un **tiret**, jamais la
 * mention « (défaut) ».
 *
 * Ce n'est pas une omission. Côté facture, `null` se lit au futur — l'écriture
 * n'est pas passée, nommer le compte cible est une information juste. Côté avoir,
 * **l'écriture est déjà passée** : afficher « 3000 (défaut) » affirmerait un
 * repli qui n'aura pas lieu, soit un mensonge sur une pièce comptable.
 *
 * Le cas est résiduel — 16-1a matérialise le compte à la validation. Restent les
 * avoirs antérieurs au déploiement que le backfill de 16-1a-bis n'a pas pu
 * reprendre (écriture retouchée à la main).
 */
export function creditNoteRevenueAccountLabel(
	accounts: AccountResponse[],
	revenueAccountId: number | null
): string {
	if (revenueAccountId === null) return '—';
	return label(accounts, revenueAccountId);
}
