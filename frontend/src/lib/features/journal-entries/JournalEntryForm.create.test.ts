// Story 15-12b (#543 ; revue de code P1, E-3) — le formulaire en MODE CRÉATION
// face au filet sous un bilan clos. `LATER_FISCAL_YEAR_CLOSED` n'est pas nommé
// dans le `switch` d'erreurs de `JournalEntryForm.svelte` : il y retombe sur le
// `default`, qui affiche le message du serveur. Ce test écrit cette intention —
// le message qui nomme l'exercice postérieur clos et la réparation — et le
// formulaire reste ouvert (ni `onSuccess`, ni `onStale`).

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, fireEvent, screen } from '@testing-library/svelte';

const { createMock, toastError, notifyMock } = vi.hoisted(() => ({
	createMock: vi.fn(),
	toastError: vi.fn(),
	notifyMock: vi.fn(() => false)
}));

vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string) => fallback
}));
vi.mock('$lib/features/onboarding/onboarding.svelte', () => ({
	i18nMsg: (_key: string, fallback: string) => fallback
}));
vi.mock('./journal-entries.api', () => ({
	createJournalEntry: createMock,
	updateJournalEntry: vi.fn()
}));
vi.mock('svelte-sonner', () => ({
	toast: { error: toastError, success: vi.fn() }
}));
vi.mock('$lib/shared/utils/notify', () => ({
	notifyMissingFiscalYearOrFallback: notifyMock
}));

import JournalEntryForm from './JournalEntryForm.svelte';
import type { AccountResponse } from '$lib/features/accounts/accounts.types';
import { pinClock } from '$lib/shared/utils/pin-clock.test-helper';

function account(id: number, number: string): AccountResponse {
	return {
		id,
		companyId: 1,
		number,
		name: `Compte ${number}`,
		accountType: 'Asset',
		parentId: null,
		active: true,
		postable: true,
		version: 1,
		createdAt: '2026-01-01T00:00:00',
		updatedAt: '2026-01-01T00:00:00'
	} as AccountResponse;
}

/** Choisit le compte `number` dans le sélecteur d'indice `index`. */
async function pick(index: number, number: string) {
	const input = screen.getAllByRole('textbox', { name: 'Compte' })[index];
	await fireEvent.focus(input);
	const option = screen
		.getAllByRole('option')
		.find((o) => (o.textContent ?? '').includes(number));
	if (!option) throw new Error(`option ${number} introuvable`);
	await fireEvent.mouseDown(option);
	await fireEvent.blur(input);
}

describe('JournalEntryForm — mode création', () => {
	beforeEach(() => {
		createMock.mockReset();
		toastError.mockReset();
		notifyMock.mockClear();
	});

	it('LATER_FISCAL_YEAR_CLOSED : toast du message serveur (repli `default`), formulaire ouvert', async () => {
		const serveur =
			'L’exercice « Exercice 2027 » est clôturé : aucune écriture datée avant sa date de début ne peut être enregistrée, modifiée ni supprimée.';
		createMock.mockRejectedValueOnce({
			code: 'LATER_FISCAL_YEAR_CLOSED',
			status: 400,
			message: serveur
		});
		const onSuccess = vi.fn();
		const onStale = vi.fn();
		render(JournalEntryForm, {
			props: {
				accounts: [account(1, '1000'), account(2, '2000')],
				accountsLoadError: false,
				onSuccess,
				onCancel: vi.fn(),
				onStale
			}
		});

		await fireEvent.input(document.getElementById('entry-description')!, {
			target: { value: 'Saisie sous un bilan clos' }
		});
		await pick(0, '1000');
		await pick(1, '2000');
		const montants = screen.getAllByPlaceholderText('0.00');
		// Ligne 1 : débit ; ligne 2 : crédit (ordre débit, crédit par ligne).
		await fireEvent.input(montants[0], { target: { value: '100.00' } });
		await fireEvent.input(montants[3], { target: { value: '100.00' } });

		await fireEvent.click(screen.getByRole('button', { name: 'Valider' }));
		await new Promise((r) => setTimeout(r, 0));

		expect(createMock).toHaveBeenCalledTimes(1);
		expect(toastError).toHaveBeenCalledWith(serveur);
		expect(onSuccess).not.toHaveBeenCalled();
		expect(onStale).not.toHaveBeenCalled();
	});
});

