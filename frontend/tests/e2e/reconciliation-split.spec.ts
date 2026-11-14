/**
 * Story 8-5a-bis T7 — Tests E2E Playwright pour l'éclatement de transaction
 * (FR48).
 *
 * Deux scénarios d'origine :
 *   1. split button structure visible / empty state stable.
 *   2. accessibility axe scan zero violations sur /reconciliation.
 *
 * Story 36-2 (#1150, #1151) — le bloc `R0` joue l'éclatement **depuis l'écran**, sur une vraie
 * transaction en attente (relevé saisi par l'API, patron de `reconciliation-manual-entry.spec.ts`) :
 * comptes choisis dans la liste, montants tapés au clavier, clic sur *Éclater*, écriture relue
 * par l'API. C'est le seul test qui fait traverser la frontière HTTP à un éclatement saisi à
 * l'écran — les tests Rust (`reconciliation_split_e2e.rs`) envoient leurs montants en chaînes
 * et ne voient pas ce que le dialogue envoie. Chaque variante **relève** l'état observé (ligne
 * `R0 <variante> …` dans la sortie et en pièce jointe) avant d'asserter l'attendu ; le bloc se
 * rejoue tel quel contre une image publiée (`--grep R0`), d'où des localisateurs présents en
 * 0.17.6 (fiche 36-2, D1).
 *
 * Pré-requis :
 *   - MariaDB up + KESH_TEST_MODE=true.
 *   - Onboarding avec au moins 1 bank_account.
 *   - Playwright browsers (Ubuntu 26.04 : PLAYWRIGHT_HOST_PLATFORM_OVERRIDE=ubuntu24.04-x64).
 *   - Captures : `KESH_E2E_CAPTURES_DIR` (sinon la sortie du test), préfixe `KESH_R0_TAG`.
 */

import { expect, test, type Locator, type Page, type TestInfo } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';
import path from 'path';
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

test('reconciliation page exposes the split button structure', async ({
	page,
}) => {
	await login(page);
	await page.goto('/reconciliation');

	// Le sélecteur de compte bancaire OU l'état "no account" doit apparaître.
	const accountSelect = page.getByTestId('bank-account-select');
	const noAccount = page.getByTestId('reconciliation-no-account');
	await expect(accountSelect.or(noAccount)).toBeVisible();

	if (await accountSelect.isVisible()) {
		// Empty state attendu sur compte fraîchement seedé. Le bouton
		// « Éclater » est rendu par row tx pending — sans fixtures de tx
		// pending, aucun bouton ne sera visible mais le composant doit
		// avoir monté correctement (smoke test : pas d'erreur JS, état
		// empty stable, cohérent avec reconciliation-manual.spec.ts).
		await expect(page.getByTestId('reconciliation-empty')).toBeVisible({
			timeout: 5000,
		});
		// Aucune row → aucun bouton split. C'est attendu.
		await expect(page.getByTestId('split-button')).toHaveCount(0);
	}
});

test('reconciliation page with split modal accessibility — axe scan zero violations', async ({
	page,
}) => {
	await login(page);
	await page.goto('/reconciliation');
	await page
		.getByTestId('bank-account-select')
		.or(page.getByTestId('reconciliation-no-account'))
		.first()
		.waitFor();
	const results = await new AxeBuilder({ page })
		.withTags(['wcag2a', 'wcag2aa'])
		.analyze();
	expect(results.violations).toEqual([]);
});

// ---------------------------------------------------------------------------------------------
// Story 36-2 — R0 : l'éclatement joué depuis l'écran (#1150, #1151).
// ---------------------------------------------------------------------------------------------

const R0_IBAN = 'CH4431999123000889012';
/** Préfixe des captures : `tete`, `0176`, `apres`… (`KESH_R0_TAG`). */
const R0_TAG = process.env.KESH_R0_TAG ?? 'local';

type Compte = { id: number; number: string };

