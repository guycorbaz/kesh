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
