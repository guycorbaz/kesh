// Story 14-4 — tests Vitest pour la page « Soldes de départ ».
//
// Couvre AC-E/AC-H côté frontend :
// - grille visible SEULEMENT si `canEnter` (statut READY) ;
// - état verrouillé + message selon chacune des 4 `reason` ;
// - état de chargement puis échec du fetch statut → message `status-error` +
//   bouton Réessayer, PAS de grille (P3-BH3-2) ;
// - filtre de grille : comptes actifs + postables + Asset/Liability seulement ;
// - bouton Générer désactivé tant que non équilibré, actif une fois équilibré ;
// - submit appelle `generateOpeningBalances` avec les lignes non vides
//   uniquement ;
// - succès → rechargement du statut → état verrouillé in-place (P1-M2-BH) ;
// - Consultation (statut 403) → état d'erreur, pas de grille (403 backend =
//   filet, l'entrée de menu est masquée par le layout).
//
// Mocks (hoistés AVANT l'import du composant) : API opening-balances +
// accounts + i18n (fallback) + notify.

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/svelte';
import type { AccountResponse } from '$lib/features/accounts/accounts.types';
import type { OpeningBalancesStatus } from '$lib/features/opening-balances/opening-balances.types';

vi.mock('$app/environment', () => ({ browser: true }));

// i18nMsg renvoie le fallback (déterministe, couvre la copie fallback svelte),
// variables interpolées — les messages de la Story 25-7 nomment des comptes et
// des montants, que les tests vérifient.
vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string, args?: Record<string, unknown>) =>
		fallback.replace(/\{ \$(\w+) \}/g, (_m: string, k: string) => String(args?.[k] ?? '')),
}));

const notifySuccessMock = vi.fn();
vi.mock('$lib/shared/utils/notify', () => ({
	notifySuccess: (...args: unknown[]) => notifySuccessMock(...args),
	notifyError: vi.fn(),
}));

const getStatusMock = vi.fn<() => Promise<OpeningBalancesStatus>>();
const generateMock = vi.fn();
const completeMock = vi.fn();
vi.mock('$lib/features/opening-balances/opening-balances.api', () => ({
	getOpeningBalancesStatus: () => getStatusMock(),
	generateOpeningBalances: (req: unknown) => generateMock(req),
	completeOpeningBalances: (req: unknown) => completeMock(req),
}));

const fetchAccountsMock = vi.fn<() => Promise<AccountResponse[]>>();
vi.mock('$lib/features/accounts/accounts.api', () => ({
	fetchAccounts: () => fetchAccountsMock(),
}));

import Page from './+page.svelte';

function acc(overrides: Partial<AccountResponse>): AccountResponse {
	return {
		id: 1,
		companyId: 1,
		number: '1000',
		name: 'Banque',
		accountType: 'Asset',
		parentId: null,
		active: true,
		role: null,
		postable: true,
		version: 1,
		createdAt: '2026-01-01T00:00:00',
		updatedAt: '2026-01-01T00:00:00',
		...overrides,
	};
}

const ASSET = acc({ id: 1, number: '1000', name: 'Banque', accountType: 'Asset' });
const RETAINED = acc({
	id: 2,
	number: '2970',
	name: 'Report à nouveau',
	accountType: 'Liability',
	role: 'RetainedEarnings',
});
const LIABILITY = acc({ id: 6, number: '2000', name: 'Dettes', accountType: 'Liability' });
const REVENUE = acc({ id: 3, number: '3000', name: 'Ventes', accountType: 'Revenue' });
const NON_POSTABLE = acc({
	id: 4,
	number: '2979',
	name: 'Résultat',
	accountType: 'Liability',
	postable: false,
});
const ARCHIVED = acc({ id: 5, number: '1090', name: 'Ancien', accountType: 'Asset', active: false });

/**
 * Champs du mode « compléter » (Story 25-7) pour les statuts qui ne
 * l'exercent pas. `NO_ENTRIES` est la raison d'une société vierge.
 */
const NO_COMPLEMENT = {
	canComplete: false,
	completeReason: 'NO_ENTRIES',
	completableAccounts: [],
	complementDate: null,
	complementDateKind: null,
	complementFiscalYear: null,
	retainedEarningsAccount: null,
} as const satisfies Partial<OpeningBalancesStatus>;

