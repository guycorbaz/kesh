// Story 8-5a-bis FR48 — tests Vitest pour TransactionSplitModal.svelte.
// Couvre AC #93 / #95 (balance live indicator, submit disabled tant que
// balance ≠ exact, ≥ 2 lignes obligatoires).
//
// Pattern : mock du module `reconciliation.api` + render via
// `@testing-library/svelte`. Mocks i18n + API definis AVANT l'import
// du composant (hoisting Vitest).

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, fireEvent, waitFor } from '@testing-library/svelte';
import type { ReconciliationProposal } from './reconciliation.types';

vi.mock('./reconciliation.api', () => ({
	splitTransaction: vi.fn(),
}));

// Story 35-7 (AC3) : le repli est interpolé (patron de `ContactPersonsManager.test.ts`), pour
// que « Ligne { $line } : compte requis » nomme la ligne en défaut.
vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string, args?: Record<string, string | number>) =>
		args ? fallback.replace(/\{\s*\$(\w+)\s*\}/g, (_, k) => String(args[k] ?? '')) : fallback,
}));

// Story 19-5 — mock listProjects (chargé via $effect dans le composant).
const listProjectsMock = vi.fn(async () => [] as unknown[]);
vi.mock('$lib/features/projects/projects.api', () => ({
	listProjects: () => listProjectsMock(),
}));

import * as api from './reconciliation.api';
import TransactionSplitModal from './TransactionSplitModal.svelte';
import CounterpartyModalHost from './CounterpartyModalHost.test.svelte';
import type { AccountResponse } from '$lib/features/accounts/accounts.types';

const mockApi = vi.mocked(api);

function makeProposal(amount: string = '-10700.00'): ReconciliationProposal {
	return {
		bankTransactionId: 42,
		transaction: {
			bookingDate: '2026-05-31',
			valueDate: '2026-05-31',
			amount,
			currency: 'CHF',
			counterpartyName: 'BATCH SALARIES',
		},
		candidates: [],
		notice: null,
		alreadyBooked: null,
	};
}

function makeAccount(
	id: number,
	number: string,
	name: string,
	accountType: AccountResponse['accountType'],
): AccountResponse {
	return {
		id,
		companyId: 1,
		number,
		name,
		accountType,
		parentId: null,
		active: true,
		role: null,
		postable: true,
		letterable: false,
		version: 1,
		createdAt: '2026-01-01T00:00:00Z',
		updatedAt: '2026-01-01T00:00:00Z',
	};
}

function makeAccounts(): AccountResponse[] {
	return [
		makeAccount(5000, '5000', 'Salaires', 'Expense'),
		makeAccount(5700, '5700', 'Charges sociales', 'Expense'),
		// Compte de classe 1 : proposé depuis la Story 34-3 (aucun filtre de classe) ;
		// seul le compte de la banque, reçu en `bankLedgerAccountId`, est écarté.
		makeAccount(1020, '1020', 'Banque', 'Asset'),
	];
}

/**
 * Choisit le compte `numero` sur la ligne `ligne`, avec le vrai `AccountAutocomplete` (Story
 * 28-2, B14). Partagé depuis la Story 36-2 (V-b) : sans compte choisi, `clientError` s'arrête
 * à « Ligne N : compte requis » avant toute règle de montant ou d'équilibre.
 */
async function choisirCompte(
	findByTestId: (id: string) => Promise<HTMLElement>,
	ligne: number,
	numero: string,
) {
	const rang = await findByTestId(`split-row-${ligne}`);
	const champ = rang.querySelector('input[aria-autocomplete="list"]') as HTMLInputElement;
	await fireEvent.focus(champ);
	const options = await waitFor(() => {
		const o = Array.from(rang.querySelectorAll<HTMLElement>('[role="option"]'));
		expect(o.length).toBeGreaterThan(0);
		return o;
	});
	const option = options.find((o) => o.textContent?.includes(numero));
	expect(option, `le compte ${numero} doit être proposé`).toBeTruthy();
	await fireEvent.mouseDown(option as HTMLElement);
}

