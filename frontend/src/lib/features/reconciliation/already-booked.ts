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
 * de la ligne — pour le second côté d'un virement aussi (Story 34-11b).
 *
 * ⛔ **Pas de sélecteur Fluent** (`{ $count -> … }`) dans ces messages : le client reçoit les
 * messages pré-résolus, sans arguments (`crates/kesh-i18n/src/loader.rs`, `all_messages`), et
 * `i18nMsg` ne remplace que `{ $nom }` — un sélecteur arriverait figé sur sa variante par
 * défaut. D'où deux clés, `-one` et `-other`, choisies ici (choix C-e342-21).
 *
 * **Story 34-11b (#972, AC-10)** : l'écriture d'un virement, rapprochée sur un autre compte
 * bancaire, est **candidate** de l'autre côté — `matchedElsewhereCount` compte, parmi les
 * écritures candidates, celles qui sont déjà rapprochées (une partie d'`entryCount`, par
 * construction du serveur). Le conseil n'est plus de rejeter le second côté, mais de le
 * rattacher à l'écriture du virement.
 */
import { formatSwissDate } from '$lib/features/reports/reports.api';
import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
import type { ReconciliationProposal, TransactionSummary } from './reconciliation.types';

/**
 * Le texte de l'avertissement d'une ligne, ou `null` si elle n'en porte pas.
 *
 * Avec `k = settlementCount + entryCount` et `m = matchedElsewhereCount` (Story 34-11b,
 * AC-10 ; `m ≤ k` par construction du serveur) :
 * - (a) `k > 0`, `m = 0` : la phrase « déjà comptabilisée », au singulier si `k = 1` ;
 * - (b′) `m > 0`, `k − m ≤ 0` : « l'autre côté d'un virement », avec la consigne de
 *   rattacher la transaction à l'écriture du virement (`≤` et non `=` : un `k < m` que le
 *   serveur ne rend pas se lit comme (b′), jamais comme une ligne muette) ;
 * - (c′) `m > 0`, `k − m > 0` : (a), puis « l'une d'elles est l'écriture d'un virement » ;
 * - rien si `k = 0` et `m = 0`, ni pour `alreadyBooked: null`. Jamais « 0 écriture ».
 *
 * Les clés sont littérales, une par appel : une clé calculée échapperait aux gardes i18n.
 */
export function alreadyBookedWarning(p: ReconciliationProposal): string | null {
	const counts = p.alreadyBooked;
	if (!counts) return null;
	const k = counts.settlementCount + counts.entryCount;
	const m = counts.matchedElsewhereCount;
	if (m > 0 && k - m <= 0) {
		return i18nMsg(
			'reconciliation-labels-already-booked-transfer',
			'Cette transaction est probablement l’autre côté d’un virement entre vos comptes : l’écriture de ce virement est déjà rapprochée d’une transaction d’un autre de vos comptes bancaires. Rattachez-la à cette écriture (« Rattacher à une écriture existante ») plutôt que de passer une écriture : le virement serait compté deux fois.',
		);
	}
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
	if (m > 0) {
		parts.push(
			i18nMsg(
				'reconciliation-labels-already-booked-transfer-among',
				'L’une d’elles est l’écriture d’un virement déjà rapprochée sur un autre de vos comptes bancaires : c’est probablement celle-ci qu’il faut rattacher.',
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