function readyStatus(): OpeningBalancesStatus {
	return {
		fiscalYear: { id: 12, name: 'Exercice 2026', startDate: '2026-01-01', status: 'Open' },
		canEnter: true,
		reason: 'READY',
	...NO_COMPLEMENT,
	};
}

beforeEach(() => {
	getStatusMock.mockReset();
	generateMock.mockReset();
	completeMock.mockReset();
	fetchAccountsMock.mockReset();
	notifySuccessMock.mockReset();
	fetchAccountsMock.mockResolvedValue([
		ASSET,
		RETAINED,
		LIABILITY,
		REVENUE,
		NON_POSTABLE,
		ARCHIVED,
	]);
});

describe('états chargement / erreur (P3-BH3-2)', () => {
	it('affiche le chargement tant que le statut est en vol, sans grille', async () => {
		// Promesse jamais résolue pendant le test.
		getStatusMock.mockReturnValue(new Promise(() => {}));
		fetchAccountsMock.mockReturnValue(new Promise(() => {}));

		render(Page);

		expect(await screen.findByTestId('opening-balances-loading')).toBeTruthy();
		expect(screen.queryByTestId('opening-balances-grid')).toBeNull();
	});

	it('échec du fetch statut → message status-error + Réessayer, PAS de grille', async () => {
		getStatusMock.mockRejectedValue({ code: 'NETWORK_ERROR', message: 'boom' });

		render(Page);

		expect(await screen.findByTestId('opening-balances-status-error')).toBeTruthy();
		expect(screen.getByTestId('opening-balances-retry')).toBeTruthy();
		expect(screen.queryByTestId('opening-balances-grid')).toBeNull();
	});

	it('Réessayer relance le chargement et affiche la grille en cas de succès', async () => {
		getStatusMock.mockRejectedValueOnce({ code: 'NETWORK_ERROR', message: 'boom' });
		getStatusMock.mockResolvedValue(readyStatus());

		render(Page);

		const retry = await screen.findByTestId('opening-balances-retry');
		await fireEvent.click(retry);

		expect(await screen.findByTestId('opening-balances-grid')).toBeTruthy();
	});

	it('double-clic sur Réessayer → UN SEUL load() (bouton désactivé pendant le chargement, Pass 3 ECH3-1)', async () => {
		// Mount : échec immédiat → écran d'erreur avec Réessayer.
		getStatusMock.mockRejectedValueOnce({ code: 'NETWORK_ERROR', message: 'boom', status: 0 });
		getStatusMock.mockResolvedValue(readyStatus());

		render(Page);
		const retry = await screen.findByTestId('opening-balances-retry');
		// Double-clic rapide (pas d'await entre les deux) : `loading = true` est
		// posé synchroniquement par le 1er load() et Svelte 5 flushe le
		// re-render AVANT le 2e clic — la branche d'erreur (et son bouton) est
		// DÉMONTÉE, le 2e clic tombe dans le vide (Pass 4 BH4-LOW : c'est le
		// démontage qui protège, pas un attribut disabled). La course
		// last-writer-wins est fermée à la source ; le jeton `loadGen` du
		// composant reste la défense en profondeur.
		fireEvent.click(retry);
		fireEvent.click(retry);

		expect(await screen.findByTestId('opening-balances-grid')).toBeTruthy();
		// mount (échec) + UN retry — pas deux.
		expect(getStatusMock).toHaveBeenCalledTimes(2);
		expect(screen.queryByTestId('opening-balances-status-error')).toBeNull();
	});

	it('Consultation (403 backend) → état erreur, pas de grille', async () => {
		getStatusMock.mockRejectedValue({ code: 'FORBIDDEN', message: 'Accès refusé' });

		render(Page);

		expect(await screen.findByTestId('opening-balances-status-error')).toBeTruthy();
		expect(screen.queryByTestId('opening-balances-grid')).toBeNull();
		expect(screen.queryByTestId('opening-balances-generate')).toBeNull();
	});
});

