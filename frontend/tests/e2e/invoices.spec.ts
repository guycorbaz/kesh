import { expect, test } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';
import { seedTestState, clearAuthStorage, authedApiContext, disposeContextSafe } from './helpers/test-state';
import {
	createContactWithAddressViaApi,
	ensurePrimaryBankAccountViaApi,
	createAndValidateInvoiceViaApi,
} from './helpers/api-fixtures';

test.beforeAll(async () => {
	await seedTestState('with-company');
});

test.afterEach(async ({ page }) => {
	// Clear localStorage after each test to prevent token bleed to next test
	await clearAuthStorage(page);
});

/**
 * Tests E2E — Factures brouillon (Story 5.1)
 *
 * Prérequis seed DB :
 * - admin bootstrap (admin / admin123)
 * - une `companies` active
 *
 * Les tests créent leurs propres contacts et factures avec suffixes uniques.
 */

async function login(page: import('@playwright/test').Page) {
	await page.goto('/login');
	await page.fill('#username', 'admin');
	await page.fill('#password', 'admin123');
	await page.click('button[type="submit"]');
	await expect(page).toHaveURL('/');
}

function uniq(prefix: string): string {
	return `${prefix} ${Date.now()}-${Math.floor(Math.random() * 1e6)}`;
}

async function createContactViaApi(page: import('@playwright/test').Page, name: string): Promise<number> {
	const ctx = await authedApiContext(page);
	try {
		const res = await ctx.post('/api/v1/contacts', {
			data: {
				contactType: 'Entreprise',
				name,
				isClient: true,
				isSupplier: false,
				defaultPaymentTerms: '30 jours net',
			},
		});
		expect(res.ok(), `createContactViaApi failed: ${res.status()}`).toBeTruthy();
		const json = await res.json();
		return json.id as number;
	} finally {
		await disposeContextSafe(ctx);
	}
}

async function createProductViaApi(
	page: import('@playwright/test').Page,
	name: string,
	unitPrice: string,
	vatRate: string,
): Promise<number> {
	const ctx = await authedApiContext(page);
	try {
		const res = await ctx.post('/api/v1/products', {
			data: { name, unitPrice, vatRate },
		});
		expect(res.ok(), `createProductViaApi failed: ${res.status()}`).toBeTruthy();
		const json = await res.json();
		return json.id as number;
	} finally {
		await disposeContextSafe(ctx);
	}
}

test.describe('Factures — liste', () => {
	test('affiche le titre et le bouton Nouvelle facture', async ({ page }) => {
		await login(page);
		await page.goto('/invoices');
		await expect(page.getByRole('heading', { name: 'Factures' })).toBeVisible();
		await expect(page.getByRole('button', { name: /Nouvelle facture/ })).toBeVisible();
	});

	// Note (D-6-1-D) : avec le seed E2E actuel (bootstrap admin seul), la liste
	// /invoices est vide → ce test axe valide uniquement l'empty state. Une fois
	// Story 6-4 (`seed_accounting_company`) en place, étendre pour couvrir l'état
	// peuplé (badges statut, contraste lignes de tableau, etc.).
	test('axe-core sans violations sur la liste factures (empty state)', async ({ page }) => {
		await login(page);
		await page.goto('/invoices');
		await page.waitForLoadState('networkidle');
		const results = await new AxeBuilder({ page }).analyze();
		expect(results.violations).toEqual([]);
	});
});

/**
 * Story 21-6a (#231, D10) — badge « suspendu » + filtre en liste factures.
 *
 * Avant cette story, suspendre une facture la faisait disparaître de la liste
 * à rappeler sans qu'aucune surface de lecture ne la signale : elle devenait
 * introuvable, donc impossible à réactiver.
 */
