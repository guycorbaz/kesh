/**
 * Story 27-4 (#634) — un lot de réconciliation qui contient une transaction disparue est traité
 * proposition par proposition, de bout en bout.
 *
 * La page *Réconciliation* est ouverte, deux transactions cochées ; **puis** l'import de l'une
 * est supprimé (par l'API, comme depuis un autre onglet) ; *Rejeter* : l'autre est rejetée, la
 * transaction supprimée sort dans les *Échecs partiels* avec son motif, et aucun bandeau
 * d'erreur ne s'affiche. C'est le seul test qui voit le `200` traverser la frontière HTTP
 * jusqu'à l'écran (`CLAUDE.md`, § *E2E*) — avant la story, le lot entier était refusé en `400`.
 *
 * ⚠️ **Relevés UNIQUES à chaque exécution** (patron de `reconciliation-cancel.spec.ts`) :
 * références, montants et contrepartie réécrits. La **règle d'affectation** porte une
 * contrepartie unique à l'exécution ; elle rendait les deux transactions sélectionnables, et
 * n'est plus nécessaire à la sélection depuis la Story 29-6 (toute ligne porte une case) — elle
 * est gardée, le spec éprouvant toujours un rejet en lot dont une transaction a disparu. Le
 * ciblage par `data-tx-id` suffit à ne cocher aucune transaction d'un autre spec. La règle est
 * supprimée en `finally`, avec les deux imports (choix C-27-4-24, C-27-4-41, C-29-6-19).
 *
 * Pré-requis : MariaDB up + KESH_TEST_MODE=true (cf. docs/testing.md).
 */

import { expect, test, type APIRequestContext, type Page } from '@playwright/test';
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

async function bankAccountId(ctx: APIRequestContext): Promise<number> {
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
	return ba!.id;
}

/** Importe un relevé d'une transaction, réécrit et unique ; rend `(importId, txId)`. */
async function importOne(
	ctx: APIRequestContext,
	baId: number,
	unique: string,
	tag: string,
	cents: number,
	counterparty: string,
): Promise<{ importId: number; txId: number }> {
	let xml = fs.readFileSync(FIXTURE, 'utf8');
	for (const [from, to] of [
		['STMT-2026-05-001', `STMT-DEL-${tag}-${unique}`],
		['BANK-TX-42', `BANK-TX-DEL-${tag}-${unique}`],
		['E2E-2026-05-001', `E2E-DEL-${tag}-${unique}`],
		['<Amt Ccy="CHF">1234.56</Amt>', `<Amt Ccy="CHF">${(cents / 100).toFixed(2)}</Amt>`],
		['<Nm>Acme SA</Nm>', `<Nm>${counterparty}</Nm>`],
	]) {
		expect(xml.includes(from), `fixture : « ${from} » introuvable`).toBeTruthy();
		xml = xml.replace(from, to);
	}
	const imported = await ctx.post('/api/v1/bank-imports', {
		multipart: {
			bankAccountId: baId.toString(),
			file: {
				name: `deleted-${tag}-${unique}.xml`,
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
	return { importId, txId: detail.transactions[0].id };
}

test('un lot de rejet dont un import a été supprimé entre-temps traite l’autre transaction', async ({
	page,
}) => {
	await login(page);
	const unique = `${Date.now()}`;
	const counterparty = `Import supprimé ${unique}`;
	const base = 10000 + (Number(unique.slice(-5)) % 40000);

	const ctx = await authedApiContext(page);
	let ruleId: number | null = null;
	// Les deux imports sont suivis pour le `finally` (revue de code P1, E4 / B LOW-1) : un échec
	// entre l'import de A et sa suppression ne laisse rien en base ; un 404 y est sans effet.
	let importA: number | null = null;
	let importB: number | null = null;
	try {
		const baId = await bankAccountId(ctx);
		const accounts = (await (await ctx.get('/api/v1/accounts')).json()) as Array<{
			id: number;
			number: string;
		}>;
		const revenue = accounts.find((a) => a.number === '3000');
		expect(revenue, 'compte 3000 du préréglage').toBeTruthy();

		const a = await importOne(ctx, baId, unique, 'A', base, counterparty);
		importA = a.importId;
		const b = await importOne(ctx, baId, unique, 'B', base + 1, counterparty);
		importB = b.importId;

		const rule = await ctx.post('/api/v1/reconciliation/rules', {
			data: {
				label: `E2E 27-4 ${unique}`,
				matchType: 'counterparty_contains',
				matchValue: counterparty,
				counterpartyAccountId: revenue!.id,
			},
		});
		expect(rule.status(), await rule.text()).toBe(201);
		ruleId = ((await rule.json()) as { id: number }).id;

		await page.goto('/reconciliation');
		await page.getByTestId('bank-account-select').selectOption(baId.toString());
		const boxA = page.locator(`[data-testid="tx-checkbox"][data-tx-id="${a.txId}"]`);
		const boxB = page.locator(`[data-testid="tx-checkbox"][data-tx-id="${b.txId}"]`);
		await expect(boxA).toBeVisible({ timeout: 10000 });
		await expect(boxB).toBeVisible();
		await boxA.check();
		await boxB.check();

		// La page est restée ouverte : l'import A est supprimé « ailleurs ».
		const deleted = await ctx.delete(`/api/v1/bank-imports/${a.importId}`);
		expect(deleted.status(), await deleted.text()).toBe(204);

		await page.getByTestId('reconciliation-reject-btn').click();

		// Exactement une opération réussie : « 1 opération(s)… », et non « 10 » ni « 11 ».
		await expect(page.getByTestId('reconciliation-success')).toHaveText(/^\s*1\s+\D/);
		const item = page.locator(
			`[data-testid="reconciliation-failed-item"][data-tx-id="${a.txId}"]`,
		);
		await expect(item).toHaveAttribute('data-error-code', 'BANK_TRANSACTION_NOT_FOUND');
		await expect(item).toContainText('Transaction introuvable');
		await expect(item).toContainText('supprimée avec son import');
		await expect(page.getByTestId('reconciliation-error')).toHaveCount(0);

		// La liste, rechargée après le lot, ne propose plus ni B (rejetée) ni A (supprimée) :
		// il n'y a rien à refaire.
		for (const txId of [a.txId, b.txId]) {
			await expect(
				page.locator(`[data-testid="reconciliation-row"][data-tx-id="${txId}"]`),
			).toHaveCount(0);
		}
		// Côté serveur : B existe encore, en attente (un rejet ne rapproche pas).
		const txB = await ctx.get(`/api/v1/reconciliation/transactions/${b.txId}`);
		expect(txB.status()).toBe(200);
		expect(((await txB.json()) as { status: string }).status).toBe('pending');
	} finally {
		if (ruleId !== null) await ctx.delete(`/api/v1/reconciliation/rules/${ruleId}`);
		for (const id of [importA, importB]) {
			if (id !== null) await ctx.delete(`/api/v1/bank-imports/${id}`);
		}
		await disposeContextSafe(ctx);
	}
});
