/**
 * Story 29-6 (#665, AC10) — une transaction sans proposition se coche et se rejette depuis la
 * page *Réconciliation*.
 *
 * Avant la story, une ligne « Aucune correspondance » n'avait pas de case : *Rejeter* n'agissant
 * que sur les lignes cochées, aucun geste de l'écran ne l'écartait — alors que le manuel prescrit
 * de rejeter la transaction d'un client qui a payé quelques centimes de trop, une fois le
 * règlement enregistré depuis la fiche de la facture. C'est le seul test qui voit l'identifiant
 * d'une ligne sans proposition traverser la frontière HTTP jusqu'à `…/reject` (`CLAUDE.md`,
 * § *E2E*).
 *
 * Parcours : la ligne porte une case ; cochée, *Accepter* est grisé et la rangée des boutons dit
 * pourquoi ; *Rejeter* → « 1 opération(s) réussie(s). », la ligne quitte la liste ; au détail de
 * l'import, « Rejetée le » ; *Annuler le rejet* → la ligne revient dans la liste.
 *
 * ⚠️ **Relevé UNIQUE à chaque exécution** (patron de `reconciliation-unreject.spec.ts`) :
 * références, montant et contrepartie réécrits, l'import supprimé en `finally`. Le montant est
 * tiré **au-dessus de 90 000 CHF**, hors de la plage des factures que montent les autres specs de
 * la base partagée (au plus quelques milliers, relevé au développement) : aucune facture n'en est
 * candidate. Même réserve que le patron sur la fenêtre des 100 transactions les plus récentes.
 *
 * ⚠️ **Sélecteurs par `data-testid`** : la garde `e2e-selecteurs-traduits.test.ts` (KF-043, #326)
 * refuse tout sélecteur neuf figé sur un libellé français ; les textes se vérifient par
 * `toContainText` sur un élément ciblé.
 *
 * # Recette visuelle
 *
 * Une fois par schéma de couleurs (`page.emulateMedia`), deux captures — la ligne cochée avec
 * l'indication, le bilan après le rejet — dans `KESH_E2E_CAPTURES_DIR` s'il est posé, sinon dans
 * le dossier de sortie du test. Hors dépôt, sans `toHaveScreenshot`.
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

/** Importe un relevé d'une transaction, réécrit et unique, qu'aucune facture ni règle ne rapproche. */
async function importUnmatched(
	ctx: APIRequestContext,
	baId: number,
	tag: string,
): Promise<{ importId: number; txId: number }> {
	const unique = `${Date.now()}-${tag}`;
	const cents = 9_000_000 + (Date.now() % 900_000);
	let xml = fs.readFileSync(FIXTURE, 'utf8');
	for (const [from, to] of [
		['STMT-2026-05-001', `STMT-REJ296-${unique}`],
		['BANK-TX-42', `BANK-TX-REJ296-${unique}`],
		['E2E-2026-05-001', `E2E-REJ296-${unique}`],
		['<Amt Ccy="CHF">1234.56</Amt>', `<Amt Ccy="CHF">${(cents / 100).toFixed(2)}</Amt>`],
		['<Nm>Acme SA</Nm>', `<Nm>Sans correspondance ${unique}</Nm>`],
	]) {
		expect(xml.includes(from), `fixture : « ${from} » introuvable`).toBeTruthy();
		xml = xml.replace(from, to);
	}
	const imported = await ctx.post('/api/v1/bank-imports', {
		multipart: {
			bankAccountId: baId.toString(),
			file: {
				name: `reject-unmatched-${unique}.xml`,
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

/** Le dossier des captures : `KESH_E2E_CAPTURES_DIR`, sinon la sortie du test. */
function capturePath(testInfo: TestInfo, name: string): string {
	const dossier = process.env.KESH_E2E_CAPTURES_DIR;
	return dossier ? path.join(dossier, `${name}.png`) : testInfo.outputPath(`${name}.png`);
}

for (const [colorScheme, suffixe] of [
	['light', 'clair'],
	['dark', 'sombre'],
] as const) {
	test(`(${suffixe}) — une ligne « Aucune correspondance » se coche, se rejette, et son rejet s’annule`, async ({
		page,
	}, testInfo) => {
		await page.emulateMedia({ colorScheme });
		await login(page);
		const ctx = await authedApiContext(page);
		let importId: number | null = null;
		try {
			const baId = await bankAccountId(ctx);
			const r = await importUnmatched(ctx, baId, suffixe);
			importId = r.importId;

			await page.goto('/reconciliation');
			await page.getByTestId('bank-account-select').selectOption(baId.toString());
			const ligne = page.locator(`[data-testid="reconciliation-row"][data-tx-id="${r.txId}"]`);
			await expect(ligne).toBeVisible({ timeout: 10000 });
			await expect(ligne).toContainText('Aucune correspondance');

			// La ligne sans proposition porte une case (avant la story : aucune).
			const caseLigne = page.locator(`[data-testid="tx-checkbox"][data-tx-id="${r.txId}"]`);
			await expect(caseLigne).toHaveCount(1);
			await caseLigne.check();
			await expect(page.getByTestId('reconciliation-accept-btn')).toBeDisabled();
			await expect(page.getByTestId('reconciliation-reject-btn')).toBeEnabled();
			const indication = page.getByTestId('reconciliation-unmatched-not-acceptable-hint');
			await expect(indication).toBeVisible();
			await page.getByTestId('reconciliation-proposals').screenshot({
				path: capturePath(testInfo, `29-6-ligne-cochee-${suffixe}`),
			});

			await page.getByTestId('reconciliation-reject-btn').click();
			await expect(page.getByTestId('reconciliation-success')).toContainText('1');
			await expect(ligne).toHaveCount(0);
			await expect(indication).toHaveCount(0);
			await page.getByTestId('reconciliation-proposals').screenshot({
				path: capturePath(testInfo, `29-6-apres-rejet-${suffixe}`),
			});

			// Le rejet a traversé la frontière HTTP : le détail de l'import le montre.
			await page.goto(`/bank-import/${r.importId}`);
			const detail = page.locator(`[data-testid="detail-tx-row"][data-tx-id="${r.txId}"]`);
			await expect(detail.getByTestId('detail-tx-rejected-on')).toContainText('Rejetée le');
			await detail.getByTestId('detail-tx-unreject').click();
			await expect(detail.getByTestId('detail-tx-rejected-on')).toHaveCount(0);

			// Réversible : la ligne revient dans la liste.
			await page.goto('/reconciliation');
			await page.getByTestId('bank-account-select').selectOption(baId.toString());
			await expect(
				page.locator(`[data-testid="reconciliation-row"][data-tx-id="${r.txId}"]`),
			).toBeVisible({ timeout: 10000 });
		} finally {
			if (importId !== null) await ctx.delete(`/api/v1/bank-imports/${importId}`);
			await disposeContextSafe(ctx);
		}
	});
}
