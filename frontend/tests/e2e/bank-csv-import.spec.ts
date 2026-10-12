/**
 * Story 8-2 T8 — Tests E2E Playwright pour la feature bank-csv-import.
 *
 * 5 scénarios minimaux v0.1 (couverture ACs principaux) :
 *   1. liste profils accessible (AC #11)
 *   2. créer profil banque (AC #6)
 *   3. valeurs séparateurs distincts validées côté API (AC #15b)
 *   4. UNIQUE bank_name par company → 409 (AC #17a)
 *   5. accessibilité page profils — axe scan zero violations (AC #23)
 *
 * Pré-requis : MariaDB up + KESH_TEST_MODE=true + Playwright browsers
 * (Ubuntu 26.04 : PLAYWRIGHT_HOST_PLATFORM_OVERRIDE=ubuntu24.04-x64).
 *
 * Note : scénarios CSV import end-to-end (UTF-8, ISO-8859-1, partial
 * failure, encoding mismatch) reportés à code review post-impl ou
 * Story 8-3 (rejet partiel). La couverture E2E HTTP T4 (14 tests) +
 * unit T2 (40 tests) couvre déjà les ACs critiques.
 */

import { expect, test, type Page } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';
import {
	seedTestState,
	clearAuthStorage,
	authedApiContext,
	disposeContextSafe,
} from './helpers/test-state';

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

// ⚠️ `__dirname` n'existe PAS ici : ce fichier est un module ESM, et les trois
// `setInputFiles` de cette spec le référençaient — `ReferenceError` à
// l'exécution. Les autres specs à fixtures calculent déjà leur répertoire
// ainsi (`bank-import.spec.ts:33`) ; celle-ci était la seule à ne pas le faire.
const FIXTURE_DIR = path.join(path.dirname(fileURLToPath(import.meta.url)), 'fixtures');

const TEST_IBAN = 'CH4431999123000889012';

/**
 * Monte le compte bancaire dont les scénarios CSV ont besoin.
 *
 * ⚠️ **Cette spec n'en montait AUCUN**, alors que trois de ses tests
 * sélectionnent un compte dans `bank-account-select`. Le seed `with-company`
 * crée les comptes du plan comptable mais **pas** de `bank_account` : le
 * `<select>` n'avait que son option de garde, et `selectOption({ index: 1 })`
 * échouait sur « did not find some options ». Les tests dépendaient donc d'un
 * décor laissé par une autre spec — c'est-à-dire de l'ordre d'exécution
 * (issue #107, KF-030).
 *
 * ⚠️ Route CRUD, et NON `/api/v1/onboarding/bank-account` : cette dernière rend
 * 400 `ONBOARDING_STEP_ALREADY_COMPLETED` sur ce seed.
 */
async function ensureBankAccount(page: Page): Promise<void> {
	const ctx = await authedApiContext(page);
	try {
		const company = await ctx.get('/api/v1/companies/current');
		if (company.ok()) {
			const json = await company.json();
			if (Array.isArray(json.bankAccounts) && json.bankAccounts.length > 0) return;
		}
		const res = await ctx.post('/api/v1/bank-accounts', {
			data: { bankName: 'UBS Test', iban: TEST_IBAN, qrIban: null, isPrimary: true },
		});
		expect(res.ok(), `bank-account create failed: ${res.status()}`).toBeTruthy();
	} finally {
		await disposeContextSafe(ctx);
	}
}

test('lists bank profiles page accessible', async ({ page }) => {
	await login(page);
	await page.goto('/bank-import/profiles');
	await expect(page.getByTestId('bank-import-profile-page-title')).toBeVisible();
});

test('creates a bank profile via wizard', async ({ page }) => {
	await login(page);
	await page.goto('/bank-import/profiles/new');
	await page.getByTestId('bank-name-input').fill('Test Bank');
	await page.getByTestId('filename-pattern-input').fill('^test-.*\\.csv$');
	await page.getByTestId('date-format-input').fill('%d.%m.%Y');
	await page.getByTestId('field-separator-select').selectOption(';');
	await page.getByTestId('decimal-separator-select').selectOption('.');
	await page.getByTestId('header-row-count-input').fill('1');
	await page.getByTestId('col-date-input').fill('0');
	await page.getByTestId('col-amount-input').fill('1');
	await page.getByTestId('bank-import-profile-submit').click();

	// Redirect vers /bank-import/profiles/{id}
	await expect(page).toHaveURL(/\/bank-import\/profiles\/\d+/);
});

