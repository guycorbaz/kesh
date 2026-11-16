/**
 * Story 36-18a (#1050, #1159, #1172, #1174 ; D3, C-36-18-3) — les deux messages communs
 * d'un montant saisi hors du journal : soldes de départ, prix et montant attendu d'une
 * facture fournisseur, règlement d'une facture.
 *
 * Une seule définition des textes (DRY), à côté du lecteur (`balance.ts`), qui n'importe
 * pas l'i18n — ses tests n'ont ainsi pas à la simuler. Les clés `common-*` sont globales
 * (`frontend/scripts/lint-i18n-ownership.js`).
 */

import { i18nMsg } from '$lib/shared/utils/i18n.svelte';

/**
 * Le message d'un montant `ambiguous` (« 1.250 ») ou `unreadable` (« 12'34 »). `max`
 * est le nombre de décimales du champ : 4 pour les soldes de départ et la facture
 * fournisseur, 2 pour le règlement (au centime).
 */
export function amountEntryMessage(problem: 'ambiguous' | 'unreadable', max: 2 | 4): string {
	if (problem === 'ambiguous') {
		return i18nMsg(
			'common-amount-ambiguous',
			"Montant ambigu : un point ou une virgule suivis de trois chiffres peuvent séparer les milliers. Écrivez 1'250 pour mille deux cent cinquante, ou 1.25 pour un franc vingt-cinq.",
		);
	}
	return i18nMsg(
		'common-amount-unreadable',
		"Montant illisible : chiffres, apostrophe des milliers (1'250), point ou virgule décimale, { $max } décimales au plus.",
		{ max },
	);
}
