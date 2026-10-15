/**
 * Story 29-9 (#649, #678, AC9, AC11) — la page *Réconciliation* dit qu'il reste des transactions
 * au-delà des 100 plus récentes, et les montre jusqu'à 500 ; chaque case nomme sa transaction,
 * et l'indication « ne peut pas être acceptée » vit dans une région d'annonce présente dès le
 * rendu.
 *
 * C'est le seul test qui voit `hasMore` traverser la frontière HTTP jusqu'à l'avis, et le
 * `?limit=500` du bouton jusqu'au serveur (`CLAUDE.md`, § *E2E*).
 *
 * ⚠️ **Relevé SYNTHÉTIQUE de 501 transactions**, bâti à chaque exécution en répétant l'entrée de
 * `camt053_v04_minimal.xml` : montant, `AcctSvcrRef`, `EndToEndId` et contrepartie distincts par
 * entrée et par exécution — le dédoublonnage par défaut de l'import est un *skip* silencieux sur
 * (date, montant, référence, compte), et une entrée en double fausserait le compte sans erreur.
 * La spec vérifie que l'import en crée **501**. Dates : une par jour, du 2025-01-01 au
 * 2026-05-16 ; la liste est triée par date décroissante. `seedTestState('with-company')` remet la
 * base à zéro : la spec ne voit que ses propres transactions. L'import est supprimé en `finally`.
 *
 * # Recette visuelle (action B2 de la rétrospective de l'Epic 27)
 *
 * Les deux états — avis et bouton, puis avis plafonné — sont capturés en clair et en sombre
 * (`page.emulateMedia`), dans `KESH_E2E_CAPTURES_DIR` s'il est posé, sinon dans le dossier de
 * sortie du test. Hors dépôt, sans `toHaveScreenshot`.
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

const ICI = path.dirname(fileURLToPath(import.meta.url));
const FIXTURE = path.join(ICI, 'fixtures', 'camt053_v04_minimal.xml');
const CATALOGUE_FR = path.join(ICI, '..', '..', '..', 'crates', 'kesh-i18n', 'locales', 'fr-CH', 'messages.ftl');
const TEST_IBAN = 'CH4431999123000889012';
const N = 501;

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

/** La date `AAAA-MM-JJ` de l'entrée `i` : un jour par entrée depuis le 2025-01-01. */
function dateOf(i: number): string {
	return new Date(Date.UTC(2025, 0, 1 + i)).toISOString().slice(0, 10);
}