describe('TransactionSplitModal', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	afterEach(() => {
		vi.restoreAllMocks();
	});

	it('renders modal with 2 initial lines + balance indicator when open=true', async () => {
		const { findByTestId } = render(TransactionSplitModal, {
			open: true,
			onOpenChange: () => {},
			bankAccountId: 17,
			proposal: makeProposal(),
			accounts: makeAccounts(),
			onSuccess: () => {},
		});

		// Au moins 2 lignes initialement (MIN_SPLITS).
		await findByTestId('split-row-0');
		await findByTestId('split-row-1');
		// Balance indicator + boutons.
		await findByTestId('split-balance-indicator');
		await findByTestId('split-submit');
		await findByTestId('split-cancel');
		await findByTestId('split-add-line');
	});

	// Story 19-5 — colonne projet par ligne visible seulement si projets présents.
	it('shows a per-line project selector when projects exist', async () => {
		listProjectsMock.mockResolvedValueOnce([
			{ id: 5, parentId: null, code: 'CHALET', name: 'Chalet', archived: false },
		]);
		const { findByTestId } = render(TransactionSplitModal, {
			open: true,
			onOpenChange: () => {},
			bankAccountId: 17,
			proposal: makeProposal(),
			accounts: makeAccounts(),
			onSuccess: () => {},
		});
		// Chaque ligne a son sélecteur projet.
		await findByTestId('split-line-project-0');
		await findByTestId('split-line-project-1');
	});

	it('hides the per-line project selector when no project exists', async () => {
		listProjectsMock.mockResolvedValueOnce([]);
		const { findByTestId, queryByTestId } = render(TransactionSplitModal, {
			open: true,
			onOpenChange: () => {},
			bankAccountId: 17,
			proposal: makeProposal(),
			accounts: makeAccounts(),
			onSuccess: () => {},
		});
		await findByTestId('split-row-0');
		expect(queryByTestId('split-line-project-0')).toBeNull();
	});

	it('disables submit until balance is exact', async () => {
		const { findByTestId } = render(TransactionSplitModal, {
			open: true,
			onOpenChange: () => {},
			bankAccountId: 17,
			proposal: makeProposal('-100.00'),
			accounts: makeAccounts(),
			onSuccess: () => {},
		});

		const submit = (await findByTestId('split-submit')) as HTMLButtonElement;
		// Initialement balance = 0 ≠ |-100| → submit disabled.
		expect(submit.disabled).toBe(true);
	});

	it('shows imbalance message when sum ≠ |tx.amount|', async () => {
		const { findByTestId } = render(TransactionSplitModal, {
			open: true,
			onOpenChange: () => {},
			bankAccountId: 17,
			proposal: makeProposal('-100.00'),
			accounts: makeAccounts(),
			onSuccess: () => {},
		});

		// Remplir un montant qui ne balance pas.
		const amount0 = (await findByTestId('split-amount-0')) as HTMLInputElement;
		await fireEvent.input(amount0, { target: { value: '40' } });
		const amount1 = (await findByTestId('split-amount-1')) as HTMLInputElement;
		await fireEvent.input(amount1, { target: { value: '30' } });

		const indicator = await findByTestId('split-balance-indicator');
		// 40 + 30 = 70 ≠ 100 → écart -30.00.
		expect(indicator.textContent).toContain('non équilibrée');
	});

	it('add-line and remove-line update split rows count (min 2, max 50)', async () => {
		const { findByTestId, queryByTestId } = render(TransactionSplitModal, {
			open: true,
			onOpenChange: () => {},
			bankAccountId: 17,
			proposal: makeProposal('-100.00'),
			accounts: makeAccounts(),
			onSuccess: () => {},
		});

		// Add une ligne.
		const addBtn = await findByTestId('split-add-line');
		await fireEvent.click(addBtn);
		await findByTestId('split-row-2');

		// Remove la 3e ligne.
		const removeBtn = await findByTestId('split-remove-2');
		await fireEvent.click(removeBtn);
		expect(queryByTestId('split-row-2')).toBeNull();

		// Tenter de retirer encore (min 2) — le bouton doit être disabled.
		const remove0 = (await findByTestId('split-remove-0')) as HTMLButtonElement;
		expect(remove0.disabled).toBe(true);
	});
});

