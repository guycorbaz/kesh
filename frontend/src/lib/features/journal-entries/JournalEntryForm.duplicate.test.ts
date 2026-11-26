// Story 53-1a (#1355, table J) — le formulaire de saisie face au doublon et à l'issue
// inconnue : J5 (doublon probable, confirmable), J6 (réponse perdue), J7 (erreur
// ordinaire), les gardes synchrones (un clic = une requête), le cas permis de J2a (une
// recherche de cours qui se résout pendant l'attente écrit la ligne, et le corps porte
// le cours mis à jour — 37-3b AC-2) et le câblage de `canWriteEntry` (relevé C1, C2).

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, fireEvent, screen, waitFor, cleanup } from '@testing-library/svelte';

const { createMock, toastError, toastSuccess } = vi.hoisted(() => ({
	createMock: vi.fn(),
	toastError: vi.fn(),
	toastSuccess: vi.fn()
}));

vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string, args?: Record<string, unknown>) =>
		fallback.replace(/\{ \$(\w+) \}/g, (_m: string, k: string) => String(args?.[k] ?? ''))
}));
vi.mock('$lib/features/onboarding/onboarding.svelte', () => ({
	i18nMsg: (_key: string, fallback: string, args?: Record<string, unknown>) =>
		fallback.replace(/\{ \$(\w+) \}/g, (_m: string, k: string) => String(args?.[k] ?? ''))
}));
vi.mock('./journal-entries.api', () => ({
	createJournalEntry: createMock,
	updateJournalEntry: vi.fn()
}));
vi.mock('svelte-sonner', () => ({
	toast: { error: toastError, success: toastSuccess }
}));
vi.mock('$lib/shared/utils/notify', () => ({
	notifyMissingFiscalYearOrFallback: vi.fn(() => false)
}));
vi.mock('$lib/shared/utils/local-date', () => ({ todayLocal: () => '2026-03-15' }));

import JournalEntryForm from './JournalEntryForm.svelte';
import * as submitState from './entry-submit-state';
import type { AccountResponse } from '$lib/features/accounts/accounts.types';
import { createRateLookupCache } from './rate-lookup';

function account(id: number, number: string, extra: Partial<AccountResponse> = {}): AccountResponse {
	return {
		id,
		companyId: 1,
		number,
		name: `Compte ${number}`,
		accountType: 'Asset',
		parentId: null,
		active: true,
		role: null,
		postable: true,
		letterable: false,
		version: 1,
		createdAt: '2026-01-01T00:00:00',
		updatedAt: '2026-01-01T00:00:00',
		...extra
	};
}

const CHF1 = account(1, '1000');
const CHF2 = account(2, '2000');
const USD = account(10, '1022', { currency: 'USD' });
const GAIN = account(31, '6840', { accountType: 'Revenue' });
const LOSS = account(32, '6940', { accountType: 'Expense' });

const tick = () => new Promise((r) => setTimeout(r, 0));

async function pick(index: number, number: string) {
	const input = screen.getAllByRole('textbox', { name: 'Compte' })[index];
	await fireEvent.focus(input);
	const option = screen.getAllByRole('option').find((o) => (o.textContent ?? '').includes(number));
	if (!option) throw new Error(`option ${number} introuvable`);
	await fireEvent.mouseDown(option);
	await fireEvent.blur(input);
}

const onSuccess = vi.fn();
const onCancel = vi.fn();

/** Une écriture CHF valide : 100.00 de `1000` à `2000`, « Vente 1 ». */
async function mountSaisie() {
	const view = render(JournalEntryForm, {
		props: {
			accounts: [CHF1, CHF2],
			accountsLoadError: false,
			onSuccess,
			onCancel
		}
	});
	await fireEvent.input(document.getElementById('entry-description')!, {
		target: { value: 'Vente 1' }
	});
	await pick(0, '1000');
	await pick(1, '2000');
	const montants = screen.getAllByPlaceholderText('0.00');
	await fireEvent.input(montants[0], { target: { value: '100.00' } });
	await fireEvent.input(montants[3], { target: { value: '100.00' } });
	return view;
}

