// Story 37-2 (AC-6) — écran Cours de change : mentions et ↑ par cas, valeur égale →
// reset (jamais PUT), version et valeur affichée envoyées, 409, genres et dialogue
// U4, condition U3 à false et à true, Consultation, exercice clos, JPY par unité,
// « Récupérer » qui poursuit après un échec avec un seul POST par colonne, en-têtes
// des exercices (1er janvier, 1er juillet, 1er avril, 15.03, six et dix-huit mois)
// et liaisons des colonnes.
//
// ⛔ `isApiError` n'est pas mocké : une erreur simulée porte `code` ET `status`.

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/svelte';
import { pinClock } from '$lib/shared/utils/pin-clock.test-helper';
import type { FiscalYearResponse } from '$lib/features/fiscal-years/fiscal-years.types';
import { afcRow, fy, published, row, yearGrid } from './exchange-rates.test-fixtures';
import type { Cell, FiscalYearRatesResponse } from './exchange-rates.types';

const { auth, api, settings, fys } = vi.hoisted(() => ({
	auth: { currentUser: { role: 'Admin' } as { role: string } | null },
	api: {
		fetchRates: vi.fn(),
		getFetchStatus: vi.fn(),
		getYearGrid: vi.fn(),
		putRate: vi.fn(),
		putRateKinds: vi.fn(),
		resetRate: vi.fn(),
	},
	settings: { fetchCompanyCurrent: vi.fn() },
	fys: { listFiscalYears: vi.fn() },
}));

vi.mock('$lib/app/stores/auth.svelte', () => ({ authState: auth }));
vi.mock('./exchange-rates.api', () => api);
vi.mock('$lib/features/settings/settings.api', () => settings);
vi.mock('$lib/features/fiscal-years/fiscal-years.api', () => fys);

import ExchangeRatesScreen from './ExchangeRatesScreen.svelte';

// Aujourd'hui : 10.03.2025 à Zurich.
pinClock('2025-03-10T10:00:00Z');

const STATUS_NONE = { trigger: null, lastRunAt: null, status: null, error: null, kind: null, rateDate: null };

function company(multiCurrencyEnabled = true) {
	return { company: { name: 'Test SA', multiCurrencyEnabled }, bankAccounts: [] };
}

async function monter(
	grid: FiscalYearRatesResponse,
	opts: { fiscalYears?: FiscalYearResponse[]; canManageSeveralCompanies?: boolean } = {},
) {
	settings.fetchCompanyCurrent.mockResolvedValue(company());
	fys.listFiscalYears.mockResolvedValue(opts.fiscalYears ?? [fy()]);
	api.getYearGrid.mockResolvedValue(grid);
	api.getFetchStatus.mockResolvedValue(STATUS_NONE);
	const r = render(ExchangeRatesScreen, {
		canManageSeveralCompanies: opts.canManageSeveralCompanies ?? false,
	});
	await screen.findByTestId('exchange-rates-table');
	return r;
}

/** Saisit une valeur dans un champ puis le quitte (validation du champ). */
async function saisir(testKey: string, value: string) {
	const input = screen.getByTestId(`exchange-rates-input-${testKey}`) as HTMLInputElement;
	await fireEvent.input(input, { target: { value } });
	await fireEvent.blur(input);
}

/** Un exercice au 1er juillet 2024, genres `day`, une ligne USD aux cases variées. */
function gridVarie(): FiscalYearRatesResponse {
	return yearGrid('2024-07-01', '2025-06-30', 'day', 'day', (g) => [
		row(g, 'USD', {
			'month:2024-07-01': { effectiveRate: '0.8846000000', source: 'afc', row: afcRow(1), published: published('0.8846') },
			'month:2024-08-01': {
				effectiveRate: '0.8846000000',
				source: 'afc',
				row: afcRow(4),
				published: published('0.8850000000'),
				corrected: true,
			},
			'month:2024-09-01': { effectiveRate: '0.8846', source: 'afc', row: afcRow(1) },
			'month:2024-10-01': {
				effectiveRate: '0.9000000000',
				source: 'manual',
				row: { ...afcRow(2), updatedByKesh: false },
				published: published('0.8846000000'),
			},
			'month:2024-11-01': { effectiveRate: '0.9', source: 'manual', row: afcRow(1) },
			'month:2024-12-01': { effectiveRate: '0.8846', source: 'manual', row: afcRow(1), published: published('0.8846') },
			'month:2025-01-01': { effectiveRate: '0.9', source: 'ecb', row: afcRow(1), published: published('0.8846') },
			'month:2025-02-01': { published: published('0.8846') },
			opening: { published: published('0.9', { sourceDate: '2024-06-28' }), effectiveRate: '0.9', source: 'afc', row: afcRow(1) },
		}),
		row(g, 'JPY'),
	]);
}

