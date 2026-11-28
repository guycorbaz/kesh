/**
 * Story 46-6b — la balance des comptes par groupes, de bout en bout : écran
 * groupé par défaut (quatre colonnes, résultat reporté après les groupes),
 * sous-totaux en pied, niveau 1, ouverture d'un groupe à la main, export CSV et
 * PDF aux mêmes paramètres que l'écran — et leur contenu —, hiérarchie masquée
 * qui rend l'écran d'avant, zéro violation axe sur l'onglet groupé.
 *
 * ⚠️ Ce que seul un E2E vérifie : que les paramètres de groupement traversent la
 * frontière HTTP dans la requête de la balance ET dans celle de son export, et
 * que le fichier téléchargé est groupé comme l'écran.
 *
 * Plan : le preset `with-company`, plus l'arbre `19` > `190` > `1900`, `1910`
 * mouvementé contre `3000`, créé par `preparerArbre` de l'aide commune
 * (`helpers/reports-groupes.ts`).
 *
 * Captures : `KESH_CAPTURES_DIR` (thème clair), une par état, et les deux
 * fichiers exportés.
 */

import { readFileSync, writeFileSync } from 'node:fs';
import AxeBuilder from '@axe-core/playwright';
import { expect, test, type Page } from '@playwright/test';
import { seedTestState, clearAuthStorage } from './helpers/test-state';
import { login, preparerArbre } from './helpers/reports-groupes';

test.beforeAll(async () => {
	await seedTestState('with-company');
});

test.afterEach(async ({ page }) => {
	await clearAuthStorage(page);
});

const CAPTURES = process.env.KESH_CAPTURES_DIR ?? 'test-results';

async function capture(page: Page, nom: string): Promise<void> {
	await page.screenshot({ path: `${CAPTURES}/46-6b-${nom}.png`, fullPage: true });
}

const BALANCE = /\/api\/v1\/reports\/trial-balance\?/;

/** Les cellules d'une ligne, espaces resserrés. */
async function cellules(page: Page, testId: string): Promise<string[]> {
	const textes = await page.getByTestId(testId).locator('td').allInnerTexts();
	return textes.map((t) => t.replace(/\s+/g, ' ').trim());
}

/** Exporte l'onglet affiché ; rend l'URL de l'export et le fichier, gardé dans les captures. */
async function exporter(page: Page, format: 'csv' | 'pdf', nom: string): Promise<[URL, Buffer]> {
	const exportReq = page.waitForRequest((r) => /reports\/trial-balance\/export\?/.test(r.url()));
	const download = page.waitForEvent('download');
	await page.getByRole('button', { name: format === 'csv' ? /export csv/i : /export pdf/i }).click();
	const url = new URL((await exportReq).url());
	const fichier = await download;
	const chemin = await fichier.path();
	expect(chemin, 'fichier téléchargé').toBeTruthy();
	const corps = readFileSync(chemin!);
	writeFileSync(`${CAPTURES}/46-6b-${nom}.${format}`, corps);
	return [url, corps];
}

