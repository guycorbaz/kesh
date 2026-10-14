/**
 * Story 29-3 (#527, AC6) — une facture reconnue par le seul nom de son client passe devant la
 * règle d'affectation qui vise la même contrepartie, de bout en bout : la ligne montre la
 * facture, pas le badge *Règle* ; la page dit la priorité au-dessus de la liste ; « Accepter »
 * règle la facture, dont il reste 0,01 dû.
 *
 * Montage, fixtures synthétiques, base E2E PARTAGÉE :
 *
 * - facture de 972,90 (`createAndValidateInvoiceViaApi`), contact au nom unique à l'exécution ;
 * - relevé d'une transaction de **972,89** (montant non exact : score de montant 0) du
 *   2026-05-15, dont **toutes** les références (`<Ref>`, `<Ustrd>`, `<EndToEndId>`,
 *   `<AcctSvcrRef>`, identifiant du relevé) sont remplacées par des valeurs uniques de préfixe
 *   `PRIO` — qu'aucun numéro de facture (`F-{YEAR}-{SEQ:04}`) ne contient ni ne commence par les
 *   mêmes quatre caractères : une autre facture ouverte de 972,90 n'en tire aucun score ;
 * - contrepartie = nom du contact + valeur de la règle, tous deux uniques : la facture marque
 *   0,1 (contact), la règle correspond.
 *
 * Ce qui tient la facture hors de portée des autres specs, c'est le reste de 0,01 que laisse
 * l'acceptation. Si le parcours échoue avant, le `finally` règle la facture de 972,89 par l'API,
 * ce qui laisse le même reste (C-29-3-18) ; la règle est supprimée, l'import aussi s'il n'a rien rapproché (409 sinon, toléré). Chaque nettoyage
 * a son propre `try`, statut vérifié ; l'erreur d'origine n'est jamais masquée.
 *
 * Limite : la ligne est cherchée dans la première page de propositions (`GET /proposals`,
 * 100 transactions au plus, lues par identifiant croissant). Si d'autres specs laissaient plus de
 * 100 transactions en attente sur le compte de test partagé, la transaction neuve en sortirait et
 * `candidate-top-1` n'apparaîtrait jamais (délai de 10 s, sans diagnostic propre) — aucun cas
 * constaté à ce jour.
 *
 * # Recette visuelle
 *
 * Le scénario tourne une fois par schéma de couleurs (`page.emulateMedia`) et capture la page
 * avec la phrase, dans `KESH_E2E_CAPTURES_DIR` s'il est posé, sinon dans la sortie du test.
 * Sans `toHaveScreenshot` : aucune référence de comparaison.
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
	createContactWithAddressViaApi,
	createAndValidateInvoiceViaApi,
} from './helpers/api-fixtures';

const FIXTURE = path.join(
	path.dirname(fileURLToPath(import.meta.url)),
	'fixtures',
	'camt053_v04_minimal.xml',
);
const TEST_IBAN = 'CH4431999123000889012';
const PRIORITY = 'Une facture proposée passe toujours devant une règle.';

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

type BankAccount = { id: number; journalAccountId: number | null } & Record<string, unknown>;

/**
 * L'acceptation d'une facture exige un compte bancaire lié au grand livre
 * (`BANK_ACCOUNT_NOT_CONFIGURED` sinon). Le compte de test partagé ne l'est pas d'office : on le
 * lie à un compte de liquidités (`10xx`) le temps du scénario, et on rend l'état d'origine
 * ensuite (`restoreJournalLink`). Rend l'état d'origine.
 */
async function linkToLedger(ctx: APIRequestContext, baId: number): Promise<BankAccount | null> {
	const list = (await (await ctx.get('/api/v1/bank-accounts')).json()) as BankAccount[];
	const ba = list.find((b) => b.id === baId)!;
	if (ba.journalAccountId !== null) return null;
	const accounts = (await (await ctx.get('/api/v1/accounts')).json()) as Array<{
		id: number;
		number: string;
		accountType: string;
		active: boolean;
		postable: boolean;
	}>;
	const liquid = accounts.find(
		(a) => a.active && a.postable && a.accountType === 'Asset' && a.number.startsWith('10'),
	);
	expect(liquid, 'un compte de liquidités imputable').toBeTruthy();
	const put = await ctx.put(`/api/v1/bank-accounts/${baId}`, {
		data: { ...ba, journalAccountId: liquid!.id },
	});
	expect(put.ok(), `liaison du compte bancaire: ${put.status()} ${await put.text()}`).toBeTruthy();
	return ba;
}