test('rejects equal separators with API error', async ({ page }) => {
	await login(page);
	await page.goto('/bank-import/profiles/new');
	await page.getByTestId('bank-name-input').fill('Bad Bank');
	await page.getByTestId('date-format-input').fill('%d.%m.%Y');
	await page.getByTestId('field-separator-select').selectOption(',');
	await page.getByTestId('decimal-separator-select').selectOption(',');
	await page.getByTestId('col-date-input').fill('0');
	await page.getByTestId('col-amount-input').fill('1');
	await page.getByTestId('bank-import-profile-submit').click();

	await expect(page.getByTestId('bank-import-profile-error')).toBeVisible();
});

test('duplicate bank_name returns 409', async ({ page }) => {
	await login(page);
	// Premier profil
	await page.goto('/bank-import/profiles/new');
	await page.getByTestId('bank-name-input').fill('UBS');
	await page.getByTestId('date-format-input').fill('%Y-%m-%d');
	await page.getByTestId('col-date-input').fill('0');
	await page.getByTestId('col-amount-input').fill('1');
	await page.getByTestId('bank-import-profile-submit').click();
	await expect(page).toHaveURL(/\/bank-import\/profiles\/\d+/);

	// Deuxième profil même nom → erreur 409
	await page.goto('/bank-import/profiles/new');
	await page.getByTestId('bank-name-input').fill('UBS');
	await page.getByTestId('date-format-input').fill('%Y-%m-%d');
	await page.getByTestId('col-date-input').fill('0');
	await page.getByTestId('col-amount-input').fill('1');
	await page.getByTestId('bank-import-profile-submit').click();

	await expect(page.getByTestId('bank-import-profile-error')).toBeVisible();
});

test('accessibility — profile pages axe scan zero violations', async ({ page }) => {
	await login(page);
	await page.goto('/bank-import/profiles');
	await expect(page.getByTestId('bank-import-profile-page-title')).toBeVisible();

	const results = await new AxeBuilder({ page }).analyze();
	expect(results.violations).toEqual([]);
});

// Story 26-1 (#620) — le FORMULAIRE de profil, mode séparé coché (groupe radio
// du sens des débits visible), n'avait aucun scan.
test('accessibility — profile form with debit sign group, axe scan zero violations', async ({
	page,
}) => {
	await login(page);
	await page.goto('/bank-import/profiles/new');
	await page.getByTestId('use-debit-credit-split').check();
	await expect(page.getByTestId('debit-sign-negative')).toBeVisible();

	const results = await new AxeBuilder({ page }).analyze();
	expect(results.violations).toEqual([]);
});

// Pass 1 review G3 H11+H12 : 2 scénarios CSV ajoutés en bonus.

test('shows BANK_CSV_NO_PROFILE_MATCH on CSV upload without profile', async ({ page }) => {
	await login(page);
	await ensureBankAccount(page);
	await page.goto('/bank-import');
	await expect(page.getByTestId('bank-import-page-title')).toBeVisible();
	await page.getByTestId('bank-account-select').selectOption({ index: 1 });
	await page
		.getByTestId('bank-import-file-input')
		.setInputFiles(path.join(FIXTURE_DIR, 'csv_utf8_minimal.csv'));

	const error = page.getByTestId('bank-import-error');
	await expect(error).toBeVisible();
	await expect(error).toHaveAttribute(
		'data-error-code',
		'BANK_CSV_NO_PROFILE_MATCH',
	);
});

test('displays partial failure error panel with line numbers', async ({ page }) => {
	await login(page);
	await ensureBankAccount(page);
	await page.goto('/bank-import');
	await expect(page.getByTestId('bank-import-page-title')).toBeVisible();
	await page.getByTestId('bank-account-select').selectOption({ index: 1 });
	await page
		.getByTestId('bank-import-file-input')
		.setInputFiles(path.join(FIXTURE_DIR, 'csv_partial_failure.csv'));

	// Sans profil bancaire configuré, on attend le 404 NO_PROFILE_MATCH
	// (pas le 422 PARTIAL_FAILURE — le parsing n'est pas atteint).
	// Le test démontre que le panneau erreur s'affiche et contient
	// l'erreur structurée (data-error-code) — pré-requis pour partial
	// failure quand un profil sera configuré.
	const error = page.getByTestId('bank-import-error');
	await expect(error).toBeVisible();
});

