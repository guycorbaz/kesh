// Story 28-2 (#520, AC9 — A3 et `handleDelete`) — la liste des profils
// bancaires affiche le message d'un refus, sans préfixe ni « [object Object] » ;
// un refus de suppression est notifié, et la liste reste affichée.

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, fireEvent } from '@testing-library/svelte';

const { listMock, deleteMock, notifyErrorMock } = vi.hoisted(() => ({
	listMock: vi.fn(),
	deleteMock: vi.fn(),
	notifyErrorMock: vi.fn(),
}));

vi.mock('./bank-profile.api', () => ({
	listBankProfiles: () => listMock(),
	deleteBankProfile: (id: number) => deleteMock(id),
}));
vi.mock('$lib/shared/utils/notify', () => ({ notifyError: notifyErrorMock }));

import BankProfileList from './BankProfileList.svelte';

const PROFIL = {
	id: 3,
	bankName: 'Banque Test',
	filenamePattern: null,
	encoding: null,
};

describe('BankProfileList — erreurs (Story 28-2, A3)', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		vi.spyOn(console, 'error').mockImplementation(() => {});
	});

	afterEach(() => {
		vi.unstubAllGlobals();
	});

	it('chargement refusé par une ApiError → son message, sans préfixe', async () => {
		listMock.mockRejectedValue({ code: 'FORBIDDEN', message: 'Accès refusé.', status: 403 });
		const { findByText, container } = render(BankProfileList);
		expect(await findByText('Accès refusé.')).toBeTruthy();
		expect(container.textContent).not.toContain('[object Object]');
		expect(container.textContent).not.toContain('Erreur chargement profils');
	});

	it('chargement en échec imprévu → le générique', async () => {
		listMock.mockRejectedValue(new Error('boom'));
		const { findByText, container } = render(BankProfileList);
		expect(await findByText('Erreur inattendue.')).toBeTruthy();
		expect(container.textContent).not.toContain('boom');
	});

	it('suppression refusée → notifyError avec le message, la liste reste affichée', async () => {
		listMock.mockResolvedValue({ items: [PROFIL] });
		deleteMock.mockRejectedValue({ code: 'CONFLICT', message: 'Profil utilisé par un import.', status: 409 });
		// jsdom n'implémente pas `window.confirm` (patron `RulesList.test.ts`).
		vi.stubGlobal('confirm', () => true);
		const { findByTestId, findAllByTestId } = render(BankProfileList);
		await fireEvent.click(await findByTestId('bank-import-profile-delete-button'));
		await vi.waitFor(() => expect(notifyErrorMock).toHaveBeenCalledWith('Profil utilisé par un import.'));
		expect(deleteMock).toHaveBeenCalledWith(3);
		expect(await findAllByTestId('bank-import-profile-list-row')).toHaveLength(1);
	});
});
