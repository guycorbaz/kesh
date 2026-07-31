// Story 16-1b — sélecteur de compte de produit par ligne dans le formulaire de
// facture (AC14). Premier test de ce composant : il n'en avait aucun.
//
// Pattern projet : mocks i18n + API définis AVANT l'import du composant
// (hoisting Vitest), render via `@testing-library/svelte` (Svelte 5).
//
// ⚠️ Le mock i18n reproduit l'INTERPOLATION du vrai `i18nMsg`. Le mock habituel
// du dépôt, `(_key, fallback) => fallback`, ferait porter les assertions sur le
// littéral `{ $lines }` au lieu des numéros de lignes — le test passerait en
// n'observant rien.

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, fireEvent } from '@testing-library/svelte';
import type { AccountResponse } from '$lib/features/accounts/accounts.types';
import type { InvoiceResponse } from '$lib/features/invoices/invoices.types';

vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string, args?: Record<string, string | number>) =>
		args
			? fallback.replace(/\{\s*\$(\w+)\s*\}/g, (_m, k: string) => String(args[k] ?? ''))
			: fallback,
}));

const fetchAccountsMock = vi.fn();
vi.mock('$lib/features/accounts/accounts.api', () => ({
	fetchAccounts: (includeArchived?: boolean) => fetchAccountsMock(includeArchived),
}));

const getInvoiceSettingsMock = vi.fn();
const getInvoiceMock = vi.fn();
const updateInvoiceMock = vi.fn();
vi.mock('$lib/features/invoices/invoices.api', () => ({
	getInvoiceSettings: () => getInvoiceSettingsMock(),
	getInvoice: (id: number) => getInvoiceMock(id),
	createInvoice: vi.fn(),
	updateInvoice: (id: number, req: unknown) => updateInvoiceMock(id, req),
}));

vi.mock('$lib/features/contacts/contacts.api', () => ({
	getContact: vi.fn(async () => ({ id: 1, name: 'Client' })),
}));

vi.mock('$lib/features/vat-rates', () => ({
	getVatRates: vi.fn(async () => [{ rate: '8.10' }, { rate: '2.60' }]),
}));

vi.mock('$lib/features/projects/projects.api', () => ({
	listProjects: vi.fn(async () => []),
}));

vi.mock('$app/navigation', () => ({ goto: vi.fn() }));
vi.mock('$lib/shared/utils/notify', () => ({ notifyError: vi.fn(), notifySuccess: vi.fn() }));

import InvoiceForm from './InvoiceForm.svelte';

const MARKER = /Compte invalide/;
const BLOCK_MSG = /Compte de produit invalide sur les lignes suivantes/;

