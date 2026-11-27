/**
 * Story 49-1c (T7, D5, D6, D7) — le type d'une ligne de facture à l'écran, et les
 * seules conversions entre l'API et l'état du formulaire.
 */
import { describe, expect, it } from 'vitest';
import {
	deplacerLigne,
	estChiffree,
	estLigneDeTexte,
	ligneAEnvoyer,
	ligneDepuisReponse,
	type LigneFormulaire,
} from './invoice-lines';
import type { InvoiceLineResponse } from './invoices.types';

function reponse(over: Partial<InvoiceLineResponse> = {}): InvoiceLineResponse {
	return {
		id: 1,
		invoiceId: 7,
		position: 0,
		description: 'Loyer',
		quantity: '2.0000',
		unitPrice: '1200.0000',
		vatRate: '8.10',
		lineTotal: '2400.0000',
		revenueAccountId: 42,
		createdAt: '2026-11-19T10:00:00',
		...over,
	};
}

const neutre = {
	quantity: '0.0000',
	unitPrice: '0.0000',
	vatRate: '0.00',
	lineTotal: '0.0000',
	revenueAccountId: null,
};

describe('estChiffree (D6)', () => {
	it('vrai pour un type absent et pour priced, faux pour comment et heading', () => {
		expect(estChiffree({})).toBe(true);
		expect(estChiffree({ lineType: undefined })).toBe(true);
		expect(estChiffree({ lineType: 'priced' })).toBe(true);
		expect(estChiffree({ lineType: 'comment' })).toBe(false);
		expect(estChiffree({ lineType: 'heading' })).toBe(false);
	});
});

describe('estLigneDeTexte (D7)', () => {
	it('comment et heading sont du texte ; priced et un type absent, non', () => {
		expect(estLigneDeTexte('comment')).toBe(true);
		expect(estLigneDeTexte('heading')).toBe(true);
		expect(estLigneDeTexte('priced')).toBe(false);
		expect(estLigneDeTexte(undefined)).toBe(false);
	});

	it('null vaut absent (revue P1, E-2) : ligne chiffrée, non du texte', () => {
		expect(estChiffree({ lineType: null })).toBe(true);
		expect(estLigneDeTexte(null)).toBe(false);
	});

	it('table de littéraux : chaque valeur a son attendu écrit à la main (revue P1 B-2, P2 B-1)', () => {
		// [valeur, estLigneDeTexte attendu, estChiffree attendu]
		const table: [unknown, boolean, boolean][] = [
			['priced', false, true],
			['comment', true, false],
			['heading', true, false],
			[undefined, false, true],
			[null, false, true],
			// Un type qu'un serveur plus récent ajouterait : du texte, jamais une ligne chiffrée.
			['quote', true, false],
		];
		for (const [valeur, texte, chiffree] of table) {
			expect(estLigneDeTexte(valeur as never), `estLigneDeTexte(${String(valeur)})`).toBe(texte);
			expect(estChiffree({ lineType: valeur as never }), `estChiffree(${String(valeur)})`).toBe(chiffree);
		}
	});
});

describe('ligneDepuisReponse (D5)', () => {
	it('une ligne chiffrée garde son type, ses montants lisibles et son compte', () => {
		expect(ligneDepuisReponse(reponse({ lineType: 'priced' }), 'k1')).toEqual({
			lineType: 'priced',
			description: 'Loyer',
			quantity: '2',
			unitPrice: '1200.00',
			vatRate: '8.10',
			revenueAccountId: 42,
			_uiKey: 'k1',
		});
	});

	it('une ligne sans lineType (données d’avant la 0.20.0) devient priced', () => {
		const l = ligneDepuisReponse(reponse(), 'k2');
		expect(l.lineType).toBe('priced');
		expect(l.quantity).toBe('2');
		expect(l.revenueAccountId).toBe(42);
	});

	it('un commentaire garde son type et son texte, sans montant ni compte', () => {
		expect(
			ligneDepuisReponse(
				reponse({ lineType: 'comment', description: 'Merci\nà bientôt', ...neutre }),
				'k3',
			),
		).toEqual({
			lineType: 'comment',
			description: 'Merci\nà bientôt',
			quantity: '',
			unitPrice: '',
			vatRate: '',
			revenueAccountId: null,
			_uiKey: 'k3',
		});
	});

	it('une ligne heading relue reste heading (jamais convertie)', () => {
		const l = ligneDepuisReponse(reponse({ lineType: 'heading', description: 'Travaux', ...neutre }), 'k4');
		expect(l.lineType).toBe('heading');
		expect(l.description).toBe('Travaux');
	});
});

