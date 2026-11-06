/**
 * Story v017-4b (#849, #831 ; AC-21) — la liste des factures fournisseurs : le badge du
 * brouillon, le total aux décimales exactes, les annulées masquées par défaut.
 *
 * ⚠️ Chaque test nomme la MUTATION qu'il attrape. Patron des `*-page.test.ts` : mocks
 * hoistés AVANT l'import du composant.
 */
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, fireEvent, cleanup, waitFor } from '@testing-library/svelte';
import type { SupplierInvoiceListItemResponse } from '$lib/features/supplier-invoices/supplier-invoices.types';

// v0172-correctifs (#870, AC-B2 bis) — `page` réactif ; `goto` réaffecte son URL,
// résolue contre l'URL courante (un `goto('?cancelled=1')` relatif garde le chemin).
vi.mock('$app/state', async () => ({
	page: (await import('./page-state.test-helper.svelte')).pageState,
}));
vi.mock('$app/navigation', () => ({
	goto: vi.fn(async (cible: string) => {
		const m = await import('./page-state.test-helper.svelte');
		m.pageState.url = new URL(cible, m.pageState.url);
	}),
}));
import { pageState } from './page-state.test-helper.svelte';
import { goto } from '$app/navigation';
vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_k: string, fallback: string, args?: Record<string, string | number>) =>
		args ? fallback.replace(/\{\s*\$(\w+)\s*\}/g, (_, n) => String(args[n] ?? '')) : fallback,
}));
vi.mock('$lib/app/stores/auth.svelte', () => ({
	authState: { currentUser: { role: 'Comptable', username: 'c', userId: '1' } },
}));
const listMock = vi.fn();
vi.mock('$lib/features/supplier-invoices/supplier-invoices.api', () => ({
	listSupplierInvoices: (q: unknown) => listMock(q),
}));

import Page from './+page.svelte';

function item(over: Partial<SupplierInvoiceListItemResponse>): SupplierInvoiceListItemResponse {
	return {
		id: 1,
		contactId: 1,
		contactName: 'Dubarde SA',
		supplierInvoiceNumber: 'FF-1',
		internalNumber: null,
		status: 'open',
		invoiceDate: '2026-05-10',
		dueDate: null,
		totalAmount: '100.00',
		...over,
	};
}

beforeEach(() => {
	pageState.url = new URL('http://localhost/supplier-invoices');
	vi.mocked(goto).mockClear();
	listMock.mockReset();
	listMock.mockImplementation(async () => ({
		items: [
			item({ id: 1, status: 'draft', supplierInvoiceNumber: 'FF-BR', totalAmount: '100.0040' }),
			item({ id: 2, status: 'open', supplierInvoiceNumber: 'FF-VA', totalAmount: '100.00' }),
		],
		total: 2,
		offset: 0,
		limit: 100,
	}));
});
afterEach(() => cleanup());