describe('état verrouillé — les 4 reasons (D6)', () => {
	it('NO_FISCAL_YEAR → verrou avec message, pas de grille', async () => {
		getStatusMock.mockResolvedValue({ fiscalYear: null, canEnter: false, reason: 'NO_FISCAL_YEAR', ...NO_COMPLEMENT });

		render(Page);

		const locked = await screen.findByTestId('opening-balances-locked');
		expect(locked.getAttribute('data-reason')).toBe('NO_FISCAL_YEAR');
		expect(locked.textContent).toContain('Aucun exercice comptable');
		expect(screen.queryByTestId('opening-balances-grid')).toBeNull();
	});

	it('FIRST_YEAR_CLOSED → verrou avec message, pas de grille', async () => {
		getStatusMock.mockResolvedValue({
			fiscalYear: { id: 12, name: 'Exercice 2026', startDate: '2026-01-01', status: 'Closed' },
			canEnter: false,
			reason: 'FIRST_YEAR_CLOSED',
		...NO_COMPLEMENT,
		});

		render(Page);

		const locked = await screen.findByTestId('opening-balances-locked');
		expect(locked.getAttribute('data-reason')).toBe('FIRST_YEAR_CLOSED');
		expect(locked.textContent).toContain('clôturé');
		expect(screen.queryByTestId('opening-balances-grid')).toBeNull();
	});

	it('ALREADY_HAS_ENTRIES → verrou + liens journal et bilan', async () => {
		getStatusMock.mockResolvedValue({
			fiscalYear: { id: 12, name: 'Exercice 2026', startDate: '2026-01-01', status: 'Open' },
			canEnter: false,
			reason: 'ALREADY_HAS_ENTRIES',
		...NO_COMPLEMENT,
		});

		render(Page);

		const locked = await screen.findByTestId('opening-balances-locked');
		expect(locked.getAttribute('data-reason')).toBe('ALREADY_HAS_ENTRIES');
		expect(screen.getByTestId('opening-balances-goto-journal')).toBeTruthy();
		expect(screen.getByTestId('opening-balances-goto-balance-sheet')).toBeTruthy();
		expect(screen.queryByTestId('opening-balances-grid')).toBeNull();
	});

	it('READY → grille visible, pas de verrou', async () => {
		getStatusMock.mockResolvedValue(readyStatus());

		render(Page);

		expect(await screen.findByTestId('opening-balances-grid')).toBeTruthy();
		expect(screen.queryByTestId('opening-balances-locked')).toBeNull();
	});
});

describe('grille — filtre des comptes (D4)', () => {
	it('liste seulement les comptes actifs + postables de type Asset/Liability', async () => {
		getStatusMock.mockResolvedValue(readyStatus());

		render(Page);

		await screen.findByTestId('opening-balances-grid');
		expect(screen.getByTestId('opening-balances-row-1000')).toBeTruthy();
		expect(screen.getByTestId('opening-balances-row-2970')).toBeTruthy();
		// Revenue exclu (fausserait le P&L), non-postable exclu, archivé exclu.
		expect(screen.queryByTestId('opening-balances-row-3000')).toBeNull();
		expect(screen.queryByTestId('opening-balances-row-2979')).toBeNull();
		expect(screen.queryByTestId('opening-balances-row-1090')).toBeNull();
	});

	it('0 compte éligible → message empty-grid explicite, pas de table ni de bouton (Pass 1 ECH-LOW)', async () => {
		getStatusMock.mockResolvedValue(readyStatus());
		// Plan atypique : seulement des comptes inéligibles (Revenue, non-postable, archivé).
		fetchAccountsMock.mockResolvedValue([REVENUE, NON_POSTABLE, ARCHIVED]);

		render(Page);

		expect(await screen.findByTestId('opening-balances-empty-grid')).toBeTruthy();
		expect(screen.queryByTestId('opening-balances-grid')).toBeNull();
		expect(screen.queryByTestId('opening-balances-generate')).toBeNull();
	});

	it('affiche le badge de rôle quand le compte en a un', async () => {
		getStatusMock.mockResolvedValue(readyStatus());

		render(Page);

		await screen.findByTestId('opening-balances-grid');
		expect(screen.getByTestId('opening-balances-row-2970-role-badge')).toBeTruthy();
		expect(screen.queryByTestId('opening-balances-row-1000-role-badge')).toBeNull();
	});
});