const valider = () => screen.getByRole('button', { name: 'Valider' }) as HTMLButtonElement;
const champs = () => screen.getByTestId('journal-entry-fields') as HTMLFieldSetElement;

const DOUBLON = {
	code: 'JOURNAL_ENTRY_PROBABLE_DUPLICATE',
	status: 409,
	message: 'message du serveur, jamais affiché',
	details: { journalEntryId: 7, entryNumber: 12, entryDate: '2026-03-15', confirmable: true }
};
const COUPURE = { code: 'NETWORK_ERROR', status: 0, message: 'réseau coupé' };

beforeEach(() => {
	createMock.mockReset();
	toastError.mockReset();
	toastSuccess.mockReset();
	onSuccess.mockReset();
	onCancel.mockReset();
});
afterEach(() => {
	vi.restoreAllMocks();
	cleanup();
});

describe('JournalEntryForm — création : J5, doublon probable', () => {
	it('AC-9/AC-10 : texte du catalogue, écriture citée, lien en nouvel onglet, deux boutons, figé, « Annuler » actif', async () => {
		await mountSaisie();
		createMock.mockRejectedValueOnce(DOUBLON);
		await fireEvent.click(valider());
		await tick();

		const avis = screen.getByTestId('journal-entry-probable-duplicate');
		expect(avis).toBeTruthy();
		// Le texte du catalogue (M20 : jamais `err.message`).
		expect(screen.getByTestId('journal-entry-probable-duplicate-text').textContent).toContain(
			'Une écriture identique existe déjà à cette date'
		);
		expect(avis.textContent).not.toContain('jamais affiché');
		const lien = screen.getByTestId('journal-entry-duplicate-link') as HTMLAnchorElement;
		expect(lien.getAttribute('href')).toBe('/journal-entries/7');
		expect(lien.getAttribute('target')).toBe('_blank');
		expect(lien.textContent).toContain('Écriture n° 12 du 15.03.2026');
		expect(screen.getByTestId('journal-entry-confirm-not-duplicate')).toBeTruthy();
		expect(screen.getByTestId('journal-entry-modify-entry')).toBeTruthy();
		// « Valider » masqué, saisie figée sans rien vider, « Annuler » actif.
		expect(screen.queryByRole('button', { name: 'Valider' })).toBeNull();
		expect(champs().disabled).toBe(true);
		expect((document.getElementById('entry-description') as HTMLInputElement).value).toBe('Vente 1');
		expect((screen.getByRole('button', { name: 'Annuler' }) as HTMLButtonElement).disabled).toBe(false);
		expect(toastError).not.toHaveBeenCalled();
	});

	it('AC-10 (M18) : la confirmation renvoie le corps gardé avec confirmNotDuplicate, une requête', async () => {
		await mountSaisie();
		createMock.mockRejectedValueOnce(DOUBLON);
		await fireEvent.click(valider());
		await tick();
		const premier = createMock.mock.calls[0][0];
		expect(premier.confirmNotDuplicate).toBeUndefined();

		createMock.mockResolvedValueOnce({});
		const bouton = screen.getByTestId('journal-entry-confirm-not-duplicate');
		// Deux clics avant le rendu : une seule requête (garde synchrone).
		void fireEvent.click(bouton);
		void fireEvent.click(bouton);
		await tick();
		await tick();

		expect(createMock).toHaveBeenCalledTimes(2);
		expect(createMock.mock.calls[1][0]).toEqual({ ...premier, confirmNotDuplicate: true });
		expect(onSuccess).toHaveBeenCalledTimes(1);
	});

	it('AC-10 : « Modifier la saisie » rend J1 et l’envoi suivant est ordinaire', async () => {
		await mountSaisie();
		createMock.mockRejectedValueOnce(DOUBLON);
		await fireEvent.click(valider());
		await tick();

		await fireEvent.click(screen.getByTestId('journal-entry-modify-entry'));
		expect(screen.queryByTestId('journal-entry-probable-duplicate')).toBeNull();
		expect(champs().disabled).toBe(false);

		createMock.mockResolvedValueOnce({});
		await fireEvent.click(valider());
		await tick();
		expect(createMock).toHaveBeenCalledTimes(2);
		expect('confirmNotDuplicate' in createMock.mock.calls[1][0]).toBe(false);
	});

	it('le code de doublon sans `details` lisibles : J7, le message du serveur, aucun bouton de confirmation', async () => {
		await mountSaisie();
		createMock.mockRejectedValueOnce({ ...DOUBLON, details: { journalEntryId: 'sept' } });
		await fireEvent.click(valider());
		await tick();
		expect(screen.queryByTestId('journal-entry-probable-duplicate')).toBeNull();
		expect(toastError).toHaveBeenCalledWith('message du serveur, jamais affiché');
		expect(champs().disabled).toBe(false);
		expect(valider().disabled).toBe(false);
	});
});

