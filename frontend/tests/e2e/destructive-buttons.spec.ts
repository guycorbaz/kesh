/**
 * Story 28-1 (#640) — les boutons d'action destructive se voient.
 *
 * Recette visuelle (AC8) : le style **calculé** de la variante `destructive` du
 * `Button` partagé, mesuré dans Chromium, et une capture par état.
 *
 * - (a) `/design-system` (route publique) : le bouton destructif au repos,
 *   survolé, focalisé au clavier, et l'exemple désactivé (valeurs de l'AC3).
 * - (b) le détail d'un import : *Supprimer l'import* — le symptôme de la recette
 *   de la v0.13.2 — et son dialogue de confirmation, *Annuler* `outline` à côté de
 *   *Supprimer définitivement*, analysé par axe (`color-contrast`).
 *
 * Chaque scénario tourne deux fois, en clair et en sombre émulé (C-28-1-7 : la
 * variante n'a aucune classe `dark:` et doit rendre la même chose) — deux tests
 * distincts, pour qu'un échec en clair laisse tourner la passe sombre et ses
 * captures (C-28-1-29) ; le titre porte le schéma (Playwright refuse deux titres
 * identiques dans un fichier, C-28-1-36).
 *
 * ## Délais et lectures (C-28-1-19, C-28-1-32)
 *
 * La base du bouton porte `transition-all` (150 ms) : une valeur lue juste après
 * `hover()` ou le focus est intermédiaire. Toute assertion de VALEUR passe donc
 * par `softExpect(…).toHaveCSS(…)`, qui réessaie (2 s), et qui est `soft` : elle
 * rend le test rouge sans l'arrêter, si bien que la capture de l'état part quand
 * même. Les assertions de MONTAGE gardent le délai par défaut et ne sont pas
 * `soft`. Délai de test porté à 120 s : en phase rouge, les assertions de valeur
 * expirent l'une après l'autre.
 *
 * ## Captures (C-28-1-10, C-28-1-29, C-28-1-33)
 *
 * Prises dans le `finally` de chaque step, donc aussi quand une action lève, sous
 * le nom `<scénario>-<état>-<clair|sombre>.png`, dans `KESH_E2E_CAPTURES_DIR`
 * s'il est défini, sinon `testInfo.outputPath()`. Elles ne sont ni versionnées ni
 * comparées (pas de `toHaveScreenshot`) : ce sont les pièces de la recette
 * humaine. Les captures du bouton sont cadrées 8 px au-delà de sa boîte de
 * bordure : l'anneau `ring-3` et le contour `:focus-visible` (2 px décalés de
 * 2 px) en sont hors, et `locator.screenshot()` les rognerait.
 *
 * L'E2E ne recalcule aucun contraste (C-28-1-31) : c'est le rôle de la garde
 * Vitest `button-destructive.test.ts`, depuis `app.css`.
 *
 * Aucun sélecteur sur un libellé (KF-043, #326) : `data-testid` ou `data-slot`.
 * Données SYNTHÉTIQUES (fixture `csv_e2e_delete_import.csv`).
 *
 * Pré-requis : cf. `docs/testing.md` § « Prérequis Playwright local ».
 */

import { expect, test, type Locator, type Page, type TestInfo } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';
import path from 'path';
import { mkdirSync } from 'fs';
import { seedTestState, clearAuthStorage } from './helpers/test-state';
import {
	DELETE_IMPORT_FILENAME,
	importByScreen,
	login,
	mount,
} from './helpers/bank-import-fixtures';

test.describe.configure({ timeout: 120_000 });

/** Assertions de valeur : réessai 2 s (transition de 150 ms), sans arrêter le test. */
const softExpect = expect.configure({ soft: true, timeout: 2_000 });

const ROUGE = 'rgb(185, 28, 28)';
const ROUGE_SURVOL = 'rgb(153, 27, 27)';
const BLANC = 'rgb(255, 255, 255)';
const CONTOUR_FOCUS = 'rgb(59, 130, 246)';

