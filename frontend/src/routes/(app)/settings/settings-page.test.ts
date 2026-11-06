// Story 26-2b (#622, AC9, site D9) — *Paramètres* nomme chaque compte bancaire
// avec sa description. La réponse simulée de `/api/v1/companies/current` est
// typée `CompanyCurrentResponse` : `description` y est obligatoire, `npm run
// check` refuse une simulation qui l'omet.
//
// Le nom du fichier n'est pas `+page.test.ts` : SvelteKit réserve le préfixe `+`.

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, fireEvent } from '@testing-library/svelte';
import type { CompanyCurrentResponse } from '$lib/features/settings/settings.types';

vi.mock('$app/environment', () => ({ browser: true }));
vi.mock('$lib/features/onboarding/onboarding.svelte', () => ({
	i18nMsg: (_key: string, fallback: string) => fallback,
}));
vi.mock('$lib/shared/utils/notify', () => ({ notifySuccess: vi.fn() }));
const fetchCompanyCurrentMock = vi.fn<() => Promise<CompanyCurrentResponse>>();
const lockBooksMock = vi.fn();
const releaseBooksLockMock = vi.fn();
// Story v0172-session (AC-8) : la fabrique remplace le module entier — elle
// reçoit la lecture et l'enregistrement du réglage de la durée d'inactivité.
const fetchSessionSettingsMock = vi.fn();
const updateSessionSettingsMock = vi.fn();
const updateCompanyAddressMock = vi.fn();
vi.mock('$lib/features/settings/settings.api', () => ({
	fetchCompanyCurrent: () => fetchCompanyCurrentMock(),
	updateCompanyAddress: (...a: unknown[]) => updateCompanyAddressMock(...a),
	updateCompanyContactDetails: vi.fn(),
	updateCompanyEmail: vi.fn(),
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
		vi.clearAllMocks();
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
		vi.clearAllMocks();
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
		vi.clearAllMocks();
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
		vi.clearAllMocks();
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

	it('un conflit de version recharge la société et le dit', async () => {
		connecter('Admin');
		fetchCompanyCurrentMock.mockResolvedValueOnce(societe());
		fetchCompanyCurrentMock.mockResolvedValueOnce(societe({ version: 9 }));
		updateCompanyAddressMock.mockRejectedValue({
			code: 'OPTIMISTIC_LOCK_CONFLICT',
			message: 'conflit',
			status: 409,
		});
		const { findByTestId, getByTestId } = render(Page);
		await fireEvent.click(await findByTestId('settings-company-address-edit'));
		await fireEvent.click(getByTestId('settings-company-address-save'));
		const error = await findByTestId('settings-company-address-error');
		expect(error.textContent).toContain('rechargées');
		expect(fetchCompanyCurrentMock).toHaveBeenCalledTimes(2);

		// Le second essai part avec la version rechargée.
		updateCompanyAddressMock.mockResolvedValue(societe({ version: 10 }).company);
		await fireEvent.click(getByTestId('settings-company-address-save'));
		await vi.waitFor(() => expect(updateCompanyAddressMock).toHaveBeenCalledTimes(2));
		expect(updateCompanyAddressMock.mock.calls[1][0].version).toBe(9);
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
