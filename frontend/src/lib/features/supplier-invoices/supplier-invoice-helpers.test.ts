import { describe, expect, it } from 'vitest';
import {
	formatSupplierInvoiceTotal,
	supplierInvoiceStatusLabel,
} from './supplier-invoice-helpers';

describe('formatSupplierInvoiceTotal', () => {
	it('formate au format suisse', () => {
		expect(formatSupplierInvoiceTotal('1081.0000')).toBe("1’081.00");
	});
	it("retourne '' pour vide/invalide", () => {
		expect(formatSupplierInvoiceTotal(null)).toBe('');
		expect(formatSupplierInvoiceTotal('')).toBe('');
		expect(formatSupplierInvoiceTotal('abc')).toBe('');
	});
});

describe('supplierInvoiceStatusLabel', () => {
	it('mappe les statuts', () => {
		expect(supplierInvoiceStatusLabel('open')).toBe('Ouverte');
		expect(supplierInvoiceStatusLabel('paid')).toBe('Payée');
		expect(supplierInvoiceStatusLabel('cancelled')).toBe('Annulée');
	});
});
