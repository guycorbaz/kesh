/**
 * Story v017-4b (#849, AC-25) — les messages des refus de la dévalidation d'une
 * facture fournisseur.
 *
 * ⚠️ Chaque test nomme la MUTATION qu'il attrape (M29 : un code rend le repli).
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

const REPLI = 'La facture n’a pas pu être dévalidée.';

describe('supplierInvoiceUnvalidateMessage', () => {
	it('chaque code de la précédence a son propre message, jamais le repli (M29)', () => {
		const vus = new Set<string>();
		for (const code of SUPPLIER_INVOICE_UNVALIDATE_CODES) {
			const m = supplierInvoiceUnvalidateMessage(code, null);
			expect(m, code).not.toBe(REPLI);
			expect(vus.has(m), `${code} : texte en double`).toBe(false);
			vus.add(m);
		}
		// La précédence de la fiche parente : trois motifs propres à la pièce, ceux de
		// `delete_in_tx`, plus l'état et la version.
		expect([...SUPPLIER_INVOICE_UNVALIDATE_CODES].sort()).toEqual(
			[
				'SUPPLIER_INVOICE_PAID',
				'ILLEGAL_STATE_TRANSITION',
				'OPTIMISTIC_LOCK_CONFLICT',
				'SUPPLIER_INVOICE_IN_PAYMENT_BATCH',
				'MATCHED_BANK_TRANSACTION',
				'FISCAL_YEAR_CLOSED',
				'LATER_FISCAL_YEAR_CLOSED',
				'ENTRY_IS_REVERSED',
				'PERIOD_LOCKED',
				'ENTRY_LETTERED',
			].sort(),
		);
	});

	it.each([
		['SUPPLIER_INVOICE_PAID', 'error-supplier-invoice-unvalidate-blocked-paid'],
		['SUPPLIER_INVOICE_IN_PAYMENT_BATCH', 'error-supplier-invoice-unvalidate-blocked-in-payment-batch'],
		['MATCHED_BANK_TRANSACTION', 'error-supplier-invoice-unvalidate-blocked-matched'],
	])(
		'%s : le texte du serveur, mot pour mot le FTL fr-CH, suffixé du numéro de la pièce',
		(code, cle) => {
			const attendu = valeurFr(cle);
			expect(attendu).toBeDefined();
			expect(supplierInvoiceUnvalidateMessage(code, null)).toBe(attendu);
			expect(supplierInvoiceUnvalidateMessage(code, 'Lot 4')).toBe(`${attendu} (Lot 4)`);
		},
	);

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
		// Le libellé n'est lu que par les motifs propres à la pièce, et par l'exercice
		// postérieur nommé (ci-dessous).
		if (code !== 'LATER_FISCAL_YEAR_CLOSED') {
			expect(supplierInvoiceUnvalidateMessage(code, 'ignoré')).toBe(attendu);
		}
	});

	// Revue P1 (E-4) — le nom de l'exercice postérieur clos, que le serveur donne dans
	// `details.fiscalYearName`, est repris.
	it('LATER_FISCAL_YEAR_CLOSED nommé : le texte du catalogue, le nom substitué', () => {
		const gabarit = valeurFr('supplier-invoices-unvalidate-blocked-later-fiscal-year-closed-named');
		expect(gabarit).toBeDefined();
		expect(supplierInvoiceUnvalidateMessage('LATER_FISCAL_YEAR_CLOSED', 'Exercice 2027')).toBe(
			gabarit!.replace('{ $name }', 'Exercice 2027'),
		);
	});

	it('un code inconnu rend le repli générique, égal au catalogue', () => {
		expect(supplierInvoiceUnvalidateMessage('INTERNAL_ERROR', null)).toBe(REPLI);
		expect(valeurFr('supplier-invoices-unvalidate-error')).toBe(REPLI);
	});

	it('aucun message ne nomme l’état validé « ouverte » (règle Q4)', () => {
		for (const code of SUPPLIER_INVOICE_UNVALIDATE_CODES) {
			expect(supplierInvoiceUnvalidateMessage(code, null)).not.toMatch(/ouvert/i);
		}
	});
});
