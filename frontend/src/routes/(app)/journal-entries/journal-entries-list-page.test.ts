/**
 * v0172-correctifs (#880, AC-C2) — la liste du journal dit ses dates en forme suisse :
 * la date de chaque écriture et celle du bandeau « Livres verrouillés jusqu'au … ».
 *
 * Premier test de cette page. Patron des `*-page.test.ts` : mocks hissés AVANT
 * l'import du composant ; le `page` de `$app/state` est un objet simple (la page ne lit
 * l'adresse qu'au montage).
 */
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, fireEvent, cleanup, waitFor } from '@testing-library/svelte';

// Story 48-10 (V1-9) : l'adresse se règle par test (signet `?amountMin=1.250`) ; l'URL écrite se
// lit dans les appels de `goto`.
const adresse = vi.hoisted(() => ({ url: new URL('http://localhost/journal-entries') }));
const gotoMock = vi.hoisted(() => vi.fn());
vi.mock('$app/state', () => ({
	page: {
		get url() {
			return adresse.url;
		},
	},
}));
vi.mock('$app/navigation', () => ({ goto: gotoMock }));
vi.mock('$lib/features/onboarding/onboarding.svelte', () => ({
	i18nMsg: (_k: string, fallback: string, args?: Record<string, string | number>) =>
		args ? fallback.replace(/\{\s*\$(\w+)\s*\}/g, (_, n) => String(args[n] ?? '')) : fallback,
}));
vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_k: string, fallback: string, args?: Record<string, string | number>) =>
		args ? fallback.replace(/\{\s*\$(\w+)\s*\}/g, (_, n) => String(args[n] ?? '')) : fallback,
	i18nLocale: () => 'fr-CH',
}));
const fetchEntriesMock = vi.fn();
vi.mock('$lib/features/journal-entries/journal-entries.api', () => ({
	fetchJournalEntries: (q: unknown) => fetchEntriesMock(q),
}));
vi.mock('$lib/features/accounts/accounts.api', () => ({ fetchAccounts: vi.fn(async () => []) }));
vi.mock('$lib/features/projects/projects.api', () => ({ listProjects: vi.fn(async () => []) }));
vi.mock('$lib/features/invoices/invoices.api', () => ({
	getInvoiceSettings: vi.fn(async () => ({ defaultVatRecoverableAccountId: null })),
}));
const companyMock = vi.fn();
vi.mock('$lib/features/settings/settings.api', () => ({
	fetchCompanyCurrent: () => companyMock(),
}));

import Page from './+page.svelte';

function entry(id: number, entryDate: string) {
	return {
		id,
		companyId: 1,
		fiscalYearId: 1,
		entryNumber: id,
		entryDate,
		journal: 'Achats',
		description: `Écriture ${id}`,
		version: 1,
		reversesEntryId: null,
		lines: [],
		createdAt: '2026-01-01T00:00:00',
		updatedAt: '2026-01-01T00:00:00',
	};
}

beforeEach(() => {
	fetchEntriesMock.mockResolvedValue({
		items: [entry(1, '2026-01-29'), entry(2, '2026-03-02')],
		total: 2,
		offset: 0,
		limit: 50,
	});
	companyMock.mockResolvedValue({ company: { booksLockedThrough: '2025-12-31' } });
});
afterEach(() => {
	cleanup();
	vi.clearAllMocks();
});

describe('liste du journal — dates en forme suisse (v0172, #880)', () => {
	it('chaque écriture dit 29.01.2026 / 02.03.2026, jamais l’ISO (MC2)', async () => {
		const { container } = render(Page);
		await waitFor(() => expect(container.textContent).toContain('Écriture 2'));
		expect(container.textContent).toContain('29.01.2026');
		expect(container.textContent).toContain('02.03.2026');
		expect(container.textContent).not.toContain('2026-01-29');
		expect(container.textContent).not.toContain('2026-03-02');
	});

	it('le bandeau du verrou dit « jusqu’au 31.12.2025 inclus » (MC2)', async () => {
		const { findByTestId } = render(Page);
		const banner = await findByTestId('books-locked-banner');
		expect(banner.textContent).toContain('31.12.2025');
		expect(banner.textContent).not.toContain('2025-12-31');
	});
});

