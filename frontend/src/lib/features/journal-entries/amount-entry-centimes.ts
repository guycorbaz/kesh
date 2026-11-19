/**
 * Story 48-10 (#1175, #1176 ; revue P1, B4, B5) — la lecture d'un montant saisi **au centime** :
 * les frais d'un niveau de rappel et le montant minimal d'une facture.
 *
 * Même grammaire que l'écriture (`readJournalAmount`) — « 1'250 » clair, « 1.250 » ambigu —, plus
 * une borne : **deux décimales significatives au plus**, ce que le message illisible annonce
 * (`amountEntryMessage(…, 2)`) et ce que le serveur exige (`scale_within(…, 2)`). « 10.0100 » reste
 * clair (les zéros de fin ne comptent pas) et part `10.01` ; « 10.125 » ou « 0.125 » sont
 * illisibles ici, au lieu d'être refusés par le serveur dans un autre bloc. La borne ne refuse
 * aucune forme que le serveur accepte : il lit déjà la valeur à deux décimales au plus.
 *
 * ⚠️ Exception, héritée de la grammaire du journal et antérieure à cette borne : une forme hors de
 * `readJournalAmount` reste illisible même quand le serveur l'accepterait — plus de quatre décimales
 * écrites, même nulles (« 10.01000 »), « .5 », « 5. », exposant, signe (revue P2, C1).
 */

import { readJournalAmount } from './balance';

/** La lecture d'un montant au centime ; `payload` pour une saisie claire. */
export type CentimesEntryReading =
	| { kind: 'empty' }
	| { kind: 'clear'; payload: string }
	| { kind: 'ambiguous' | 'unreadable' };

/**
 * Lit une saisie **rognée** au centime. `payload` : la charge de `readJournalAmount` (« 1'250 » →
 * `1250`, « 1,25 » → `1.25`), ramenée à deux décimales quand la saisie en écrit davantage, toutes
 * nulles au-delà (« 10.0100 » → `10.01`, « 1250.0000 » → `1250.00`) : le serveur des frais compte
 * les décimales écrites, sans les normaliser.
 */
export function readCentimesEntry(raw: string): CentimesEntryReading {
	const r = readJournalAmount(raw.trim());
	if (r.kind !== 'clear') return { kind: r.kind };
	if (!r.value.round(2, 0).eq(r.value)) return { kind: 'unreadable' };
	const fraction = r.payload.split('.')[1] ?? '';
	return { kind: 'clear', payload: fraction.length > 2 ? r.value.toFixed(2) : r.payload };
}

/** Le refus à afficher sous le champ : `ambiguous`, `unreadable`, ou `null` (claire ou vide). */
export function centimesEntryProblem(
	reading: CentimesEntryReading,
): 'ambiguous' | 'unreadable' | null {
	return reading.kind === 'ambiguous' || reading.kind === 'unreadable' ? reading.kind : null;
}