// Story 28-2 (#520, AC9 — B14, C-28-2-26) — tests de SOUMISSION, avec le vrai
// `AccountAutocomplete` sur chacune des deux lignes, et des montants dont la
// somme égale la valeur absolue de la transaction (`balanceExact`).
describe('TransactionSplitModal — soumission rejetée (Story 28-2, B14)', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		vi.spyOn(console, 'error').mockImplementation(() => {});
	});

	afterEach(() => {
		vi.restoreAllMocks();
	});

	async function soumettre(rejet: unknown) {
		mockApi.splitTransaction.mockRejectedValue(rejet);
		const r = render(TransactionSplitModal, {
			open: true,
			onOpenChange: () => {},
			bankAccountId: 17,
			proposal: makeProposal('-100.00'),
			accounts: makeAccounts(),
			onSuccess: () => {},
		});
		await choisirCompte(r.findByTestId, 0, '5000');
		await fireEvent.input(await r.findByTestId('split-amount-0'), { target: { value: '60' } });
		await choisirCompte(r.findByTestId, 1, '5700');
		await fireEvent.input(await r.findByTestId('split-amount-1'), { target: { value: '40' } });
		const submit = (await r.findByTestId('split-submit')) as HTMLButtonElement;
		await waitFor(() => expect(submit.disabled).toBe(false));
		await fireEvent.click(submit);
		await waitFor(() => expect(mockApi.splitTransaction).toHaveBeenCalledTimes(1));
		return r;
	}

	it('(i) rejet new Error → split-error affiche le générique', async () => {
		const r = await soumettre(new Error('boom'));
		const erreur = await r.findByTestId('split-error');
		expect(erreur.textContent).toContain('Erreur inattendue.');
		expect(erreur.textContent).not.toContain('boom');
		expect(erreur.textContent).not.toContain('[object Object]');
	});

	it('(ii) rejet 412 BANK_ACCOUNT_NOT_CONFIGURED → l’indication du compte non configuré (M26)', async () => {
		const r = await soumettre({ code: 'BANK_ACCOUNT_NOT_CONFIGURED', message: 'm', status: 412 });
		expect(await r.findByTestId('split-bank-not-configured')).toBeTruthy();
	});
});

// v0172-correctifs (#880, AC-C2) — le rappel de la transaction dit sa date en forme suisse.
describe('TransactionSplitModal — date en forme suisse (v0172, #880)', () => {
	it('le rappel dit 31.05.2026, jamais 2026-05-31', async () => {
		const { findByTestId } = render(TransactionSplitModal, {
			open: true,
			onOpenChange: () => {},
			bankAccountId: 17,
			proposal: makeProposal(),
			accounts: makeAccounts(),
			onSuccess: () => {},
		});
		const summary = await findByTestId('split-tx-summary');
		expect(summary.textContent).toContain('31.05.2026');
		expect(summary.textContent).not.toContain('2026-05-31');
	});
});