test.describe('Factures — suspension des rappels (21-6a)', () => {
	/** Suspend une facture via l'API 21-5a (le toggle UI arrive en 21-6c). */
	async function pauseInvoiceViaApi(
		page: import('@playwright/test').Page,
		invoiceId: number,
		note: string,
	): Promise<void> {
		const ctx = await authedApiContext(page);
		try {
			const get = await ctx.get(`/api/v1/invoices/${invoiceId}`);
			expect(get.ok(), `get invoice failed: ${get.status()}`).toBeTruthy();
			const { version } = await get.json();
			const res = await ctx.put(`/api/v1/invoices/${invoiceId}/dunning-pause`, {
				data: { version, note },
			});
			expect(res.ok(), `dunning-pause failed: ${res.status()}`).toBeTruthy();
		} finally {
			await disposeContextSafe(ctx);
		}
	}

	test('badge « suspendu » visible et filtre tri-état fonctionnel', async ({ page }) => {
		await login(page);
		await ensurePrimaryBankAccountViaApi(page);

		const pausedName = uniq('Suspendu SA');
		const activeName = uniq('Actif SA');
		const pausedContact = await createContactWithAddressViaApi(page, pausedName);
		const activeContact = await createContactWithAddressViaApi(page, activeName);
		const pausedInvoice = await createAndValidateInvoiceViaApi(page, pausedContact);
		await createAndValidateInvoiceViaApi(page, activeContact);

		await pauseInvoiceViaApi(page, pausedInvoice, 'litige en cours');

		// Sans filtre : les deux factures, et le badge signale la suspendue.
		await page.goto('/invoices');
		const pausedRow = page.locator('tbody tr', { hasText: pausedName });
		const activeRow = page.locator('tbody tr', { hasText: activeName });
		await expect(pausedRow).toBeVisible();
		await expect(activeRow).toBeVisible();
		await expect(pausedRow.getByTestId('invoice-paused-badge')).toBeVisible();
		await expect(activeRow.getByTestId('invoice-paused-badge')).toHaveCount(0);

		// La note de suspension est lisible en infobulle (seule surface visuelle en v1)…
		await expect(pausedRow.getByTestId('invoice-paused-badge')).toHaveAttribute(
			'title',
			/litige en cours/,
		);
		// …ET annoncée aux lecteurs d'écran. `title` seul ne suffit pas : son
		// annonce est notoirement peu fiable, et un `aria-label` réduit à
		// « Suspendu » priverait définitivement l'utilisateur non-voyant du
		// motif de la suspension. Choix délibéré, verrouillé ici (cf. Dev
		// Agent Record, déviation AC 18).
		await expect(pausedRow.getByTestId('invoice-paused-badge')).toHaveAttribute(
			'aria-label',
			/litige en cours/,
		);

		// Filtre « Suspendus » → la facture active disparaît.
		await page.getByTestId('invoice-paused-filter').selectOption('paused');
		await expect(pausedRow).toBeVisible();
		await expect(activeRow).toHaveCount(0);
		await expect(page).toHaveURL(/paused=paused/);

		// Filtre « Actifs » → la suspendue disparaît.
		await page.getByTestId('invoice-paused-filter').selectOption('not-paused');
		await expect(activeRow).toBeVisible();
		await expect(pausedRow).toHaveCount(0);
		await expect(page).toHaveURL(/paused=not-paused/);

		// Retour à « Tous » → défaut, param absent de l'URL.
		await page.getByTestId('invoice-paused-filter').selectOption('all');
		await expect(pausedRow).toBeVisible();
		await expect(activeRow).toBeVisible();
		await expect(page).not.toHaveURL(/paused=/);
	});

	test('le filtre survit à un rechargement (synchro URL bidirectionnelle)', async ({ page }) => {
		await login(page);
		await page.goto('/invoices?paused=paused');
		await expect(page.getByTestId('invoice-paused-filter')).toHaveValue('paused');
	});

	// Changer de filtre depuis une page > 1 doit ramener à la première page :
	// sinon l'offset survit à un jeu de résultats rétréci et l'utilisateur voit
	// une liste vide en croyant que le filtre est cassé.
	test('changer le filtre remet la pagination à zéro', async ({ page }) => {
		await login(page);
		await page.goto('/invoices?offset=20');
		await expect(page).toHaveURL(/offset=20/);
		await page.getByTestId('invoice-paused-filter').selectOption('paused');
		await expect(page).not.toHaveURL(/offset=/);
	});

	test('une valeur de filtre invalide dans l’URL retombe sur le défaut', async ({ page }) => {
		await login(page);
		await page.goto('/invoices?paused=bogus');
		// La whitelist client neutralise la valeur : aucun 400, retour à « all ».
		await expect(page.getByTestId('invoice-paused-filter')).toHaveValue('all');
	});

	// Étend le test axe historique de ce fichier, qui n'exerçait que l'empty
	// state (son commentaire demandait explicitement d'aller jusqu'à l'état
	// peuplé). Ce test a effectivement attrapé un défaut réel du badge 21-6a :
	// le patron `PaymentStatusBadge` colore le texte avec la MÊME variable que
	// la teinte de fond → 3.69:1, sous le minimum AA. Corrigé (11.4:1).
	//
	// `button-name` est désactivé : dette a11y PRÉ-EXISTANTE (#256) — les
	// boutons d'action des lignes sont icône seule, `aria-hidden`, sans
	// `aria-label`. Vérifié hors du diff 21-6a ; la règle de la story interdit
	// de corriger une dette pré-existante ici. RETIRER ce disableRules à la
	// fermeture de #256.
	test('axe-core sans violations sur la liste peuplée (badge suspendu)', async ({ page }) => {
		await login(page);
		await ensurePrimaryBankAccountViaApi(page);
		const name = uniq('A11y SA');
		const contact = await createContactWithAddressViaApi(page, name);
		const invoice = await createAndValidateInvoiceViaApi(page, contact);
		await pauseInvoiceViaApi(page, invoice, 'contrôle a11y');

		await page.goto('/invoices');
		await expect(page.getByTestId('invoice-paused-badge').first()).toBeVisible();
		await page.waitForLoadState('networkidle');
		const results = await new AxeBuilder({ page }).disableRules(['button-name']).analyze();
		expect(results.violations).toEqual([]);
	});
});