/** Centimes → `NNN.NN`. */
const enFrancs = (cents: number): string => (cents / 100).toFixed(2);

/** Un nombre de centimes propre à l'exécution, entre `min` et `min + span`. */
function centimesUniques(min: number, span = 30000): number {
	return min + ((Date.now() + Math.floor(Math.random() * 997)) % span);
}

function r0Capture(testInfo: TestInfo, name: string): string {
	const dossier = process.env.KESH_E2E_CAPTURES_DIR;
	const fichier = `r0-${R0_TAG}-${name}.png`;
	return dossier ? path.join(dossier, fichier) : testInfo.outputPath(fichier);
}

/**
 * Le message de `clientError` (`split-client-error`, fiche 36-2, D4). R0 s'est joué avant D4,
 * à la tête et contre 0.17.6, en localisant ce message par sa boîte — la seule
 * `div.rounded-md.border-destructive` du dialogue sans `data-testid` (D1, C-36-2-10) ; depuis
 * D4, par son `testid`, sans quoi le retrait du `testid` (mutation M4) passerait inaperçu.
 */
function messageClient(modal: Locator): Locator {
	return modal.getByTestId('split-client-error');
}

function champCompte(modal: Locator, ligne: number): Locator {
	return modal.getByTestId(`split-row-${ligne}`).locator('input[aria-autocomplete="list"]');
}

/**
 * Les options de la liste du champ de compte d'une ligne — bornées à sa `listbox` : le
 * sélecteur de projet de la ligne porte lui aussi des `<option>` (rôle `option`).
 */
function optionsDeCompte(modal: Locator, ligne: number): Locator {
	return modal.getByTestId(`split-row-${ligne}`).getByRole('listbox').getByRole('option');
}

/** Choisit le compte `numero` dans la liste, à la souris (`mousedown` sur l'option). */
async function choisirCompte(modal: Locator, ligne: number, numero: string): Promise<void> {
	const champ = champCompte(modal, ligne);
	await champ.click();
	await champ.fill(numero);
	const option = optionsDeCompte(modal, ligne).filter({ hasText: numero });
	await expect(option.first()).toBeVisible();
	await option.first().dispatchEvent('mousedown');
	await expect(champ).toHaveValue(new RegExp(`^${numero}`));
}

/**
 * Tape un montant au clavier (`pressSequentially`, jamais `fill`, qui refuse un texte non
 * numérique sur un `input[type=number]` et masquerait ce que le navigateur en fait).
 */
async function taperMontant(modal: Locator, ligne: number, texte: string): Promise<void> {
	const champ = modal.getByTestId(`split-amount-${ligne}`);
	await champ.fill('');
	if (texte !== '') await champ.pressSequentially(texte);
}

type Releve = { bouton: 'actif' | 'grisé'; message: string; indicateur: string; ligne: string };

/**
 * Relève l'état du dialogue sur une ligne `R0 <variante> bouton=… valeurs=… message=…
 * indicateur=… mesures=…`, écrite dans la sortie et jointe au test.
 */
async function relever(
	modal: Locator,
	testInfo: TestInfo,
	variante: string,
	mesures = '',
): Promise<Releve> {
	const bouton = (await modal.getByTestId('split-submit').isEnabled()) ? 'actif' : 'grisé';
	const valeurs: Array<Record<string, unknown>> = [];
	for (let i = 0; ; i++) {
		const montant = modal.getByTestId(`split-amount-${i}`);
		if ((await montant.count()) === 0) break;
		const lu = await montant.evaluate((el) => {
			const input = el as HTMLInputElement;
			return { type: input.type, texte: input.value, illisible: input.validity.badInput };
		});
		valeurs.push({ compte: await champCompte(modal, i).inputValue(), montant: lu });
	}
	const boite = messageClient(modal);
	const message = (await boite.count()) > 0 ? (await boite.innerText()).trim() : 'aucun';
	const indicateur = (await modal.getByTestId('split-balance-indicator').innerText()).trim();
	const ligne = `R0 ${variante} bouton=${bouton} valeurs=${JSON.stringify(valeurs)} message=${message} indicateur=${indicateur} mesures=${mesures}`;
	console.log(ligne);
	await testInfo.attach(`R0 ${variante}`, { body: ligne, contentType: 'text/plain' });
	return { bouton, message, indicateur, ligne };
}

