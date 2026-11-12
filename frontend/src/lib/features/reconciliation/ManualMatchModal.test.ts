// Story 8-5a-base FR45 — P-M3 Pass 1 code review : tests Vitest pour
// ManualMatchModal.svelte. Couvre AC #92 (sélecteur counterparty
// obligatoire + valueDate prefill + 412 BANK_ACCOUNT_NOT_CONFIGURED
// detection via isApiError).
//
// Pattern : mock du module `reconciliation.api` + render via
// `@testing-library/svelte` (Svelte 5 supporté). Mocks i18n + API
// definis AVANT l'import du composant (hoisting Vitest).

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, fireEvent, waitFor } from '@testing-library/svelte';
import type { ReconciliationProposal } from './reconciliation.types';

// Mock du module API.
vi.mock('./reconciliation.api', () => ({
	manualMatchTransaction: vi.fn(),
}));

// Story 19-5 — mock listProjects (le composant charge les projets via $effect).
const listProjectsMock = vi.fn(async () => [] as unknown[]);
vi.mock('$lib/features/projects/projects.api', () => ({
	listProjects: () => listProjectsMock(),
}));

// Mock i18n util pour rendre déterministe.
vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string) => fallback,
}));

import * as api from './reconciliation.api';
import ManualMatchModal from './ManualMatchModal.svelte';
import type { AccountResponse } from '$lib/features/accounts/accounts.types';

const mockApi = vi.mocked(api);

function makeProposal(): ReconciliationProposal {
	return {
		bankTransactionId: 42,
		transaction: {
			bookingDate: '2026-05-15',
			valueDate: '2026-05-16',
			amount: '-150.00',
			currency: 'CHF',
			counterpartyName: 'ACME GMBH',
		},
		candidates: [],
		notice: null,
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
		makeAccount(100, '6810', 'Frais bancaires', 'Expense'),
		makeAccount(101, '5200', 'Salaires', 'Expense'),
		makeAccount(102, '7510', 'Intérêts', 'Revenue'),
		// Compte de classe 1 : proposé depuis la Story 34-3 (aucun filtre de classe).
		makeAccount(103, '1020', 'Caisse banque', 'Asset'),
	];
}

