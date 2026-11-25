/**
 * Test de TYPE de la charge d'une ligne de facture (Story 49-5, lot A, #1464) :
 * `npm run check` (svelte-check) le vérifie, vitest l'exécute seulement pour que
 * le fichier compte comme une suite. Chaque cas tient sur UNE ligne, la directive
 * sur la ligne qui précède : une propriété manquante rougit à la déclaration, une
 * propriété en trop à la ligne de la propriété ; sur une ligne, les deux coïncident.
 */
import { describe, it, expect } from 'vitest';
import type { CreateInvoiceLineRequest } from './invoices.types';

describe('CreateInvoiceLineRequest — la charge est typée par son type (#1464)', () => {
	it('refuse en type une ligne chiffrée incomplète ou une ligne de texte chiffrée', () => {
		// @ts-expect-error (a) une ligne `priced` sans `quantity`
		const a: CreateInvoiceLineRequest = { lineType: 'priced', description: 'x', unitPrice: '1.00', vatRate: '8.10' };
		// @ts-expect-error (b) une ligne `priced` sans `unitPrice`
		const b: CreateInvoiceLineRequest = { lineType: 'priced', description: 'x', quantity: '1', vatRate: '8.10' };
		// @ts-expect-error (c) une ligne `priced` sans `vatRate`
		const c: CreateInvoiceLineRequest = { lineType: 'priced', description: 'x', quantity: '1', unitPrice: '1.00' };
		// @ts-expect-error (d) une ligne `comment` avec `quantity`
		const d: CreateInvoiceLineRequest = { lineType: 'comment', description: 'x', quantity: '1' };
		// @ts-expect-error (e) une ligne `heading` avec `vatRate`
		const e: CreateInvoiceLineRequest = { lineType: 'heading', description: 'x', vatRate: '8.10' };
		const f: CreateInvoiceLineRequest = { lineType: 'priced', description: 'x', quantity: '1', unitPrice: '1.00', vatRate: '8.10' };
		const g: CreateInvoiceLineRequest = { lineType: 'comment', description: 'x' };
		expect([a, b, c, d, e, f, g]).toHaveLength(7);
	});
});