describe('JournalEntryForm — création : J6, issue inconnue', () => {
	it('AC-8 : figé, avis, lien filtré en nouvel onglet, aucun envoi actif ; Valider et Ctrl+S sans effet ; la reprise n’envoie rien', async () => {
		await mountSaisie();
		createMock.mockRejectedValueOnce(COUPURE);
		await fireEvent.click(valider());
		await tick();

		// L'avis, jamais « Erreur lors de la sauvegarde » (M16).
		expect(screen.getByTestId('journal-entry-unknown-outcome-text').textContent).toContain(
			'L’enregistrement a peut-être abouti'
		);
		expect(toastError).not.toHaveBeenCalled();
		const lien = screen.getByTestId('journal-entry-unknown-outcome-link') as HTMLAnchorElement;
		expect(lien.getAttribute('href')).toBe('/journal-entries?dateFrom=2026-03-15&dateTo=2026-03-15');
		expect(lien.getAttribute('target')).toBe('_blank');
		expect(champs().disabled).toBe(true);
		expect(screen.queryByRole('button', { name: 'Valider' })).toBeNull();
		expect((screen.getByRole('button', { name: 'Annuler' }) as HTMLButtonElement).disabled).toBe(false);

		// Ctrl+S sans effet.
		await fireEvent.keyDown(window, { key: 's', ctrlKey: true });
		await tick();
		expect(createMock).toHaveBeenCalledTimes(1);

		// La reprise rend la saisie intacte, sans requête (M17).
		await fireEvent.click(screen.getByTestId('journal-entry-resume-entry'));
		expect(createMock).toHaveBeenCalledTimes(1);
		expect(screen.queryByTestId('journal-entry-unknown-outcome')).toBeNull();
		expect(champs().disabled).toBe(false);
		expect((document.getElementById('entry-description') as HTMLInputElement).value).toBe('Vente 1');
		expect(valider().disabled).toBe(false);
	});

	it('AC-9 : après la reprise, l’envoi rend le doublon : J5', async () => {
		await mountSaisie();
		createMock.mockRejectedValueOnce(COUPURE);
		await fireEvent.click(valider());
		await tick();
		await fireEvent.click(screen.getByTestId('journal-entry-resume-entry'));

		createMock.mockRejectedValueOnce(DOUBLON);
		await fireEvent.click(valider());
		await tick();
		expect(createMock).toHaveBeenCalledTimes(2);
		expect(screen.getByTestId('journal-entry-probable-duplicate')).toBeTruthy();
		expect(champs().disabled).toBe(true);
	});

	it('toute erreur 5xx, ou hors ApiError, est une issue inconnue', async () => {
		await mountSaisie();
		createMock.mockRejectedValueOnce({ code: 'INTERNAL_ERROR', status: 503, message: 'panne' });
		await fireEvent.click(valider());
		await tick();
		expect(screen.getByTestId('journal-entry-unknown-outcome')).toBeTruthy();
	});
});