type Scheme = 'light' | 'dark';
const SCHEMES: readonly Scheme[] = ['light', 'dark'] as const;
const SUFFIXE: Record<Scheme, string> = { light: 'clair', dark: 'sombre' };

/** Chemin d'une capture : `KESH_E2E_CAPTURES_DIR` s'il est défini, sinon le dossier du test. */
function cheminCapture(testInfo: TestInfo, nom: string): string {
	const dir = process.env.KESH_E2E_CAPTURES_DIR;
	if (dir) {
		mkdirSync(dir, { recursive: true });
		return path.join(dir, nom);
	}
	return testInfo.outputPath(nom);
}

/**
 * Capture d'un bouton, cadrée 8 px au-delà de sa boîte de bordure (bornée à 0
 * en haut et à gauche) pour que l'anneau et le contour de focus y figurent.
 * Une capture qui échoue est signalée en annotation, sans masquer l'erreur du
 * step qui l'a appelée depuis son `finally`.
 */
async function capturerBouton(
	page: Page,
	cible: Locator,
	testInfo: TestInfo,
	nom: string,
): Promise<void> {
	try {
		await cible.scrollIntoViewIfNeeded({ timeout: 2_000 });
		const box = await cible.boundingBox({ timeout: 2_000 });
		if (!box) throw new Error('boîte introuvable');
		const x = Math.max(0, box.x - 8);
		const y = Math.max(0, box.y - 8);
		await page.screenshot({
			path: cheminCapture(testInfo, nom),
			clip: { x, y, width: box.x + box.width + 8 - x, height: box.y + box.height + 8 - y },
		});
	} catch (e) {
		testInfo.annotations.push({ type: 'capture-manquee', description: `${nom} : ${String(e)}` });
	}
}

/** Capture d'un élément entier (le dialogue), à sa boîte. */
async function capturerElement(cible: Locator, testInfo: TestInfo, nom: string): Promise<void> {
	try {
		await cible.screenshot({ path: cheminCapture(testInfo, nom), timeout: 5_000 });
	} catch (e) {
		testInfo.annotations.push({ type: 'capture-manquee', description: `${nom} : ${String(e)}` });
	}
}

test.afterEach(async ({ page }) => {
	await clearAuthStorage(page);
});

test.describe('a — /design-system', () => {
	for (const scheme of SCHEMES) {
		test(`a — états du bouton destructif (${scheme})`, async ({ page }, testInfo) => {
			const s = SUFFIXE[scheme];
			await page.emulateMedia({ colorScheme: scheme });
			await page.goto('/design-system');
			const bouton = page.getByTestId('ds-destructive');
			const desactive = page.getByTestId('ds-destructive-disabled');
			await expect(bouton).toBeVisible();
			await expect(desactive).toBeVisible();

			await test.step('repos', async () => {
				try {
					await page.mouse.move(0, 0);
					await softExpect(bouton).toHaveCSS('background-color', ROUGE);
					await softExpect(bouton).toHaveCSS('color', BLANC);
				} finally {
					await capturerBouton(page, bouton, testInfo, `a-repos-${s}.png`);
				}
			});

			await test.step('survol', async () => {
				try {
					await bouton.hover();
					await softExpect(bouton).toHaveCSS('background-color', ROUGE_SURVOL);
					await softExpect(bouton).toHaveCSS('color', BLANC);
				} finally {
					await capturerBouton(page, bouton, testInfo, `a-survol-${s}.png`);
				}
			});

			await test.step('focus clavier', async () => {
				try {
					// Le pointeur d'abord éloigné : sinon le survol resterait actif (C-28-1-33).
					await page.mouse.move(0, 0);
					await bouton.focus();
					// Le focus doit venir du clavier pour que `:focus-visible` s'applique.
					await page.keyboard.press('Shift+Tab');
					await page.keyboard.press('Tab');
					await softExpect
						.poll(() => bouton.evaluate((el) => el.matches(':focus-visible')))
						.toBe(true);
					await softExpect(bouton).toHaveCSS('outline-style', 'solid');
					await softExpect(bouton).toHaveCSS('outline-color', CONTOUR_FOCUS);
					await softExpect(bouton).toHaveCSS('border-top-color', ROUGE);
					// Fond AU REPOS : preuve que le pointeur a quitté le bouton.
					await softExpect(bouton).toHaveCSS('background-color', ROUGE);
				} finally {
					await capturerBouton(page, bouton, testInfo, `a-focus-${s}.png`);
				}
			});

			await test.step('désactivé', async () => {
				try {
					await softExpect(desactive).toBeDisabled();
					await softExpect(desactive).toHaveCSS('opacity', '0.5');
					await softExpect(desactive).toHaveCSS('background-color', ROUGE);
				} finally {
					await capturerBouton(page, desactive, testInfo, `a-desactive-${s}.png`);
				}
			});
		});
	}
});

