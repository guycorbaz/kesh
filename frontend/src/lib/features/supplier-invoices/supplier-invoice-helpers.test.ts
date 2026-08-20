import { afterEach, describe, expect, it, vi } from 'vitest';

const getMock = vi.fn();
vi.mock('$lib/shared/utils/api-client', () => ({ apiClient: { get: (u: string) => getMock(u) } }));

import { loadI18nMessages } from '$lib/shared/utils/i18n.svelte';
import {
	formatSupplierInvoiceTotal,
	supplierInvoiceStatusLabel,
} from './supplier-invoice-helpers';

// Le dictionnaire est un état de module partagé : le vider après chaque cas, sinon le
// test qui charge de l'allemand fait échouer les assertions de repli des suivants.
afterEach(async () => {
	getMock.mockResolvedValue({ locale: 'fr-CH', messages: {} });
	await loadI18nMessages();
});

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
	// ⚠️ Ces trois assertions portent sur le REPLI, pas sur le français en dur : hors
	// navigateur le dictionnaire est vide et `i18nMsg` rend son 2e argument. Elles
	// verrouillaient le français avant la passe 4 de la story 23-3, où les valeurs
	// n'appelaient pas encore `i18nMsg` — un test vert sur un défaut visible à l'écran.
	it('mappe les statuts sur leur repli français', () => {
		expect(supplierInvoiceStatusLabel('open')).toBe('Ouverte');
		expect(supplierInvoiceStatusLabel('paid')).toBe('Payée');
		expect(supplierInvoiceStatusLabel('cancelled')).toBe('Annulée');
	});

	// Borne anti-régression : le défaut d'origine était l'ABSENCE d'appel i18n. Une
	// assertion de valeur ne l'aurait jamais vu — celle-ci le voit, en empruntant le
	// chemin réel (dictionnaire servi par l'API, cf. `i18n.svelte.test.ts`).
	it('passe réellement par le dictionnaire i18n', async () => {
		getMock.mockResolvedValue({
			locale: 'de-CH',
			messages: {
				'supplier-invoices-status-open': 'Offen',
				'supplier-invoices-status-paid': 'Bezahlt',
				'supplier-invoices-status-cancelled': 'Storniert',
			},
		});
		await loadI18nMessages();
		expect(supplierInvoiceStatusLabel('open')).toBe('Offen');
		expect(supplierInvoiceStatusLabel('paid')).toBe('Bezahlt');
		expect(supplierInvoiceStatusLabel('cancelled')).toBe('Storniert');
	});
});