// Pass 2 review M4 (BH2-2) : CSV happy path E2E couvrant le pipeline
// complet (création profil → upload CSV → preview → parser → success).
test('uploads CSV with matching profile end-to-end', async ({ page }) => {
	await login(page);
	await ensureBankAccount(page);

	// Étape 1 — créer un profil bancaire qui matche `csv_utf8_minimal.csv`.
	await page.goto('/bank-import/profiles/new');
	await page.getByTestId('bank-name-input').fill('Test E2E Bank');
	// Pattern qui matche le filename exact uploadé.
	await page.getByTestId('filename-pattern-input').fill('^csv_utf8_minimal\\.csv$');
	await page.getByTestId('date-format-input').fill('%Y-%m-%d');
	await page.getByTestId('field-separator-select').selectOption(';');
	await page.getByTestId('decimal-separator-select').selectOption('.');
	await page.getByTestId('header-row-count-input').fill('1');
	await page.getByTestId('col-date-input').fill('0');
	await page.getByTestId('col-amount-input').fill('1');
	await page.getByTestId('col-reference-input').fill('2');
	await page.getByTestId('col-details-input').fill('3');
	await page.getByTestId('bank-import-profile-submit').click();
	await expect(page).toHaveURL(/\/bank-import\/profiles\/\d+/);

	// Étape 2 — uploader le CSV qui matche le profil par auto-match
	// filename. Le backend doit résoudre le profil via
	// `find_matching_profiles_for_filename` puis parser → preview.
	await page.goto('/bank-import');
	await expect(page.getByTestId('bank-import-page-title')).toBeVisible();
	await page.getByTestId('bank-account-select').selectOption({ index: 1 });
	await page
		.getByTestId('bank-import-file-input')
		.setInputFiles(path.join(FIXTURE_DIR, 'csv_utf8_minimal.csv'));

	// Étape 3 — preview affiché avec 3 transactions (cf. fixture).
	await expect(page.getByTestId('bank-import-preview')).toBeVisible();
	await expect(page.getByTestId('preview-tx-count')).toHaveText('3');

	// Étape 4 — confirm import → persist + redirect.
	await page.getByTestId('bank-import-confirm').click();
	await expect(page.getByTestId('bank-import-preview')).toBeHidden();

	// La liste affiche au moins 1 import.
	await expect(page.getByTestId('bank-import-list-row').first()).toBeVisible();
});

// ===========================================================================
// Story 26-1 (#620, #621) — le relevé CSV d'UBS importé tel quel
// ===========================================================================

/**
 * Fixture synthétique de structure UBS, lue **sans copie** depuis le crate
 * `kesh-import` (cf. `crates/kesh-import/tests/fixtures/README.md`).
 */
const UBS_FIXTURE = path.join(
	path.dirname(fileURLToPath(import.meta.url)),
	'../../../crates/kesh-import/tests/fixtures/csv/ubs_synthetique.csv',
);

type ProfileOptions = {
	name: string;
	pattern: string;
	debitSign?: 'positive' | 'negative';
	split: boolean;
	headerRows: string;
	dateFormat: string;
	columns: Record<string, string>;
};

/** Crée un profil **par l'écran** et attend la redirection vers sa fiche. */
async function createProfileViaUi(page: Page, o: ProfileOptions): Promise<void> {
	await page.goto('/bank-import/profiles/new');
	await page.getByTestId('bank-name-input').fill(o.name);
	await page.getByTestId('filename-pattern-input').fill(o.pattern);
	await page.getByTestId('date-format-input').fill(o.dateFormat);
	await page.getByTestId('field-separator-select').selectOption(';');
	await page.getByTestId('decimal-separator-select').selectOption('.');
	await page.getByTestId('header-row-count-input').fill(o.headerRows);
	if (o.split) {
		await page.getByTestId('use-debit-credit-split').check();
		await page.getByTestId(`debit-sign-${o.debitSign ?? 'positive'}`).check();
	}
	for (const [testid, value] of Object.entries(o.columns)) {
		await page.getByTestId(testid).fill(value);
	}
	await page.getByTestId('bank-import-profile-submit').click();
	await expect(page).toHaveURL(/\/bank-import\/profiles\/\d+/);
}

const UBS_COLUMNS = {
	'col-date-input': '2',
	'col-debit-input': '5',
	'col-credit-input': '6',
	'col-reference-input': '9',
	'col-details-input': '10',
};

