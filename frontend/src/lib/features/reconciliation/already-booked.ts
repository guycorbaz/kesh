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
 *   serveur ne rend pas se lit comme (b′), jamais comme une ligne muette) ; au pluriel si
 *   `m ≥ 2` (clé `-transfer-other`, revue P2 E2-3) ;
 * - (c′) `m > 0`, `k − m > 0` : (a) **pour les `k − m` non rapprochées seules**, puis les `m`
 *   écritures de virement déjà rapprochées de l'autre côté, au singulier ou au pluriel (revue
 *   P1, B1-1 : « N non rapprochées … l'une d'elles est déjà rapprochée » se contredisait) ;
 * - rien si `k = 0` et `m = 0`, ni pour `alreadyBooked: null`. Jamais « 0 écriture ».
 *
 * Les clés sont littérales, une par appel : une clé calculée échapperait aux gardes i18n.
 */
export function alreadyBookedWarning(p: ReconciliationProposal): string | null {
	const counts = p.alreadyBooked;
	if (!counts) return null;
	const k = counts.settlementCount + counts.entryCount;
	const m = counts.matchedElsewhereCount;
	// Revue P2 (E2-3) : (b′) au pluriel quand plusieurs écritures de virement sont proposées.
	if (m > 1 && k - m <= 0) {
		return i18nMsg(
			'reconciliation-labels-already-booked-transfer-other',
			'Cette transaction est probablement l’autre côté d’un virement entre vos comptes : { $count } écritures de virement sont déjà rapprochées de transactions d’autres de vos comptes bancaires. Rattachez-la à celle de son virement (« Rattacher à une écriture existante ») plutôt que de passer une écriture : le virement serait compté deux fois.',
			{ count: m },
		);
	}
	if (m > 0 && k - m <= 0) {
		return i18nMsg(
			'reconciliation-labels-already-booked-transfer',
			'Cette transaction est probablement l’autre côté d’un virement entre vos comptes : l’écriture de ce virement est déjà rapprochée d’une transaction d’un autre de vos comptes bancaires. Rattachez-la à cette écriture (« Rattacher à une écriture existante ») plutôt que de passer une écriture : le virement serait compté deux fois.',
		);
	}
	const parts: string[] = [];
	// Les écritures non rapprochées seules : celles de virement se disent à part (c′).
	const n = k - m;
	if (n === 1) {
		parts.push(
			i18nMsg(
				'reconciliation-labels-already-booked-one',
				'Cette transaction semble déjà comptabilisée : une écriture non rapprochée du compte de la banque lui correspond. Rattachez-la (« Rattacher à une écriture existante ») plutôt que de passer une écriture : elle serait comptée deux fois.',
			),
		);
	} else if (n > 1) {
		parts.push(
			i18nMsg(
				'reconciliation-labels-already-booked-other',
				'Cette transaction semble déjà comptabilisée : { $count } écritures non rapprochées du compte de la banque lui correspondent. Rattachez-la (« Rattacher à une écriture existante ») plutôt que de passer une écriture : elle serait comptée deux fois.',
				{ count: n },
			),
		);
	}
	if (m === 1) {
		parts.push(
			i18nMsg(
				'reconciliation-labels-already-booked-transfer-among-one',
				'S’y ajoute l’écriture d’un virement, déjà rapprochée d’une transaction d’un autre de vos comptes bancaires : si cette transaction est l’autre côté de ce virement, c’est à elle qu’il faut la rattacher.',
			),
		);
	} else if (m > 1) {
		parts.push(
			i18nMsg(
				'reconciliation-labels-already-booked-transfer-among-other',
				'S’y ajoutent { $count } écritures de virement, déjà rapprochées de transactions d’autres de vos comptes bancaires : si cette transaction est l’autre côté de l’un de ces virements, c’est à son écriture qu’il faut la rattacher.',
				{ count: m },
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
