// Story 26-2b (#622, AC9, site D9) — *Paramètres* nomme chaque compte bancaire
// avec sa description. La réponse simulée de `/api/v1/companies/current` est
// typée `CompanyCurrentResponse` : `description` y est obligatoire, `npm run
// check` refuse une simulation qui l'omet.
//
// Le nom du fichier n'est pas `+page.test.ts` : SvelteKit réserve le préfixe `+`.

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, fireEvent } from '@testing-library/svelte';
import type { CompanyCurrentResponse } from '$lib/features/settings/settings.types';
import { PENDING_FIELD_CLASS } from '$lib/features/settings/company-conflict';

vi.mock('$app/environment', () => ({ browser: true }));
vi.mock('$lib/features/onboarding/onboarding.svelte', () => ({
	i18nMsg: (_key: string, fallback: string) => fallback,
}));
const notifySuccessMock = vi.fn();
vi.mock('$lib/shared/utils/notify', () => ({
	notifySuccess: (...a: unknown[]) => notifySuccessMock(...a),
}));
const fetchCompanyCurrentMock = vi.fn<() => Promise<CompanyCurrentResponse>>();
const lockBooksMock = vi.fn();
const releaseBooksLockMock = vi.fn();
// Story v0172-session (AC-8) : la fabrique remplace le module entier — elle
// reçoit la lecture et l'enregistrement du réglage de la durée d'inactivité.
const fetchSessionSettingsMock = vi.fn();
const updateSessionSettingsMock = vi.fn();
const updateCompanyAddressMock = vi.fn();
const updateCompanyContactDetailsMock = vi.fn();
const updateCompanyEmailMock = vi.fn();
vi.mock('$lib/features/settings/settings.api', () => ({
	fetchCompanyCurrent: () => fetchCompanyCurrentMock(),
	updateCompanyAddress: (...a: unknown[]) => updateCompanyAddressMock(...a),
	updateCompanyContactDetails: (...a: unknown[]) => updateCompanyContactDetailsMock(...a),
	updateCompanyEmail: (...a: unknown[]) => updateCompanyEmailMock(...a),
	lockBooks: (...a: unknown[]) => lockBooksMock(...a),
	releaseBooksLock: (...a: unknown[]) => releaseBooksLockMock(...a),
	fetchSessionSettings: (...a: unknown[]) => fetchSessionSettingsMock(...a),
	updateSessionSettings: (...a: unknown[]) => updateSessionSettingsMock(...a),
}));

import Page from './+page.svelte';
import { authState } from '$lib/app/stores/auth.svelte';

// Story v0172-session (0.17.3, CR #891, AC-8) — la section « Session » n'est
// rendue que pour l'Administrateur ; les autres rôles ne lisent pas le réglage.
describe('page Paramètres — section Session (Story v0172-session)', () => {
	beforeEach(() => {
		// Revue P3 (P3-B3, E3-3, A3-3) : `resetAllMocks` vide aussi les files `…Once` et les
		// implémentations posées par un test, que `clearAllMocks` laissait au test suivant.
		vi.resetAllMocks();
		fetchSessionSettingsMock.mockResolvedValue({
			sessionIdleMinutes: 60,
			version: 0,
			min: 1,
			max: 1440,
		});
	});

	it('Administrateur : la section est rendue et lit le réglage', async () => {
		authState.login(
			{ userId: '1', username: 'admin', role: 'Admin', expiresIn: 900 },
			{ broadcast: false },
		);
		fetchCompanyCurrentMock.mockResolvedValue(reponse(null));
		const { findByTestId } = render(Page);
		await findByTestId('settings-session');
		expect(fetchSessionSettingsMock).toHaveBeenCalledTimes(1);
		authState.clearSession({ broadcast: false });
	});

	for (const role of ['Comptable', 'Consultation']) {
		it(`${role} : section absente, aucun GET du réglage`, async () => {
			authState.login({ userId: '2', username: 'u', role, expiresIn: 900 }, { broadcast: false });
			fetchCompanyCurrentMock.mockResolvedValue(reponse(null));
			const { findByTestId, queryByTestId } = render(Page);
			await findByTestId('settings-books-lock');
			expect(queryByTestId('settings-session')).toBeNull();
			expect(fetchSessionSettingsMock).not.toHaveBeenCalled();
			authState.clearSession({ broadcast: false });
		});
	}
});

function reponse(description: string | null): CompanyCurrentResponse {
	return {
		company: {
			id: 1,
			name: 'Test SA',
			address: 'Rue Test 1\n1000 Lausanne',
			ideNumber: null,
			orgType: 'Pme',
			accountingLanguage: 'FR',
			instanceLanguage: 'FR',
			version: 1,
		} as unknown as CompanyCurrentResponse['company'],
		bankAccounts: [
			{
				id: 7,
				bankName: 'UBS',
				description,
				iban: 'CH9300762011623852957',
				qrIban: null,
				isPrimary: true,
			},
		],
	};
}

describe('page Paramètres — nom des comptes bancaires (Story 26-2b, D9)', () => {
	beforeEach(() => {
		// Revue P3 (P3-B3, E3-3, A3-3) : `resetAllMocks` vide aussi les files `…Once` et les
		// implémentations posées par un test, que `clearAllMocks` laissait au test suivant.
		vi.resetAllMocks();
	});

	it('avec description : « UBS — Personnel »', async () => {
		fetchCompanyCurrentMock.mockResolvedValue(reponse('Personnel'));
		const { findByTestId } = render(Page);
		expect((await findByTestId('settings-bank-name-7')).textContent).toBe('UBS — Personnel');
	});

	it('sans description : la banque seule', async () => {
		fetchCompanyCurrentMock.mockResolvedValue(reponse(null));
		const { findByTestId } = render(Page);
		expect((await findByTestId('settings-bank-name-7')).textContent).toBe('UBS');
	});
});

// Story 28-2 (#520, AC9 — A1/A2) — un refus du verrou de période s'affiche par
// son message, jamais « [object Object] » ; une panne imprévue par le générique.
describe('page Paramètres — erreurs du verrou de période (Story 28-2, A1/A2)', () => {
	const REFUS = {
		code: 'VALIDATION_ERROR',
		message: 'La date doit suivre le verrou actuel.',
		status: 400,
	};

	beforeEach(() => {
		// Revue P3 (P3-B3, E3-3, A3-3) : `resetAllMocks` vide aussi les files `…Once` et les
		// implémentations posées par un test, que `clearAllMocks` laissait au test suivant.
		vi.resetAllMocks();
		vi.spyOn(console, 'error').mockImplementation(() => {});
	});

	function verrouillee(): CompanyCurrentResponse {
		const r = reponse(null);
		(r.company as unknown as { booksLockedThrough: string }).booksLockedThrough = '2026-03-31';
		return r;
	}

	async function verrouiller(rejet: unknown): Promise<HTMLElement> {
		fetchCompanyCurrentMock.mockResolvedValue(reponse(null));
		lockBooksMock.mockRejectedValue(rejet);
		const { findByTestId, container } = render(Page);
		await findByTestId('settings-books-lock');
		const date = container.querySelector('#books-lock-through') as HTMLInputElement;
		await fireEvent.input(date, { target: { value: '2026-06-30' } });
		await fireEvent.click(await findByTestId('books-lock-submit'));
		const erreur = await findByTestId('books-lock-error');
		expect(lockBooksMock).toHaveBeenCalledTimes(1);
		return erreur;
	}

	async function deverrouiller(rejet: unknown): Promise<HTMLElement> {
		fetchCompanyCurrentMock.mockResolvedValue(verrouillee());
		releaseBooksLockMock.mockRejectedValue(rejet);
		const { findByTestId, container } = render(Page);
		await fireEvent.click(await findByTestId('books-lock-release-toggle'));
		// Le motif est saisi : sans lui, `submitUnlock` sort avant tout appel
		// et le test passerait sans exercer A2.
		const motif = container.querySelector('#books-unlock-motif') as HTMLInputElement;
		await fireEvent.input(motif, { target: { value: 'Correction' } });
		await fireEvent.click(await findByTestId('books-lock-release-submit'));
		const erreur = await findByTestId('books-lock-error');
		expect(releaseBooksLockMock).toHaveBeenCalledTimes(1);
		return erreur;
	}

	it('A1 — verrouillage refusé par une ApiError → son message', async () => {
		const erreur = await verrouiller(REFUS);
		expect(erreur.textContent).toBe(REFUS.message);
		expect(document.body.textContent).not.toContain('[object Object]');
	});

	it('A1 — verrouillage en échec imprévu → le générique, pas le message technique', async () => {
		const erreur = await verrouiller(new Error('boom'));
		expect(erreur.textContent).toBe('Erreur inattendue.');
		expect(erreur.textContent).not.toContain('boom');
	});

	it('A2 — déverrouillage refusé par une ApiError → son message', async () => {
		const erreur = await deverrouiller(REFUS);
		expect(erreur.textContent).toBe(REFUS.message);
		expect(document.body.textContent).not.toContain('[object Object]');
	});

	it('A2 — déverrouillage en échec imprévu → le générique', async () => {
		const erreur = await deverrouiller(new Error('boom'));
		expect(erreur.textContent).toBe('Erreur inattendue.');
	});
});