// Story 34-3 (#967, AC-3, AC-4) — *Éclater* propose la même liste que *Écriture manuelle* : tout
// compte actif et imputable, de toute classe, sauf celui de la banque, qu'il reçoit désormais du
// parent (il l'écartait jusque-là par le seul accident du filtre des classes 5, 6 et 7). L'avis
// débiteurs / créanciers s'affiche sous la ligne concernée.
describe('TransactionSplitModal — contreparties et avis (Story 34-3)', () => {
	function planDeTest(): AccountResponse[] {
		return [
			makeAccount(201, '1000', 'Caisse', 'Asset'),
			makeAccount(202, '2100', 'Emprunt', 'Liability'),
			makeAccount(203, '3200', 'Ventes de marchandises', 'Revenue'),
			makeAccount(204, '4200', 'Achats de marchandises', 'Expense'),
			makeAccount(205, '5000', 'Salaires', 'Expense'),
			makeAccount(206, '6900', 'Frais financiers', 'Expense'),
			makeAccount(207, '7500', 'Produits annexes', 'Revenue'),
			makeAccount(208, '8500', 'Produits hors exploitation', 'Revenue'),
			makeAccount(209, '9100', 'Bilan d’ouverture', 'Liability'),
			makeAccount(210, '1020', 'Banque', 'Asset'),
			{ ...makeAccount(211, '2800', 'Fonds propres', 'Liability'), postable: false },
			{ ...makeAccount(212, '6500', 'Ancien compte', 'Expense'), active: false },
			{ ...makeAccount(213, '1100', 'Débiteurs', 'Asset'), role: 'Receivable' },
			{ ...makeAccount(214, '2000', 'Créanciers', 'Liability'), role: 'Payable' },
		];
	}
	const ATTENDU = [
		'1000', '1100', '2000', '2100', '3200', '4200', '5000', '6900', '7500', '8500', '9100',
	];

	function monter(bankLedgerAccountId: number | null) {
		return render(TransactionSplitModal, {
			open: true,
			onOpenChange: () => {},
			bankAccountId: 17,
			proposal: makeProposal(),
			accounts: planDeTest(),
			bankLedgerAccountId,
			onSuccess: () => {},
		});
	}

	/** Ce que les aides lisent du rendu (le type générique de `render` ne s'infère pas ici). */
	type Rendu = { findByTestId: (id: string) => Promise<HTMLElement> };

	async function optionsDeLigne(r: Rendu, ligne: number): Promise<HTMLElement[]> {
		const rang = await r.findByTestId(`split-row-${ligne}`);
		const champ = rang.querySelector('input[aria-autocomplete="list"]') as HTMLInputElement;
		await fireEvent.focus(champ);
		return waitFor(() => {
			const o = Array.from(rang.querySelectorAll<HTMLElement>('[role="option"]'));
			expect(o.length).toBeGreaterThan(0);
			return o;
		});
	}

	async function choisir(r: Rendu, ligne: number, numero: string) {
		const rang = await r.findByTestId(`split-row-${ligne}`);
		const champ = rang.querySelector('input[aria-autocomplete="list"]') as HTMLInputElement;
		await fireEvent.focus(champ);
		// Saisir le numéro : le libellé d'un compte déjà choisi filtrerait la liste.
		await fireEvent.input(champ, { target: { value: numero } });
		const option = (await optionsDeLigne(r, ligne)).find((o) => o.textContent?.includes(numero));
		expect(option, `le compte ${numero} doit être proposé`).toBeTruthy();
		await fireEvent.mouseDown(option as HTMLElement);
	}

	function numeros(options: HTMLElement[]): string[] {
		return options.map((o) => o.querySelector('.font-mono')?.textContent?.trim() ?? '').sort();
	}

	it('AC-3 — chaque ligne propose tout compte actif et imputable, sauf celui de la banque', async () => {
		const r = monter(210);
		expect(numeros(await optionsDeLigne(r, 0))).toEqual([...ATTENDU].sort());
		expect(numeros(await optionsDeLigne(r, 1))).toEqual([...ATTENDU].sort());
	});

	it('AC-3 — Éclater n’offre pas le compte de la banque', async () => {
		const r = monter(210);
		expect(numeros(await optionsDeLigne(r, 0))).not.toContain('1020');
	});

	it('AC-3 — repli null : aucun filtrage par identifiant (le serveur tranche)', async () => {
		const r = monter(null);
		expect(numeros(await optionsDeLigne(r, 0))).toContain('1020');
	});

	it('AC-4 — l’avis s’affiche sous la seule ligne concernée, avec son rôle', async () => {
		const r = monter(210);
		await choisir(r, 0, '5000');
		await choisir(r, 1, '1100');
		const avis = await r.findAllByTestId('counterparty-claim-account-notice');
		expect(avis).toHaveLength(1);
		expect(avis[0].getAttribute('data-role')).toBe('Receivable');
		expect(avis[0].getAttribute('data-line')).toBe('1');
		expect(avis[0].textContent).toContain('Ce compte est celui des débiteurs.');
		await choisir(r, 1, '2000');
		await waitFor(() => {
			const a = r.getAllByTestId('counterparty-claim-account-notice');
			expect(a).toHaveLength(1);
			expect(a[0].getAttribute('data-role')).toBe('Payable');
			expect(a[0].textContent).toContain('Ce compte est celui des créanciers.');
		});
		await choisir(r, 1, '6900');
		await waitFor(() =>
			expect(r.queryByTestId('counterparty-claim-account-notice')).toBeNull(),
		);
	});

	// Revue de code P1 (B1-2, E1-3, A1-3) — les lignes sont rendues par indice (`(i)`) : l'avis
	// doit suivre la ligne après une suppression, et deux lignes sur le même compte en portent
	// chacune un.
	it('AC-4 — après suppression d’une ligne, l’avis suit sa ligne (data-line du nouvel indice)', async () => {
		const r = monter(210);
		await fireEvent.click(await r.findByTestId('split-add-line'));
		await choisir(r, 0, '5000');
		await choisir(r, 1, '6900');
		await choisir(r, 2, '2000');
		await waitFor(() => {
			const a = r.getAllByTestId('counterparty-claim-account-notice');
			expect(a).toHaveLength(1);
			expect(a[0].getAttribute('data-line')).toBe('2');
		});
		await fireEvent.click(await r.findByTestId('split-remove-0'));
		await waitFor(() => {
			const a = r.getAllByTestId('counterparty-claim-account-notice');
			expect(a).toHaveLength(1);
			expect(a[0].getAttribute('data-role')).toBe('Payable');
			expect(a[0].getAttribute('data-line')).toBe('1');
		});
		// La rangée qui porte l'avis est bien celle qui suit la ligne 1, et aucune autre.
		expect(r.queryByTestId('split-row-notice-0')).toBeNull();
		expect(r.getByTestId('split-row-notice-1')).toBeTruthy();
	});

	it('AC-4 — deux lignes sur le compte débiteurs : un avis sous chacune', async () => {
		const r = monter(210);
		await choisir(r, 0, '1100');
		await choisir(r, 1, '1100');
		await waitFor(() => {
			const a = r.getAllByTestId('counterparty-claim-account-notice');
			expect(a.map((n) => n.getAttribute('data-line'))).toEqual(['0', '1']);
			expect(a.every((n) => n.getAttribute('data-role') === 'Receivable')).toBe(true);
		});
		// Sans projet, la rangée couvre les quatre colonnes de l'en-tête.
		const cellule = r.getByTestId('split-row-notice-0').querySelector('td') as HTMLTableCellElement;
		expect(cellule.getAttribute('colspan')).toBe('4');
		expect(r.getByTestId('split-lines-table').querySelectorAll('thead th')).toHaveLength(4);
	});

	it('AC-4 — avec des projets, la rangée de l’avis couvre les cinq colonnes', async () => {
		listProjectsMock.mockResolvedValueOnce([
			{ id: 5, parentId: null, code: 'CHALET', name: 'Chalet', archived: false },
		]);
		const r = monter(210);
		await r.findByTestId('split-line-project-0');
		await choisir(r, 0, '1100');
		const rangee = await r.findByTestId('split-row-notice-0');
		const cellule = rangee.querySelector('td') as HTMLTableCellElement;
		expect(cellule.getAttribute('colspan')).toBe('5');
		// Contrôle croisé : l'en-tête compte bien cinq colonnes.
		expect(r.getByTestId('split-lines-table').querySelectorAll('thead th')).toHaveLength(5);
	});
});

