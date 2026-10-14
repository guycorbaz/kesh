// Story 29-9 (#646, AC7) — la suppression d'une personne de contact demande confirmation :
// « Supprimer » l'archive, et aucun écran ne la rétablit. Refus → aucun appel ; acceptation →
// suppression, puis liste relue.

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, fireEvent } from '@testing-library/svelte';
import type { ContactPerson } from './contact-persons.api';

const { listMock, deleteMock } = vi.hoisted(() => ({ listMock: vi.fn(), deleteMock: vi.fn() }));

vi.mock('./contact-persons.api', () => ({
	listContactPersons: (contactId: number) => listMock(contactId),
	createContactPerson: vi.fn(),
	deleteContactPerson: (id: number) => deleteMock(id),
}));

// Le mock rend le repli, variables `{ $x }` interpolées.
vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string, args?: Record<string, string | number>) =>
		args ? fallback.replace(/\{\s*\$(\w+)\s*\}/g, (_, k) => String(args[k] ?? '')) : fallback,
}));

import ContactPersonsManager from './ContactPersonsManager.svelte';

const PERSONNE: ContactPerson = {
	id: 31,
	contactId: 7,
	firstName: 'Anna',
	lastName: 'Muster',
	role: null,
	email: null,
	phone: null,
	version: 1,
};

describe('ContactPersonsManager — confirmation de la suppression (Story 29-9, #646)', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		listMock.mockResolvedValue([PERSONNE]);
		deleteMock.mockResolvedValue(undefined);
	});

	afterEach(() => {
		vi.unstubAllGlobals();
	});

	it('V10 — refus ⇒ aucune suppression, la personne reste listée ; la question la nomme (M11)', async () => {
		const confirmMock = vi.fn(() => false);
		// jsdom n'implémente pas `window.confirm` (patron `BankProfileList.test.ts`).
		vi.stubGlobal('confirm', confirmMock);
		const { findByTestId, getByText } = render(ContactPersonsManager, { contactId: 7 });
		await fireEvent.click(await findByTestId('contact-person-delete-31'));
		expect(confirmMock).toHaveBeenCalledTimes(1);
		expect(confirmMock.mock.calls[0][0]).toContain('Anna Muster');
		expect(deleteMock).not.toHaveBeenCalled();
		expect(getByText('Anna Muster')).toBeTruthy();
		expect(listMock).toHaveBeenCalledTimes(1);
	});

	it('V11 — acceptation ⇒ suppression de la personne, puis liste relue (M12)', async () => {
		const confirmMock = vi.fn(() => true);
		vi.stubGlobal('confirm', confirmMock);
		listMock.mockResolvedValueOnce([PERSONNE]).mockResolvedValue([]);
		const { findByTestId } = render(ContactPersonsManager, { contactId: 7 });
		await fireEvent.click(await findByTestId('contact-person-delete-31'));
		expect(confirmMock).toHaveBeenCalledTimes(1);
		await vi.waitFor(() => expect(deleteMock).toHaveBeenCalledWith(31));
		await vi.waitFor(() => expect(listMock).toHaveBeenCalledTimes(2));
		expect(await findByTestId('contact-persons-empty')).toBeTruthy();
	});
});