describe('liste des factures fournisseurs', () => {
	it('demande par défaut les brouillons, validées et payées — jamais les annulées (M28)', async () => {
		render(Page);
		await waitFor(() => expect(listMock).toHaveBeenCalledTimes(1));
		// v0172 (#870) : la liste se pagine, 50 par page (était `limit: 100` sans `offset`).
		expect(listMock.mock.calls[0][0]).toEqual({
			limit: 50,
			offset: 0,
			status: ['draft', 'open', 'paid'],
		});
	});

	it('la case « Afficher les factures annulées » relit la liste sans filtre', async () => {
		const { findByTestId } = render(Page);
		const box = (await findByTestId('supplier-invoices-show-cancelled')) as HTMLInputElement;
		expect(box.checked).toBe(false);
		await fireEvent.click(box);
		await waitFor(() => expect(listMock).toHaveBeenCalledTimes(2));
		expect(listMock.mock.calls[1][0]).toEqual({ limit: 50, offset: 0 });
		await fireEvent.click(box);
		await waitFor(() => expect(listMock).toHaveBeenCalledTimes(3));
		expect(listMock.mock.calls[2][0]).toEqual({
			limit: 50,
			offset: 0,
			status: ['draft', 'open', 'paid'],
		});
	});

	it('un brouillon porte le badge « Brouillon », une facture validée « Validée »', async () => {
		const { findAllByTestId } = render(Page);
		const badges = await findAllByTestId('supplier-invoice-status-badge');
		expect(badges.map((b) => b.textContent?.trim())).toEqual(['Brouillon', 'Validée']);
		expect(badges[0].getAttribute('data-status')).toBe('draft');
	});

	it('le TTC s’affiche aux décimales exactes : 100.0040 reste 100.0040 (#831, M27)', async () => {
		const { findAllByTestId } = render(Page);
		const totals = await findAllByTestId('supplier-invoice-row-total');
		expect(totals.map((t) => t.textContent?.trim())).toEqual(['100.0040', '100.00']);
	});

	// Revue P1 (B-4 = E-8) : deux bascules rapprochées, réponses croisées — la liste est
	// celle de la DERNIÈRE bascule.
	it('une réponse périmée ne remplace pas celle de la dernière bascule', async () => {
		const { findByTestId, findAllByTestId } = render(Page);
		const box = (await findByTestId('supplier-invoices-show-cancelled')) as HTMLInputElement;
		await findAllByTestId('supplier-invoice-row');
		let resolveFirst: (v: unknown) => void = () => {};
		listMock.mockImplementationOnce(
			() =>
				new Promise((r) => {
					resolveFirst = r;
				}),
		);
		listMock.mockImplementationOnce(async () => ({
			items: [item({ id: 3, status: 'draft', supplierInvoiceNumber: 'FF-DERNIERE' })],
			total: 1,
			offset: 0,
			limit: 100,
		}));
		await fireEvent.click(box); // cochée : lecture lente
		// v0172 (#870) : la case navigue ; la lecture part quand l'adresse a changé.
		// Deux clics dans la même tâche reviendraient à l'adresse de départ sans
		// qu'aucune lecture ne parte : on attend la première.
		await waitFor(() => expect(listMock).toHaveBeenCalledTimes(2));
		await fireEvent.click(box); // décochée : lecture rapide
		await waitFor(() => expect(listMock).toHaveBeenCalledTimes(3));
		resolveFirst({
			items: [item({ id: 4, status: 'cancelled', supplierInvoiceNumber: 'FF-PERIMEE' })],
			total: 1,
			offset: 0,
			limit: 100,
		});
		await waitFor(async () => {
			const rows = await findAllByTestId('supplier-invoice-row');
			expect(rows.map((r) => r.textContent)).toEqual([expect.stringContaining('FF-DERNIERE')]);
		});
		await new Promise((r) => setTimeout(r, 20));
		const rows = await findAllByTestId('supplier-invoice-row');
		expect(rows.map((r) => r.textContent)).toEqual([expect.stringContaining('FF-DERNIERE')]);
		expect(box.checked).toBe(false);
	});
});

// Story v0171 (#871) — les dates de la liste en forme suisse.
describe('liste des factures fournisseurs — dates (v0171)', () => {
	beforeEach(() => {
		listMock.mockImplementation(async () => ({
			items: [
				item({ id: 1, invoiceDate: '2026-01-29', dueDate: '2026-02-28' }),
				item({ id: 2, invoiceDate: '2026-03-02', dueDate: null }),
			],
			total: 2,
			offset: 0,
			limit: 100,
		}));
	});

	it('date de facture `29.01.2026`, jamais l’ISO (mutation M5 : formatSwissDate retiré de `invoiceDate`)', async () => {
		const { findAllByTestId, container } = render(Page);
		const dates = await findAllByTestId('supplier-invoice-row-date');
		expect(dates.map((d) => d.textContent?.trim())).toEqual(['29.01.2026', '02.03.2026']);
		expect(container.textContent).not.toContain('2026-01-29');
	});

	it('échéance `28.02.2026`, échéance nulle `—` (mutation M5 : formatSwissDate retiré de `dueDate`)', async () => {
		const { findAllByTestId, container } = render(Page);
		const dues = await findAllByTestId('supplier-invoice-row-due');
		expect(dues.map((d) => d.textContent?.trim())).toEqual(['28.02.2026', '—']);
		expect(container.textContent).not.toContain('2026-02-28');
	});
});

