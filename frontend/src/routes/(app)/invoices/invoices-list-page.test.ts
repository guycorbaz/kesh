// Story v0171 (#879) — la liste des factures clientes : la date d'une ligne et la date
// citée par la confirmation de suppression s'affichent en forme suisse (`02.03.2026`,
// jamais `2026-03-02`). Mutation M5 : retirer `formatSwissDate` d'un site rougit le
// test qui le nomme.
//
// Le nom du fichier n'est pas `+page.test.ts` : SvelteKit réserve le préfixe `+`.

import { describe, it, expect, vi, afterEach } from 'vitest';
import { render, fireEvent, cleanup, waitFor } from '@testing-library/svelte';
import type { InvoiceListItemResponse } from '$lib/features/invoices/invoices.types';

vi.mock('$app/navigation', () => ({ goto: vi.fn() }));
vi.mock('$app/state', () => ({
	page: { params: {}, url: new URL('http://localhost/invoices') },
}));
vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_k: string, fallback: string, args?: Record<string, string | number>) =>
		args ? fallback.replace(/\{\s*\$(\w+)\s*\}/g, (_, n) => String(args[n] ?? '')) : fallback,
}));
vi.mock('$lib/shared/utils/notify', () => ({ notifyError: vi.fn(), notifySuccess: vi.fn() }));
vi.mock('$lib/app/stores/auth.svelte', () => ({
	authState: { currentUser: { role: 'Admin', username: 'a', userId: '1' } },
}));
vi.mock('$lib/features/contacts/contacts.api', () => ({
	getContact: vi.fn(),
	listContacts: vi.fn(async () => ({ items: [], total: 0, offset: 0, limit: 20 })),
}));
vi.mock('$lib/features/invoices/invoices.api', () => ({
	listInvoices: vi.fn(async () => ({
		items: [facture()],
		total: 1,
		offset: 0,
		limit: 20,
	})),
	deleteInvoice: vi.fn(),
}));

import Page from './+page.svelte';

function facture(over: Partial<InvoiceListItemResponse> = {}): InvoiceListItemResponse {
	return {
		id: 5,
		companyId: 1,
		contactId: 1,
		contactName: 'Client Démo SA',
		invoiceNumber: null,
		status: 'draft',
		date: '2026-03-02',
		dueDate: '2026-04-01',
		paymentTerms: null,
		totalAmount: '100.00',
		totalTtc: '100.00',
		amountSettled: '0.00',
		amountDue: '100.00',
		paidAt: null,
		dunningPausedAt: null,
		dunningPausedNote: null,
		version: 1,
		createdAt: '2026-03-02T09:00:00',
		...over,
	} as InvoiceListItemResponse;
}

afterEach(() => {
	cleanup();
	vi.clearAllMocks();
});

describe('liste des factures — dates (v0171)', () => {
	it('la date d’une ligne `02.03.2026` (mutation M5 : site `inv.date`)', async () => {
		const { findByTestId, container } = render(Page);
		expect((await findByTestId('invoice-row-date')).textContent?.trim()).toBe('02.03.2026');
		expect(container.textContent).not.toContain('2026-03-02');
	});

	it('la confirmation de suppression cite `02.03.2026` (mutation M5 : site `deleteTarget.date`)', async () => {
		const { findByLabelText } = render(Page);
		await fireEvent.click(await findByLabelText('Supprimer la facture Client Démo SA'));
		await waitFor(() =>
			expect(document.body.textContent).toContain(
				'Confirmer la suppression de la facture du 02.03.2026 pour Client Démo SA',
			),
		);
		expect(document.body.textContent).not.toContain('2026-03-02');
	});
});
