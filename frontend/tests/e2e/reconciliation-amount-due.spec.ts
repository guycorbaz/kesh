/**
 * Story 25-4-c (#420) — le rapprochement propose le SOLDE d'une facture déjà
 * réglée en partie, de bout en bout.
 *
 * Une facture validée (900.— HT @ 8.1 % → 972.90 TTC), un règlement partiel,
 * puis un relevé dont la transaction porte exactement le reste dû. La page de
 * réconciliation doit proposer la facture, afficher le reste — et le TTC en
 * mention. Avant la correction, la facture n'était même pas candidate : le
 * filtre comparait le TTC.
 *
 * ⚠️ **Des montants UNIQUES à chaque exécution** : la base est partagée entre
 * specs. Le règlement partiel varie, donc le reste aussi ; le relevé est
 * réécrit (références, montant) comme dans `reconciliation-cancel.spec.ts`.
 *
 * Pré-requis : MariaDB up + KESH_TEST_MODE=true (cf. docs/testing.md).
 */

import { expect, test, type Page } from '@playwright/test';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';
import {
	seedTestState,
	clearAuthStorage,
	authedApiContext,
	disposeContextSafe,
} from './helpers/test-state';
import {
	createContactWithAddressViaApi,
	createAndValidateInvoiceViaApi,
} from './helpers/api-fixtures';

const FIXTURE = path.join(
	path.dirname(fileURLToPath(import.meta.url)),
	'fixtures',
	'camt053_v04_minimal.xml',
);
const TEST_IBAN = 'CH4431999123000889012';
/** TTC de la facture de `createAndValidateInvoiceViaApi` : 4.5 × 200.— @ 8.1 %. */
const TTC_CENTS = 97290;

test.beforeAll(async () => {
	await seedTestState('with-company');
});

test.afterEach(async ({ page }) => {
	await clearAuthStorage(page);
});

async function login(page: Page): Promise<void> {
	await page.goto('/login');
	await page.fill('#username', 'admin');
	await page.fill('#password', 'admin123');
	await page.click('button[type="submit"]');
	await expect(page).toHaveURL('/');
}

/** Un montant en centimes, au format du backend (`Decimal::normalize`). */
function normalized(cents: number): string {
	return String(cents / 100);
}

/**
 * Le décor : une facture réglée en partie, et un relevé importé dont la
 * transaction porte son reste. Rend `(bankAccountId, invoiceId, resteCents)`.
 */
async function partiallySettledInvoiceAndItsBalance(
	page: Page,
): Promise<{ bankAccountId: number; invoiceId: number; dueCents: number }> {
	const unique = `${Date.now()}`;
	const contactId = await createContactWithAddressViaApi(page, `Solde ${unique}`);
	// Date de facture dans la fenêtre de 30 jours autour du relevé (2026-05-15).
	const invoiceId = await createAndValidateInvoiceViaApi(page, contactId, '2026-05-10');

	const ctx = await authedApiContext(page);
	try {
		// Règlement partiel, de 100.00 à 499.99 — le reste est donc unique.
		const partialCents = 10000 + (Number(unique.slice(-6)) % 40000);
		const accounts = (await (await ctx.get('/api/v1/accounts?includeArchived=false')).json()) as {
			id: number;
			number: string;
			accountType: string;
			active: boolean;
			postable: boolean;
		}[];
		const cash = accounts.find(
			(a) => a.active && a.postable && a.accountType === 'Asset' && a.number.startsWith('10'),
		);
		expect(cash, 'un compte de liquidités imputable').toBeTruthy();
		const settled = await ctx.post(`/api/v1/invoices/${invoiceId}/settlements`, {
			data: {
				settlementType: 'internal_account',
				accountId: cash!.id,
				amount: (partialCents / 100).toFixed(2),
				settledOn: '2026-05-12',
			},
		});
		expect(settled.ok(), `règlement partiel: ${settled.status()} ${await settled.text()}`).toBeTruthy();
		const dueCents = TTC_CENTS - partialCents;

		type Ba = { id: number; iban: string };
		const findAccount = async (): Promise<Ba | undefined> => {
			const list = await ctx.get('/api/v1/bank-accounts');
			expect(list.ok()).toBeTruthy();
			return ((await list.json()) as Ba[]).find((b) => b.iban === TEST_IBAN);
		};
		let ba = await findAccount();
		if (!ba) {
			const res = await ctx.post('/api/v1/bank-accounts', {
				data: { bankName: 'UBS Test', iban: TEST_IBAN, qrIban: null, isPrimary: true },
			});
			expect(res.ok(), `bank-account create: ${res.status()}`).toBeTruthy();
			ba = await findAccount();
		}
		const bankAccountId = ba!.id;

		const xml = fs
			.readFileSync(FIXTURE, 'utf8')
			.replace('STMT-2026-05-001', `STMT-SOLDE-${unique}`)
			.replace('BANK-TX-42', `BANK-TX-SOLDE-${unique}`)
			.replace('E2E-2026-05-001', `E2E-SOLDE-${unique}`)
			.replace('<Amt Ccy="CHF">1234.56</Amt>', `<Amt Ccy="CHF">${(dueCents / 100).toFixed(2)}</Amt>`);
		const imported = await ctx.post('/api/v1/bank-imports', {
			multipart: {
				bankAccountId: bankAccountId.toString(),
				file: {
					name: `solde-${unique}.xml`,
					mimeType: 'application/xml',
					buffer: Buffer.from(xml, 'utf8'),
				},
				confirmBalanceMismatch: 'true',
				confirmDuplicateFile: 'true',
			},
		});
		expect(imported.status(), await imported.text()).toBe(201);
		return { bankAccountId, invoiceId, dueCents };
	} finally {
		await disposeContextSafe(ctx);
	}
}

test('le solde d’une facture réglée en partie est proposé, avec son reste', async ({ page }) => {
	await login(page);
	const { bankAccountId, invoiceId, dueCents } = await partiallySettledInvoiceAndItsBalance(page);

	await page.goto('/reconciliation');
	await page.getByTestId('bank-account-select').selectOption(bankAccountId.toString());

	const candidate = page.locator(
		`[data-testid="candidate-top-1"][data-invoice-id="${invoiceId}"]`,
	);
	await expect(candidate).toBeVisible({ timeout: 10000 });
	await expect(candidate).toContainText(`(${normalized(dueCents)},`);
	await expect(candidate.getByTestId('candidate-top-1-amount-due-of')).toContainText(
		normalized(TTC_CENTS),
	);
});
