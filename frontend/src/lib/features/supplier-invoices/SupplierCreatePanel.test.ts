// Story v0174-correctifs-3 (#817, AC-3.15) — le panneau de création d'un fournisseur affiche
// tel quel le refus du serveur pour un pays inconnu (« XX ») : le chemin existait, ce test
// le fige. Données inventées.
import { describe, it, expect, vi, afterEach } from 'vitest';
import { render, cleanup, fireEvent, waitFor } from '@testing-library/svelte';

vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_k: string, fallback: string) => fallback,
}));
const createContactMock = vi.hoisted(() => vi.fn());
vi.mock('$lib/features/contacts/contacts.api', () => ({
	createContact: (p: unknown) => createContactMock(p),
	listContacts: vi.fn(async () => ({ items: [], total: 0, offset: 0, limit: 20 })),
}));

import SupplierCreatePanel from './SupplierCreatePanel.svelte';

afterEach(() => {
	cleanup();
	vi.clearAllMocks();
});

describe('SupplierCreatePanel — refus du pays (#817)', () => {
	it('le message du serveur s’affiche tel quel', async () => {
		const refus =
			'adresse : le pays « XX » n’est pas un code ISO 3166-1 alpha-2 connu (ex. CH, FR, DE)';
		createContactMock.mockRejectedValue({ code: 'VALIDATION_ERROR', status: 400, message: refus });
		const { getByTestId } = render(SupplierCreatePanel, {
			initialName: 'Fournisseur XX',
			initialAddress: {
				street: 'Rue des Essais',
				building: '7',
				postalCode: '1200',
				city: 'Genève',
				country: 'XX',
			},
			onCreated: vi.fn(),
			onCancel: vi.fn(),
		});
		await fireEvent.click(getByTestId('supplier-new-create'));
		await waitFor(() => expect(getByTestId('supplier-new-error').textContent).toBe(refus));
		expect(createContactMock).toHaveBeenCalledWith(
			expect.objectContaining({ addressStructured: expect.objectContaining({ country: 'XX' }) }),
		);
	});
});
