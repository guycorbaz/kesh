/**
 * L'écran de chaque état de l'assistant de configuration — Story 36-17 (#1187).
 *
 * Avant la story, la page de l'assistant n'avait d'écran que pour les étapes 0 à 6 : aux
 * étapes 7 (compte bancaire saisi ou passé, finalisation pas encore faite) et 8
 * (finalisée), elle ne rendait rien — la « page blanche avec seulement le logo » de
 * #1187. Ce module fixe la correspondance **état → écran** (fiche, D8), sans `{:else}`
 * vide possible : toute combinaison a un écran, au pire celui de repli.
 *
 * Fonctions pures, sans état ni appel d'API ; la page les lit et rend l'écran nommé.
 */

/** Les écrans de l'assistant ; chacun porte `data-testid="onboarding-screen-<id>"`. */
export type OnboardingScreen =
	| '0'
	| '1'
	| '2'
	| '3'
	| '4'
	| '5'
	| '6'
	| 'unloaded'
	| 'finalizing'
	| 'finalize-error'
	| 'resume'
	| 'done'
	| 'demo-loading'
	| 'demo-ready'
	| 'fallback';

/** Ce que la page sait de l'assistant : l'état du store et ses propres drapeaux. */
export interface OnboardingScreenInput {
	/** L'état a été lu au moins une fois (`onboardingState.loaded`). */
	loaded: boolean;
	stepCompleted: number;
	isDemo: boolean;
	/**
	 * Une finalisation est en cours, ou l'étape bancaire qui la précède (drapeau local de
	 * la page, posé AVANT l'appel de l'étape bancaire : sans lui, l'état « étape 7, aucun
	 * appel » existerait entre la fin de `skipBank` et le début de `finalize`). Lu à
	 * l'étape 7 seulement.
	 */
	finalizing: boolean;
	/** La dernière finalisation a échoué (drapeau local). Lu à l'étape 7 seulement. */
	finalizeFailed: boolean;
	/** Les données de démonstration se chargent (drapeau local). Lu aux étapes 0 à 2. */
	demoLoading: boolean;
}

/**
 * L'écran à rendre (fiche, D8). À l'étape 7 du chemin production, précédence :
 * échec > en cours > reprise. « Terminé » se lit `≥ 8` (une restauration pose 8 au
 * moins, le préréglage de test `post-onboarding` pose 10).
 */
export function onboardingScreenFor(s: OnboardingScreenInput): OnboardingScreen {
	if (!s.loaded) return 'unloaded';
	const step = s.stepCompleted;
	if (!Number.isInteger(step) || step < 0) return 'fallback';
	if (step <= 2) {
		if (s.demoLoading) return 'demo-loading';
		return String(step) as OnboardingScreen;
	}
	if (s.isDemo) return 'demo-ready';
	if (step <= 6) return String(step) as OnboardingScreen;
	if (step === 7) {
		if (s.finalizeFailed) return 'finalize-error';
		if (s.finalizing) return 'finalizing';
		return 'resume';
	}
	return 'done';
}

/**
 * La bannière « Configuration incomplète » de l'application (fiche, D5) : chemin
 * production, état lu, étapes 6 et 7. L'application est ouverte dès l'étape 6
 * (`(app)/+layout.ts`) ; à l'étape 7, la finalisation n'est pas faite (exercice, taux de
 * TVA, réglages de facturation absents) et la bannière le rappelle.
 */
export function showIncompleteBanner(s: {
	loaded: boolean;
	stepCompleted: number;
	isDemo: boolean;
}): boolean {
	return !s.isDemo && s.loaded && s.stepCompleted >= 6 && s.stepCompleted < 8;
}

/**
 * Les langues proposées par l'assistant (interface à l'étape 0, langue comptable à
 * l'étape 4) et leur nom affiché, partagés par les deux écrans et le récapitulatif de la
 * page terminée. Les noms sont ceux de chaque langue dans sa propre langue : ils ne se
 * traduisent pas.
 */
export const ONBOARDING_LANGUAGES: ReadonlyArray<readonly [code: string, label: string]> = [
	['FR', 'Français'],
	['DE', 'Deutsch'],
	['IT', 'Italiano'],
	['EN', 'English'],
];

/** Le nom affiché d'une langue de l'assistant ; le code tel quel s'il est inconnu. */
export function onboardingLanguageLabel(code: string): string {
	return ONBOARDING_LANGUAGES.find(([c]) => c === code.toUpperCase())?.[1] ?? code;
}
