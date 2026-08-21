/**
 * Helpers purs pour les lots de paiement — Story 12.3 (#191).
 */

import Big from 'big.js';
import { formatSwissAmount } from '$lib/features/journal-entries/balance';
import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
import type { PaymentBatchStatus } from './payment-batches.types';

/** Formate un montant (string décimal) au format suisse. `''` si vide/invalide. */
export function formatBatchTotal(d: string | null | undefined): string {
	if (!d) return '';
	try {
		return formatSwissAmount(new Big(d));
	} catch {
		return '';
	}
}

/**
 * Libellé traduit du statut d'un lot de paiement.
 *
 * ⚠️ **Ces trois valeurs étaient en dur, et rien ne pouvait le signaler** — c'est l'angle
 * mort #255 : sans appel à `i18nMsg`, ni le moissonneur, ni l'allowlist, ni les gardes i18n
 * ne les voyaient. Le doc-comment qui les couvrait — *« Libellé FR du statut d'un X (fallback
 * i18n côté composant) »* — était copié à l'identique dans trois domaines, dont celui où le
 * défaut a fini par se voir à l'écran (story 23-3).
 *
 * ⚠️ **Le défaut était LATENT ici, et c'est la traduction qui l'aurait activé** :
 * `payment-batches-col-status` est encore à l'allowlist, donc personne ne lit aujourd'hui
 * « Status » au-dessus de « Généré ». La story 23-4 traduira cet en-tête — d'où la 23-3b
 * AVANT elle, faute de quoi elle figeait le défaut au lieu de le corriger.
 *
 * ⚠️ **« Généré » est devenu « Créé »** (arbitrage de Guy, 2026-08-20). Le verbe *créer*
 * devient uniforme dans le domaine — `Créer une facture`, `Facture créée.`, et maintenant un
 * lot `Créé` —, et les trois cibles suivent le même verbe. La valeur est **relevée** sur
 * `imported-supplier-invoices-completed`, pas inventée.
 */
export function paymentBatchStatusLabel(status: PaymentBatchStatus): string {
	switch (status) {
		case 'generated':
			return i18nMsg('payment-batches-status-generated', 'Créé');
		case 'confirmed':
			return i18nMsg('payment-batches-status-confirmed', 'Confirmé');
		case 'cancelled':
			return i18nMsg('payment-batches-status-cancelled', 'Annulé');
		default:
			return status;
	}
}

/**
 * Libellé traduit d'un code d'échec per-facture (`FailedProposal`).
 *
 * Les codes viennent du backend et restent des **constantes canoniques** — cf. la
 * § *Pattern batch — FailedProposal per-proposal* de `CLAUDE.md`. Seul leur affichage est
 * traduit ; un code inconnu retombe sur sa valeur brute, ce qui vaut mieux qu'une case vide.
 *
 * ⚠️ **Les six libellés sont RELEVÉS, pas inventés** : `SUPPLIER_INVOICE_NOT_FOUND` sur
 * `reminders-error-invoice-not-found`, les deux IBAN sur
 * `imported-supplier-invoices-error-invalid-iban` — **sans** « du créancier », qui ne
 * s'applique pas ici —, et *ouverte* est en partie A du glossaire. `NO_PAYMENT_COORDINATES`
 * dit `Keine Zahlungsverbindung` en allemand : c'est le terme bancaire, et non le calque
 * `Zahlungskoordinaten`.
 *
 * ⚠️ **`ALREADY_IN_GENERATED_BATCH` change de français, et c'est une conséquence directe de
 * l'arbitrage sur le statut.** Il disait « Déjà dans un lot **en cours** » — une troisième
 * formulation pour un statut qui s'appelle `generated` dans le code et s'affiche « Créé » à
 * l'écran. L'utilisateur ne pouvait pas faire le lien entre les deux ; le message reprend
 * désormais le mot exact de la colonne Statut. Laisser cette ligne de côté aurait rendu le
 * correctif incohérent avec lui-même.
 *
 * ⚠️ **Les six clés sont écrites en toutes lettres, jamais construites par gabarit.** Une
 * clé statique est vue par `i18n-keys.test.ts` dès qu'elle manque d'un catalogue ; une clé
 * dynamique demande un motif déclaré, et une carte peut grandir sans qu'aucune garde ne
 * rougisse — c'est exactement ce qui a été mesuré en passe 4 de la story 23-3.
 */
export function failedItemLabel(code: string): string {
	switch (code) {
		case 'SUPPLIER_INVOICE_NOT_FOUND':
			return i18nMsg('payment-batches-failed-supplier-invoice-not-found', 'Facture introuvable');
		case 'SUPPLIER_INVOICE_NOT_OPEN':
			return i18nMsg('payment-batches-failed-supplier-invoice-not-open', 'Facture non ouverte');
		case 'NO_PAYMENT_COORDINATES':
			return i18nMsg(
				'payment-batches-failed-no-payment-coordinates',
				'Pas de coordonnées de paiement (IBAN/QR-IBAN)'
			);
		case 'ALREADY_IN_GENERATED_BATCH':
			return i18nMsg('payment-batches-failed-already-in-generated-batch', 'Déjà dans un lot créé');
		case 'INVALID_IBAN':
			return i18nMsg('payment-batches-failed-invalid-iban', 'IBAN invalide');
		case 'INVALID_QR_IBAN':
			return i18nMsg('payment-batches-failed-invalid-qr-iban', 'QR-IBAN invalide');
		default:
			return code;
	}
}