const mention = (k: string) => screen.getByTestId(`exchange-rates-mention-${k}`);
const arrow = (k: string) => screen.queryByTestId(`exchange-rates-arrow-${k}`);

describe('ExchangeRatesScreen — mentions et ↑ (U1, A2, A30)', () => {
	beforeEach(() => {
		vi.resetAllMocks();
		auth.currentUser = { role: 'Admin' };
	});

	it('chaque cas a sa mention et son ↑', async () => {
		await monter(gridVarie());
		expect(mention('USD-month-2024-07-01').textContent).toBe('AFC ✓');
		expect(arrow('USD-month-2024-07-01')).toBeNull();

		const corrige = mention('USD-month-2024-08-01');
		expect(corrige.textContent).toBe('AFC corrigé : 0.8850 (le 03.12.2024)');
		expect(corrige.className).toContain('text-error');
		expect(arrow('USD-month-2024-08-01')?.getAttribute('aria-label')).toBe('Reprendre la valeur corrigée');

		expect(mention('USD-month-2024-09-01').textContent).toBe('AFC (valeur publiée non relue)');
		expect(arrow('USD-month-2024-09-01')).toBeNull();

		expect(mention('USD-month-2024-10-01').textContent).toBe('AFC : 0.8846');
		expect(arrow('USD-month-2024-10-01')?.getAttribute('aria-label')).toBe('Reprendre la valeur publiée');

		expect(mention('USD-month-2024-11-01').textContent).toBe('AFC : non publié');
		expect(arrow('USD-month-2024-11-01')?.getAttribute('aria-label')).toBe('Retirer le cours saisi');

		expect(mention('USD-month-2024-12-01').textContent).toBe('AFC ✓ · saisi');
		expect(arrow('USD-month-2024-12-01')).not.toBeNull();

		expect(mention('USD-month-2025-01-01').textContent).toBe('AFC : 0.8846 (ecb)');
		expect(arrow('USD-month-2025-01-01')?.getAttribute('aria-label')).toBe('Reprendre la valeur publiée');

		const vide = screen.getByTestId('exchange-rates-input-USD-month-2025-02-01') as HTMLInputElement;
		expect(vide.value).toBe('');
		expect(mention('USD-month-2025-02-01').textContent).toBe('AFC : 0.8846');
		expect(arrow('USD-month-2025-02-01')).not.toBeNull();

		expect(mention('USD-month-2025-03-01').textContent).toBe('AFC : non publié');
		expect(arrow('USD-month-2025-03-01')).toBeNull();
	});

	it('libellés « cours mensuel AFC » / « cours journalier », jamais « cours moyen »', async () => {
		await monter(gridVarie());
		const texte = screen.getByTestId('exchange-rates-screen').textContent ?? '';
		expect(texte).toContain('cours mensuel');
		expect(texte).toContain('cours journalier');
		expect(texte).not.toMatch(/cours moyen/i);
	});

	it('copie automatique et date de publication au survol / sous la case', async () => {
		await monter(gridVarie());
		expect(screen.getByTestId('exchange-rates-cell-USD-month-2024-07-01').getAttribute('title')).toContain(
			'Copié automatiquement par Kesh',
		);
		expect(screen.getByTestId('exchange-rates-cell-USD-month-2024-10-01').getAttribute('title')).toContain(
			'Retenu par un utilisateur',
		);
		expect(screen.getByTestId('exchange-rates-published-on-USD-opening-2024-06-30').textContent).toBe(
			'publié le 28.06.2024',
		);
		// Clôture au 30.06.2025 : jour non passé, raison au survol.
		expect(screen.getByTestId('exchange-rates-cell-USD-closing-2025-06-30').getAttribute('title')).toContain(
			'Jour non passé',
		);
	});
});