/**
 * Story 21-6c (#231) — Toggle de suspension + historique des rappels sur la
 * fiche facture. FRONTEND PUR (endpoints livrés 21-5a).
 *
 * Le scénario clé est l'**anti-régression du piège n°1** : pause/resume
 * renvoient un `DunningPauseResponse` (version incrémentée), pas la facture
 * entière — si la nouvelle version n'est pas ré-appliquée, la prochaine action
 * (ici « Marquer payée ») prend un 409. Le test le prouve bout-en-bout.
 */
test.describe('Factures — suspension & historique sur la fiche (21-6c)', () => {
	/** Enregistre un rappel manuel (papier) via l'API 21-5a — peuple l'historique. */
	async function recordManualReminderViaApi(
		page: import('@playwright/test').Page,
		invoiceId: number,
		levelNumber: number,
		note: string,
	): Promise<void> {
		const ctx = await authedApiContext(page);
		try {
			const today = new Date().toISOString().slice(0, 10);
			const res = await ctx.post(`/api/v1/invoices/${invoiceId}/reminders/manual`, {
				// #249 : sentAt = NaiveDateTime (jamais date nue).
				data: { levelNumber, sentAt: `${today}T12:00:00`, note },
			});
			expect(res.ok(), `manual reminder failed: ${res.status()}`).toBeTruthy();
		} finally {
			await disposeContextSafe(ctx);
		}
	}

	test('toggle suspension bout-en-bout + pas de 409 sur l’action suivante', async ({ page }) => {
		await login(page);
		await ensurePrimaryBankAccountViaApi(page);

		const contact = await createContactWithAddressViaApi(page, uniq('Toggle SA'));
		const invoiceId = await createAndValidateInvoiceViaApi(page, contact);

		await page.goto(`/invoices/${invoiceId}`);

		// État initial : active → bouton « Suspendre », pas de badge.
		await expect(page.getByTestId('invoice-paused-badge')).toHaveCount(0);
		await page.getByTestId('dunning-pause-button').click();

		// Modale : saisir un motif, confirmer.
		await page.getByTestId('dunning-pause-note').fill('litige client');
		await page.getByTestId('dunning-pause-confirm').click();

		// Badge « Suspendu » visible, motif en infobulle, bouton bascule « Reprendre ».
		await expect(page.getByTestId('invoice-paused-badge')).toBeVisible();
		await expect(page.getByTestId('invoice-paused-badge')).toHaveAttribute(
			'title',
			/litige client/,
		);
		await expect(page.getByTestId('dunning-resume-button')).toBeVisible();

		// Reprise directe (pas de modale, D-c1) → le badge disparaît. Prouve la
		// ré-application de version À TRAVERS pause→resume (chaque toggle rejoue
		// une version incrémentée : sans ré-application, le 2e appel prendrait 409).
		await page.getByTestId('dunning-resume-button').click();
		await expect(page.getByTestId('invoice-paused-badge')).toHaveCount(0);
		await expect(page.getByTestId('dunning-pause-button')).toBeVisible();

		// Re-suspendre, puis anti-régression piège n°1 : après pause (version
		// incrémentée serveur), une action version-portante D'UN AUTRE type DOIT
		// réussir sans 409 → « Marquer payée ».
		await page.getByTestId('dunning-pause-button').click();
		await page.getByTestId('dunning-pause-confirm').click();
		await expect(page.getByTestId('invoice-paused-badge')).toBeVisible();
		await page.getByRole('button', { name: 'Marquer payée' }).click();
		await page.getByRole('button', { name: 'Confirmer le paiement' }).click();
		// Succès prouvé : le bouton bascule en « Dé-marquer payée » (paidAt posé).
		await expect(page.getByRole('button', { name: 'Dé-marquer payée' })).toBeVisible();
		// Le badge de suspension persiste (la pause n'a pas été perdue).
		await expect(page.getByTestId('invoice-paused-badge')).toBeVisible();
		// BH1 (review P1) : facture payée = hors dunning → boutons Suspendre/Reprendre masqués.
		await expect(page.getByTestId('dunning-pause-button')).toHaveCount(0);
		await expect(page.getByTestId('dunning-resume-button')).toHaveCount(0);
	});

	test('reprise d’une facture supprimée entre-temps → retour liste (fiche fantôme)', async ({
		page,
	}) => {
		await login(page);
		await ensurePrimaryBankAccountViaApi(page);

		const contact = await createContactWithAddressViaApi(page, uniq('Fantôme SA'));
		const invoiceId = await createAndValidateInvoiceViaApi(page, contact);
		// Suspendre via l'API pour que le bouton « Reprendre » soit rendu.
		const ctxPause = await authedApiContext(page);
		try {
			const get = await ctxPause.get(`/api/v1/invoices/${invoiceId}`);
			const { version } = await get.json();
			const res = await ctxPause.put(`/api/v1/invoices/${invoiceId}/dunning-pause`, {
				data: { version, note: 'à supprimer' },
			});
			expect(res.ok()).toBeTruthy();
		} finally {
			await disposeContextSafe(ctxPause);
		}

		await page.goto(`/invoices/${invoiceId}`);
		await expect(page.getByTestId('dunning-resume-button')).toBeVisible();

		// La facture est supprimée par un autre acteur (autre onglet/utilisateur).
		const ctxDel = await authedApiContext(page);
		try {
			const del = await ctxDel.delete(`/api/v1/invoices/${invoiceId}`);
			expect(del.ok(), `delete failed: ${del.status()}`).toBeTruthy();
		} finally {
			await disposeContextSafe(ctxDel);
		}

		// Reprendre → 404 → retour liste (patron fiche fantôme, review P1 ECH1) :
		// pas de blocage sur une fiche introuvable.
		await page.getByTestId('dunning-resume-button').click();
		await expect(page).toHaveURL(/\/invoices$/);
	});

	test('suspension : une erreur transitoire garde la modale ouverte + préserve le motif', async ({
		page,
	}) => {
		await login(page);
		await ensurePrimaryBankAccountViaApi(page);

		const contact = await createContactWithAddressViaApi(page, uniq('Transient SA'));
		const invoiceId = await createAndValidateInvoiceViaApi(page, contact);

		await page.goto(`/invoices/${invoiceId}`);

		// Forcer un 500 transitoire sur le PUT dunning-pause (une seule fois).
		let intercepted = 0;
		await page.route('**/api/v1/invoices/*/dunning-pause', async (route) => {
			intercepted += 1;
			await route.fulfill({
				status: 500,
				contentType: 'application/json',
				body: JSON.stringify({ code: 'INTERNAL', message: 'boom' }),
			});
		});

		await page.getByTestId('dunning-pause-button').click();
		await page.getByTestId('dunning-pause-note').fill('motif précieux');
		await page.getByTestId('dunning-pause-confirm').click();

		// Review P1 ECH2 : la modale reste ouverte, l'erreur inline s'affiche, et le
		// motif tapé n'est PAS perdu (pas de fermeture + reset).
		expect(intercepted).toBeGreaterThan(0);
		await expect(page.getByTestId('dunning-pause-confirm')).toBeVisible();
		await expect(page.getByTestId('dunning-pause-note')).toHaveValue('motif précieux');
	});

	test('historique des rappels affiché (canal manuel, ligne visible)', async ({ page }) => {
		await login(page);
		await ensurePrimaryBankAccountViaApi(page);

		const contact = await createContactWithAddressViaApi(page, uniq('Histo SA'));
		const invoiceId = await createAndValidateInvoiceViaApi(page, contact);
		// Le manuel n'exige PAS une facture échue, seulement validée + non payée.
		await recordManualReminderViaApi(page, invoiceId, 1, 'recommandé A+');

		await page.goto(`/invoices/${invoiceId}`);

		const history = page.getByTestId('reminder-history');
		await expect(history).toBeVisible();
		const rows = history.getByTestId('reminder-history-row');
		await expect(rows).toHaveCount(1);
		await expect(rows.first()).toContainText('Manuel');
	});

	// axe scopé (AC 18) : sous-arbre de la fiche facture (badge suspendu +
	// historique + boutons Suspendre/Reprendre). `color-contrast` et `button-name`
	// désactivés = dettes PRÉ-EXISTANTES #253 (PaymentStatusBadge .unpaid) et #256
	// (boutons d'action icône seule) neutralisées — hors scope de cette story. Le
	// badge suspendu (AA 11.4:1) est couvert par le test axe non-neutralisé du
	// describe 21-6a ; les boutons de cette story portent un libellé texte.
	test('axe-core sans violations sur la fiche (badge suspendu + historique)', async ({ page }) => {
		await login(page);
		await ensurePrimaryBankAccountViaApi(page);

		const contact = await createContactWithAddressViaApi(page, uniq('Axe 6c SA'));
		const invoiceId = await createAndValidateInvoiceViaApi(page, contact);
		await recordManualReminderViaApi(page, invoiceId, 1, 'axe');

		await page.goto(`/invoices/${invoiceId}`);
		// Suspendre pour rendre le badge présent dans le sous-arbre audité.
		await page.getByTestId('dunning-pause-button').click();
		await page.getByTestId('dunning-pause-confirm').click();
		await expect(page.getByTestId('invoice-paused-badge')).toBeVisible();
		await expect(page.getByTestId('reminder-history')).toBeVisible();
		await page.waitForLoadState('networkidle');

		const results = await new AxeBuilder({ page })
			.include('[data-testid="invoice-detail"]')
			.disableRules(['color-contrast', 'button-name'])
			.analyze();
		expect(results.violations).toEqual([]);
	});
});