describe('journal — une date en cours de frappe (Story 33-4)', () => {
	for (const [id, cle] of [
		['filter-date-from', 'dateFrom'],
		['filter-date-to', 'dateTo'],
	]) {
		it(`#${id} : l’année tapée n’interroge pas et ne vide pas la liste (mutation : valeur appliquée sans contrôle)`, async () => {
			const { container } = render(Page);
			await waitFor(() => expect(container.textContent).toContain('Écriture 2'));
			const input = container.querySelector(`#${id}`) as HTMLInputElement;
			const avant = fetchEntriesMock.mock.calls.length;
			// Chromium émet `change` à chaque segment.
			for (const v of ['0002-11-06', '0020-11-06', '0202-11-06']) {
				await fireEvent.change(input, { target: { value: v } });
				expect(input.value).toBe(v);
			}
			await new Promise((r) => setTimeout(r, 0));
			expect(fetchEntriesMock.mock.calls.length).toBe(avant);
			expect(container.textContent).toContain('Écriture 2');

			await fireEvent.change(input, { target: { value: '2026-11-06' } });
			await waitFor(() =>
				expect(fetchEntriesMock).toHaveBeenLastCalledWith(
					expect.objectContaining({ [cle]: '2026-11-06' }),
				),
			);
			await fireEvent.change(input, { target: { value: '0202-11-06' } });
			await fireEvent.blur(input);
			expect(input.value).toBe('2026-11-06');
		});
	}
});

