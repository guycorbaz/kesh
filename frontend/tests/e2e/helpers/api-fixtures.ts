/**
 * Fixtures API partagées entre specs Playwright (Story 20-4 — extraites
 * d'`invoices.spec.ts`, DRY : consommées aussi par `invoice-send-email.spec.ts`).
 *
 * Toutes passent par `authedApiContext` (cookies HttpOnly clonés du browser
 * context — exige un `login(page)` préalable) et ciblent le backend
 * `KESH_BACKEND_URL`.
 */
import { expect } from '@playwright/test';
import type { Page } from '@playwright/test';
import { authedApiContext, disposeContextSafe } from './test-state';

/**
 * Contact `Personne` PDF-ready (adresse structurée #213, firstName/lastName
 * obligatoires). `email`/`salutation` optionnels (Story 20-4 : le round-trip
 * d'envoi exige un contact AVEC e-mail et civilité genrée — le backend
 * refuse l'envoi sans e-mail, 400 CONTACT_EMAIL_MISSING).
 */
export async function createContactWithAddressViaApi(
	page: Page,
	name: string,
	email?: string,
	salutation?: 'Monsieur' | 'Madame' | 'Neutre',
	paymentTermsDays?: number,
): Promise<number> {
	const ctx = await authedApiContext(page);
	try {
		const res = await ctx.post('/api/v1/contacts', {
			data: {
				contactType: 'Personne',
				name,
				firstName: 'Pia',
				lastName: name,
				isClient: true,
				isSupplier: false,
				addressStructured: {
					street: 'Marktgasse',
					building: '28',
					postalCode: '9400',
					city: 'Rorschach',
					country: 'CH',
				},
				defaultPaymentTerms: '30 jours net',
				// #245 : délai structuré optionnel (prime sur le texte libre).
				...(paymentTermsDays !== undefined
					? { defaultPaymentTermsDays: paymentTermsDays }
					: {}),
				...(email !== undefined ? { email } : {}),
				...(salutation !== undefined ? { salutation } : {}),
			},
		});
		expect(res.ok(), `createContactWithAddress failed: ${res.status()}`).toBeTruthy();
		return (await res.json()).id as number;
	} finally {
		await disposeContextSafe(ctx);
	}
}

/** Le QR-bill exige un compte bancaire principal (le seed `with-company`
 * n'en configure pas depuis v014-1). 409 toléré (déjà créé). */
export async function ensurePrimaryBankAccountViaApi(page: Page): Promise<void> {
	const ctx = await authedApiContext(page);
	try {
		const resp = await ctx.post('/api/v1/bank-accounts', {
			data: {
				bankName: 'Banque E2E PDF',
				iban: 'CH9300762011623852957',
				isPrimary: true,
			},
		});
		expect([200, 201, 409]).toContain(resp.status());
	} finally {
		await disposeContextSafe(ctx);
	}
}

/** Crée une facture 1 ligne et la valide. Retourne son id. */
export async function createAndValidateInvoiceViaApi(
	page: Page,
	contactId: number,
): Promise<number> {
	const today = new Date().toISOString().slice(0, 10);
	const ctx = await authedApiContext(page);
	try {
		const createRes = await ctx.post('/api/v1/invoices', {
			data: {
				contactId,
				date: today,
				dueDate: today,
				paymentTerms: '30 jours net',
				lines: [
					{
						description: 'Conseil stratégique',
						quantity: '4.5',
						unitPrice: '200.00',
						vatRate: '8.10',
					},
				],
			},
		});
		expect(createRes.ok(), `create invoice failed: ${createRes.status()}`).toBeTruthy();
		const invoice = await createRes.json();
		const validateRes = await ctx.post(`/api/v1/invoices/${invoice.id}/validate`);
		expect(validateRes.ok(), `validate failed: ${validateRes.status()}`).toBeTruthy();
		return invoice.id as number;
	} finally {
		await disposeContextSafe(ctx);
	}
}
