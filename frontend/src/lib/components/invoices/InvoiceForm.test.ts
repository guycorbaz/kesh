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

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
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
	// v0190-correction (#1162, AC2) : la frappe dans le champ client lance une recherche.
	listContacts: vi.fn(async () => ({ items: [], total: 0, offset: 0, limit: 50 })),
}));

vi.mock('$lib/features/vat-rates', () => ({
	getVatRates: vi.fn(async () => [{ rate: '8.10' }, { rate: '2.60' }]),
}));

vi.mock('$lib/features/projects/projects.api', () => ({
	listProjects: vi.fn(async () => []),
}));

// Story 16-2b (#144) — le catalogue, consommé par `ProductPicker`.
const listProductsMock = vi.fn();
vi.mock('$lib/features/products/products.api', () => ({
	listProducts: (q?: unknown) => listProductsMock(q),
}));

vi.mock('$app/navigation', () => ({ goto: vi.fn() }));
vi.mock('$lib/shared/utils/notify', () => ({ notifyError: vi.fn(), notifySuccess: vi.fn() }));

import InvoiceForm from './InvoiceForm.svelte';
import { pinClock } from '$lib/shared/utils/pin-clock.test-helper';
import * as contactsApi from '$lib/features/contacts/contacts.api';
import type { ContactResponse } from '$lib/features/contacts/contacts.types';

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
	listProductsMock.mockResolvedValue({ items: [], total: 0, limit: 20, offset: 0 });
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

		// AC8 exige que le message soit ATTEIGNABLE. Un bouton `disabled` porte
		// `pointer-events-none` : son `title` ne s'affiche jamais et l'élément
		// n'est pas focusable. Le message doit donc être rendu. (Passe 2 de revue.)
		const rendered = getByTestId('invoice-lines-invalid-error');
		expect(rendered.textContent).toMatch(BLOCK_MSG);
		expect(rendered.textContent).toContain('2, 4');
		expect(rendered.getAttribute('role')).toBe('alert');
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

	it("…et le BANDEAU global ne s'affiche pas non plus pour ce défaut non-postable", async () => {
		// Le backend accepte ce compte (16-1a D3-bis). Annoncer « ne pourront pas
		// être validées » serait FAUX — et contredirait la ligne qui désigne le
		// même compte sans marqueur. C'est l'incohérence que la passe 1 a créée en
		// oubliant l'exemption sur le bandeau. (Passe 2 de revue.)
		getInvoiceSettingsMock.mockResolvedValue({
			defaultReceivableAccountId: 1100,
			defaultRevenueAccountId: COLLECTIVE.id,
		});
		const { queryByTestId } = render(InvoiceForm, { invoice: invoiceWith([COLLECTIVE.id]) });
		await settle();
		expect(queryByTestId('invoice-default-account-warning')).toBeNull();
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

describe('InvoiceForm — le CRITICAL de la passe 2 : vider puis enregistrer', () => {
	it("vider un compte puis enregistrer IMMÉDIATEMENT persiste `null`, pas l'ancien compte", async () => {
		// L'ancienne conception différait la réconciliation de 150 ms. Séquence
		// réelle : `blur` (timer armé) -> `click` -> `onSubmit` lit les lignes DANS
		// LA MÊME TÂCHE, où le compte valait encore l'ancien. Le `PUT` partait avec
		// l'ancienne valeur pendant que l'écran affichait « suit le défaut
		// société », et la redirection démontait le composant avant l'échéance du
		// timer. Écriture comptable fausse, en silence.
		updateInvoiceMock.mockResolvedValue({ id: 7 });
		const { container, getByTestId } = render(InvoiceForm, {
			invoice: invoiceWith([SERVICES.id]),
		});
		await settle();

		const field = accountInputs(container)[0];
		expect(field.value).toBe('3200 — Honoraires');

		// Pas de `settle()` entre le blur et le clic : c'est tout l'enjeu.
		await fireEvent.input(field, { target: { value: '' } });
		await fireEvent.blur(field);
		await fireEvent.click(getByTestId('create-invoice-button'));
		await settle();

		expect(updateInvoiceMock).toHaveBeenCalled();
		const payload = updateInvoiceMock.mock.calls[0][1] as {
			lines: { revenueAccountId: number | null }[];
		};
		expect(payload.lines[0].revenueAccountId).toBeNull();
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

	it('marqueur et blocage se taisent ENSEMBLE quand les réglages manquent', async () => {
		// La passe 1 n'avait désarmé que le blocage : le champ criait « Compte
		// invalide » pendant que le bouton restait actif — l'incohérence que
		// `account-validity.ts` déclare pire que l'absence de fonctionnalité.
		getInvoiceSettingsMock.mockRejectedValue(new Error('réseau'));
		const { queryByText, getByTestId } = render(InvoiceForm, {
			invoice: invoiceWith([COLLECTIVE.id]),
		});
		await settle();

		expect(queryByText(MARKER)).toBeNull();
		expect(isDisabled(getByTestId('create-invoice-button'))).toBe(false);
	});

	it("après un échec de réglages, un clic sur Enregistrer ABOUTIT", async () => {
		// La garde de la passe 1 débloquait le bouton mais l'enfermement survivait
		// dans `onSubmit` : `freshSettings.X !== invoiceSettings?.X` avec
		// `invoiceSettings` à `null` compare à `undefined`, donc TOUJOURS vrai —
		// tout submit retournait « Rechargez la page ». Le test de la passe 1
		// s'arrêtait au bouton cliquable, sans jamais vérifier qu'un clic aboutit.
		getInvoiceSettingsMock.mockRejectedValue(new Error('réseau'));
		updateInvoiceMock.mockResolvedValue({ id: 7 });
		const { getByTestId } = render(InvoiceForm, { invoice: invoiceWith([null]) });
		await settle();

		// Le rafraîchissement des réglages dans `onSubmit` réussit cette fois.
		getInvoiceSettingsMock.mockResolvedValue({
			defaultReceivableAccountId: 1100,
			defaultRevenueAccountId: DEFAULT_REVENUE.id,
		});
		await fireEvent.click(getByTestId('create-invoice-button'));
		await settle();

		expect(updateInvoiceMock).toHaveBeenCalled();
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
		await fireEvent.click(getAllByLabelText('Supprimer la ligne 2')[0]);
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

describe('InvoiceForm — pré-remplissage depuis le catalogue (Story 16-2b, AC-B3)', () => {
	/** Article du catalogue, avec ou sans compte de produit. */
	function product(id: number, name: string, account: number | null) {
		return {
			id,
			companyId: 1,
			name,
			description: null,
			unitPrice: '250.00',
			vatRate: '8.10',
			defaultRevenueAccountId: account,
			active: true,
			version: 1,
			createdAt: '2026-01-01T00:00:00',
			updatedAt: '2026-01-01T00:00:00',
		};
	}

	/**
	 * Ouvre le picker, choisit le premier article proposé, et **vérifie qu'une
	 * ligne a bien été ajoutée** — en rendant le décompte avant/après.
	 *
	 * ⚠️ Rendre le décompte n'est pas une commodité : c'est ce qui empêche le
	 * test négatif de passer à vide. Si le picker cesse d'émettre sa sélection,
	 * aucune ligne n'est ajoutée, la dernière ligne préexistante est vide, et
	 * une assertion `value === ''` **passe** sur un pré-remplissage qui n'a pas
	 * eu lieu. *(Passe 1 de revue.)*
	 */
	async function pickFirstProduct(container: HTMLElement, getByText: (t: string) => HTMLElement) {
		const before = accountInputs(container).length;
		await fireEvent.click(getByText('Depuis catalogue'));
		await settle();
		// Le dialog du picker est rendu en PORTAL, hors du conteneur du
		// composant : le chercher dans `container` ne rendrait jamais rien.
		//
		// ⚠️ Ancré sur `[role="dialog"]` : un `document.querySelector('ul li
		// button')` global ramasserait le premier `<ul>` monté n'importe où —
		// menu, liste de suggestions d'un autre champ — et cliquerait à côté.
		const picker = document.querySelector('[role="dialog"]');
		expect(picker, 'le dialog du picker doit être monté').toBeTruthy();
		const entry = picker!.querySelector('ul li button') as HTMLElement | null;
		expect(entry, 'le picker doit proposer au moins un article').toBeTruthy();
		await fireEvent.click(entry!);
		await settle();

		const after = accountInputs(container).length;
		expect(after, 'le choix dans le catalogue doit AJOUTER une ligne').toBe(before + 1);
		return { before, after };
	}

	it("la ligne créée porte le compte de l'article", async () => {
		listProductsMock.mockResolvedValue({
			items: [product(1, 'Prestation conseil', SERVICES.id)],
			total: 1,
			limit: 20,
			offset: 0,
		});
		const { container, getByText } = render(InvoiceForm, { invoice: invoiceWith([]) });
		await settle();

		await pickFirstProduct(container, getByText);

		const inputs = accountInputs(container);
		// C'est la valeur du compte de l'ARTICLE qui doit apparaître, pas un
		// champ vide retombé en silence sur le défaut société.
		expect(inputs[inputs.length - 1].value).toContain(SERVICES.number);
	});

	it("un article SANS compte laisse la ligne au défaut société, sans rien inventer", async () => {
		listProductsMock.mockResolvedValue({
			items: [product(2, 'Article neutre', null)],
			total: 1,
			limit: 20,
			offset: 0,
		});
		const { container, getByText } = render(InvoiceForm, { invoice: invoiceWith([]) });
		await settle();

		await pickFirstProduct(container, getByText);

		const inputs = accountInputs(container);
		const last = inputs[inputs.length - 1];
		// `null` = suivre le défaut société : le champ reste vide de valeur, et
		// c'est le placeholder qui nomme le compte par défaut (D9 de 16-1b).
		expect(last.value).toBe('');
	});
});

// Story v0171 (#878, AC-11) — les champs quantité et prix unitaire d'une ligne de facture
// cliente portent la forme lisible de la valeur du serveur (`DECIMAL(19,4)`) : quantité
// `readableAmount(q, 0)` (`1.0000` ⇒ `1`), prix `readableAmount(p, 2)` (`100.0000` ⇒
// `100.00`), sans perte de précision (`33.3333` gardé). Trois sites : `initLines`
// (ouverture d'un brouillon), `reloadInvoice` (« Recharger » après un conflit de
// version), `onProductSelect` (ajout depuis un article). Mutations M6 / M6-ter : retirer
// le helper d'un site rougit le seul test de ce site.
describe('InvoiceForm — quantité et prix lisibles (v0171, #878)', () => {
	function ligneServeur(position: number, quantity: string, unitPrice: string) {
		return { ...line(position, null), quantity, unitPrice };
	}

	function brouillon(lignes: ReturnType<typeof ligneServeur>[]): InvoiceResponse {
		return { ...invoiceWith([]), lines: lignes } as unknown as InvoiceResponse;
	}

	/** Les champs quantité et prix, dans l'ordre des lignes : `[[q, p], …]`. */
	function champs(container: HTMLElement): [string, string][] {
		const inputs = Array.from(
			container.querySelectorAll<HTMLInputElement>(
				'[data-testid="invoice-lines-table"] input[inputmode="decimal"]',
			),
		).map((i) => i.value);
		const paires: [string, string][] = [];
		for (let k = 0; k < inputs.length; k += 2) paires.push([inputs[k], inputs[k + 1]]);
		return paires;
	}

	function payloadLines(): { quantity: string; unitPrice: string }[] {
		return (updateInvoiceMock.mock.calls.at(-1)![1] as { lines: { quantity: string; unitPrice: string }[] })
			.lines;
	}

	it('ouverture d’un brouillon : `1.0000` ⇒ `1`, `100.0000` ⇒ `100.00`, `100.2000` ⇒ `100.20`, `33.3333` gardé (mutation M6 / M6-ter : `initLines`)', async () => {
		const { container } = render(InvoiceForm, {
			invoice: brouillon([
				ligneServeur(1, '1.0000', '100.0000'),
				ligneServeur(2, '2.5000', '100.2000'),
				ligneServeur(3, '33.3333', '33.3333'),
				ligneServeur(4, '0.0125', '0.0100'),
			]),
		});
		await settle();
		expect(champs(container)).toEqual([
			['1', '100.00'],
			['2.5', '100.20'],
			['33.3333', '33.3333'],
			['0.0125', '0.01'],
		]);
	});

	it('après « Recharger » sur conflit de version : `1` et `100.00` (mutation M6 / M6-ter : `reloadInvoice`)', async () => {
		updateInvoiceMock.mockRejectedValue({
			code: 'OPTIMISTIC_LOCK_CONFLICT',
			status: 409,
			message: 'conflit',
		});
		getInvoiceMock.mockResolvedValue(
			brouillon([ligneServeur(1, '1.0000', '100.0000'), ligneServeur(2, '33.3333', '33.3333')]),
		);
		// Départ déjà lisible : seul `reloadInvoice` peut produire la forme brute.
		const { container, getByTestId, getByText } = render(InvoiceForm, {
			invoice: brouillon([ligneServeur(1, '1', '100.00')]),
		});
		await settle();
		await fireEvent.click(getByTestId('create-invoice-button'));
		await settle();
		await fireEvent.click(getByText('Recharger'));
		await settle();
		expect(champs(container)).toEqual([
			['1', '100.00'],
			['33.3333', '33.3333'],
		]);
	});

	it('ajout d’une ligne depuis un article de prix `100.0000` ⇒ prix `100.00` (mutation M6-ter : `onProductSelect`)', async () => {
		listProductsMock.mockResolvedValue({
			items: [
				{
					id: 1,
					companyId: 1,
					name: 'Article à quatre décimales',
					description: null,
					unitPrice: '100.0000',
					vatRate: '8.10',
					defaultRevenueAccountId: null,
					active: true,
					version: 1,
					createdAt: '2026-01-01T00:00:00',
					updatedAt: '2026-01-01T00:00:00',
				},
			],
			total: 1,
			limit: 20,
			offset: 0,
		});
		const { container, getByText } = render(InvoiceForm, { invoice: invoiceWith([]) });
		await settle();
		await fireEvent.click(getByText('Depuis catalogue'));
		await settle();
		const picker = document.querySelector('[role="dialog"]');
		const entry = picker?.querySelector('ul li button') as HTMLElement | null;
		expect(entry, 'le picker doit proposer l’article').toBeTruthy();
		await fireEvent.click(entry!);
		await settle();
		expect(champs(container).at(-1)).toEqual(['1', '100.00']);
	});

	it('aller-retour : enregistrer sans toucher envoie `"1"` et `"100.00"`, le total ne change pas', async () => {
		updateInvoiceMock.mockResolvedValue({ id: 7 });
		const { container, getByTestId } = render(InvoiceForm, {
			invoice: brouillon([ligneServeur(1, '1.0000', '100.0000')]),
		});
		await settle();
		const total = () => container.querySelector('tfoot')?.textContent ?? '';
		const avant = total();
		expect(avant).toContain('100.00');
		await fireEvent.click(getByTestId('create-invoice-button'));
		await settle();
		expect(payloadLines()[0]).toMatchObject({ quantity: '1', unitPrice: '100.00' });
		expect(total()).toBe(avant);
	});

	it('aller-retour : `33.3333` part tel quel, aucune précision perdue', async () => {
		updateInvoiceMock.mockResolvedValue({ id: 7 });
		const { getByTestId } = render(InvoiceForm, {
			invoice: brouillon([ligneServeur(1, '33.3333', '33.3333')]),
		});
		await settle();
		await fireEvent.click(getByTestId('create-invoice-button'));
		await settle();
		expect(payloadLines()[0]).toMatchObject({ quantity: '33.3333', unitPrice: '33.3333' });
	});

	it('aller-retour : saisir `2.5` dans la quantité l’envoie `"2.5"`', async () => {
		updateInvoiceMock.mockResolvedValue({ id: 7 });
		const { container, getByTestId } = render(InvoiceForm, {
			invoice: brouillon([ligneServeur(1, '1.0000', '10.0000')]),
		});
		await settle();
		const qte = container.querySelectorAll<HTMLInputElement>(
			'[data-testid="invoice-lines-table"] input[inputmode="decimal"]',
		)[0];
		await fireEvent.input(qte, { target: { value: '2.5' } });
		await fireEvent.click(getByTestId('create-invoice-button'));
		await settle();
		expect(payloadLines()[0]).toMatchObject({ quantity: '2.5' });
	});
});

// v0172-correctifs (#882, C-v0172b-9, C-v0172b-27) — une ligne dont le taux de TVA a été
// désactivé : le `<select>` le propose, libellé « (désactivé) », le refus dit de choisir
// un taux actif, et le repli `8.10` ne touche plus que les lignes neuves.
describe('InvoiceForm — taux de TVA désactivé (v0172, #882)', () => {
	const ACTIFS = [{ rate: '8.10' }, { rate: '2.60' }, { rate: '0.00' }];

	function brouillonAuTaux(rate: string): InvoiceResponse {
		return { ...invoiceWith([null]), lines: [{ ...line(1, null), vatRate: rate }] } as unknown as InvoiceResponse;
	}
	function selects(container: HTMLElement): HTMLSelectElement[] {
		return Array.from(container.querySelectorAll('[data-testid="invoice-line-vat-rate"]'));
	}
	function optionLabels(s: HTMLSelectElement): string[] {
		return Array.from(s.options).map((o) => o.textContent?.trim() ?? '');
	}
	const DISABLED_MSG = 'Ligne 1 : le taux de TVA 7.70% a été désactivé. Choisissez un taux actif.';

	beforeEach(async () => {
		const { getVatRates } = await import('$lib/features/vat-rates');
		vi.mocked(getVatRates).mockImplementation(async () => ACTIFS as never);
	});
	afterEach(async () => {
		const { getVatRates } = await import('$lib/features/vat-rates');
		vi.mocked(getVatRates).mockImplementation(async () => [{ rate: '8.10' }, { rate: '2.60' }] as never);
	});

	it('AC-B7 — un brouillon à 7.70 s’ouvre sur `7.70% (désactivé)` ; Enregistrer refuse en le disant (rouge avant correctif)', async () => {
		const { container, getByTestId } = render(InvoiceForm, { invoice: brouillonAuTaux('7.70') });
		await settle();
		const [s] = selects(container);
		expect(s.value).toBe('7.70');
		expect(optionLabels(s)).toContain('7.70% (désactivé)');
		await fireEvent.click(getByTestId('create-invoice-button'));
		await settle();
		expect(updateInvoiceMock).not.toHaveBeenCalled();
		expect(container.textContent).toContain(DISABLED_MSG);
		expect(container.textContent).not.toContain('taux TVA invalide');
	});

	it('AC-B7 — choisir 8.10 retire le libellé et le refus ; l’envoi porte `vatRate: "8.10"`', async () => {
		updateInvoiceMock.mockResolvedValue({ id: 7 });
		const { container, getByTestId } = render(InvoiceForm, { invoice: brouillonAuTaux('7.70') });
		await settle();
		const [s] = selects(container);
		await fireEvent.change(s, { target: { value: '8.10' } });
		await settle();
		expect(optionLabels(selects(container)[0]).some((l) => l.includes('(désactivé)'))).toBe(false);
		await fireEvent.click(getByTestId('create-invoice-button'));
		await settle();
		expect(container.textContent).not.toContain(DISABLED_MSG);
		const sent = updateInvoiceMock.mock.calls.at(-1)![1] as { lines: Record<string, unknown>[] };
		expect(sent.lines[0].vatRate).toBe('8.10');
		expect(Object.keys(sent.lines[0]).filter((k) => k.startsWith('_')), 'aucune marque d’interface envoyée').toEqual([]);
	});

	it('AC-B7 — après « Recharger » (conflit de version), la ligne à 7.70 garde son taux, désactivé', async () => {
		updateInvoiceMock.mockRejectedValue({ code: 'OPTIMISTIC_LOCK_CONFLICT', status: 409, message: 'conflit' });
		getInvoiceMock.mockResolvedValue(brouillonAuTaux('7.70'));
		const { container, getByTestId, getByText } = render(InvoiceForm, {
			invoice: brouillonAuTaux('8.10'),
		});
		await settle();
		await fireEvent.click(getByTestId('create-invoice-button'));
		await settle();
		await fireEvent.click(getByText('Recharger'));
		await settle();
		const [s] = selects(container);
		expect(s.value).toBe('7.70');
		expect(optionLabels(s)).toContain('7.70% (désactivé)');
	});

	it('AC-B7 — une ligne ajoutée depuis un article à 7.70 propose `7.70% (désactivé)` (MB4)', async () => {
		listProductsMock.mockResolvedValue({
			items: [
				{
					id: 1,
					companyId: 1,
					name: 'Article à l’ancien taux',
					description: null,
					unitPrice: '10.00',
					vatRate: '7.70',
					defaultRevenueAccountId: null,
					active: true,
					version: 1,
					createdAt: '2026-01-01T00:00:00',
					updatedAt: '2026-01-01T00:00:00',
				},
			],
			total: 1,
			limit: 20,
			offset: 0,
		});
		const { container, getByText } = render(InvoiceForm, { invoice: invoiceWith([null]) });
		await settle();
		await fireEvent.click(getByText('Depuis catalogue'));
		await settle();
		const entry = document.querySelector('[role="dialog"] ul li button') as HTMLElement | null;
		expect(entry, 'le picker doit proposer l’article').toBeTruthy();
		await fireEvent.click(entry!);
		await settle();
		const s = selects(container).at(-1)!;
		expect(s.value).toBe('7.70');
		expect(optionLabels(s)).toContain('7.70% (désactivé)');
	});

	it('AC-B8 — une ligne à taux actif : aucune option « (désactivé) »', async () => {
		const { container } = render(InvoiceForm, { invoice: brouillonAuTaux('2.60') });
		await settle();
		const [s] = selects(container);
		expect(s.value).toBe('2.60');
		expect(optionLabels(s)).toEqual(['8.10%', '2.60%', '0.00%']);
	});

	it('AC-B8 — cas 8.10 : un brouillon à 8.10, taux désactivé, n’est PAS réécrit en 2.60 (MB9 ; rouge avant correctif)', async () => {
		const { getVatRates } = await import('$lib/features/vat-rates');
		vi.mocked(getVatRates).mockImplementation(async () => [{ rate: '2.60' }, { rate: '0.00' }] as never);
		const { container } = render(InvoiceForm, { invoice: brouillonAuTaux('8.10') });
		await settle();
		const [s] = selects(container);
		expect(s.value).toBe('8.10');
		expect(optionLabels(s)).toContain('8.10% (désactivé)');
	});

	it('AC-B8 — cas 8.10 : une ligne NEUVE créée avant la résolution des taux est repointée sur 2.60', async () => {
		const { getVatRates } = await import('$lib/features/vat-rates');
		let resolve: (v: unknown) => void = () => {};
		vi.mocked(getVatRates).mockImplementation(
			() => new Promise((r) => (resolve = r as (v: unknown) => void)) as never,
		);
		const { container } = render(InvoiceForm, {});
		await settle();
		resolve([{ rate: '2.60' }, { rate: '0.00' }]);
		await settle();
		const [s] = selects(container);
		expect(s.value).toBe('2.60');
		expect(optionLabels(s).some((l) => l.includes('(désactivé)'))).toBe(false);
	});
});

// Story v0174-correctifs-3 (#889, AC-3.6, site 1) — la date par défaut d'une facture neuve
// est celle du poste : à 00 h 30 le 5 novembre à Zurich, le 5, non la veille (UTC).
describe('InvoiceForm — date par défaut locale (#889)', () => {
	pinClock();
	it('une facture neuve est datée du 5 novembre', async () => {
		const { container } = render(InvoiceForm, {});
		await settle();
		expect((container.querySelector('#invoice-date') as HTMLInputElement).value).toBe(
			'2026-11-05',
		);
	});
});

// Story 36-1 (#1095, #1165, AC3) — la touche Entrée dans le champ « Compte de
// produit » d'une ligne. jsdom n'implémente pas la soumission implicite d'un
// `<form>` : `entreeSimulee` la rejoue comme le navigateur — si l'événement
// n'a pas été empêché, le formulaire est soumis.
describe('InvoiceForm — Entrée dans le compte de produit (Story 36-1)', () => {
	/** *Entrée* dans `champ`, puis la soumission implicite si rien ne l'a empêchée. */
	async function entreeSimulee(champ: HTMLInputElement): Promise<boolean> {
		const nonEmpechee = await fireEvent.keyDown(champ, { key: 'Enter' });
		if (nonEmpechee) champ.form!.requestSubmit();
		await settle();
		return nonEmpechee;
	}

	function compteEnvoye(i: number): number | null | undefined {
		const payload = updateInvoiceMock.mock.calls[0][1] as {
			lines: { revenueAccountId: number | null }[];
		};
		return payload.lines[i].revenueAccountId;
	}

	it('a. (#1095) texte sans correspondance, Entrée : rien ne part, le champ montre la saisie', async () => {
		updateInvoiceMock.mockResolvedValue({ id: 7 });
		const { container } = render(InvoiceForm, { invoice: invoiceWith([SERVICES.id]) });
		await settle();
		const field = accountInputs(container)[0];
		expect(field.value).toBe('3200 — Honoraires');

		await fireEvent.input(field, { target: { value: 'zzz' } });
		const nonEmpechee = await entreeSimulee(field);

		// L'envoi d'abord : c'est lui, le défaut (#1095) — l'ancien compte partait.
		expect(updateInvoiceMock).not.toHaveBeenCalled();
		expect(nonEmpechee).toBe(false);
		expect(field.value).toBe('zzz');
	});

	it('b. numéro exact, Entrée (choisit), Entrée (envoie) : le compte choisi part', async () => {
		updateInvoiceMock.mockResolvedValue({ id: 7 });
		const { container } = render(InvoiceForm, { invoice: invoiceWith([SERVICES.id]) });
		await settle();
		const field = accountInputs(container)[0];

		await fireEvent.input(field, { target: { value: '3000' } });
		expect(await entreeSimulee(field)).toBe(false);
		expect(updateInvoiceMock).not.toHaveBeenCalled();
		expect(field.value).toBe('3000 — Ventes');

		expect(await entreeSimulee(field)).toBe(true);
		expect(updateInvoiceMock).toHaveBeenCalledTimes(1);
		expect(compteEnvoye(0)).toBe(DEFAULT_REVENUE.id);
	});

	it('c. (#1165) ligne sans compte, focus, Entrée : le formulaire part avec `null`, non le premier compte', async () => {
		updateInvoiceMock.mockResolvedValue({ id: 7 });
		const { container } = render(InvoiceForm, { invoice: invoiceWith([null]) });
		await settle();
		const field = accountInputs(container)[0];
		expect(field.value).toBe('');

		await fireEvent.focus(field);
		const nonEmpechee = await entreeSimulee(field);

		// L'envoi d'abord (#1165) : aujourd'hui, la première Entrée choisit le
		// premier compte de produit et rien ne part.
		expect(updateInvoiceMock).toHaveBeenCalledTimes(1);
		expect(compteEnvoye(0)).toBeNull();
		expect(nonEmpechee).toBe(true);
		expect(field.value).toBe('');
	});
});

// Story v0190-correction (#1162, AC2 et AC7) — D4, décision de Guy du 2026-11-17 : dans
// le formulaire de facture, un texte tapé dans le champ client et non choisi n'est PAS
// jeté au départ du champ ; l'enregistrement est refusé tant qu'il est affiché, par le
// bouton comme par la soumission implicite (*Entrée*), avec « Choisissez le client dans
// la liste ». Un brouillon blanc, lui, est jeté au départ (D1) : le champ remontre X.
describe('InvoiceForm — le client affiché est celui qui part (v0190-correction, #1162)', () => {
	async function monterFacture() {
		updateInvoiceMock.mockResolvedValue({ id: 7 });
		const r = render(InvoiceForm, { invoice: invoiceWith([SERVICES.id]) });
		await settle();
		const champ = r.container.querySelector('input[role="combobox"]') as HTMLInputElement;
		expect(champ.value).toBe('Client');
		return { ...r, champ };
	}

	it('AC7 — « zzz » tapé, champ quitté, Enregistrer : rien ne part, le message le dit, le texte reste', async () => {
		const { champ, getByTestId, container } = await monterFacture();
		await fireEvent.input(champ, { target: { value: 'zzz' } });
		// En jsdom le clic ne déplace pas le focus : le départ du champ est explicite.
		await fireEvent.blur(champ);
		await fireEvent.click(getByTestId('create-invoice-button'));
		await settle();

		expect(updateInvoiceMock).not.toHaveBeenCalled();
		expect(container.textContent).toContain('Choisissez le client dans la liste');
		expect(champ.value).toBe('zzz');
	});

	it('AC7 — même refus par la soumission implicite (Entrée dans un autre champ du formulaire)', async () => {
		const { champ } = await monterFacture();
		await fireEvent.input(champ, { target: { value: 'zzz' } });
		await fireEvent.blur(champ);
		champ.form!.requestSubmit();
		await settle();

		expect(updateInvoiceMock).not.toHaveBeenCalled();
		expect(champ.value).toBe('zzz');
	});

	it('AC2 — champ vidé au clavier puis quitté : le brouillon blanc est jeté, le client X part et s’affiche', async () => {
		const { champ, getByTestId } = await monterFacture();
		await fireEvent.input(champ, { target: { value: '' } });
		await fireEvent.blur(champ);
		expect(champ.value).toBe('Client');
		await fireEvent.click(getByTestId('create-invoice-button'));
		await settle();

		expect(updateInvoiceMock).toHaveBeenCalledTimes(1);
		expect((updateInvoiceMock.mock.calls[0][1] as { contactId: number }).contactId).toBe(1);
	});

	it('F1 — après le refus, le texte tapé tombe (champ vidé puis quitté) : le bandeau du refus se retire', async () => {
		const { champ, getByTestId, queryByTestId } = await monterFacture();
		await fireEvent.input(champ, { target: { value: 'zzz' } });
		await fireEvent.blur(champ);
		await fireEvent.click(getByTestId('create-invoice-button'));
		await settle();
		expect(getByTestId('invoice-form-error').textContent).toContain('Choisissez le client dans la liste');

		await fireEvent.input(champ, { target: { value: '' } });
		await fireEvent.blur(champ);
		await settle();
		expect(champ.value).toBe('Client');
		expect(queryByTestId('invoice-form-error')).toBeNull();
	});

	it('F2 — un texte tapé pendant l’attente des réglages arrête l’envoi', async () => {
		const { champ, getByTestId } = await monterFacture();
		let tenir!: (v: unknown) => void;
		getInvoiceSettingsMock.mockImplementationOnce(
			() => new Promise((res) => (tenir = res)),
		);
		await fireEvent.click(getByTestId('create-invoice-button'));
		// L'envoi attend les réglages ; le champ client est encore éditable.
		await fireEvent.input(champ, { target: { value: 'zzz' } });
		await settle();
		tenir({ defaultReceivableAccountId: 1100, defaultRevenueAccountId: DEFAULT_REVENUE.id });
		await settle();

		expect(updateInvoiceMock).not.toHaveBeenCalled();
		expect(getByTestId('invoice-form-error').textContent).toContain('Choisissez le client dans la liste');
		expect(champ.value).toBe('zzz');
	});
});

// Story 36-3b (#1171) — le prix unitaire d'une ligne de facture client se lit comme le
// montant d'une écriture (`readJournalAmount`) : « 1.250 » est refusé comme ambigu,
// « 1'250 » accepté et envoyé `1250` ; un prix enregistré au millième se rouvre sur
// quatre décimales (`1.2340`) et n'est pas refusé à tort. `describe` à part, en fin de
// fichier (fichier commun avec la 36-1).
describe('InvoiceForm — prix unitaire ambigu (36-3b, #1171)', () => {
	const AMBIGU = 'Ligne 1 : prix unitaire ambigu';
	const ILLISIBLE = 'Ligne 1 : prix unitaire illisible';

	function ligneAuPrix(position: number, unitPrice: string, quantity = '1') {
		return { ...line(position, null), quantity, unitPrice };
	}
	function brouillonAuxPrix(lignes: ReturnType<typeof ligneAuPrix>[]): InvoiceResponse {
		return { ...invoiceWith([]), lines: lignes } as unknown as InvoiceResponse;
	}
	function decimaux(container: HTMLElement): HTMLInputElement[] {
		return Array.from(
			container.querySelectorAll<HTMLInputElement>(
				'[data-testid="invoice-lines-table"] input[inputmode="decimal"]',
			),
		);
	}
	const prix = (container: HTMLElement, i = 0) => decimaux(container)[2 * i + 1];
	const quantite = (container: HTMLElement, i = 0) => decimaux(container)[2 * i];
	/** La classe `border-destructive` elle-même : la classe de base de `Input` porte
	 *  `aria-invalid:border-destructive`, qu'un `toContain` sur `className` confondrait. */
	const borde = (el: HTMLElement) => el.classList.contains('border-destructive');
	const ambigus = (container: HTMLElement) =>
		container.querySelectorAll('[data-testid="invoice-line-unit-price-ambiguous"]').length;
	const prixAffiches = (container: HTMLElement) =>
		decimaux(container)
			.filter((_, k) => k % 2 === 1)
			.map((i) => i.value);
	const totauxDeLigne = (container: HTMLElement) =>
		Array.from(container.querySelectorAll('[data-testid="invoice-lines-table"] tbody td.font-mono')).map(
			(td) => td.textContent?.trim() ?? '',
		);
	const totalFacture = (container: HTMLElement) =>
		container.querySelector('[data-testid="invoice-lines-table"] tfoot')?.textContent ?? '';
	/** Les prix de la dernière charge envoyée ; `[]` si aucune n'est partie (une assertion
	 *  qui compare rougit alors en le disant, au lieu de lever). */
	const prixEnvoyes = () =>
		(
			updateInvoiceMock.mock.calls.at(-1)?.[1] as { lines: { unitPrice: string }[] } | undefined
		)?.lines.map((l) => l.unitPrice) ?? [];

	async function taper(el: HTMLInputElement, value: string) {
		await fireEvent.input(el, { target: { value } });
		await settle();
	}

	describe('AC2 — saisie et refus', () => {
		it('« 1.250 » : message ambigu sous le champ, bordure, total 0.00 ; Enregistrer refuse en nommant la ligne', async () => {
			updateInvoiceMock.mockResolvedValue({ id: 7 });
			const { container, getByTestId, queryByTestId } = render(InvoiceForm, {
				invoice: brouillonAuxPrix([ligneAuPrix(1, '100.0000')]),
			});
			await settle();
			await taper(prix(container), '1.250');

			expect(queryByTestId('invoice-line-unit-price-ambiguous')).not.toBeNull();
			expect(queryByTestId('invoice-line-unit-price-unreadable')).toBeNull();
			expect(borde(prix(container))).toBe(true);
			expect(totauxDeLigne(container)[0]).toBe('0.00');

			await fireEvent.click(getByTestId('create-invoice-button'));
			await settle();
			expect(container.textContent).toContain(AMBIGU);
			expect(updateInvoiceMock).not.toHaveBeenCalled();
		});

		it('la rangée du message couvre les sept colonnes du tableau (V1-1)', async () => {
			const { container, getByTestId } = render(InvoiceForm, {
				invoice: brouillonAuxPrix([ligneAuPrix(1, '100.0000')]),
			});
			await settle();
			await taper(prix(container), '1.250');
			const cell = getByTestId('invoice-line-unit-price-ambiguous') as HTMLTableCellElement;
			expect(cell.tagName).toBe('TD');
			expect(cell.colSpan).toBe(7);
			const enTetes = container.querySelectorAll('[data-testid="invoice-lines-table"] thead th');
			expect(enTetes.length).toBe(7);
		});

		it('« 1.250 » puis Entrée (soumission du formulaire) : même refus, aucune requête', async () => {
			updateInvoiceMock.mockResolvedValue({ id: 7 });
			const { container } = render(InvoiceForm, {
				invoice: brouillonAuxPrix([ligneAuPrix(1, '100.0000')]),
			});
			await settle();
			await taper(prix(container), '1.250');
			await fireEvent.submit(container.querySelector('form')!);
			await settle();
			expect(container.textContent).toContain(AMBIGU);
			expect(updateInvoiceMock).not.toHaveBeenCalled();
		});

		it("« 12'34 » : message illisible sous le champ, bordure ; Enregistrer refuse « prix unitaire illisible »", async () => {
			updateInvoiceMock.mockResolvedValue({ id: 7 });
			const { container, getByTestId, queryByTestId } = render(InvoiceForm, {
				invoice: brouillonAuxPrix([ligneAuPrix(1, '100.0000')]),
			});
			await settle();
			await taper(prix(container), "12'34");

			expect(queryByTestId('invoice-line-unit-price-unreadable')).not.toBeNull();
			expect(queryByTestId('invoice-line-unit-price-ambiguous')).toBeNull();
			expect(borde(prix(container))).toBe(true);

			await fireEvent.click(getByTestId('create-invoice-button'));
			await settle();
			expect(container.textContent).toContain(ILLISIBLE);
			expect(container.textContent).not.toContain(AMBIGU);
			expect(updateInvoiceMock).not.toHaveBeenCalled();
		});

		it('prix vidé : aucun message sous le champ, refus « valeurs numériques invalides » (inchangé)', async () => {
			updateInvoiceMock.mockResolvedValue({ id: 7 });
			const { container, getByTestId, queryByTestId } = render(InvoiceForm, {
				invoice: brouillonAuxPrix([ligneAuPrix(1, '100.0000')]),
			});
			await settle();
			await taper(prix(container), '');

			expect(queryByTestId('invoice-line-unit-price-ambiguous')).toBeNull();
			expect(queryByTestId('invoice-line-unit-price-unreadable')).toBeNull();
			expect(borde(prix(container))).toBe(false);

			await fireEvent.click(getByTestId('create-invoice-button'));
			await settle();
			expect(container.textContent).toContain('Ligne 1 : valeurs numériques invalides');
			expect(container.textContent).not.toContain(ILLISIBLE);
			expect(updateInvoiceMock).not.toHaveBeenCalled();
		});

		// Revue P1, B1 / E2 — le refus nomme la ligne en cause, non la première.
		it.each([
			['1.250', 'Ligne 2 : prix unitaire ambigu'],
			["12'34", 'Ligne 2 : prix unitaire illisible'],
		])('deux lignes, la seconde au prix « %s » : le bandeau dit « %s »', async (saisie, attendu) => {
			updateInvoiceMock.mockResolvedValue({ id: 7 });
			const { container, getByTestId } = render(InvoiceForm, {
				invoice: brouillonAuxPrix([ligneAuPrix(1, '100.0000'), ligneAuPrix(2, '100.0000')]),
			});
			await settle();
			await taper(prix(container, 1), saisie);
			await fireEvent.click(getByTestId('create-invoice-button'));
			await settle();
			expect(container.textContent).toContain(attendu);
			expect(container.textContent).not.toContain('Ligne 1 : prix unitaire');
			expect(updateInvoiceMock).not.toHaveBeenCalled();
		});

		// Revue P1, E3 — l'ordre de `validateClient` : description, puis prix, puis quantité
		// (C-36-3b-9).
		it('quantité 0 et prix « 1.250 » : le prix est refusé avant la quantité', async () => {
			const { container, getByTestId } = render(InvoiceForm, {
				invoice: brouillonAuxPrix([ligneAuPrix(1, '100.0000')]),
			});
			await settle();
			await taper(quantite(container), '0');
			await taper(prix(container), '1.250');
			await fireEvent.click(getByTestId('create-invoice-button'));
			await settle();
			expect(container.textContent).toContain(AMBIGU);
			expect(container.textContent).not.toContain('quantité > 0');
			expect(updateInvoiceMock).not.toHaveBeenCalled();
		});

		it('description vide et prix « 1.250 » : la description est refusée avant le prix', async () => {
			const { container, getByTestId } = render(InvoiceForm, {
				invoice: brouillonAuxPrix([ligneAuPrix(1, '100.0000')]),
			});
			await settle();
			const description = container.querySelector<HTMLInputElement>(
				'[data-testid="invoice-lines-table"] tbody tr input[type="text"]:not([inputmode])',
			)!;
			await taper(description, '');
			await taper(prix(container), '1.250');
			await fireEvent.click(getByTestId('create-invoice-button'));
			await settle();
			expect(container.textContent).toContain('Ligne 1 : description requise');
			expect(container.textContent).not.toContain(AMBIGU);
			expect(updateInvoiceMock).not.toHaveBeenCalled();
		});

		it('quantité 0 et prix clair : la quantité reste refusée « quantité > 0 »', async () => {
			const { container, getByTestId } = render(InvoiceForm, {
				invoice: brouillonAuxPrix([ligneAuPrix(1, '100.0000')]),
			});
			await settle();
			await taper(quantite(container), '0');
			await fireEvent.click(getByTestId('create-invoice-button'));
			await settle();
			expect(container.textContent).toContain('Ligne 1 : quantité > 0');
			expect(updateInvoiceMock).not.toHaveBeenCalled();
		});
	});

	describe('AC3 — accepté et normalisé', () => {
		it("« 1'250 » × 2 : aucun message, total de ligne et de facture 2’500.00 ; part `1250`", async () => {
			updateInvoiceMock.mockResolvedValue({ id: 7 });
			const { container, getByTestId, queryByTestId } = render(InvoiceForm, {
				invoice: brouillonAuxPrix([ligneAuPrix(1, '100.0000')]),
			});
			await settle();
			await taper(quantite(container), '2');
			await taper(prix(container), "1'250");

			expect(queryByTestId('invoice-line-unit-price-ambiguous')).toBeNull();
			expect(queryByTestId('invoice-line-unit-price-unreadable')).toBeNull();
			expect(borde(prix(container))).toBe(false);
			expect(totauxDeLigne(container)[0]).toBe('2’500.00');
			expect(totalFacture(container)).toContain('2’500.00');

			await fireEvent.click(getByTestId('create-invoice-button'));
			await settle();
			expect(updateInvoiceMock).toHaveBeenCalledTimes(1);
			expect(prixEnvoyes()).toEqual(['1250']);
		});

		it('« 1,25 » part `1.25`', async () => {
			updateInvoiceMock.mockResolvedValue({ id: 7 });
			const { container, getByTestId } = render(InvoiceForm, {
				invoice: brouillonAuxPrix([ligneAuPrix(1, '100.0000')]),
			});
			await settle();
			await taper(prix(container), '1,25');
			await fireEvent.click(getByTestId('create-invoice-button'));
			await settle();
			expect(prixEnvoyes()).toEqual(['1.25']);
		});
	});

	// Revue P1, A1 — assertions `expect.soft` : sous une mutation des sites de chargement
	// (M3 à M5), la forme rougit, mais aussi « aucun message ambigu », « aucun refus » et
	// « la charge part » — le refus à tort que D2 prévient, chacun nommé au journal.
	describe('AC4 — un prix enregistré n’est pas refusé à tort', () => {
		const PRIX_SERVEUR = ['1.2340', '1.2500', '12.3450'];
		const ROUVERTS = ['1.2340', '1.25', '12.3450'];

		it('ouverture d’un brouillon : `1.2340`, `1.25`, `12.3450`, aucun message ; renvoyés tels quels (M3 : `initLines`)', async () => {
			updateInvoiceMock.mockResolvedValue({ id: 7 });
			const { container, getByTestId } = render(InvoiceForm, {
				invoice: brouillonAuxPrix(PRIX_SERVEUR.map((p, k) => ligneAuPrix(k + 1, p))),
			});
			await settle();
			expect.soft(prixAffiches(container), 'forme rouverte').toEqual(ROUVERTS);
			// `querySelectorAll` : sous mutation, plusieurs lignes portent le message, et
			// `queryByTestId` lèverait au lieu d'asserter.
			expect.soft(ambigus(container), 'message sous le champ').toBe(0);
			await fireEvent.click(getByTestId('create-invoice-button'));
			await settle();
			expect.soft(container.textContent, 'bandeau').not.toContain('prix unitaire ambigu');
			expect.soft(updateInvoiceMock, 'charge partie').toHaveBeenCalledTimes(1);
			expect(prixEnvoyes(), 'charge envoyée').toEqual(ROUVERTS);
		});

		it('après « Recharger » sur conflit de version : mêmes formes, aucun message (M4 : `reloadFromServer`)', async () => {
			updateInvoiceMock.mockRejectedValueOnce({
				code: 'OPTIMISTIC_LOCK_CONFLICT',
				status: 409,
				message: 'conflit',
			});
			getInvoiceMock.mockResolvedValue(
				brouillonAuxPrix(PRIX_SERVEUR.map((p, k) => ligneAuPrix(k + 1, p))),
			);
			const { container, getByTestId, getByText } = render(InvoiceForm, {
				invoice: brouillonAuxPrix([ligneAuPrix(1, '100.00')]),
			});
			await settle();
			await fireEvent.click(getByTestId('create-invoice-button'));
			await settle();
			await fireEvent.click(getByText('Recharger'));
			await settle();
			expect.soft(prixAffiches(container), 'forme rouverte').toEqual(ROUVERTS);
			// `querySelectorAll` : sous mutation, plusieurs lignes portent le message, et
			// `queryByTestId` lèverait au lieu d'asserter.
			expect.soft(ambigus(container), 'message sous le champ').toBe(0);
			updateInvoiceMock.mockResolvedValue({ id: 7 });
			await fireEvent.click(getByTestId('create-invoice-button'));
			await settle();
			expect.soft(container.textContent, 'bandeau').not.toContain('prix unitaire ambigu');
			expect.soft(updateInvoiceMock, 'charge partie').toHaveBeenCalledTimes(2);
			expect(prixEnvoyes(), 'charge envoyée').toEqual(ROUVERTS);
		});

		it('ligne ajoutée depuis un article au prix `1.2340` : `1.2340`, aucun message (M5 : `onProductSelect`)', async () => {
			updateInvoiceMock.mockResolvedValue({ id: 7 });
			listProductsMock.mockResolvedValue({
				items: [
					{
						id: 1,
						companyId: 1,
						name: 'Article au millième',
						description: null,
						unitPrice: '1.2340',
						vatRate: '8.10',
						defaultRevenueAccountId: null,
						active: true,
						version: 1,
						createdAt: '2026-01-01T00:00:00',
						updatedAt: '2026-01-01T00:00:00',
					},
				],
				total: 1,
				limit: 20,
				offset: 0,
			});
			// Un brouillon à une ligne complète : `invoiceWith([])` ouvrirait une ligne neuve
			// sans description, que la validation refuserait avant le prix.
			const { container, getByText, getByTestId } = render(InvoiceForm, {
				invoice: brouillonAuxPrix([ligneAuPrix(1, '100.0000')]),
			});
			await settle();
			await fireEvent.click(getByText('Depuis catalogue'));
			await settle();
			const entry = document.querySelector('[role="dialog"] ul li button') as HTMLElement | null;
			expect(entry, 'le picker doit proposer l’article').toBeTruthy();
			await fireEvent.click(entry!);
			await settle();
			expect.soft(prixAffiches(container).at(-1), 'forme rouverte').toBe('1.2340');
			// `querySelectorAll` : sous mutation, plusieurs lignes portent le message, et
			// `queryByTestId` lèverait au lieu d'asserter.
			expect.soft(ambigus(container), 'message sous le champ').toBe(0);
			await fireEvent.click(getByTestId('create-invoice-button'));
			await settle();
			expect.soft(container.textContent, 'bandeau').not.toContain('prix unitaire ambigu');
			expect.soft(updateInvoiceMock, 'charge partie').toHaveBeenCalledTimes(1);
			expect(prixEnvoyes(), 'charge envoyée').toEqual(['100.00', '1.2340']);
		});
	});

	describe('AC5 — un prix retapé pendant l’attente des réglages', () => {
		/** Le second appel (celui de l'envoi) retape le prix avant de résoudre. */
		function retaperPendantLAttente(container: HTMLElement, valeur: string) {
			const reglages = { defaultReceivableAccountId: 1100, defaultRevenueAccountId: DEFAULT_REVENUE.id };
			getInvoiceSettingsMock.mockImplementationOnce(async () => {
				await fireEvent.input(prix(container), { target: { value: valeur } });
				return reglages;
			});
		}

		it('« 1.250 » tapé pendant l’attente : aucune requête, bandeau « prix unitaire ambigu », bouton de nouveau actif', async () => {
			updateInvoiceMock.mockResolvedValue({ id: 7 });
			const { container, getByTestId } = render(InvoiceForm, {
				invoice: brouillonAuxPrix([ligneAuPrix(1, '100.0000')]),
			});
			await settle();
			retaperPendantLAttente(container, '1.250');
			await fireEvent.click(getByTestId('create-invoice-button'));
			await settle();
			expect(updateInvoiceMock).not.toHaveBeenCalled();
			expect(container.textContent).toContain(AMBIGU);
			expect(isDisabled(getByTestId('create-invoice-button'))).toBe(false);
		});

		it('prix vidé pendant l’attente : aucune requête, jamais un prix à zéro (V1-2)', async () => {
			updateInvoiceMock.mockResolvedValue({ id: 7 });
			const { container, getByTestId } = render(InvoiceForm, {
				invoice: brouillonAuxPrix([ligneAuPrix(1, '100.0000')]),
			});
			await settle();
			retaperPendantLAttente(container, '');
			await fireEvent.click(getByTestId('create-invoice-button'));
			await settle();
			expect(updateInvoiceMock).not.toHaveBeenCalled();
			expect(container.textContent).toContain('Ligne 1 : valeurs numériques invalides');
			expect(isDisabled(getByTestId('create-invoice-button'))).toBe(false);
		});

		// Revue P1, E4 — illisible pendant l'attente : arrêt ; clair pendant l'attente :
		// la charge porte la valeur relue, non celle validée avant l'attente.
		it("« 12'34 » tapé pendant l’attente : aucune requête, bandeau « prix unitaire illisible »", async () => {
			updateInvoiceMock.mockResolvedValue({ id: 7 });
			const { container, getByTestId } = render(InvoiceForm, {
				invoice: brouillonAuxPrix([ligneAuPrix(1, '100.0000')]),
			});
			await settle();
			retaperPendantLAttente(container, "12'34");
			await fireEvent.click(getByTestId('create-invoice-button'));
			await settle();
			expect(updateInvoiceMock).not.toHaveBeenCalled();
			expect(container.textContent).toContain(ILLISIBLE);
			expect(isDisabled(getByTestId('create-invoice-button'))).toBe(false);
		});

		it("« 1'250 » tapé pendant l’attente : la charge part `1250`, non `100.00`", async () => {
			updateInvoiceMock.mockResolvedValue({ id: 7 });
			const { container, getByTestId } = render(InvoiceForm, {
				invoice: brouillonAuxPrix([ligneAuPrix(1, '100.0000')]),
			});
			await settle();
			retaperPendantLAttente(container, "1'250");
			await fireEvent.click(getByTestId('create-invoice-button'));
			await settle();
			expect(updateInvoiceMock).toHaveBeenCalledTimes(1);
			expect(prixEnvoyes()).toEqual(['1250']);
		});
	});
});

// Story 48-13 (#1233) — le client d'un brouillon ouvert en modification se charge après
// l'ouverture (`getContact`). Sa réponse, quand elle arrive APRÈS un geste de
// l'utilisateur dans le champ client (choix, texte tapé), est jetée : elle ne remplace plus
// le client choisi, ne réécrit plus le bandeau « Impossible de charger le contact », et
// n'efface plus le texte tapé (D1, D10). Sans geste, elle s'applique comme avant.
describe('InvoiceForm — le client choisi n’est plus écrasé par un chargement tardif (Story 48-13, #1233)', () => {
	const getContactMock = vi.mocked(contactsApi.getContact);
	const listContactsMock = vi.mocked(contactsApi.listContacts);
	const ORIGINE = { id: 1, name: 'Client' } as ContactResponse;
	const CHOISI = { id: 2, name: 'Autre client' } as ContactResponse;
	const SERVEUR = { id: 3, name: 'Client du serveur' } as ContactResponse;

	/** Une réponse de `getContact` tenue à la main. */
	function differe() {
		let resolve!: (c: ContactResponse) => void;
		let reject!: (e: unknown) => void;
		const promise = new Promise<ContactResponse>((res, rej) => {
			resolve = res;
			reject = rej;
		});
		return { promise, resolve, reject };
	}

	/** Le chargement du client d'origine (id 1) attend ; les autres répondent tout de suite. */
	function retarderOrigine() {
		const d = differe();
		getContactMock.mockImplementation((id: number) =>
			id === ORIGINE.id ? d.promise : Promise.resolve(id === SERVEUR.id ? SERVEUR : CHOISI),
		);
		return d;
	}

	async function monter() {
		updateInvoiceMock.mockResolvedValue({ id: 7 });
		const r = render(InvoiceForm, { invoice: invoiceWith([SERVICES.id]) });
		await settle();
		const champ = r.container.querySelector('input[role="combobox"]') as HTMLInputElement;
		return { ...r, champ };
	}

	/** Choix SANS frappe : focus du champ (recherche vide), puis clic sur l'option. */
	async function choisirSansTaper(container: HTMLElement, champ: HTMLInputElement) {
		listContactsMock.mockResolvedValueOnce({ items: [CHOISI], total: 1, offset: 0, limit: 50 });
		await fireEvent.focus(champ);
		await settle();
		const option = container.querySelector('[role="option"]') as HTMLElement | null;
		expect(option, 'la liste doit proposer le client').toBeTruthy();
		await fireEvent.mouseDown(option!);
		await settle();
	}

	function contactEnvoye(): number {
		return (updateInvoiceMock.mock.calls.at(-1)![1] as { contactId: number }).contactId;
	}

	afterEach(() => {
		getContactMock.mockImplementation(async () => ORIGINE);
	});

	it('AC1 — client choisi avant la réponse : la réponse tardive ne le remplace pas, il part (M1, M3)', async () => {
		const d = retarderOrigine();
		const { champ, container, getByTestId } = await monter();
		expect(champ.value).toBe('');

		await choisirSansTaper(container, champ);
		expect(champ.value).toBe('Autre client');

		d.resolve(ORIGINE);
		await settle();
		expect(champ.value).toBe('Autre client');

		await fireEvent.click(getByTestId('create-invoice-button'));
		await settle();
		expect(updateInvoiceMock).toHaveBeenCalledTimes(1);
		expect(contactEnvoye()).toBe(CHOISI.id);
	});

	it('AC1 — client choisi avant un échec tardif : aucun bandeau « Impossible de charger le contact » (M2)', async () => {
		const d = retarderOrigine();
		const { champ, container, queryByTestId } = await monter();
		await choisirSansTaper(container, champ);

		d.reject({ code: 'INTERNAL_ERROR', message: 'panne', status: 500 });
		await settle();
		expect(queryByTestId('invoice-form-error')).toBeNull();
		expect(container.textContent).not.toContain('Impossible de charger le contact');
		expect(champ.value).toBe('Autre client');
	});

	it('AC1 — sans geste, la réponse tardive s’applique comme avant (le client d’origine s’affiche et part)', async () => {
		const d = retarderOrigine();
		const { champ, getByTestId } = await monter();
		d.resolve(ORIGINE);
		await settle();
		expect(champ.value).toBe('Client');
		await fireEvent.click(getByTestId('create-invoice-button'));
		await settle();
		expect(contactEnvoye()).toBe(ORIGINE.id);
	});

	it('AC1 — sans geste, un échec tardif s’affiche comme avant', async () => {
		const d = retarderOrigine();
		const { getByTestId } = await monter();
		d.reject({ code: 'INTERNAL_ERROR', message: 'panne', status: 500 });
		await settle();
		expect(getByTestId('invoice-form-error').textContent).toContain('Impossible de charger le contact (panne)');
	});

	// D2 : filet de non-régression, sans preuve du rouge. La modale de conflit ne s'ouvre
	// qu'après un envoi, qui exige un client : pendant le chargement initial, il n'y en a
	// un que par un choix, qui jette déjà la réponse initiale (D1).
	it('AC2 — une réponse initiale arrivée après « Recharger » ne remplace pas le client rechargé', async () => {
		const d = retarderOrigine();
		updateInvoiceMock.mockRejectedValueOnce({
			code: 'OPTIMISTIC_LOCK_CONFLICT',
			status: 409,
			message: 'conflit',
		});
		getInvoiceMock.mockResolvedValue({ ...invoiceWith([SERVICES.id]), contactId: SERVEUR.id, version: 2 });
		const { champ, container, getByTestId, getByText } = await monter();
		await choisirSansTaper(container, champ);
		await fireEvent.click(getByTestId('create-invoice-button'));
		await settle();
		await fireEvent.click(getByText('Recharger'));
		await settle();
		expect(champ.value).toBe('Client du serveur');

		d.resolve(ORIGINE);
		await settle();
		expect(champ.value).toBe('Client du serveur');
		updateInvoiceMock.mockResolvedValue({ id: 7 });
		await fireEvent.click(getByTestId('create-invoice-button'));
		await settle();
		expect(contactEnvoye()).toBe(SERVEUR.id);
	});

	it('AC13 — un texte tapé avant la réponse est gardé, et l’envoi refusé (M8)', async () => {
		const d = retarderOrigine();
		const { champ, getByTestId } = await monter();
		await fireEvent.input(champ, { target: { value: 'Mar' } });
		await settle();

		d.resolve(ORIGINE);
		await settle();
		expect(champ.value).toBe('Mar');

		await fireEvent.blur(champ);
		await fireEvent.click(getByTestId('create-invoice-button'));
		await settle();
		expect(updateInvoiceMock).not.toHaveBeenCalled();
		expect(getByTestId('invoice-form-error').textContent).toContain('Choisissez le client dans la liste');
		expect(champ.value).toBe('Mar');
	});

	// AC9 a (contrainte de Guy du 2026-11-18, factures réelles) : non-régression, ne
	// rougit pas. Un brouillon ouvert puis enregistré sans aucun geste envoie ce qui a été
	// chargé, champ pour champ.
	it('AC9 a — ouvert puis enregistré sans geste : le corps envoyé est celui du brouillon chargé', async () => {
		const brouillon = {
			...invoiceWith([SERVICES.id, null]),
			dueDate: '2026-07-15',
			paymentTerms: '30 jours net',
		} as unknown as InvoiceResponse;
		updateInvoiceMock.mockResolvedValue({ id: 7 });
		const { container, getByTestId } = render(InvoiceForm, { invoice: brouillon });
		await settle();
		const champ = container.querySelector('input[role="combobox"]') as HTMLInputElement;
		expect(champ.value).toBe('Client');
		// Revue P1 (A2) : le total affiché est celui du brouillon chargé (2 × 100.00),
		// non le total comparé à lui-même.
		const total = () => container.querySelector('tfoot')?.textContent ?? '';
		expect(total()).toContain('200.00');

		await fireEvent.click(getByTestId('create-invoice-button'));
		await settle();
		expect(updateInvoiceMock).toHaveBeenCalledTimes(1);
		expect(updateInvoiceMock.mock.calls[0]).toEqual([
			7,
			{
				contactId: 1,
				date: '2026-06-15',
				dueDate: '2026-07-15',
				paymentTerms: '30 jours net',
				projectId: null,
				version: 1,
				lines: [
					{
						lineType: 'priced',
						description: 'Ligne 1',
						quantity: '1',
						unitPrice: '100.00',
						vatRate: '8.10',
						revenueAccountId: SERVICES.id,
					},
					{
						lineType: 'priced',
						description: 'Ligne 2',
						quantity: '1',
						unitPrice: '100.00',
						vatRate: '8.10',
						revenueAccountId: null,
					},
				],
			},
		]);
		expect(total()).toContain('200.00');
	});
});

// Story 49-1c (#1242, T8) — la ligne de commentaire dans le formulaire : saisie, envoi,
// et relecture typée du brouillon, à l'ouverture comme après un conflit de version.
describe('InvoiceForm — ligne de commentaire (Story 49-1c)', () => {
	/** Une ligne de commentaire telle que l'API la rend (valeurs neutres). */
	function commentaire(position: number, description: string) {
		return {
			id: 100 + position,
			invoiceId: 7,
			position,
			lineType: 'comment' as const,
			description,
			quantity: '0.0000',
			unitPrice: '0.0000',
			vatRate: '0.00',
			lineTotal: '0.0000',
			revenueAccountId: null,
			createdAt: '',
		};
	}

	/** Brouillon `[chiffrée, commentaire]` : la ligne chiffrée porte un compte. */
	function brouillonAvecCommentaire(texte = 'Merci de votre confiance'): InvoiceResponse {
		return {
			...invoiceWith([SERVICES.id]),
			lines: [{ ...line(1, SERVICES.id), lineType: 'priced' }, commentaire(2, texte)],
		} as unknown as InvoiceResponse;
	}

	const LIGNES_ATTENDUES = [
		{
			lineType: 'priced',
			description: 'Ligne 1',
			quantity: '1',
			unitPrice: '100.00',
			vatRate: '8.10',
			revenueAccountId: SERVICES.id,
		},
		{ lineType: 'comment', description: 'Merci de votre confiance' },
	];

	function commentFields(container: HTMLElement): HTMLTextAreaElement[] {
		return Array.from(container.querySelectorAll('[data-testid="invoice-line-comment"]'));
	}

	it('AC3 — « Commentaire » ajoute en fin de tableau un champ multiligne, sans taux, compte ni total ; le total ne change pas', async () => {
		const { container, getByTestId } = render(InvoiceForm, { invoice: invoiceWith([null]) });
		await settle();
		const total = () => container.querySelector('tfoot')?.textContent ?? '';
		expect(total()).toContain('100.00');
		const tauxAvant = container.querySelectorAll('[data-testid="invoice-line-vat-rate"]').length;
		const comptesAvant = accountInputs(container).length;

		await fireEvent.click(getByTestId('invoice-add-comment-line'));
		await settle();

		const [champ] = commentFields(container);
		expect(champ.tagName).toBe('TEXTAREA');
		expect(champ.getAttribute('aria-label')).toBe('Commentaire — ligne 2');
		const rangees = container.querySelectorAll('[data-testid="invoice-lines-table"] tbody > tr');
		expect(rangees[rangees.length - 1].getAttribute('data-testid')).toBe('invoice-line-comment-row');
		// La rangée du commentaire : une cellule de texte sur six colonnes, puis la suppression.
		const cellules = rangees[rangees.length - 1].querySelectorAll(':scope > td');
		expect(cellules).toHaveLength(2);
		// Le colspan couvre toutes les colonnes de l'en-tête rendu sauf la dernière (la suppression).
		const nbEntetes = container.querySelectorAll('[data-testid="invoice-lines-table"] thead th').length;
		expect(Number(cellules[0].getAttribute('colspan'))).toBe(nbEntetes - 1);
		expect(container.querySelectorAll('[data-testid="invoice-line-vat-rate"]')).toHaveLength(tauxAvant);
		expect(accountInputs(container)).toHaveLength(comptesAvant);

		await fireEvent.input(champ, { target: { value: 'Merci\nà bientôt' } });
		await settle();
		expect(total()).toContain('100.00');
	});

	it('AC3 — un commentaire vide bloque l’envoi en nommant la ligne', async () => {
		const { container, getByTestId } = render(InvoiceForm, { invoice: invoiceWith([null]) });
		await settle();
		await fireEvent.click(getByTestId('invoice-add-comment-line'));
		await settle();
		await fireEvent.click(getByTestId('create-invoice-button'));
		await settle();
		expect(updateInvoiceMock).not.toHaveBeenCalled();
		expect(getByTestId('invoice-form-error').textContent).toContain(
			'Ligne 2 : le texte du commentaire est obligatoire',
		);
		expect(commentFields(container)).toHaveLength(1);
	});

	it('AC3, AC4 — brouillon [chiffrée, commentaire] ouvert puis enregistré : chaque ligne part avec son type', async () => {
		updateInvoiceMock.mockResolvedValue({ id: 7 });
		const { container, getByTestId } = render(InvoiceForm, { invoice: brouillonAvecCommentaire() });
		await settle();
		expect(commentFields(container).map((c) => c.value)).toEqual(['Merci de votre confiance']);
		await fireEvent.click(getByTestId('create-invoice-button'));
		await settle();
		expect(updateInvoiceMock).toHaveBeenCalledTimes(1);
		const sent = updateInvoiceMock.mock.calls[0][1] as { lines: unknown[] };
		expect(sent.lines).toEqual(LIGNES_ATTENDUES);
	});

	it('AC4 — de même après « Recharger » sur un conflit de version', async () => {
		updateInvoiceMock.mockRejectedValueOnce({
			code: 'OPTIMISTIC_LOCK_CONFLICT',
			status: 409,
			message: 'conflit',
		});
		getInvoiceMock.mockResolvedValue(brouillonAvecCommentaire());
		// Départ : un brouillon chiffré seulement ; le serveur, lui, porte le commentaire.
		const { container, getByTestId, getByText } = render(InvoiceForm, {
			invoice: invoiceWith([SERVICES.id]),
		});
		await settle();
		expect(commentFields(container)).toHaveLength(0);
		await fireEvent.click(getByTestId('create-invoice-button'));
		await settle();
		await fireEvent.click(getByText('Recharger'));
		await settle();
		expect(commentFields(container).map((c) => c.value)).toEqual(['Merci de votre confiance']);

		updateInvoiceMock.mockResolvedValue({ id: 7 });
		await fireEvent.click(getByTestId('create-invoice-button'));
		await settle();
		expect(updateInvoiceMock).toHaveBeenCalledTimes(2);
		const sent = updateInvoiceMock.mock.calls[1][1] as { lines: unknown[] };
		expect(sent.lines).toEqual(LIGNES_ATTENDUES);
	});

	it('AC4 — une ligne heading relue repart heading', async () => {
		updateInvoiceMock.mockResolvedValue({ id: 7 });
		const brouillon = {
			...invoiceWith([null]),
			lines: [line(1, null), { ...commentaire(2, 'Travaux'), lineType: 'heading' }],
		} as unknown as InvoiceResponse;
		const { getByTestId } = render(InvoiceForm, { invoice: brouillon });
		await settle();
		await fireEvent.click(getByTestId('create-invoice-button'));
		await settle();
		const sent = updateInvoiceMock.mock.calls[0][1] as { lines: unknown[] };
		expect(sent.lines[1]).toEqual({ lineType: 'heading', description: 'Travaux' });
	});

	it('AC3 — taux désactivé : le refus ne nomme que la ligne chiffrée, jamais le commentaire', async () => {
		const brouillon = {
			...invoiceWith([null]),
			lines: [commentaire(1, 'Préambule'), { ...line(2, null), vatRate: '7.70' }],
		} as unknown as InvoiceResponse;
		const { container, getByTestId } = render(InvoiceForm, { invoice: brouillon });
		await settle();
		// Un seul sélecteur de taux : celui de la ligne chiffrée.
		expect(container.querySelectorAll('[data-testid="invoice-line-vat-rate"]')).toHaveLength(1);
		await fireEvent.click(getByTestId('create-invoice-button'));
		await settle();
		expect(updateInvoiceMock).not.toHaveBeenCalled();
		expect(getByTestId('invoice-form-error').textContent).toBe(
			'Ligne 2 : le taux de TVA 7.70% a été désactivé. Choisissez un taux actif.',
		);
	});
});

// Story 49-2 (#1756, #1242, T2) — réordonner les lignes d'un brouillon : boutons ↑ / ↓,
// focus, annonce, états désactivés, ordre envoyé et ordre rechargé.
describe('InvoiceForm — réordonner les lignes (Story 49-2)', () => {
	/** Brouillon `[A chiffrée (compte 3200), C commentaire, B chiffrée (sans compte)]`. */
	function brouillonABC(texteCommentaire = 'Merci de votre confiance'): InvoiceResponse {
		return {
			...invoiceWith([SERVICES.id]),
			lines: [
				{ ...line(1, SERVICES.id), lineType: 'priced', description: 'Prestation A' },
				{
					id: 102,
					invoiceId: 7,
					position: 2,
					lineType: 'comment',
					description: texteCommentaire,
					quantity: '0.0000',
					unitPrice: '0.0000',
					vatRate: '0.00',
					lineTotal: '0.0000',
					revenueAccountId: null,
					createdAt: '',
				},
				{ ...line(3, null), lineType: 'priced', description: 'Article B', unitPrice: '50.00' },
			],
		} as unknown as InvoiceResponse;
	}

	const monter = (c: HTMLElement, n: number) =>
		c.querySelector(`button[aria-label="Monter la ligne ${n}"]`) as HTMLButtonElement;
	const descendre = (c: HTMLElement, n: number) =>
		c.querySelector(`button[aria-label="Descendre la ligne ${n}"]`) as HTMLButtonElement;
	const actif = () => (document.activeElement as HTMLElement | null)?.getAttribute('aria-label');

	/** Les descriptions des lignes, de haut en bas (commentaire : son texte). */
	function ordre(c: HTMLElement): string[] {
		const rows = Array.from(c.querySelectorAll('tbody > tr'));
		const out: string[] = [];
		for (const r of rows) {
			const t = r.querySelector('textarea');
			const i = r.querySelector('input[type="text"]') as HTMLInputElement | null;
			if (t) out.push((t as HTMLTextAreaElement).value);
			else if (i) out.push(i.value);
		}
		return out;
	}

	async function cliquer(el: HTMLElement) {
		await fireEvent.click(el);
		await settle();
	}

	const corps = () =>
		(updateInvoiceMock.mock.calls[0][1] as { lines: Record<string, unknown>[] }).lines;

	it('AC1 — ↑ désactivé sur la première ligne, ↓ sur la dernière, chaque ligne en porte deux', async () => {
		const { container } = render(InvoiceForm, { invoice: brouillonABC() });
		await settle();
		expect(isDisabled(monter(container, 1))).toBe(true);
		expect(isDisabled(descendre(container, 1))).toBe(false);
		expect(isDisabled(monter(container, 2))).toBe(false);
		expect(isDisabled(descendre(container, 2))).toBe(false);
		expect(isDisabled(monter(container, 3))).toBe(false);
		expect(isDisabled(descendre(container, 3))).toBe(true);
		expect(monter(container, 2).getAttribute('title')).toBe('Monter la ligne 2');
	});

	it("AC1 — une facture d'une seule ligne : les deux boutons désactivés", async () => {
		const { container } = render(InvoiceForm, { invoice: invoiceWith([null]) });
		await settle();
		expect(isDisabled(monter(container, 1))).toBe(true);
		expect(isDisabled(descendre(container, 1))).toBe(true);
	});

	it('AC2, AC3 — ↓ deux fois sur A : corps envoyé [C, B, A], chacune avec son type, son compte, son total', async () => {
		updateInvoiceMock.mockResolvedValue({ id: 7 });
		const { container, getByTestId } = render(InvoiceForm, { invoice: brouillonABC() });
		await settle();
		const total = () => container.querySelector('tfoot')?.textContent ?? '';
		const avant = total();
		expect(avant).toContain('150.00');

		await cliquer(descendre(container, 1));
		expect(ordre(container)).toEqual(['Merci de votre confiance', 'Prestation A', 'Article B']);
		await cliquer(descendre(container, 2));
		expect(ordre(container)).toEqual(['Merci de votre confiance', 'Article B', 'Prestation A']);
		expect(total()).toBe(avant);
		// L'`AccountAutocomplete` de A, déplacée, montre toujours SON compte.
		expect(accountInputs(container).map((b) => b.value)).toEqual(['', '3200 — Honoraires']);

		await fireEvent.click(getByTestId('create-invoice-button'));
		await settle();
		expect(corps()).toEqual([
			{ lineType: 'comment', description: 'Merci de votre confiance' },
			{
				lineType: 'priced',
				description: 'Article B',
				quantity: '1',
				unitPrice: '50.00',
				vatRate: '8.10',
				revenueAccountId: null,
			},
			{
				lineType: 'priced',
				description: 'Prestation A',
				quantity: '1',
				unitPrice: '100.00',
				vatRate: '8.10',
				revenueAccountId: SERVICES.id,
			},
		]);
	});

	it('AC2 — ↑ du commentaire : il passe en tête, le prix illisible de la ligne suivante la suit', async () => {
		const b = brouillonABC();
		(b.lines[0] as { unitPrice: string }).unitPrice = '1,5,5';
		const { container, queryAllByTestId } = render(InvoiceForm, { invoice: b });
		await settle();
		expect(queryAllByTestId('invoice-line-unit-price-unreadable')).toHaveLength(1);
		await cliquer(monter(container, 2));
		expect(ordre(container)).toEqual(['Merci de votre confiance', 'Prestation A', 'Article B']);
		// Le message est dans le même bloc que sa ligne : il la suit, sans être perdu.
		const rows = Array.from(container.querySelectorAll('tbody > tr'));
		const iMsg = rows.findIndex((r) => r.querySelector('[data-testid="invoice-line-unit-price-unreadable"]'));
		const iA = rows.findIndex((r) => (r.querySelector('input[type="text"]') as HTMLInputElement | null)?.value === 'Prestation A');
		expect(iMsg).toBe(iA + 1);
	});

	it('AC4 — le nom accessible suit le rang ; focus sur le même bouton, ou sur l’autre à la borne', async () => {
		const { container } = render(InvoiceForm, { invoice: brouillonABC() });
		await settle();
		// A (1re) descend : elle devient la 2e, son bouton ↓ garde le focus.
		await cliquer(descendre(container, 1));
		expect(actif()).toBe('Descendre la ligne 2');
		// A descend encore : 3e et dernière, ↓ désactivé, le focus passe à ↑ de la même ligne.
		await cliquer(descendre(container, 2));
		expect(isDisabled(descendre(container, 3))).toBe(true);
		expect(actif()).toBe('Monter la ligne 3');
		// A remonte : 2e, ↑ garde le focus.
		await cliquer(monter(container, 3));
		expect(actif()).toBe('Monter la ligne 2');
	});

	it('AC4 — à la borne haute, le focus passe à ↓ de la même ligne (le rappel du focus est nécessaire)', async () => {
		const { container } = render(InvoiceForm, { invoice: brouillonABC() });
		await settle();
		// Le commentaire (2e) monte en tête : ↑ devient désactivé.
		await cliquer(monter(container, 2));
		expect(isDisabled(monter(container, 1))).toBe(true);
		expect(actif()).toBe('Descendre la ligne 1');
	});

	it('AC4 — la zone aria-live existe avant tout déplacement, annonce la position, et ré-annonce', async () => {
		const { container, getByTestId } = render(InvoiceForm, { invoice: brouillonABC() });
		await settle();
		const zone = getByTestId('invoice-lines-move-status');
		expect(zone.getAttribute('aria-live')).toBe('polite');
		expect(zone.closest('table')).toBeNull();
		expect(zone.textContent?.trim()).toBe('');
		await cliquer(descendre(container, 1));
		expect(zone.textContent?.trim()).toBe('Ligne déplacée en position 2 sur 3');
		await cliquer(monter(container, 2));
		expect(zone.textContent?.trim()).toBe('Ligne déplacée en position 1 sur 3');
	});

	it('AC5 — un déplacement efface le bandeau d’erreur', async () => {
		const { container, getByTestId, queryByTestId } = render(InvoiceForm, {
			invoice: brouillonABC(''),
		});
		await settle();
		await fireEvent.click(getByTestId('create-invoice-button'));
		await settle();
		expect(getByTestId('invoice-form-error').textContent).toContain('Ligne 2 :');
		await cliquer(descendre(container, 1));
		expect(queryByTestId('invoice-form-error')).toBeNull();
	});

	it('AC5 — pendant un envoi, les boutons de déplacement sont désactivés', async () => {
		updateInvoiceMock.mockReturnValue(new Promise(() => {}));
		const { container, getByTestId } = render(InvoiceForm, { invoice: brouillonABC() });
		await settle();
		await fireEvent.click(getByTestId('create-invoice-button'));
		await settle();
		for (let n = 1; n <= 3; n++) {
			expect(isDisabled(monter(container, n))).toBe(true);
			expect(isDisabled(descendre(container, n))).toBe(true);
		}
	});

	it('AC5, AC6 — modale de conflit : boutons désactivés ; « Recharger » rend l’ordre du serveur', async () => {
		updateInvoiceMock.mockRejectedValue({
			code: 'OPTIMISTIC_LOCK_CONFLICT',
			status: 409,
			message: 'conflit',
		});
		getInvoiceMock.mockResolvedValue({ ...brouillonABC(), version: 2 });
		const { container, getByTestId, getByText } = render(InvoiceForm, {
			invoice: brouillonABC(),
		});
		await settle();
		await cliquer(descendre(container, 1));
		expect(ordre(container)).toEqual(['Merci de votre confiance', 'Prestation A', 'Article B']);
		await fireEvent.click(getByTestId('create-invoice-button'));
		await settle();
		expect(updateInvoiceMock).toHaveBeenCalled();
		for (let n = 1; n <= 3; n++) {
			expect(isDisabled(monter(container, n))).toBe(true);
			expect(isDisabled(descendre(container, n))).toBe(true);
		}
		await fireEvent.click(getByText('Recharger'));
		await settle();
		expect(ordre(container)).toEqual(['Prestation A', 'Merci de votre confiance', 'Article B']);
		expect(isDisabled(descendre(container, 1))).toBe(false);
	});

	it('AC4 (L7) — deux déplacements qui aboutissent au même texte : la zone est vidée entre les deux, donc ré-annoncée', async () => {
		const { container, getByTestId } = render(InvoiceForm, { invoice: brouillonABC() });
		await settle();
		const zone = getByTestId('invoice-lines-move-status');
		await cliquer(descendre(container, 1)); // A passe en 2e : « position 2 sur 3 »
		const annonce = 'Ligne déplacée en position 2 sur 3';
		expect(zone.textContent?.trim()).toBe(annonce);

		// Les valeurs successives de la zone pendant le 2e déplacement (ancienne valeur de chaque mutation).
		const anciennes: string[] = [];
		const obs = new MutationObserver((records) => {
			for (const r of records) anciennes.push((r.oldValue ?? '').trim());
		});
		obs.observe(zone, { subtree: true, childList: true, characterData: true, characterDataOldValue: true });
		await cliquer(monter(container, 3)); // B passe en 2e : le MÊME texte
		obs.disconnect();

		expect(zone.textContent?.trim()).toBe(annonce);
		// Texte → vide → texte : sans le vidage, la zone ne change pas et rien n'est ré-annoncé.
		expect(anciennes).toEqual([annonce, '']);
	});

	it('AC4 (L8) — une répétition de touche sur ↑ ou ↓ est neutralisée, la première frappe ne l’est pas', async () => {
		const { container } = render(InvoiceForm, { invoice: brouillonABC() });
		await settle();
		for (const bouton of [descendre(container, 1), monter(container, 3)]) {
			// `fireEvent` rend `false` quand l'événement a été annulé (`preventDefault`).
			expect(await fireEvent.keyDown(bouton, { key: 'Enter' })).toBe(true);
			expect(await fireEvent.keyDown(bouton, { key: 'Enter', repeat: true })).toBe(false);
		}
	});

	it('la poubelle est verrouillée pendant un envoi, et son nom porte le rang de la ligne', async () => {
		updateInvoiceMock.mockReturnValue(new Promise(() => {}));
		const { container, getByTestId } = render(InvoiceForm, { invoice: brouillonABC() });
		await settle();
		const poubelle = (n: number) =>
			container.querySelector(`button[aria-label="Supprimer la ligne ${n}"]`) as HTMLButtonElement;
		for (let n = 1; n <= 3; n++) expect(isDisabled(poubelle(n))).toBe(false);
		await fireEvent.click(getByTestId('create-invoice-button'));
		await settle();
		for (let n = 1; n <= 3; n++) expect(isDisabled(poubelle(n))).toBe(true);
		await fireEvent.click(poubelle(2));
		await settle();
		expect(ordre(container)).toEqual(['Prestation A', 'Merci de votre confiance', 'Article B']);
	});

	it('AC5 (L10) — « Annuler » pendant un rechargement : ↑, ↓ et poubelle restent verrouillés, la réponse rend l’ordre du serveur', async () => {
		updateInvoiceMock.mockRejectedValue({
			code: 'OPTIMISTIC_LOCK_CONFLICT',
			status: 409,
			message: 'conflit',
		});
		let repondre!: (v: InvoiceResponse) => void;
		getInvoiceMock.mockReturnValue(
			new Promise<InvoiceResponse>((r) => {
				repondre = r;
			}),
		);
		const { container, getByTestId, getByText } = render(InvoiceForm, {
			invoice: brouillonABC(),
		});
		await settle();
		await fireEvent.click(getByTestId('create-invoice-button'));
		await settle();
		await fireEvent.click(getByText('Recharger')); // la réponse tarde
		await settle();
		const dialogue = document.querySelector('[role="dialog"]') as HTMLElement;
		const annuler = Array.from(dialogue.querySelectorAll('button')).find(
			(b) => b.textContent?.trim() === 'Annuler',
		) as HTMLButtonElement;
		await fireEvent.click(annuler);
		await settle();

		const poubelle = (n: number) =>
			container.querySelector(`button[aria-label="Supprimer la ligne ${n}"]`) as HTMLButtonElement;
		for (let n = 1; n <= 3; n++) {
			expect(isDisabled(monter(container, n))).toBe(true);
			expect(isDisabled(descendre(container, n))).toBe(true);
			expect(isDisabled(poubelle(n))).toBe(true);
		}
		repondre({ ...brouillonABC(), version: 2 });
		await settle();
		expect(ordre(container)).toEqual(['Prestation A', 'Merci de votre confiance', 'Article B']);
		expect(isDisabled(descendre(container, 1))).toBe(false);
		expect(isDisabled(poubelle(1))).toBe(false);
	});
});
