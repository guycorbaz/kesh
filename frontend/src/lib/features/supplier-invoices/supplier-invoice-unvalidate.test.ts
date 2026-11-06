/**
 * Story v017-4b (#849, AC-25) — les messages des refus de la dévalidation d'une
 * facture fournisseur ; Story v0172-annulees (#868, AC-8, T8) — le geste s'appelle
 * « repasser en brouillon », et chaque refus nomme ce qu'il faut défaire d'abord (lot,
 * transaction et sa date, écriture de paiement et sa date), à partir de `details`.
 *
 * ⚠️ Chaque test nomme la MUTATION qu'il attrape (M29 : un code rend le repli ; M11 :
 * la date de la transaction absente du message).
 * i18nMsg rend son repli, variables substituées : les textes vérifiés sont ceux du
 * repli français, qui égale le catalogue fr-CH.
 */
import { describe, it, expect, vi } from 'vitest';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';

vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_k: string, fallback: string, args?: Record<string, string | number>) =>
		args ? fallback.replace(/\{\s*\$(\w+)\s*\}/g, (_, n) => String(args[n] ?? '')) : fallback,
}));

import {
	SUPPLIER_INVOICE_UNVALIDATE_CODES,
	supplierInvoiceUnvalidateMessage,
} from './supplier-invoice-unvalidate';

/** La valeur fr-CH d'une clé sur une seule ligne du catalogue. */
function valeurFr(cle: string): string | undefined {
	const texte = readFileSync(join('../crates/kesh-i18n/locales', 'fr-CH', 'messages.ftl'), 'utf-8');
	const prefixe = `${cle} = `;
	return texte
		.split('\n')
		.find((l) => l.startsWith(prefixe))
		?.slice(prefixe.length);
}

/** Le gabarit fr-CH d'une clé, arguments substitués. */
function rempli(cle: string, args: Record<string, string>): string {
	const gabarit = valeurFr(cle);
	expect(gabarit, cle).toBeDefined();
	return gabarit!.replace(/\{\s*\$(\w+)\s*\}/g, (_, n) => args[n] ?? '');
}

const REPLI = 'La facture n’a pas pu repasser en brouillon.';

/** Des `details` complets, tels que le serveur les rend (AC-8). */
const DETAILS = {
	batchId: 4,
	transactionDate: '2026-06-17',
	entryId: 90,
	entryNumber: 12,
	entryDate: '2026-06-18',
};