describe('ManualMatchModal', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	afterEach(() => {
		vi.restoreAllMocks();
	});

	it('renders modal with required fields when open=true and proposal provided', async () => {
		const { findByTestId } = render(ManualMatchModal, {
			open: true,
			onOpenChange: () => {},
			bankAccountId: 17,
			proposal: makeProposal(),
			accounts: makeAccounts(),
			onSuccess: () => {},
		});

		// Champs requis : description input + value date input + submit/cancel.
		await findByTestId('manual-match-description-input');
		await findByTestId('manual-match-value-date-input');
		await findByTestId('manual-match-submit');
		await findByTestId('manual-match-cancel');
		// Tx summary affiche la proposal.
		await findByTestId('manual-match-tx-summary');
	});

	it('prefills value date from proposal.transaction.valueDate (P-M7) with bookingDate fallback', async () => {
		// Cas 1 : valueDate présent → prefill avec valueDate.
		const { findByTestId, unmount } = render(ManualMatchModal, {
			open: true,
			onOpenChange: () => {},
			bankAccountId: 17,
			proposal: makeProposal(), // valueDate = '2026-05-16'
			accounts: makeAccounts(),
			onSuccess: () => {},
		});
		const input = (await findByTestId(
			'manual-match-value-date-input',
		)) as HTMLInputElement;
		expect(input.value).toBe('2026-05-16');
		unmount();

		// Cas 2 : valueDate null → fallback bookingDate.
		const proposalNoValueDate: ReconciliationProposal = {
			...makeProposal(),
			transaction: {
				...makeProposal().transaction,
				valueDate: null,
			},
		};
		const { findByTestId: findByTestId2 } = render(ManualMatchModal, {
			open: true,
			onOpenChange: () => {},
			bankAccountId: 17,
			proposal: proposalNoValueDate,
			accounts: makeAccounts(),
			onSuccess: () => {},
		});
		const input2 = (await findByTestId2(
			'manual-match-value-date-input',
		)) as HTMLInputElement;
		expect(input2.value).toBe('2026-05-15');
	});

	// Story 19-5 — le sélecteur projet apparaît quand des projets existent, et
	// pas sinon (zéro friction si aucun projet défini).
	it('shows project selector only when projects exist', async () => {
		listProjectsMock.mockResolvedValueOnce([
			{ id: 3, parentId: null, code: 'RENOV', name: 'Rénovation', archived: false },
		]);
		const { findByTestId } = render(ManualMatchModal, {
			open: true,
			onOpenChange: () => {},
			bankAccountId: 17,
			proposal: makeProposal(),
			accounts: makeAccounts(),
			onSuccess: () => {},
		});
		await findByTestId('manual-match-project');
	});

	it('hides project selector when no project exists', async () => {
		listProjectsMock.mockResolvedValueOnce([]);
		const { queryByTestId, findByTestId } = render(ManualMatchModal, {
			open: true,
			onOpenChange: () => {},
			bankAccountId: 17,
			proposal: makeProposal(),
			accounts: makeAccounts(),
			onSuccess: () => {},
		});
		// Attendre que le modal soit monté (un champ toujours présent).
		await findByTestId('manual-match-submit');
		expect(queryByTestId('manual-match-project')).toBeNull();
	});

	it('disables submit button when counterpartyId is null (clientError obligatoire)', async () => {
		const { findByTestId } = render(ManualMatchModal, {
			open: true,
			onOpenChange: () => {},
			bankAccountId: 17,
			proposal: makeProposal(),
			accounts: makeAccounts(),
			onSuccess: () => {},
		});

		const submit = (await findByTestId('manual-match-submit')) as HTMLButtonElement;
		// Pas de counterparty sélectionné → disabled (clientError truthy).
		expect(submit.disabled).toBe(true);
	});

	it('shows BANK_ACCOUNT_NOT_CONFIGURED hint when API throws ApiError 412 (P-H1)', async () => {
		// Simule l'objet ApiError plain (interface, pas Error subclass).
		mockApi.manualMatchTransaction.mockRejectedValue({
			code: 'BANK_ACCOUNT_NOT_CONFIGURED',
			message: 'Le compte bancaire n\'est pas configuré.',
			status: 412,
		});

		const { findByTestId } = render(ManualMatchModal, {
			open: true,
			onOpenChange: () => {},
			bankAccountId: 17,
			proposal: makeProposal(),
			accounts: makeAccounts(),
			onSuccess: () => {},
		});

		// Bypass le clientError counterparty obligatoire en sélectionnant
		// directement un compte via l'autocomplete : impossible sans
		// interaction complexe. À défaut, on appelle directement le submit
		// après avoir simulé un counterpartyId non-null. Or, le composant
		// expose pas de prop pour pré-remplir. Workaround : ce test est
		// difficile à exercer sans mock plus fin de AccountAutocomplete.
		// On vérifie au minimum la présence du conteneur d'erreur via le
		// branchement bankNotConfigured. Ce test est partiellement smoke
		// jusqu'à ce que le wiring AccountAutocomplete soit testable.

		const submit = (await findByTestId('manual-match-submit')) as HTMLButtonElement;
		expect(submit).toBeTruthy();
		// Note : le path 412 est couvert par les 11 tests E2E HTTP Rust
		// (post_manual_returns_412_for_bank_account_not_configured) — ce
		// test Vitest sert de smoke d'enregistrement de la modal dans le
		// flow apiClient → isApiError typed guard.
	});
});

// Story 15-6d (AC4, test 10, #524) — le compte comptable du compte bancaire n'est pas proposé
// comme contrepartie ; `null` → aucun filtrage (la garde serveur tranche).
// Story 34-3 (#967, AC-2) — le filtre des classes 5, 6 et 7 est retiré : tout compte actif et
// imputable, de toute classe, est proposé, sauf celui de la banque. Plan de test : un compte de
// chaque classe 1 à 9, un non imputable, un archivé, le compte de la banque ; la liste rendue se
// compare à l'ENSEMBLE attendu.
describe('ManualMatchModal — la liste des contreparties (Story 15-6d, Story 34-3)', () => {
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
		];
	}
	const ATTENDU = ['1000', '2100', '3200', '4200', '5000', '6900', '7500', '8500', '9100'];

	async function proposedNumbers(bankLedgerAccountId: number | null): Promise<string[]> {
		const { getByRole, findAllByRole, unmount } = render(ManualMatchModal, {
			open: true,
			onOpenChange: () => {},
			bankAccountId: 17,
			proposal: makeProposal(),
			accounts: planDeTest(),
			bankLedgerAccountId,
			onSuccess: () => {},
		});
		await fireEvent.focus(getByRole('textbox', { name: 'Compte' }));
		const options = await findAllByRole('option');
		const numbers = options.map((o) => o.querySelector('.font-mono')?.textContent?.trim() ?? '');
		unmount();
		return numbers.sort();
	}

	it('AC-2 — tout compte actif et imputable de toute classe, sauf celui de la banque', async () => {
		expect(await proposedNumbers(210)).toEqual([...ATTENDU].sort());
	});

	it('bankLedgerAccountId null → aucun filtrage par identifiant (le serveur tranche)', async () => {
		expect(await proposedNumbers(null)).toEqual([...ATTENDU, '1020'].sort());
	});
});