describe('équilibre et génération (D3)', () => {
	async function renderReadyGrid() {
		getStatusMock.mockResolvedValue(readyStatus());
		render(Page);
		await screen.findByTestId('opening-balances-grid');
	}

	it('bouton Générer désactivé tant que non équilibré', async () => {
		await renderReadyGrid();

		const btn = screen.getByTestId('opening-balances-generate') as HTMLButtonElement;
		expect(btn.disabled).toBe(true);

		// Saisie déséquilibrée : débit seul.
		await fireEvent.input(screen.getByTestId('opening-balances-debit-1000'), {
			target: { value: '100' },
		});
		expect((screen.getByTestId('opening-balances-generate') as HTMLButtonElement).disabled).toBe(
			true
		);
	});

	it('bouton actif une fois équilibré, submit envoie SEULEMENT les lignes non vides', async () => {
		generateMock.mockResolvedValue({ id: 1 });
		await renderReadyGrid();

		await fireEvent.input(screen.getByTestId('opening-balances-debit-1000'), {
			target: { value: '100' },
		});
		await fireEvent.input(screen.getByTestId('opening-balances-credit-2970'), {
			target: { value: '100' },
		});

		const btn = screen.getByTestId('opening-balances-generate') as HTMLButtonElement;
		await waitFor(() => expect(btn.disabled).toBe(false));

		await fireEvent.click(btn);

		await waitFor(() => expect(generateMock).toHaveBeenCalledTimes(1));
		expect(generateMock).toHaveBeenCalledWith({
			lines: [
				{ accountId: 1, debit: '100', credit: '0' },
				{ accountId: 2, debit: '0', credit: '100' },
			],
		});
	});

	it('une ligne « 0 » explicite est traitée comme vide : non envoyée au POST (Pass 2 ECH2-2)', async () => {
		generateMock.mockResolvedValue({ id: 1 });
		await renderReadyGrid();

		await fireEvent.input(screen.getByTestId('opening-balances-debit-1000'), {
			target: { value: '100' },
		});
		await fireEvent.input(screen.getByTestId('opening-balances-credit-2970'), {
			target: { value: '100' },
		});
		// « 0 » tapé explicitement dans une ligne inutilisée — le serveur
		// rejetterait cette ligne (EntryLineDebitCreditExclusive) si envoyée.
		await fireEvent.input(screen.getByTestId('opening-balances-debit-2000'), {
			target: { value: '0' },
		});

		const btn = screen.getByTestId('opening-balances-generate') as HTMLButtonElement;
		await waitFor(() => expect(btn.disabled).toBe(false));
		await fireEvent.click(btn);

		await waitFor(() => expect(generateMock).toHaveBeenCalledTimes(1));
		// SEULES les 2 lignes à montant > 0 partent — la ligne « 0 » est filtrée.
		expect(generateMock).toHaveBeenCalledWith({
			lines: [
				{ accountId: 1, debit: '100', credit: '0' },
				{ accountId: 2, debit: '0', credit: '100' },
			],
		});
	});

	it('saisir un débit vide le crédit de la même ligne (exclusivité)', async () => {
		await renderReadyGrid();

		const debit = screen.getByTestId('opening-balances-debit-1000') as HTMLInputElement;
		const credit = screen.getByTestId('opening-balances-credit-1000') as HTMLInputElement;

		await fireEvent.input(credit, { target: { value: '50' } });
		await fireEvent.input(debit, { target: { value: '100' } });

		await waitFor(() => expect(credit.value).toBe(''));
		expect(debit.value).toBe('100');
	});

	it('succès → toast + rechargement statut → verrou ALREADY_HAS_ENTRIES in-place (P1-M2-BH)', async () => {
		generateMock.mockResolvedValue({ id: 1 });
		getStatusMock.mockResolvedValueOnce(readyStatus());
		getStatusMock.mockResolvedValue({
			fiscalYear: { id: 12, name: 'Exercice 2026', startDate: '2026-01-01', status: 'Open' },
			canEnter: false,
			reason: 'ALREADY_HAS_ENTRIES',
		...NO_COMPLEMENT,
		});

		render(Page);
		await screen.findByTestId('opening-balances-grid');

		await fireEvent.input(screen.getByTestId('opening-balances-debit-1000'), {
			target: { value: '100' },
		});
		await fireEvent.input(screen.getByTestId('opening-balances-credit-2970'), {
			target: { value: '100' },
		});
		const btn = screen.getByTestId('opening-balances-generate') as HTMLButtonElement;
		await waitFor(() => expect(btn.disabled).toBe(false));
		await fireEvent.click(btn);

		// Verrou in-place : pas de redirection, liens bilan + journal proposés.
		const locked = await screen.findByTestId('opening-balances-locked');
		expect(locked.getAttribute('data-reason')).toBe('ALREADY_HAS_ENTRIES');
		expect(notifySuccessMock).toHaveBeenCalledTimes(1);
		expect(screen.getByTestId('opening-balances-goto-balance-sheet')).toBeTruthy();
		expect(screen.queryByTestId('opening-balances-grid')).toBeNull();
	});

	it('échec 409 (course perdue) → rechargement du statut → verrou in-place (Pass 3 BH3-LOW)', async () => {
		generateMock.mockRejectedValue({
			code: 'ILLEGAL_STATE_TRANSITION',
			message: 'La société contient déjà des écritures.',
			status: 409,
		});
		getStatusMock.mockResolvedValueOnce(readyStatus());
		getStatusMock.mockResolvedValue({
			fiscalYear: { id: 12, name: 'Exercice 2026', startDate: '2026-01-01', status: 'Open' },
			canEnter: false,
			reason: 'ALREADY_HAS_ENTRIES',
		...NO_COMPLEMENT,
		});

		render(Page);
		await screen.findByTestId('opening-balances-grid');

		await fireEvent.input(screen.getByTestId('opening-balances-debit-1000'), {
			target: { value: '100' },
		});
		await fireEvent.input(screen.getByTestId('opening-balances-credit-2970'), {
			target: { value: '100' },
		});
		const btn = screen.getByTestId('opening-balances-generate') as HTMLButtonElement;
		await waitFor(() => expect(btn.disabled).toBe(false));
		await fireEvent.click(btn);

		// L'écran se verrouille in-place comme le chemin succès — la grille ne
		// reste pas active à rejouer un POST voué au même 409.
		const locked = await screen.findByTestId('opening-balances-locked');
		expect(locked.getAttribute('data-reason')).toBe('ALREADY_HAS_ENTRIES');
		expect(screen.queryByTestId('opening-balances-grid')).toBeNull();
	});

	it('échec serveur → err.message affiché inline tel quel (AC-E)', async () => {
		generateMock.mockRejectedValue({
			code: 'ILLEGAL_STATE_TRANSITION',
			message: 'La société contient déjà des écritures.',
			status: 409,
		});
		await renderReadyGrid();

		await fireEvent.input(screen.getByTestId('opening-balances-debit-1000'), {
			target: { value: '100' },
		});
		await fireEvent.input(screen.getByTestId('opening-balances-credit-2970'), {
			target: { value: '100' },
		});
		const btn = screen.getByTestId('opening-balances-generate') as HTMLButtonElement;
		await waitFor(() => expect(btn.disabled).toBe(false));
		await fireEvent.click(btn);

		const err = await screen.findByTestId('opening-balances-submit-error');
		expect(err.textContent).toContain('La société contient déjà des écritures.');
	});
});

