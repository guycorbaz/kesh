import { expect, test, type Page, type Response } from '@playwright/test';
import {
	seedTestState,
	clearAuthStorage,
	authedApiContext,
	disposeContextSafe,
} from './helpers/test-state';

/**
 * Story 33-4 (#606) — un filtre de date tapé au clavier, caractère par caractère.
 *
 * ⛔ **`pressSequentially`, jamais `fill()`** : `fill()` pose la date entière, et c'est
 * ce qui a laissé passer le défaut. Au clavier, l'année passe par `0002`, `0020`, `0202`
 * (la première assertion le prouve), années que le serveur refuse : l'écran n'interroge
 * pas tant que la date n'est pas complète, et garde la dernière requête.
 *
 * Les listes sont peuplées : une liste vidée se verrait. Les filtres `to` visent l'an
 * 9999 et les `from` l'an 2020, pour que la date finale laisse la ligne à l'écran.
 */

test.beforeAll(async () => {
	await seedTestState('with-data');
});

test.afterEach(async ({ page }) => {
	await clearAuthStorage(page);
});

async function login(page: Page) {
	await page.goto('/login');
	await page.fill('#username', 'admin');
	await page.fill('#password', 'admin123');
	await page.click('button[type="submit"]');
	await expect(page).toHaveURL('/');
}

/** Une facture validée et une écriture : de quoi peupler les quatre listes. */
async function peupler(page: Page) {
	const api = await authedApiContext(page);
	try {
		const contacts = await (await api.get('/api/v1/contacts')).json();
		const facture = await api.post('/api/v1/invoices', {
			data: {
				contactId: contacts.items[0].id,
				date: new Date().toISOString().slice(0, 10),
				lines: [{ description: 'Frappe', quantity: '1', unitPrice: '100.00', vatRate: '8.10' }],
			},
		});
		expect(facture.status(), await facture.text()).toBe(201);
		const { id } = await facture.json();
		const validee = await api.post(`/api/v1/invoices/${id}/validate`);
		expect(validee.status(), await validee.text()).toBe(200);
	} finally {
		await disposeContextSafe(api);
	}
}

interface Filtre {
	nom: string;
	url: string;
	champ: string;
	api: string;
	param: string;
	ligne: string;
	/** Le jour et le mois, puis l'année chiffre à chiffre. */
	jourMois: string;
	annee: string;
	attendueIncomplete: string;
	attendueFinale: string;
}

const de = { jourMois: '0101', annee: '2020', attendueIncomplete: '0202-01-01', attendueFinale: '2020-01-01' };
const a = { jourMois: '3112', annee: '9999', attendueIncomplete: '0999-12-31', attendueFinale: '9999-12-31' };

const FILTRES: Filtre[] = [
	{ nom: 'factures, depuis', url: '/invoices', champ: '#invoice-date-from', api: '/api/v1/invoices?', param: 'dateFrom', ligne: '[data-testid="invoice-row-date"]', ...de },
	{ nom: 'factures, jusqu’à', url: '/invoices', champ: '#invoice-date-to', api: '/api/v1/invoices?', param: 'dateTo', ligne: '[data-testid="invoice-row-date"]', ...a },
	{ nom: 'échéancier', url: '/invoices/due-dates', champ: '#dd-due-before', api: '/api/v1/invoices/due-dates?', param: 'dueBefore', ligne: '[data-testid="due-dates-date"]', ...a },
	{ nom: 'écritures, début', url: '/journal-entries', champ: '#filter-date-from', api: '/api/v1/journal-entries?', param: 'dateFrom', ligne: 'tbody tr', ...de },
	{ nom: 'écritures, fin', url: '/journal-entries', champ: '#filter-date-to', api: '/api/v1/journal-entries?', param: 'dateTo', ligne: 'tbody tr', ...a },
	{ nom: 'journal d’audit, du', url: '/audit-log', champ: '[data-testid="audit-log-filter-date-from"]', api: '/api/v1/audit-log?', param: 'dateFrom', ligne: '[data-testid="audit-log-row"]', ...de },
	{ nom: 'journal d’audit, au', url: '/audit-log', champ: '[data-testid="audit-log-filter-date-to"]', api: '/api/v1/audit-log?', param: 'dateTo', ligne: '[data-testid="audit-log-row"]', ...a },
];

for (const f of FILTRES) {
	test(`${f.nom} : la frappe n’interroge pas, ne notifie rien, ne vide pas la liste`, async ({ page }) => {
		await login(page);
		await peupler(page);
		await page.goto(f.url);
		await expect(page.locator(f.ligne).first()).toBeVisible();

		const envoyees: string[] = [];
		const refus: string[] = [];
		const surReponse = (r: Response) => {
			if (!r.url().includes(f.api)) return;
			envoyees.push(new URL(r.url()).searchParams.get(f.param) ?? '');
			if (r.status() >= 400) refus.push(`${r.status()} ${r.url()}`);
		};
		page.on('response', surReponse);

		const champ = page.locator(f.champ);
		await champ.focus();
		await champ.pressSequentially(f.jourMois + f.annee.slice(0, 3), { delay: 40 });
		// La preuve que la frappe passe bien par ces années.
		await expect(champ).toHaveValue(f.attendueIncomplete);
		await page.waitForTimeout(400);
		expect(envoyees, 'aucune requête tant que la date est incomplète').toEqual([]);
		await expect(page.locator(f.ligne).first()).toBeVisible();
		await expect(page.locator('[data-sonner-toast]')).toHaveCount(0);

		await champ.pressSequentially(f.annee.slice(3), { delay: 40 });
		await expect(champ).toHaveValue(f.attendueFinale);
		await expect.poll(() => envoyees).toContain(f.attendueFinale);
		await expect(page.locator(f.ligne).first()).toBeVisible();
		expect(refus, 'aucun refus du serveur').toEqual([]);
		expect(envoyees.every((v) => v === f.attendueFinale)).toBe(true);
		await expect(page.locator('[data-sonner-toast]')).toHaveCount(0);

		// La sortie du champ restaure une date laissée incomplète.
		await champ.focus();
		await champ.pressSequentially(f.jourMois + f.annee.slice(0, 2), { delay: 40 });
		await page.locator('h1').first().click();
		await expect(champ).toHaveValue(f.attendueFinale);
		expect(refus).toEqual([]);
		page.off('response', surReponse);
	});
}
