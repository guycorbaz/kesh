/**
 * Story 28-5b (#526) — une transaction rejetée se retrouve dans le détail de son import, et son
 * rejet s'annule depuis l'écran.
 *
 * Avant la story, *Rejeter* faisait quitter à la transaction la liste de la réconciliation, et
 * aucun geste de l'interface ne l'y remettait : le détail de l'import la montrait en `pending`,
 * sans aucune action. C'est le seul test qui voit `autoMatchRejectedAt` traverser la frontière
 * HTTP jusqu'à la mention « Rejetée le », et le `204` de `…/unreject` jusqu'au retour dans la
 * liste (`CLAUDE.md`, § *E2E*).
 *
 * ⚠️ **Relevé UNIQUE à chaque exécution** (patron de `reconciliation-deleted-import.spec.ts`) :
 * références, montant et contrepartie réécrits ; l'import et l'utilisateur Consultation sont
 * retirés en `finally` — l'import supprimé, l'utilisateur **désactivé** (Kesh n'a pas de route de
 * suppression d'utilisateur, C-28-5-36).
 *
 * # Recette visuelle (action B2 de la rétrospective de l'Epic 27)
 *
 * Le scénario 1 tourne une fois par schéma de couleurs (`page.emulateMedia`) et capture le
 * détail avec la rejetée, le détail après l'annulation et l'indication de la page
 * *Réconciliation* — six captures —, dans `KESH_E2E_CAPTURES_DIR` s'il est posé, sinon dans le
 * dossier de sortie du test. Hors dépôt, sans `toHaveScreenshot` : aucune référence de comparaison.
 *
 * Pré-requis : MariaDB up + KESH_TEST_MODE=true (cf. docs/testing.md).
 */

import { expect, test, type APIRequestContext, type Page, type TestInfo } from '@playwright/test';
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

