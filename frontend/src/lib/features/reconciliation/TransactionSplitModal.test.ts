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

vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string) => fallback,
}));

// Story 19-5 — mock listProjects (chargé via $effect dans le composant).
const listProjectsMock = vi.fn(async () => [] as unknown[]);
vi.mock('$lib/features/projects/projects.api', () => ({
	listProjects: () => listProjectsMock(),
}));

import * as api from './reconciliation.api';
import TransactionSplitModal from './TransactionSplitModal.svelte';
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
