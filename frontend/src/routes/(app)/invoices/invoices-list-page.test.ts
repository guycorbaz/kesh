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

import { tick } from 'svelte';
import Page from './+page.svelte';
import { listInvoices } from '$lib/features/invoices/invoices.api';
import { notifyError } from '$lib/shared/utils/notify';

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
		// Le dialogue est rendu en portail : le lire seul, sans la ligne de la liste (dont
		// la date est l'objet de l'autre test).
		const dialog = () => document.querySelector('[role="dialog"]')?.textContent ?? '';
		await waitFor(() =>
			expect(dialog()).toContain(
				'Confirmer la suppression de la facture du 02.03.2026 pour Client Démo SA',
			),
		);
		expect(dialog()).not.toContain('2026-03-02');
	});
});

describe('liste des factures — une date en cours de frappe (Story 33-4)', () => {
	const frappe = ['0002-11-06', '0020-11-06', '0202-11-06'];

	for (const id of ['invoice-date-from', 'invoice-date-to']) {
		it(`#${id} : l’année tapée n’interroge pas, ne notifie rien, ne vide pas la liste (mutation : valeur appliquée sans contrôle)`, async () => {
			const { findByTestId, container } = render(Page);
			await findByTestId('invoice-row-date');
			const input = container.querySelector(`#${id}`) as HTMLInputElement;
			const avant = vi.mocked(listInvoices).mock.calls.length;
			for (const v of frappe) {
				await fireEvent.input(input, { target: { value: v } });
				expect(input.value, 'le champ garde la frappe').toBe(v);
			}
			await tick();
			expect(vi.mocked(listInvoices).mock.calls.length).toBe(avant);
			expect(notifyError).not.toHaveBeenCalled();
			expect((await findByTestId('invoice-row-date')).textContent?.trim()).toBe('02.03.2026');

			await fireEvent.input(input, { target: { value: '2026-11-06' } });
			const cle = id === 'invoice-date-from' ? 'dateFrom' : 'dateTo';
			await waitFor(() =>
				expect(vi.mocked(listInvoices)).toHaveBeenLastCalledWith(
					expect.objectContaining({ [cle]: '2026-11-06' }),
				),
			);
		});
	}

	it('la sortie du champ restaure la dernière date valide', async () => {
		const { findByTestId, container } = render(Page);
		await findByTestId('invoice-row-date');
		const input = container.querySelector('#invoice-date-from') as HTMLInputElement;
		await fireEvent.input(input, { target: { value: '2026-01-31' } });
		await fireEvent.input(input, { target: { value: '0202-01-31' } });
		await fireEvent.blur(input);
		expect(input.value).toBe('2026-01-31');
	});
});
