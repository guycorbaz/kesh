/**
 * Story 34-4 (#828, AC-5, AC-6) — le signal « transaction déjà comptabilisée » de la page de
 * réconciliation.
 *
 * Le serveur compte, pour chaque transaction en attente, ce qui lui ressemble déjà aux livres
 * (`alreadyBooked`, `GET /reconciliation/proposals`) ; ce module en tire **le texte de
 * l'avertissement** de la ligne et **la condition** qui fait demander confirmation avant
 * *Accepter*, *Écriture manuelle* et *Éclater*. Une seule condition pour les deux : une ligne
 * est signalée si et seulement si elle porte un avertissement.
 *
 * Rien n'est refusé, rattaché ni rejeté d'office (règle « un appariement automatique propose,
 * il ne crée jamais ») : l'avertissement renvoie au bouton *Rattacher à une écriture existante*
 * de la ligne, ou au rejet pour le second côté d'un virement.
 *
 * ⛔ **Pas de sélecteur Fluent** (`{ $count -> … }`) dans ces messages : le client reçoit les
 * messages pré-résolus, sans arguments (`crates/kesh-i18n/src/loader.rs`, `all_messages`), et
 * `i18nMsg` ne remplace que `{ $nom }` — un sélecteur arriverait figé sur sa variante par
 * défaut. D'où deux clés, `-one` et `-other`, choisies ici (choix C-e342-21).
 *
 * ⚠️ **Provisoire** (fiche 34-4, AC-5, F4-2) : la Story 34-11b rendra l'écriture d'un virement
 * candidate de l'autre côté, si bien que `matchedElsewhereCount` deviendra une partie
 * d'`entryCount` ; elle réécrit alors la condition des cas (b) et (c) et leurs textes.
 */
import { formatSwissDate } from '$lib/features/reports/reports.api';
import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
import type { ReconciliationProposal, TransactionSummary } from './reconciliation.types';

/**
 * Le texte de l'avertissement d'une ligne, ou `null` si elle n'en porte pas.
 *
 * Avec `k = settlementCount + entryCount` et `m = matchedElsewhereCount` :
 * - (a) `k > 0`, `m = 0` : la phrase « déjà comptabilisée », au singulier si `k = 1` ;
 * - (b) `k = 0`, `m > 0` : « l'autre côté d'un virement », avec la consigne de rejeter ;
 * - (c) `k > 0`, `m > 0` : (a), puis la phrase du cas combiné, sans la consigne de rejeter
 *   (choix C-e342-7) ;
 * - rien si `k = 0` et `m = 0`, ni pour `alreadyBooked: null`. Jamais « 0 écriture ».
 *
 * Les clés sont littérales, une par appel : une clé calculée échapperait aux gardes i18n.
 */
export function alreadyBookedWarning(p: ReconciliationProposal): string | null {
	const counts = p.alreadyBooked;
	if (!counts) return null;
	const k = counts.settlementCount + counts.entryCount;
	const m = counts.matchedElsewhereCount;
	const parts: string[] = [];
	if (k === 1) {
		parts.push(
			i18nMsg(
				'reconciliation-labels-already-booked-one',
				'Cette transaction semble déjà comptabilisée : une écriture non rapprochée du compte de la banque lui correspond. Rattachez-la (« Rattacher à une écriture existante ») plutôt que de passer une écriture : elle serait comptée deux fois.',
			),
		);
	} else if (k > 1) {
		parts.push(
			i18nMsg(
				'reconciliation-labels-already-booked-other',
				'Cette transaction semble déjà comptabilisée : { $count } écritures non rapprochées du compte de la banque lui correspondent. Rattachez-la (« Rattacher à une écriture existante ») plutôt que de passer une écriture : elle serait comptée deux fois.',
				{ count: k },
			),
		);
	}
	if (m > 0 && k === 0) {
		parts.push(
			i18nMsg(
				'reconciliation-labels-already-booked-elsewhere',
				'Une transaction de sens contraire, sur un autre de vos comptes bancaires, est déjà rapprochée d’une écriture du même montant : c’est probablement l’autre côté d’un virement entre vos comptes. Ne passez pas d’écriture pour elle : rejetez-la.',
			),
		);
	} else if (m > 0) {
		parts.push(
			i18nMsg(
				'reconciliation-labels-already-booked-combined',
				'Si rien de ce que propose « Rattacher à une écriture existante » ne lui correspond, elle est peut-être l’autre côté d’un virement entre vos comptes : une transaction de sens contraire, sur un autre de vos comptes bancaires, est rapprochée d’une écriture du même montant.',
			),
		);
	}
	return parts.length > 0 ? parts.join(' ') : null;
}

/**
 * La ligne est-elle signalée ? — exactement quand elle porte un avertissement
 * ([`alreadyBookedWarning`]) : c'est elle qui fait demander confirmation (AC-6).
 */
export function isAlreadyBooked(p: ReconciliationProposal): boolean {
	return alreadyBookedWarning(p) !== null;
}

/**
 * « date · montant devise · contrepartie » — la désignation d'une transaction dans le bilan
 * d'un lot (Story 15-5c) et dans la confirmation (Story 34-4). Date en forme suisse ; une
 * contrepartie vide ou absente n'est pas nommée.
 */
export function describeTransaction(t: TransactionSummary): string {
	return [formatSwissDate(t.bookingDate), `${t.amount} ${t.currency}`, t.counterpartyName]
		.filter((v) => v !== null && v !== undefined && v !== '')
		.join(' · ');
}
