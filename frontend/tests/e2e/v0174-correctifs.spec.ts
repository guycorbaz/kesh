/**
 * Story v0174-correctifs-3 (0.17.4) — les écrans de la release, de bout en bout, sur le
 * serveur réellement servi : toutes les factures validées d'un lot (#888, AC-3.4), le refus
 * du repassage en brouillon qui nomme la date du verrou (#869, AC-3.13), le projet archivé
 * d'un brouillon sous une racine archivée (#872), le pays d'une adresse (#817).
 *
 * Décor synthétique : le seed `with-company`, des fournisseurs, factures et projets créés par
 * l'API. Les captures de la recette visuelle (thème clair) s'écrivent dans
 * `KESH_E2E_CAPTURES_DIR`, sinon dans la sortie du test.
 */
import path from 'node:path';
import { expect, test, type APIRequestContext, type Page, type TestInfo } from '@playwright/test';
import {
	seedTestState,
	clearAuthStorage,
	authedApiContext,
	disposeContextSafe,
} from './helpers/test-state';

test.beforeAll(async () => {
	await seedTestState('with-company');
});

test.afterEach(async ({ page }) => {
	await clearAuthStorage(page);
});

function capturePath(testInfo: TestInfo, name: string): string {
	const dossier = process.env.KESH_E2E_CAPTURES_DIR;
	return dossier ? path.join(dossier, `${name}.png`) : testInfo.outputPath(`${name}.png`);
}

async function capture(page: Page, testInfo: TestInfo, name: string): Promise<void> {
	await page.screenshot({ path: capturePath(testInfo, `v0174c-${name}`), fullPage: true });
}

async function login(page: Page) {
	await page.goto('/login');
	await page.fill('#username', 'admin');
	await page.fill('#password', 'admin123');
	await page.click('button[type="submit"]');
	await expect(page).toHaveURL('/');
}

function uniq(prefix: string): string {
	return `${prefix} ${Date.now()}-${Math.floor(Math.random() * 1e6)}`;
}

/** Une date `AAAA-MM-JJ` décalée de `days` jours depuis aujourd'hui (UTC, décor seulement). */
function daysFromToday(days: number): string {
	const d = new Date();
	d.setUTCDate(d.getUTCDate() + days);
	return d.toISOString().slice(0, 10);
}

/** Les comptes et réglages du décor : compte de charge, compte créanciers désigné. */
async function expenseAccount(ctx: APIRequestContext): Promise<number> {
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
	const s = await (await ctx.get('/api/v1/company/invoice-settings')).json();
	if (s.defaultPayableAccountId == null) {
		const put = await ctx.put('/api/v1/company/invoice-settings', {
			data: { ...s, defaultPayableAccountId: payable!.id },
		});
		expect(put.ok(), `compte créanciers : ${put.status()}`).toBeTruthy();
	}
	return expense!.id;
}

async function supplier(ctx: APIRequestContext, name: string): Promise<number> {
	const res = await ctx.post('/api/v1/contacts', {
		data: { contactType: 'Entreprise', name, isClient: false, isSupplier: true },
	});
	expect(res.ok(), `fournisseur : ${res.status()}`).toBeTruthy();
	return (await res.json()).id as number;
}

/** Un brouillon de facture fournisseur ; validé si `validate`. Rend `{ id, version }`. */
async function supplierInvoice(
	ctx: APIRequestContext,
	opts: {
		contactId: number;
		expenseAccountId: number;
		invoiceDate: string;
		projectId?: number | null;
		validate: boolean;
	},
): Promise<{ id: number; version: number }> {
	const res = await ctx.post('/api/v1/supplier-invoices', {
		data: {
			contactId: opts.contactId,
			supplierInvoiceNumber: uniq('FF'),
			invoiceDate: opts.invoiceDate,
			creditorIban: 'CH5604835012345678009',
			projectId: opts.projectId ?? null,
			lines: [
				{
					description: 'Fourniture',
					quantity: '1',
					unitPrice: '10.00',
					vatRate: '0',
					expenseAccountId: opts.expenseAccountId,
				},
			],
		},
	});
	expect(res.ok(), `facture : ${res.status()} ${await res.text()}`).toBeTruthy();
	const draft = (await res.json()) as { id: number; version: number };
	if (!opts.validate) return draft;
	const val = await ctx.post(`/api/v1/supplier-invoices/${draft.id}/validate`, {
		data: { version: draft.version },
	});
	expect(val.ok(), `validation : ${val.status()} ${await val.text()}`).toBeTruthy();
	return (await val.json()) as { id: number; version: number };
}

