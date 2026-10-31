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
import { authedApiContext, clearAuthStorage, disposeContextSafe } from './test-state';

/** Mot de passe des utilisateurs créés par `createUser`. */
export const USER_PASSWORD = 'MotDePasse12345';

/**
 * Crée un utilisateur du rôle donné (`Admin`, `Comptable`, `Consultation`), en se
 * connectant comme l'Admin du seed ; se déconnecte ensuite et rend son nom. Son mot de
 * passe est `USER_PASSWORD`. Extraite d'`audit-log.spec.ts` (Story v016-2b, #816) :
 * `bank-manual-statement.spec.ts` l'emploie aussi.
 */
export async function createUser(page: Page, role: string): Promise<string> {
	await page.goto('/login');
	await page.fill('#username', 'admin');
	await page.fill('#password', 'admin123');
	await page.click('button[type="submit"]');
	await expect(page).toHaveURL('/');
	const ctx = await authedApiContext(page);
	const username = `${role.toLowerCase()}-${Date.now()}-${Math.floor(Math.random() * 1e6)}`;
	try {
		const res = await ctx.post('/api/v1/users', {
			data: { username, password: USER_PASSWORD, role },
		});
		expect(res.ok(), `création de l'utilisateur ${role} : ${res.status()}`).toBeTruthy();
	} finally {
		await disposeContextSafe(ctx);
	}
	await clearAuthStorage(page);
	return username;
}

/**
 * Échéance suffisamment passée pour être éligible au niveau 1 (seuil `today - 15j` :
 * grâce 5 + délai niveau 1 = 10). Story 21-6c — promue depuis `reminders.spec.ts`
 * (M1) pour être réutilisée par `homepage-reminders.spec.ts` sans duplication.
 */
export function overdueDate(days = 25): string {
	const d = new Date();
	d.setDate(d.getDate() - days);
	return d.toISOString().slice(0, 10);
}

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
/**
 * Crée et valide une facture.
 *
 * `dueDate` optionnel (défaut = aujourd'hui) : passer une échéance passée permet
 * de produire une facture ÉLIGIBLE À UN RAPPEL. Seuil niveau 1 (config seedée
 * par défaut) = `today - 15j` (grâce 5 + délai niveau 1 = 10). Story 21-6b.
 *
 * La date de facture est alignée sur `dueDate` quand celui-ci est fourni : la
 * validation #245 (`due_date >= date`) rejette une échéance antérieure à la
 * date de facture — une facture échue est donc émise ET échue dans le passé.
 */