test.describe("b — Supprimer l'import", () => {
	// Base remise à zéro avant CHAQUE passe : un seul import du fichier par test,
	// donc jamais l'avertissement de doublon qui désactive *Confirmer*
	// (`BankImportUpload.svelte:345`, C-28-1-29).
	test.beforeEach(async () => {
		await seedTestState('with-company');
	});

	for (const scheme of SCHEMES) {
		test(`b — Supprimer l'import et son dialogue (${scheme})`, async ({ page }, testInfo) => {
			const s = SUFFIXE[scheme];
			await page.emulateMedia({ colorScheme: scheme });

			// Montage — délais par défaut, assertions dures.
			await login(page);
			await mount(page);
			await importByScreen(page);
			await page.getByTestId('bank-import-confirm').click();
			await expect(page.getByTestId('bank-import-preview')).toBeHidden();
			const row = page
				.getByTestId('bank-import-list-row')
				.filter({ hasText: DELETE_IMPORT_FILENAME });
			await expect(row).toHaveCount(1);
			await row.getByTestId('bank-import-list-link').click();
			await expect(page).toHaveURL(/\/bank-import\/\d+$/);

			const supprimer = page.getByTestId('bank-import-delete-button');
			await expect(supprimer).toBeVisible();

			await test.step('bouton', async () => {
				try {
					await page.mouse.move(0, 0);
					await softExpect(supprimer).toHaveCSS('background-color', ROUGE);
					await softExpect(supprimer).toHaveCSS('color', BLANC);
				} finally {
					await capturerBouton(page, supprimer, testInfo, `b-bouton-${s}.png`);
				}
			});

			const dialogue = page.getByTestId('bank-import-delete-dialog');
			await test.step('dialogue', async () => {
				try {
					await supprimer.click();
					await expect(dialogue).toBeVisible();
					const confirmer = page.getByTestId('bank-import-delete-submit');
					// *Annuler* n'a pas de data-testid : sélecteur structurel, sans libellé.
					const annuler = dialogue.locator(
						'[data-slot="dialog-footer"] [data-slot="button"]:not([data-testid])',
					);
					await expect(annuler).toHaveCount(1);
					await page.mouse.move(0, 0);
					await softExpect(confirmer).toHaveCSS('background-color', ROUGE);
					await softExpect(confirmer).toHaveCSS('color', BLANC);
					await softExpect(annuler).not.toHaveCSS('background-color', ROUGE);
					const axe = await new AxeBuilder({ page })
						.include('[data-testid="bank-import-delete-dialog"]')
						.withRules(['color-contrast'])
						.analyze();
					softExpect(axe.violations, JSON.stringify(axe.violations, null, 2)).toEqual([]);
				} finally {
					await capturerElement(dialogue, testInfo, `b-dialogue-${s}.png`);
				}
			});

			// Refermé par *Annuler* : rien n'est supprimé.
			await dialogue
				.locator('[data-slot="dialog-footer"] [data-slot="button"]:not([data-testid])')
				.click();
			await expect(dialogue).toBeHidden();
			await expect(supprimer).toBeVisible();
		});
	}
});
