// Story 53-1a (D8, revue P1 E1) — la modale « Remplacer le brouillon ? » de l'assistant TVA,
// hors du `<fieldset>`, ne doit ni laisser partir la saisie par Ctrl+S ni la réécrire une
// fois l'envoi parti (J2b, J5, J6) : l'écran montrerait une écriture, la confirmation en
// enverrait une autre. L'assistant est remplacé par un bouton qui rend un résultat fixe.

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, fireEvent, screen, cleanup } from '@testing-library/svelte';

const { createMock, toastError } = vi.hoisted(() => ({ createMock: vi.fn(), toastError: vi.fn() }));

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
vi.mock('svelte-sonner', () => ({ toast: { error: toastError, success: vi.fn() } }));
vi.mock('$lib/shared/utils/notify', () => ({
	notifyMissingFiscalYearOrFallback: vi.fn(() => false)
}));
vi.mock('$lib/shared/utils/local-date', () => ({ todayLocal: () => '2026-03-15' }));
vi.mock('./VatPurchaseAssistant.svelte', async () => ({
	default: (await import('./test-stubs/VatAssistantStub.svelte')).default
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
		role: null,
		postable: true,
		letterable: false,
		version: 1,
		createdAt: '',
		updatedAt: ''
	};
}

const tick = () => new Promise((r) => setTimeout(r, 0));

async function pick(index: number, number: string) {
	const input = screen.getAllByRole('textbox', { name: 'Compte' })[index];
	await fireEvent.focus(input);
	const option = screen.getAllByRole('option').find((o) => (o.textContent ?? '').includes(number));
	if (!option) throw new Error(`option ${number} introuvable`);
	await fireEvent.mouseDown(option);
	await fireEvent.blur(input);
}

/** Une saisie valide « Vente 1 », puis la modale de remplacement ouverte par l'assistant. */
async function saisieAvecModale() {
	render(JournalEntryForm, {
		props: {
			accounts: [account(1, '1000'), account(2, '2000')],
			accountsLoadError: false,
			onSuccess: vi.fn(),
			onCancel: vi.fn()
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
	await fireEvent.click(screen.getByTestId('stub-vat-apply'));
	expect(screen.getByRole('dialog')).toBeTruthy();
}

beforeEach(() => {
	createMock.mockReset();
	toastError.mockReset();
});
afterEach(() => cleanup());

describe('JournalEntryForm — la modale de remplacement de l’assistant TVA et le gel', () => {
	it('Ctrl+S, modale ouverte, n’envoie rien', async () => {
		await saisieAvecModale();
		await fireEvent.keyDown(window, { key: 's', ctrlKey: true });
		await tick();
		expect(createMock).not.toHaveBeenCalled();
		expect(screen.getByRole('dialog')).toBeTruthy();
	});

	it('envoi parti, modale ouverte : la modale se ferme et « Remplacer » ne réécrit pas la saisie figée (J5)', async () => {
		await saisieAvecModale();
		createMock.mockRejectedValueOnce({
			code: 'JOURNAL_ENTRY_PROBABLE_DUPLICATE',
			status: 409,
			message: 'serveur',
			details: { journalEntryId: 7, entryNumber: 12, entryDate: '2026-03-15', confirmable: true }
		});
		await fireEvent.click(screen.getByRole('button', { name: 'Valider' }));
		await tick();
		expect(screen.getByTestId('journal-entry-probable-duplicate')).toBeTruthy();
		expect(screen.queryByRole('dialog')).toBeNull();
		// La saisie figée montre toujours l'écriture envoyée, non celle de l'assistant.
		expect((document.getElementById('entry-description') as HTMLInputElement).value).toBe('Vente 1');
		expect((screen.getAllByPlaceholderText('0.00')[0] as HTMLInputElement).value).toBe('100.00');
	});
});
