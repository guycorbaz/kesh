/**
 * Story v017-2 (#809) — l'assistant d'installation affiche le motif d'un refus du serveur.
 *
 * Avant la story, sept étapes (S1 à S7) attrapaient le refus par un `catch {` nu et
 * affichaient un message générique ; la finalisation (S9, S11) comparait le code
 * `'VALIDATION'`, que le serveur ne rend jamais (`VALIDATION_ERROR`). Ici, pour chaque
 * site de l'inventaire de la fiche :
 *
 * - une `ApiError` à message → ce message, tel quel ;
 * - une valeur qui n'est pas une `ApiError` (`TypeError`) → le message **propre à
 *   l'étape**, et la valeur journalisée ;
 * - une `ApiError` à message blanc (S1, S7, S8) → le message de l'étape.
 *
 * La simulation d'`i18nMsg` rend le repli **et** enregistre ses appels : c'est ce qui
 * distingue une clé d'un littéral (mutation M11).
 *
 * ⚠️ `_loading` est un état de module, et `finalize` relâche son verrou par
 * `setTimeout(0)` après un rejet (`onboarding.svelte.ts`) : chaque scénario de
 * finalisation est un `it` distinct, et `atStep` relit l'état (ce qui remet `_loading`
 * à `false`).
 */
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';

const { i18nMsgMock, toastError } = vi.hoisted(() => ({
	i18nMsgMock: vi.fn((_key: string, fallback: string) => fallback),
	toastError: vi.fn(),
}));

vi.mock('$lib/app/stores/auth.svelte', () => ({
	authState: { currentUser: { role: 'Admin' }, isAuthenticated: true },
}));
vi.mock('$lib/features/onboarding/onboarding.api', () => ({
	fetchState: vi.fn(),
	setLanguage: vi.fn(),
	setMode: vi.fn(),
	seedDemo: vi.fn(),
	resetDemo: vi.fn(),
	startProduction: vi.fn(),
	setOrgType: vi.fn(),
	setAccountingLanguage: vi.fn(),
	setCoordinates: vi.fn(),
	setBankAccount: vi.fn(),
	skipBank: vi.fn(),
	finalize: vi.fn(),
}));
vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: i18nMsgMock,
	loadI18nMessages: vi.fn(),
}));
vi.mock('$app/navigation', () => ({ goto: vi.fn() }));
vi.mock('svelte-sonner', () => ({ toast: { success: vi.fn(), error: toastError } }));

import * as api from '$lib/features/onboarding/onboarding.api';
import { onboardingState } from '$lib/features/onboarding/onboarding.svelte';
import Page from './+page.svelte';

type Etat = { stepCompleted: number; isDemo: boolean; uiMode: null; isStub: boolean };
const etat = (stepCompleted: number): Etat => ({
	stepCompleted,
	isDemo: false,
	uiMode: null,
	isStub: false,
});

async function atStep(stepCompleted: number) {
	vi.mocked(api.fetchState).mockResolvedValue(etat(stepCompleted));
	await onboardingState.fetchState();
}

function remplir(id: string, valeur: string) {
	const el = document.getElementById(id) as HTMLInputElement;
	el.value = valeur;
	return fireEvent.input(el);
}

function soumettre() {
	const form = document.querySelector('form') as HTMLFormElement;
	return fireEvent.submit(form);
}

type Site = {
	nom: string;
	etape: number;
	appel: keyof typeof api;
	cle: string;
	repli: string;
	declencher: () => Promise<unknown>;
};

