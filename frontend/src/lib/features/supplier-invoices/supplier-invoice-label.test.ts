import { describe, expect, it } from 'vitest';
import {
	supplierInvoiceLongLabel,
	supplierInvoicePreferredNumber,
	supplierInvoiceShortLabel,
} from './supplier-invoice-label';

const cas: {
	nom: string;
	interne: string | null | undefined;
	fournisseur: string | null | undefined;
	prefere: string | null;
	courte: string;
	longue: string;
}[] = [
	{
		nom: 'les deux numéros',
		interne: 'FF-2026-001',
		fournisseur: 'ABC-123',
		prefere: 'FF-2026-001',
		courte: 'FF-2026-001',
		longue: 'FF-2026-001 (ABC-123)',
	},
	{
		nom: "l'interne seul",
		interne: 'FF-2026-001',
		fournisseur: null,
		prefere: 'FF-2026-001',
		courte: 'FF-2026-001',
		longue: 'FF-2026-001',
	},
	{
		nom: 'le fournisseur seul',
		interne: null,
		fournisseur: 'ABC-123',
		prefere: 'ABC-123',
		courte: 'ABC-123',
		longue: 'ABC-123',
	},
	{
		nom: 'ni l’un ni l’autre',
		interne: null,
		fournisseur: undefined,
		prefere: null,
		courte: '#7',
		longue: '#7',
	},
	{
		nom: 'un numéro du fournisseur blanc vaut absent',
		interne: 'FF-2026-001',
		fournisseur: '   ',
		prefere: 'FF-2026-001',
		courte: 'FF-2026-001',
		longue: 'FF-2026-001',
	},
	{
		nom: 'un numéro interne blanc vaut absent',
		interne: ' ',
		fournisseur: 'ABC-123',
		prefere: 'ABC-123',
		courte: 'ABC-123',
		longue: 'ABC-123',
	},
	{ nom: 'les deux blancs', interne: '', fournisseur: ' ', prefere: null, courte: '#7', longue: '#7' },
	{
		nom: 'les blancs de bord tombent, comme au contrôle K8',
		interne: ' FF-2026-001 ',
		fournisseur: ' ABC ',
		prefere: 'FF-2026-001',
		courte: 'FF-2026-001',
		longue: 'FF-2026-001 (ABC)',
	},
];

describe('le libellé d’une facture fournisseur', () => {
	for (const c of cas) {
		it(c.nom, () => {
			const f = { id: 7, internalNumber: c.interne, supplierInvoiceNumber: c.fournisseur };
			expect(supplierInvoicePreferredNumber(c.interne, c.fournisseur)).toBe(c.prefere);
			expect(supplierInvoiceShortLabel(f)).toBe(c.courte);
			expect(supplierInvoiceLongLabel(f)).toBe(c.longue);
		});
	}
});
