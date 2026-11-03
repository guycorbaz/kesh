// Story v017-3b (#687, AC-10) — le TTC d'une facture fournisseur sur sa fiche, et
// l'avertissement d'une facture EXISTANTE au total sous le centime.
//
// ⚠️ Chaque test nomme la mutation qu'il attrape. i18nMsg rend son repli, variables
// substituées : les textes vérifiés sont ceux du repli français (= catalogue fr-CH).

import { describe, it, expect, vi, afterEach } from 'vitest';
import { render, cleanup } from '@testing-library/svelte';

vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string, args?: Record<string, string | number>) =>
		args
			? fallback.replace(/\{\s*\$(\w+)\s*\}/g, (_, n) => String(args[n] ?? ''))
			: fallback,
}));

import SupplierInvoiceTotal from './SupplierInvoiceTotal.svelte';

const AVERTISSEMENT =
	'Le total de cette facture (100.0040) n’est pas au centime : un paiement bancaire de ce montant ne pourra être rapproché d’aucune transaction de relevé. Voir le manuel, § « Une facture fournisseur au total sous le centime ».';

afterEach(() => cleanup());

describe('SupplierInvoiceTotal', () => {
	it.each(['open', 'paid'] as const)(
		'%s, 100.0040 : le TTC exact et l’avertissement au texte exact (M11, M14, M17)',
		(status) => {
			const { getByTestId } = render(SupplierInvoiceTotal, {
				props: { totalAmount: '100.0040', status },
			});
			expect(getByTestId('supplier-invoice-total').textContent?.trim()).toBe('100.0040');
			expect(getByTestId('supplier-invoice-subcentime-warning').textContent?.trim()).toBe(AVERTISSEMENT);
		},
	);

	it('annulée, 100.0040 : aucun avertissement (M7), le TTC reste exact', () => {
		const { getByTestId, queryByTestId } = render(SupplierInvoiceTotal, {
			props: { totalAmount: '100.0040', status: 'cancelled' },
		});
		expect(queryByTestId('supplier-invoice-subcentime-warning')).toBeNull();
		expect(getByTestId('supplier-invoice-total').textContent?.trim()).toBe('100.0040');
	});

	it('au centime : aucun avertissement, deux décimales', () => {
		const { getByTestId, queryByTestId } = render(SupplierInvoiceTotal, {
			props: { totalAmount: '100.0000', status: 'open' },
		});
		expect(queryByTestId('supplier-invoice-subcentime-warning')).toBeNull();
		expect(getByTestId('supplier-invoice-total').textContent?.trim()).toBe('100.00');
	});
});