/** Le relevé synthétique : l'entrée de la fixture répétée `N` fois, chacune unique. */
function syntheticStatement(unique: string): string {
	const xml = fs.readFileSync(FIXTURE, 'utf8');
	const debut = xml.indexOf('<Ntry>');
	const fin = xml.indexOf('</Ntry>') + '</Ntry>'.length;
	expect(debut > 0 && fin > debut, 'fixture : bloc <Ntry> introuvable').toBeTruthy();
	const modele = xml.slice(debut, fin);
	const entrees = Array.from({ length: N }, (_, i) => {
		let e = modele;
		for (const [from, to] of [
			['<Amt Ccy="CHF">1234.56</Amt>', `<Amt Ccy="CHF">${(1000 + i).toFixed(2)}</Amt>`],
			['<Dt>2026-05-15</Dt>', `<Dt>${dateOf(i)}</Dt>`],
			['<Dt>2026-05-15</Dt>', `<Dt>${dateOf(i)}</Dt>`],
			['BANK-TX-42', `OLDER-${unique}-${i}`],
			['E2E-2026-05-001', `E2E-OLDER-${unique}-${i}`],
			['<Nm>Acme SA</Nm>', `<Nm>Ancienne ${i} ${unique}</Nm>`],
		]) {
			expect(e.includes(from), `fixture : « ${from} » introuvable`).toBeTruthy();
			e = e.replace(from, to);
		}
		return e;
	});
	let out = xml.slice(0, debut) + entrees.join('\n') + xml.slice(fin);
	for (const [from, to] of [
		['STMT-2026-05-001', `STMT-OLDER-${unique}`],
		['<FrDtTm>2026-05-01T00:00:00</FrDtTm>', '<FrDtTm>2025-01-01T00:00:00</FrDtTm>'],
		['<Dt>2026-05-01</Dt>', '<Dt>2025-01-01</Dt>'],
	]) {
		expect(out.includes(from), `fixture : « ${from} » introuvable`).toBeTruthy();
		out = out.replace(from, to);
	}
	return out;
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

async function capturerDeuxSchemas(page: Page, testInfo: TestInfo, cible: string, nom: string) {
	for (const [colorScheme, suffixe] of [
		['light', 'clair'],
		['dark', 'sombre'],
	] as const) {
		await page.emulateMedia({ colorScheme });
		await page.locator(cible).screenshot({ path: capturePath(testInfo, `${nom}-${suffixe}`) });
	}
	await page.emulateMedia({ colorScheme: 'light' });
}

test('au-delà de 100 : l’avis et le bouton ; au plafond de 500 : l’avis seul ; cases nommées, région d’annonce', async ({
	page,
}, testInfo) => {
	test.setTimeout(120_000);
	await login(page);
	const ctx = await authedApiContext(page);
	let importId: number | null = null;
	try {
		const baId = await bankAccountId(ctx);
		const unique = `${Date.now()}`;
		const imported = await ctx.post('/api/v1/bank-imports', {
			multipart: {
				bankAccountId: baId.toString(),
				file: {
					name: `show-older-${unique}.xml`,
					mimeType: 'application/xml',
					buffer: Buffer.from(syntheticStatement(unique), 'utf8'),
				},
				confirmBalanceMismatch: 'true',
				confirmDuplicateFile: 'true',
			},
		});
		expect(imported.status(), await imported.text()).toBe(201);
		const imp = (await imported.json()) as { id: number; transactionCount: number };
		importId = imp.id;
		expect(imp.transactionCount, 'les 501 entrées doivent être créées, aucune écartée').toBe(N);

		const detail = (await (await ctx.get(`/api/v1/bank-imports/${importId}`)).json()) as {
			transactions: Array<{ id: number; bookingDate: string }>;
		};
		// La 150e par ordre décroissant de date : l'entrée N - 150.
		const cent50e = detail.transactions.find((t) => t.bookingDate === dateOf(N - 150));
		expect(cent50e, 'la 150e transaction est introuvable dans le détail').toBeDefined();

		await page.goto('/reconciliation');
		await page.getByTestId('bank-account-select').selectOption(baId.toString());
		const lignes = page.getByTestId('reconciliation-row');
		await expect(lignes).toHaveCount(100, { timeout: 15000 });
		await expect(page.getByTestId('reconciliation-more')).toContainText('100');
		const bouton = page.getByTestId('reconciliation-show-older-btn');
		await expect(bouton).toBeVisible();
		await expect(page.locator(`[data-testid="reconciliation-row"][data-tx-id="${cent50e!.id}"]`)).toHaveCount(0);

		// #678 — la case de la première ligne porte le nom de ce que la ligne affiche.
		const premiere = lignes.first();
		const cellules = premiere.locator('td');
		const [date, montant, contrepartie] = await Promise.all([
			cellules.nth(1).innerText(),
			cellules.nth(2).innerText(),
			cellules.nth(3).innerText(),
		]);
		const nom = catalogueFr('reconciliation-labels-select-transaction')
			.replace('{ $date }', date.trim())
			.replace('{ $amount } { $currency }', montant.trim())
			.replace('{ $counterparty }', contrepartie.trim());
		const caseAcocher = page.getByRole('checkbox', { name: nom, exact: true });
		await expect(caseAcocher).toHaveCount(1);
		const region = page.getByTestId('reconciliation-selection-status');
		await expect(region).toHaveAttribute('aria-live', 'polite');
		await expect(region).toHaveText('');
		await caseAcocher.check();
		await expect(region.getByTestId('reconciliation-unmatched-not-acceptable-hint')).toBeVisible();
		await caseAcocher.uncheck();
		await expect(page.getByTestId('reconciliation-unmatched-not-acceptable-hint')).toHaveCount(0);
		await expect(region).toHaveCount(1);

		await capturerDeuxSchemas(page, testInfo, '[data-testid="reconciliation-more"] >> xpath=..', 'avis-et-bouton');

		// Le bouton porte la liste au plafond.
		await bouton.click();
		await expect(lignes).toHaveCount(500, { timeout: 15000 });
		const plafond = page.getByTestId('reconciliation-more-capped');
		await expect(plafond).toContainText('500');
		await expect(page.getByTestId('reconciliation-show-older-btn')).toHaveCount(0);
		await expect(page.locator(`[data-testid="reconciliation-row"][data-tx-id="${cent50e!.id}"]`)).toHaveCount(1);

		await capturerDeuxSchemas(page, testInfo, '[data-testid="reconciliation-more-capped"]', 'avis-plafonne');
	} finally {
		if (importId !== null) await ctx.delete(`/api/v1/bank-imports/${importId}`);
		await disposeContextSafe(ctx);
	}
});