// ---------------------------------------------------------------------------
// Story 25-7 (#445) — avertissement, totaux, complément
// ---------------------------------------------------------------------------

async function type(testId: string, value: string) {
	await fireEvent.input(screen.getByTestId(testId), { target: { value } });
}

describe('AC 1 — avertissement du report à-nouveau', () => {
	it('saisie équilibrée sans rien sur 2970 → avertissement NO_AMOUNT qui nomme le compte, Générer reste actif', async () => {
		getStatusMock.mockResolvedValue(readyStatus());
		render(Page);
		await screen.findByTestId('opening-balances-grid');

		await type('opening-balances-debit-1000', '100');
		await type('opening-balances-credit-2000', '100');

		const warning = await screen.findByTestId('opening-balances-no-retained-earnings');
		expect(warning.getAttribute('data-kind')).toBe('NO_AMOUNT');
		expect(warning.textContent).toContain('2970');
		const btn = screen.getByTestId('opening-balances-generate') as HTMLButtonElement;
		await waitFor(() => expect(btn.disabled).toBe(false));
	});

	it('montant porté sur 2970 → pas d’avertissement', async () => {
		getStatusMock.mockResolvedValue(readyStatus());
		render(Page);
		await screen.findByTestId('opening-balances-grid');

		await type('opening-balances-debit-1000', '100');
		await type('opening-balances-credit-2970', '100');

		await waitFor(() =>
			expect(
				(screen.getByTestId('opening-balances-generate') as HTMLButtonElement).disabled
			).toBe(false)
		);
		expect(screen.queryByTestId('opening-balances-no-retained-earnings')).toBeNull();
	});

	it('saisie non équilibrée → pas d’avertissement', async () => {
		getStatusMock.mockResolvedValue(readyStatus());
		render(Page);
		await screen.findByTestId('opening-balances-grid');
		await type('opening-balances-debit-1000', '100');
		expect(screen.queryByTestId('opening-balances-no-retained-earnings')).toBeNull();
	});

	it('aucun compte ne porte le rôle → variante NO_ROLE', async () => {
		fetchAccountsMock.mockResolvedValue([ASSET, LIABILITY]);
		getStatusMock.mockResolvedValue(readyStatus());
		render(Page);
		await screen.findByTestId('opening-balances-grid');
		await type('opening-balances-debit-1000', '100');
		await type('opening-balances-credit-2000', '100');
		const warning = await screen.findByTestId('opening-balances-no-retained-earnings');
		expect(warning.getAttribute('data-kind')).toBe('NO_ROLE');
	});

	it('compte de report non imputable → variante NOT_POSTABLE', async () => {
		fetchAccountsMock.mockResolvedValue([ASSET, LIABILITY, { ...RETAINED, postable: false }]);
		getStatusMock.mockResolvedValue(readyStatus());
		render(Page);
		await screen.findByTestId('opening-balances-grid');
		expect(screen.queryByTestId('opening-balances-debit-2970')).toBeNull();
		await type('opening-balances-debit-1000', '100');
		await type('opening-balances-credit-2000', '100');
		const warning = await screen.findByTestId('opening-balances-no-retained-earnings');
		expect(warning.getAttribute('data-kind')).toBe('NOT_POSTABLE');
		expect(warning.textContent).toContain('2970');
	});
});