/**
 * Le critère du grisé de la table B (fiche 36-2, V4-3) : chaque ligne montre un compte, les
 * montants tapés (admis par construction de la variante) font la somme, et le bouton est grisé.
 */
async function griseReproduit(modal: Locator, releve: Releve, nLignes: number): Promise<boolean> {
	for (let i = 0; i < nLignes; i++) {
		if ((await champCompte(modal, i).inputValue()).trim() === '') return false;
	}
	return releve.bouton === 'grisé';
}

type Transaction = {
	modal: Locator;
	ligne: Locator;
	amount: string;
	importId: number;
	txId: number;
	comptes: Compte[];
};

/**
 * Un relevé d'une ligne de débit de `cents`, saisi par l'API ; la page *Réconciliation* ouverte
 * sur son compte et le dialogue *Éclater* ouvert sur sa ligne. Nettoyage en `finally` :
 * rapprochement annulé s'il a eu lieu, import supprimé.
 */
async function avecEclatement(
	page: Page,
	cents: number,
	libelle: string,
	corps: (t: Transaction) => Promise<void>,
): Promise<void> {
	await login(page);
	const amount = enFrancs(cents);
	const unique = `${Date.now()}${Math.floor(Math.random() * 1000)}`;
	const bankAccountId = await wiredBankAccountViaApi(page, R0_IBAN);
	const ctx = await authedApiContext(page);
	let importId: number | null = null;
	let txId: number | null = null;
	try {
		const comptes = (await (await ctx.get('/api/v1/accounts')).json()) as Compte[];
		importId = await createManualDebitStatementViaApi(page, {
			bankAccountId,
			bookingDate: todayInZurich(),
			amount,
			counterpartyName: `Éclatement 36-2 ${unique}`,
			details: `${libelle} ${unique}`,
		});
		const detail = (await (await ctx.get(`/api/v1/bank-imports/${importId}`)).json()) as {
			transactions: Array<{ id: number }>;
		};
		expect(detail.transactions).toHaveLength(1);
		txId = detail.transactions[0].id;

		await page.goto('/reconciliation');
		await page.getByTestId('bank-account-select').selectOption(bankAccountId.toString());
		const ligne = page.locator(`[data-testid="reconciliation-row"][data-tx-id="${txId}"]`);
		await expect(ligne).toBeVisible({ timeout: 10000 });
		await page.locator(`[data-testid="split-button"][data-tx-id="${txId}"]`).click();
		const modal = page.getByTestId('transaction-split-modal');
		await expect(modal).toBeVisible();
		await corps({ modal, ligne, amount, importId, txId, comptes });
	} finally {
		if (txId !== null) {
			await ctx.post(`/api/v1/reconciliation/transactions/${txId}/cancel`, { data: {} });
		}
		if (importId !== null) await ctx.delete(`/api/v1/bank-imports/${importId}`);
		await disposeContextSafe(ctx);
	}
}

/**
 * Clique sur *Éclater* et relève l'issue : `succès` (le dialogue se ferme) ou le texte de
 * l'erreur d'envoi (`split-error`).
 */
async function eclater(page: Page, modal: Locator, testInfo: TestInfo, variante: string): Promise<string> {
	await modal.getByTestId('split-submit').click();
	const erreur = modal.getByTestId('split-error');
	await expect
		.poll(async () => (await modal.isHidden()) || (await erreur.isVisible()), { timeout: 10000 })
		.toBe(true);
	const issue = (await modal.isHidden()) ? 'succès' : `erreur: ${(await erreur.innerText()).trim()}`;
	const ligne = `R0 ${variante} envoi=${issue}`;
	console.log(ligne);
	await testInfo.attach(`R0 ${variante} envoi`, { body: ligne, contentType: 'text/plain' });
	if (issue !== 'succès') await page.screenshot({ path: r0Capture(testInfo, `${variante}-envoi`) });
	return issue;
}