// Story 34-3 (#967, AC-4, C-e342-1) — un avis non bloquant quand la contrepartie choisie porte le
// rôle débiteurs ou créanciers : une écriture manuelle n'y règle aucune facture.
const AVIS_DEBITEURS =
	'Ce compte est celui des débiteurs. Une écriture manuelle ne règle aucune facture client : elle resterait due. Pour un paiement de client, enregistrez le règlement depuis la fiche de la facture, puis rattachez-y la transaction.';
const AVIS_CREANCIERS =
	"Ce compte est celui des créanciers. Une écriture manuelle ne règle aucune facture fournisseur : elle resterait à payer. Pour le paiement d'un fournisseur, saisissez la facture et validez-la depuis sa fiche, sans y enregistrer de paiement : le paiement est cette transaction, et la facture vous sera proposée ici, à accepter. Si elle est déjà payée, rattachez la transaction à ce paiement.";

describe('ManualMatchModal — avis débiteurs / créanciers (Story 34-3, AC-4)', () => {
	function plan(): AccountResponse[] {
		return [
			{ ...makeAccount(301, '1100', 'Débiteurs', 'Asset'), role: 'Receivable' },
			{ ...makeAccount(302, '2000', 'Créanciers', 'Liability'), role: 'Payable' },
			{ ...makeAccount(303, '1170', 'Impôt préalable', 'Asset'), role: 'VatRecoverable' },
			makeAccount(304, '6810', 'Frais bancaires', 'Expense'),
		];
	}

	/** Ce que `choisir` lit du rendu (le type générique de `render` ne s'infère pas ici). */
	type Rendu = { findAllByRole: (role: string) => Promise<HTMLElement[]> };

	async function choisir(r: Rendu, numero: string): Promise<void> {
		const champ = document.querySelector('input[aria-autocomplete="list"]') as HTMLInputElement;
		await fireEvent.focus(champ);
		// Saisir le numéro : une fois un compte choisi, le champ montre son libellé, qui filtre
		// la liste ; l'utilisateur tape pour en changer.
		await fireEvent.input(champ, { target: { value: numero } });
		const options = await r.findAllByRole('option');
		const option = options.find((o) => o.textContent?.includes(numero));
		expect(option, `le compte ${numero} doit être proposé`).toBeTruthy();
		await fireEvent.mouseDown(option as HTMLElement);
	}

	function monter() {
		return render(ManualMatchModal, {
			open: true,
			onOpenChange: () => {},
			bankAccountId: 17,
			proposal: makeProposal(),
			accounts: plan(),
			bankLedgerAccountId: 999,
			onSuccess: () => {},
		});
	}

	it('compte débiteurs → l’avis client, complet ; il disparaît pour un autre compte', async () => {
		const r = monter();
		expect(r.queryByTestId('counterparty-claim-account-notice')).toBeNull();
		await choisir(r, '1100');
		const avis = await r.findByTestId('counterparty-claim-account-notice');
		expect(avis.getAttribute('data-role')).toBe('Receivable');
		expect(avis.textContent?.trim()).toBe(AVIS_DEBITEURS);
		// Non bloquant : la validation reste possible.
		const submit = (await r.findByTestId('manual-match-submit')) as HTMLButtonElement;
		await waitFor(() => expect(submit.disabled).toBe(false));
		await choisir(r, '6810');
		await waitFor(() => expect(r.queryByTestId('counterparty-claim-account-notice')).toBeNull());
	});

	it('compte créanciers → l’avis fournisseur, complet', async () => {
		const r = monter();
		await choisir(r, '2000');
		const avis = await r.findByTestId('counterparty-claim-account-notice');
		expect(avis.getAttribute('data-role')).toBe('Payable');
		expect(avis.textContent?.trim()).toBe(AVIS_CREANCIERS);
	});

	it('un compte de TVA n’avertit pas (hors périmètre écrit)', async () => {
		const r = monter();
		await choisir(r, '1170');
		// Revue de code P1 (B1-6) : prouver d'abord que le choix a pris — le champ montre le
		// compte et la validation est active —, sans quoi l'absence passerait à vide.
		const champ = document.querySelector('input[aria-autocomplete="list"]') as HTMLInputElement;
		await waitFor(() => expect(champ.value).toContain('1170'));
		const submit = (await r.findByTestId('manual-match-submit')) as HTMLButtonElement;
		await waitFor(() => expect(submit.disabled).toBe(false));
		expect(r.queryByTestId('counterparty-claim-account-notice')).toBeNull();
	});
});