describe('JournalEntryForm — création : J7 et gardes', () => {
	it('J7 : un refus ordinaire laisse la saisie active, avec le message du serveur', async () => {
		await mountSaisie();
		createMock.mockRejectedValueOnce({ code: 'ENTRY_UNBALANCED', status: 400, message: 'déséquilibre' });
		await fireEvent.click(valider());
		await tick();
		expect(toastError).toHaveBeenCalledWith('déséquilibre');
		expect(champs().disabled).toBe(false);
		expect(valider().disabled).toBe(false);
	});

	it('AC-11 (a, M22) : deux clics avant le rendu, ou Ctrl+S puis clic, ne font qu’une requête', async () => {
		await mountSaisie();
		let sortie: (v: unknown) => void = () => {};
		createMock.mockReturnValueOnce(new Promise((r) => (sortie = r)));
		const bouton = valider();
		void fireEvent.click(bouton);
		void fireEvent.click(bouton);
		void fireEvent.keyDown(window, { key: 's', ctrlKey: true });
		await tick();
		expect(createMock).toHaveBeenCalledTimes(1);
		sortie({});
		await tick();
		expect(createMock).toHaveBeenCalledTimes(1);
	});

	it('succès : toast, onSuccess, une seule requête', async () => {
		await mountSaisie();
		createMock.mockResolvedValueOnce({});
		await fireEvent.click(valider());
		await tick();
		expect(toastSuccess).toHaveBeenCalledTimes(1);
		expect(onSuccess).toHaveBeenCalledTimes(1);
	});
});

// ---------------------------------------------------------------------------
// Le cas permis de J2a et le câblage de `canWriteEntry` (relevé C)
// ---------------------------------------------------------------------------

/** Un cache dont le premier appel rend tout de suite, les suivants attendent qu'on les résolve. */
function cacheDifferee() {
	const resolutions: Array<(rate: string) => void> = [];
	const reponse = (currency: string, date: string, rate: string) => ({
		currency,
		kind: 'month',
		rateDate: date,
		effectiveRate: rate,
		source: 'afc',
		publishedRate: null,
		publishedUnit: null,
		sourceDate: null
	});
	let appels = 0;
	const lookup = vi.fn((currency: string, date: string) => {
		appels += 1;
		if (appels === 1) return Promise.resolve(reponse(currency, date, '0.8846'));
		return new Promise((resolve) => {
			resolutions.push((rate) => resolve(reponse(currency, date, rate)));
		});
	});
	const fetchRates = vi.fn();
	const cache = createRateLookupCache({
		lookup: lookup as unknown as Parameters<typeof createRateLookupCache>[0]['lookup'],
		fetch: fetchRates,
		canFetch: () => true,
		today: () => '2026-03-15'
	});
	return { cache, lookup, resolutions, fetchRates };
}

const val = (id: string) => (screen.getByTestId(id) as HTMLInputElement).value;

/** Ligne 0 : USD 100 au débit (cours 0.8846) ; ligne 1 : CHF 88.46 au crédit. */
async function saisieUsd(cache: ReturnType<typeof cacheDifferee>['cache']) {
	render(JournalEntryForm, {
		props: {
			accounts: [CHF1, USD, GAIN, LOSS],
			accountsLoadError: false,
			multiCurrency: true,
			companyFx: { fxGainAccountId: 31, fxLossAccountId: 32 },
			rateLookup: cache,
			onSuccess,
			onCancel
		}
	});
	await fireEvent.input(document.getElementById('entry-description')!, {
		target: { value: 'Écriture USD' }
	});
	await pick(0, '1022');
	await waitFor(() => expect(val('journal-entry-line-fx-rate-0')).toBe('0.8846'));
	await fireEvent.input(screen.getByTestId('journal-entry-line-fx-debit-0'), { target: { value: '100' } });
	await pick(1, '1000');
	const montants = screen.getAllByPlaceholderText('0.00');
	await fireEvent.input(montants[montants.length - 1], { target: { value: '88.46' } });
}