describe('supplierInvoiceUnvalidateMessage', () => {
	it('chaque code de la précédence a son propre message, jamais le repli (M29)', () => {
		const vus = new Set<string>();
		for (const code of SUPPLIER_INVOICE_UNVALIDATE_CODES) {
			const m = supplierInvoiceUnvalidateMessage(code, DETAILS);
			expect(m, code).not.toBe(REPLI);
			expect(vus.has(m), `${code} : texte en double`).toBe(false);
			vus.add(m);
		}
		// Story v0172-annulees : douze codes — cinq motifs propres à la pièce, ceux de
		// `delete_in_tx`, plus l'état et la version.
		expect([...SUPPLIER_INVOICE_UNVALIDATE_CODES].sort()).toEqual(
			[
				'SUPPLIER_INVOICE_PAID',
				'SUPPLIER_INVOICE_PAID_RECONCILED',
				'ILLEGAL_STATE_TRANSITION',
				'OPTIMISTIC_LOCK_CONFLICT',
				'SUPPLIER_INVOICE_IN_PAYMENT_BATCH',
				'SUPPLIER_INVOICE_DETACHED_PAYMENT',
				'MATCHED_BANK_TRANSACTION',
				'FISCAL_YEAR_CLOSED',
				'LATER_FISCAL_YEAR_CLOSED',
				'ENTRY_IS_REVERSED',
				'PERIOD_LOCKED',
				'ENTRY_LETTERED',
			].sort(),
		);
	});

	it('SUPPLIER_INVOICE_PAID : mot pour mot le FTL fr-CH, sans suffixe', () => {
		const attendu = valeurFr('error-supplier-invoice-unvalidate-blocked-paid');
		expect(attendu).toBeDefined();
		expect(supplierInvoiceUnvalidateMessage('SUPPLIER_INVOICE_PAID', DETAILS)).toBe(attendu);
		expect(supplierInvoiceUnvalidateMessage('SUPPLIER_INVOICE_PAID', null)).toBe(attendu);
	});

	it('SUPPLIER_INVOICE_PAID_RECONCILED : la date de la transaction, au format suisse (M11)', () => {
		const m = supplierInvoiceUnvalidateMessage('SUPPLIER_INVOICE_PAID_RECONCILED', DETAILS);
		expect(m).toBe(
			rempli('error-supplier-invoice-unvalidate-blocked-paid-reconciled', { date: '17.06.2026' }),
		);
		expect(m).toContain('17.06.2026');
	});

	it('SUPPLIER_INVOICE_IN_PAYMENT_BATCH : le lot nommé par `batchId`', () => {
		expect(supplierInvoiceUnvalidateMessage('SUPPLIER_INVOICE_IN_PAYMENT_BATCH', DETAILS)).toBe(
			rempli('error-supplier-invoice-unvalidate-blocked-in-payment-batch', { batch: '4' }),
		);
	});

	it('SUPPLIER_INVOICE_DETACHED_PAYMENT : sans date de transaction, la clé sans date', () => {
		const sansDate = { ...DETAILS, transactionDate: null };
		expect(supplierInvoiceUnvalidateMessage('SUPPLIER_INVOICE_DETACHED_PAYMENT', sansDate)).toBe(
			rempli('error-supplier-invoice-unvalidate-blocked-detached-payment', {
				entry: '12',
				entryDate: '18.06.2026',
			}),
		);
	});

	it('SUPPLIER_INVOICE_DETACHED_PAYMENT : rapproché, la clé avec la date (M11)', () => {
		const m = supplierInvoiceUnvalidateMessage('SUPPLIER_INVOICE_DETACHED_PAYMENT', DETAILS);
		expect(m).toBe(
			rempli('error-supplier-invoice-unvalidate-blocked-detached-payment-reconciled', {
				entry: '12',
				entryDate: '18.06.2026',
				date: '17.06.2026',
			}),
		);
	});

	it('MATCHED_BANK_TRANSACTION : la date de la transaction (M11)', () => {
		expect(supplierInvoiceUnvalidateMessage('MATCHED_BANK_TRANSACTION', DETAILS)).toBe(
			rempli('error-supplier-invoice-unvalidate-blocked-matched', { date: '17.06.2026' }),
		);
	});

	it.each([
		['OPTIMISTIC_LOCK_CONFLICT', 'supplier-invoices-unvalidate-blocked-stale'],
		['ILLEGAL_STATE_TRANSITION', 'supplier-invoices-unvalidate-blocked-not-validated'],
		['FISCAL_YEAR_CLOSED', 'supplier-invoices-unvalidate-blocked-fiscal-year-closed'],
		['LATER_FISCAL_YEAR_CLOSED', 'supplier-invoices-unvalidate-blocked-later-fiscal-year-closed'],
		['ENTRY_IS_REVERSED', 'supplier-invoices-unvalidate-blocked-reversed'],
		['PERIOD_LOCKED', 'supplier-invoices-unvalidate-blocked-period-locked'],
		['ENTRY_LETTERED', 'supplier-invoices-unvalidate-blocked-lettered'],
	])('%s : son texte, mot pour mot le FTL fr-CH, sans suffixe', (code, cle) => {
		const attendu = valeurFr(cle);
		expect(attendu).toBeDefined();
		expect(supplierInvoiceUnvalidateMessage(code, null)).toBe(attendu);
		expect(supplierInvoiceUnvalidateMessage(code, DETAILS)).toBe(attendu);
	});

	// Revue P1 (E-4) — le nom de l'exercice postérieur clos, que le serveur donne dans
	// `details.fiscalYearName`, est repris.
	it('LATER_FISCAL_YEAR_CLOSED nommé : le texte du catalogue, le nom substitué', () => {
		expect(
			supplierInvoiceUnvalidateMessage('LATER_FISCAL_YEAR_CLOSED', { fiscalYearName: 'Exercice 2027' }),
		).toBe(
			rempli('supplier-invoices-unvalidate-blocked-later-fiscal-year-closed-named', {
				name: 'Exercice 2027',
			}),
		);
	});

	// Story v0174-correctifs-3 (#869, AC-3.12) — le serveur donne la borne du verrou
	// (`details.lockedThrough`) ; l'écran la nomme, en forme suisse. Sans elle, la phrase
	// d'aujourd'hui.
	it('PERIOD_LOCKED nommé : la date du verrou, au format suisse (MC-869)', () => {
		const m = supplierInvoiceUnvalidateMessage('PERIOD_LOCKED', { lockedThrough: '2026-03-31' });
		expect(m).toContain('31.03.2026');
		expect(m).toBe(
			rempli('supplier-invoices-unvalidate-blocked-period-locked-named', { date: '31.03.2026' }),
		);
		expect(supplierInvoiceUnvalidateMessage('PERIOD_LOCKED', null)).toBe(
			valeurFr('supplier-invoices-unvalidate-blocked-period-locked'),
		);
	});

	it('un code inconnu rend le repli générique, égal au catalogue', () => {
		expect(supplierInvoiceUnvalidateMessage('INTERNAL_ERROR', null)).toBe(REPLI);
		expect(valeurFr('supplier-invoices-unvalidate-error')).toBe(REPLI);
	});

	it('le vocabulaire du geste : « repasser en brouillon », jamais « dévalider » (C-v0172a-20)', () => {
		for (const code of [...SUPPLIER_INVOICE_UNVALIDATE_CODES, 'INTERNAL_ERROR']) {
			expect(supplierInvoiceUnvalidateMessage(code, DETAILS), code).not.toMatch(/d[ée]valid/i);
		}
	});

	it('aucun message ne nomme l’état validé « ouverte » (règle Q4)', () => {
		for (const code of SUPPLIER_INVOICE_UNVALIDATE_CODES) {
			expect(supplierInvoiceUnvalidateMessage(code, DETAILS)).not.toMatch(/ouvert/i);
		}
	});
});
