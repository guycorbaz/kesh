/**
 * Story 25-3-b (#418) — annuler un rapprochement, de bout en bout.
 *
 * Importer un relevé, rapprocher sa transaction (rapprochement manuel, par
 * l'API — le décor), puis l'ANNULER par l'interface, depuis le détail de
 * l'import : le seul écran qui montre une transaction rapprochée. La
 * transaction revient « à rapprocher », et dans les propositions.
 *
 * ⚠️ **Un relevé UNIQUE à chaque exécution** : la fixture minimale est
 * réimportée par d'autres specs sur la base partagée ; ses références et son
 * montant sont réécrits pour que ce scénario ne dépende ni de l'ordre des
 * specs, ni d'un doublon.
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

const FIXTURE = path.join(
	path.dirname(fileURLToPath(import.meta.url)),
	'fixtures',
	'camt053_v04_minimal.xml',
);
const TEST_IBAN = 'CH4431999123000889012';

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

/**
 * Le décor : un compte bancaire câblé sur un compte du grand livre, un import
 * unique, sa transaction rapprochée manuellement. Rend `(importId, txId)`.
 */
async function reconciledTransaction(page: Page): Promise<{ importId: number; txId: number }> {
	const ctx = await authedApiContext(page);
	try {
		const accounts = (await (await ctx.get('/api/v1/accounts')).json()) as Array<{
			id: number;
			number: string;
		}>;
		const ledger = accounts.find((a) => a.number === '1000');
		const revenue = accounts.find((a) => a.number === '3000');
		expect(ledger && revenue, 'comptes 1000 et 3000 du préréglage').toBeTruthy();

		type Ba = { id: number; iban: string; version: number };
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
		// Le rapprochement manuel exige un compte bancaire câblé sur le grand livre.
		const wired = await ctx.patch(`/api/v1/bank-accounts/${bankAccountId}`, {
			data: { journalAccountId: ledger!.id, version: ba!.version },
		});
		expect(wired.ok(), `bank-account wiring: ${wired.status()}`).toBeTruthy();

		const unique = `${Date.now()}`;
		const amount = `${100 + (Number(unique.slice(-5)) % 800)}.${unique.slice(-2)}`;
		const xml = fs
			.readFileSync(FIXTURE, 'utf8')
			.replace('STMT-2026-05-001', `STMT-CANCEL-${unique}`)
			.replace('BANK-TX-42', `BANK-TX-CANCEL-${unique}`)
			.replace('E2E-2026-05-001', `E2E-CANCEL-${unique}`)
			.replace('<Amt Ccy="CHF">1234.56</Amt>', `<Amt Ccy="CHF">${amount}</Amt>`);
		const imported = await ctx.post('/api/v1/bank-imports', {
			multipart: {
				bankAccountId: bankAccountId.toString(),
				file: {
					name: `cancel-${unique}.xml`,
					mimeType: 'application/xml',
					buffer: Buffer.from(xml, 'utf8'),
				},
				confirmBalanceMismatch: 'true',
				confirmDuplicateFile: 'true',
			},
		});
		expect(imported.status(), await imported.text()).toBe(201);
		const importId = ((await imported.json()) as { id: number }).id;

		const detail = (await (await ctx.get(`/api/v1/bank-imports/${importId}`)).json()) as {
			transactions: Array<{ id: number }>;
		};
		const txId = detail.transactions[0].id;

		const matched = await ctx.post('/api/v1/reconciliation/manual', {
			data: {
				bankAccountId,
				bankTransactionId: txId,
				counterpartyAccountId: revenue!.id,
				description: 'Recette E2E',
			},
		});
		expect(matched.status(), await matched.text()).toBe(200);
		return { importId, txId };
	} finally {
		await disposeContextSafe(ctx);
	}
}

test('rapprocher puis annuler le rapprochement depuis le détail de l’import', async ({ page }) => {
	await login(page);
	const { importId, txId } = await reconciledTransaction(page);

	await page.goto(`/bank-import/${importId}`);
	const row = page.locator(`[data-testid="detail-tx-row"][data-tx-id="${txId}"]`);
	await expect(row).toContainText('reconciled');
	await expect(row.getByTestId('detail-tx-entry-link')).toBeVisible();

	await row.getByTestId('detail-tx-cancel-reconciliation').click();
	const dialog = page.getByTestId('reconciliation-cancel-dialog');
	await expect(dialog.getByTestId('reconciliation-cancel-confirm-text')).toContainText(
		'écriture inverse',
	);
	await dialog.getByTestId('reconciliation-cancel-submit').click();

	// Le détail est relu : la transaction est de nouveau « à rapprocher ».
	await expect(dialog).toBeHidden();
	await expect(row).toContainText('pending');
	await expect(row.getByTestId('detail-tx-cancel-reconciliation')).toHaveCount(0);

	// ⛔ Et elle revient dans les propositions — c'est ce que l'écran de
	// réconciliation lit ; sans cela, elle serait perdue pour l'utilisateur.
	const ctx = await authedApiContext(page);
	try {
		const tx = (await (await ctx.get(`/api/v1/reconciliation/transactions/${txId}`)).json()) as {
			status: string;
			matchedEntryId: number | null;
		};
		expect(tx.status).toBe('pending');
		expect(tx.matchedEntryId).toBeNull();
	} finally {
		await disposeContextSafe(ctx);
	}
});