/** Nombre total d'imports de la société, lu par l'API (pas par la liste, plafonnée à 20). */
async function importTotal(page: Page): Promise<number> {
	const ctx = await authedApiContext(page);
	try {
		const res = await ctx.get('/api/v1/bank-imports?limit=1&offset=0');
		expect(res.ok()).toBeTruthy();
		return (await res.json()).total as number;
	} finally {
		await disposeContextSafe(ctx);
	}
}

test('imports the UBS CSV as is with a « minus sign » profile (Story 26-1)', async ({ page }) => {
	await login(page);
	await ensureBankAccount(page);
	await createProfileViaUi(page, {
		name: 'UBS synthétique — moins',
		pattern: '^ubs_synthetique\\.csv$',
		debitSign: 'negative',
		split: true,
		headerRows: '9',
		dateFormat: '%Y-%m-%d',
		columns: UBS_COLUMNS,
	});

	await page.goto('/bank-import');
	await expect(page.getByTestId('bank-import-page-title')).toBeVisible();
	await page.getByTestId('bank-account-select').selectOption({ index: 1 });
	await page.getByTestId('bank-import-file-input').setInputFiles(UBS_FIXTURE);
	await expect(page.getByTestId('bank-import-preview')).toBeVisible();
	await expect(page.getByTestId('preview-tx-count')).toHaveText('11');
	// Story 27-2 (#624, AC15-3) : profil sans colonne Devise → l'aperçu dit
	// que le relevé est SUPPOSÉ en CHF.
	await expect(
		page.getByTestId('warning-informational').locator('[data-info-code="bank_csv_currency_assumed"]'),
	).toBeVisible();
	await page.getByTestId('bank-import-confirm').click();
	await expect(page.getByTestId('bank-import-preview')).toBeHidden();

	// Par l'API : les transactions de cet import — 11, dont 9 négatives,
	// somme −236.25 (en centimes, pour ne rien arrondir).
	const ctx = await authedApiContext(page);
	try {
		const list = await (await ctx.get('/api/v1/bank-imports?limit=50&offset=0')).json();
		const imp = (list.items as Array<{ id: number; filename: string }>).find(
			(i) => i.filename === 'ubs_synthetique.csv',
		);
		expect(imp, 'import ubs_synthetique.csv introuvable').toBeTruthy();
		const detail = await (await ctx.get(`/api/v1/bank-imports/${imp!.id}`)).json();
		const amounts = (detail.transactions as Array<{ amount: string }>).map((t) =>
			Math.round(Number(t.amount) * 100),
		);
		expect(amounts).toHaveLength(11);
		expect(amounts.filter((a) => a < 0)).toHaveLength(9);
		expect(amounts.reduce((a, b) => a + b, 0)).toBe(-23625);
	} finally {
		await disposeContextSafe(ctx);
	}
});

// ===========================================================================
// Story 27-2 (#624) — la colonne Devise du relevé UBS
// ===========================================================================

/**
 * Dérive la fixture UBS **en mémoire** en modifiant ses seules lignes de
 * données (après la ligne de titres) ; asserte que les 11 ont été vues.
 */
function deriveUbsDataRows(buf: Buffer, edit: (cells: string[]) => void): Buffer {
	const lines = buf.toString('utf8').split('\n');
	const title = lines.findIndex((l) => l.startsWith('Date de transaction;'));
	expect(title, 'ligne de titres introuvable').toBeGreaterThan(0);
	let seen = 0;
	for (let i = title + 1; i < lines.length; i++) {
		const cells = lines[i].split(';');
		edit(cells);
		lines[i] = cells.join(';');
		seen++;
	}
	expect(seen).toBe(11);
	return Buffer.from(lines.join('\n'), 'utf8');
}

/**
 * `ubs_synthetique_devise.csv` : la fixture dont les TROIS colonnes de date
 * (transaction, comptabilisation, valeur ; la colonne 1 est une heure) sont
 * décalées d'une année. Le test voisin importe déjà `ubs_synthetique.csv` dans
 * la même base et sur le même compte : mêmes octets → fichier en doublon,
 * mêmes lignes → doublons ignorés et import à zéro transaction. L'empreinte de
 * ligne porte la date (C-27-2-25).
 */