// Story 33-5 (#909, AC-9) — corriger l'adresse de la société.
describe('page Paramètres — adresse de la société (Story 33-5)', () => {
	const structured = {
		street: 'Rue Test',
		building: '1',
		postalCode: '1000',
		city: 'Lausanne',
		country: 'SU',
	};

	function societe(overrides: Record<string, unknown> = {}): CompanyCurrentResponse {
		const base = reponse(null);
		return {
			...base,
			company: {
				...base.company,
				addressStructured: structured,
				addressCountryKnown: true,
				...overrides,
			} as CompanyCurrentResponse['company'],
		};
	}

	function connecter(role: string) {
		authState.login({ userId: '1', username: 'u', role, expiresIn: 900 }, { broadcast: false });
	}

	beforeEach(() => {
		// Revue P3 (P3-B3, E3-3, A3-3) : `resetAllMocks` vide aussi les files `…Once` et les
		// implémentations posées par un test, que `clearAllMocks` laissait au test suivant.
		vi.resetAllMocks();
		fetchSessionSettingsMock.mockResolvedValue({
			sessionIdleMinutes: 60,
			version: 0,
			min: 1,
			max: 1440,
		});
	});

	it('Administrateur : préremplit depuis les composants et envoie la version', async () => {
		connecter('Admin');
		fetchCompanyCurrentMock.mockResolvedValue(societe());
		updateCompanyAddressMock.mockResolvedValue({
			...societe().company,
			address: 'Rue Test 1\n1000 Lausanne\nCH',
			version: 2,
		});
		const { findByTestId, getByTestId, queryByTestId } = render(Page);
		await fireEvent.click(await findByTestId('settings-company-address-edit'));

		const postal = getByTestId('settings-company-address-postal-code-input') as HTMLInputElement;
		expect(postal.value).toBe('1000');
		const country = getByTestId('settings-company-address-country-input') as HTMLInputElement;
		expect(country.value).toBe('SU');
		await fireEvent.input(country, { target: { value: 'CH' } });
		await fireEvent.click(getByTestId('settings-company-address-save'));

		await vi.waitFor(() => expect(updateCompanyAddressMock).toHaveBeenCalledTimes(1));
		expect(updateCompanyAddressMock).toHaveBeenCalledWith({
			address: { ...structured, country: 'CH' },
			version: 1,
		});
		await vi.waitFor(() =>
			expect(queryByTestId('settings-company-address-country-input')).toBeNull(),
		);
		authState.clearSession({ broadcast: false });
	});

	it('une société sans composants part de CH, le serveur en décide', async () => {
		connecter('Admin');
		fetchCompanyCurrentMock.mockResolvedValue(
			societe({
				addressStructured: { street: '', building: '', postalCode: '', city: '', country: '' },
			}),
		);
		const { findByTestId, getByTestId } = render(Page);
		await fireEvent.click(await findByTestId('settings-company-address-edit'));
		const country = getByTestId('settings-company-address-country-input') as HTMLInputElement;
		expect(country.value).toBe('CH');
		authState.clearSession({ broadcast: false });
	});

	it('affiche le refus du serveur et garde le formulaire ouvert', async () => {
		connecter('Admin');
		fetchCompanyCurrentMock.mockResolvedValue(societe());
		updateCompanyAddressMock.mockRejectedValue({
			code: 'VALIDATION_ERROR',
			message: 'adresse : le pays « XX » est inconnu',
			status: 400,
		});
		const { findByTestId, getByTestId } = render(Page);
		await fireEvent.click(await findByTestId('settings-company-address-edit'));
		await fireEvent.click(getByTestId('settings-company-address-save'));
		const error = await findByTestId('settings-company-address-error');
		expect(error.textContent).toContain('XX');
		expect(getByTestId('settings-company-address-country-input')).toBeTruthy();
		authState.clearSession({ broadcast: false });
	});

	// Story 34-6 (#953, AC-1) — récrit : ce test affirmait le défaut (« le second essai
	// part avec la version rechargée » et réussit, sans rien montrer de l'écriture
	// concurrente). La ville relue est maintenant affichée sous son champ, la saisie
	// reste, et rien ne part avant le second clic.
	it('un conflit de version montre la valeur enregistrée entre-temps avant de la remplacer', async () => {
		connecter('Admin');
		fetchCompanyCurrentMock.mockResolvedValueOnce(societe());
		fetchCompanyCurrentMock.mockResolvedValueOnce(
			societe({ version: 9, addressStructured: { ...structured, city: 'Nyon' } }),
		);
		updateCompanyAddressMock.mockRejectedValueOnce({
			code: 'OPTIMISTIC_LOCK_CONFLICT',
			message: 'conflit',
			status: 409,
		});
		const { findByTestId, getByTestId } = render(Page);
		await fireEvent.click(await findByTestId('settings-company-address-edit'));
		await fireEvent.input(getByTestId('settings-company-address-city-input'), {
			target: { value: 'Morges' },
		});
		await fireEvent.click(getByTestId('settings-company-address-save'));

		const relue = await findByTestId('settings-company-address-conflict-changed-city');
		expect(relue.textContent).toContain('Nyon');
		expect((await findByTestId('settings-company-address-conflict-notice')).textContent).toContain(
			'remplacera',
		);
		expect((getByTestId('settings-company-address-city-input') as HTMLInputElement).value).toBe('Morges');
		expect(fetchCompanyCurrentMock).toHaveBeenCalledTimes(2);
		expect(updateCompanyAddressMock).toHaveBeenCalledTimes(1);

		// Second clic : la saisie part, avec la version relue.
		updateCompanyAddressMock.mockResolvedValue(societe({ version: 10 }).company);
		await fireEvent.click(getByTestId('settings-company-address-save'));
		await vi.waitFor(() => expect(updateCompanyAddressMock).toHaveBeenCalledTimes(2));
		expect(updateCompanyAddressMock.mock.calls[1][0]).toEqual({
			address: { ...structured, city: 'Morges' },
			version: 9,
		});
		authState.clearSession({ broadcast: false });
	});

	for (const role of ['Comptable', 'Consultation']) {
		it(`${role} : lit l'adresse et l'avis, sans bouton Modifier`, async () => {
			connecter(role);
			fetchCompanyCurrentMock.mockResolvedValue(societe({ addressCountryKnown: false }));
			const { findByTestId, queryByTestId } = render(Page);
			await findByTestId('settings-company-address');
			expect(queryByTestId('settings-company-address-edit')).toBeNull();
			expect(queryByTestId('settings-company-address-country-unknown')).not.toBeNull();
			authState.clearSession({ broadcast: false });
		});
	}

	it("l'avis suit le jugement du serveur, et rien d'autre", async () => {
		connecter('Admin');
		fetchCompanyCurrentMock.mockResolvedValue(societe({ addressCountryKnown: true }));
		const { findByTestId, queryByTestId } = render(Page);
		await findByTestId('settings-company-address');
		expect(queryByTestId('settings-company-address-country-unknown')).toBeNull();
		authState.clearSession({ broadcast: false });
	});

	it("l'avis disparaît quand la correction est enregistrée", async () => {
		connecter('Admin');
		fetchCompanyCurrentMock.mockResolvedValue(societe({ addressCountryKnown: false }));
		updateCompanyAddressMock.mockResolvedValue(
			societe({ addressCountryKnown: true, version: 2 }).company,
		);
		const { findByTestId, getByTestId, queryByTestId } = render(Page);
		await findByTestId('settings-company-address-country-unknown');
		await fireEvent.click(getByTestId('settings-company-address-edit'));
		await fireEvent.click(getByTestId('settings-company-address-save'));
		await vi.waitFor(() =>
			expect(queryByTestId('settings-company-address-country-unknown')).toBeNull(),
		);
		authState.clearSession({ broadcast: false });
	});
});