// Story 36-3 (#1051, AC4) — le montant ambigu « 1.234 » refusé à l'écran, l'apostrophe
// des milliers admise. Les deux lignes portent un compte imputable (V1-4 de la
// validation) : sans compte, la ligne serait `partial` pour une autre raison, et
// « Valider désactivé » ne prouverait rien. Le cas témoin « 1.23 » montre que,
// comptes choisis, le bouton s'active bien.
describe('JournalEntryForm — montant ambigu (Story 36-3)', () => {
	beforeEach(() => {
		createMock.mockReset();
		createMock.mockResolvedValue({});
	});

	async function saisir(debit: string, credit: string) {
		render(JournalEntryForm, {
			props: {
				accounts: [account(1, '1000'), account(2, '2000')],
				accountsLoadError: false,
				onSuccess: vi.fn(),
				onCancel: vi.fn(),
				onStale: vi.fn()
			}
		});
		await fireEvent.input(document.getElementById('entry-description')!, {
			target: { value: 'Montant ambigu' }
		});
		await pick(0, '1000');
		await pick(1, '2000');
		const montants = screen.getAllByPlaceholderText('0.00');
		await fireEvent.input(montants[0], { target: { value: debit } });
		await fireEvent.input(montants[3], { target: { value: credit } });
		return montants;
	}

	const valider = () => screen.getByRole('button', { name: 'Valider' }) as HTMLButtonElement;

	it('témoin : « 1.23 » des deux côtés, comptes choisis → Valider actif', async () => {
		await saisir('1.23', '1.23');
		expect(valider().disabled).toBe(false);
		expect(screen.queryByText(/Montant ambigu/)).toBeNull();
	});

	it('« 1.234 » des deux côtés → message ambigu, Valider désactivé, Ctrl+S sans envoi', async () => {
		const montants = await saisir('1.234', '1.234');
		expect(screen.getAllByText(/Montant ambigu/).length).toBeGreaterThan(0);
		expect(screen.queryByText(/Montant illisible/)).toBeNull();
		// `classList` et non `className` : la classe de base porte
		// `aria-invalid:border-destructive`, qu'un `toContain` prendrait pour la bordure.
		expect(montants[0].classList.contains('border-destructive')).toBe(true);
		expect(montants[3].classList.contains('border-destructive')).toBe(true);
		expect(valider().disabled).toBe(true);
		await fireEvent.keyDown(window, { key: 's', ctrlKey: true });
		await new Promise((r) => setTimeout(r, 0));
		expect(createMock).not.toHaveBeenCalled();
	});

	// Revue P1 (A3) — sur une même ligne, un débit ambigu et un crédit illisible : seul le
	// message ambigu s'affiche (prioritaire), et rien ne part.
	it('même ligne : débit « 1.234 », crédit « 12’34 » → message ambigu seul', async () => {
		const montants = await saisir('', '');
		// montants[0], [1] : débit et crédit de la ligne 1.
		await fireEvent.input(montants[0], { target: { value: '1.234' } });
		await fireEvent.input(montants[1], { target: { value: "12'34" } });
		expect(screen.getAllByText(/Montant ambigu/)).toHaveLength(1);
		expect(screen.queryByText(/Montant illisible/)).toBeNull();
		expect(montants[0].classList.contains('border-destructive')).toBe(true);
		expect(montants[1].classList.contains('border-destructive')).toBe(true);
		await fireEvent.keyDown(window, { key: 's', ctrlKey: true });
		await new Promise((r) => setTimeout(r, 0));
		expect(createMock).not.toHaveBeenCalled();
	});

	it('« 12’34 » → message illisible', async () => {
		await saisir("12'34", "12'34");
		expect(screen.getAllByText(/Montant illisible/).length).toBeGreaterThan(0);
		expect(screen.queryByText(/Montant ambigu/)).toBeNull();
		expect(valider().disabled).toBe(true);
	});

	it('« 1’234,50 » des deux côtés → équilibré, envoyé en 1234.50', async () => {
		const montants = await saisir("1'234,50", "1'234,50");
		expect(screen.getByText(/Équilibré/)).toBeTruthy();
		expect(montants[0].classList.contains('border-destructive')).toBe(false);
		expect(montants[3].classList.contains('border-destructive')).toBe(false);
		expect(valider().disabled).toBe(false);
		await fireEvent.click(valider());
		await new Promise((r) => setTimeout(r, 0));
		expect(createMock).toHaveBeenCalledTimes(1);
		const payload = createMock.mock.calls[0][0] as {
			lines: Array<{ accountId: number; debit: string; credit: string }>;
		};
		expect(payload.lines).toEqual([
			expect.objectContaining({ accountId: 1, debit: '1234.50', credit: '0' }),
			expect.objectContaining({ accountId: 2, debit: '0', credit: '1234.50' })
		]);
	});
});

// Story v0174-correctifs-3 (#889, AC-3.6) — la date par défaut d'une écriture est celle du
// poste (déjà locale avant la 0.17.4 : ce test fige le remplacement par `todayLocal`).
describe('JournalEntryForm — date par défaut locale (#889)', () => {
	pinClock();
	it('à 00 h 30 le 5 novembre à Zurich, l’écriture est datée du 5', () => {
		render(JournalEntryForm, {
			props: {
				accounts: [account(1, '1000')],
				accountsLoadError: false,
				onSuccess: vi.fn(),
				onCancel: vi.fn(),
				onStale: vi.fn()
			}
		});
		expect((document.getElementById('entry-date') as HTMLInputElement).value).toBe('2026-11-05');
	});
});
