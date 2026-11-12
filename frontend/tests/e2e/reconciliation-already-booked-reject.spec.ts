/**
 * Story 35-4 (#1023, #1024) — *Rejeter* une ligne signalée « déjà comptabilisée » demande
 * confirmation ; une confirmation ouverte bloque la page, si bien que la liste ne se relit pas
 * sous elle.
 *
 * Deux tests, distincts de celui de la 34-4 (`reconciliation-already-booked.spec.ts`), même
 * montage : un Comptable saisit par l'API une écriture `D charge / C banque` d'un montant
 * unique, puis un relevé dont un **débit** l'égale (`createManualDebitStatementViaApi`) : la ligne
 * est signalée.
 *
 * 1. **#1023 (AC7)** — la ligne cochée, *Rejeter* → la confirmation la nomme et dit ce que fait
 *    le rejet ; *Annuler*, puis le clic hors du dialogue, la laissent dans la liste ; *Rejeter
 *    quand même* la retire. C'est le seul test qui voit le geste traverser la frontière HTTP.
 * 2. **#1024 (étape 0 de la story)** — plus de cent transactions en attente (relevé CAMT
 *    synthétique, `helpers/camt-synthetic.ts`), la confirmation ouverte : le bouton *Afficher
 *    aussi les plus anciennes* ne reçoit pas le clic, Tab ne sort pas du dialogue, aucune
 *    relecture de la liste ne part ; Échap et le clic hors valent *Annuler*. Le chemin de #1024
 *    (une relecture sous une confirmation ouverte) n'est donc pas atteignable dans un navigateur
 *    par la page elle-même ; la story le ferme quand même (`load()` referme la confirmation), en
 *    Vitest.
 *
 * ⚠️ Montants UNIQUES à chaque exécution (base partagée entre specs). En `finally` : l'écriture
 * saisie est contre-passée (non rapprochée, elle signalerait toute transaction de son montant
 * dans les autres specs) et les imports sont supprimés — un import dont une ligne est rejetée se
 * supprime (`rejected_count`, `kesh-db/src/repositories/bank_imports.rs`).
 *
 * # Recette visuelle
 *
 * Capture de la confirmation de *Rejeter*, dans `KESH_E2E_CAPTURES_DIR` s'il est posé, sinon
 * dans le dossier de sortie du test. Thème clair seulement (Kesh n'a pas de thème sombre).
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
import {
	createManualDebitStatementViaApi,
	todayInZurich,
	wiredBankAccountViaApi,
} from './helpers/api-fixtures';
import { syntheticCamtStatement } from './helpers/camt-synthetic';

const TEST_IBAN = 'CH4431999123000889012';
const CATALOGUE_FR = path.join(
	path.dirname(fileURLToPath(import.meta.url)),
	'..',
	'..',
	'..',
	'crates',
	'kesh-i18n',
	'locales',
	'fr-CH',
	'messages.ftl',
);

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

/** Le dossier des captures : `KESH_E2E_CAPTURES_DIR`, sinon la sortie du test. */
function capturePath(testInfo: TestInfo, name: string): string {
	const dossier = process.env.KESH_E2E_CAPTURES_DIR;
	return dossier ? path.join(dossier, `${name}.png`) : testInfo.outputPath(`${name}.png`);
}

/** La valeur fr-CH d'une clé, lue au catalogue. */
function catalogueFr(cle: string): string {
	const prefixe = `${cle} = `;
	const ligne = fs
		.readFileSync(CATALOGUE_FR, 'utf8')
		.split('\n')
		.find((l) => l.startsWith(prefixe));
	expect(ligne, `clé absente du catalogue fr-CH : ${cle}`).toBeDefined();
	return ligne!.slice(prefixe.length);
}

type Signalee = { txId: number; importId: number; journalEntryId: number; counterparty: string };

/**
 * Une écriture `D charge / C banque` d'un montant unique, puis un relevé saisi dont le débit
 * l'égale : la transaction est signalée « déjà comptabilisée ». Les identifiants sont rendus au
 * fur et à mesure dans `out`, pour le nettoyage d'un montage interrompu.
 */
