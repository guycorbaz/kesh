/**
 * Story 36-3b (#1171) — le prix unitaire d'une ligne de facture client.
 *
 * Le prix se lit comme le montant d'une écriture (`readJournalAmount`, Story 36-3,
 * C-e360-17), sans grammaire propre : point ou virgule décimale, quatre décimales au
 * plus, apostrophe des milliers (« 1'250 »), aucun signe. « 1.250 » — un point ou une
 * virgule suivis de trois chiffres, sans groupement avant — est **ambigu** et refusé,
 * au lieu d'être lu 1,25 franc en silence.
 *
 * Module à part, et non `invoice-helpers.ts` (validation P1, V1-5) : `readableAmount`
 * vit dans `supplier-invoice-form.ts`, qui importe lui-même `invoice-helpers.ts` ; l'y
 * importer créerait un cycle.
 */

import Big from 'big.js';
import { readJournalAmount } from '$lib/features/journal-entries/balance';
import { readableAmount } from '$lib/features/supplier-invoices/supplier-invoice-form';

/**
 * La charge à envoyer pour un prix saisi : le `payload` d'une lecture claire
 * (« 1'250 » → `1250`, « 1,25 » → `1.25`, « 100.00 » → `100.00`) ; `null` pour une
 * saisie ambiguë, illisible **ou vide** (C-36-3b-2) — un prix effacé ne part jamais à
 * zéro, contrairement à `journalAmountPayload`, qui rend `'0'` pour le vide.
 */
export function unitPricePayload(raw: string): string | null {
	const r = readJournalAmount(raw);
	return r.kind === 'clear' ? r.payload : null;
}

/**
 * Le refus à afficher sous le champ du prix : `ambiguous`, `unreadable`, ou `null`
 * (prix clair ou vide — le vide n'a pas de message sous le champ, comme avant la story).
 */
export function unitPriceProblem(raw: string): 'ambiguous' | 'unreadable' | null {
	const kind = readJournalAmount(raw).kind;
	return kind === 'ambiguous' || kind === 'unreadable' ? kind : null;
}

/**
 * La forme d'un prix venu du serveur (`DECIMAL(19,4)`) dans son champ (D2,
 * C-36-3b-3) : la forme lisible de #878 (`readableAmount(raw, 2)` : « 1.2500 » →
 * `1.25`, « 100.0000 » → `100.00`), **sauf** si elle serait ambiguë (« 1.2340 » →
 * `1.234`) : alors les quatre décimales du serveur (`1.2340`), forme jamais ambiguë.
 * Une valeur illisible pour big.js est rendue telle quelle (contrat de `readableAmount`).
 */
export function unitPriceFieldValue(raw: string): string {
	const lisible = readableAmount(raw, 2);
	if (readJournalAmount(lisible).kind !== 'ambiguous') return lisible;
	return new Big(raw).toFixed(4);
}
