/**
 * Les deux prédicats du type d'une ligne de facture (Story 49-1c, D6, D7),
 * seule source de la règle pour le total, le formulaire et les deux fiches.
 *
 * Module à part, sans dépendance, et réexporté par `invoice-lines.ts` : celui-ci
 * importe `supplier-invoice-form.ts` (pour `readableAmount`), qui importe
 * lui-même `invoice-helpers.ts` ; `invoice-helpers.ts` (le total) ne peut donc
 * pas importer `invoice-lines.ts` sans créer un cycle (même motif que
 * `unit-price.ts`, Story 36-3b, V1-5).
 */

import type { InvoiceLineType } from './invoices.types';

/**
 * Une ligne est-elle chiffrée ? **Absent vaut `priced`** (D6) : c'est le cas
 * des données d'avant la 0.20.0 et des fixtures qui ne portent pas le champ.
 */
export function estChiffree(l: { lineType?: InvoiceLineType | null }): boolean {
	return l.lineType == null || l.lineType === 'priced';
}

/**
 * Une ligne de ce type s'affiche-t-elle comme du texte sur toute la largeur
 * (D7) ? `comment`, et `heading` — rendu comme un commentaire jusqu'à la
 * Story 49-3, même règle que le PDF (49-1b, D1). Un type absent est chiffré.
 *
 * Définie comme **le contraire exact** de `estChiffree` : l'écran, le total,
 * les contrôles et la charge choisissent donc toujours le même côté, même
 * pour une valeur hors de l'union (un type qu'un serveur plus récent ajouterait).
 */
export function estLigneDeTexte(t: InvoiceLineType | null | undefined): boolean {
	return !estChiffree({ lineType: t });
}