// v0172-correctifs (#870) — la liste se pagine et se filtre par statut depuis
// l'adresse, lue de façon réactive (C-v0172b-7, C-v0172b-26).
describe('liste des factures fournisseurs — pagination et filtre (v0172, #870)', () => {
	/** Une base de `total` factures : la page demandée, comme le serveur la rendrait. */
	function serveur(total: number) {
		listMock.mockImplementation(async (q: { limit: number; offset: number }) => {
			const n = Math.max(0, Math.min(q.limit, total - q.offset));
			return {
				items: Array.from({ length: n }, (_, i) =>
					item({ id: q.offset + i + 1, supplierInvoiceNumber: `FF-${q.offset + i + 1}` }),
				),
				total,
				offset: q.offset,
				limit: q.limit,
			};
		});
	}
	const DEFAULT = ['draft', 'open', 'paid'];
	const lastQuery = () => listMock.mock.calls[listMock.mock.calls.length - 1][0];
	const at = (search: string) => {
		pageState.url = new URL(`http://localhost/supplier-invoices${search}`);
	};

	it('AC-B1 — `?status=draft` : la liste ne demande que les brouillons, le bandeau le dit, la case est masquée', async () => {
		at('?status=draft');
		const { findByTestId, queryByTestId } = render(Page);
		await waitFor(() => expect(listMock).toHaveBeenCalledTimes(1));
		expect(listMock.mock.calls[0][0]).toEqual({ limit: 50, offset: 0, status: ['draft'] });
		const banner = await findByTestId('supplier-invoices-filter-banner');
		expect(banner.textContent).toContain('« Brouillon »');
		expect(banner.textContent).toContain('tous exercices confondus');
		const all = await findByTestId('supplier-invoices-filter-clear');
		// Revue de code P1 (E-5, C-v0172b-35) : « toutes » dit vrai — les annulées comprises.
		expect(all.getAttribute('href')).toBe('/supplier-invoices?cancelled=1');
		expect(queryByTestId('supplier-invoices-show-cancelled')).toBeNull();
	});

	// Revue de code P1 (B-4 = E-4) : une liste vide dit ce qui est vide, non « aucune
	// facture enregistrée ».
	it('liste filtrée vide : « Aucune facture « Brouillon ». », jamais « Aucune facture fournisseur enregistrée. »', async () => {
		serveur(0);
		at('?status=draft');
		const { findByTestId, container } = render(Page);
		const empty = await findByTestId('supplier-invoices-empty');
		expect(empty.textContent?.trim()).toBe('Aucune facture « Brouillon ».');
		expect(container.textContent).not.toContain('Aucune facture fournisseur enregistrée.');
	});

	it('liste par défaut vide : les annulées sont masquées, le message le dit ; avec la case cochée, « aucune facture enregistrée »', async () => {
		serveur(0);
		const { findByTestId } = render(Page);
		expect((await findByTestId('supplier-invoices-empty')).textContent?.trim()).toBe(
			'Aucune facture fournisseur à afficher : les factures annulées sont masquées.',
		);
		at('?cancelled=1');
		await waitFor(async () =>
			expect((await findByTestId('supplier-invoices-empty')).textContent?.trim()).toBe(
				'Aucune facture fournisseur enregistrée.',
			),
		);
	});

	it('AC-B1 — `?status=bogus` est ignoré : liste par défaut, sans bandeau', async () => {
		at('?status=bogus&limit=0&offset=-3');
		const { queryByTestId, findByTestId } = render(Page);
		await waitFor(() => expect(listMock).toHaveBeenCalledTimes(1));
		expect(listMock.mock.calls[0][0]).toEqual({ limit: 50, offset: 0, status: DEFAULT });
		await findByTestId('supplier-invoices-show-cancelled');
		expect(queryByTestId('supplier-invoices-filter-banner')).toBeNull();
	});

	it('AC-B2 — 120 factures : Précédent désactivé, Suivant demande offset 50 et réécrit l’adresse (statut gardé), page 3 sans Suivant (MB1)', async () => {
		serveur(120);
		at('?status=draft');
		const { findByTestId } = render(Page);
		const count = await findByTestId('supplier-invoices-count');
		expect(count.textContent?.trim()).toBe('120 factures');
		const prev = (await findByTestId('supplier-invoices-prev')) as HTMLButtonElement;
		const next = (await findByTestId('supplier-invoices-next')) as HTMLButtonElement;
		expect(prev.disabled).toBe(true);
		expect(next.disabled).toBe(false);
		await fireEvent.click(next);
		await waitFor(() => expect(lastQuery()).toEqual({ limit: 50, offset: 50, status: ['draft'] }));
		expect(pageState.url.searchParams.get('offset')).toBe('50');
		expect(pageState.url.searchParams.get('status')).toBe('draft');
		await fireEvent.click(next);
		await waitFor(() => expect(lastQuery()).toEqual({ limit: 50, offset: 100, status: ['draft'] }));
		await waitFor(async () => {
			expect(((await findByTestId('supplier-invoices-next')) as HTMLButtonElement).disabled).toBe(true);
		});
		expect(((await findByTestId('supplier-invoices-prev')) as HTMLButtonElement).disabled).toBe(false);
	});

	it('AC-B2 — navigation par lien : de `?status=draft&offset=50`, l’adresse nue relit la liste par défaut, page 1 (MB8)', async () => {
		serveur(120);
		at('?status=draft&offset=50');
		render(Page);
		await waitFor(() => expect(lastQuery()).toEqual({ limit: 50, offset: 50, status: ['draft'] }));
		// Un clic sur `<a href="/supplier-invoices">` (bandeau, menu) : même instance de page.
		at('');
		await waitFor(() => expect(lastQuery()).toEqual({ limit: 50, offset: 0, status: DEFAULT }));
		// Retour arrière du navigateur : la liste suit l'adresse.
		at('?status=draft&offset=50');
		await waitFor(() => expect(lastQuery()).toEqual({ limit: 50, offset: 50, status: ['draft'] }));
	});

	it('AC-B2 — page au-delà du total : `?offset=500` pour 120 factures ramène à la dernière page', async () => {
		serveur(120);
		at('?offset=500');
		const { findAllByTestId, findByTestId } = render(Page);
		await waitFor(() => expect(lastQuery()).toEqual({ limit: 50, offset: 100, status: DEFAULT }));
		expect(vi.mocked(goto)).toHaveBeenCalledWith(
			'/supplier-invoices?offset=100',
			expect.objectContaining({ replaceState: true }),
		);
		await waitFor(async () => expect(await findAllByTestId('supplier-invoice-row')).toHaveLength(20));
		await findByTestId('supplier-invoices-count');
	});

	it('AC-B2 — basculer la case ou retirer le filtre remet `offset` à 0', async () => {
		serveur(120);
		at('?offset=50');
		const { findByTestId } = render(Page);
		const box = (await findByTestId('supplier-invoices-show-cancelled')) as HTMLInputElement;
		await waitFor(() => expect(lastQuery()).toEqual({ limit: 50, offset: 50, status: DEFAULT }));
		await fireEvent.click(box);
		await waitFor(() => expect(lastQuery()).toEqual({ limit: 50, offset: 0 }));
		expect(pageState.url.searchParams.get('offset')).toBeNull();
	});

	it('AC-B2 bis — la case : `?cancelled=1`, Suivant le garde, retour arrière la décoche', async () => {
		serveur(120);
		const { findByTestId } = render(Page);
		const box = (await findByTestId('supplier-invoices-show-cancelled')) as HTMLInputElement;
		await fireEvent.click(box);
		await waitFor(() => expect(lastQuery()).toEqual({ limit: 50, offset: 0 }));
		expect(pageState.url.search).toBe('?cancelled=1');
		expect(pageState.url.pathname).toBe('/supplier-invoices');
		await fireEvent.click(await findByTestId('supplier-invoices-next'));
		await waitFor(() => expect(lastQuery()).toEqual({ limit: 50, offset: 50 }));
		expect(pageState.url.searchParams.get('cancelled')).toBe('1');
		at('');
		await waitFor(() => expect(lastQuery()).toEqual({ limit: 50, offset: 0, status: DEFAULT }));
		await waitFor(() => expect(box.checked).toBe(false));
	});

	it('une seule facture : « 1 facture », au singulier', async () => {
		serveur(1);
		const { findByTestId } = render(Page);
		expect((await findByTestId('supplier-invoices-count')).textContent?.trim()).toBe('1 facture');
	});
});