/** L'inventaire de la fiche, S1 à S8 et S10 (S9, S11 : la finalisation, plus bas). */
const SITES: Site[] = [
	{
		nom: 'S1 langue',
		etape: 0,
		appel: 'setLanguage',
		cle: 'onboarding-language-error',
		repli: 'Erreur lors du choix de langue',
		declencher: () => fireEvent.click(screen.getByText('Français')),
	},
	{
		nom: 'S2 mode',
		etape: 1,
		appel: 'setMode',
		cle: 'onboarding-mode-error',
		repli: 'Erreur lors du choix de mode',
		declencher: () => fireEvent.click(screen.getByText('Guidé')),
	},
	{
		nom: 'S3 démonstration',
		etape: 2,
		appel: 'seedDemo',
		cle: 'onboarding-demo-load-error',
		repli: 'Erreur lors du chargement des données de démo',
		declencher: () => fireEvent.click(screen.getByText('Explorer avec des données de démo')),
	},
	{
		nom: 'S4 production',
		etape: 2,
		appel: 'startProduction',
		cle: 'onboarding-setup-start-error',
		repli: 'Erreur lors du démarrage de la configuration',
		declencher: () => fireEvent.click(screen.getByText('Configurer pour la production')),
	},
	{
		nom: 'S5 type d’organisation',
		etape: 3,
		appel: 'setOrgType',
		cle: 'onboarding-org-type-error',
		repli: 'Erreur lors du choix du type',
		declencher: () => fireEvent.click(screen.getByTestId('onboarding-org-type-independant')),
	},
	{
		nom: 'S6 langue comptable',
		etape: 4,
		appel: 'setAccountingLanguage',
		cle: 'onboarding-accounting-lang-error',
		repli: 'Erreur lors du choix de la langue comptable',
		declencher: () => fireEvent.click(screen.getByText('Français')),
	},
	{
		nom: 'S7 coordonnées',
		etape: 5,
		appel: 'setCoordinates',
		cle: 'onboarding-address-save-error',
		repli: 'Erreur lors de la sauvegarde des coordonnées',
		declencher: async () => {
			// Les gardes client exigent nom, NPA et localité avant l'appel.
			await remplir('coord-name', 'Exemple Sàrl');
			await remplir('coord-npa', '1000');
			await remplir('coord-city', 'Lausanne');
			return soumettre();
		},
	},
	{
		nom: 'S8 compte bancaire',
		etape: 6,
		appel: 'setBankAccount',
		cle: 'onboarding-bank-save-error',
		repli: 'Erreur lors de la sauvegarde du compte bancaire',
		declencher: async () => {
			await remplir('bank-name', 'Banque Exemple');
			await remplir('bank-iban', 'CH93 0076 2011 6238 5295 7');
			return soumettre();
		},
	},
	{
		nom: 'S10 saut du compte bancaire',
		etape: 6,
		appel: 'skipBank',
		cle: 'onboarding-bank-skip-error',
		repli: 'Erreur lors du saut du compte bancaire',
		declencher: () => fireEvent.click(screen.getByText('Configurer plus tard')),
	},
];

const BLANCS = new Set(['S1 langue', 'S7 coordonnées', 'S8 compte bancaire']);

let consoleError: ReturnType<typeof vi.spyOn>;

beforeEach(() => {
	toastError.mockClear();
	i18nMsgMock.mockClear();
	consoleError = vi.spyOn(console, 'error').mockImplementation(() => {});
});

afterEach(() => {
	consoleError.mockRestore();
});

async function jouer(site: Site, rejet: unknown) {
	await atStep(site.etape);
	vi.mocked(api[site.appel] as (...a: unknown[]) => Promise<Etat>).mockRejectedValueOnce(rejet);
	const { unmount } = render(Page);
	await site.declencher();
	await vi.waitFor(() => expect(toastError).toHaveBeenCalledTimes(1));
	unmount();
}

