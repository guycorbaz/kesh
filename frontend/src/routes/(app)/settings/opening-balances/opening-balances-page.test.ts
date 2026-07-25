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

// i18nMsg renvoie le fallback (déterministe, couvre la copie fallback svelte).
vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string) => fallback,
}));

const notifySuccessMock = vi.fn();
vi.mock('$lib/shared/utils/notify', () => ({
	notifySuccess: (...args: unknown[]) => notifySuccessMock(...args),
	notifyError: vi.fn(),
}));

const getStatusMock = vi.fn<() => Promise<OpeningBalancesStatus>>();
const generateMock = vi.fn();
vi.mock('$lib/features/opening-balances/opening-balances.api', () => ({
	getOpeningBalancesStatus: () => getStatusMock(),
	generateOpeningBalances: (req: unknown) => generateMock(req),
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

function readyStatus(): OpeningBalancesStatus {
	return {
		fiscalYear: { id: 12, name: 'Exercice 2026', startDate: '2026-01-01', status: 'Open' },
		canEnter: true,
		reason: 'READY',
	};
}

beforeEach(() => {
	getStatusMock.mockReset();
	generateMock.mockReset();
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
		// posé synchroniquement par le 1er load() et `disabled={loading}` avale
		// le 2e clic — la course last-writer-wins (un load périmé qui écraserait
		// l'état d'un load plus récent) est fermée à la source ; le jeton
		// `loadGen` du composant reste la défense en profondeur.
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
		getStatusMock.mockResolvedValue({ fiscalYear: null, canEnter: false, reason: 'NO_FISCAL_YEAR' });

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