describe('ExchangeRatesScreen — écritures', () => {
	beforeEach(() => {
		vi.resetAllMocks();
		auth.currentUser = { role: 'Admin' };
	});

	it('↑ d’une correction (condition U3 à false) : reset direct, version et valeur affichée', async () => {
		await monter(gridVarie());
		api.resetRate.mockResolvedValue({});
		await fireEvent.click(arrow('USD-month-2024-08-01')!);
		await waitFor(() =>
			expect(api.resetRate).toHaveBeenCalledWith({
				currency: 'USD',
				kind: 'month',
				rateDate: '2024-08-01',
				version: 4,
				publishedRate: '0.8850000000',
			}),
		);
		expect(screen.queryByTestId('exchange-rates-correction-dialog')).toBeNull();
		await waitFor(() => expect(api.getYearGrid).toHaveBeenCalledTimes(2));
	});

	it('↑ d’une case vide à valeur publiée : reset sans version', async () => {
		await monter(gridVarie());
		api.resetRate.mockResolvedValue({});
		await fireEvent.click(arrow('USD-month-2025-02-01')!);
		await waitFor(() =>
			expect(api.resetRate).toHaveBeenCalledWith({
				currency: 'USD',
				kind: 'month',
				rateDate: '2025-02-01',
				publishedRate: '0.8846',
			}),
		);
	});

	it('valeur entrée égale à la valeur publiée → reset, jamais PUT', async () => {
		await monter(gridVarie());
		api.resetRate.mockResolvedValue({});
		await saisir('USD-month-2024-10-01', '0.8846');
		await waitFor(() => expect(api.resetRate).toHaveBeenCalledTimes(1));
		expect(api.resetRate.mock.calls[0][0]).toMatchObject({ version: 2, publishedRate: '0.8846000000' });
		expect(api.putRate).not.toHaveBeenCalled();
	});

	it('déjà afc à cette valeur → rien ; champ vidé → restauré, sans appel', async () => {
		await monter(gridVarie());
		await saisir('USD-month-2024-07-01', '0.8846');
		await saisir('USD-month-2024-10-01', '');
		expect(api.resetRate).not.toHaveBeenCalled();
		expect(api.putRate).not.toHaveBeenCalled();
		expect((screen.getByTestId('exchange-rates-input-USD-month-2024-10-01') as HTMLInputElement).value).toBe(
			'0.9000',
		);
	});

	it('autre valeur → PUT avec la version et la valeur saisie, puis relecture', async () => {
		await monter(gridVarie());
		api.putRate.mockResolvedValue({});
		await saisir('USD-month-2024-10-01', '0,9123');
		await waitFor(() =>
			expect(api.putRate).toHaveBeenCalledWith({
				currency: 'USD',
				kind: 'month',
				rateDate: '2024-10-01',
				rate: '0.9123',
				version: 2,
			}),
		);
		await waitFor(() => expect(api.getYearGrid).toHaveBeenCalledTimes(2));
	});

	it('cours JPY saisi par unité : 0.0056 envoyé tel quel, sans version', async () => {
		await monter(gridVarie());
		api.putRate.mockResolvedValue({});
		await saisir('JPY-month-2024-07-01', '0.0056');
		await waitFor(() =>
			expect(api.putRate).toHaveBeenCalledWith({
				currency: 'JPY',
				kind: 'month',
				rateDate: '2024-07-01',
				rate: '0.0056',
			}),
		);
	});

	it('saisie invalide : refusée à l’écran, sans appel', async () => {
		await monter(gridVarie());
		await saisir('JPY-month-2024-07-01', '0.12345678901');
		expect(api.putRate).not.toHaveBeenCalled();
		expect(screen.getByTestId('exchange-rates-message').textContent).toContain('Cours JPY invalide');
	});

	it('409 → rechargement avec un message', async () => {
		await monter(gridVarie());
		api.putRate.mockRejectedValue({ code: 'OPTIMISTIC_LOCK_CONFLICT', message: 'Conflit', status: 409 });
		await saisir('USD-month-2024-10-01', '0.95');
		await waitFor(() => expect(api.getYearGrid).toHaveBeenCalledTimes(2));
		expect((await screen.findByTestId('exchange-rates-message')).textContent).toContain('a changé entre-temps');
	});

	it('409 sur un ↑ → rechargement avec un message', async () => {
		await monter(gridVarie());
		api.resetRate.mockRejectedValue({ code: 'OPTIMISTIC_LOCK_CONFLICT', message: 'Conflit', status: 409 });
		await fireEvent.click(arrow('USD-month-2024-10-01')!);
		await waitFor(() => expect(api.getYearGrid).toHaveBeenCalledTimes(2));
		expect((await screen.findByTestId('exchange-rates-message')).textContent).toContain('a changé entre-temps');
	});

	it('FX_RATE_INVALID du serveur : son message est affiché', async () => {
		await monter(gridVarie());
		api.putRate.mockRejectedValue({ code: 'FX_RATE_INVALID', message: 'Cours USD invalide (serveur)', status: 400 });
		await saisir('USD-month-2024-10-01', '0.95');
		expect((await screen.findByTestId('exchange-rates-message')).textContent).toBe('Cours USD invalide (serveur)');
	});

	it('revue P1 (37-2-CR-P1-5) — PUT refusé (500) : la case revient à la valeur enregistrée', async () => {
		await monter(gridVarie());
		const avant = (screen.getByTestId('exchange-rates-input-USD-month-2024-10-01') as HTMLInputElement).value;
		api.putRate.mockRejectedValue({ code: 'INTERNAL_ERROR', message: 'Erreur interne', status: 500 });
		await saisir('USD-month-2024-10-01', '0.95');
		expect((await screen.findByTestId('exchange-rates-message')).textContent).toBe('Erreur interne');
		await waitFor(() =>
			expect((screen.getByTestId('exchange-rates-input-USD-month-2024-10-01') as HTMLInputElement).value).toBe(avant),
		);
		expect(avant).not.toBe('0.95');
	});

	it('revue P1 (37-2-CR-P1-6) — une lecture périmée du tableau est ignorée', async () => {
		await monter(gridVarie());
		let lente: (g: FiscalYearRatesResponse) => void = () => {};
		api.getYearGrid.mockImplementationOnce(() => new Promise((r) => (lente = r)));
		api.getYearGrid.mockResolvedValueOnce(gridVarie());
		await fireEvent.click(screen.getByTestId('exchange-rates-all'));
		await fireEvent.click(screen.getByTestId('exchange-rates-all'));
		await waitFor(() => expect(api.getYearGrid).toHaveBeenCalledTimes(3));
		const ancienne = gridVarie();
		ancienne.rows = [];
		lente(ancienne);
		await new Promise((r) => setTimeout(r, 0));
		expect(screen.getByTestId('exchange-rates-input-USD-month-2024-10-01')).toBeTruthy();
	});
});