/** L'écriture rapprochée de `txId`, lue par l'API : les montants portés par chaque compte. */
async function montantsDeLEcriture(
	page: Page,
	importId: number,
	txId: number,
): Promise<Map<number, string>> {
	const ctx = await authedApiContext(page);
	try {
		const apres = (await (await ctx.get(`/api/v1/bank-imports/${importId}`)).json()) as {
			transactions: Array<{ id: number; matchedEntryId: number | null }>;
		};
		const entryId = apres.transactions.find((t) => t.id === txId)?.matchedEntryId;
		expect(entryId, 'la transaction est rapprochée d’une écriture').toBeTruthy();
		const entry = (await (await ctx.get(`/api/v1/journal-entries/${entryId}`)).json()) as {
			lines: Array<{ accountId: number; debit: string; credit: string }>;
		};
		const parCompte = new Map<number, string>();
		for (const l of entry.lines) {
			const montant = Number(l.debit) !== 0 ? l.debit : l.credit;
			parCompte.set(l.accountId, Number(montant).toFixed(2));
		}
		return parCompte;
	} finally {
		await disposeContextSafe(ctx);
	}
}

function compte(comptes: Compte[], numero: string): Compte {
	const c = comptes.find((a) => a.number === numero);
	expect(c, `compte ${numero} du préréglage`).toBeTruthy();
	return c!;
}

/** Un éclatement complet et juste passe, et l'écriture porte les montants attendus. */
async function eclaterEtVerifier(
	page: Page,
	t: Transaction,
	testInfo: TestInfo,
	variante: string,
	attendus: Array<[string, string]>,
): Promise<void> {
	const issue = await eclater(page, t.modal, testInfo, variante);
	expect(issue, `${variante} : l'éclatement passe depuis l'écran`).toBe('succès');
	await expect(t.ligne).toHaveCount(0);
	// AC10 : l'état « succès » — dialogue fermé, la ligne a quitté la liste.
	await page.screenshot({ path: r0Capture(testInfo, `${variante.replace(/[()′]/g, '')}-succes`) });
	const montants = await montantsDeLEcriture(page, t.importId, t.txId);
	for (const [numero, montant] of attendus) {
		expect(montants.get(compte(t.comptes, numero).id), `${variante} : compte ${numero}`).toBe(montant);
	}
}