async function monterLigneSignalee(
	page: Page,
	ctx: APIRequestContext,
	bankAccountId: number,
	out: Partial<Signalee>,
): Promise<Signalee> {
	const cents = 30000 + (Date.now() % 29000);
	const amount = (cents / 100).toFixed(2);
	const unique = `${Date.now()}${Math.floor(Math.random() * 1000)}`;
	const today = todayInZurich();
	const ba = (
		(await (await ctx.get('/api/v1/bank-accounts')).json()) as Array<{
			id: number;
			journalAccountId: number | null;
		}>
	).find((b) => b.id === bankAccountId);
	expect(ba?.journalAccountId, 'compte bancaire relié au grand livre').toBeTruthy();
	const accounts = (await (await ctx.get('/api/v1/accounts')).json()) as Array<{
		id: number;
		accountType: string;
		active: boolean;
		postable: boolean;
	}>;
	const charge = accounts.find((a) => a.accountType === 'Expense' && a.active && a.postable);
	expect(charge, 'un compte de charge imputable').toBeTruthy();
	const entry = await ctx.post('/api/v1/journal-entries', {
		data: {
			entryDate: today,
			journal: 'Banque',
			description: `Dépense déjà payée 35-4 ${unique}`,
			lines: [
				{ accountId: charge!.id, debit: amount, credit: '0' },
				{ accountId: ba!.journalAccountId, debit: '0', credit: amount },
			],
		},
	});
	expect(entry.status(), await entry.text()).toBe(201);
	out.journalEntryId = ((await entry.json()) as { id: number }).id;

	out.counterparty = `Fournisseur 35-4 ${unique}`;
	out.importId = await createManualDebitStatementViaApi(page, {
		bankAccountId,
		bookingDate: today,
		amount,
		counterpartyName: out.counterparty,
		details: `Paiement déjà comptabilisé 35-4 ${unique}`,
	});
	const detail = (await (await ctx.get(`/api/v1/bank-imports/${out.importId}`)).json()) as {
		transactions: Array<{ id: number }>;
	};
	expect(detail.transactions).toHaveLength(1);
	out.txId = detail.transactions[0].id;
	return out as Signalee;
}

/** Contre-passe l'écriture saisie et supprime les imports, quoi qu'il en soit du test. */
async function nettoyer(
	ctx: APIRequestContext,
	montage: Partial<Signalee>,
	autresImports: number[] = [],
): Promise<void> {
	if (montage.journalEntryId !== undefined) {
		await ctx.post(`/api/v1/journal-entries/${montage.journalEntryId}/reverse`);
	}
	for (const id of [montage.importId, ...autresImports]) {
		if (id !== undefined) await ctx.delete(`/api/v1/bank-imports/${id}`);
	}
}

test('Rejeter une ligne signalée demande confirmation ; Annuler et le clic hors la laissent ; Rejeter quand même la retire (#1023)', async ({
	page,
}, testInfo) => {
	await login(page);
	const bankAccountId = await wiredBankAccountViaApi(page, TEST_IBAN);
	const ctx = await authedApiContext(page);
	const montage: Partial<Signalee> = {};
	try {
		const { txId, counterparty } = await monterLigneSignalee(page, ctx, bankAccountId, montage);

		await page.goto('/reconciliation');
		await page.getByTestId('bank-account-select').selectOption(bankAccountId.toString());
		const ligne = page.locator(`[data-testid="reconciliation-row"][data-tx-id="${txId}"]`);
		await expect(ligne).toBeVisible({ timeout: 10000 });
		await expect(
			page.locator(`[data-testid="already-booked-warning"][data-tx-id="${txId}"]`),
		).toBeVisible();
		const caseLigne = page.locator(`[data-testid="tx-checkbox"][data-tx-id="${txId}"]`);
		const rejeter = page.getByTestId('reconciliation-reject-btn');
		const confirmation = page.getByTestId('already-booked-confirm');

		// Rejeter → la confirmation nomme la ligne, dit le rejet et sa portée → Annuler.
		await caseLigne.check();
		await rejeter.click();
		await expect(confirmation).toBeVisible();
		const item = confirmation.getByTestId('already-booked-confirm-item');
		await expect(item).toHaveCount(1);
		await expect(item).toContainText(counterparty);
		await expect(confirmation.getByTestId('already-booked-confirm-intro')).toHaveText(
			catalogueFr('reconciliation-labels-already-booked-confirm-intro-reject-one'),
		);
		await expect(confirmation.getByTestId('already-booked-confirm-continue')).toHaveText(
			catalogueFr('reconciliation-labels-already-booked-confirm-reject'),
		);
		await confirmation.screenshot({ path: capturePath(testInfo, 'e354-confirmation-rejeter') });
		await confirmation.getByTestId('already-booked-confirm-cancel').click();
		await expect(confirmation).toHaveCount(0);
		await expect(ligne).toBeVisible();
		await expect(page.getByTestId('reconciliation-success')).toHaveCount(0);

		// Le clic hors du dialogue vaut Annuler.
		await expect(caseLigne).toBeChecked();
		await rejeter.click();
		await expect(confirmation).toBeVisible();
		await page.mouse.click(5, 5);
		await expect(confirmation).toHaveCount(0);
		await expect(ligne).toBeVisible();
		await expect(page.getByTestId('reconciliation-success')).toHaveCount(0);

		// Rejeter quand même → la ligne quitte la liste.
		await rejeter.click();
		await confirmation.getByTestId('already-booked-confirm-continue').click();
		await expect(page.getByTestId('reconciliation-success')).toContainText('1');
		await expect(ligne).toHaveCount(0);
		await expect(confirmation).toHaveCount(0);

		// Le rejet a traversé la frontière HTTP : le détail de l'import le montre.
		await page.goto(`/bank-import/${montage.importId}`);
		const detail = page.locator(`[data-testid="detail-tx-row"][data-tx-id="${txId}"]`);
		await expect(detail.getByTestId('detail-tx-rejected-on')).toContainText('Rejetée le');
	} finally {
		await nettoyer(ctx, montage);
		await disposeContextSafe(ctx);
	}
});

