// Story 20-3b2 — tests Vitest des wrappers envoi de facture par e-mail.
import { describe, expect, it, vi } from 'vitest';

vi.mock('$lib/shared/utils/api-client', () => ({
	apiClient: {
		get: vi.fn(),
		post: vi.fn(),
		put: vi.fn(),
		patch: vi.fn(),
		delete: vi.fn(),
	},
}));

import { apiClient } from '$lib/shared/utils/api-client';
import { getInvoiceEmailPreview, listInvoices, sendInvoiceEmail } from './invoices.api';

describe('invoices.api — envoi par e-mail (20-3b2)', () => {
	it('getInvoiceEmailPreview GET le bon path', async () => {
		(apiClient.get as ReturnType<typeof vi.fn>).mockResolvedValue({
			to: 'pia@example.ch',
			language: 'FR',
			subject: 'Facture',
			body: 'Bonjour',
		});
		const preview = await getInvoiceEmailPreview(42);
		expect(apiClient.get).toHaveBeenCalledWith('/api/v1/invoices/42/email-preview');
		expect(preview.to).toBe('pia@example.ch');
	});

	it('sendInvoiceEmail POST subject/body au bon path (jamais de champ to)', async () => {
		(apiClient.post as ReturnType<typeof vi.fn>).mockResolvedValue({ id: 42, emailedAt: 'x' });
		await sendInvoiceEmail(42, { subject: 'Facture F-1', body: 'Bonjour' });
		expect(apiClient.post).toHaveBeenCalledWith('/api/v1/invoices/42/send-email', {
			subject: 'Facture F-1',
			body: 'Bonjour',
		});
		const payload = (apiClient.post as ReturnType<typeof vi.fn>).mock.calls[0][1] as Record<
			string,
			unknown
		>;
		expect('to' in payload).toBe(false);
	});
});

// Story 21-6a (D10) — filtre « rappels suspendus » de la liste des factures.
describe('invoices.api — filtre paused (21-6a)', () => {
	const emptyList = { items: [], total: 0, offset: 0, limit: 20 };

	it('sérialise ?paused=paused', async () => {
		(apiClient.get as ReturnType<typeof vi.fn>).mockResolvedValue(emptyList);
		await listInvoices({ paused: 'paused' });
		expect(apiClient.get).toHaveBeenCalledWith('/api/v1/invoices?paused=paused');
	});

	it('sérialise ?paused=not-paused', async () => {
		(apiClient.get as ReturnType<typeof vi.fn>).mockResolvedValue(emptyList);
		await listInvoices({ paused: 'not-paused' });
		expect(apiClient.get).toHaveBeenCalledWith('/api/v1/invoices?paused=not-paused');
	});

	it("omet le param quand la valeur vaut 'all' (défaut backend, no-op)", async () => {
		(apiClient.get as ReturnType<typeof vi.fn>).mockResolvedValue(emptyList);
		await listInvoices({ paused: 'all' });
		expect(apiClient.get).toHaveBeenCalledWith('/api/v1/invoices');
	});

	it('omet le param quand il est absent', async () => {
		(apiClient.get as ReturnType<typeof vi.fn>).mockResolvedValue(emptyList);
		await listInvoices({});
		expect(apiClient.get).toHaveBeenCalledWith('/api/v1/invoices');
	});

	it('cohabite avec les autres filtres sans les écraser', async () => {
		(apiClient.get as ReturnType<typeof vi.fn>).mockResolvedValue(emptyList);
		await listInvoices({ status: 'validated', paused: 'paused', limit: 20 });
		expect(apiClient.get).toHaveBeenCalledWith(
			'/api/v1/invoices?status=validated&paused=paused&limit=20',
		);
	});
});