describe('ExchangeRatesScreen — dialogue U3 (condition unique)', () => {
	beforeEach(() => {
		vi.resetAllMocks();
		auth.currentUser = { role: 'Admin' };
	});

	it('condition à true : ↑ d’une correction ouvre le dialogue, « Corriger » envoie reset', async () => {
		await monter(gridVarie(), { canManageSeveralCompanies: true });
		api.resetRate.mockResolvedValue({});
		await fireEvent.click(arrow('USD-month-2024-08-01')!);
		expect(await screen.findByTestId('exchange-rates-correction-dialog')).toBeTruthy();
		expect(api.resetRate).not.toHaveBeenCalled();
		await fireEvent.click(screen.getByTestId('exchange-rates-correction-confirm'));
		await waitFor(() =>
			expect(api.resetRate).toHaveBeenCalledWith({
				currency: 'USD',
				kind: 'month',
				rateDate: '2024-08-01',
				version: 4,
				publishedRate: '0.8850000000',
			}),
		);
	});

	it('condition à true : « Annuler » n’envoie rien', async () => {
		await monter(gridVarie(), { canManageSeveralCompanies: true });
		await fireEvent.click(arrow('USD-month-2024-08-01')!);
		await fireEvent.click(await screen.findByTestId('exchange-rates-correction-cancel'));
		await waitFor(() => expect(screen.queryByTestId('exchange-rates-correction-dialog')).toBeNull());
		expect(api.resetRate).not.toHaveBeenCalled();
	});

	it('condition à true : le ↑ d’un cours saisi n’ouvre pas le dialogue', async () => {
		await monter(gridVarie(), { canManageSeveralCompanies: true });
		api.resetRate.mockResolvedValue({});
		await fireEvent.click(arrow('USD-month-2024-10-01')!);
		await waitFor(() => expect(api.resetRate).toHaveBeenCalledTimes(1));
		expect(screen.queryByTestId('exchange-rates-correction-dialog')).toBeNull();
	});
});