async function restoreJournalLink(ctx: APIRequestContext, original: BankAccount): Promise<void> {
	const list = (await (await ctx.get('/api/v1/bank-accounts')).json()) as BankAccount[];
	const current = list.find((b) => b.id === original.id);
	if (current) {
		const res = await ctx.put(`/api/v1/bank-accounts/${original.id}`, {
			data: { ...current, journalAccountId: original.journalAccountId },
		});
		if (!res.ok()) throw new Error(`restauration du lien : statut ${res.status()}`);
	}
}

async function amountDue(ctx: APIRequestContext, invoiceId: number): Promise<number> {
	const res = await ctx.get(`/api/v1/invoices/${invoiceId}`);
	expect(res.ok(), `facture ${invoiceId}: ${res.status()}`).toBeTruthy();
	return Number(((await res.json()) as { amountDue: string | number }).amountDue);
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
	test(`(${suffixe}) une facture reconnue par son client passe devant la règle, et « Accepter » la règle`, async ({
		page,
	}, testInfo) => {
		await page.emulateMedia({ colorScheme });
		await login(page);
		const unique = `${Date.now()}${suffixe}`;
		const contactName = `Priorite ${unique}`;
		const ruleValue = `Regle${unique}`;
		const contactId = await createContactWithAddressViaApi(page, contactName);
		// Dans la fenêtre de dates de la proposition et de l'acceptation (relevé du 2026-05-15).
		const invoiceId = await createAndValidateInvoiceViaApi(page, contactId, '2026-05-10');

		const ctx = await authedApiContext(page);
		let ruleId: number | null = null;
		let importId: number | null = null;
		let unlinked: BankAccount | null = null;
		let parcoursReussi = false;
		try {
			const baId = await bankAccountId(ctx);
			unlinked = await linkToLedger(ctx, baId);
			// Le nom ENREGISTRÉ du contact (une `Personne` le compose : « Pia <nom> »), relu plutôt
			// que supposé : c'est lui que le score de contact compare à la contrepartie.
			const contact = await ctx.get(`/api/v1/contacts/${contactId}`);
			expect(contact.ok(), `contact ${contactId}: ${contact.status()}`).toBeTruthy();
			const storedName = ((await contact.json()) as { name: string }).name;
			expect(storedName).toContain(contactName);
			const accounts = (await (await ctx.get('/api/v1/accounts')).json()) as Array<{
				id: number;
				number: string;
				accountType: string;
				active: boolean;
				postable: boolean;
			}>;
			const revenue = accounts.find((a) => a.number === '3000');
			expect(revenue, 'compte 3000 du préréglage').toBeTruthy();

			const rule = await ctx.post('/api/v1/reconciliation/rules', {
				data: {
					label: `E2E 29-3 ${unique}`,
					matchType: 'counterparty_contains',
					matchValue: ruleValue,
					counterpartyAccountId: revenue!.id,
				},
			});
			expect(rule.status(), await rule.text()).toBe(201);
			ruleId = ((await rule.json()) as { id: number }).id;

			// Chaque remplacement est vérifié : une fixture modifiée ailleurs ne doit pas laisser
			// passer une référence d'origine.
			let xml = fs.readFileSync(FIXTURE, 'utf8');
			for (const [from, to] of [
				['STMT-2026-05-001', `PRIO-STMT-${unique}`],
				['BANK-TX-42', `PRIO-SVCR-${unique}`],
				['E2E-2026-05-001', `PRIO-E2E-${unique}`],
				['<Ref>RF18539007547034</Ref>', `<Ref>PRIO-REF-${unique}</Ref>`],
				['<Ustrd>Loyer mai 2026</Ustrd>', `<Ustrd>PRIO-USTRD-${unique}</Ustrd>`],
				['<Amt Ccy="CHF">1234.56</Amt>', '<Amt Ccy="CHF">972.89</Amt>'],
				['<Nm>Acme SA</Nm>', `<Nm>${storedName} ${ruleValue}</Nm>`],
			]) {
				expect(xml.includes(from), `fixture : « ${from} » introuvable`).toBeTruthy();
				xml = xml.replace(from, to);
			}
			const imported = await ctx.post('/api/v1/bank-imports', {
				multipart: {
					bankAccountId: baId.toString(),
					file: {
						name: `priority-${unique}.xml`,
						mimeType: 'application/xml',
						buffer: Buffer.from(xml, 'utf8'),
					},
					confirmBalanceMismatch: 'true',
					confirmDuplicateFile: 'true',
				},
			});
			expect(imported.status(), await imported.text()).toBe(201);
			importId = ((await imported.json()) as { id: number }).id;
			const detail = (await (await ctx.get(`/api/v1/bank-imports/${importId}`)).json()) as {
				transactions: Array<{ id: number }>;
			};
			const txId = detail.transactions[0].id;

			await page.goto('/reconciliation');
			await page.getByTestId('bank-account-select').selectOption(baId.toString());
			const row = page.locator(`[data-testid="reconciliation-row"][data-tx-id="${txId}"]`);
			await expect(
				row.locator(`[data-testid="candidate-top-1"][data-invoice-id="${invoiceId}"]`),
			).toBeVisible({ timeout: 10000 });
			await expect(row.getByTestId('candidate-rule-badge')).toHaveCount(0);
			const hint = page.getByTestId('reconciliation-priority-hint');
			await expect(hint).toContainText(PRIORITY);
			await page.screenshot({ path: capturePath(testInfo, `reconciliation-priorite-${suffixe}`) });

			await row.getByTestId('tx-checkbox').check();
			await page.getByTestId('reconciliation-accept-btn').click();
			await expect(page.getByTestId('reconciliation-success')).toHaveText(/^\s*1\s+\D/);
			await expect(row).toHaveCount(0);
			await expect(page.getByTestId('reconciliation-failed')).toHaveCount(0);
			expect(await amountDue(ctx, invoiceId)).toBe(0.01);
			parcoursReussi = true;
		} finally {
			// Chaque nettoyage dans son propre `try` : l'échec de l'un n'empêche pas les suivants, et
			// l'erreur d'origine du parcours n'est jamais masquée (un `throw` dans un `finally`
			// l'écraserait). Les échecs de nettoyage sont rapportés, et ne font échouer le test que
			// si le parcours lui-même a réussi.
			const cleanupFailures: string[] = [];
			const step = async (label: string, run: () => Promise<void>): Promise<void> => {
				try {
					await run();
				} catch (e) {
					cleanupFailures.push(`${label} : ${e instanceof Error ? e.message : String(e)}`);
				}
			};
			const ok = (res: { ok(): boolean; status(): number }, what: string): void => {
				if (!res.ok()) throw new Error(`${what} : statut ${res.status()}`);
			};
			// C-29-3-18 — une facture restée due à 972,90 serait candidate À MONTANT EXACT pour
			// toute transaction de 972,90 d'un autre spec : la régler laisse le même reste de 0,01.
			await step('règlement de secours', async () => {
				if ((await amountDue(ctx, invoiceId)) !== 972.9) return;
				const accountsRes = await ctx.get('/api/v1/accounts');
				ok(accountsRes, 'lecture des comptes');
				const cash = ((await accountsRes.json()) as Array<{
					id: number;
					number: string;
					accountType: string;
					active: boolean;
					postable: boolean;
				}>).find((a) => a.active && a.postable && a.accountType === 'Asset' && a.number.startsWith('10'));
				if (!cash) throw new Error('aucun compte de liquidités imputable');
				const settled = await ctx.post(`/api/v1/invoices/${invoiceId}/settlements`, {
					data: {
						settlementType: 'internal_account',
						accountId: cash.id,
						amount: '972.89',
						settledOn: '2026-05-15',
					},
				});
				ok(settled, 'règlement de secours');
			});
			if (ruleId !== null) {
				const id = ruleId;
				await step('suppression de la règle', async () =>
					ok(await ctx.delete(`/api/v1/reconciliation/rules/${id}`), 'suppression de la règle'),
				);
			}
			if (importId !== null) {
				const id = importId;
				// 409 attendu après un parcours réussi : l'import porte une transaction rapprochée,
				// que le serveur refuse de supprimer (`BANK_IMPORT_HAS_RECONCILED_TRANSACTIONS`).
				// L'ancienne suppression, dont le statut n'était pas lu, ne supprimait donc rien
				// dans ce cas ; elle ne supprime que l'import d'un parcours échoué avant « Accepter ».
				await step('suppression de l’import', async () => {
					const res = await ctx.delete(`/api/v1/bank-imports/${id}`);
					if (res.status() !== 409) ok(res, 'statut');
				});
			}
			if (unlinked !== null) {
				const original = unlinked;
				await step('restauration du lien du compte bancaire', () => restoreJournalLink(ctx, original));
			}
			await step('libération du contexte', () => disposeContextSafe(ctx));
			if (cleanupFailures.length > 0) {
				console.warn(`[reconciliation-priority] nettoyage incomplet : ${cleanupFailures.join(' ; ')}`);
				if (parcoursReussi) throw new Error(`nettoyage incomplet : ${cleanupFailures.join(' ; ')}`);
			}
		}
	});
}