test.describe('R0 — l’éclatement joué depuis l’écran (#1150, #1151)', () => {
	test('R0 (a) comptes choisis à la souris, montants justes : l’éclatement passe', async ({
		page,
	}, testInfo) => {
		const cents = centimesUniques(20000);
		const premier = Math.floor(cents / 2);
		await avecEclatement(page, cents, 'R0 (a)', async (t) => {
			await choisirCompte(t.modal, 0, '4000');
			await choisirCompte(t.modal, 1, '2000');
			await taperMontant(t.modal, 0, enFrancs(premier));
			await taperMontant(t.modal, 1, enFrancs(cents - premier));
			const r = await relever(t.modal, testInfo, '(a)');
			console.log(`R0 (a) grise_reproduit=${await griseReproduit(t.modal, r, 2)}`);
			await t.modal.screenshot({ path: r0Capture(testInfo, 'a-equilibre') });
			// AC3 : l'état équilibré dit le montant, lu au catalogue (tue M2-bis, M2-ter).
			expect.soft(r.indicateur).toBe(`Balance équilibrée : ${t.amount}`);
			expect.soft(r.indicateur).not.toContain('{ $');
			expect.soft(r.bouton).toBe('actif');
			await eclaterEtVerifier(page, t, testInfo, '(a)', [
				['4000', enFrancs(premier)],
				['2000', enFrancs(cents - premier)],
			]);
		});
	});

	test('R0 (b) compte tapé puis Tab : le champ se vide et le message le dit', async ({
		page,
	}, testInfo) => {
		const cents = centimesUniques(20000);
		const premier = Math.floor(cents / 2);
		await avecEclatement(page, cents, 'R0 (b)', async (t) => {
			const champ = champCompte(t.modal, 0);
			await champ.click();
			await champ.pressSequentially('4000');
			await champ.press('Tab');
			await choisirCompte(t.modal, 1, '2000');
			await taperMontant(t.modal, 0, enFrancs(premier));
			await taperMontant(t.modal, 1, enFrancs(cents - premier));
			const r = await relever(t.modal, testInfo, '(b)');
			console.log(`R0 (b) grise_reproduit=${await griseReproduit(t.modal, r, 2)}`);
			await t.modal.screenshot({ path: r0Capture(testInfo, 'b-compte-tape') });
			expect.soft(await champ.inputValue(), 'AC7 : le champ quitté sans choisir est vide').toBe('');
			expect.soft(r.message).toBe('Ligne 1 : compte requis');
			expect.soft(r.bouton).toBe('grisé');
			await expect(messageClient(t.modal)).toBeInViewport();
		});
	});

	test('R0 (c) montants à virgule décimale (70,00) : lus, l’éclatement passe', async ({
		page,
	}, testInfo) => {
		const cents = centimesUniques(20000);
		const complement = enFrancs(cents - 7000).replace('.', ',');
		await avecEclatement(page, cents, 'R0 (c)', async (t) => {
			await choisirCompte(t.modal, 0, '4000');
			await choisirCompte(t.modal, 1, '2000');
			await taperMontant(t.modal, 0, '70,00');
			await taperMontant(t.modal, 1, complement);
			const r = await relever(t.modal, testInfo, '(c)');
			console.log(`R0 (c) grise_reproduit=${await griseReproduit(t.modal, r, 2)}`);
			await t.modal.screenshot({ path: r0Capture(testInfo, 'c-virgule') });
			expect.soft(r.bouton).toBe('actif');
			await eclaterEtVerifier(page, t, testInfo, '(c)', [
				['4000', '70.00'],
				['2000', enFrancs(cents - 7000)],
			]);
		});
	});

	test('R0 (d) apostrophe des milliers (1’200.50) : lue, l’éclatement passe', async ({
		page,
	}, testInfo) => {
		const reste = centimesUniques(1000, 20000);
		const cents = 120050 + reste;
		await avecEclatement(page, cents, 'R0 (d)', async (t) => {
			await choisirCompte(t.modal, 0, '4000');
			await choisirCompte(t.modal, 1, '2000');
			await taperMontant(t.modal, 0, "1'200.50");
			await taperMontant(t.modal, 1, enFrancs(reste));
			const r = await relever(t.modal, testInfo, '(d)');
			console.log(`R0 (d) grise_reproduit=${await griseReproduit(t.modal, r, 2)}`);
			await t.modal.screenshot({ path: r0Capture(testInfo, 'd-apostrophe') });
			expect.soft(r.bouton).toBe('actif');
			await eclaterEtVerifier(page, t, testInfo, '(d)', [
				['4000', '1200.50'],
				['2000', enFrancs(reste)],
			]);
		});
	});

	test('R0 (e) montants refusés : le bouton reste grisé et le message dit pourquoi', async ({
		page,
	}, testInfo) => {
		const cents = centimesUniques(20000);
		const cas: Array<[string, string, string]> = [
			['-70.00', 'Ligne 1 : montant illisible', 'signe'],
			['', 'Ligne 1 : montant > 0 requis', 'vide'],
			['0,00', 'Ligne 1 : montant > 0 requis', 'zero'],
			['1.234', 'Ligne 1 : montant ambigu', 'ambigu'],
			['0.005', 'Ligne 1 : deux décimales au plus', 'centime'],
		];
		await avecEclatement(page, cents, 'R0 (e)', async (t) => {
			await choisirCompte(t.modal, 0, '4000');
			await choisirCompte(t.modal, 1, '2000');
			await taperMontant(t.modal, 1, t.amount);
			for (const [saisie, attendu, nom] of cas) {
				await taperMontant(t.modal, 0, saisie);
				const r = await relever(t.modal, testInfo, `(e) « ${saisie} »`);
				await t.modal.screenshot({ path: r0Capture(testInfo, `e-${nom}`) });
				expect.soft(r.bouton, `« ${saisie} » : bouton`).toBe('grisé');
				expect.soft(r.message, `« ${saisie} » : message`).toContain(attendu);
				if (r.message !== 'aucun') {
					await expect.soft(messageClient(t.modal)).toBeInViewport();
				}
			}
		});
	});

	test('R0 (e′) 60.0000 : lu, et part au centime', async ({ page }, testInfo) => {
		const reste = centimesUniques(1000, 20000);
		const cents = 6000 + reste;
		await avecEclatement(page, cents, 'R0 (e′)', async (t) => {
			await choisirCompte(t.modal, 0, '4000');
			await choisirCompte(t.modal, 1, '2000');
			await taperMontant(t.modal, 0, '60.0000');
			await taperMontant(t.modal, 1, enFrancs(reste));
			const r = await relever(t.modal, testInfo, '(e′)');
			expect.soft(r.bouton).toBe('actif');
			await eclaterEtVerifier(page, t, testInfo, '(e′)', [
				['4000', '60.00'],
				['2000', enFrancs(reste)],
			]);
		});
	});

	test('R0 (f) somme fausse : l’indicateur et le message disent l’attendu et l’écart', async ({
		page,
	}, testInfo) => {
		const cents = centimesUniques(20000);
		await avecEclatement(page, cents, 'R0 (f)', async (t) => {
			await choisirCompte(t.modal, 0, '4000');
			await choisirCompte(t.modal, 1, '2000');
			await taperMontant(t.modal, 0, '10.00');
			await taperMontant(t.modal, 1, '20.00');
			const ecart = enFrancs(3000 - cents);
			const r = await relever(t.modal, testInfo, '(f)');
			await t.modal.screenshot({ path: r0Capture(testInfo, 'f-somme-fausse') });
			expect.soft(r.bouton).toBe('grisé');
			expect
				.soft(r.indicateur)
				.toBe(`Balance non équilibrée — attendu ${t.amount}, écart ${ecart}`);
			expect
				.soft(r.message)
				.toBe(
					`L'éclatement n'équilibre pas le montant de la transaction : attendu ${t.amount}, écart ${ecart}.`,
				);
			await expect(messageClient(t.modal)).toBeInViewport();
		});
	});

	test('R0 (g) largeur : les champs d’une ligne remplie se lisent (#1151)', async ({
		page,
	}, testInfo) => {
		// La colonne *Projet* n'apparaît que si un projet existe : créé avant d'ouvrir la page.
		await login(page);
		const api = await authedApiContext(page);
		try {
			const code = `R0-${Date.now().toString().slice(-6)}`;
			const res = await api.post('/api/v1/projects', {
				data: { code, name: 'Projet R0 36-2', parentId: null },
			});
			expect(res.ok(), `projet : ${res.status()}`).toBeTruthy();
		} finally {
			await disposeContextSafe(api);
		}
		const cents = 120050 + centimesUniques(1000, 20000);
		await avecEclatement(page, cents, 'R0 (g)', async (t) => {
			await expect(t.modal.getByTestId('split-line-project-0')).toBeVisible();
			await choisirCompte(t.modal, 0, '4000');
			await taperMontant(t.modal, 0, "1'200.50");
			await t.modal.getByTestId('split-description-0').fill('Fournitures de bureau, novembre');
			const champs: Array<[string, Locator]> = [
				['compte', champCompte(t.modal, 0)],
				['montant', t.modal.getByTestId('split-amount-0')],
				['description', t.modal.getByTestId('split-description-0')],
				['projet', t.modal.getByTestId('split-line-project-0')],
			];
			const mesures: string[] = [];
			const tronques: string[] = [];
			for (const [nom, champ] of champs) {
				const m = await champ.evaluate((el) => ({ s: el.scrollWidth, c: el.clientWidth }));
				mesures.push(`${nom}:${m.s}/${m.c}`);
				if (m.s > m.c) tronques.push(nom);
			}
			const boite = await t.modal.boundingBox();
			mesures.push(`dialogue:${Math.round(boite?.width ?? 0)}x${Math.round(boite?.height ?? 0)}`);
			await relever(t.modal, testInfo, '(g)', mesures.join(','));
			await t.modal.screenshot({ path: r0Capture(testInfo, 'g-largeur') });
			expect.soft(tronques, 'AC6 a : aucun champ tronqué').toEqual([]);
			await expect.soft(t.modal.getByTestId('split-submit')).toBeInViewport();
			await expect.soft(messageClient(t.modal)).toBeInViewport();
		});
	});

	test('R0 (h) six lignes : la liste des comptes de la dernière s’ouvre sans être coupée', async ({
		page,
	}, testInfo) => {
		const cents = centimesUniques(20000);
		await avecEclatement(page, cents, 'R0 (h)', async (t) => {
			// Le champ de compte de la ligne 1 a le focus à l'ouverture, et sa liste ouverte couvre
			// *Ajouter une ligne* : le quitter d'abord (clic sur le rappel de la transaction).
			await t.modal.getByTestId('split-tx-summary').click();
			for (let k = 0; k < 4; k++) await t.modal.getByTestId('split-add-line').click();
			await expect(t.modal.getByTestId('split-row-5')).toBeVisible();
			const champ = champCompte(t.modal, 5);
			await champ.focus();
			const premiere = optionsDeCompte(t.modal, 5).first();
			await expect(premiere).toBeVisible();
			const style = await t.modal.evaluate((el) => {
				const s = getComputedStyle(el);
				return { overflowY: s.overflowY, maxHeight: s.maxHeight };
			});
			const boite = await t.modal.boundingBox();
			const submit = await t.modal.getByTestId('split-submit').boundingBox();
			const vh = page.viewportSize()?.height ?? 720;
			const mesures = [
				`overflowY:${style.overflowY}`,
				`maxHeight:${style.maxHeight}`,
				`dialogue:${Math.round(boite?.y ?? 0)}+${Math.round(boite?.height ?? 0)}`,
				`submit_dans_fenetre:${submit !== null && submit.y + submit.height <= vh}`,
			].join(',');
			await relever(t.modal, testInfo, '(h) six lignes', mesures);
			await page.screenshot({ path: r0Capture(testInfo, 'h-six-lignes-liste') });
			await expect.soft(premiere, 'AC6 b : la première option se voit').toBeInViewport();
			expect.soft(style.overflowY, 'AC6 b : le dialogue ne défile pas').toBe('visible');
			expect.soft(style.maxHeight, 'AC6 b : le dialogue n’a pas de hauteur maximale').toBe('none');

			// Le seuil de #1181 : lignes ajoutées une à une jusqu'à ce que le bouton sorte (relevé).
			await t.modal.getByTestId('split-tx-summary').click();
			let seuil = 'jamais (50 lignes)';
			for (let n = 6; n <= 50; n++) {
				const b = await t.modal.getByTestId('split-submit').boundingBox();
				if (b === null || b.y + b.height > vh) {
					seuil = `${n} lignes`;
					break;
				}
				if (n < 50) await t.modal.getByTestId('split-add-line').click();
			}
			const ligne = `R0 (h) seuil_1181=${seuil}`;
			console.log(ligne);
			await testInfo.attach('R0 (h) seuil', { body: ligne, contentType: 'text/plain' });
		});
	});
});