describe('ExchangeRatesScreen — genres d’ouverture et de clôture (R6, U4)', () => {
	beforeEach(() => {
		vi.resetAllMocks();
		auth.currentUser = { role: 'Admin' };
		api.putRateKinds.mockResolvedValue({ fiscalYear: fy(), nextFiscalYear: null });
	});

	const courant = fy({ id: 1, name: '2024/25', startDate: '2024-07-01', endDate: '2025-06-30' });
	const suivant = fy({ id: 2, name: '2025/26', startDate: '2025-07-01', endDate: '2026-06-30' });

	it('changer le genre d’ouverture → PUT /rate-kinds puis relecture', async () => {
		await monter(gridVarie(), { fiscalYears: [courant] });
		await fireEvent.click(screen.getByTestId('exchange-rates-opening-month'));
		await waitFor(() =>
			expect(api.putRateKinds).toHaveBeenCalledWith(1, {
				openingRateKind: 'month',
				closingRateKind: 'day',
				applyToNextOpening: false,
			}),
		);
		await waitFor(() => expect(api.getYearGrid).toHaveBeenCalledTimes(2));
		expect(fys.listFiscalYears).toHaveBeenCalledTimes(2);
	});

	it('libellés des choix : veille et dernier mois de l’exercice précédent (T4)', async () => {
		await monter(gridVarie(), { fiscalYears: [courant] });
		const kinds = screen.getByTestId('exchange-rates-kinds').textContent ?? '';
		expect(kinds).toContain('cours journalier de la veille de l’ouverture (30.06.2024)');
		expect(kinds).toContain('cours mensuel du dernier mois de l’exercice précédent (juin 2024)');
		expect(kinds).toContain('cours journalier de la date de clôture (30.06.2025)');
		expect(kinds).toContain('cours mensuel du dernier mois (juin 2025)');
		expect(kinds).toContain('Le cours d’ouverture est le cours de clôture de l’exercice précédent');
	});

	it('sans exercice suivant : pas de dialogue, applyToNextOpening false', async () => {
		await monter(gridVarie(), { fiscalYears: [courant] });
		await fireEvent.click(screen.getByTestId('exchange-rates-closing-month'));
		await waitFor(() =>
			expect(api.putRateKinds).toHaveBeenCalledWith(1, {
				openingRateKind: 'day',
				closingRateKind: 'month',
				applyToNextOpening: false,
			}),
		);
		expect(screen.queryByTestId('exchange-rates-kinds-dialog')).toBeNull();
	});

	it('avec un exercice suivant : dialogue, case cochée par défaut, valeur envoyée', async () => {
		await monter(gridVarie(), { fiscalYears: [courant, suivant] });
		await fireEvent.click(screen.getByTestId('exchange-rates-closing-month'));
		const caseSuivant = (await screen.findByTestId('exchange-rates-kinds-apply-next')) as HTMLInputElement;
		expect(caseSuivant.checked).toBe(true);
		expect(caseSuivant.disabled).toBe(false);
		expect(screen.getByTestId('exchange-rates-kinds-dialog').textContent).toContain('2025/26');
		expect(api.putRateKinds).not.toHaveBeenCalled();
		await fireEvent.click(screen.getByTestId('exchange-rates-kinds-save'));
		await waitFor(() =>
			expect(api.putRateKinds).toHaveBeenCalledWith(1, {
				openingRateKind: 'day',
				closingRateKind: 'month',
				applyToNextOpening: true,
			}),
		);
	});

	it('case décochée → applyToNextOpening false', async () => {
		await monter(gridVarie(), { fiscalYears: [courant, suivant] });
		await fireEvent.click(screen.getByTestId('exchange-rates-closing-month'));
		await fireEvent.click(await screen.findByTestId('exchange-rates-kinds-apply-next'));
		await fireEvent.click(screen.getByTestId('exchange-rates-kinds-save'));
		await waitFor(() =>
			expect(api.putRateKinds).toHaveBeenCalledWith(1, expect.objectContaining({ applyToNextOpening: false })),
		);
	});

	it('« Annuler » n’envoie rien', async () => {
		await monter(gridVarie(), { fiscalYears: [courant, suivant] });
		await fireEvent.click(screen.getByTestId('exchange-rates-closing-month'));
		await fireEvent.click(await screen.findByTestId('exchange-rates-kinds-cancel'));
		expect(api.putRateKinds).not.toHaveBeenCalled();
	});

	it('exercice suivant clos : case désactivée avec la raison', async () => {
		await monter(gridVarie(), { fiscalYears: [courant, { ...suivant, status: 'Closed' }] });
		await fireEvent.click(screen.getByTestId('exchange-rates-closing-month'));
		const caseSuivant = (await screen.findByTestId('exchange-rates-kinds-apply-next')) as HTMLInputElement;
		expect(caseSuivant.disabled).toBe(true);
		expect(screen.getByTestId('exchange-rates-kinds-next-closed')).toBeTruthy();
		await fireEvent.click(screen.getByTestId('exchange-rates-kinds-save'));
		await waitFor(() =>
			expect(api.putRateKinds).toHaveBeenCalledWith(1, expect.objectContaining({ applyToNextOpening: false })),
		);
	});

	it('réponse applied:false affichée', async () => {
		await monter(gridVarie(), { fiscalYears: [courant, suivant] });
		api.putRateKinds.mockResolvedValue({
			fiscalYear: courant,
			nextFiscalYear: { id: 2, openingRateKind: 'day', applied: false, reason: 'closed' },
		});
		await fireEvent.click(screen.getByTestId('exchange-rates-closing-month'));
		await fireEvent.click(await screen.findByTestId('exchange-rates-kinds-save'));
		expect((await screen.findByTestId('exchange-rates-message')).textContent).toContain(
			'L’exercice suivant (2025/26) est clos',
		);
	});
});

