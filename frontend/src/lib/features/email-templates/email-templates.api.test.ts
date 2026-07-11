// Story 20-2 — Tests unitaires Vitest du client API email-templates.
//
// Vérifie le contrat figé 20-1 :
// - URLs construites avec la langue en MAJUSCULES (`Language::from_str`
//   backend est case-sensitive).
// - Méthodes HTTP correctes (GET liste, GET/PUT/DELETE unique).
// - Erreurs structurées backend (409 OPTIMISTIC_LOCK_CONFLICT, 422
//   EMAIL_TEMPLATE_UNKNOWN_VARIABLES) propagées en `ApiError` typé avec
//   `details.unknownVariables` accessible.

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import {
	listEmailTemplates,
	getEmailTemplate,
	updateEmailTemplate,
	restoreEmailTemplateDefault,
} from './email-templates.api';
import { isApiError } from '$lib/shared/utils/api-client';

/** Réponse JSON OK générique. */
function okJson(status: number, body: unknown): Partial<Response> {
	return {
		ok: true,
		status,
		json: () => Promise.resolve(body),
		headers: new Headers(),
	};
}

/** Réponse 204 sans corps (DELETE restaure défaut). */
function noContentResponse(): Partial<Response> {
	return {
		ok: true,
		status: 204,
		json: () => Promise.reject(new SyntaxError('Unexpected end of JSON input')),
		headers: new Headers(),
	};
}

/** Réponse d'erreur structurée `{ error: { code, message, details? } }`. */
function errorResponse(
	status: number,
	code: string,
	message: string,
	details?: Record<string, unknown>,
): Partial<Response> {
	return {
		ok: false,
		status,
		json: () => Promise.resolve({ error: { code, message, details } }),
		headers: new Headers(),
	};
}

const SAMPLE: unknown = {
	templateType: 'invoice_send',
	language: 'FR',
	subject: 'Facture {invoiceNumber}',
	body: '{salutation}, montant {amount}. {companyName}',
	version: 1,
	isDefault: false,
	allowedVariables: ['salutation', 'contactName', 'invoiceNumber', 'amount', 'dueDate', 'companyName'],
};

describe('email-templates.api', () => {
	let mockFetch: ReturnType<typeof vi.fn>;

	beforeEach(() => {
		mockFetch = vi.fn().mockResolvedValue(okJson(200, [SAMPLE]) as Response);
		vi.stubGlobal('fetch', mockFetch);
	});

	afterEach(() => {
		vi.unstubAllGlobals();
	});

	it('listEmailTemplates GET /api/v1/admin/email-templates', async () => {
		await listEmailTemplates();
		const [url, init] = mockFetch.mock.calls[0] as [string, RequestInit];
		expect(url).toBe('/api/v1/admin/email-templates');
		expect(init.method ?? 'GET').toBe('GET');
	});

	it('getEmailTemplate construit une URL langue en MAJUSCULES', async () => {
		mockFetch.mockResolvedValue(okJson(200, SAMPLE) as Response);
		await getEmailTemplate('invoice_send', 'FR');
		const [url] = mockFetch.mock.calls[0] as [string, RequestInit];
		expect(url).toBe('/api/v1/admin/email-templates/invoice_send/FR');
	});

	it('updateEmailTemplate PUT avec payload subject/body/expectedVersion', async () => {
		mockFetch.mockResolvedValue(okJson(200, SAMPLE) as Response);
		await updateEmailTemplate('invoice_send', 'DE', {
			subject: 'S',
			body: 'B',
			expectedVersion: 3,
		});
		const [url, init] = mockFetch.mock.calls[0] as [string, RequestInit];
		expect(url).toBe('/api/v1/admin/email-templates/invoice_send/DE');
		expect(init.method).toBe('PUT');
		expect(JSON.parse(init.body as string)).toEqual({
			subject: 'S',
			body: 'B',
			expectedVersion: 3,
		});
	});

	it('restoreEmailTemplateDefault DELETE et résout sur 204 sans corps', async () => {
		mockFetch.mockResolvedValue(noContentResponse() as Response);
		await expect(restoreEmailTemplateDefault('invoice_send', 'IT')).resolves.toBeUndefined();
		const [url, init] = mockFetch.mock.calls[0] as [string, RequestInit];
		expect(url).toBe('/api/v1/admin/email-templates/invoice_send/IT');
		expect(init.method).toBe('DELETE');
	});

	it('propage le 409 OPTIMISTIC_LOCK_CONFLICT en ApiError typé', async () => {
		mockFetch.mockResolvedValue(
			errorResponse(409, 'OPTIMISTIC_LOCK_CONFLICT', 'Conflit de version') as Response,
		);
		try {
			await updateEmailTemplate('invoice_send', 'FR', { subject: 'S', body: 'B', expectedVersion: 1 });
			expect.unreachable('aurait dû throw');
		} catch (err) {
			expect(isApiError(err)).toBe(true);
			if (isApiError(err)) {
				expect(err.status).toBe(409);
				expect(err.code).toBe('OPTIMISTIC_LOCK_CONFLICT');
			}
		}
	});

	it('propage le 422 EMAIL_TEMPLATE_UNKNOWN_VARIABLES avec details.unknownVariables', async () => {
		mockFetch.mockResolvedValue(
			errorResponse(422, 'EMAIL_TEMPLATE_UNKNOWN_VARIABLES', 'Variables inconnues', {
				unknownVariables: ['foo', 'bar'],
			}) as Response,
		);
		try {
			await updateEmailTemplate('invoice_send', 'FR', { subject: '{foo}', body: '{bar}' });
			expect.unreachable('aurait dû throw');
		} catch (err) {
			expect(isApiError(err)).toBe(true);
			if (isApiError(err)) {
				expect(err.status).toBe(422);
				expect(err.code).toBe('EMAIL_TEMPLATE_UNKNOWN_VARIABLES');
				expect(err.details?.unknownVariables).toEqual(['foo', 'bar']);
			}
		}
	});
});