test('la confirmation ouverte bloque la page : ni « Afficher aussi les plus anciennes », ni Tab, ni relecture ; Échap et le clic hors valent Annuler (#1024)', async ({
	page,
}) => {
	test.setTimeout(120_000);
	await login(page);
	const bankAccountId = await wiredBankAccountViaApi(page, TEST_IBAN);
	const ctx = await authedApiContext(page);
	const montage: Partial<Signalee> = {};
	let camtId: number | undefined;
	try {
		const { txId } = await monterLigneSignalee(page, ctx, bankAccountId, montage);
		// 101 transactions plus anciennes : la liste en montre cent, et le bouton apparaît.
		const unique = `${Date.now()}`;
		const imported = await ctx.post('/api/v1/bank-imports', {
			multipart: {
				bankAccountId: bankAccountId.toString(),
				file: {
					name: `e354-${unique}.xml`,
					mimeType: 'application/xml',
					buffer: Buffer.from(
						syntheticCamtStatement(101, unique, { tag: 'E354', name: 'Ancienne 35-4' }),
						'utf8',
					),
				},
				confirmBalanceMismatch: 'true',
				confirmDuplicateFile: 'true',
			},
		});
		expect(imported.status(), await imported.text()).toBe(201);
		const imp = (await imported.json()) as { id: number; transactionCount: number };
		camtId = imp.id;
		expect(imp.transactionCount).toBe(101);

		// Toute relecture de la liste passe par `GET …/reconciliation/proposals`.
		let relectures = 0;
		page.on('request', (r) => {
			if (r.url().includes('/reconciliation/proposals')) relectures++;
		});

		await page.goto('/reconciliation');
		await page.getByTestId('bank-account-select').selectOption(bankAccountId.toString());
		const lignes = page.getByTestId('reconciliation-row');
		await expect(lignes).toHaveCount(100, { timeout: 15000 });
		const ligne = page.locator(`[data-testid="reconciliation-row"][data-tx-id="${txId}"]`);
		await expect(ligne).toBeVisible();
		const plusAnciennes = page.getByTestId('reconciliation-show-older-btn');
		await expect(plusAnciennes).toBeVisible();

		const confirmation = page.getByTestId('already-booked-confirm');
		await ligne.getByTestId('manual-match-button').click();
		await expect(confirmation).toBeVisible();
		const avant = relectures;

		// Le bouton est sous la modale : le clic ne l'atteint pas.
		await expect(plusAnciennes.click({ timeout: 2000 })).rejects.toThrow(
			/intercepts pointer events/,
		);
		// Tab reste dans le dialogue.
		for (let i = 0; i < 6; i++) {
			await page.keyboard.press('Tab');
			const dedans = await page.evaluate(
				() =>
					document.activeElement?.closest('[data-testid="already-booked-confirm"]') !== null,
			);
			expect(dedans, `Tab n° ${i + 1} : le focus reste dans le dialogue`).toBe(true);
		}
		// Aucune relecture n'est partie, la liste est intacte.
		expect(relectures).toBe(avant);
		await expect(lignes).toHaveCount(100);
		await expect(confirmation).toBeVisible();

		// Échap vaut Annuler : rien ne s'ouvre.
		await page.keyboard.press('Escape');
		await expect(confirmation).toHaveCount(0);
		await expect(page.getByTestId('manual-match-modal')).toHaveCount(0);

		// Le clic hors vaut Annuler.
		await ligne.getByTestId('manual-match-button').click();
		await expect(confirmation).toBeVisible();
		await page.mouse.click(5, 5);
		await expect(confirmation).toHaveCount(0);
		await expect(page.getByTestId('manual-match-modal')).toHaveCount(0);
		expect(relectures).toBe(avant);
		await expect(lignes).toHaveCount(100);
	} finally {
		await nettoyer(ctx, montage, camtId === undefined ? [] : [camtId]);
		await disposeContextSafe(ctx);
	}
});