// Story 33-1b (#926) — les deux colonnes, le tri et la recherche par l'adresse.
describe('liste des factures fournisseurs — le numéro interne (33-1b, #926)', () => {
	const lastQuery = () => listMock.mock.calls[listMock.mock.calls.length - 1][0];
	const at = (search: string) => {
		pageState.url = new URL(`http://localhost/supplier-invoices${search}`);
	};

	it('AC2 — deux colonnes ; un brouillon jamais validé rend « — », un brouillon numéroté son numéro', async () => {
		listMock.mockImplementation(async () => ({
			items: [
				item({ id: 1, status: 'draft', internalNumber: null, supplierInvoiceNumber: 'ABC' }),
				item({ id: 2, status: 'draft', internalNumber: 'FF-2026-002', supplierInvoiceNumber: null }),
			],
			total: 2,
			offset: 0,
			limit: 50,
		}));
		const { findAllByTestId } = render(Page);
		const internes = await findAllByTestId('supplier-invoice-row-internal-number');
		const fournisseurs = await findAllByTestId('supplier-invoice-row-supplier-number');
		expect(internes.map((c) => c.textContent?.trim())).toEqual(['—', 'FF-2026-002']);
		expect(fournisseurs.map((c) => c.textContent?.trim())).toEqual(['ABC', '—']);
	});

	it('AC3 — l’en-tête trie par l’adresse, ramène `offset` à 0 (M9), et dit le sens par `aria-sort`', async () => {
		at('?offset=50');
		const { findByTestId } = render(Page);
		const bouton = await findByTestId('supplier-invoices-sort-internal-number');
		expect(bouton.closest('th')?.getAttribute('aria-sort')).toBe('none');
		await fireEvent.click(bouton);
		await waitFor(() => {
			expect(pageState.url.searchParams.get('sort')).toBe('internalNumber');
			expect(pageState.url.searchParams.get('order')).toBe('asc');
			expect(pageState.url.searchParams.get('offset')).toBeNull();
		});
		await waitFor(() => expect(lastQuery().sort).toBe('internalNumber'));
		expect(lastQuery().order).toBe('asc');
		expect(lastQuery().offset).toBe(0);
		const trie = await findByTestId('supplier-invoices-sort-internal-number');
		expect(trie.closest('th')?.getAttribute('aria-sort')).toBe('ascending');
		await fireEvent.click(trie);
		await waitFor(() => expect(lastQuery().order).toBe('desc'));
	});

	it('AC3 — `sort` absent : aucun tri demandé, l’ordre d’aujourd’hui', async () => {
		render(Page);
		await waitFor(() => expect(listMock).toHaveBeenCalled());
		expect(lastQuery()).toEqual({ limit: 50, offset: 0, status: ['draft', 'open', 'paid'] });
	});

	it('AC4 — `?q=` part au serveur, et le champ l’affiche ; la liste vide le dit', async () => {
		listMock.mockImplementation(async () => ({ items: [], total: 0, offset: 0, limit: 50 }));
		at('?q=FF-2026');
		const { findByTestId } = render(Page);
		await waitFor(() => expect(lastQuery().q).toBe('FF-2026'));
		const champ = (await findByTestId('supplier-invoices-search')) as HTMLInputElement;
		expect(champ.value).toBe('FF-2026');
		expect((await findByTestId('supplier-invoices-empty')).textContent?.trim()).toBe(
			'Aucune facture ne correspond à « FF-2026 ».',
		);
	});

	it('AC4 — la frappe est temporisée, puis écrit `q` à l’adresse et remet `offset` à 0', async () => {
		at('?offset=50');
		const { findByTestId } = render(Page);
		const champ = await findByTestId('supplier-invoices-search');
		await fireEvent.input(champ, { target: { value: 'Dupont' } });
		expect(pageState.url.searchParams.get('q')).toBeNull();
		await waitFor(() => expect(pageState.url.searchParams.get('q')).toBe('Dupont'));
		expect(pageState.url.searchParams.get('offset')).toBeNull();
		await waitFor(() => expect(lastQuery().q).toBe('Dupont'));
	});
});
