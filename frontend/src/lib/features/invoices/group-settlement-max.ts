/**
 * Story 43-7 (#823) — la lecture du champ « Écart admis pour un règlement de plusieurs
 * factures » de *Administration → Facturation* : un montant de 0 à 100.00, au centime.
 *
 * Même grammaire que le journal (`readJournalAmount`) : « 1.250 » est ambigu, « 1'250 » et
 * « 1,25 » sont lus. Pas d'import de l'i18n ici : l'écran choisit les textes.
 */

import { readJournalAmount } from '$lib/features/journal-entries/balance';

/** Le plafond le plus haut que le serveur admet (la `CHECK` de la colonne). */
export const GROUP_MAX_CEILING = 100;

export type GroupMaxReading =
	| { ok: true; payload: string }
	| { ok: false; problem: 'empty' | 'ambiguous' | 'unreadable' | 'scale' | 'range' };

/** Lit la saisie rognée du champ ; `payload` est la chaîne décimale à envoyer au serveur. */
export function readGroupMax(raw: string): GroupMaxReading {
	const r = readJournalAmount(raw.trim());
	if (r.kind !== 'clear') return { ok: false, problem: r.kind };
	// Les décimales se comptent sur la forme canonique : « 5.0000 » est un 5 au centime.
	const canonical = r.value.toFixed();
	const dot = canonical.indexOf('.');
	if (dot >= 0 && canonical.length - dot - 1 > 2) return { ok: false, problem: 'scale' };
	if (r.value.gt(GROUP_MAX_CEILING)) return { ok: false, problem: 'range' };
	return { ok: true, payload: r.payload };
}
