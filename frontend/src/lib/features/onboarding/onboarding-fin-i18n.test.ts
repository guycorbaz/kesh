// @vitest-environment node
/**
 * Story 36-17 (#1187, AC9, AC6) — les textes de la fin de l'assistant.
 *
 * Pour chaque clé que la story écrit ou réécrit (liste FERMÉE ci-dessous) :
 * - elle existe dans les **quatre** catalogues ;
 * - l'ensemble de ses replis littéraux, dans les sources de l'assistant et de la
 *   bannière, est **exactement** `[valeur fr-CH]` ;
 * - les deux clés mortes de D2 (`onboarding-already-finalized`,
 *   `onboarding-finalize-incomplete`) ne figurent plus dans aucun catalogue ni aucune
 *   source.
 *
 * Angle mort écrit : les autres clés `onboarding-*` (étapes 0 à 6, antérieures à la
 * story) restent hors de cette garde (#758).
 */
import { describe, it, expect } from 'vitest';
import { catalogueDe, releveDesReplis, sourcesDesDossiers } from '$lib/shared/i18n-repli-catalogue';

const RACINE_FTL = '../crates/kesh-i18n/locales';
const LOCALES = ['fr-CH', 'de-CH', 'it-CH', 'en-CH'] as const;
const DOSSIERS = ['src/lib/features/onboarding', 'src/routes/onboarding', 'src/lib/shared/components'];

/** Les clés écrites ou réécrites par la story (fiche, T5). */
const CLES = [
	'incomplete-banner-text',
	'incomplete-banner-cta',
	'onboarding-state-load-error',
	'onboarding-retry',
	'onboarding-finalizing',
	'onboarding-finalizing-desc',
	// `onboarding-finalize-remedy` retirée à l'intégration de la 0.18.0 (la 36-19 corrige #1200).
	'onboarding-finalize-error',
	'onboarding-done-title',
	'onboarding-done-created',
	'onboarding-done-company',
	'onboarding-done-chart',
	'onboarding-done-fiscal-year',
	'onboarding-done-bank-account',
	// Revue P1 (B7/E7, E10).
	'onboarding-done-credit-card',
	'onboarding-finalize-remedy-link',
	'onboarding-done-summary-error',
	'onboarding-done-todo',
	'onboarding-done-todo-vat',
	'onboarding-done-todo-bank',
	'onboarding-start',
	'onboarding-demo-loading',
	'onboarding-demo-ready',
	'onboarding-demo-navigate-error',
	'onboarding-fallback',
	'onboarding-fallback-reload',
];
const MORTES = ['onboarding-already-finalized', 'onboarding-finalize-incomplete'];

describe('fin de l’assistant — textes (#1187, AC9)', () => {
	const fr = catalogueDe(RACINE_FTL, 'fr-CH');
	const parCle = releveDesReplis(sourcesDesDossiers(DOSSIERS));

	it('chaque clé existe dans les quatre catalogues', () => {
		for (const locale of LOCALES) {
			const valeurs = catalogueDe(RACINE_FTL, locale);
			expect(CLES.filter((c) => !valeurs.get(c)), locale).toEqual([]);
		}
	});

	it.each(CLES)('%s : demandée, et son seul repli est la valeur fr-CH', (cle) => {
		const replis = parCle.get(cle);
		expect(replis, `${cle} demandée par un site littéral`).toBeDefined();
		expect([...replis!]).toEqual([fr.get(cle)]);
	});

	it('le texte neuf de la bannière (D5)', () => {
		expect(fr.get('incomplete-banner-text')).toBe("Configuration incomplète : terminez-la avant d'utiliser Kesh.");
	});

	it('les clés mortes de D2 ont disparu des catalogues et des sources', () => {
		for (const locale of LOCALES) {
			const valeurs = catalogueDe(RACINE_FTL, locale);
			expect(MORTES.filter((c) => valeurs.has(c)), locale).toEqual([]);
		}
		expect(MORTES.filter((c) => parCle.has(c))).toEqual([]);
	});
});