// Revue P1 (E-L6) — Annuler referme sans envoyer, et Modifier repart de l'enregistré.
describe('page Paramètres — adresse : Annuler', () => {
	it('Annuler n’envoie rien et Modifier repart des valeurs enregistrées', async () => {
		authState.login({ userId: '1', username: 'u', role: 'Admin', expiresIn: 900 }, { broadcast: false });
		// Revue P3 (P3-B3, E3-3, A3-3) : `resetAllMocks` vide aussi les files `…Once` et les
		// implémentations posées par un test, que `clearAllMocks` laissait au test suivant.
		vi.resetAllMocks();
		fetchSessionSettingsMock.mockResolvedValue({ sessionIdleMinutes: 60, version: 0, min: 1, max: 1440 });
		const base = reponse(null);
		fetchCompanyCurrentMock.mockResolvedValue({
			...base,
			company: {
				...base.company,
				addressStructured: { street: 'Rue Test', building: '1', postalCode: '1000', city: 'Lausanne', country: 'CH' },
				addressCountryKnown: true,
			} as CompanyCurrentResponse['company'],
		});
		const { findByTestId, getByTestId, queryByTestId } = render(Page);
		await fireEvent.click(await findByTestId('settings-company-address-edit'));
		await fireEvent.input(getByTestId('settings-company-address-city-input'), {
			target: { value: 'Ailleurs' },
		});
		await fireEvent.click(getByTestId('settings-company-address-cancel'));
		expect(queryByTestId('settings-company-address-city-input')).toBeNull();
		expect(updateCompanyAddressMock).not.toHaveBeenCalled();
		await fireEvent.click(getByTestId('settings-company-address-edit'));
		const city = getByTestId('settings-company-address-city-input') as HTMLInputElement;
		expect(city.value).toBe('Lausanne');
		authState.clearSession({ broadcast: false });
	});
});

