/**
 * Helpers purs pour les avoirs (notes de crédit) — Story 12.1.
 */

import Big from 'big.js';
import { formatSwissAmount } from '$lib/features/journal-entries/balance';
import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
import type { CreditNoteStatus } from './credit-notes.types';

/**
 * Formate un montant d'avoir (string décimal 4 décimales) au format suisse :
 * `"1000.0000"` → `"1’000.00"`. Retourne `''` si vide/invalide.
 */
export function formatCreditNoteTotal(d: string | null | undefined): string {
	if (!d) return '';
	try {
		return formatSwissAmount(new Big(d));
	} catch {
		return '';
	}
}

/**
 * Libellé traduit du statut d'un avoir.
 *
 * ⚠️ **Même défaut, même doc-comment, troisième domaine** — cf. `paymentBatchStatusLabel`.
 * Le patron *« Libellé FR du statut d'un X (fallback i18n côté composant) »* était copié mot
 * pour mot ; c'est ce qui a fait de la 23-3b une story et non un correctif ponctuel. Ici
 * aussi le défaut est **latent** : `credit-notes-col-status` est encore à l'allowlist, et
 * c'est la story 23-5 qui l'activerait.
 *
 * ⚠️ **`Émis` ne se confond pas avec `validé`**, qui désigne l'acte d'immuabilité d'une
 * facture. La valeur est **relevée** sur `credit-note-revenue-account-archived` —
 * `ausgestellt` / `emessa` / `issued`. ⚠️ **L'accord suit la langue, pas le français** : un
 * *avoir* est masculin, mais `Gutschrift` et `nota di credito` sont féminins, d'où `Emessa`
 * et `Annullata` en italien.
 */
export function creditNoteStatusLabel(status: CreditNoteStatus): string {
	switch (status) {
		case 'draft':
			return i18nMsg('credit-notes-status-draft', 'Brouillon');
		case 'issued':
			return i18nMsg('credit-notes-status-issued', 'Émis');
		case 'cancelled':
			return i18nMsg('credit-notes-status-cancelled', 'Annulé');
		default:
			return status;
	}
}