test.describe('0.17.4 — les écrans des correctifs', () => {
	test('#888 — le formulaire d’un lot propose toutes les factures validées (AC-3.4)', async ({
		page,
	}, testInfo) => {
		test.setTimeout(240_000);
		await login(page);
		const ctx = await authedApiContext(page);
		try {
			const expense = await expenseAccount(ctx);
			const contactId = await supplier(ctx, uniq('Fournisseur lot 888'));
			for (let i = 0; i < 205; i++) {
				await supplierInvoice(ctx, {
					contactId,
					expenseAccountId: expense,
					invoiceDate: daysFromToday(-5),
					validate: true,
				});
			}
			const open = await (
				await ctx.get('/api/v1/supplier-invoices?status=open&limit=1')
			).json();
			expect(open.total).toBeGreaterThanOrEqual(205);

			await page.goto('/payment-batches');
			await page.getByTestId('payment-batch-new').click();
			await expect(page.getByTestId('payment-batch-invoice-checkbox')).toHaveCount(open.total);
			await capture(page, testInfo, '888-lot-toutes-les-factures');
		} finally {
			await disposeContextSafe(ctx);
		}
	});

	test('#869 — le refus du repassage en brouillon nomme la date du verrou (AC-3.13)', async ({
		page,
	}, testInfo) => {
		await login(page);
		const ctx = await authedApiContext(page);
		const lockedThrough = daysFromToday(-10);
		try {
			const expense = await expenseAccount(ctx);
			const contactId = await supplier(ctx, uniq('Fournisseur verrou 869'));
			const inv = await supplierInvoice(ctx, {
				contactId,
				expenseAccountId: expense,
				invoiceDate: daysFromToday(-20),
				validate: true,
			});
			const lock = await ctx.post('/api/v1/companies/current/books-lock', {
				data: { through: lockedThrough },
			});
			expect(lock.ok(), `verrou : ${lock.status()} ${await lock.text()}`).toBeTruthy();

			await page.goto(`/supplier-invoices/${inv.id}`);
			await page.getByTestId('supplier-invoice-unvalidate').click();
			await page.getByTestId('supplier-invoice-unvalidate-confirm').click();
			const [y, m, d] = lockedThrough.split('-');
			await expect(page.getByTestId('supplier-invoice-unvalidate-error')).toContainText(
				`${d}.${m}.${y}`,
			);
			await capture(page, testInfo, '869-refus-verrou-nomme-la-date');
		} finally {
			// Le verrou levé : les tests suivants du fichier n'en dépendent pas.
			await ctx.post('/api/v1/companies/current/books-lock/release', {
				data: { through: null, motif: 'Fin du test E2E #869' },
			});
			await disposeContextSafe(ctx);
		}
	});

	test('#872 — un brouillon montre son projet archivé sous une racine archivée', async ({
		page,
	}, testInfo) => {
		await login(page);
		const ctx = await authedApiContext(page);
		try {
			const expense = await expenseAccount(ctx);
			const code = `R${Date.now() % 100000}`;
			const root = await (
				await ctx.post('/api/v1/projects', { data: { code, name: 'Racine 872', parentId: null } })
			).json();
			const child = await (
				await ctx.post('/api/v1/projects', {
					data: { code: `${code}-E`, name: 'Enfant 872', parentId: root.id },
				})
			).json();
			const contactId = await supplier(ctx, uniq('Fournisseur projet 872'));
			const draft = await supplierInvoice(ctx, {
				contactId,
				expenseAccountId: expense,
				invoiceDate: daysFromToday(-3),
				projectId: child.id,
				validate: false,
			});
			const a1 = await ctx.post(`/api/v1/projects/${child.id}/archive`, {
				data: { version: child.version },
			});
			expect(a1.ok(), `archivage de l’enfant : ${a1.status()}`).toBeTruthy();
			const a2 = await ctx.post(`/api/v1/projects/${root.id}/archive`, {
				data: { version: root.version },
			});
			expect(a2.ok(), `archivage de la racine : ${a2.status()}`).toBeTruthy();

			await page.goto(`/supplier-invoices/${draft.id}/edit`);
			const select = page.getByTestId('supplier-invoice-project');
			await expect(select).toHaveValue(String(child.id));
			await expect(select.locator(`option[value="${child.id}"]`)).toContainText('(archivé)');
			await capture(page, testInfo, '872-projet-archive-sous-racine-archivee');
		} finally {
			await disposeContextSafe(ctx);
		}
	});

	test('#817 — le pays « XX » est refusé, à l’écran des contacts et au panneau fournisseur', async ({
		page,
	}, testInfo) => {
		await login(page);
		await page.goto('/contacts');
		await page.getByRole('button', { name: /Nouveau contact/i }).click();
		await page.fill('#form-name', uniq('Contact pays XX'));
		await page.fill('#form-street', 'Rue des Essais');
		await page.fill('#form-npa', '1200');
		await page.fill('#form-city', 'Genève');
		await page.fill('#form-country', 'XX');
		await page.locator('form button[type="submit"]').click();
		await expect(page.getByText('« XX »').first()).toBeVisible();
		await capture(page, testInfo, '817-contacts-refus-xx');

		await page.goto('/supplier-invoices/new');
		await page.getByTestId('supplier-new-open').click();
		const panel = page.getByTestId('supplier-new-panel');
		await panel.getByTestId('supplier-new-name').fill(uniq('Fournisseur pays XX'));
		await panel.getByTestId('supplier-new-street').fill('Rue des Essais');
		await panel.getByTestId('supplier-new-postal-code').fill('1200');
		await panel.getByTestId('supplier-new-city').fill('Genève');
		await panel.getByLabel('Pays').fill('XX');
		await panel.getByTestId('supplier-new-create').click();
		await expect(panel.getByTestId('supplier-new-error')).toContainText('« XX »');
		await capture(page, testInfo, '817-panneau-fournisseur-refus-xx');
	});

	test('#817 — un contact au pays inconnu se signale à l’édition', async ({ page }, testInfo) => {
		await login(page);
		// Un contact enregistré avant la 0.17.4 avec « SU » ne se crée plus par l'API : le
		// décor crée un contact « CH », puis la réponse du serveur est réécrite comme pour un
		// tel contact (`addressCountryKnown: false`, jugé par le serveur, AC-2.6) — l'écran
		// n'a que ce champ pour le signaler.
		const ctx = await authedApiContext(page);
		const name = uniq('Ancien contact SU');
		try {
			const res = await ctx.post('/api/v1/contacts', {
				data: {
					contactType: 'Entreprise',
					name,
					isClient: true,
					addressStructured: {
						street: 'Rue des Essais',
						building: '7',
						postalCode: '1200',
						city: 'Genève',
						country: 'CH',
					},
				},
			});
			expect(res.ok(), `contact : ${res.status()}`).toBeTruthy();
		} finally {
			await disposeContextSafe(ctx);
		}
		await page.route('**/api/v1/contacts?**', async (route) => {
			const response = await route.fetch();
			const body = await response.json();
			for (const c of body.items ?? []) {
				if (c.name === name) {
					c.addressStructured.country = 'SU';
					c.addressCountryKnown = false;
				}
			}
			await route.fulfill({ response, json: body });
		});
		await page.goto(`/contacts?search=${encodeURIComponent(name)}`);
		await page.getByRole('button', { name: 'Modifier' }).first().click();
		await expect(page.getByTestId('contact-country-unknown')).toContainText('« SU »');
		await capture(page, testInfo, '817-contact-pays-inconnu-avis');
	});
});