export async function createAndValidateInvoiceViaApi(
	page: Page,
	contactId: number,
	dueDate?: string,
): Promise<number> {
	const today = new Date().toISOString().slice(0, 10);
	const invoiceDate = dueDate ?? today;
	const ctx = await authedApiContext(page);
	try {
		const createRes = await ctx.post('/api/v1/invoices', {
			data: {
				contactId,
				date: invoiceDate,
				dueDate: dueDate ?? today,
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

/**
 * Story 25-6-a (#388) — poste une écriture équilibrée à deux lignes (compte de
 * classe 10 au débit, de classe 3 au crédit), datée du jour : la plus récente,
 * donc en tête de la tuile « Dernières écritures ». Renvoie son id.
 */
export async function createJournalEntryViaApi(
	page: Page,
	description: string,
	amount = '321.40',
): Promise<number> {
	const ctx = await authedApiContext(page);
	try {
		const accRes = await ctx.get('/api/v1/accounts?includeArchived=false');
		expect(accRes.ok(), `list accounts: ${accRes.status()}`).toBeTruthy();
		const accounts: Array<{ id: number; number: string }> = await accRes.json();
		const debit = accounts.find((a) => /^10[0-9]{2}$/.test(a.number)) ?? accounts[0];
		const credit = accounts.find((a) => /^3[0-9]{3}$/.test(a.number)) ?? accounts[1];
		const res = await ctx.post('/api/v1/journal-entries', {
			data: {
				// Date LOCALE (Zurich), pas UTC : entre minuit et 2 h, la date UTC est
				// la veille — et le 1er janvier, un autre exercice.
				entryDate: new Date().toLocaleDateString('sv-SE', { timeZone: 'Europe/Zurich' }),
				journal: 'OD',
				description,
				lines: [
					{ accountId: debit.id, debit: amount, credit: '0.00' },
					{ accountId: credit.id, debit: '0.00', credit: amount },
				],
			},
		});
		expect(res.ok(), `create entry: ${res.status()}`).toBeTruthy();
		return ((await res.json()) as { id: number }).id;
	} finally {
		await disposeContextSafe(ctx);
	}
}

/**
 * Story 30-1b (#686) — le compte bancaire du relevé `iban`, créé au besoin et câblé sur le
 * compte `1000` du grand livre. Rend son id. Partagé par `reconciliation-link.spec.ts` et
 * `reconciliation-supplier.spec.ts` (revue de code P1 de la 30-1b, A1 : la copie locale de
 * la première a été retirée).
 */
export async function wiredBankAccountViaApi(page: Page, iban: string): Promise<number> {
	const ctx = await authedApiContext(page);
	try {
		type Ba = { id: number; iban: string; version: number; journalAccountId: number | null };
		const find = async (): Promise<Ba | undefined> => {
			const list = await ctx.get('/api/v1/bank-accounts');
			expect(list.ok()).toBeTruthy();
			return ((await list.json()) as Ba[]).find((b) => b.iban === iban);
		};
		let ba = await find();
		if (!ba) {
			const res = await ctx.post('/api/v1/bank-accounts', {
				data: { bankName: 'UBS Test', iban, qrIban: null, isPrimary: true },
			});
			expect(res.ok(), `bank-account create: ${res.status()}`).toBeTruthy();
			ba = await find();
		}
		if (ba!.journalAccountId === null) {
			const accounts = (await (await ctx.get('/api/v1/accounts')).json()) as Array<{
				id: number;
				number: string;
			}>;
			const ledger = accounts.find((a) => a.number === '1000');
			expect(ledger, 'compte 1000 du préréglage').toBeTruthy();
			const wired = await ctx.patch(`/api/v1/bank-accounts/${ba!.id}`, {
				data: { journalAccountId: ledger!.id, version: ba!.version },
			});
			expect(wired.ok(), `bank-account wiring: ${wired.status()}`).toBeTruthy();
		}
		return ba!.id;
	} finally {
		await disposeContextSafe(ctx);
	}
}

/**
 * Story 30-1b (#686) — un fournisseur et une facture fournisseur **ouverte** d'une ligne
 * (sans TVA) de `amount`, créés par l'API (patron de `payment-batches.spec.ts`). Le compte
 * créanciers est désigné aux réglages s'il ne l'est pas (le seed `with-company` ne le désigne
 * pas). `paymentReference` et `number` sont facultatifs (la facture « sans QR » n'en porte
 * aucun). Rend `{ supplierId, supplierInvoiceId }`.
 */
export async function createSupplierInvoiceViaApi(
	page: Page,
	opts: {
		supplierName: string;
		number: string | null;
		paymentReference: string | null;
		invoiceDate: string;
		dueDate: string | null;
		amount: string;
	},
): Promise<{ supplierId: number; supplierInvoiceId: number }> {
	const ctx = await authedApiContext(page);
	try {
		const accounts = (await (await ctx.get('/api/v1/accounts')).json()) as Array<{
			id: number;
			number: string;
			accountType: string;
			active: boolean;
			postable: boolean;
		}>;
		const payable = accounts.find((a) => a.number === '2000');
		const expense = accounts.find((a) => a.accountType === 'Expense' && a.active && a.postable);
		expect(payable && expense, 'comptes 2000 et de charge').toBeTruthy();
		const settings = await (await ctx.get('/api/v1/company/invoice-settings')).json();
		if (settings.defaultPayableAccountId == null) {
			const put = await ctx.put('/api/v1/company/invoice-settings', {
				data: { ...settings, defaultPayableAccountId: payable!.id },
			});
			expect(put.ok(), `compte créanciers : ${put.status()}`).toBeTruthy();
		}
		const supRes = await ctx.post('/api/v1/contacts', {
			data: {
				contactType: 'Entreprise',
				name: opts.supplierName,
				isClient: false,
				isSupplier: true,
			},
		});
		expect(supRes.ok(), `fournisseur : ${supRes.status()}`).toBeTruthy();
		const supplierId = (await supRes.json()).id as number;
		const invRes = await ctx.post('/api/v1/supplier-invoices', {
			data: {
				contactId: supplierId,
				supplierInvoiceNumber: opts.number,
				invoiceDate: opts.invoiceDate,
				dueDate: opts.dueDate,
				paymentReference: opts.paymentReference,
				lines: [
					{
						description: 'Fourniture',
						quantity: '1',
						unitPrice: opts.amount,
						vatRate: '0',
						expenseAccountId: expense!.id,
					},
				],
			},
		});
		expect(invRes.ok(), `facture fournisseur : ${invRes.status()} ${await invRes.text()}`).toBeTruthy();
		return { supplierId, supplierInvoiceId: (await invRes.json()).id as number };
	} finally {
		await disposeContextSafe(ctx);
	}
}