describe('assistant — le motif du refus (#809, AC-2, AC-3)', () => {
	it.each(SITES.map((s) => [s.nom, s] as const))(
		'%s : une ApiError à message → ce message, tel quel',
		async (_nom, site) => {
			const motif = `MOTIF-${site.nom.split(' ')[0]}`;
			await jouer(site, { code: 'VALIDATION_ERROR', message: motif, status: 400 });
			expect(toastError).toHaveBeenCalledWith(motif);
			expect(consoleError).not.toHaveBeenCalled();
		}
	);

	it.each(SITES.map((s) => [s.nom, s] as const))(
		'%s : une TypeError → le message de l’étape, journalisée',
		async (_nom, site) => {
			const panne = new TypeError('Failed to fetch');
			await jouer(site, panne);
			expect(toastError).toHaveBeenCalledWith(site.repli);
			expect(i18nMsgMock).toHaveBeenCalledWith(site.cle, site.repli);
			expect(consoleError).toHaveBeenCalledWith('[errorMessageOf]', panne);
		}
	);

	it.each(SITES.filter((s) => BLANCS.has(s.nom)).map((s) => [s.nom, s] as const))(
		'%s : une ApiError à message blanc → le message de l’étape',
		async (_nom, site) => {
			await jouer(site, { code: 'VALIDATION_ERROR', message: '  ', status: 400 });
			expect(toastError).toHaveBeenCalledWith(site.repli);
		}
	);
});

describe('assistant — la finalisation dit son motif (#809, AC-4)', () => {
	const DEJA = "L'onboarding est déjà finalisé. Veuillez recharger la page.";
	const ECHEC = 'Finalisation échouée — réessayez.';

	const CHEMINS = [
		{
			nom: 'S9 après la banque',
			preparer: () => vi.mocked(api.setBankAccount).mockResolvedValueOnce(etat(7)),
			declencher: async () => {
				await remplir('bank-name', 'Banque Exemple');
				await remplir('bank-iban', 'CH93 0076 2011 6238 5295 7');
				return soumettre();
			},
		},
		{
			nom: 'S11 après le saut',
			preparer: () => vi.mocked(api.skipBank).mockResolvedValueOnce(etat(7)),
			declencher: () => fireEvent.click(screen.getByText('Configurer plus tard')),
		},
	];

	async function finaliser(chemin: (typeof CHEMINS)[number], rejet: unknown) {
		await atStep(6);
		chemin.preparer();
		vi.mocked(api.finalize).mockRejectedValueOnce(rejet);
		const { unmount } = render(Page);
		await chemin.declencher();
		await vi.waitFor(() => expect(toastError).toHaveBeenCalledTimes(1));
		unmount();
		// Laisser `finalize` relâcher son verrou (`setTimeout(0)`) avant le scénario suivant.
		await new Promise((r) => setTimeout(r, 0));
	}

	for (const chemin of CHEMINS) {
		it(`${chemin.nom} : un refus VALIDATION_ERROR → son motif`, async () => {
			await finaliser(chemin, { code: 'VALIDATION_ERROR', message: 'MOTIF-FIN', status: 400 });
			expect(toastError).toHaveBeenCalledWith('MOTIF-FIN');
		});

		it(`${chemin.nom} : ONBOARDING_STEP_ALREADY_COMPLETED → l'avis propre, non le message du serveur`, async () => {
			await finaliser(chemin, {
				code: 'ONBOARDING_STEP_ALREADY_COMPLETED',
				message: 'MOTIF-DEJA',
				status: 400,
			});
			expect(toastError).toHaveBeenCalledWith(DEJA);
			expect(toastError).not.toHaveBeenCalledWith('MOTIF-DEJA');
		});

		it(`${chemin.nom} : une TypeError → le message de la finalisation`, async () => {
			await finaliser(chemin, new TypeError('Failed to fetch'));
			expect(toastError).toHaveBeenCalledWith(ECHEC);
		});
	}
});

describe('assistant — le champ Pays indique la forme attendue (#809, AC-6)', () => {
	it('l’indication est sous le champ, reliée par aria-describedby, sans maxlength', async () => {
		await atStep(5);
		const { unmount } = render(Page);
		const champ = document.getElementById('coord-country') as HTMLInputElement;
		expect(champ.getAttribute('aria-describedby')).toBe('coord-country-hint');
		const indication = document.getElementById('coord-country-hint');
		expect(indication?.textContent).toContain('code ISO à deux lettres');
		expect(champ.hasAttribute('maxlength')).toBe(false);
		expect(i18nMsgMock).toHaveBeenCalledWith(
			'onboarding-field-country-hint',
			'code ISO à deux lettres, ex. CH, FR, DE'
		);
		unmount();
	});
});
