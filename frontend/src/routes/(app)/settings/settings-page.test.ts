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
vi.mock('$lib/features/settings/settings.api', () => ({
	fetchCompanyCurrent: () => fetchCompanyCurrentMock(),
	updateCompanyContactDetails: vi.fn(),
	updateCompanyEmail: vi.fn(),
	lockBooks: (...a: unknown[]) => lockBooksMock(...a),
	releaseBooksLock: (...a: unknown[]) => releaseBooksLockMock(...a),
}));

import Page from './+page.svelte';

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
