/**
 * Story v017-4b (#849, #831 ; AC-21) — la liste des factures fournisseurs : le badge du
 * brouillon, le total aux décimales exactes, les annulées masquées par défaut.
 *
 * ⚠️ Chaque test nomme la MUTATION qu'il attrape. Patron des `*-page.test.ts` : mocks
 * hoistés AVANT l'import du composant.
 */
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, fireEvent, cleanup, waitFor } from '@testing-library/svelte';
import type { SupplierInvoiceListItemResponse } from '$lib/features/supplier-invoices/supplier-invoices.types';

vi.mock('$app/navigation', () => ({ goto: vi.fn() }));
vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_k: string, fallback: string, args?: Record<string, string | number>) =>
		args ? fallback.replace(/\{\s*\$(\w+)\s*\}/g, (_, n) => String(args[n] ?? '')) : fallback,
}));
vi.mock('$lib/app/stores/auth.svelte', () => ({
	authState: { currentUser: { role: 'Comptable', username: 'c', userId: '1' } },
}));
const listMock = vi.fn();
vi.mock('$lib/features/supplier-invoices/supplier-invoices.api', () => ({
	listSupplierInvoices: (q: unknown) => listMock(q),
}));

import Page from './+page.svelte';

function item(over: Partial<SupplierInvoiceListItemResponse>): SupplierInvoiceListItemResponse {
	return {
		id: 1,
		contactId: 1,
		contactName: 'Dubarde SA',
		supplierInvoiceNumber: 'FF-1',
		status: 'open',
		invoiceDate: '2026-05-10',
		dueDate: null,
		totalAmount: '100.00',
		...over,
	};
}

beforeEach(() => {
	listMock.mockReset();
	listMock.mockImplementation(async () => ({
		items: [
			item({ id: 1, status: 'draft', supplierInvoiceNumber: 'FF-BR', totalAmount: '100.0040' }),
			item({ id: 2, status: 'open', supplierInvoiceNumber: 'FF-VA', totalAmount: '100.00' }),
		],
		total: 2,
		offset: 0,
		limit: 100,
	}));
});
afterEach(() => cleanup());

describe('liste des factures fournisseurs', () => {
	it('demande par défaut les brouillons, validées et payées — jamais les annulées (M28)', async () => {
		render(Page);
		await waitFor(() => expect(listMock).toHaveBeenCalledTimes(1));
		expect(listMock.mock.calls[0][0]).toEqual({ limit: 100, status: ['draft', 'open', 'paid'] });
	});

	it('la case « Afficher les factures annulées » relit la liste sans filtre', async () => {
		const { findByTestId } = render(Page);
		const box = (await findByTestId('supplier-invoices-show-cancelled')) as HTMLInputElement;
		expect(box.checked).toBe(false);
		await fireEvent.click(box);
		await waitFor(() => expect(listMock).toHaveBeenCalledTimes(2));
		expect(listMock.mock.calls[1][0]).toEqual({ limit: 100 });
		await fireEvent.click(box);
		await waitFor(() => expect(listMock).toHaveBeenCalledTimes(3));
		expect(listMock.mock.calls[2][0]).toEqual({ limit: 100, status: ['draft', 'open', 'paid'] });
	});

	it('un brouillon porte le badge « Brouillon », une facture validée « Validée »', async () => {
		const { findAllByTestId } = render(Page);
		const badges = await findAllByTestId('supplier-invoice-status-badge');
		expect(badges.map((b) => b.textContent?.trim())).toEqual(['Brouillon', 'Validée']);
		expect(badges[0].getAttribute('data-status')).toBe('draft');
	});

	it('le TTC s’affiche aux décimales exactes : 100.0040 reste 100.0040 (#831, M27)', async () => {
		const { findAllByTestId } = render(Page);
		const totals = await findAllByTestId('supplier-invoice-row-total');
		expect(totals.map((t) => t.textContent?.trim())).toEqual(['100.0040', '100.00']);
	});

	// Revue P1 (B-4 = E-8) : deux bascules rapprochées, réponses croisées — la liste est
	// celle de la DERNIÈRE bascule.
	it('une réponse périmée ne remplace pas celle de la dernière bascule', async () => {
		const { findByTestId, findAllByTestId } = render(Page);
		const box = (await findByTestId('supplier-invoices-show-cancelled')) as HTMLInputElement;
		await findAllByTestId('supplier-invoice-row');
		let resolveFirst: (v: unknown) => void = () => {};
		listMock.mockImplementationOnce(
			() =>
				new Promise((r) => {
					resolveFirst = r;
				}),
		);
		listMock.mockImplementationOnce(async () => ({
			items: [item({ id: 3, status: 'draft', supplierInvoiceNumber: 'FF-DERNIERE' })],
			total: 1,
			offset: 0,
			limit: 100,
		}));
		await fireEvent.click(box); // cochée : lecture lente
		await fireEvent.click(box); // décochée : lecture rapide
		await waitFor(() => expect(listMock).toHaveBeenCalledTimes(3));
		resolveFirst({
			items: [item({ id: 4, status: 'cancelled', supplierInvoiceNumber: 'FF-PERIMEE' })],
			total: 1,
			offset: 0,
			limit: 100,
		});
		await waitFor(async () => {
			const rows = await findAllByTestId('supplier-invoice-row');
			expect(rows.map((r) => r.textContent)).toEqual([expect.stringContaining('FF-DERNIERE')]);
		});
		await new Promise((r) => setTimeout(r, 20));
		const rows = await findAllByTestId('supplier-invoice-row');
		expect(rows.map((r) => r.textContent)).toEqual([expect.stringContaining('FF-DERNIERE')]);
		expect(box.checked).toBe(false);
	});
});