describe('ligneAEnvoyer (D3, D5)', () => {
	const base: LigneFormulaire = {
		lineType: 'priced',
		description: 'Loyer',
		quantity: '1',
		unitPrice: '1200.00',
		vatRate: '8.10',
		revenueAccountId: null,
		_uiKey: 'k',
		_vatPlaceholder: false,
	};

	it('une ligne chiffrée : les champs d’avant la 0.20.0 plus lineType priced, sans clé d’interface', () => {
		expect(ligneAEnvoyer(base)).toEqual({
			lineType: 'priced',
			description: 'Loyer',
			quantity: '1',
			unitPrice: '1200.00',
			vatRate: '8.10',
			revenueAccountId: null,
		});
	});

	it('un commentaire : { lineType, description } exactement, même avec des champs de montant remplis', () => {
		expect(
			ligneAEnvoyer({
				...base,
				lineType: 'comment',
				description: 'Merci',
				quantity: '3',
				unitPrice: '9.00',
				vatRate: '8.10',
				revenueAccountId: 7,
			}),
		).toEqual({ lineType: 'comment', description: 'Merci' });
	});

	it('une ligne heading part heading', () => {
		expect(ligneAEnvoyer({ ...base, lineType: 'heading', description: 'Travaux' })).toEqual({
			lineType: 'heading',
			description: 'Travaux',
		});
	});
});

describe('deplacerLigne (Story 49-2, D4)', () => {
	const a = { id: 'a' };
	const b = { id: 'b' };
	const c = { id: 'c' };

	it('descend une ligne du milieu', () => {
		const entree = [a, b, c];
		const sortie = deplacerLigne(entree, 1, 1);
		expect(sortie.map((x) => x.id)).toEqual(['a', 'c', 'b']);
	});

	it('monte une ligne du milieu', () => {
		expect(deplacerLigne([a, b, c], 1, -1).map((x) => x.id)).toEqual(['b', 'a', 'c']);
	});

	it('monte la dernière et descend la première', () => {
		expect(deplacerLigne([a, b, c], 2, -1).map((x) => x.id)).toEqual(['a', 'c', 'b']);
		expect(deplacerLigne([a, b, c], 0, 1).map((x) => x.id)).toEqual(['b', 'a', 'c']);
	});

	it('rend une copie inchangée aux deux bornes', () => {
		const entree = [a, b, c];
		for (const [i, sens] of [
			[0, -1],
			[2, 1],
		] as const) {
			const sortie = deplacerLigne(entree, i, sens);
			expect(sortie).not.toBe(entree);
			expect(sortie.map((x) => x.id)).toEqual(['a', 'b', 'c']);
		}
	});

	it('rend une copie inchangée pour un index hors bornes', () => {
		for (const i of [-1, 3, 99, 1.5, Number.NaN]) {
			const sortie = deplacerLigne([a, b, c], i, 1);
			expect(sortie.map((x) => x.id)).toEqual(['a', 'b', 'c']);
			expect(deplacerLigne([a, b, c], i, -1).map((x) => x.id)).toEqual(['a', 'b', 'c']);
		}
	});

	it("traite un tableau d'une ligne et un tableau vide", () => {
		expect(deplacerLigne([a], 0, 1)).toEqual([a]);
		expect(deplacerLigne([a], 0, -1)).toEqual([a]);
		expect(deplacerLigne([], 0, 1)).toEqual([]);
	});

	it('rend un tableau neuf des mêmes objets, sans modifier celui reçu', () => {
		const entree = [a, b, c];
		const sortie = deplacerLigne(entree, 0, 1);
		expect(sortie).not.toBe(entree);
		expect(entree).toEqual([a, b, c]);
		expect(entree[0]).toBe(a);
		expect(entree[1]).toBe(b);
		expect(sortie[0]).toBe(b);
		expect(sortie[1]).toBe(a);
		expect(sortie[2]).toBe(c);
	});
});