test('balance par groupes : défaut, pied, niveau 1, ouverture à la main, exports, à plat (Story 46-6b)', async ({
	page,
}) => {
	await login(page);
	const g19 = await preparerArbre(page);

	await page.goto('/reports');
	await page.waitForLoadState('networkidle');

	// 1. Première visite : groupée, tous les comptes, sous-totaux sur le titre (AC-1).
	// Story 55-1a (D16) — plus de *Générer* : le clic sur l'onglet charge la balance.
	let req = page.waitForRequest((r) => BALANCE.test(r.url()));
	await page.locator('#reports-tab-trial-balance').click();
	await expect(page.getByTestId('grouping-controls')).toBeVisible();
	let url = new URL((await req).url());
	expect(url.searchParams.get('detailLevel')).toBe('all');
	expect(url.searchParams.get('subtotalPosition')).toBe('header');
	expect(url.searchParams.has('open')).toBe(false);
	// Ouverture, débit, crédit, clôture du groupe : ses deux comptes.
	expect((await cellules(page, 'grouped-group-19')).slice(2)).toEqual([
		'0.00',
		'1’250.00',
		'0.00',
		'1’250.00',
	]);
	expect((await cellules(page, 'grouped-account-1900')).slice(2)).toEqual([
		'0.00',
		'1’000.00',
		'0.00',
		'1’000.00',
	]);
	await expect(page.getByTestId('group-toggle-19')).toHaveAttribute('aria-expanded', 'true');
	// Le résultat reporté suit les groupes, hors d'eux ; totaux et contrôles restent.
	await expect(page.locator('tbody tr').last()).toHaveAttribute('data-testid', 'tb-retained');
	await expect(page.getByTestId('tb-opening-check')).toHaveText('✓');
	await expect(page.getByTestId('tb-movements-check')).toHaveText('✓');
	await expect(page.getByRole('tabpanel').getByRole('columnheader')).toHaveCount(6);
	await capture(page, '1-defaut-tous-les-comptes');

	// Zéro violation axe sur l'onglet Balance groupé (AC-1).
	const axe = await new AxeBuilder({ page }).withTags(['wcag2a', 'wcag2aa']).analyze();
	expect(axe.violations).toEqual([]);

	// 2. Sous-totaux en pied : « Total <nom> », le titre sans montant.
	req = page.waitForRequest((r) => BALANCE.test(r.url()));
	await page.getByTestId('grouping-position').selectOption('footer');
	expect(new URL((await req).url()).searchParams.get('subtotalPosition')).toBe('footer');
	await expect(page.getByTestId('grouped-subtotal-19')).toContainText('Total Actifs du test');
	expect((await cellules(page, 'grouped-subtotal-19')).slice(2)).toEqual([
		'0.00',
		'1’250.00',
		'0.00',
		'1’250.00',
	]);
	expect((await cellules(page, 'grouped-group-19')).slice(2)).toEqual(['', '', '', '']);
	await capture(page, '2-sous-totaux-en-pied');
	req = page.waitForRequest((r) => BALANCE.test(r.url()));
	await page.getByTestId('grouping-position').selectOption('header');
	await req;

	// 3. Niveau 1 : 19 fermé sur son sous-total, sa descendance cachée.
	req = page.waitForRequest((r) => BALANCE.test(r.url()));
	await page.getByTestId('grouping-level').selectOption('1');
	expect(new URL((await req).url()).searchParams.get('detailLevel')).toBe('1');
	await expect(page.getByTestId('grouped-account-1900')).toHaveCount(0);
	await expect(page.getByTestId('group-toggle-19')).toHaveAttribute('aria-expanded', 'false');
	await expect(page.getByTestId('grouped-group-19')).toContainText('1’250.00');
	await capture(page, '3-niveau-1');

	// 4. Ouverture de 19 à la main : 190 paraît, fermé par le niveau.
	req = page.waitForRequest((r) => BALANCE.test(r.url()));
	await page.getByTestId('group-toggle-19').click();
	url = new URL((await req).url());
	expect(url.searchParams.get('open')).toBe(String(g19));
	await expect(page.getByTestId('group-toggle-190')).toHaveAttribute('aria-expanded', 'false');
	await expect(page.getByTestId('grouped-group-190')).toContainText('1’250.00');
	await expect(page.getByTestId('grouped-account-1900')).toHaveCount(0);
	await capture(page, '4-groupe-ouvert-a-la-main');

	// 5. Export CSV : les paramètres de l'écran, et le fichier groupé comme lui (AC-2).
	const [csvUrl, csv] = await exporter(page, 'csv', '5-export-niveau-1-ouvert');
	expect(csvUrl.searchParams.get('format')).toBe('csv');
	expect(csvUrl.searchParams.get('detailLevel')).toBe('1');
	expect(csvUrl.searchParams.get('subtotalPosition')).toBe('header');
	expect(csvUrl.searchParams.get('open')).toBe(String(g19));
	expect(csvUrl.searchParams.has('closed')).toBe(false);
	const lignes = csv.toString('utf8').replace(/^﻿/, '').split('\r\n');
	expect(lignes[0]).toBe(
		'Niveau;TypeLigne;NumeroCompte;NomCompte;SoldeOuverture;TotalDebit;TotalCredit;SoldeCloture',
	);
	expect(lignes).toContain('1;groupe;19;Actifs du test;0.00;1250.00;0.00;1250.00');
	expect(lignes).toContain('2;groupe;190;Liquidités du test;0.00;1250.00;0.00;1250.00');
	expect(lignes.some((l) => l.includes(';1900;'))).toBe(false);
	expect(lignes.some((l) => l.startsWith(';;Total;'))).toBe(true);

	// 6. Export PDF, mêmes paramètres : l'en-tête dit le niveau et l'ajustement.
	const [pdfUrl, pdf] = await exporter(page, 'pdf', '6-export-niveau-1-ouvert');
	expect(pdfUrl.searchParams.get('detailLevel')).toBe('1');
	expect(pdfUrl.searchParams.get('open')).toBe(String(g19));
	expect(pdf.subarray(0, 5).toString()).toBe('%PDF-');

	// 7. Hiérarchie masquée : l'écran d'avant, requête sans paramètre.
	req = page.waitForRequest((r) => BALANCE.test(r.url()));
	await page.getByTestId('grouping-shown').uncheck();
	url = new URL((await req).url());
	expect(url.searchParams.has('detailLevel')).toBe(false);
	await expect(page.getByTestId('grouped-group-19')).toHaveCount(0);
	await expect(page.getByTestId('tb-row').first()).toBeVisible();
	await expect(page.getByRole('link', { name: '1900' })).toBeVisible();
	await expect(page.getByTestId('grouping-level')).toBeDisabled();
	await capture(page, '7-hierarchie-masquee');

	// 7 bis. Export de l'écran à plat : sans paramètre, l'export de l'API est groupé
	// (présentation de l'écran par défaut) ; l'écran à plat demande donc
	// `detailLevel=flat`, et le fichier n'a pas les colonnes groupées.
	const [platUrl, plat] = await exporter(page, 'csv', '7-a-plat');
	expect(platUrl.searchParams.get('detailLevel')).toBe('flat');
	expect(platUrl.searchParams.has('subtotalPosition')).toBe(false);
	const platTexte = plat.toString('utf8').replace(/^\uFEFF/, '');
	expect(platTexte.split(/\r?\n/)[0]).not.toContain('Niveau;TypeLigne');
	expect(platTexte).toContain('1900');

	// La remise en place laisse la vue par défaut pour les specs suivantes.
	req = page.waitForRequest((r) => BALANCE.test(r.url()));
	await page.getByTestId('grouping-shown').check();
	await req;
	await page.getByTestId('grouping-open-all').click();
});