describe('ExchangeRatesScreen — rôles et exercice clos', () => {
	beforeEach(() => {
		vi.resetAllMocks();
	});

	it('Consultation : ni champ, ni ↑, ni choix, ni récupération', async () => {
		auth.currentUser = { role: 'Consultation' };
		await monter(gridVarie());
		expect(screen.queryAllByRole('textbox')).toHaveLength(0);
		expect(screen.queryByTestId('exchange-rates-arrow-USD-month-2024-08-01')).toBeNull();
		expect(screen.queryByTestId('exchange-rates-opening-day')).toBeNull();
		expect(screen.getByTestId('exchange-rates-opening-text')).toBeTruthy();
		expect(screen.queryByTestId('exchange-rates-fetch-missing')).toBeNull();
		expect(screen.getByTestId('exchange-rates-value-USD-month-2024-10-01').textContent).toBe('0.9000');
	});

	it('exercice clos : lecture seule, choix désactivés, « Récupérer » offert', async () => {
		auth.currentUser = { role: 'Comptable' };
		await monter(gridVarie(), { fiscalYears: [fy({ status: 'Closed' })] });
		expect(screen.queryAllByRole('textbox')).toHaveLength(0);
		expect(screen.queryByTestId('exchange-rates-arrow-USD-month-2024-10-01')).toBeNull();
		expect((screen.getByTestId('exchange-rates-opening-day') as HTMLInputElement).disabled).toBe(true);
		expect((screen.getByTestId('exchange-rates-closing-month') as HTMLInputElement).disabled).toBe(true);
		expect(screen.getByTestId('exchange-rates-fetch-missing')).toBeTruthy();
		expect(screen.getByTestId('exchange-rates-closed')).toBeTruthy();
	});

	it('revue P1 (37-2-CR-P1-7) — échec de lecture des exercices : l’erreur, pas « aucun exercice »', async () => {
		auth.currentUser = { role: 'Admin' };
		settings.fetchCompanyCurrent.mockResolvedValue(company());
		fys.listFiscalYears.mockRejectedValue({ code: 'INTERNAL_ERROR', message: 'Lecture impossible', status: 500 });
		render(ExchangeRatesScreen);
		expect((await screen.findByTestId('exchange-rates-load-error')).textContent).toBe('Lecture impossible');
		expect(screen.queryByTestId('exchange-rates-no-fiscal-year')).toBeNull();
		expect(screen.queryByTestId('exchange-rates-mono')).toBeNull();
	});

	it('mono-devise : message, aucun exercice lu', async () => {
		auth.currentUser = { role: 'Admin' };
		settings.fetchCompanyCurrent.mockResolvedValue(company(false));
		render(ExchangeRatesScreen);
		expect(await screen.findByTestId('exchange-rates-mono')).toBeTruthy();
		expect(fys.listFiscalYears).not.toHaveBeenCalled();
	});
});