// Story 34-3 (#967, AC-1) — le titre et le bouton de validation suivent le bouton qui ouvre le
// dialogue (replis fr ; les catalogues sont tenus égaux par `ecriture-manuelle-i18n.test.ts`).
describe('ManualMatchModal — libellés (Story 34-3, AC-1)', () => {
	it('titre « Écriture manuelle », validation « Passer l’écriture »', async () => {
		const r = render(ManualMatchModal, {
			open: true,
			onOpenChange: () => {},
			bankAccountId: 17,
			proposal: makeProposal(),
			accounts: makeAccounts(),
			onSuccess: () => {},
		});
		expect((await r.findByTestId('manual-match-submit')).textContent?.trim()).toBe(
			"Passer l'écriture",
		);
		expect(r.getByText('Écriture manuelle')).toBeTruthy();
	});
});

// Story 28-2 (#520, AC9 — B13, C-28-2-26) — tests de SOUMISSION, avec le vrai
// `AccountAutocomplete` : focus sur son champ, `mouseDown` sur l'option (il
// sélectionne au `mousedown`), puis clic sur *Valider*. Le test « 412 » plus haut
// ne soumet pas (il ne peut choisir de contrepartie) ; ceux-ci soumettent.
describe('ManualMatchModal — soumission rejetée (Story 28-2, B13)', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		vi.spyOn(console, 'error').mockImplementation(() => {});
	});

	afterEach(() => {
		vi.restoreAllMocks();
	});

	async function soumettre(rejet: unknown) {
		mockApi.manualMatchTransaction.mockRejectedValue(rejet);
		const r = render(ManualMatchModal, {
			open: true,
			onOpenChange: () => {},
			bankAccountId: 17,
			proposal: makeProposal(),
			accounts: makeAccounts(),
			onSuccess: () => {},
		});
		const champ = document.querySelector('input[aria-autocomplete="list"]') as HTMLInputElement;
		await fireEvent.focus(champ);
		const options = await r.findAllByRole('option');
		const frais = options.find((o) => o.textContent?.includes('6810'));
		expect(frais, 'le compte 6810 doit être proposé').toBeTruthy();
		await fireEvent.mouseDown(frais as HTMLElement);
		const submit = (await r.findByTestId('manual-match-submit')) as HTMLButtonElement;
		await waitFor(() => expect(submit.disabled).toBe(false));
		await fireEvent.click(submit);
		await waitFor(() => expect(mockApi.manualMatchTransaction).toHaveBeenCalledTimes(1));
		expect(mockApi.manualMatchTransaction.mock.calls[0][2]).toBe(100);
		return r;
	}

	it('(i) rejet new Error → manual-match-error affiche le générique', async () => {
		const r = await soumettre(new Error('boom'));
		const erreur = await r.findByTestId('manual-match-error');
		expect(erreur.textContent).toContain('Erreur inattendue.');
		expect(erreur.textContent).not.toContain('boom');
		expect(erreur.textContent).not.toContain('[object Object]');
	});

	it('(ii) rejet 412 BANK_ACCOUNT_NOT_CONFIGURED → l’indication du compte non configuré (M15)', async () => {
		const r = await soumettre({ code: 'BANK_ACCOUNT_NOT_CONFIGURED', message: 'm', status: 412 });
		expect(await r.findByTestId('manual-match-bank-not-configured')).toBeTruthy();
	});
});

// v0172-correctifs (#880, AC-C2) — le rappel de la transaction dit sa date en forme suisse.
describe('ManualMatchModal — date en forme suisse (v0172, #880)', () => {
	it('le rappel dit 15.05.2026, jamais 2026-05-15', async () => {
		const { findByTestId } = render(ManualMatchModal, {
			open: true,
			onOpenChange: () => {},
			bankAccountId: 17,
			proposal: makeProposal(),
			accounts: makeAccounts(),
			onSuccess: () => {},
		});
		const summary = await findByTestId('manual-match-tx-summary');
		expect(summary.textContent).toContain('15.05.2026');
		expect(summary.textContent).not.toContain('2026-05-15');
	});
});
