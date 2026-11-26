// Story 37-3b (AC-2 bis) — l'assistant Conversion : aperçu avec la ligne de
// différence, corps envoyé (devise de chaque compte, ligne de différence portée,
// sans `completeDifference`), refus affichés sur la colonne de `details.entry`
// ou au-dessus de l'aperçu, bouton désactivé avec son motif.
//
// Story 53-1b (#1365, table K) — une conversion ne passe pas deux fois : doublon
// probable (K5 : colonne, lien, confirmation avec le corps gardé), issue inconnue (K6 :
// figé, lien de vérification, reprise sans envoi), erreur ordinaire (K7), gel (K2a/K2b),
// envoi unique, et l'envoi qui attend les recherches de cours en vol (AC-9).

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, fireEvent, screen, waitFor } from '@testing-library/svelte';

const { conversionMock } = vi.hoisted(() => ({ conversionMock: vi.fn() }));

vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string, args?: Record<string, unknown>) =>
		fallback.replace(/\{ \$(\w+) \}/g, (_m: string, k: string) => String(args?.[k] ?? ''))
}));
vi.mock('./journal-entries.api', () => ({ createConversion: conversionMock }));
vi.mock('svelte-sonner', () => ({ toast: { error: vi.fn(), success: vi.fn() } }));
vi.mock('$lib/shared/utils/local-date', () => ({ todayLocal: () => '2026-03-05' }));

import ConversionAssistant from './ConversionAssistant.svelte';
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
		createdAt: '',
		updatedAt: '',
		...extra
	};
}

const ACCOUNTS = [
	account(21, '1021', { currency: 'EUR' }),
	account(22, '1022', { currency: 'USD' }),
	account(30, '1090'),
	account(31, '6840', { accountType: 'Revenue' }),
	account(32, '6940', { accountType: 'Expense' })
];

/** Une recherche de cours **retenue** (AC-9) : la date retenue ne se résout qu'à la demande. */
let held: { date: string; settle: (outcome: 'rate' | 'none') => void } | null = null;

function cache() {
	const rates: Record<string, string> = { EUR: '0.947', USD: '0.8846' };
	return createRateLookupCache({
		lookup: async (currency, date, kind) => {
			if (held !== null && date.startsWith(held.date.slice(0, 7)) && currency === 'USD') {
				const slot = held;
				await new Promise<void>((resolve) => {
					slot.settle = (outcome) => {
						if (outcome === 'none') rates.USD = '';
						else rates.USD = '0.9';
						resolve();
					};
				});
				if (rates.USD === '')
					throw { code: 'RATE_UNAVAILABLE', status: 404, message: 'aucun cours' };
			}
			return {
			currency,
			kind,
			rateDate: date,
			effectiveRate: rates[currency],
			source: 'afc',
			publishedRate: null,
			publishedUnit: null,
			sourceDate: null
			};
		},
		fetch: vi.fn(),
		canFetch: () => false,
		today: () => '2026-03-05'
	});
}

async function pickIn(side: 'debit' | 'credit', number: string) {
	const col = screen.getByTestId(`conversion-column-${side}`);
	const input = col.querySelector('input[aria-label="Compte"]') as HTMLInputElement;
	await fireEvent.focus(input);
	const option = screen.getAllByRole('option').find((o) => (o.textContent ?? '').includes(number));
	if (!option) throw new Error(`option ${number} introuvable`);
	await fireEvent.mouseDown(option);
	await fireEvent.blur(input);
}

function mount() {
	return render(ConversionAssistant, {
		props: {
			accounts: ACCOUNTS,
			company: { fxGainAccountId: 31, fxLossAccountId: 32, fxTransitAccountId: 30 },
			openFiscalYears: [{ startDate: '2026-01-01', endDate: '2026-12-31' }],
			cache: cache(),
			onDone: vi.fn()
		}
	});
}

async function fill() {
	mount();
	await pickIn('debit', '1021');
	await pickIn('credit', '1022');
	await waitFor(() =>
		expect((screen.getByTestId('conversion-credit-rate') as HTMLInputElement).value).toBe('0.8846')
	);
	await fireEvent.input(screen.getByTestId('conversion-debit-amount'), { target: { value: '1000' } });
	await fireEvent.input(screen.getByTestId('conversion-credit-amount'), { target: { value: '1078.40' } });
}

