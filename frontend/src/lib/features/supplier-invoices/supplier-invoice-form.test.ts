// Story v017-3b (#687, AC-12) — le miroir de `purchase_line_total` (kesh-core) et le
// refus serveur `SUPPLIER_INVOICE_LINE_NOT_IN_CENTIMES`.
//
// ⚠️ Chaque test nomme la mutation qu'il attrape. i18nMsg rend son repli, variables
// substituées : les textes vérifiés sont ceux du repli français (= catalogue fr-CH).

import { describe, it, expect, vi } from 'vitest';

vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string, args?: Record<string, string | number>) =>
		args
			? fallback.replace(/\{\s*\$(\w+)\s*\}/g, (_, n) => String(args[n] ?? ''))
			: fallback,
}));

import { describeSubmitError, emptyLine, firstLineNotInCentimes, lineNotInCentimes } from './supplier-invoice-form';
import type { CreateSupplierInvoiceLineRequest } from './supplier-invoices.types';

function line(quantity: string, unitPrice: string): CreateSupplierInvoiceLineRequest {
	return { ...emptyLine(), description: 'x', quantity, unitPrice, expenseAccountId: 40 };
}

describe('lineNotInCentimes — les cas de l’AC-1', () => {
	it.each([
		['1', '12.50'],
		['2', '0.125'],
		['4', '2.5000'],
		['0.1', '0.1'],
		['0.3', '0.1'],
	])('%s × %s est au centime', (q, p) => {
		expect(lineNotInCentimes(line(q, p))).toBeNull();
	});

	it.each([
		['3', '33.3333', '99.9999'],
		['1.5', '3.333', '4.9995'],
		['1', '0.005', '0.005'],
	])('%s × %s n’est pas au centime (montant %s)', (q, p, total) => {
		expect(lineNotInCentimes(line(q, p))).toBe(total);
	});

	it('accepte la virgule décimale, comme AMOUNT_RE', () => {
		expect(lineNotInCentimes(line('3', '33,3333'))).toBe('99.9999');
		expect(lineNotInCentimes(line('1', '12,50'))).toBeNull();
	});

	it('lit une quantité vide comme 1, comme l’envoi', () => {
		expect(lineNotInCentimes(line('', '0.005'))).toBe('0.005');
		expect(lineNotInCentimes(line('', '12.50'))).toBeNull();
	});

	it('ne refuse rien sur une saisie vide ou illisible', () => {
		expect(lineNotInCentimes(line('1', ''))).toBeNull();
		expect(lineNotInCentimes(line('abc', '1.005'))).toBeNull();
	});

	it('affiche le montant sans notation exponentielle', () => {
		expect(lineNotInCentimes(line('0.0001', '0.0001'))).toBe('0.00000001');
	});

	it('firstLineNotInCentimes rend la première ligne fautive (0-based) et son montant', () => {
		expect(firstLineNotInCentimes([line('1', '10'), line('3', '33.3333')])).toEqual({
			index: 1,
			lineTotal: '99.9999',
		});
		expect(firstLineNotInCentimes([line('1', '10')])).toBeNull();
	});
});

describe('describeSubmitError — SUPPLIER_INVOICE_LINE_NOT_IN_CENTIMES', () => {
	it('désigne la ligne et dit le texte du refus serveur (même clé qu’errors.rs)', () => {
		const e = describeSubmitError({
			code: 'SUPPLIER_INVOICE_LINE_NOT_IN_CENTIMES',
			status: 400,
			message: 'serveur',
			details: { line: 2, lineTotal: '4.9995' },
		});
		expect(e.field).toBe('line');
		expect(e.line).toBe(2);
		expect(e.message).toBe(
			'La ligne 2 donne un montant de 4.9995, qui n’est pas au centime. Saisissez la ligne au montant facturé par le fournisseur, au centime.',
		);
	});
});
