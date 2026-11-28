/**
 * Story 46-3 (revue d'intégration de l'Epic 46, L4) — le fichier CSV téléchargé
 * depuis l'écran est celui que l'écran montre.
 *
 * ⚠️ Ce que seul ce test relie : `reports-groupes.spec.ts` ne lit que l'URL de
 * l'export, `reports_export_e2e.rs` que le contenu pour des paramètres écrits à
 * la main. Ici, bilan au niveau 1, groupe 19 ouvert à la main : le CONTENU du
 * fichier reçu est comparé aux lignes et aux sous-totaux de l'écran.
 */

import { readFile } from 'node:fs/promises';
import { expect, test } from '@playwright/test';
import { seedTestState, clearAuthStorage } from './helpers/test-state';
import { login, preparerArbre } from './helpers/reports-groupes';

test.beforeAll(async () => {
	await seedTestState('with-company');
});

test.afterEach(async ({ page }) => {
	await clearAuthStorage(page);
});

test('export CSV du bilan depuis l\'écran : le fichier suit la hiérarchie affichée (Story 46-3)', async ({
	page,
}) => {
	await login(page);
	await preparerArbre(page);
	// Story 55-1a (D16) — plus de *Générer* : le bilan se charge à l'ouverture de la page.
	let req = page.waitForRequest((r) => /reports\/balance-sheet\?/.test(r.url()));
	await page.goto('/reports');
	await req;
	await page.waitForLoadState('networkidle');

	// Niveau 1, puis ouverture de 19 à la main : 19 ouvert, 190 fermé par le niveau.
	req = page.waitForRequest((r) => /reports\/balance-sheet\?/.test(r.url()));
	await page.getByTestId('grouping-level').selectOption('1');
	await req;
	req = page.waitForRequest((r) => /reports\/balance-sheet\?/.test(r.url()));
	await page.getByTestId('group-toggle-19').click();
	await req;
	await expect(page.getByTestId('grouped-group-19')).toContainText('1’250.00');
	await expect(page.getByTestId('grouped-group-190')).toContainText('1’250.00');
	await expect(page.getByTestId('grouped-account-1900')).toHaveCount(0);

	const download = page.waitForEvent('download');
	await page.getByRole('button', { name: /export csv/i }).click();
	const path = await (await download).path();
	const texte = (await readFile(path, 'utf8')).replace(/^﻿/, '');
	const lignes = texte.split(/\r?\n/).filter((l) => l !== '');
	const cellules = lignes.map((l) => l.split(';'));

	// En-tête de la forme groupée (colonnes Niveau et TypeLigne).
	expect(lignes[0]).toMatch(/^Section;Niveau;TypeLigne;NumeroCompte;NomCompte;Solde/);

	// Les lignes de l'écran : 19 (niveau 1) et 190 (niveau 2) en groupes, chacun avec son sous-total.
	const groupe = (num: string) => cellules.find((c) => c[2] === 'groupe' && c[3] === num);
	expect(groupe('19'), 'ligne du groupe 19').toBeTruthy();
	expect(groupe('19')![1]).toBe('1');
	expect(groupe('19')![5]).toBe('1250.00');
	expect(groupe('190'), 'ligne du groupe 190').toBeTruthy();
	expect(groupe('190')![1]).toBe('2');
	expect(groupe('190')![5]).toBe('1250.00');

	// Ce que l'écran cache, le fichier ne le contient pas : ni 1900 ni 1910.
	expect(cellules.some((c) => c[3] === '1900' || c[3] === '1910')).toBe(false);
	expect(texte).not.toContain('Banque du test');

	// Le total de section garde sa forme (deux cellules vides de plus) et la valeur de l'écran.
	expect(cellules.every((c) => c.length >= 6)).toBe(true);
	const total = cellules.find((c) => c[0] === 'Total actifs');
	expect(total, 'ligne « Total actifs »').toBeTruthy();
	expect(total![5]).toBe('1250.00');
});
