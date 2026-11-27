/**
 * Les lignes d'une facture client à l'écran (Story 49-1c, #1242) : le type
 * d'une ligne, et **les seules** conversions entre l'API et l'état du
 * formulaire de facture.
 *
 * Pourquoi un module : avant la 49-1c, deux sites d'`InvoiceForm.svelte`
 * recopiaient les champs d'une ligne relue à la main (ouverture du brouillon,
 * rechargement après un conflit de version). Un champ oublié dans l'un d'eux
 * ne se voyait qu'après un conflit (avertissement de la Story 16-1a) ; un
 * `lineType` oublié ferait redevenir chiffrée une ligne de commentaire, que le
 * serveur refuserait pour sa quantité nulle. Les deux relectures passent donc
 * par `ligneDepuisReponse`, et l'envoi par `ligneAEnvoyer`.
 */

import type {
	CreateInvoiceLineRequest,
	InvoiceLineResponse,
	InvoiceLineType,
} from './invoices.types';
import { readableAmount } from '$lib/features/supplier-invoices/supplier-invoice-form';
import { unitPriceFieldValue } from './unit-price';
import { estChiffree, estLigneDeTexte } from './invoice-line-type';

export { estChiffree, estLigneDeTexte };

/**
 * L'état d'une ligne dans le formulaire de facture (D5). Les champs de montant
 * restent des `string` (ceux des champs de saisie) ; une ligne de texte y porte
 * `''`, jamais envoyé (`ligneAEnvoyer`).
 */
export type LigneFormulaire = {
	lineType: InvoiceLineType;
	description: string;
	quantity: string;
	unitPrice: string;
	vatRate: string;
	/** `null` = suivre le compte de produit par défaut de la société. */
	revenueAccountId: number | null;
	/** Identifiant d'interface stable ; jamais envoyé au serveur. */
	_uiKey: string;
	/** Taux provisoire posé avant la résolution des taux actifs (v0172, #882). */
	_vatPlaceholder?: boolean;
};

/**
 * Une ligne relue du serveur, dans l'état du formulaire. Le type est **gardé**
 * (une ligne `heading` relue reste `heading`, D5) ; un type absent devient
 * `priced`. Une ligne de texte n'a pas de montant à éditer : ses champs de
 * montant sont vides.
 */
export function ligneDepuisReponse(l: InvoiceLineResponse, uiKey: string): LigneFormulaire {
	const lineType = l.lineType ?? 'priced';
	if (estLigneDeTexte(lineType)) {
		return {
			lineType,
			description: l.description,
			quantity: '',
			unitPrice: '',
			vatRate: '',
			revenueAccountId: null,
			_uiKey: uiKey,
		};
	}
	return {
		lineType,
		description: l.description,
		quantity: readableAmount(l.quantity, 0),
		unitPrice: unitPriceFieldValue(l.unitPrice),
		vatRate: l.vatRate,
		// AC9-bis de la Story 16-1a : recopier le compte de la réponse, sinon le
		// réenregistrement effacerait la ventilation de la ligne.
		revenueAccountId: l.revenueAccountId,
		_uiKey: uiKey,
	};
}

/**
 * Échange la ligne `index` avec sa voisine (Story 49-2, D4) : `sens = -1` la
 * monte, `1` la descend. Rend un **nouveau** tableau des **mêmes objets**
 * (chaque ligne garde son `_uiKey`, donc son élément du DOM) ; celui reçu n'est
 * pas modifié. Un `index` hors bornes ou un déplacement au-delà d'une extrémité
 * rend une copie inchangée : jamais d'exception, jamais de ligne perdue ni
 * dupliquée.
 */
export function deplacerLigne<T>(lignes: readonly T[], index: number, sens: -1 | 1): T[] {
	const copie = [...lignes];
	const cible = index + sens;
	if (
		!Number.isInteger(index) ||
		index < 0 ||
		index >= copie.length ||
		cible < 0 ||
		cible >= copie.length
	) {
		return copie;
	}
	[copie[index], copie[cible]] = [copie[cible], copie[index]];
	return copie;
}

/**
 * La charge d'une ligne du formulaire (D3, D5). Une ligne chiffrée : ses champs
 * d'avant la 0.20.0 plus `lineType: 'priced'` ; le prix est celui que
 * l'appelant a lu et normalisé. Une ligne de texte : **`{ lineType,
 * description }` seulement** — aucun montant ne part, le serveur refuserait
 * une quantité non nulle.
 */
export function ligneAEnvoyer(l: LigneFormulaire): CreateInvoiceLineRequest {
	if (estLigneDeTexte(l.lineType)) {
		return { lineType: l.lineType, description: l.description };
	}
	return {
		lineType: 'priced',
		description: l.description,
		quantity: l.quantity,
		unitPrice: l.unitPrice,
		vatRate: l.vatRate,
		revenueAccountId: l.revenueAccountId,
	};
}