test.describe('Factures — création brouillon', () => {
	test('crée une facture avec une ligne libre et la persiste', async ({ page }) => {
		await login(page);
		const contactName = uniq('Client');
		await createContactViaApi(page, contactName);

		await page.goto('/invoices/new');
		await expect(page.getByRole('heading', { name: 'Nouvelle facture' })).toBeVisible();

		// Sélection du contact via le combobox
		// Strict-mode fix (KF #57 cascade-cleared) : `getByRole('combobox')` matche
		// 2 éléments (contact picker + VAT rate select `data-testid=invoice-line-vat-rate`).
		// Discriminer par placeholder du picker contact (cf. ContactPicker component).
		await page.getByRole('combobox', { name: /Rechercher un contact/ }).click();
		await page.getByRole('combobox', { name: /Rechercher un contact/ }).fill(contactName);
		await page.getByRole('option', { name: new RegExp(contactName) }).first().click();

		// Ligne libre par défaut : remplir description, quantité, prix
		const firstRow = page.locator('tbody tr').first();
		await firstRow.locator('input[type="text"]').first().fill('Conseil stratégique');
		const numericInputs = firstRow.locator('input[inputmode="decimal"]');
		await numericInputs.nth(0).fill('4.5');
		await numericInputs.nth(1).fill('200.00');

		await page.getByRole('button', { name: 'Créer la facture' }).click();
		// Depuis Story 5.x (review P5), la création redirige vers la vue DÉTAIL.
		await expect(page).toHaveURL(/\/invoices\/\d+$/);
		await expect(page.getByRole('heading', { name: 'Facture' })).toBeVisible();
		// Persistance : la facture apparaît dans la liste (nom du contact).
		await page.goto('/invoices');
		await expect(page.locator('tbody').getByText(contactName)).toBeVisible({ timeout: 5000 });
	});

	// Story 21-1 (#245) — pré-remplissage échéance + libellé depuis le délai
	// de paiement structuré du contact, à la sélection dans le formulaire.
	test('pré-remplit échéance et conditions depuis le délai du contact (#245)', async ({
		page
	}) => {
		await login(page);
		const contactName = uniq('Client PTD');
		// Contact avec délai structuré 30 jours (helper étendu Story 21-1).
		await createContactWithAddressViaApi(page, contactName, undefined, undefined, 30);

		await page.goto('/invoices/new');
		await expect(page.getByRole('heading', { name: 'Nouvelle facture' })).toBeVisible();

		// Avant sélection : échéance et conditions vides.
		await expect(page.locator('#invoice-due-date')).toHaveValue('');
		await expect(page.locator('#invoice-payment-terms')).toHaveValue('');

		await page.getByRole('combobox', { name: /Rechercher un contact/ }).click();
		await page.getByRole('combobox', { name: /Rechercher un contact/ }).fill(contactName);
		await page.getByRole('option', { name: new RegExp(contactName) }).first().click();

		// Échéance = date du jour + 30 (calcul UTC identique à addDaysIso).
		const today = new Date().toISOString().slice(0, 10);
		const m = /^(\d{4})-(\d{2})-(\d{2})$/.exec(today)!;
		const expectedDue = new Date(Date.UTC(Number(m[1]), Number(m[2]) - 1, Number(m[3]) + 30))
			.toISOString()
			.slice(0, 10);
		await expect(page.locator('#invoice-due-date')).toHaveValue(expectedDue);
		// Libellé serveur (langue contact héritée = FR sur le seed E2E).
		await expect(page.locator('#invoice-payment-terms')).toHaveValue('Payable à 30 jours net');

		// Le garde non-destructif du prefill (`!dueDate`/`!paymentTerms` dans
		// `onContactSelect`) n'est pas ré-exercé ici : la ré-ouverture du
		// ContactPicker après une sélection est une interaction E2E fragile
		// (le dropdown ne se rouvre pas de façon déterministe sur un contact
		// déjà sélectionné). Le garde est un simple conditionnel — couvert par
		// le fait qu'en édition de facture (`initialInvoice` peuplé) le prefill
		// ne tire jamais.
	});

	test('crée une facture avec projet analytique (Story 19-4)', async ({ page }) => {
		await login(page);
		const contactName = uniq('Client Projet');
		await createContactViaApi(page, contactName);

		// Projet actif via l'API (code unique par run).
		const code = `E2E194-${Date.now() % 1000000}`;
		const setupCtx = await authedApiContext(page);
		let projectId: number;
		try {
			const resp = await setupCtx.post('/api/v1/projects', {
				data: { parentId: null, code, name: 'Projet E2E 19-4', description: null, startDate: null, endDate: null }
			});
			expect(resp.ok()).toBeTruthy();
			projectId = (await resp.json()).id;
		} finally {
			await disposeContextSafe(setupCtx);
		}

		await page.goto('/invoices/new');
		await expect(page.getByRole('heading', { name: 'Nouvelle facture' })).toBeVisible();

		await page.getByRole('combobox', { name: /Rechercher un contact/ }).click();
		await page.getByRole('combobox', { name: /Rechercher un contact/ }).fill(contactName);
		await page.getByRole('option', { name: new RegExp(contactName) }).first().click();

		const firstRow = page.locator('tbody tr').first();
		await firstRow.locator('input[type="text"]').first().fill('Prestation projet');
		const numericInputs = firstRow.locator('input[inputmode="decimal"]');
		await numericInputs.nth(0).fill('1');
		await numericInputs.nth(1).fill('500.00');

		// Sélecteur projet document-level (arbre 2 niveaux).
		await page.getByTestId('invoice-project').selectOption({
			label: `${code} — Projet E2E 19-4`
		});

		// Capturer la réponse du POST pour vérifier la persistance ground-truth.
		const [resp] = await Promise.all([
			page.waitForResponse(
				(r) => r.url().includes('/api/v1/invoices') && r.request().method() === 'POST'
			),
			page.getByRole('button', { name: 'Créer la facture' }).click()
		]);
		expect(resp.ok()).toBeTruthy();
		const created: { id: number; projectId: number | null } = await resp.json();
		expect(created.projectId).toBe(projectId);

		// La vue détail affiche le libellé du projet.
		await page.goto(`/invoices/${created.id}`);
		await expect(page.getByTestId('invoice-project')).toHaveText(`${code} — Projet E2E 19-4`);
	});

	test('crée une facture avec 1 ligne libre + 1 ligne catalogue et persiste après reload (AC #1, #2)', async ({
		page,
	}) => {
		await login(page);
		const contactName = uniq('ClientCombo');
		const productName = uniq('Prod');
		await createContactViaApi(page, contactName);
		await createProductViaApi(page, productName, '150.00', '8.10');

		await page.goto('/invoices/new');

		// Contact
		// Strict-mode fix (KF #57 cascade-cleared) : `getByRole('combobox')` matche
		// 2 éléments (contact picker + VAT rate select `data-testid=invoice-line-vat-rate`).
		// Discriminer par placeholder du picker contact (cf. ContactPicker component).
		await page.getByRole('combobox', { name: /Rechercher un contact/ }).click();
		await page.getByRole('combobox', { name: /Rechercher un contact/ }).fill(contactName);
		await page.getByRole('option', { name: new RegExp(contactName) }).first().click();

		// Ligne libre (celle par défaut)
		const firstRow = page.locator('tbody tr').first();
		await firstRow.locator('input[type="text"]').first().fill('Prestation libre');
		const firstRowNumerics = firstRow.locator('input[inputmode="decimal"]');
		await firstRowNumerics.nth(0).fill('2');
		await firstRowNumerics.nth(1).fill('100.00');

		// Ligne depuis catalogue
		await page.getByRole('button', { name: /Depuis catalogue/ }).click();
		await expect(page.getByRole('dialog')).toBeVisible();
		await page.getByPlaceholder(/Rechercher un produit/).fill(productName);
		await page
			.getByRole('dialog')
			.getByRole('button')
			.filter({ hasText: productName })
			.first()
			.click();

		// Le formulaire doit maintenant contenir 2 lignes, la 2e pré-remplie
		await expect(page.locator('tbody tr')).toHaveCount(2);
		const secondRow = page.locator('tbody tr').nth(1);
		await expect(secondRow.locator('input[type="text"]').first()).toHaveValue(productName);
		const secondRowNumerics = secondRow.locator('input[inputmode="decimal"]');
		await expect(secondRowNumerics.nth(1)).toHaveValue('150.0000'); // prix catalogue à 4 décimales

		// Soumettre
		await page.getByRole('button', { name: 'Créer la facture' }).click();
		// Depuis Story 5.x (review P5), la création redirige vers la vue DÉTAIL.
		await expect(page).toHaveURL(/\/invoices\/\d+$/);

		// La persistance des lignes se vérifie sur la page d'ÉDITION (le
		// testid invoice-lines-table appartient au formulaire InvoiceForm).
		const detailUrl = page.url();
		await page.goto(`${detailUrl}/edit`);

		// Reload dur — l'état doit être identique. Scope à la table de lignes via testid
		// (un autre <tbody> peut exister sur la page : tableau récap, picker produits, etc.).
		await page.reload();
		const linesTable = page.locator('[data-testid="invoice-lines-table"]');
		await expect(linesTable).toBeVisible();
		// Les descriptions du formulaire vivent dans des <input> — asserter la
		// VALUE (toContainText ne voit pas les valeurs d'inputs).
		const rows = linesTable.locator('tbody tr');
		await expect(rows).toHaveCount(2);
		await expect(rows.nth(0).locator('input[type="text"]').first()).toHaveValue(
			'Prestation libre'
		);
		await expect(rows.nth(1).locator('input[type="text"]').first()).toHaveValue(productName);
	});
});