// Story 35-7 (#1000, AC3, D1, C-e350-36) — une ligne dont le compte se révèle être celui de la
// banque, une fois son identifiant résolu par le parent, est vidée (champ ET valeur) ; l'autre
// ligne reste intacte, et le message existant nomme la ligne en défaut.
describe('TransactionSplitModal — compte retiré de la sélection (Story 35-7, AC3)', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	function plan(): AccountResponse[] {
		return [
			makeAccount(401, '6900', 'Frais financiers', 'Expense'),
			makeAccount(410, '1020', 'Banque', 'Asset'),
		];
	}

	function monter() {
		// L'hôte passe les props une à une (un `rerender` rejouerait la remise à zéro du dialogue et
		// viderait toutes les lignes quel que soit le code : le test passerait à vide).
		return render(CounterpartyModalHost, {
			modal: 'split',
			proposal: makeProposal('-100.00'),
			accounts: plan(),
			lateBankLedgerAccountId: 410,
		});
	}

	type Rendu = { findByTestId: (id: string) => Promise<HTMLElement> };

	async function resoudreBanque(r: { getByTestId: (id: string) => HTMLElement }) {
		await fireEvent.click(r.getByTestId('host-resolve-bank'));
	}

	async function champ(r: Rendu, ligne: number): Promise<HTMLInputElement> {
		const rang = await r.findByTestId(`split-row-${ligne}`);
		return rang.querySelector('input[aria-autocomplete="list"]') as HTMLInputElement;
	}

	async function choisir(r: Rendu, ligne: number, numero: string) {
		const rang = await r.findByTestId(`split-row-${ligne}`);
		const c = await champ(r, ligne);
		await fireEvent.focus(c);
		await fireEvent.input(c, { target: { value: numero } });
		const options = await waitFor(() => {
			const o = Array.from(rang.querySelectorAll<HTMLElement>('[role="option"]'));
			expect(o.length).toBeGreaterThan(0);
			return o;
		});
		const option = options.find((o) => o.textContent?.includes(numero));
		expect(option, `le compte ${numero} doit être proposé`).toBeTruthy();
		await fireEvent.mouseDown(option as HTMLElement);
	}

	async function remplir(r: Rendu, ligne: number, numero: string, montant: string) {
		await choisir(r, ligne, numero);
		await fireEvent.input(await r.findByTestId(`split-amount-${ligne}`), {
			target: { value: montant },
		});
	}

	it('la ligne 1 sur le compte de la banque → elle seule vidée, « Ligne 1 : compte requis »', async () => {
		const r = monter();
		await remplir(r, 0, '1020', '60');
		await remplir(r, 1, '6900', '40');
		const submit = (await r.findByTestId('split-submit')) as HTMLButtonElement;
		// Prémisse : les deux choix ont pris.
		await waitFor(() => expect(submit.disabled).toBe(false));
		expect((await champ(r, 0)).value).toBe('1020 — Banque');

		await resoudreBanque(r);

		// Story 58-6 (#1079) : la VALEUR d'abord. Le champ qui s'affiche vide ne prouve rien : il
		// l'est dès que le compte de la banque sort des options, même si la ligne garde son
		// identifiant. Le bouton désactivé, le message et l'absence d'appel prouvent que la valeur
		// est nulle ; le texte du champ vient ensuite.
		await waitFor(() => expect(submit.disabled).toBe(true));
		expect(await r.findByText('Ligne 1 : compte requis')).toBeTruthy();
		await fireEvent.click(submit);
		expect(mockApi.splitTransaction).not.toHaveBeenCalled();
		await waitFor(async () => expect((await champ(r, 0)).value).toBe(''));
		expect((await champ(r, 1)).value).toBe('6900 — Frais financiers');
	});

	it('la ligne 2 seule sur le compte de la banque, la ligne 1 remplie → « Ligne 2 : compte requis »', async () => {
		const r = monter();
		await remplir(r, 0, '6900', '60');
		await remplir(r, 1, '1020', '40');
		const submit = (await r.findByTestId('split-submit')) as HTMLButtonElement;
		await waitFor(() => expect(submit.disabled).toBe(false));

		await resoudreBanque(r);

		// Story 58-6 (#1079) : la valeur d'abord (voir le test précédent), le texte du champ ensuite.
		await waitFor(() => expect(submit.disabled).toBe(true));
		expect(await r.findByText('Ligne 2 : compte requis')).toBeTruthy();
		expect(mockApi.splitTransaction).not.toHaveBeenCalled();
		await waitFor(async () => expect((await champ(r, 1)).value).toBe(''));
		expect((await champ(r, 0)).value).toBe('6900 — Frais financiers');
	});

	// Story 58-6 (#1083) : la boucle de l'effet parcourt TOUTES les lignes, non les deux premières.
	// Le message nomme la première ligne en défaut (« Ligne 2 »), non la dernière.
	it('trois lignes, les lignes 2 et 3 sur le compte de la banque → toutes deux vidées, « Ligne 2 : compte requis »', async () => {
		const r = monter();
		await fireEvent.click(await r.findByTestId('split-add-line'));
		await remplir(r, 0, '6900', '50');
		await remplir(r, 1, '1020', '30');
		await remplir(r, 2, '1020', '20');
		const submit = (await r.findByTestId('split-submit')) as HTMLButtonElement;
		// Prémisse : les trois choix ont pris.
		await waitFor(() => expect(submit.disabled).toBe(false));
		expect((await champ(r, 2)).value).toBe('1020 — Banque');

		await resoudreBanque(r);

		await waitFor(() => expect(submit.disabled).toBe(true));
		expect(await r.findByText('Ligne 2 : compte requis')).toBeTruthy();
		expect(r.queryByText('Ligne 3 : compte requis')).toBeNull();
		await fireEvent.click(submit);
		expect(mockApi.splitTransaction).not.toHaveBeenCalled();
		await waitFor(async () => expect((await champ(r, 1)).value).toBe(''));
		expect((await champ(r, 2)).value).toBe('');
		expect((await champ(r, 0)).value).toBe('6900 — Frais financiers');
	});

	it('deux comptes restés proposés ne sont pas touchés, la ligne 3 sur la banque est vidée', async () => {
		const r = monter();
		await fireEvent.click(await r.findByTestId('split-add-line'));
		await remplir(r, 0, '6900', '50');
		await remplir(r, 1, '6900', '30');
		await remplir(r, 2, '1020', '20');
		const submit = (await r.findByTestId('split-submit')) as HTMLButtonElement;
		await waitFor(() => expect(submit.disabled).toBe(false));

		await resoudreBanque(r);

		await waitFor(() => expect(submit.disabled).toBe(true));
		expect(await r.findByText('Ligne 3 : compte requis')).toBeTruthy();
		expect((await champ(r, 0)).value).toBe('6900 — Frais financiers');
		expect((await champ(r, 1)).value).toBe('6900 — Frais financiers');
		await waitFor(async () => expect((await champ(r, 2)).value).toBe(''));
		await fireEvent.click(submit);
		expect(mockApi.splitTransaction).not.toHaveBeenCalled();
	});
});

