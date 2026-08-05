// Story 16-2b (#144) — fiche produit : chargement du plan comptable.
//
// ⚠️ CE FICHIER EXISTE POUR UNE SEULE ASSERTION, ET ELLE EST LA RAISON D'ÊTRE
// D'AC-B2 : `expect(fetchAccountsMock).toHaveBeenCalledWith(true)`.
//
// Pourquoi une assertion dédiée, alors qu'un test fonctionnel « le libellé du
// compte archivé reste affiché » semble couvrir la même chose : 16-1b l'a
// **mesuré**. Le mock de `fetchAccounts` rend la liste complète quel que soit
// son argument, donc un test fonctionnel reste **vert** sous la mutation
// `fetchAccounts(true) → fetchAccounts()`. Seule l'assertion sur l'argument
// l'attrape.
//
// L'assertion homologue d'`InvoiceForm.test.ts` ne couvre PAS ce site : c'est
// un appel **distinct**, dans un autre composant, et la story le dit — « déjà
// en place et non touché par cette story ». La passe 1 de `bmad-code-review` a
// constaté qu'aucun fichier de test n'existait pour cette page, alors que T-B2
// et T-B4 étaient cochées ; c'est ce fichier qui les rend vraies.
//
// Pattern projet : mocks hoistés AVANT l'import du composant, render via
// `@testing-library/svelte` (Svelte 5).

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render } from '@testing-library/svelte';
import type { AccountResponse } from '$lib/features/accounts/accounts.types';

vi.mock('$app/environment', () => ({ browser: true }));
vi.mock('$app/navigation', () => ({ goto: vi.fn() }));
vi.mock('$app/state', () => ({ page: { url: new URL('http://localhost/products') } }));

vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string) => fallback,
}));

const notifyErrorMock = vi.fn();
vi.mock('$lib/shared/utils/notify', () => ({
	notifyError: (...args: unknown[]) => notifyErrorMock(...args),
	notifySuccess: vi.fn(),
}));

const listProductsMock = vi.fn();
vi.mock('$lib/features/products/products.api', () => ({
	listProducts: (q?: unknown) => listProductsMock(q),
	createProduct: vi.fn(),
	updateProduct: vi.fn(),
	archiveProduct: vi.fn(),
}));

const fetchAccountsMock = vi.fn();
vi.mock('$lib/features/accounts/accounts.api', () => ({
	fetchAccounts: (includeArchived?: boolean) => fetchAccountsMock(includeArchived),
}));

vi.mock('$lib/features/vat-rates', () => ({
	getVatRates: vi.fn(async () => [{ rate: '8.10', label: 'product-vat-normal' }]),
}));

import Page from './+page.svelte';

function acc(partial: Partial<AccountResponse> & { id: number; number: string }): AccountResponse {
	return {
		companyId: 1,
		name: 'Ventes',
		accountType: 'Revenue',
		active: true,
		role: null,
		postable: true,
		parentId: null,
		version: 1,
		createdAt: '',
		updatedAt: '',
		...partial,
	} as AccountResponse;
}

const SALES = acc({ id: 3000, number: '3000', name: 'Ventes' });
const ARCHIVED = acc({ id: 3900, number: '3900', name: 'Ventes closes', active: false });

/** Laisse passer `onMount` et les `$effect` asynchrones (comptes, TVA, liste). */
async function settle() {
	await new Promise((r) => setTimeout(r, 50));
}

beforeEach(() => {
	vi.clearAllMocks();
	fetchAccountsMock.mockResolvedValue([SALES, ARCHIVED]);
	listProductsMock.mockResolvedValue({ items: [], total: 0, limit: 20, offset: 0 });
});

describe('fiche produit — chargement du plan comptable (Story 16-2b, AC-B2)', () => {
	it('AC-B2 : `fetchAccounts` est appelé AVEC `true` — le tueur de la mutation 2', async () => {
		render(Page);
		await settle();

		// ⚠️ NE PAS remplacer par `toHaveBeenCalled()` : c'est l'ARGUMENT qui est
		// l'objet du test. Sans le flag, un article dont le compte a été archivé
		// depuis verrait son champ paraître VIDE — et D-B2 garantit qu'aucun
		// marqueur ne viendrait le nuancer sur cette fiche.
		expect(fetchAccountsMock).toHaveBeenCalledWith(true);
	});

	it("un échec de chargement du plan comptable est SIGNALÉ, pas avalé", async () => {
		fetchAccountsMock.mockRejectedValue(new Error('réseau'));
		render(Page);
		await settle();

		// Sans cette notification, l'utilisateur trouve, à la place de
		// l'autocomplétion, un champ de saisie d'identifiant TECHNIQUE brut,
		// sans aucune explication — et peut y taper un numéro de compte en
		// croyant bien faire. *(Passe 1 de revue.)*
		expect(notifyErrorMock).toHaveBeenCalled();
	});

	it("le chargement nominal ne notifie AUCUNE erreur", async () => {
		render(Page);
		await settle();

		// Contre-épreuve : sans elle, une notification posée inconditionnellement
		// ferait passer le test précédent sans rien mesurer.
		expect(notifyErrorMock).not.toHaveBeenCalled();
	});
});
