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

// Story 48-15 (#1332, AC6) — le pays d'un fournisseur neuf n'est plus coupé à deux
// caractères : « Suisse » part entier, le serveur le refuse en le disant, la saisie
// reste ; « CH » s'enregistre. NPA et localité renseignés : un bloc d'adresse vide
// est ignoré par le serveur, pays compris (#1342, hors périmètre).
describe('SupplierCreatePanel — le pays ne coupe pas un texte collé (#1332)', () => {
	const ADRESSE = { street: 'Rue des Essais', building: '7', postalCode: '1200', city: 'Genève' };

	function monter(onCreated = vi.fn()) {
		return render(SupplierCreatePanel, {
			initialName: 'Démo Fournisseur SA',
			initialAddress: { ...ADRESSE, country: '' },
			onCreated,
			onCancel: vi.fn(),
		});
	}

	// Rouge par M11 (`maxlength="2"` remis). Sonde positive d'abord : le champ est
	// trouvé (`getByTestId` lève sinon), puis l'attribut est absent.
	it('pays : aucun maxlength', () => {
		const champ = monter().getByTestId('supplier-new-country');
		expect(champ.tagName).toBe('INPUT');
		expect(champ.hasAttribute('maxlength')).toBe(false);
	});

	it('« Suisse » part entier, le refus du serveur s’affiche, la saisie reste ; « CH » s’enregistre', async () => {
		const refus = 'adresse : le pays doit être un code ISO à 2 lettres (ex. CH)';
		createContactMock.mockRejectedValueOnce({ code: 'VALIDATION_ERROR', status: 400, message: refus });
		const onCreated = vi.fn();
		const { getByTestId } = monter(onCreated);
		const champ = getByTestId('supplier-new-country') as HTMLInputElement;
		await fireEvent.input(champ, { target: { value: 'Suisse' } });
		await fireEvent.click(getByTestId('supplier-new-create'));
		await waitFor(() => expect(getByTestId('supplier-new-error').textContent).toBe(refus));
		expect(createContactMock).toHaveBeenCalledWith(
			expect.objectContaining({ addressStructured: expect.objectContaining({ country: 'Suisse' }) }),
		);
		expect(champ.value).toBe('Suisse');
		expect(onCreated).not.toHaveBeenCalled();

		const cree = { id: 7, name: 'Démo Fournisseur SA' };
		createContactMock.mockResolvedValueOnce(cree);
		await fireEvent.input(champ, { target: { value: 'CH' } });
		await fireEvent.click(getByTestId('supplier-new-create'));
		await waitFor(() => expect(onCreated).toHaveBeenCalledWith(cree));
		expect(createContactMock).toHaveBeenLastCalledWith(
			expect.objectContaining({ addressStructured: expect.objectContaining({ country: 'CH' }) }),
		);
	});
});