// Story 36-2 (#1150) — le bouton *Éclater* s'active quand les lignes sont complètes et
// équilibrées, sinon le message dit ce qui manque ; les montants partent en chaînes décimales
// au centime (le serveur, `serde-str`, refuse un nombre JSON). Le mock d'`i18nMsg` rend le
// repli interpolé : ces tests vérifient le repli et ses arguments, le catalogue l'est par
// `split-i18n.test.ts` et par l'E2E R0.
describe('TransactionSplitModal — Éclater, montants et messages (Story 36-2)', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		mockApi.splitTransaction.mockResolvedValue({ bankTransactionId: 42, journalEntryId: 7 });
	});

	afterEach(() => {
		vi.restoreAllMocks();
	});

	/** Monte le dialogue, choisit `5000` et `5700`, et tape les deux montants. */
	async function remplir(montants: [string, string], txAmount = '-100.00') {
		const r = render(TransactionSplitModal, {
			open: true,
			onOpenChange: () => {},
			bankAccountId: 17,
			proposal: makeProposal(txAmount),
			accounts: makeAccounts(),
			onSuccess: () => {},
		});
		await choisirCompte(r.findByTestId, 0, '5000');
		await choisirCompte(r.findByTestId, 1, '5700');
		await fireEvent.input(await r.findByTestId('split-amount-0'), {
			target: { value: montants[0] },
		});
		await fireEvent.input(await r.findByTestId('split-amount-1'), {
			target: { value: montants[1] },
		});
		const submit = (await r.findByTestId('split-submit')) as HTMLButtonElement;
		return { r, submit };
	}

	/** Clique sur *Éclater* et rend les montants envoyés. */
	async function envoyer(submit: HTMLButtonElement): Promise<unknown[]> {
		await waitFor(() => expect(submit.disabled).toBe(false));
		await fireEvent.click(submit);
		await waitFor(() => expect(mockApi.splitTransaction).toHaveBeenCalledTimes(1));
		return mockApi.splitTransaction.mock.calls[0][2].map((s) => s.amount);
	}

	it('V-a — les montants partent en chaînes décimales au centime, jamais en nombres', async () => {
		const { submit } = await remplir(['60', '40']);
		const montants = await envoyer(submit);
		expect(montants.map((m) => typeof m)).toEqual(['string', 'string']);
		expect(montants).toEqual(['60.00', '40.00']);
	});

	it('V-a — « 70,00 » et « 30,00 » tapés partent « 70.00 » et « 30.00 »', async () => {
		const { submit } = await remplir(['70,00', '30,00']);
		expect(await envoyer(submit)).toEqual(['70.00', '30.00']);
	});

	it('V-b — somme fausse : le message et l’indicateur disent l’attendu et l’écart', async () => {
		const { r, submit } = await remplir(['40', '30']);
		const message = await r.findByTestId('split-client-error');
		// Revue P1 (B3) : le message dit quoi faire, l'indicateur constate — deux phrases.
		expect(message.textContent?.trim()).toBe(
			'Ajustez les montants : leur somme doit égaler 100.00 (écart actuel -30.00).',
		);
		const indicateur = await r.findByTestId('split-balance-indicator');
		expect(indicateur.textContent?.trim()).toBe(
			'Balance non équilibrée — attendu 100.00, écart -30.00',
		);
		expect(submit.disabled).toBe(true);
	});

	// C-36-2-19 (R0 (f)) : le serveur rend le montant normalisé (« -388.2 ») ; l'attendu se
	// dit à deux décimales, comme l'écart.
	it('V-b — un montant normalisé par le serveur se dit à deux décimales', async () => {
		const { r } = await remplir(['10', '20'], '-388.2');
		const message = await r.findByTestId('split-client-error');
		expect(message.textContent).toContain('doit égaler 388.20 (écart actuel -358.20)');
		const indicateur = await r.findByTestId('split-balance-indicator');
		expect(indicateur.textContent).toContain('attendu 388.20, écart -358.20');
	});

	// Revue P1 (B3) : une ligne sans montant lu — l'écart n'est pas calculable, et se dit tel.
	it('V-b — une ligne sans montant lu : « écart non calculable », jamais « écart — »', async () => {
		const { r } = await remplir(['', '40']);
		const indicateur = await r.findByTestId('split-balance-indicator');
		expect(indicateur.textContent?.trim()).toBe(
			'Balance non équilibrée — attendu 100.00, écart non calculable',
		);
		expect(indicateur.textContent).not.toContain('—,');
	});

	// Revue P1 (E4) : une transaction au crédit (montant positif) s'éclate comme un débit.
	it('V-f — transaction au crédit : la valeur absolue est attendue, l’envoi passe', async () => {
		const { r, submit } = await remplir(['60', '40'], '100.00');
		expect(r.queryByTestId('split-client-error')).toBeNull();
		expect((await r.findByTestId('split-balance-indicator')).textContent).toContain(
			'Balance équilibrée : 100.00',
		);
		expect(await envoyer(submit)).toEqual(['60.00', '40.00']);
	});

	// Revue P1 (E4) : un montant de transaction illisible (`expectedBig` nul) — rien ne part,
	// l'écart n'est pas calculable et le message générique le dit.
	it('V-f — montant de transaction illisible : grisé, écart non calculable', async () => {
		const { r, submit } = await remplir(['60', '40'], 'abc');
		expect(submit.disabled).toBe(true);
		expect((await r.findByTestId('split-client-error')).textContent?.trim()).toBe(
			"L'éclatement n'équilibre pas le montant de la transaction.",
		);
		expect((await r.findByTestId('split-balance-indicator')).textContent).toContain(
			'écart non calculable',
		);
	});

	// Revue P1 (E4) : un montant de transaction nul — aucune somme positive ne l'égale.
	it('V-f — montant de transaction nul : grisé, l’écart est dit', async () => {
		const { r, submit } = await remplir(['60', '40'], '0.00');
		expect(submit.disabled).toBe(true);
		expect((await r.findByTestId('split-client-error')).textContent).toContain(
			'doit égaler 0.00 (écart actuel +100.00)',
		);
	});

	// Revue P1 (E4, B4) : les lignes sont rendues par indice ; après la suppression d'une ligne
	// saisie, les lectures et l'envoi suivent les lignes restantes.
	it('V-f — envoi après suppression d’une ligne saisie : les montants des lignes restantes', async () => {
		const { r } = await remplir(['5', '40']);
		await fireEvent.click(await r.findByTestId('split-add-line'));
		await choisirCompte(r.findByTestId, 2, '5000');
		await fireEvent.input(await r.findByTestId('split-amount-2'), { target: { value: '60' } });
		await fireEvent.click(await r.findByTestId('split-remove-0'));
		const submit = (await r.findByTestId('split-submit')) as HTMLButtonElement;
		expect(await envoyer(submit)).toEqual(['40.00', '60.00']);
		expect(mockApi.splitTransaction.mock.calls[0][2].map((s) => s.counterpartyAccountId)).toEqual([
			5700, 5000,
		]);
	});

	it('V-c — à l’ouverture, le message nommé dit « Ligne 1 : compte requis »', async () => {
		const { findByTestId } = render(TransactionSplitModal, {
			open: true,
			onOpenChange: () => {},
			bankAccountId: 17,
			proposal: makeProposal('-100.00'),
			accounts: makeAccounts(),
			onSuccess: () => {},
		});
		const message = await findByTestId('split-client-error');
		expect(message.textContent?.trim()).toBe('Ligne 1 : compte requis');
	});

	// V-d — la table de D5 : une saisie de la ligne 1, la ligne 2 complétant la somme.
	it.each([
		['', 'Ligne 1 : montant > 0 requis'],
		['0', 'Ligne 1 : montant > 0 requis'],
		['0,00', 'Ligne 1 : montant > 0 requis'],
		['-70.00', 'Ligne 1 : montant illisible'],
		['7O', 'Ligne 1 : montant illisible'],
		['1.234', 'Ligne 1 : montant ambigu'],
		['0.005', 'Ligne 1 : deux décimales au plus'],
		['30.0050', 'Ligne 1 : deux décimales au plus'],
		// Revue P1 (B1, E5) : une espace (en tête, en fin, entre les milliers) rend la saisie
		// illisible, et le message dit « sans signe ni espace ».
		[' 70', 'Ligne 1 : montant illisible'],
		['70 ', 'sans signe ni espace'],
		['1 200.50', 'Ligne 1 : montant illisible'],
		// Revue P1 (B4) : seize chiffres entiers dépassent la grammaire (quinze au plus).
		['1234567890123456', 'Ligne 1 : montant illisible'],
	])('V-d — « %s » : bouton grisé, « %s »', async (saisie, attendu) => {
		const { r, submit } = await remplir([saisie, '40']);
		const message = await r.findByTestId('split-client-error');
		expect(message.textContent).toContain(attendu);
		expect(submit.disabled).toBe(true);
	});

	it.each([
		['60.0000', '40', '-100.00', ['60.00', '40.00']],
		['70,00', '30', '-100.00', ['70.00', '30.00']],
		["1'200.50", '100', '-1300.50', ['1200.50', '100.00']],
		['0.10', '0.20', '-0.30', ['0.10', '0.20']],
		// Revue P1 (B4, E3) : l'apostrophe typographique U+2019 (ponctuation automatique de macOS
		// et d'iOS), et quinze chiffres entiers, le plus long que lit la grammaire.
		['1’200.50', '100', '-1300.50', ['1200.50', '100.00']],
		['123456789012345', '1', '-123456789012346.00', ['123456789012345.00', '1.00']],
	])('V-d — « %s » + « %s » sur %s : lu, part %j', async (a, b, tx, attendus) => {
		const { r, submit } = await remplir([a, b], tx);
		expect(r.queryByTestId('split-client-error')).toBeNull();
		expect(await envoyer(submit)).toEqual(attendus);
	});

	it('V-e — lignes équilibrées : l’indicateur dit « Balance équilibrée » et le montant', async () => {
		const { r, submit } = await remplir(['60', '40']);
		await waitFor(() => expect(submit.disabled).toBe(false));
		const indicateur = await r.findByTestId('split-balance-indicator');
		expect(indicateur.textContent).toContain('Balance équilibrée');
		expect(indicateur.textContent).toContain('100.00');
		expect(indicateur.textContent).not.toContain('{ $');
	});
});