async function login(page: Page, username = 'admin', password = 'admin123'): Promise<void> {
	await page.goto('/login');
	await page.fill('#username', username);
	await page.fill('#password', password);
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

/** Importe un relevé d'une transaction, réécrit et unique, puis la REJETTE par l'API. */
async function importAndReject(
	ctx: APIRequestContext,
	baId: number,
	tag: string,
): Promise<{ importId: number; txId: number }> {
	const unique = `${Date.now()}-${tag}`;
	const cents = 10000 + (Date.now() % 40000);
	let xml = fs.readFileSync(FIXTURE, 'utf8');
	for (const [from, to] of [
		['STMT-2026-05-001', `STMT-UNREJ-${unique}`],
		['BANK-TX-42', `BANK-TX-UNREJ-${unique}`],
		['E2E-2026-05-001', `E2E-UNREJ-${unique}`],
		['<Amt Ccy="CHF">1234.56</Amt>', `<Amt Ccy="CHF">${(cents / 100).toFixed(2)}</Amt>`],
		['<Nm>Acme SA</Nm>', `<Nm>Rejet annulé ${unique}</Nm>`],
	]) {
		expect(xml.includes(from), `fixture : « ${from} » introuvable`).toBeTruthy();
		xml = xml.replace(from, to);
	}
	const imported = await ctx.post('/api/v1/bank-imports', {
		multipart: {
			bankAccountId: baId.toString(),
			file: {
				name: `unreject-${unique}.xml`,
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
	const rejected = await ctx.post('/api/v1/reconciliation/reject', {
		data: { bankAccountId: baId, bankTransactionIds: [txId] },
	});
	expect(rejected.status(), await rejected.text()).toBe(200);
	const body = (await rejected.json()) as { rejected: Array<{ bankTransactionId: number }> };
	expect(body.rejected.map((r) => r.bankTransactionId)).toEqual([txId]);
	return { importId, txId };
}

/** Le dossier des captures : `KESH_E2E_CAPTURES_DIR`, sinon la sortie du test. */
function capturePath(testInfo: TestInfo, name: string): string {
	const dossier = process.env.KESH_E2E_CAPTURES_DIR;
	return dossier ? path.join(dossier, `${name}.png`) : testInfo.outputPath(`${name}.png`);
}

for (const [colorScheme, suffixe] of [
	['light', 'clair'],
	['dark', 'sombre'],
] as const) {
	test(`scénario 1 (${suffixe}) — la rejetée porte sa mention, son rejet s’annule, elle revient dans la liste`, async ({
		page,
	}, testInfo) => {
		await page.emulateMedia({ colorScheme });
		await login(page);
		const ctx = await authedApiContext(page);
		let importId: number | null = null;
		try {
			const baId = await bankAccountId(ctx);
			const r = await importAndReject(ctx, baId, suffixe);
			importId = r.importId;

			await page.goto(`/bank-import/${r.importId}`);
			const ligne = page.locator(`[data-testid="detail-tx-row"][data-tx-id="${r.txId}"]`);
			await expect(ligne.getByTestId('detail-tx-rejected-on')).toContainText('Rejetée le');
			const bouton = ligne.getByTestId('detail-tx-unreject');
			await expect(bouton).toHaveText('Annuler le rejet');
			await page.getByTestId('detail-tx-table').screenshot({
				path: capturePath(testInfo, `detail-rejetee-${suffixe}`),
			});

			await bouton.click();
			// KF-043 (#326) : le toast se cible par son type, sans filtre de libellé.
			await expect(page.locator('[data-sonner-toast][data-type="success"]')).toBeVisible();
			// Assertion décisive : la page a relu son détail, la mention a disparu.
			await expect(ligne.getByTestId('detail-tx-rejected-on')).toHaveCount(0);
			await expect(ligne.getByTestId('detail-tx-unreject')).toHaveCount(0);
			await page.getByTestId('detail-tx-table').screenshot({
				path: capturePath(testInfo, `detail-apres-annulation-${suffixe}`),
			});

			// La transaction est de nouveau proposée.
			await page.goto('/reconciliation');
			await page.getByTestId('bank-account-select').selectOption(baId.toString());
			await expect(
				page.locator(`[data-testid="reconciliation-row"][data-tx-id="${r.txId}"]`),
			).toBeVisible({ timeout: 10000 });
			const hint = page.getByTestId('reconciliation-rejected-hint');
			await expect(hint).toBeVisible();
			await hint.screenshot({ path: capturePath(testInfo, `reconciliation-indication-${suffixe}`) });
		} finally {
			if (importId !== null) await ctx.delete(`/api/v1/bank-imports/${importId}`);
			await disposeContextSafe(ctx);
		}
	});
}

test('scénario 2 — rôle Consultation : la mention est visible, le bouton absent', async ({ page }) => {
	await login(page);
	const ctx = await authedApiContext(page);
	const username = `consult-unrej-${Date.now()}`;
	let importId: number | null = null;
	let userId: number | null = null;
	try {
		const baId = await bankAccountId(ctx);
		const r = await importAndReject(ctx, baId, 'consultation');
		importId = r.importId;
		const created = await ctx.post('/api/v1/users', {
			data: { username, password: 'MotDePasse12345', role: 'Consultation' },
		});
		expect(created.ok(), `create user failed: ${created.status()}`).toBeTruthy();
		userId = ((await created.json()) as { id: number }).id;

		await clearAuthStorage(page);
		await login(page, username, 'MotDePasse12345');
		await page.goto(`/bank-import/${r.importId}`);
		const ligne = page.locator(`[data-testid="detail-tx-row"][data-tx-id="${r.txId}"]`);
		await expect(ligne.getByTestId('detail-tx-rejected-on')).toContainText('Rejetée le');
		await expect(ligne.getByTestId('detail-tx-unreject')).toHaveCount(0);

		// T12 — observation, sans correction (« Exclu » de la fiche) : la page *Réconciliation* sous
		// Consultation. L'indication est hors de toute branche : elle doit s'afficher même si la
		// liste est refusée à ce rôle (C-28-5-22). Ce que la page affiche d'autre est journalisé.
		await page.goto('/reconciliation');
		const select = page.getByTestId('bank-account-select');
		if (await select.isVisible()) await select.selectOption(baId.toString());
		await expect(page.getByTestId('reconciliation-rejected-hint')).toBeVisible();
		const erreurs = await page.getByTestId('reconciliation-error').allTextContents();
		console.log(`[28-5b T12] Consultation sur /reconciliation — erreur affichée : ${JSON.stringify(erreurs)}`);
	} finally {
		// Le contexte d'API reste celui de l'administrateur, ouvert avant le changement de session.
		if (importId !== null) await ctx.delete(`/api/v1/bank-imports/${importId}`);
		if (userId !== null) await ctx.put(`/api/v1/users/${userId}/disable`);
		await disposeContextSafe(ctx);
	}
});

test('scénario 3 — la page Réconciliation dit où retrouver une transaction rejetée', async ({ page }) => {
	await login(page);
	const ctx = await authedApiContext(page);
	try {
		const baId = await bankAccountId(ctx);
		await page.goto('/reconciliation');
		await page.getByTestId('bank-account-select').selectOption(baId.toString());
		await expect(page.getByTestId('reconciliation-rejected-hint')).toContainText(
			'Elle reste dans le détail de son import (Importer des relevés)',
		);
	} finally {
		await disposeContextSafe(ctx);
	}
});