describe('AC 2 — totaux actif / passif et montant à porter', () => {
	it('montant à porter = actifs − passifs, au crédit du report', async () => {
		getStatusMock.mockResolvedValue(readyStatus());
		render(Page);
		await screen.findByTestId('opening-balances-grid');

		await type('opening-balances-debit-1000', '1000');
		await type('opening-balances-credit-2000', '600');

		expect(screen.getByTestId('opening-balances-total-assets').textContent).toBe('1’000.00');
		expect(screen.getByTestId('opening-balances-total-liabilities').textContent).toBe('600.00');
		const carry = screen.getByTestId('opening-balances-amount-to-carry');
		expect(carry.getAttribute('data-side')).toBe('credit');
		expect(carry.textContent).toContain('400.00');

		await type('opening-balances-credit-2970', '400');
		expect(screen.getByTestId('opening-balances-retained-entered').textContent).toBe('400.00');
		expect(screen.getByTestId('opening-balances-remaining-gap').textContent).toBe('0.00');
	});

	it('épingle la limite : équilibrée sans report, le montant à porter vaut 0', async () => {
		getStatusMock.mockResolvedValue(readyStatus());
		render(Page);
		await screen.findByTestId('opening-balances-grid');
		await type('opening-balances-debit-1000', '1000');
		await type('opening-balances-credit-2000', '1000');
		const carry = screen.getByTestId('opening-balances-amount-to-carry');
		expect(carry.getAttribute('data-side')).toBe('none');
		expect(carry.textContent).toContain('0.00');
	});
});

function completableStatus(overrides: Partial<OpeningBalancesStatus> = {}): OpeningBalancesStatus {
	return {
		fiscalYear: { id: 12, name: 'Exercice 2026', startDate: '2026-01-01', status: 'Open' },
		canEnter: false,
		reason: 'ALREADY_HAS_ENTRIES',
		canComplete: true,
		completeReason: 'READY',
		completableAccounts: [
			{ id: 7, number: '1100', name: 'Poste', accountType: 'Asset' },
			{ id: 6, number: '2000', name: 'Dettes', accountType: 'Liability' },
		],
		complementDate: '2026-01-01',
		complementDateKind: 'OPENING_DAY',
		complementFiscalYear: { id: 12, name: 'Exercice 2026' },
		retainedEarningsAccount: { id: 2, number: '2970', name: 'Report à nouveau' },
		...overrides,
	};
}

