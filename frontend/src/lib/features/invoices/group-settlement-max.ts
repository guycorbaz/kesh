/**
 * Story 43-7 (#823) — la lecture du champ « Écart admis pour un règlement de plusieurs
 * factures » de *Administration → Facturation* : un montant de 0 à 100.00, au centime.
 *
 * Même grammaire que le journal (`readJournalAmount`) : « 1.250 » est ambigu, « 1'250 » et
 * « 1,25 » sont lus. Pas d'import de l'i18n ici : l'écran choisit les textes.
 */

import Big from 'big.js';
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

/**
 * Revue P2 (B2-1) — la saisie `raw` change-t-elle le plafond que le serveur a rendu
 * (`stored`) ? Les **valeurs** se comparent, par `readGroupMax` puis une égalité décimale :
 * « 5,01 » contre « 5.01 », « 5,00 » contre « 5.00 » ne sont pas des changements. Une saisie
 * illisible compte pour un changement (l'écran la refuse avant tout envoi).
 */
export function groupMaxChanged(raw: string, stored: string | undefined): boolean {
	const now = readGroupMax(raw);
	if (!now.ok) return true;
	if (stored === undefined) return true;
	const before = readGroupMax(stored);
	if (!before.ok) return true;
	return !new Big(now.payload).eq(new Big(before.payload));
}