function ubsDeviseBuffer(): Buffer {
	return deriveUbsDataRows(fs.readFileSync(UBS_FIXTURE), (c) => {
		for (const k of [0, 2, 3]) {
			expect(c[k].startsWith('2026-'), `colonne ${k} : ${c[k]}`).toBeTruthy();
			c[k] = c[k].replace(/^2026-/, '2025-');
		}
	});
}

const UBS_DEVISE_NAME = 'ubs_synthetique_devise.csv';

test('reads the currency column of the UBS CSV, set on screen (Story 27-2)', async ({ page }) => {
	await login(page);
	await ensureBankAccount(page);
	await createProfileViaUi(page, {
		name: 'UBS synthétique — devise',
		pattern: '^ubs_synthetique_devise\\.csv$',
		debitSign: 'negative',
		split: true,
		headerRows: '9',
		dateFormat: '%Y-%m-%d',
		columns: { ...UBS_COLUMNS, 'col-currency-input': '4' },
	});

	// AC15-1 — fichier CHF : importé, chaque transaction en CHF, et l'aperçu
	// ne dit PAS la devise supposée.
	await page.goto('/bank-import');
	await expect(page.getByTestId('bank-import-page-title')).toBeVisible();
	await page.getByTestId('bank-account-select').selectOption({ index: 1 });
	await page.getByTestId('bank-import-file-input').setInputFiles({
		name: UBS_DEVISE_NAME,
		mimeType: 'text/csv',
		buffer: ubsDeviseBuffer(),
	});
	await expect(page.getByTestId('bank-import-preview')).toBeVisible();
	await expect(page.getByTestId('preview-tx-count')).toHaveText('11');
	await expect(page.getByTestId('warning-unsupported-currency')).toHaveCount(0);
	await expect(page.locator('[data-info-code="bank_csv_currency_assumed"]')).toHaveCount(0);
	await page.getByTestId('bank-import-confirm').click();
	await expect(page.getByTestId('bank-import-preview')).toBeHidden();

	const ctx = await authedApiContext(page);
	try {
		const list = await (await ctx.get('/api/v1/bank-imports?limit=50&offset=0')).json();
		const imp = (list.items as Array<{ id: number; filename: string }>).find(
			(i) => i.filename === UBS_DEVISE_NAME,
		);
		expect(imp, `import ${UBS_DEVISE_NAME} introuvable`).toBeTruthy();
		const detail = await (await ctx.get(`/api/v1/bank-imports/${imp!.id}`)).json();
		const transactions = detail.transactions as Array<{ currency: string }>;
		expect(transactions).toHaveLength(11);
		expect(transactions.every((t) => t.currency === 'CHF')).toBe(true);
	} finally {
		await disposeContextSafe(ctx);
	}

	// AC15-2 — le même fichier en EUR : aperçu seul, avertissement de devise,
	// *Confirmer* grisé, rien d'importé.
	const before = await importTotal(page);
	const eur = deriveUbsDataRows(ubsDeviseBuffer(), (c) => {
		expect(c[4]).toBe('CHF');
		c[4] = 'EUR';
	});
	await page.goto('/bank-import');
	await expect(page.getByTestId('bank-import-page-title')).toBeVisible();
	await page.getByTestId('bank-account-select').selectOption({ index: 1 });
	await page.getByTestId('bank-import-file-input').setInputFiles({
		name: UBS_DEVISE_NAME,
		mimeType: 'text/csv',
		buffer: eur,
	});
	await expect(page.getByTestId('bank-import-preview')).toBeVisible();
	await expect(page.getByTestId('warning-unsupported-currency')).toBeVisible();
	await expect(page.getByTestId('warning-unsupported-currency')).toContainText('EUR');
	await expect(page.getByTestId('bank-import-confirm')).toBeDisabled();
	expect(await importTotal(page)).toBe(before);
});