// Story 48-10 (#1177 ; AC4, AC5, AC12) — les bornes de montant du filtre se lisent comme le
// montant d'une écriture. Avant la story, « 1.250 » partait tel quel : le serveur le lisait 1,25
// et la liste, fausse, ne le disait pas ; « 1'250 » ou « 1,25 » étaient refusés en 400, que la page
// taisait (liste vide). Une borne ambiguë ou illisible est désormais omise, en le disant ; le
// champ garde sa saisie.
describe('journal — bornes de montant du filtre (Story 48-10)', () => {
	afterEach(() => {
		adresse.url = new URL('http://localhost/journal-entries');
	});

	const PAS_APPLIQUE = 'pas appliqué';

	/** Dernière requête et dernière URL écrite. */
	const derniereRequete = () =>
		fetchEntriesMock.mock.calls[fetchEntriesMock.mock.calls.length - 1][0] as Record<string, unknown>;
	const derniereUrl = () => {
		const appel = gotoMock.mock.calls[gotoMock.mock.calls.length - 1];
		return new URL(String(appel[0]));
	};

	async function monter() {
		const r = render(Page);
		await waitFor(() => expect(r.container.textContent).toContain('Écriture 2'));
		return r;
	}

	/** Tape une borne et attend le rechargement (300 ms d'attente de frappe). */
	async function taper(container: HTMLElement, id: string, valeur: string) {
		const input = container.querySelector(`#${id}`) as HTMLInputElement;
		const avant = fetchEntriesMock.mock.calls.length;
		await fireEvent.input(input, { target: { value: valeur } });
		await waitFor(() => expect(fetchEntriesMock.mock.calls.length).toBeGreaterThan(avant));
		return input;
	}

	for (const [id, cle, testid] of [
		['filter-amount-min', 'amountMin', 'journal-entries-filter-amount-min-problem'],
		['filter-amount-max', 'amountMax', 'journal-entries-filter-amount-max-problem'],
	] as const) {
		describe(`#${id}`, () => {
			// Rouge avant le correctif : `{ [cle]: '1.250' }` partait (M7, M8, M11).
			it('« 1.250 » → message ambigu et « pas appliqué », borne omise de la requête et de l’URL, saisie gardée', async () => {
				const { container, getByTestId } = await monter();
				const input = await taper(container, id, '1.250');
				const msg = getByTestId(testid);
				expect(msg.textContent).toContain('Montant ambigu');
				expect(msg.textContent).toContain(PAS_APPLIQUE);
				expect(derniereRequete()[cle]).toBeUndefined();
				await waitFor(() => expect(derniereUrl().searchParams.has(cle)).toBe(false));
				expect(input.value).toBe('1.250');
			});

			// M10c : « 4 décimales au plus ».
			it("« 12'34 » → message illisible, « 4 décimales au plus », borne omise (M10c)", async () => {
				const { container, getByTestId } = await monter();
				await taper(container, id, "12'34");
				const msg = getByTestId(testid);
				expect(msg.textContent).toContain('Montant illisible');
				expect(msg.textContent).toContain('4 décimales au plus');
				expect(msg.textContent).toContain(PAS_APPLIQUE);
				expect(derniereRequete()[cle]).toBeUndefined();
			});

			// La grammaire du journal n'admet aucun signe : « -5 » n'atteint plus le serveur.
			// Revue P1 — une borne ambiguë bordée de blancs : rognée, puis refusée comme ambiguë.
			it('«  1.250  » (bordé de blancs) → ambigu, borne omise, saisie gardée', async () => {
				const { container, getByTestId } = await monter();
				const input = await taper(container, id, ' 1.250 ');
				expect(getByTestId(testid).textContent).toContain('Montant ambigu');
				expect(derniereRequete()[cle]).toBeUndefined();
				expect(input.value).toBe(' 1.250 ');
			});

			it('« -5 » → illisible, borne omise', async () => {
				const { container, getByTestId } = await monter();
				await taper(container, id, '-5');
				expect(getByTestId(testid).textContent).toContain('Montant illisible');
				expect(derniereRequete()[cle]).toBeUndefined();
			});

			// Rouge avant le correctif : « 1'250 » partait tel quel (le serveur rendait 400).
			it("« 1'250 » → requête et URL portent 1250", async () => {
				const { container, queryByTestId } = await monter();
				await taper(container, id, "1'250");
				expect(queryByTestId(testid)).toBeNull();
				expect(derniereRequete()[cle]).toBe('1250');
				await waitFor(() => expect(derniereUrl().searchParams.get(cle)).toBe('1250'));
			});

			it('« 100 » → part « 100 »', async () => {
				const { container } = await monter();
				await taper(container, id, '100');
				expect(derniereRequete()[cle]).toBe('100');
			});

			// AC5 — rien de juste n'est refusé (M15 : « 1250 » bordé de blancs).
			for (const [saisie, charge] of [
				['1.25', '1.25'],
				['1250', '1250'],
				["1'250.50", '1250.50'],
				['1’250', '1250'],
				['1,25', '1.25'],
				['0.50', '0.50'],
				[' 1250 ', '1250'],
				['1250.0000', '1250.0000'],
			]) {
				it(`« ${saisie} » sans message, part « ${charge} » (M15)`, async () => {
					const { container, queryByTestId } = await monter();
					await taper(container, id, saisie);
					expect(queryByTestId(testid)).toBeNull();
					expect(derniereRequete()[cle]).toBe(charge);
				});
			}

			// Signet ancien : la borne ambiguë s'affiche, n'est pas envoyée, et l'URL la retire (M11).
			it(`URL ?${cle}=1.250 → message au chargement, borne non envoyée, URL réécrite sans elle`, async () => {
				adresse.url = new URL(`http://localhost/journal-entries?${cle}=1.250`);
				const { container, findByTestId } = await monter();
				const msg = await findByTestId(testid);
				expect(msg.textContent).toContain('Montant ambigu');
				expect((container.querySelector(`#${id}`) as HTMLInputElement).value).toBe('1.250');
				for (const appel of fetchEntriesMock.mock.calls) {
					expect((appel[0] as Record<string, unknown>)[cle]).toBeUndefined();
				}
				await waitFor(() => expect(gotoMock).toHaveBeenCalled());
				expect(derniereUrl().searchParams.has(cle)).toBe(false);
			});

			// Un signet `1,25`, refusé en 400 jusqu'ici, s'applique désormais.
			it(`URL ?${cle}=1,25 → borne envoyée « 1.25 »`, async () => {
				adresse.url = new URL(`http://localhost/journal-entries?${cle}=1,25`);
				await monter();
				expect(fetchEntriesMock.mock.calls[0][0]).toMatchObject({ [cle]: '1.25' });
			});

			// AC12 — accessibilité du refus (M16).
			it('« 1.250 » : aria-invalid et aria-describedby nomment le message ; corrigé, ils le quittent', async () => {
				const { container, getByTestId } = await monter();
				const input = await taper(container, id, '1.250');
				const msg = getByTestId(testid);
				expect(msg.id).not.toBe('');
				expect(input.getAttribute('aria-invalid')).toBe('true');
				expect((input.getAttribute('aria-describedby') ?? '').split(' ')).toContain(msg.id);
				const msgId = msg.id;
				await taper(container, id, "1'250");
				expect(['false', null]).toContain(input.getAttribute('aria-invalid'));
				expect((input.getAttribute('aria-describedby') ?? '').split(' ')).not.toContain(msgId);
			});
		});
	}
});