describe('JournalEntryForm — J2a et relevé C', () => {
	it('AC-11 (b, M27) : une recherche qui se résout en J2a met la ligne à jour ; le corps envoyé porte le cours du nouveau mois', async () => {
		const { cache, resolutions } = cacheDifferee();
		await saisieUsd(cache);
		// Changement de date vers un autre mois, puis « Valider » aussitôt : l'envoi
		// attend la recherche (J2a), formulaire figé, rien n'est parti.
		await fireEvent.input(document.getElementById('entry-date')!, { target: { value: '2026-04-10' } });
		await fireEvent.click(valider());
		await tick();
		expect(createMock).not.toHaveBeenCalled();
		expect(champs().disabled).toBe(true);
		expect(resolutions).toHaveLength(1);

		// La recherche se résout pendant l'attente : la ligne prend le cours d'avril.
		resolutions[0]('0.9000');
		await waitFor(() => expect(champs().disabled).toBe(false));
		// Le CHF dérivé (90.00) ne tient plus l'équilibre avec le crédit saisi : retour en
		// J1 sans envoi (table J, J2a → J1) — le corps n'a pas été formé.
		expect(createMock).not.toHaveBeenCalled();
		expect(val('journal-entry-line-fx-rate-0')).toBe('0.9');
		expect(val('journal-entry-line-fx-chf-0')).toBe('90.00');

		// La saisie rééquilibrée, l'envoi porte le cours d'avril.
		const montants = screen.getAllByPlaceholderText('0.00');
		await fireEvent.input(montants[montants.length - 1], { target: { value: '90.00' } });
		createMock.mockResolvedValueOnce({});
		await fireEvent.click(valider());
		await tick();
		expect(createMock).toHaveBeenCalledTimes(1);
		expect(createMock.mock.calls[0][0].lines[0].foreign.rate).toBe('0.9');
	});

	it('AC-11 (b, A1) : une saisie qui reste équilibrée quand le cours change part directement de J2a en J2b, avec le cours mis à jour (37-3b-CR-P1-5)', async () => {
		const { cache, resolutions } = cacheDifferee();
		const USD2 = account(11, '1023', { currency: 'USD' });
		render(JournalEntryForm, {
			props: {
				accounts: [CHF1, USD, USD2, GAIN, LOSS],
				accountsLoadError: false,
				multiCurrency: true,
				companyFx: { fxGainAccountId: 31, fxLossAccountId: 32 },
				rateLookup: cache,
				onSuccess,
				onCancel
			}
		});
		await fireEvent.input(document.getElementById('entry-description')!, {
			target: { value: 'Virement USD' }
		});
		await pick(0, '1022');
		await waitFor(() => expect(val('journal-entry-line-fx-rate-0')).toBe('0.8846'));
		await fireEvent.input(screen.getByTestId('journal-entry-line-fx-debit-0'), { target: { value: '100' } });
		await pick(1, '1023');
		await waitFor(() => expect(val('journal-entry-line-fx-rate-1')).toBe('0.8846'));
		await fireEvent.input(screen.getByTestId('journal-entry-line-fx-credit-1'), { target: { value: '100' } });

		// Autre mois, puis « Valider » aussitôt : J2a, rien n'est parti.
		await fireEvent.input(document.getElementById('entry-date')!, { target: { value: '2026-04-10' } });
		createMock.mockResolvedValueOnce({});
		await fireEvent.click(valider());
		await tick();
		expect(createMock).not.toHaveBeenCalled();
		expect(resolutions).toHaveLength(1);

		// La recherche se résout : les deux lignes prennent le cours d'avril, restent
		// équilibrées, et le PREMIER envoi — formé après l'attente — le porte.
		resolutions[0]('0.9');
		await waitFor(() => expect(createMock).toHaveBeenCalledTimes(1));
		const corps = createMock.mock.calls[0][0];
		expect(corps.lines[0].foreign.rate).toBe('0.9');
		expect(corps.lines[1].foreign.rate).toBe('0.9');
		expect(corps.lines[0].debit).toBe('90.00');
		expect(corps.lines[1].credit).toBe('90.00');
	});

	it('relevé C1 (câblage, M25) : la résolution d’une recherche consulte `canWriteEntry`', async () => {
		const spy = vi.spyOn(submitState, 'canWriteEntry').mockReturnValue(false);
		const { cache, lookup } = cacheDifferee();
		render(JournalEntryForm, {
			props: {
				accounts: [CHF1, USD, GAIN, LOSS],
				accountsLoadError: false,
				multiCurrency: true,
				companyFx: { fxGainAccountId: 31, fxLossAccountId: 32 },
				rateLookup: cache,
				onSuccess,
				onCancel
			}
		});
		await pick(0, '1022');
		await waitFor(() => expect(lookup).toHaveBeenCalledTimes(1));
		await tick();
		await tick();
		// La recherche est résolue, mais l'état ne permet pas d'écrire la saisie : le
		// cours proposé n'a pas été posé.
		expect(spy).toHaveBeenCalled();
		expect(val('journal-entry-line-fx-rate-0')).toBe('');
	});

	it('relevé C2 (câblage, M26) : l’effet de la date consulte `canWriteEntry` avant de programmer une recherche', async () => {
		const { cache, lookup } = cacheDifferee();
		render(JournalEntryForm, {
			props: {
				accounts: [CHF1, USD, GAIN, LOSS],
				accountsLoadError: false,
				multiCurrency: true,
				companyFx: { fxGainAccountId: 31, fxLossAccountId: 32 },
				rateLookup: cache,
				onSuccess,
				onCancel
			}
		});
		await pick(0, '1022');
		await waitFor(() => expect(lookup).toHaveBeenCalledTimes(1));
		await waitFor(() => expect(val('journal-entry-line-fx-rate-0')).toBe('0.8846'));

		const spy = vi.spyOn(submitState, 'canWriteEntry').mockReturnValue(false);
		await fireEvent.input(document.getElementById('entry-date')!, { target: { value: '2026-04-10' } });
		await new Promise((r) => setTimeout(r, 450));
		expect(spy).toHaveBeenCalled();
		expect(lookup).toHaveBeenCalledTimes(1);

		// Témoin : l'état permet d'écrire, la même date change, la recherche part.
		spy.mockReturnValue(true);
		await fireEvent.input(document.getElementById('entry-date')!, { target: { value: '2026-05-10' } });
		await new Promise((r) => setTimeout(r, 450));
		expect(lookup).toHaveBeenCalledTimes(2);
	});

	it('AC-11 (c, M19) : « Récupérer le mois » ne fait rien hors des états de saisie — ni récupération, ni envoi', async () => {
		const { cache, fetchRates } = cacheDifferee();
		await saisieUsd(cache);
		createMock.mockRejectedValueOnce({
			code: 'RATE_UNAVAILABLE',
			status: 400,
			message: 'cours indisponible',
			details: { currency: 'USD', month: '2026-03' }
		});
		await fireEvent.click(valider());
		await tick();
		const bouton = screen.getByRole('button', { name: 'Récupérer le mois' });

		// L'état ne permet plus l'envoi ordinaire : la garde de `fetchMissingMonth` rend
		// la main avant de toucher à l'avis, et sans récupération.
		vi.spyOn(submitState, 'canSendFrom').mockReturnValue(false);
		await fireEvent.click(bouton);
		await tick();
		expect(fetchRates).not.toHaveBeenCalled();
		expect(screen.queryByTestId('journal-entry-rate-unavailable')).not.toBeNull();
		expect(createMock).toHaveBeenCalledTimes(1);
	});
});