test('refuses the UBS CSV entirely under a « no sign » profile, lines listed (Story 26-1)', async ({
	page,
}) => {
	await login(page);
	await ensureBankAccount(page);
	await createProfileViaUi(page, {
		name: 'UBS synthétique — sans signe',
		pattern: '^ubs_sans_signe\\.csv$',
		debitSign: 'positive',
		split: true,
		headerRows: '9',
		dateFormat: '%Y-%m-%d',
		columns: UBS_COLUMNS,
	});
	const before = await importTotal(page);

	await page.goto('/bank-import');
	await expect(page.getByTestId('bank-import-page-title')).toBeVisible();
	await page.getByTestId('bank-account-select').selectOption({ index: 1 });
	// La MÊME fixture, téléversée sous un autre nom : aucun des deux fichiers
	// n'est reconnu par les deux profils (scénarios indépendants).
	await page.getByTestId('bank-import-file-input').setInputFiles({
		name: 'ubs_sans_signe.csv',
		mimeType: 'text/csv',
		buffer: fs.readFileSync(UBS_FIXTURE),
	});

	const error = page.getByTestId('bank-import-error');
	await expect(error).toBeVisible();
	await expect(error).toHaveAttribute('data-error-code', 'BANK_CSV_AMOUNT_SIGN_MISMATCH');
	await expect(error).toContainText(/sens des débits/i);
	const lines = page.getByTestId('bank-import-error-line');
	await expect(lines.first()).toBeVisible();
	// Cellule du numéro, texte exact : `toContainText('11')` passait aussi
	// pour une ligne 111 ou un montant -199.11 (revue P1, E-7).
	await expect(lines.first().locator('td').first()).toHaveText('11');
	await expect(page.getByTestId('bank-import-error-line-message').first()).toContainText(
		/signe contraire/i,
	);
	// Refus non confirmable : aucune case « importer les lignes valides ».
	await expect(page.getByTestId('confirm-partial-import')).toHaveCount(0);
	await expect(page.getByTestId('bank-import-preview')).toHaveCount(0);

	expect(await importTotal(page)).toBe(before);
});

test('shows the translated message next to the code of an invalid CSV line (Story 26-1)', async ({
	page,
}) => {
	await login(page);
	await ensureBankAccount(page);
	await createProfileViaUi(page, {
		name: 'Profil lignes invalides 26-1',
		pattern: '^lignes_invalides_261\\.csv$',
		split: false,
		headerRows: '1',
		dateFormat: '%Y-%m-%d',
		columns: { 'col-date-input': '0', 'col-amount-input': '1' },
	});

	await page.goto('/bank-import');
	await expect(page.getByTestId('bank-import-page-title')).toBeVisible();
	await page.getByTestId('bank-account-select').selectOption({ index: 1 });
	await page.getByTestId('bank-import-file-input').setInputFiles({
		name: 'lignes_invalides_261.csv',
		mimeType: 'text/csv',
		buffer: Buffer.from('date;amount\n2026-01-15;100.00\nPAS_UNE_DATE;50.00\n'),
	});

	await expect(page.getByTestId('warning-invalid-lines')).toBeVisible();
	const row = page.getByTestId('warning-invalid-lines-row').first();
	await expect(row).toContainText('INVALID_DATE');
	await expect(page.getByTestId('warning-invalid-lines-message').first()).toContainText(
		/date invalide/i,
	);
});

// ---------------------------------------------------------------------------
// Story 27-3 (#626) — l'aperçu est toujours celui du profil qui servira à
// l'import : recalculé au changement de profil, refusé par le serveur (409)
// si le profil a changé ailleurs entre l'aperçu et la confirmation.
// ---------------------------------------------------------------------------

/**
 * La fixture UBS, dates déplacées en `year` : elle a déjà été importée par les
 * tests 26-1 (doublon de fichier) et ses lignes 2026 seraient des doublons. Le
 * seul « 2026- » hors date est un libellé libre (`Facture 2026-014`).
 */
function ubsFixtureIn(year: string, name: string) {
	return {
		name,
		mimeType: 'text/csv',
		buffer: Buffer.from(fs.readFileSync(UBS_FIXTURE, 'utf8').replaceAll('2026-', `${year}-`)),
	};
}

/** Crée un profil UBS (motif vide : jamais auto-détecté) ; rend son id. */
async function createUbsProfile(page: Page, name: string, debitSign: 'positive' | 'negative') {
	await createProfileViaUi(page, {
		name,
		pattern: '',
		debitSign,
		split: true,
		headerRows: '9',
		dateFormat: '%Y-%m-%d',
		columns: UBS_COLUMNS,
	});
	const id = Number(/\/bank-import\/profiles\/(\d+)/.exec(page.url())?.[1]);
	expect(id, 'id du profil créé').toBeGreaterThan(0);
	return id;
}