describe('ConversionAssistant', () => {
	beforeEach(() => {
		conversionMock.mockReset();
		held = null;
	});

	it('désactivé avec son motif tant que la saisie est incomplète', () => {
		mount();
		expect((screen.getByTestId('conversion-submit') as HTMLButtonElement).disabled).toBe(true);
		expect(screen.getByTestId('conversion-blocker-incomplete')).toBeTruthy();
	});

	it('aperçu : gain 6.95 sur le compte de la société ; corps sans completeDifference', async () => {
		await fill();
		expect(screen.getByTestId('conversion-preview-2-difference').textContent).toContain('6.95');
		conversionMock.mockResolvedValueOnce({ entries: [] });
		await fireEvent.click(screen.getByTestId('conversion-submit'));
		await waitFor(() => expect(conversionMock).toHaveBeenCalled());
		const body = conversionMock.mock.calls[0][0];
		expect(body).not.toHaveProperty('completeDifference');
		expect(body.entries[0].lines[0].foreign).toEqual({
			currency: 'EUR',
			amount: '1000',
			rate: '0.947',
			rateSource: 'afc'
		});
		expect(body.entries[1].lines[2]).toEqual({ accountId: 31, debit: '0', credit: '6.95' });
	});

	it('refus portant details.entry : affiché sur la colonne Crédit', async () => {
		await fill();
		conversionMock.mockRejectedValueOnce({
			code: 'FX_CURRENCY_MISMATCH',
			status: 400,
			message: 'serveur',
			details: { entry: 2, line: 1, expected: 'GBP' }
		});
		await fireEvent.click(screen.getByTestId('conversion-submit'));
		const err = await screen.findByTestId('conversion-credit-error');
		expect(err.textContent).toContain('tenu en GBP');
		expect(screen.queryByTestId('conversion-debit-error')).toBeNull();
	});

	it('refus sans details.entry (transit) : au-dessus de l’aperçu, saisie conservée', async () => {
		await fill();
		conversionMock.mockRejectedValueOnce({
			code: 'FX_CONVERSION_TRANSIT_UNBALANCED',
			status: 400,
			message: 'Le compte de conversion ne revient pas à zéro',
			details: { entry1: '947.00', entry2: '-900.00' }
		});
		await fireEvent.click(screen.getByTestId('conversion-submit'));
		const err = await screen.findByTestId('conversion-global-error');
		expect(err.textContent).toContain('ne revient pas à zéro');
		expect((screen.getByTestId('conversion-debit-amount') as HTMLInputElement).value).toBe('1000');
	});

	it('CHF saisi au lieu du cours : le cours s’en déduit', async () => {
		await fill();
		await fireEvent.input(screen.getByTestId('conversion-credit-chf'), { target: { value: '950.00' } });
		expect((screen.getByTestId('conversion-credit-rate') as HTMLInputElement).value).toBe('0.8809347181');
	});

	// ----- Story 53-1b : table K -----

	const DOUBLON = {
		code: 'JOURNAL_ENTRY_PROBABLE_DUPLICATE',
		status: 409,
		message: 'serveur',
		details: { journalEntryId: 7, entryNumber: 12, entryDate: '2026-03-05', confirmable: true, entry: 1 }
	};
	const fields = () => screen.getByTestId('conversion-fields') as HTMLFieldSetElement;

	it('K5 : un doublon probable fige la saisie, nomme la colonne et cite l’écriture ; « Passer » disparaît', async () => {
		await fill();
		conversionMock.mockRejectedValueOnce(DOUBLON);
		await fireEvent.click(screen.getByTestId('conversion-submit'));
		const avis = await screen.findByTestId('conversion-probable-duplicate');
		expect(avis.textContent).toContain('colonne Débit');
		expect(screen.getByTestId('conversion-duplicate-link').getAttribute('href')).toBe('/journal-entries/7');
		expect(screen.getByTestId('conversion-duplicate-link').textContent).toContain('Écriture n° 12 du 05.03.2026');
		expect(fields().disabled).toBe(true);
		expect(screen.queryByTestId('conversion-submit')).toBeNull();
		expect(screen.getByTestId('conversion-confirm-not-duplicate')).toBeTruthy();
		expect(screen.getByTestId('conversion-modify-entry')).toBeTruthy();
	});

	it('K5 : details.entry = 2 nomme la colonne Crédit', async () => {
		await fill();
		conversionMock.mockRejectedValueOnce({ ...DOUBLON, details: { ...DOUBLON.details, entry: 2 } });
		await fireEvent.click(screen.getByTestId('conversion-submit'));
		const avis = await screen.findByTestId('conversion-probable-duplicate');
		expect(avis.textContent).toContain('colonne Crédit');
		expect(avis.textContent).not.toContain('colonne Débit');
	});

	it('K5 → K2b : la confirmation renvoie le corps gardé avec confirmNotDuplicate, une seule fois', async () => {
		await fill();
		conversionMock.mockRejectedValueOnce(DOUBLON);
		await fireEvent.click(screen.getByTestId('conversion-submit'));
		const confirmer = await screen.findByTestId('conversion-confirm-not-duplicate');
		const first = conversionMock.mock.calls[0][0];
		conversionMock.mockReturnValueOnce(new Promise(() => {}));
		// Deux événements d'envoi : une requête.
		void fireEvent.click(confirmer);
		void fireEvent.click(confirmer);
		await waitFor(() => expect(conversionMock).toHaveBeenCalledTimes(2));
		await new Promise((r) => setTimeout(r, 20));
		expect(conversionMock).toHaveBeenCalledTimes(2);
		const second = conversionMock.mock.calls[1][0];
		expect(second.confirmNotDuplicate).toBe(true);
		expect(second.entries).toBe(first.entries);
		expect(first).not.toHaveProperty('confirmNotDuplicate');
		expect(fields().disabled).toBe(true);
	});

	it('K5 → K1 : « Modifier la saisie » rend la saisie, sans envoi', async () => {
		await fill();
		conversionMock.mockRejectedValueOnce(DOUBLON);
		await fireEvent.click(screen.getByTestId('conversion-submit'));
		await fireEvent.click(await screen.findByTestId('conversion-modify-entry'));
		expect(screen.queryByTestId('conversion-probable-duplicate')).toBeNull();
		expect(fields().disabled).toBe(false);
		expect(screen.getByTestId('conversion-submit')).toBeTruthy();
		expect(conversionMock).toHaveBeenCalledTimes(1);
	});

	it('K5 sans rang lisible → K7 : le message du serveur, saisie intacte, aucun bouton de confirmation', async () => {
		await fill();
		conversionMock.mockRejectedValueOnce({
			...DOUBLON,
			details: { journalEntryId: 7, entryNumber: 12, entryDate: '2026-03-05', confirmable: true }
		});
		await fireEvent.click(screen.getByTestId('conversion-submit'));
		const err = await screen.findByTestId('conversion-global-error');
		expect(err.textContent).toBe('serveur');
		expect(screen.queryByTestId('conversion-probable-duplicate')).toBeNull();
		expect(screen.queryByTestId('conversion-confirm-not-duplicate')).toBeNull();
		expect(fields().disabled).toBe(false);
		expect((screen.getByTestId('conversion-debit-amount') as HTMLInputElement).value).toBe('1000');
	});

	it('K6 : une réponse perdue fige, n’affiche aucun échec, offre le lien et la reprise ; la reprise n’envoie rien', async () => {
		await fill();
		conversionMock.mockRejectedValueOnce({ code: 'NETWORK_ERROR', status: 0, message: 'coupure' });
		await fireEvent.click(screen.getByTestId('conversion-submit'));
		const avis = await screen.findByTestId('conversion-unknown-outcome');
		expect(avis.textContent).toContain('La conversion a peut-être été passée');
		expect(document.body.textContent).not.toContain('n’a pas pu être passée');
		expect(screen.getByTestId('conversion-unknown-outcome-link').getAttribute('href')).toBe(
			'/journal-entries?dateFrom=2026-03-05&dateTo=2026-03-05'
		);
		expect(fields().disabled).toBe(true);
		expect(screen.queryByTestId('conversion-submit')).toBeNull();
		await fireEvent.click(screen.getByTestId('conversion-resume-entry'));
		expect(conversionMock).toHaveBeenCalledTimes(1);
		expect(fields().disabled).toBe(false);
		expect(screen.getByTestId('conversion-submit')).toBeTruthy();
	});

	it('K6 : un 5xx et une erreur qui n’est pas une ApiError sont aussi une issue inconnue', async () => {
		await fill();
		conversionMock.mockRejectedValueOnce({ code: 'INTERNAL_ERROR', status: 500, message: 'x' });
		await fireEvent.click(screen.getByTestId('conversion-submit'));
		await screen.findByTestId('conversion-unknown-outcome');
		await fireEvent.click(screen.getByTestId('conversion-resume-entry'));
		conversionMock.mockRejectedValueOnce(new Error('boom'));
		await fireEvent.click(screen.getByTestId('conversion-submit'));
		await screen.findByTestId('conversion-unknown-outcome');
		expect(screen.queryByTestId('conversion-global-error')).toBeNull();
	});

	it('K7 : un refus ordinaire laisse la saisie modifiable ; l’envoi suivant repart', async () => {
		await fill();
		conversionMock.mockRejectedValueOnce({ code: 'ENTRY_UNBALANCED', status: 400, message: 'x', details: { entry: 2 } });
		await fireEvent.click(screen.getByTestId('conversion-submit'));
		await screen.findByTestId('conversion-credit-error');
		expect(fields().disabled).toBe(false);
		conversionMock.mockResolvedValueOnce({ entries: [] });
		await fireEvent.click(screen.getByTestId('conversion-submit'));
		await waitFor(() => expect(conversionMock).toHaveBeenCalledTimes(2));
	});

	it('K2b : deux événements d’envoi, une requête ; la saisie est figée pendant le vol', async () => {
		await fill();
		conversionMock.mockReturnValueOnce(new Promise(() => {}));
		const bouton = screen.getByTestId('conversion-submit');
		void fireEvent.click(bouton);
		void fireEvent.click(bouton);
		await waitFor(() => expect(conversionMock).toHaveBeenCalledTimes(1));
		await new Promise((r) => setTimeout(r, 20));
		expect(conversionMock).toHaveBeenCalledTimes(1);
		expect(fields().disabled).toBe(true);
	});

	// ----- AC-9 : l'envoi attend les recherches de cours en vol -----

	it('AC-9 (a) : une recherche en vol se résout en K2a, et createConversion reçoit le cours mis à jour', async () => {
		await fill();
		held = { date: '2026-04-10', settle: () => {} };
		await fireEvent.input(screen.getByTestId('conversion-credit-date'), { target: { value: '2026-04-10' } });
		conversionMock.mockResolvedValueOnce({ entries: [] });
		await fireEvent.click(screen.getByTestId('conversion-submit'));
		// K2a : figé, rien n'est parti, la recherche est encore en vol.
		await new Promise((r) => setTimeout(r, 20));
		expect(fields().disabled).toBe(true);
		expect(conversionMock).not.toHaveBeenCalled();
		held.settle('rate');
		await waitFor(() => expect(conversionMock).toHaveBeenCalledTimes(1));
		const body = conversionMock.mock.calls[0][0];
		expect(body.entries[1].lines[0].foreign.rate).toBe('0.9');
	});

	it('AC-9 (b) : un plan devenu non prêt pendant l’attente → K1, rien n’est envoyé', async () => {
		await fill();
		held = { date: '2026-04-10', settle: () => {} };
		await fireEvent.input(screen.getByTestId('conversion-credit-date'), { target: { value: '2026-04-10' } });
		await fireEvent.click(screen.getByTestId('conversion-submit'));
		await new Promise((r) => setTimeout(r, 20));
		expect(fields().disabled).toBe(true);
		held.settle('none');
		await waitFor(() => expect(fields().disabled).toBe(false));
		expect(conversionMock).not.toHaveBeenCalled();
		expect(screen.getByTestId('conversion-submit')).toBeTruthy();
	});
});
