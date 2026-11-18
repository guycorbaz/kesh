// @vitest-environment node
/**
 * Story 37-2 (AC-8) — gardes des textes de l'écran *Cours de change* et du bandeau.
 *
 * - Le repli de chaque clé `exchange-rates-*` est **exactement** sa valeur fr-CH, et
 *   l'ensemble des clés relevées dans les sources égale, dans les deux sens, celui
 *   des clés du catalogue qui commencent par `exchange-rates-` (module partagé
 *   `$lib/shared/i18n-repli-catalogue`, comme les écrans du serveur MCP).
 * - Chaque clé existe dans les **quatre** catalogues.
 * - Aucun texte ne dit « cours moyen » ni « moyenne du mois » (terminologie : « cours
 *   mensuel AFC ») — en fr-CH, où la forme est univoque.
 *
 * Angle mort écrit : les autres locales ne sont contrôlées que sur la présence.
 */
import { describe, it, expect } from 'vitest';
import {
	catalogueDe,
	ecartsAuCatalogue,
	releveDesReplis,
	sourcesDesDossiers,
} from '$lib/shared/i18n-repli-catalogue';

const RACINE_FTL = '../crates/kesh-i18n/locales';
const LOCALES = ['fr-CH', 'de-CH', 'it-CH', 'en-CH'] as const;
const PREFIXE = 'exchange-rates-';
const DOSSIERS = [
	'src/lib/features/exchange-rates',
	// La page `settings/exchange-rates` et la carte de *Paramètres* (`exchange-rates-title`,
	// `exchange-rates-settings-card`) : le dossier est lu récursivement.
	'src/routes/(app)/settings',
];

/**
 * Le nombre exact de clés `exchange-rates-` (borne anti-test-muet). Recompté :
 * `grep -c '^exchange-rates-' crates/kesh-i18n/locales/fr-CH/messages.ftl`.
 */
const NOMBRE_DE_CLES = 81;

describe('écran des cours de change — textes (AC-8)', () => {
	const fr = catalogueDe(RACINE_FTL, 'fr-CH');

	it('les replis exchange-rates suivent le catalogue fr-CH', () => {
		const parCle = releveDesReplis(sourcesDesDossiers(DOSSIERS));
		const cles = [...parCle.keys()].filter((c) => c.startsWith(PREFIXE));
		expect(cles.length, 'relevé des sources').toBe(NOMBRE_DE_CLES);
		expect([...fr.keys()].filter((c) => c.startsWith(PREFIXE)).length, 'catalogue fr-CH').toBe(
			NOMBRE_DE_CLES,
		);
		expect(ecartsAuCatalogue(parCle, fr, PREFIXE)).toEqual([]);
	});

	it('chaque clé existe dans les quatre catalogues', () => {
		const cles = [...fr.keys()].filter((c) => c.startsWith(PREFIXE));
		for (const locale of LOCALES) {
			const valeurs = catalogueDe(RACINE_FTL, locale);
			expect(
				cles.filter((c) => !valeurs.get(c)),
				locale,
			).toEqual([]);
		}
	});

	it('jamais « cours moyen » ni « moyenne du mois »', () => {
		const textes = [...fr.entries()].filter(([c]) => c.startsWith(PREFIXE)).map(([, v]) => v);
		for (const t of textes) expect(t).not.toMatch(/cours moyen|moyenne du mois/i);
	});
});