describe('AC 6 — le mode « compléter »', () => {
	it('sous ALREADY_HAS_ENTRIES : le bandeau reste, la grille de complément ne liste que les comptes du status', async () => {
		getStatusMock.mockResolvedValue(completableStatus());
		render(Page);
		await screen.findByTestId('opening-balances-complete');
		expect(screen.getByTestId('opening-balances-locked').getAttribute('data-reason')).toBe(
			'ALREADY_HAS_ENTRIES'
		);
		expect(screen.getByTestId('opening-balances-complete-debit-1100')).toBeTruthy();
		expect(screen.getByTestId('opening-balances-complete-credit-2000')).toBeTruthy();
		// 1000 (mouvementé) n'est pas proposé ; la grille d'ouverture est absente.
		expect(screen.queryByTestId('opening-balances-complete-debit-1000')).toBeNull();
		expect(screen.queryByTestId('opening-balances-grid')).toBeNull();
		expect(screen.queryByTestId('opening-balances-debit-1100')).toBeNull();
	});

	it('contrepartie en direct : actif → crédit du report, passif → débit, équilibre → aucune', async () => {
		getStatusMock.mockResolvedValue(completableStatus());
		render(Page);
		await screen.findByTestId('opening-balances-complete');
		const cp = () => screen.getByTestId('opening-balances-complete-counterpart');

		await type('opening-balances-complete-debit-1100', '250');
		await waitFor(() => expect(cp().getAttribute('data-side')).toBe('credit'));
		expect(cp().textContent).toContain('250.00');
		expect(cp().textContent).toContain('2970');

		await type('opening-balances-complete-credit-2000', '300');
		await waitFor(() => expect(cp().getAttribute('data-side')).toBe('debit'));
		expect(cp().textContent).toContain('50.00');

		await type('opening-balances-complete-credit-2000', '250');
		await waitFor(() => expect(cp().getAttribute('data-side')).toBe('none'));
		expect(cp().textContent).toContain('aucune contrepartie');
	});

	it('Compléter envoie les seules lignes saisies, puis recharge le status', async () => {
		vi.spyOn(window, 'confirm').mockReturnValue(true);
		completeMock.mockResolvedValue({ id: 9 });
		getStatusMock.mockResolvedValueOnce(completableStatus());
		getStatusMock.mockResolvedValue(
			completableStatus({
				completableAccounts: [{ id: 6, number: '2000', name: 'Dettes', accountType: 'Liability' }],
			})
		);
		render(Page);
		await screen.findByTestId('opening-balances-complete');

		await type('opening-balances-complete-debit-1100', '250,5');
		const btn = screen.getByTestId('opening-balances-complete-submit') as HTMLButtonElement;
		await waitFor(() => expect(btn.disabled).toBe(false));
		await fireEvent.click(btn);

		await waitFor(() =>
			expect(completeMock).toHaveBeenCalledWith({
				lines: [{ accountId: 7, debit: '250.5', credit: '0' }],
			})
		);
		await waitFor(() =>
			expect(screen.queryByTestId('opening-balances-complete-debit-1100')).toBeNull()
		);
		expect(notifySuccessMock).toHaveBeenCalledTimes(1);
	});

	it('Compléter désactivé tant que rien n’est saisi', async () => {
		getStatusMock.mockResolvedValue(completableStatus());
		render(Page);
		await screen.findByTestId('opening-balances-complete');
		expect((screen.getByTestId('opening-balances-complete-submit') as HTMLButtonElement).disabled).toBe(true);
	});

	it('canComplete faux → texte qui dit pourquoi, avec la raison', async () => {
		getStatusMock.mockResolvedValue(
			completableStatus({ canComplete: false, completeReason: 'NO_COMPLETABLE_ACCOUNT', completableAccounts: [] })
		);
		render(Page);
		const unavailable = await screen.findByTestId('opening-balances-complete-unavailable');
		expect(unavailable.getAttribute('data-reason')).toBe('NO_COMPLETABLE_ACCOUNT');
		expect(unavailable.textContent).toContain('aucun compte à compléter');
		expect(screen.queryByTestId('opening-balances-complete')).toBeNull();
	});

	it('sous NO_FISCAL_YEAR / FIRST_YEAR_CLOSED : aucune section de complément', async () => {
		getStatusMock.mockResolvedValue({ fiscalYear: null, canEnter: false, reason: 'NO_FISCAL_YEAR', ...NO_COMPLEMENT });
		render(Page);
		await screen.findByTestId('opening-balances-locked');
		expect(screen.queryByTestId('opening-balances-complete')).toBeNull();
		expect(screen.queryByTestId('opening-balances-complete-unavailable')).toBeNull();
	});

	it('le bandeau ne propose plus de supprimer toutes les écritures', async () => {
		getStatusMock.mockResolvedValue(completableStatus());
		render(Page);
		const locked = await screen.findByTestId('opening-balances-locked');
		expect(locked.textContent).not.toContain('supprimez');
		expect(locked.textContent).toContain('ci-dessous');
	});
});