describe('ExchangeRatesScreen — « Récupérer les cours manquants »', () => {
	beforeEach(() => {
		vi.resetAllMocks();
		auth.currentUser = { role: 'Admin' };
	});

	/** Exercice de six mois ; tout publié et retenu, sauf septembre et octobre (deux devises). */
	function gridManques(): FiscalYearRatesResponse {
		return yearGrid('2024-07-01', '2024-12-31', 'day', 'day', (g) => {
			const plein = (cur: string) => {
				const cells: Record<string, Partial<Cell>> = {
					opening: { effectiveRate: '0.9', source: 'afc', row: afcRow(), published: published('0.9') },
					closing: { effectiveRate: '0.9', source: 'afc', row: afcRow(), published: published('0.9') },
				};
				for (const m of g.months) {
					cells[`month:${m}`] = { effectiveRate: '0.9', source: 'afc', row: afcRow(), published: published('0.9') };
				}
				cells['month:2024-09-01'] = {};
				cells['month:2024-10-01'] = {};
				return row(g, cur, cells);
			};
			return [plein('USD'), plein('EUR')];
		});
	}

	it('un POST par colonne, l’un après l’autre ; un échec n’arrête pas les suivants', async () => {
		await monter(gridManques());
		api.fetchRates
			.mockRejectedValueOnce({ code: 'RATE_SOURCE_UNAVAILABLE', message: 'indisponible', status: 503 })
			.mockResolvedValueOnce({ kind: 'month', rateDate: '2024-10-01', sourceDate: null, count: 40, rejected: 0, corrected: 0 });
		await fireEvent.click(screen.getByTestId('exchange-rates-fetch-missing'));
		await waitFor(() => expect(api.getYearGrid).toHaveBeenCalledTimes(2));
		expect(api.fetchRates).toHaveBeenCalledTimes(2);
		expect(api.fetchRates.mock.calls[0][0]).toEqual({ year: 2024, month: 9 });
		expect(api.fetchRates.mock.calls[1][0]).toEqual({ year: 2024, month: 10 });
		// L'échec s'affiche sur les cases de sa colonne, pour chaque devise.
		for (const cur of ['USD', 'EUR']) {
			expect(
				(await screen.findByTestId(`exchange-rates-fetch-error-${cur}-month-2024-09-01`)).textContent,
			).toContain('La source des cours ne répond pas');
			expect(screen.queryByTestId(`exchange-rates-fetch-error-${cur}-month-2024-10-01`)).toBeNull();
		}
		expect(api.getFetchStatus).toHaveBeenCalledTimes(2);
	});

	it('« Revérifier » : un POST par colonne récupérable déjà présente', async () => {
		await monter(gridManques());
		api.fetchRates.mockResolvedValue({ kind: 'month', rateDate: '2024-07-01', sourceDate: null, count: 1, rejected: 0, corrected: 0 });
		await fireEvent.click(screen.getByTestId('exchange-rates-recheck'));
		await waitFor(() => expect(api.getYearGrid).toHaveBeenCalledTimes(2));
		// Six mois moins deux sans valeur publiée, plus Ouverture et Clôture (jours passés).
		expect(api.fetchRates).toHaveBeenCalledTimes(6);
		expect(api.fetchRates.mock.calls.map((c) => c[0])).toContainEqual({ date: '2024-06-30' });
		expect(api.fetchRates.mock.calls.map((c) => c[0])).toContainEqual({ date: '2024-12-31' });
	});

	it('« Ajouter une devise » : une ligne de plus, liste ISO sans CHF', async () => {
		await monter(gridManques());
		const select = screen.getByTestId('exchange-rates-add-currency-select') as HTMLSelectElement;
		const options = [...select.options].map((o) => o.value);
		expect(options).not.toContain('CHF');
		expect(options).not.toContain('USD');
		expect(options).toContain('GBP');
		await fireEvent.change(select, { target: { value: 'GBP' } });
		await fireEvent.click(screen.getByTestId('exchange-rates-add-currency'));
		expect(await screen.findByTestId('exchange-rates-row-GBP')).toBeTruthy();
	});

	it('« Toutes les devises publiées » relit avec all=true', async () => {
		await monter(gridManques());
		await fireEvent.click(screen.getByTestId('exchange-rates-all'));
		await waitFor(() => expect(api.getYearGrid).toHaveBeenLastCalledWith(1, true));
	});
});