// ---------------------------------------------------------------------------
// Story 5.3 — Téléchargement PDF QR Bill
// ---------------------------------------------------------------------------

// Story 20-4 : helpers API extraits vers helpers/api-fixtures.ts (DRY —
// partagés avec invoice-send-email.spec.ts).
test.describe('Factures — téléchargement PDF (Story 5.3)', () => {
	test('télécharge le PDF d\'une facture validée (golden path)', async ({ page, context }) => {
		await login(page);
		await ensurePrimaryBankAccountViaApi(page);
		const contactId = await createContactWithAddressViaApi(page, uniq('PDF Client'));
		const invoiceId = await createAndValidateInvoiceViaApi(page, contactId);

		await page.goto(`/invoices/${invoiceId}`);
		await expect(page.getByRole('heading', { name: 'Facture' })).toBeVisible();

		// Intercepte l'appel direct à l'endpoint PDF (plus robuste que window.open).
		// Use authedApiContext (KF #54) — page.request.* n'a pas le Bearer header.
		const pdfCtx = await authedApiContext(page);
		try {
			const pdfRes = await pdfCtx.get(`/api/v1/invoices/${invoiceId}/pdf`);
			expect(pdfRes.status()).toBe(200);
			expect(pdfRes.headers()['content-type']).toContain('application/pdf');
			const buf = await pdfRes.body();
			expect(buf.slice(0, 7).toString('utf8')).toMatch(/^%PDF-1\./);
		} finally {
			await disposeContextSafe(pdfCtx);
		}
	});

	test('bouton visible uniquement si status=validated', async ({ page }) => {
		await login(page);
		const contactName = uniq('PDF Draft');
		await createContactWithAddressViaApi(page, contactName);
		// Facture brouillon non validée
		await page.goto('/invoices/new');
		// Strict-mode fix (KF #57 cascade-cleared) : `getByRole('combobox')` matche
		// 2 éléments (contact picker + VAT rate select `data-testid=invoice-line-vat-rate`).
		// Discriminer par placeholder du picker contact (cf. ContactPicker component).
		await page.getByRole('combobox', { name: /Rechercher un contact/ }).click();
		await page.getByRole('combobox', { name: /Rechercher un contact/ }).fill(contactName);
		await page.getByRole('option', { name: new RegExp(contactName) }).first().click();
		const firstRow = page.locator('tbody tr').first();
		await firstRow.locator('input[type="text"]').first().fill('Item');
		const inputs = firstRow.locator('input[inputmode="decimal"]');
		await inputs.nth(0).fill('1');
		await inputs.nth(1).fill('50');
		await page.getByRole('button', { name: 'Créer la facture' }).click();
		await expect(page).toHaveURL(/\/invoices\/\d+$/); // redirection détail (Story 5.x P5)

		// On est déjà sur le détail du brouillon → pas de bouton PDF.
		// NB : l'accessible name du bouton vient de son aria-label
		// (« Télécharger la facture N au format PDF »).
		await expect(page.getByRole('button', { name: /Télécharger.*PDF/i })).toHaveCount(0);
	});

	test('erreur 400 INVOICE_NOT_PDF_READY affichée comme toast', async ({ page }) => {
		// AC17 : le cas d'erreur INVOICE_NOT_PDF_READY doit s'afficher sous
		// forme de toast côté UI. Le backend E2E (`invoice_pdf_e2e.rs`) couvre
		// déjà la détection backend ; ici on vérifie que le frontend affiche
		// correctement l'erreur en interceptant la réponse du serveur.
		await login(page);
		const contactId = await createContactWithAddressViaApi(page, uniq('PDF Err'));
		const invoiceId = await createAndValidateInvoiceViaApi(page, contactId);

		// Intercepte l'appel PDF pour renvoyer un 400 INVOICE_NOT_PDF_READY.
		await page.route(`**/api/v1/invoices/${invoiceId}/pdf`, async (route) => {
			await route.fulfill({
				status: 400,
				contentType: 'application/json',
				body: JSON.stringify({
					error: {
						code: 'INVOICE_NOT_PDF_READY',
						message: "Aucun compte bancaire principal n'est configuré pour cette company.",
					},
				}),
			});
		});

		await page.goto(`/invoices/${invoiceId}`);
		await page.getByRole('button', { name: /Télécharger.*PDF/i }).click();

		// Toast d'erreur affichant le message INVOICE_NOT_PDF_READY.
		await expect(
			// La clé FTL invoice-pdf-error-invoice-not-pdf-ready traduit le code —
			// le message backend intercepté n'est pas affiché tel quel.
			page.getByText(/pas prête pour la génération PDF|compte bancaire principal|INVOICE_NOT_PDF_READY/i),
		).toBeVisible({ timeout: 5000 });
	});
});