function acc(
	partial: Partial<AccountResponse> & { id: number; number: string; name: string }
): AccountResponse {
	return {
		companyId: 1,
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

/** Compte de produit par défaut de la société — cible du repli D9. */
const DEFAULT_REVENUE = acc({ id: 3000, number: '3000', name: 'Ventes' });
const SERVICES = acc({ id: 3200, number: '3200', name: 'Honoraires' });
const ARCHIVED = acc({ id: 3900, number: '3900', name: 'Ventes closes', active: false });
const COLLECTIVE = acc({ id: 3800, number: '3800', name: 'Groupe produits', postable: false });
const EXPENSE = acc({ id: 4000, number: '4000', name: 'Achats', accountType: 'Expense' });

const ALL_ACCOUNTS = [DEFAULT_REVENUE, SERVICES, ARCHIVED, COLLECTIVE, EXPENSE];

function line(position: number, revenueAccountId: number | null) {
	return {
		id: position,
		invoiceId: 7,
		position,
		description: `Ligne ${position}`,
		quantity: '1',
		unitPrice: '100.00',
		vatRate: '8.10',
		lineTotal: '100.00',
		revenueAccountId,
		createdAt: '',
	};
}

function invoiceWith(accountIds: (number | null)[]): InvoiceResponse {
	return {
		id: 7,
		companyId: 1,
		contactId: 1,
		invoiceNumber: null,
		status: 'draft',
		date: '2026-06-15',
		dueDate: null,
		paymentTerms: null,
		projectId: null,
		totalAmount: '100.00',
		version: 1,
		lines: accountIds.map((a, i) => line(i + 1, a)),
	} as unknown as InvoiceResponse;
}

/**
 * Les champs de COMPTE uniquement.
 *
 * `getAllByRole('textbox')` ramasserait aussi description / quantité / prix.
 * `aria-autocomplete="list"` est porté par le seul `AccountAutocomplete`.
 */
function accountInputs(container: HTMLElement): HTMLInputElement[] {
	return Array.from(container.querySelectorAll('input[aria-autocomplete="list"]'));
}

/** `disabled` en propriété DOM — le projet n'installe pas `jest-dom`. */
function isDisabled(el: HTMLElement): boolean {
	return (el as HTMLButtonElement).disabled;
}

/** Laisse passer les `$effect` asynchrones (comptes, réglages, TVA, projets). */
async function settle() {
	await new Promise((r) => setTimeout(r, 50));
}

beforeEach(() => {
	vi.clearAllMocks();
	fetchAccountsMock.mockResolvedValue(ALL_ACCOUNTS);
	getInvoiceSettingsMock.mockResolvedValue({
		defaultReceivableAccountId: 1100,
		defaultRevenueAccountId: DEFAULT_REVENUE.id,
	});
});

describe('InvoiceForm — chargement des comptes (Story 16-1b, AC5 / D11)', () => {
	it('charge la liste AVEC les comptes archivés — le piège n°1 de la story', async () => {
		render(InvoiceForm, { invoice: invoiceWith([null]) });
		await settle();

		// `fetchAccounts()` sans argument compile et marche… jusqu'au jour où un
		// compte est archivé : son libellé ne se résout plus et le champ paraît
		// vide. C'est ce test qui tombe si quelqu'un « simplifie » l'appel.
		expect(fetchAccountsMock).toHaveBeenCalledWith(true);
	});

	it('AC5-bis : un compte archivé référencé garde son libellé, hors du dropdown', async () => {
		const { container, queryByText } = render(InvoiceForm, {
			invoice: invoiceWith([ARCHIVED.id]),
		});
		await settle();

		expect(accountInputs(container).map((b) => b.value)).toContain('3900 — Ventes closes');

		// Résolu sur la liste complète, mais jamais re-sélectionnable — il faut
		// OUVRIR le dropdown pour l'éprouver. Sans ce `focus`, l'assertion serait
		// vraie même si le compte archivé y était proposé à tort : les items ne
		// sont simplement pas rendus tant que la liste est fermée.
		// (Assertion morte relevée en passe 1 de revue, Blind Hunter.)
		// Vider la requête pour que le dropdown propose la liste complète : le
		// filtre porte sur le texte du champ, qui vaut ici « 3900 — Ventes closes »
		// et ne matcherait aucun compte proposable.
		const field = accountInputs(container)[0];
		await fireEvent.focus(field);
		await fireEvent.input(field, { target: { value: '' } });

		expect(queryByText('Honoraires')).not.toBeNull(); // proposable
		expect(queryByText('Ventes closes')).toBeNull(); // archivé -> jamais proposé
	});

	it('AC7 : un échec de chargement des comptes ne bloque pas la saisie', async () => {
		fetchAccountsMock.mockRejectedValue(new Error('réseau'));
		const { getByTestId } = render(InvoiceForm, { invoice: invoiceWith([null]) });
		await settle();

		expect(isDisabled(getByTestId('create-invoice-button'))).toBe(false);
	});
});

describe('InvoiceForm — repli sur le défaut société (Story 16-1b, AC6 / D9)', () => {
	it('une ligne sans compte nomme le compte par défaut en placeholder', async () => {
		const { container } = render(InvoiceForm, { invoice: invoiceWith([null]) });
		await settle();

		const hinted = accountInputs(container).filter(
			(b) => b.placeholder === '3000 — Ventes (défaut société)'
		);
		expect(hinted).toHaveLength(1);
	});

	it("le placeholder n'est PAS persisté : la ligne reste à null", async () => {
		const { container } = render(InvoiceForm, { invoice: invoiceWith([null]) });
		await settle();

		const hinted = accountInputs(container).find(
			(b) => b.placeholder === '3000 — Ventes (défaut société)'
		);
		// Placeholder ≠ valeur : le champ est vide, la ligne restera `NULL`.
		expect(hinted?.value).toBe('');
	});
});

describe('InvoiceForm — blocage global (Story 16-1b, AC8)', () => {
	it('2 lignes invalides sur 4 : le message les nomme TOUTES LES DEUX', async () => {
		const { getByTestId, getAllByText } = render(InvoiceForm, {
			invoice: invoiceWith([SERVICES.id, ARCHIVED.id, SERVICES.id, EXPENSE.id]),
		});
		await settle();

		// Seules les lignes fautives portent un marqueur (2 sur 4).
		expect(getAllByText(MARKER)).toHaveLength(2);

		const btn = getByTestId('create-invoice-button');
		expect(isDisabled(btn)).toBe(true);
		expect(btn.getAttribute('title')).toMatch(BLOCK_MSG);
		expect(btn.getAttribute('title')).toContain('2, 4');
	});

	it("corriger UNE ligne laisse bloqué, avec un message ne nommant plus que l'autre", async () => {
		const { getByTestId, getAllByLabelText, getAllByText } = render(InvoiceForm, {
			invoice: invoiceWith([SERVICES.id, ARCHIVED.id, SERVICES.id, EXPENSE.id]),
		});
		await settle();

		// Les 4 lignes portent un compte, donc 4 boutons d'effacement : l'index
		// correspond à l'index de ligne. On remet la ligne 2 au défaut société.
		await fireEvent.click(getAllByLabelText(/Effacer le compte/)[1]);
		await settle();

		expect(getAllByText(MARKER)).toHaveLength(1);
		const btn = getByTestId('create-invoice-button');
		expect(isDisabled(btn)).toBe(true);
		expect(btn.getAttribute('title')).toContain('4');
		expect(btn.getAttribute('title')).not.toContain('2, 4');
	});

	it('aucune ligne invalide : le bouton est actif', async () => {
		const { getByTestId, queryByText } = render(InvoiceForm, {
			invoice: invoiceWith([SERVICES.id, null]),
		});
		await settle();

		expect(queryByText(MARKER)).toBeNull();
		expect(isDisabled(getByTestId('create-invoice-button'))).toBe(false);
	});
});

describe('InvoiceForm — exemption du défaut société (Story 16-1b, D11 / miroir 16-1a D3-bis)', () => {
	it('un défaut société non-postable, désigné explicitement, ne bloque PAS', async () => {
		// Le backend accepte ce cas (16-1a D3-bis). Si le frontend le refusait,
		// l'utilisateur serait ENFERMÉ : incapable d'enregistrer son brouillon.
		getInvoiceSettingsMock.mockResolvedValue({
			defaultReceivableAccountId: 1100,
			defaultRevenueAccountId: COLLECTIVE.id,
		});
		const { getByTestId, queryByText } = render(InvoiceForm, {
			invoice: invoiceWith([COLLECTIVE.id]),
		});
		await settle();

		expect(queryByText(MARKER)).toBeNull();
		expect(isDisabled(getByTestId('create-invoice-button'))).toBe(false);
	});

	it("l'exemption ne couvre QUE `postable` : un défaut archivé reste bloquant", async () => {
		getInvoiceSettingsMock.mockResolvedValue({
			defaultReceivableAccountId: 1100,
			defaultRevenueAccountId: ARCHIVED.id,
		});
		const { getByTestId, queryByText } = render(InvoiceForm, {
			invoice: invoiceWith([ARCHIVED.id]),
		});
		await settle();

		expect(queryByText(MARKER)).not.toBeNull();
		expect(isDisabled(getByTestId('create-invoice-button'))).toBe(true);
	});
});

describe('InvoiceForm — réglages indisponibles (Story 16-1b, passe 1 de revue)', () => {
	it("un échec de chargement des réglages ne bloque PAS l'enregistrement", async () => {
		// Régression introduite par la passe d'implémentation : sans réglages,
		// l'exemption `postable` du défaut société est absente, la ligne était
		// marquée invalide, le bouton désactivé — et `onSubmit`, seul chemin qui
		// rafraîchit les réglages, ne pouvait plus s'exécuter. Utilisateur ENFERMÉ.
		getInvoiceSettingsMock.mockRejectedValue(new Error('réseau'));
		const { getByTestId } = render(InvoiceForm, {
			invoice: invoiceWith([COLLECTIVE.id]),
		});
		await settle();

		expect(isDisabled(getByTestId('create-invoice-button'))).toBe(false);
		// Et l'échec est VISIBLE : il était affecté mais rendu nulle part.
		expect(getByTestId('invoice-settings-error')).toBeTruthy();
	});
});

describe('InvoiceForm — défaut société inutilisable (Story 16-1b, passe 1 de revue)', () => {
	it('un défaut société archivé est SIGNALÉ, sans bloquer', async () => {
		// Les lignes à `null` le suivent sans le nommer : elles échappaient au
		// contrôle par ligne, et l'utilisateur ne découvrait le problème qu'à la
		// validation, rejetée par le backend. On signale ; on ne bloque pas, car
		// le backend accepte l'enregistrement du brouillon.
		getInvoiceSettingsMock.mockResolvedValue({
			defaultReceivableAccountId: 1100,
			defaultRevenueAccountId: ARCHIVED.id,
		});
		const { getByTestId } = render(InvoiceForm, { invoice: invoiceWith([null]) });
		await settle();

		expect(getByTestId('invoice-default-account-warning')).toBeTruthy();
		expect(isDisabled(getByTestId('create-invoice-button'))).toBe(false);
	});

	it('un défaut société sain ne déclenche aucun avertissement', async () => {
		const { queryByTestId } = render(InvoiceForm, { invoice: invoiceWith([null]) });
		await settle();
		expect(queryByTestId('invoice-default-account-warning')).toBeNull();
	});
});

describe('InvoiceForm — cycle de vie des lignes (Story 16-1b, AC9 / AC9-bis)', () => {
	it("ajouter une ligne libre n'altère pas le compte des lignes existantes", async () => {
		const { getByText, container } = render(InvoiceForm, {
			invoice: invoiceWith([SERVICES.id]),
		});
		await settle();

		await fireEvent.click(getByText('Ligne libre'));
		await settle();

		const values = accountInputs(container).map((b) => b.value);
		// Le compte de la ligne d'origine survit, la nouvelle ligne naît à `null`.
		expect(values).toContain('3200 — Honoraires');
		expect(values.filter((v) => v === '3200 — Honoraires')).toHaveLength(1);
	});

	it("supprimer une ligne n'altère pas le compte des lignes restantes (AC9)", async () => {
		const { getAllByLabelText, container } = render(InvoiceForm, {
			invoice: invoiceWith([SERVICES.id, null, ARCHIVED.id]),
		});
		await settle();

		// Supprimer la ligne du MILIEU : c'est le cas où un keying par index
		// ferait glisser les comptes d'un cran.
		await fireEvent.click(getAllByLabelText('Supprimer la ligne')[1]);
		await settle();

		const values = accountInputs(container).map((b) => b.value);
		expect(values).toEqual(['3200 — Honoraires', '3900 — Ventes closes']);
	});

	it('AC9-bis : après « Recharger », les comptes viennent du SERVEUR', async () => {
		// LE site le plus dangereux des 4 : un oubli dans `reloadFromServer` n'est
		// visible qu'APRÈS un conflit de version. Chemin complet exercé ici —
		// conflit optimiste → modale → « Recharger » → remappage des lignes. Sans
		// la recopie de `revenueAccountId`, les ventilations retombent à `NULL` en
		// silence et le ré-enregistrement les efface en base ; `stripUiKey` étant
		// un spread, rien ne le signale.
		updateInvoiceMock.mockRejectedValue({
			code: 'OPTIMISTIC_LOCK_CONFLICT',
			status: 409,
			message: 'conflit',
		});
		getInvoiceMock.mockResolvedValue(invoiceWith([ARCHIVED.id, SERVICES.id]));

		const { getByTestId, getByText, container, getAllByText } = render(InvoiceForm, {
			invoice: invoiceWith([null, null]),
		});
		await settle();

		// Départ : aucun champ de compte n'est renseigné.
		expect(accountInputs(container).map((b) => b.value)).toEqual(['', '']);

		// Le PUT part et échoue en 409 → la modale de conflit s'ouvre.
		await fireEvent.click(getByTestId('create-invoice-button'));
		await settle();
		expect(updateInvoiceMock).toHaveBeenCalled();

		await fireEvent.click(getByText('Recharger'));
		await settle();

		const values = accountInputs(container).map((b) => b.value);
		expect(values).toContain('3900 — Ventes closes');
		expect(values).toContain('3200 — Honoraires');
		// Et le compte archivé rechargé est bien signalé.
		expect(getAllByText(MARKER)).toHaveLength(1);
	});
});