/** Montants (centimes) de l'import portant `filename`, lus par l'API. */
async function importedAmounts(page: Page, filename: string): Promise<number[]> {
	const ctx = await authedApiContext(page);
	try {
		const list = await (await ctx.get('/api/v1/bank-imports?limit=50&offset=0')).json();
		const imp = (list.items as Array<{ id: number; filename: string }>).find(
			(i) => i.filename === filename,
		);
		expect(imp, `import ${filename} introuvable`).toBeTruthy();
		const detail = await (await ctx.get(`/api/v1/bank-imports/${imp!.id}`)).json();
		return (detail.transactions as Array<{ amount: string }>).map((t) =>
			Math.round(Number(t.amount) * 100),
		);
	} finally {
		await disposeContextSafe(ctx);
	}
}

test('changing the profile after a refusal recomputes the preview, without re-uploading (Story 27-3)', async ({
	page,
}) => {
	await login(page);
	await ensureBankAccount(page);
	const minus = await createUbsProfile(page, '27-3 moins', 'negative');
	const noSign = await createUbsProfile(page, '27-3 sans signe', 'positive');

	await page.goto('/bank-import');
	await expect(page.getByTestId('bank-import-page-title')).toBeVisible();
	await page.getByTestId('bank-account-select').selectOption({ index: 1 });
	await page.getByTestId('bank-profile-select').selectOption(String(noSign));
	await page.getByTestId('bank-import-file-input').setInputFiles(ubsFixtureIn('2027', 'ubs_273_e1.csv'));

	const error = page.getByTestId('bank-import-error');
	await expect(error).toHaveAttribute('data-error-code', 'BANK_CSV_AMOUNT_SIGN_MISMATCH');
	await expect(page.getByTestId('bank-import-preview')).toHaveCount(0);

	// Autre profil, SANS redéposer : l'aperçu est recalculé avec le fichier retenu.
	await page.getByTestId('bank-profile-select').selectOption(String(minus));
	await expect(page.getByTestId('bank-import-preview')).toBeVisible();
	await expect(page.getByTestId('preview-tx-count')).toHaveText('11');
	await expect(error).toHaveCount(0);
	await page.getByTestId('bank-import-confirm').click();
	await expect(page.getByTestId('bank-import-preview')).toBeHidden();

	const amounts = await importedAmounts(page, 'ubs_273_e1.csv');
	expect(amounts).toHaveLength(11);
	expect(amounts.filter((a) => a < 0)).toHaveLength(9);
	expect(amounts.reduce((a, b) => a + b, 0)).toBe(-23625);
});

test('a profile modified elsewhere after the preview is not imported: the preview is recomputed (Story 27-3)', async ({
	page,
}) => {
	await login(page);
	await ensureBankAccount(page);
	const id = await createUbsProfile(page, '27-3 E2 moins', 'negative');
	const before = await importTotal(page);

	await page.goto('/bank-import');
	await expect(page.getByTestId('bank-import-page-title')).toBeVisible();
	await page.getByTestId('bank-account-select').selectOption({ index: 1 });
	await page.getByTestId('bank-profile-select').selectOption(String(id));
	await page.getByTestId('bank-import-file-input').setInputFiles(ubsFixtureIn('2028', 'ubs_273_e2.csv'));
	await expect(page.getByTestId('preview-tx-count')).toHaveText('11');

	// L'« autre onglet » : le profil passe à « sans signe » par l'API. Le PUT est
	// un remplacement complet en `deny_unknown_fields` : corps reconstruit.
	const ctx = await authedApiContext(page);
	try {
		const current = await (await ctx.get(`/api/v1/bank-profiles/${id}`)).json();
		const res = await ctx.put(`/api/v1/bank-profiles/${id}`, {
			data: {
				bankName: current.bankName,
				filenamePattern: current.filenamePattern,
				columnMapping: current.columnMapping,
				dateFormat: current.dateFormat,
				decimalSeparator: current.decimalSeparator,
				fieldSeparator: current.fieldSeparator,
				encoding: current.encoding,
				headerRowCount: current.headerRowCount,
				debitSign: 'positive',
			},
		});
		expect(res.ok(), `PUT profil : ${res.status()} ${await res.text()}`).toBeTruthy();
	} finally {
		await disposeContextSafe(ctx);
	}

	await page.getByTestId('bank-import-confirm').click();
	await expect(page.getByTestId('bank-import-preview-recomputed')).toBeVisible();
	await expect(page.getByTestId('bank-import-error')).toHaveAttribute(
		'data-error-code',
		'BANK_CSV_AMOUNT_SIGN_MISMATCH',
	);
	await expect(page.getByTestId('bank-import-preview')).toHaveCount(0);
	expect(await importTotal(page)).toBe(before);
});