describe('ExchangeRatesScreen — en-têtes et liaisons des colonnes (S4, T4, T2)', () => {
	beforeEach(() => {
		vi.resetAllMocks();
		auth.currentUser = { role: 'Admin' };
	});

	function enTetes(): string[] {
		return screen
			.getAllByTestId(/^exchange-rates-header-month-/)
			.map((h) => (h.textContent ?? '').trim());
	}

	const cas = [
		{ nom: '1er janvier', start: '2024-01-01', end: '2024-12-31', n: 12, first: 'janv. 2024', last: 'déc. 2024', ouvJour: '31.12.2023', ouvMois: 'déc. 2023', clotJour: '31.12.2024' },
		{ nom: '1er juillet', start: '2024-07-01', end: '2025-06-30', n: 12, first: 'juil. 2024', last: 'juin 2025', ouvJour: '30.06.2024', ouvMois: 'juin 2024', clotJour: '30.06.2025' },
		{ nom: '1er avril', start: '2023-04-01', end: '2024-03-31', n: 12, first: 'avr. 2023', last: 'mars 2024', ouvJour: '31.03.2023', ouvMois: 'mars 2023', clotJour: '31.03.2024' },
		{ nom: 'six mois', start: '2024-07-01', end: '2024-12-31', n: 6, first: 'juil. 2024', last: 'déc. 2024', ouvJour: '30.06.2024', ouvMois: 'juin 2024', clotJour: '31.12.2024' },
		{ nom: 'dix-huit mois', start: '2023-07-01', end: '2024-12-31', n: 18, first: 'juil. 2023', last: 'déc. 2024', ouvJour: '30.06.2023', ouvMois: 'juin 2023', clotJour: '31.12.2024' },
	];

	for (const c of cas) {
		it(`exercice ${c.nom}, genres day : ${c.n} colonnes, Ouverture ${c.ouvJour}, Clôture ${c.clotJour}`, async () => {
			const g = yearGrid(c.start, c.end, 'day', 'day', (g) => [row(g, 'USD')]);
			await monter(g, { fiscalYears: [fy({ startDate: c.start, endDate: c.end })] });
			const h = enTetes();
			expect(h).toHaveLength(c.n);
			expect(h[0]).toBe(c.first);
			expect(h[c.n - 1]).toBe(c.last);
			expect(screen.getByTestId('exchange-rates-header-opening').textContent).toContain(c.ouvJour);
			expect(screen.getByTestId('exchange-rates-header-closing').textContent).toContain(c.clotJour);
		});

		it(`exercice ${c.nom}, genres month : Ouverture propre (${c.ouvMois}), Clôture liée au dernier mois`, async () => {
			const g = yearGrid(c.start, c.end, 'month', 'month', (g) => [row(g, 'USD')]);
			await monter(g, { fiscalYears: [fy({ startDate: c.start, endDate: c.end })] });
			expect(screen.getByTestId('exchange-rates-header-opening').textContent).toContain(c.ouvMois);
			// Case propre : un champ, pas de liaison.
			expect(screen.getByTestId(`exchange-rates-input-USD-opening-${g.openingDate}`)).toBeTruthy();
			expect(screen.queryByTestId(`exchange-rates-linked-USD-opening-${g.openingDate}`)).toBeNull();
			// Clôture : la même case que la colonne du dernier mois.
			expect(screen.getByTestId(`exchange-rates-linked-USD-closing-${g.closingDate}`).textContent).toBe(
				`Même case que ${c.last}`,
			);
			expect(screen.queryByTestId(`exchange-rates-input-USD-closing-${g.closingDate}`)).toBeNull();
		});
	}

	it('exercice 15.03.2024–14.03.2025 : treize colonnes ; Ouverture 14.03.2024 ou liée à mars 2024', async () => {
		const day = yearGrid('2024-03-15', '2025-03-14', 'day', 'day', (g) => [row(g, 'USD')]);
		await monter(day, { fiscalYears: [fy({ startDate: '2024-03-15', endDate: '2025-03-14' })] });
		expect(enTetes()).toHaveLength(13);
		expect(enTetes()[0]).toBe('mars 2024');
		expect(screen.getByTestId('exchange-rates-header-opening').textContent).toContain('14.03.2024');
	});

	it('exercice 15.03.2024–14.03.2025, genre month : Ouverture liée à la première colonne', async () => {
		const month = yearGrid('2024-03-15', '2025-03-14', 'month', 'day', (g) => [row(g, 'USD')]);
		await monter(month, { fiscalYears: [fy({ startDate: '2024-03-15', endDate: '2025-03-14' })] });
		expect(screen.getByTestId('exchange-rates-linked-USD-opening-2024-03-01').textContent).toBe(
			'Même case que mars 2024',
		);
		expect(screen.getByTestId('exchange-rates-input-USD-month-2024-03-01')).toBeTruthy();
	});

	it('défilement horizontal du seul tableau, colonne des devises figée', async () => {
		const g = yearGrid('2023-07-01', '2024-12-31', 'day', 'day', (g) => [row(g, 'USD')]);
		await monter(g, { fiscalYears: [fy({ startDate: '2023-07-01', endDate: '2024-12-31' })] });
		expect(screen.getByTestId('exchange-rates-table-scroll').className).toContain('overflow-x-auto');
		const premiere = screen.getByTestId('exchange-rates-row-USD').querySelector('th');
		expect(premiere?.className).toContain('sticky');
	});

	it('exercice par défaut : celui qui couvre aujourd’hui', async () => {
		const g = yearGrid('2024-07-01', '2025-06-30', 'day', 'day', (g) => [row(g, 'USD')]);
		await monter(g, {
			fiscalYears: [
				fy({ id: 5, name: '2023/24', startDate: '2023-07-01', endDate: '2024-06-30' }),
				fy({ id: 6, name: '2024/25', startDate: '2024-07-01', endDate: '2025-06-30' }),
			],
		});
		expect(api.getYearGrid).toHaveBeenCalledWith(6, false);
		expect((screen.getByTestId('exchange-rates-fiscal-year-select') as HTMLSelectElement).value).toBe('6');
	});
});