// Story 34-6 (#953) — après un 409, chaque édition de la société montre l'écriture
// concurrente avant de l'écraser. Les trois blocs suivent la même règle
// (`companyEditConflict`), et ce bloc-ci les exerce par l'écran.
describe('page Paramètres — conflit de version des trois éditions (Story 34-6)', () => {
	const ADRESSE = { street: 'Rue Test', building: '1', postalCode: '1000', city: 'Lausanne', country: 'CH' };
	const CONFLIT = { code: 'OPTIMISTIC_LOCK_CONFLICT', message: 'conflit', status: 409 };

	function societe(overrides: Record<string, unknown> = {}): CompanyCurrentResponse {
		const base = reponse(null);
		return {
			...base,
			company: {
				...base.company,
				addressStructured: ADRESSE,
				addressCountryKnown: true,
				email: 'info@test.ch',
				phone: '+41 21 000 00 00',
				website: 'https://test.ch',
				...overrides,
			} as CompanyCurrentResponse['company'],
		};
	}

	/** Relu après le 409 : la version a bougé, plus ce que l'autre a écrit. */
	function relue(overrides: Record<string, unknown> = {}): CompanyCurrentResponse {
		return societe({ version: 9, ...overrides });
	}

	beforeEach(() => {
		// Revue P3 (P3-B3, E3-3, A3-3) : `resetAllMocks` vide aussi les files `…Once` et les
		// implémentations posées par un test, que `clearAllMocks` laissait au test suivant.
		vi.resetAllMocks();
		fetchSessionSettingsMock.mockResolvedValue({ sessionIdleMinutes: 60, version: 0, min: 1, max: 1440 });
		authState.login({ userId: '1', username: 'u', role: 'Admin', expiresIn: 900 }, { broadcast: false });
	});

	async function saisir(getByTestId: (id: string) => HTMLElement, id: string, value: string) {
		await fireEvent.input(getByTestId(id), { target: { value } });
	}

	// ---------------------------------------------------------------- adresse
	describe('adresse', () => {
		async function ouvrir(apres409: CompanyCurrentResponse) {
			fetchCompanyCurrentMock.mockResolvedValueOnce(societe());
			fetchCompanyCurrentMock.mockResolvedValueOnce(apres409);
			updateCompanyAddressMock.mockRejectedValueOnce(CONFLIT);
			const r = render(Page);
			await fireEvent.click(await r.findByTestId('settings-company-address-edit'));
			return r;
		}

		it('AC-2 — aucun champ du bloc n’a changé : le message actuel, sans valeur, et l’essai suivant aboutit', async () => {
			const { getByTestId, findByTestId, queryByTestId } = await ouvrir(relue({ phone: '+41 22 999 99 99' }));
			await saisir(getByTestId, 'settings-company-address-postal-code-input', '1003');
			await fireEvent.click(getByTestId('settings-company-address-save'));
			expect((await findByTestId('settings-company-address-error')).textContent).toContain('rechargées');
			expect(queryByTestId('settings-company-address-conflict-notice')).toBeNull();
			expect(queryByTestId('settings-company-address-conflict-changed-postal-code')).toBeNull();

			updateCompanyAddressMock.mockResolvedValue(societe({ version: 10 }).company);
			await fireEvent.click(getByTestId('settings-company-address-save'));
			await vi.waitFor(() => expect(updateCompanyAddressMock).toHaveBeenCalledTimes(2));
			expect(updateCompanyAddressMock.mock.calls[1][0]).toEqual({
				address: { ...ADRESSE, postalCode: '1003' },
				version: 9,
			});
		});

		it('le NPA en conflit se nomme en kebab-case, comme les autres testids (revue P1, B-3)', async () => {
			const { getByTestId, findByTestId } = await ouvrir(
				relue({ addressStructured: { ...ADRESSE, postalCode: '1004' } }),
			);
			await saisir(getByTestId, 'settings-company-address-postal-code-input', '1003');
			await fireEvent.click(getByTestId('settings-company-address-save'));
			expect(
				(await findByTestId('settings-company-address-conflict-changed-postal-code')).textContent,
			).toContain('1004');
		});

		it('AC-3 — déjà enregistré : l’édition se ferme, la notification le dit, aucun second envoi (M3)', async () => {
			const { getByTestId, queryByTestId } = await ouvrir(
				relue({ addressStructured: { ...ADRESSE, city: 'Morges' } }),
			);
			await saisir(getByTestId, 'settings-company-address-city-input', 'Morges');
			await fireEvent.click(getByTestId('settings-company-address-save'));
			await vi.waitFor(() => expect(queryByTestId('settings-company-address-city-input')).toBeNull());
			expect(notifySuccessMock).toHaveBeenCalledWith(expect.stringContaining('déjà enregistr'));
			expect(updateCompanyAddressMock).toHaveBeenCalledTimes(1);
		});

		it('AC-5 ter — fusion : la rue de l’autre est reprise, le NPA saisi reste, le second clic écrit les deux (M6)', async () => {
			const { getByTestId, findByTestId, queryByTestId } = await ouvrir(
				relue({ addressStructured: { ...ADRESSE, street: 'Avenue Neuve' } }),
			);
			await saisir(getByTestId, 'settings-company-address-postal-code-input', '1003');
			await fireEvent.click(getByTestId('settings-company-address-save'));

			expect((await findByTestId('settings-company-address-conflict-merged-street')).textContent).toContain(
				'Avenue Neuve',
			);
			expect(queryByTestId('settings-company-address-conflict-notice')).not.toBeNull();
			expect(queryByTestId('settings-company-address-error')).toBeNull();
			expect((getByTestId('settings-company-address-street-input') as HTMLInputElement).value).toBe('Avenue Neuve');
			expect((getByTestId('settings-company-address-postal-code-input') as HTMLInputElement).value).toBe('1003');

			updateCompanyAddressMock.mockResolvedValue(societe({ version: 10 }).company);
			await fireEvent.click(getByTestId('settings-company-address-save'));
			await vi.waitFor(() => expect(updateCompanyAddressMock).toHaveBeenCalledTimes(2));
			expect(updateCompanyAddressMock.mock.calls[1][0].address).toEqual({
				...ADRESSE,
				street: 'Avenue Neuve',
				postalCode: '1003',
			});
		});

		it('AC-5 — second conflit : la rue reprise, et la ville n’est plus en conflit (M2)', async () => {
			const { getByTestId, findByTestId, queryByTestId } = await ouvrir(
				relue({ addressStructured: { ...ADRESSE, city: 'Nyon' } }),
			);
			await saisir(getByTestId, 'settings-company-address-city-input', 'Morges');
			await fireEvent.click(getByTestId('settings-company-address-save'));
			await findByTestId('settings-company-address-conflict-changed-city');

			// Troisième écriture, sur la rue seule.
			fetchCompanyCurrentMock.mockResolvedValueOnce(
				societe({ version: 11, addressStructured: { ...ADRESSE, city: 'Nyon', street: 'Avenue Neuve' } }),
			);
			updateCompanyAddressMock.mockRejectedValueOnce(CONFLIT);
			await fireEvent.click(getByTestId('settings-company-address-save'));
			await findByTestId('settings-company-address-conflict-merged-street');
			expect(queryByTestId('settings-company-address-conflict-changed-city')).toBeNull();
			expect((getByTestId('settings-company-address-city-input') as HTMLInputElement).value).toBe('Morges');
		});

		it('AC-6 — relecture impossible : le message reste, la saisie aussi, aucun avis de valeur', async () => {
			fetchCompanyCurrentMock.mockResolvedValueOnce(societe());
			fetchCompanyCurrentMock.mockRejectedValueOnce(new Error('réseau'));
			updateCompanyAddressMock.mockRejectedValueOnce(CONFLIT);
			const { findByTestId, getByTestId, queryByTestId } = render(Page);
			await fireEvent.click(await findByTestId('settings-company-address-edit'));
			await saisir(getByTestId, 'settings-company-address-city-input', 'Morges');
			await fireEvent.click(getByTestId('settings-company-address-save'));
			expect((await findByTestId('settings-company-address-error')).textContent).toContain(
				'rechargement impossible',
			);
			expect((getByTestId('settings-company-address-city-input') as HTMLInputElement).value).toBe('Morges');
			expect(queryByTestId('settings-company-address-conflict-notice')).toBeNull();
		});

		it('AC-6 bis — Annuler puis Modifier : l’avis est remis à zéro', async () => {
			const { getByTestId, findByTestId, queryByTestId } = await ouvrir(
				relue({ addressStructured: { ...ADRESSE, city: 'Nyon' } }),
			);
			await saisir(getByTestId, 'settings-company-address-city-input', 'Morges');
			await fireEvent.click(getByTestId('settings-company-address-save'));
			await findByTestId('settings-company-address-conflict-notice');
			await fireEvent.click(getByTestId('settings-company-address-cancel'));
			await fireEvent.click(getByTestId('settings-company-address-edit'));
			expect(queryByTestId('settings-company-address-conflict-notice')).toBeNull();
			expect(queryByTestId('settings-company-address-conflict-changed-city')).toBeNull();
			expect((getByTestId('settings-company-address-city-input') as HTMLInputElement).value).toBe('Nyon');
		});

		it('AC-6 bis — un enregistrement réussi remet l’avis à zéro', async () => {
			const { getByTestId, findByTestId, queryByTestId } = await ouvrir(
				relue({ addressStructured: { ...ADRESSE, city: 'Nyon' } }),
			);
			await saisir(getByTestId, 'settings-company-address-city-input', 'Morges');
			await fireEvent.click(getByTestId('settings-company-address-save'));
			await findByTestId('settings-company-address-conflict-notice');
			updateCompanyAddressMock.mockResolvedValue(
				societe({ version: 10, addressStructured: { ...ADRESSE, city: 'Morges' } }).company,
			);
			await fireEvent.click(getByTestId('settings-company-address-save'));
			await vi.waitFor(() => expect(queryByTestId('settings-company-address-city-input')).toBeNull());
			await fireEvent.click(getByTestId('settings-company-address-edit'));
			expect(queryByTestId('settings-company-address-conflict-notice')).toBeNull();
		});
	});

	// ----------------------------------------------------------------- e-mail
	describe('e-mail (un champ)', () => {
		async function ouvrir(apres409: CompanyCurrentResponse) {
			fetchCompanyCurrentMock.mockResolvedValueOnce(societe());
			fetchCompanyCurrentMock.mockResolvedValueOnce(apres409);
			updateCompanyEmailMock.mockRejectedValueOnce(CONFLIT);
			const r = render(Page);
			await fireEvent.click(await r.findByTestId('settings-company-email-edit'));
			return r;
		}

		it('AC-1 — la valeur relue est montrée, la saisie gardée, puis le second clic la remplace', async () => {
			const { getByTestId, findByTestId } = await ouvrir(relue({ email: 'autre@test.ch' }));
			await saisir(getByTestId, 'settings-company-email-input', 'moi@test.ch');
			await fireEvent.click(getByTestId('settings-company-email-save'));
			expect((await findByTestId('settings-company-email-conflict-changed-email')).textContent).toContain(
				'autre@test.ch',
			);
			expect((getByTestId('settings-company-email-input') as HTMLInputElement).value).toBe('moi@test.ch');
			expect(updateCompanyEmailMock).toHaveBeenCalledTimes(1);
			updateCompanyEmailMock.mockResolvedValue(societe({ version: 10, email: 'moi@test.ch' }).company);
			await fireEvent.click(getByTestId('settings-company-email-save'));
			await vi.waitFor(() => expect(updateCompanyEmailMock).toHaveBeenCalledTimes(2));
			expect(updateCompanyEmailMock.mock.calls[1][0]).toEqual({ email: 'moi@test.ch', version: 9 });
		});

		it('la valeur vidée ailleurs se dit « (vide) »', async () => {
			const { getByTestId, findByTestId } = await ouvrir(relue({ email: null }));
			await saisir(getByTestId, 'settings-company-email-input', 'moi@test.ch');
			await fireEvent.click(getByTestId('settings-company-email-save'));
			expect((await findByTestId('settings-company-email-conflict-changed-email')).textContent).toContain('(vide)');
		});

		it('AC-2 — rien n’a changé dans le bloc : le message actuel', async () => {
			const { getByTestId, findByTestId, queryByTestId } = await ouvrir(relue({ phone: '+41 22 999 99 99' }));
			await saisir(getByTestId, 'settings-company-email-input', 'moi@test.ch');
			await fireEvent.click(getByTestId('settings-company-email-save'));
			expect((await findByTestId('settings-company-email-error')).textContent).toContain('rechargées');
			expect(queryByTestId('settings-company-email-conflict-notice')).toBeNull();
			// Revue P1 (A-7) : l'essai suivant aboutit, avec la version relue.
			updateCompanyEmailMock.mockResolvedValue(societe({ version: 10, email: 'moi@test.ch' }).company);
			await fireEvent.click(getByTestId('settings-company-email-save'));
			await vi.waitFor(() => expect(updateCompanyEmailMock).toHaveBeenCalledTimes(2));
			expect(updateCompanyEmailMock.mock.calls[1][0]).toEqual({ email: 'moi@test.ch', version: 9 });
		});

		it('AC-3 — déjà enregistré : l’édition se ferme sans second envoi', async () => {
			const { getByTestId, queryByTestId } = await ouvrir(relue({ email: 'moi@test.ch' }));
			await saisir(getByTestId, 'settings-company-email-input', '  moi@test.ch ');
			await fireEvent.click(getByTestId('settings-company-email-save'));
			await vi.waitFor(() => expect(queryByTestId('settings-company-email-input')).toBeNull());
			expect(updateCompanyEmailMock).toHaveBeenCalledTimes(1);
			expect(notifySuccessMock).toHaveBeenCalledWith(expect.stringContaining('déjà enregistr'));
		});
	});

	// ------------------------------------------------------------ coordonnées
	describe('coordonnées (deux champs)', () => {
		async function ouvrir(apres409: CompanyCurrentResponse) {
			fetchCompanyCurrentMock.mockResolvedValueOnce(societe());
			fetchCompanyCurrentMock.mockResolvedValueOnce(apres409);
			updateCompanyContactDetailsMock.mockRejectedValueOnce(CONFLIT);
			const r = render(Page);
			await fireEvent.click(await r.findByTestId('settings-company-contact-edit'));
			return r;
		}

		it('AC-4 — un seul des deux champs a changé : lui seul est nommé (M5)', async () => {
			const { getByTestId, findByTestId, queryByTestId } = await ouvrir(relue({ website: 'https://autre.ch' }));
			await saisir(getByTestId, 'settings-company-website-input', 'https://moi.ch');
			await fireEvent.click(getByTestId('settings-company-contact-save'));
			expect((await findByTestId('settings-company-contact-conflict-changed-website')).textContent).toContain(
				'https://autre.ch',
			);
			expect(queryByTestId('settings-company-contact-conflict-changed-phone')).toBeNull();
			expect((getByTestId('settings-company-website-input') as HTMLInputElement).value).toBe('https://moi.ch');
		});

		it('AC-5 bis a) — le site écrit ailleurs à la valeur saisie : aucun champ nommé', async () => {
			const { getByTestId, findByTestId, queryByTestId } = await ouvrir(relue({ website: 'X' }));
			await saisir(getByTestId, 'settings-company-phone-input', 'T');
			await saisir(getByTestId, 'settings-company-website-input', 'X');
			await fireEvent.click(getByTestId('settings-company-contact-save'));
			expect((await findByTestId('settings-company-contact-error')).textContent).toContain('rechargées');
			expect(queryByTestId('settings-company-contact-conflict-changed-website')).toBeNull();
			expect(queryByTestId('settings-company-contact-conflict-changed-phone')).toBeNull();
		});

		it('AC-5 bis b) — le site à la valeur saisie, le téléphone ailleurs : seul le téléphone est nommé', async () => {
			const { getByTestId, findByTestId, queryByTestId } = await ouvrir(relue({ website: 'X', phone: 'U' }));
			await saisir(getByTestId, 'settings-company-phone-input', 'T');
			await saisir(getByTestId, 'settings-company-website-input', 'X');
			await fireEvent.click(getByTestId('settings-company-contact-save'));
			expect((await findByTestId('settings-company-contact-conflict-changed-phone')).textContent).toContain('U');
			expect(queryByTestId('settings-company-contact-conflict-changed-website')).toBeNull();
		});

		it('fusion seule, sans modification : l’avis de fusion, l’édition reste ouverte', async () => {
			const { getByTestId, findByTestId, queryByTestId } = await ouvrir(relue({ phone: 'U' }));
			await fireEvent.click(getByTestId('settings-company-contact-save'));
			expect((await findByTestId('settings-company-contact-conflict-merged-phone')).textContent).toContain('U');
			expect(queryByTestId('settings-company-contact-conflict-notice')).not.toBeNull();
			expect((getByTestId('settings-company-phone-input') as HTMLInputElement).value).toBe('U');
			expect(notifySuccessMock).not.toHaveBeenCalled();
		});

		// Revue P1 (A-1) : l'AC-3 vaut aussi pour les coordonnées (AC-4).
		it('AC-3 — déjà enregistré : l’édition se ferme sans second envoi', async () => {
			const { getByTestId, queryByTestId } = await ouvrir(relue({ website: 'https://moi.ch' }));
			await saisir(getByTestId, 'settings-company-website-input', 'https://moi.ch');
			await fireEvent.click(getByTestId('settings-company-contact-save'));
			await vi.waitFor(() => expect(queryByTestId('settings-company-website-input')).toBeNull());
			expect(updateCompanyContactDetailsMock).toHaveBeenCalledTimes(1);
			expect(notifySuccessMock).toHaveBeenCalledWith(expect.stringContaining('déjà enregistr'));
		});
	});

	// ------------------------------------------- revue P1 : copie, avis, annulation
	// E1-2, A-2, A-6 : la mise à jour de la copie, l'effacement de l'avis quand la relecture
	// échoue, et la remise à zéro par l'annulation, bloc par bloc (chaque bloc a ses lignes).
	describe('copie et avis, par bloc (revue P1)', () => {
		const BLOCS = [
			{
				nom: 'e-mail',
				edit: 'settings-company-email-edit',
				save: 'settings-company-email-save',
				input: 'settings-company-email-input',
				saisie: 'moi@test.ch',
				autre: { email: 'autre@test.ch' },
				changed: 'settings-company-email-conflict-changed-email',
				notice: 'settings-company-email-conflict-notice',
				error: 'settings-company-email-error',
				mock: () => updateCompanyEmailMock,
				annuler: (getAllByText: (t: string) => HTMLElement[]) => getAllByText('Annuler')[0],
			},
			{
				nom: 'coordonnées',
				edit: 'settings-company-contact-edit',
				save: 'settings-company-contact-save',
				input: 'settings-company-website-input',
				saisie: 'https://moi.ch',
				autre: { website: 'https://autre.ch' },
				changed: 'settings-company-contact-conflict-changed-website',
				notice: 'settings-company-contact-conflict-notice',
				error: 'settings-company-contact-error',
				mock: () => updateCompanyContactDetailsMock,
				annuler: (getAllByText: (t: string) => HTMLElement[]) => getAllByText('Annuler')[0],
			},
		] as const;

		for (const b of BLOCS) {
			async function premierConflit() {
				fetchCompanyCurrentMock.mockResolvedValueOnce(societe());
				fetchCompanyCurrentMock.mockResolvedValueOnce(relue(b.autre));
				b.mock().mockRejectedValueOnce(CONFLIT);
				const r = render(Page);
				await fireEvent.click(await r.findByTestId(b.edit));
				await saisir(r.getByTestId, b.input, b.saisie);
				await fireEvent.click(r.getByTestId(b.save));
				await r.findByTestId(b.changed);
				return r;
			}

			it(`${b.nom} — second conflit : la copie a pris la valeur relue, le champ n’est plus en conflit (E1-2)`, async () => {
				const { getByTestId, findByTestId, queryByTestId } = await premierConflit();
				fetchCompanyCurrentMock.mockResolvedValueOnce(societe({ version: 11, ...b.autre }));
				b.mock().mockRejectedValueOnce(CONFLIT);
				await fireEvent.click(getByTestId(b.save));
				expect((await findByTestId(b.error)).textContent).toContain('rechargées');
				expect(queryByTestId(b.changed)).toBeNull();
				expect((getByTestId(b.input) as HTMLInputElement).value).toBe(b.saisie);
			});

			it(`${b.nom} — relecture impossible après un premier avis : l’avis s’efface, la saisie reste (A-2)`, async () => {
				const { getByTestId, findByTestId, queryByTestId } = await premierConflit();
				fetchCompanyCurrentMock.mockRejectedValueOnce(new Error('réseau'));
				b.mock().mockRejectedValueOnce(CONFLIT);
				await fireEvent.click(getByTestId(b.save));
				expect((await findByTestId(b.error)).textContent).toContain('rechargement impossible');
				expect(queryByTestId(b.notice)).toBeNull();
				expect(queryByTestId(b.changed)).toBeNull();
				expect((getByTestId(b.input) as HTMLInputElement).value).toBe(b.saisie);
			});

			it(`${b.nom} — Annuler puis Modifier : l’avis est remis à zéro (A-6)`, async () => {
				const { getByTestId, getAllByText, queryByTestId } = await premierConflit();
				await fireEvent.click(b.annuler(getAllByText));
				await fireEvent.click(getByTestId(b.edit));
				expect(queryByTestId(b.notice)).toBeNull();
				expect(queryByTestId(b.changed)).toBeNull();
			});
		}
	});

	// -------------------------------------- revue P1 (B-1, E1-1) : deux éditions ouvertes
	// Un bloc resté ouvert ne doit jamais envoyer une version plus récente que les valeurs
	// qu'il a montrées (C-e346-8).
	describe('deux éditions ouvertes (revue P1, B-1)', () => {
		it('la relecture d’un bloc ne fait pas partir l’autre avec la version relue si ses champs ont bougé', async () => {
			fetchCompanyCurrentMock.mockResolvedValueOnce(societe());
			// Relue au 409 de l'e-mail : le téléphone a été changé par un autre.
			fetchCompanyCurrentMock.mockResolvedValueOnce(relue({ phone: '+41 22 999 99 99' }));
			updateCompanyEmailMock.mockRejectedValueOnce(CONFLIT);
			const { findByTestId, getByTestId } = render(Page);
			await fireEvent.click(await findByTestId('settings-company-contact-edit'));
			await fireEvent.click(getByTestId('settings-company-email-edit'));
			await saisir(getByTestId, 'settings-company-website-input', 'https://moi.ch');
			await saisir(getByTestId, 'settings-company-email-input', 'moi@test.ch');
			await fireEvent.click(getByTestId('settings-company-email-save'));
			await findByTestId('settings-company-email-error');

			updateCompanyContactDetailsMock.mockRejectedValueOnce(CONFLIT);
			fetchCompanyCurrentMock.mockResolvedValueOnce(relue({ phone: '+41 22 999 99 99' }));
			await fireEvent.click(getByTestId('settings-company-contact-save'));
			await vi.waitFor(() => expect(updateCompanyContactDetailsMock).toHaveBeenCalledTimes(1));
			// La version des valeurs montrées (1), non la version relue par l'e-mail (9).
			expect(updateCompanyContactDetailsMock.mock.calls[0][0].version).toBe(1);
			// Son propre 409 montre alors la reprise du téléphone de l'autre.
			expect(
				(await findByTestId('settings-company-contact-conflict-merged-phone')).textContent,
			).toContain('+41 22 999 99 99');
		});

		it('un autre bloc enregistré sur la page ne provoque pas de conflit parasite', async () => {
			fetchCompanyCurrentMock.mockResolvedValueOnce(societe());
			updateCompanyContactDetailsMock.mockResolvedValueOnce(
				societe({ version: 2, website: 'https://moi.ch' }).company,
			);
			updateCompanyEmailMock.mockResolvedValueOnce(societe({ version: 3, email: 'moi@test.ch' }).company);
			const { findByTestId, getByTestId } = render(Page);
			await fireEvent.click(await findByTestId('settings-company-email-edit'));
			await fireEvent.click(getByTestId('settings-company-contact-edit'));
			await saisir(getByTestId, 'settings-company-website-input', 'https://moi.ch');
			await fireEvent.click(getByTestId('settings-company-contact-save'));
			await vi.waitFor(() => expect(updateCompanyContactDetailsMock).toHaveBeenCalledTimes(1));
			await saisir(getByTestId, 'settings-company-email-input', 'moi@test.ch');
			await fireEvent.click(getByTestId('settings-company-email-save'));
			await vi.waitFor(() => expect(updateCompanyEmailMock).toHaveBeenCalledTimes(1));
			expect(updateCompanyEmailMock.mock.calls[0][0]).toEqual({ email: 'moi@test.ch', version: 2 });
		});

		// Revue P2 (B2) : en lecture seule et non désactivés, les champs gardent le focus.
		// Revue P3 (B7) : le titre ne promet plus la relecture, que le corps n'exerce pas.
		it('les champs sont en lecture seule pendant l’envoi, sans perdre le focus (E1-3, B2)', async () => {
			fetchCompanyCurrentMock.mockResolvedValueOnce(societe());
			let release: (v: unknown) => void = () => {};
			updateCompanyEmailMock.mockReturnValueOnce(new Promise((r) => (release = r)));
			const { findByTestId, getByTestId } = render(Page);
			await fireEvent.click(await findByTestId('settings-company-email-edit'));
			await fireEvent.click(getByTestId('settings-company-email-save'));
			const input = getByTestId('settings-company-email-input') as HTMLInputElement;
			await vi.waitFor(() => expect(input.readOnly).toBe(true));
			expect(input.disabled).toBe(false);
			release(societe({ version: 2 }).company);
		});

		// Revue P2 (P2-B1 = E2-1 = A2-5 ; C-e346-10) : une relecture partie avant le succès
		// d'un autre bloc et revenue après ne fait reculer ni `data` ni la version d'une
		// édition ouverte. Le bloc adresse est celui qui adopte (A2-4).
		it('une relecture plus ancienne que le succès d’un autre bloc ne fait rien reculer', async () => {
			fetchCompanyCurrentMock.mockResolvedValueOnce(societe());
			let relire: (v: CompanyCurrentResponse) => void = () => {};
			fetchCompanyCurrentMock.mockReturnValueOnce(new Promise((r) => (relire = r)));
			updateCompanyEmailMock.mockRejectedValueOnce(CONFLIT);
			updateCompanyContactDetailsMock.mockResolvedValueOnce(
				societe({ version: 10, website: 'https://moi.ch' }).company,
			);
			const { findByTestId, getByTestId } = render(Page);
			await fireEvent.click(await findByTestId('settings-company-address-edit'));
			await fireEvent.click(getByTestId('settings-company-email-edit'));
			await fireEvent.click(getByTestId('settings-company-contact-edit'));
			// 1. L'e-mail reçoit un 409 : sa relecture part, et reste en vol.
			await fireEvent.click(getByTestId('settings-company-email-save'));
			await vi.waitFor(() => expect(fetchCompanyCurrentMock).toHaveBeenCalledTimes(2));
			// 2. Les coordonnées réussissent (version 10) : l'adresse, inchangée, adopte 10.
			await saisir(getByTestId, 'settings-company-website-input', 'https://moi.ch');
			await fireEvent.click(getByTestId('settings-company-contact-save'));
			await vi.waitFor(() => expect(getByTestId('settings-company-website').textContent).toBe('https://moi.ch'));
			// 3. La relecture de l'e-mail revient, servie avant le succès : version 9.
			relire(societe({ version: 9 }));
			await findByTestId('settings-company-email-error');
			// L'affichage ne revient pas à l'ancienne valeur, et l'adresse part avec 10.
			expect(getByTestId('settings-company-website').textContent).toBe('https://moi.ch');
			updateCompanyAddressMock.mockResolvedValueOnce(societe({ version: 11 }).company);
			await saisir(getByTestId, 'settings-company-address-city-input', 'Morges');
			await fireEvent.click(getByTestId('settings-company-address-save'));
			await vi.waitFor(() => expect(updateCompanyAddressMock).toHaveBeenCalledTimes(1));
			expect(updateCompanyAddressMock.mock.calls[0][0].version).toBe(10);
		});

		it('adresse ouverte : une relecture où ses champs ont bougé ne lui fait pas adopter la version (A2-4)', async () => {
			fetchCompanyCurrentMock.mockResolvedValueOnce(societe());
			fetchCompanyCurrentMock.mockResolvedValueOnce(
				relue({ addressStructured: { ...ADRESSE, street: 'Avenue Neuve' } }),
			);
			updateCompanyEmailMock.mockRejectedValueOnce(CONFLIT);
			const { findByTestId, getByTestId } = render(Page);
			await fireEvent.click(await findByTestId('settings-company-address-edit'));
			await fireEvent.click(getByTestId('settings-company-email-edit'));
			await saisir(getByTestId, 'settings-company-email-input', 'moi@test.ch');
			await fireEvent.click(getByTestId('settings-company-email-save'));
			await findByTestId('settings-company-email-error');
			updateCompanyAddressMock.mockRejectedValueOnce(CONFLIT);
			fetchCompanyCurrentMock.mockResolvedValueOnce(
				relue({ addressStructured: { ...ADRESSE, street: 'Avenue Neuve' } }),
			);
			await saisir(getByTestId, 'settings-company-address-city-input', 'Morges');
			await fireEvent.click(getByTestId('settings-company-address-save'));
			await vi.waitFor(() => expect(updateCompanyAddressMock).toHaveBeenCalledTimes(1));
			expect(updateCompanyAddressMock.mock.calls[0][0].version).toBe(1);
			expect(
				(await findByTestId('settings-company-address-conflict-merged-street')).textContent,
			).toContain('Avenue Neuve');
		});

		// Revue P3 (P3-B4, A3-1) : `lock_books` ne change pas la version. La version 5 rendue
		// ici est celle d'une écriture d'un AUTRE (sur un autre bloc) faite entre l'ouverture et
		// le verrou : le verrou rend la société la plus récente. Le cas du verrou seul, à
		// version égale, est le test suivant.
		it('le verrou de période fait adopter la société rendue, écrite entre-temps par un autre, aux éditions inchangées (E2-3)', async () => {
			fetchCompanyCurrentMock.mockResolvedValueOnce(societe());
			lockBooksMock.mockResolvedValueOnce(
				societe({ version: 5, phone: '+41 22 999 99 99', booksLockedThrough: '2026-01-31' }).company,
			);
			updateCompanyEmailMock.mockResolvedValueOnce(societe({ version: 6, email: 'moi@test.ch' }).company);
			const { findByTestId, getByTestId, container } = render(Page);
			await fireEvent.click(await findByTestId('settings-company-email-edit'));
			const date = container.querySelector('#books-lock-through') as HTMLInputElement;
			await fireEvent.input(date, { target: { value: '2026-01-31' } });
			await fireEvent.click(getByTestId('books-lock-submit'));
			await vi.waitFor(() => expect(lockBooksMock).toHaveBeenCalledTimes(1));
			await saisir(getByTestId, 'settings-company-email-input', 'moi@test.ch');
			await vi.waitFor(() =>
				expect((getByTestId('books-lock-submit') as HTMLButtonElement).disabled).toBe(true),
			);
			await fireEvent.click(getByTestId('settings-company-email-save'));
			await vi.waitFor(() => expect(updateCompanyEmailMock).toHaveBeenCalledTimes(1));
			expect(updateCompanyEmailMock.mock.calls[0][0].version).toBe(5);
		});

		// Revue P3 (P3-B1 = A3-1 ; C-e346-11) : le vrai comportement du serveur, un verrou qui
		// rend la MÊME version. L'écran montre la borne (la mutation `<` → `<=` d'`adoptCompany`
		// ferait ignorer la réponse), et l'édition ouverte garde sa version.
		it('verrouillage à version égale : l’écran montre la borne, l’édition ouverte garde sa version', async () => {
			fetchCompanyCurrentMock.mockResolvedValueOnce(societe());
			lockBooksMock.mockResolvedValueOnce(societe({ booksLockedThrough: '2026-01-31' }).company);
			updateCompanyEmailMock.mockResolvedValueOnce(societe({ version: 2, email: 'moi@test.ch' }).company);
			const { findByTestId, getByTestId, queryByTestId, container } = render(Page);
			await fireEvent.click(await findByTestId('settings-company-email-edit'));
			expect(queryByTestId('books-lock-release-toggle')).toBeNull();
			const date = container.querySelector('#books-lock-through') as HTMLInputElement;
			await fireEvent.input(date, { target: { value: '2026-01-31' } });
			await fireEvent.click(getByTestId('books-lock-submit'));
			await findByTestId('books-lock-release-toggle');
			await saisir(getByTestId, 'settings-company-email-input', 'moi@test.ch');
			await fireEvent.click(getByTestId('settings-company-email-save'));
			await vi.waitFor(() => expect(updateCompanyEmailMock).toHaveBeenCalledTimes(1));
			expect(updateCompanyEmailMock.mock.calls[0][0].version).toBe(1);
		});

		// Revue P3 (P3-B1, E3-1, A3-2 ; C-e346-11) : à version égale, une relecture partie
		// AVANT le verrou et revenue après ne fait pas reculer l'affichage du verrou.
		it('une relecture partie avant le verrou et revenue après ne réaffiche pas l’ancienne borne', async () => {
			fetchCompanyCurrentMock.mockResolvedValueOnce(societe());
			let relire: (v: CompanyCurrentResponse) => void = () => {};
			fetchCompanyCurrentMock.mockReturnValueOnce(new Promise((r) => (relire = r)));
			updateCompanyEmailMock.mockRejectedValueOnce(CONFLIT);
			lockBooksMock.mockResolvedValueOnce(societe({ booksLockedThrough: '2026-01-31' }).company);
			const { findByTestId, getByTestId, queryByTestId, container } = render(Page);
			await fireEvent.click(await findByTestId('settings-company-email-edit'));
			// 1. L'e-mail reçoit un 409 : sa relecture part (version 1, sans verrou).
			await fireEvent.click(getByTestId('settings-company-email-save'));
			await vi.waitFor(() => expect(fetchCompanyCurrentMock).toHaveBeenCalledTimes(2));
			// 2. L'Administrateur verrouille : même version, la borne est posée.
			const date = container.querySelector('#books-lock-through') as HTMLInputElement;
			await fireEvent.input(date, { target: { value: '2026-01-31' } });
			await fireEvent.click(getByTestId('books-lock-submit'));
			await findByTestId('books-lock-release-toggle');
			// 3. La relecture revient, servie avant le verrou.
			relire(societe());
			await findByTestId('settings-company-email-error');
			expect(queryByTestId('books-lock-release-toggle')).not.toBeNull();
		});

		// Revue P3 (P3-B2 ; C-e346-11) : une relecture plus ancienne que `data` ne juge pas le
		// bloc. Il est jugé contre la société la plus récente, et ne se ferme pas sur « déjà
		// enregistrées » quand un autre a changé la valeur depuis.
		it('une relecture plus ancienne ne ferme pas l’édition sur « déjà enregistrées » à tort', async () => {
			fetchCompanyCurrentMock.mockResolvedValueOnce(societe());
			let relireEmail: (v: CompanyCurrentResponse) => void = () => {};
			fetchCompanyCurrentMock.mockReturnValueOnce(new Promise((r) => (relireEmail = r)));
			updateCompanyEmailMock.mockRejectedValueOnce(CONFLIT);
			updateCompanyContactDetailsMock.mockRejectedValueOnce(CONFLIT);
			const { findByTestId, getByTestId, queryByTestId } = render(Page);
			await fireEvent.click(await findByTestId('settings-company-email-edit'));
			await fireEvent.click(getByTestId('settings-company-contact-edit'));
			await saisir(getByTestId, 'settings-company-email-input', 'moi@test.ch');
			// 1. L'e-mail reçoit un 409 : sa relecture part.
			await fireEvent.click(getByTestId('settings-company-email-save'));
			await vi.waitFor(() => expect(fetchCompanyCurrentMock).toHaveBeenCalledTimes(2));
			// 2. Les coordonnées reçoivent un 409 : leur relecture rend la version 7, où un autre
			//    a mis l'e-mail à « autre@test.ch ».
			fetchCompanyCurrentMock.mockResolvedValueOnce(societe({ version: 7, email: 'autre@test.ch' }));
			await saisir(getByTestId, 'settings-company-website-input', 'https://moi.ch');
			await fireEvent.click(getByTestId('settings-company-contact-save'));
			await findByTestId('settings-company-contact-error');
			// 3. La relecture de l'e-mail revient, plus ancienne (version 6), où l'e-mail valait
			//    la saisie.
			relireEmail(societe({ version: 6, email: 'moi@test.ch' }));
			expect((await findByTestId('settings-company-email-conflict-changed-email')).textContent).toContain(
				'autre@test.ch',
			);
			expect(queryByTestId('settings-company-email-input')).not.toBeNull();
			expect(notifySuccessMock).not.toHaveBeenCalled();
		});

		// Revue P3 (P3-B6, E3-4) : l'avis est effacé au départ de l'envoi. Un refus d'une autre
		// nature ne laisse pas un avis périmé, et un second conflit identique refait l'avis.
		it('l’avis est effacé au départ du nouvel envoi', async () => {
			fetchCompanyCurrentMock.mockResolvedValueOnce(societe());
			fetchCompanyCurrentMock.mockResolvedValueOnce(relue({ email: 'autre@test.ch' }));
			updateCompanyEmailMock.mockRejectedValueOnce(CONFLIT);
			const { findByTestId, getByTestId, queryByTestId } = render(Page);
			await fireEvent.click(await findByTestId('settings-company-email-edit'));
			await saisir(getByTestId, 'settings-company-email-input', 'moi@test.ch');
			await fireEvent.click(getByTestId('settings-company-email-save'));
			await findByTestId('settings-company-email-conflict-notice');
			updateCompanyEmailMock.mockRejectedValueOnce({ code: 'VALIDATION_ERROR', message: 'refus', status: 400 });
			await fireEvent.click(getByTestId('settings-company-email-save'));
			expect((await findByTestId('settings-company-email-error')).textContent).toBe('refus');
			expect(queryByTestId('settings-company-email-conflict-notice')).toBeNull();
		});

		// Revue P4 (A4-5) : la même remise à zéro, pour les coordonnées et pour l'adresse.
		it('coordonnées : l’avis est effacé au départ du nouvel envoi', async () => {
			fetchCompanyCurrentMock.mockResolvedValueOnce(societe());
			fetchCompanyCurrentMock.mockResolvedValueOnce(relue({ website: 'https://autre.ch' }));
			updateCompanyContactDetailsMock.mockRejectedValueOnce(CONFLIT);
			const { findByTestId, getByTestId, queryByTestId } = render(Page);
			await fireEvent.click(await findByTestId('settings-company-contact-edit'));
			await saisir(getByTestId, 'settings-company-website-input', 'https://moi.ch');
			await fireEvent.click(getByTestId('settings-company-contact-save'));
			await findByTestId('settings-company-contact-conflict-notice');
			updateCompanyContactDetailsMock.mockRejectedValueOnce({ code: 'VALIDATION_ERROR', message: 'refus', status: 400 });
			await fireEvent.click(getByTestId('settings-company-contact-save'));
			expect((await findByTestId('settings-company-contact-error')).textContent).toBe('refus');
			expect(queryByTestId('settings-company-contact-conflict-notice')).toBeNull();
		});

		it('adresse : l’avis est effacé au départ du nouvel envoi', async () => {
			fetchCompanyCurrentMock.mockResolvedValueOnce(societe());
			fetchCompanyCurrentMock.mockResolvedValueOnce(relue({ addressStructured: { ...ADRESSE, city: 'Nyon' } }));
			updateCompanyAddressMock.mockRejectedValueOnce(CONFLIT);
			const { findByTestId, getByTestId, queryByTestId } = render(Page);
			await fireEvent.click(await findByTestId('settings-company-address-edit'));
			await saisir(getByTestId, 'settings-company-address-city-input', 'Morges');
			await fireEvent.click(getByTestId('settings-company-address-save'));
			await findByTestId('settings-company-address-conflict-notice');
			updateCompanyAddressMock.mockRejectedValueOnce({ code: 'VALIDATION_ERROR', message: 'refus', status: 400 });
			await fireEvent.click(getByTestId('settings-company-address-save'));
			expect((await findByTestId('settings-company-address-error')).textContent).toBe('refus');
			expect(queryByTestId('settings-company-address-conflict-notice')).toBeNull();
		});
	});

	// Revue P2 (E2-2) : l'adresse retirée ailleurs pendant l'édition.
	it('adresse relue absente : les champs non touchés sont repris vides, le pays reste CH', async () => {
		fetchCompanyCurrentMock.mockResolvedValueOnce(societe());
		fetchCompanyCurrentMock.mockResolvedValueOnce(relue({ addressStructured: null }));
		updateCompanyAddressMock.mockRejectedValueOnce(CONFLIT);
		const { findByTestId, getByTestId, queryByTestId } = render(Page);
		await fireEvent.click(await findByTestId('settings-company-address-edit'));
		await saisir(getByTestId, 'settings-company-address-city-input', 'Morges');
		await fireEvent.click(getByTestId('settings-company-address-save'));
		expect((await findByTestId('settings-company-address-conflict-changed-city')).textContent).toContain('(vide)');
		expect((await findByTestId('settings-company-address-conflict-merged-street')).textContent).toContain('(vide)');
		expect((getByTestId('settings-company-address-street-input') as HTMLInputElement).value).toBe('');
		expect((getByTestId('settings-company-address-city-input') as HTMLInputElement).value).toBe('Morges');
		expect((getByTestId('settings-company-address-country-input') as HTMLInputElement).value).toBe('CH');
		expect(queryByTestId('settings-company-address-conflict-merged-country')).toBeNull();
	});

	// Revue P2 (B3, E2-4) : la région vive existe avant le conflit, vide ; l'avis s'y insère.
	it('la région vive de l’avis est présente dès l’ouverture, et l’avis y entre au conflit', async () => {
		fetchCompanyCurrentMock.mockResolvedValueOnce(societe());
		fetchCompanyCurrentMock.mockResolvedValueOnce(relue({ email: 'autre@test.ch' }));
		updateCompanyEmailMock.mockRejectedValueOnce(CONFLIT);
		const { findByTestId, getByTestId, getAllByRole } = render(Page);
		await fireEvent.click(await findByTestId('settings-company-email-edit'));
		const regions = getAllByRole('status');
		expect(regions.length).toBeGreaterThan(0);
		await saisir(getByTestId, 'settings-company-email-input', 'moi@test.ch');
		await fireEvent.click(getByTestId('settings-company-email-save'));
		const avis = await findByTestId('settings-company-email-conflict-notice');
		expect(regions.some((r) => r.contains(avis))).toBe(true);
	});

	// Revue P5 (E5-5) : la classe du champ en vol est posée sur les huit champs, et la lecture
	// seule vaut aussi pour les coordonnées et l'adresse (seul l'e-mail était exercé).
	const CHAMPS_DES_BLOCS = {
		email: ['settings-company-email-input'],
		contact: ['settings-company-phone-input', 'settings-company-website-input'],
		address: [
			'settings-company-address-street-input',
			'settings-company-address-building-input',
			'settings-company-address-postal-code-input',
			'settings-company-address-city-input',
			'settings-company-address-country-input',
		],
	} as const;

	it('les huit champs portent la classe du champ en vol', async () => {
		fetchCompanyCurrentMock.mockResolvedValueOnce(societe());
		const { findByTestId, getByTestId } = render(Page);
		await fireEvent.click(await findByTestId('settings-company-email-edit'));
		await fireEvent.click(getByTestId('settings-company-contact-edit'));
		await fireEvent.click(getByTestId('settings-company-address-edit'));
		for (const id of Object.values(CHAMPS_DES_BLOCS).flat()) {
			const champ = getByTestId(id);
			for (const classe of PENDING_FIELD_CLASS.split(' ')) {
				expect(champ.classList.contains(classe), `${id} : ${classe}`).toBe(true);
			}
		}
	});

	for (const [bloc, mock] of [
		['contact', () => updateCompanyContactDetailsMock],
		['address', () => updateCompanyAddressMock],
	] as const) {
		it(`${bloc} : les champs sont en lecture seule pendant l’envoi`, async () => {
			fetchCompanyCurrentMock.mockResolvedValueOnce(societe());
			let release: (v: unknown) => void = () => {};
			mock().mockReturnValueOnce(new Promise((r) => (release = r)));
			const { findByTestId, getByTestId } = render(Page);
			await fireEvent.click(await findByTestId(`settings-company-${bloc}-edit`));
			await fireEvent.click(getByTestId(`settings-company-${bloc}-save`));
			for (const id of CHAMPS_DES_BLOCS[bloc]) {
				await vi.waitFor(() => expect((getByTestId(id) as HTMLInputElement).readOnly).toBe(true));
			}
			release(societe({ version: 2 }).company);
		});
	}
});
