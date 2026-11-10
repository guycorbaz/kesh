// @vitest-environment node
/**
 * Story 34-3 (#967, AC-1, AC-4, AC-6) — les textes qui nomment l'écriture manuelle.
 *
 * ## 1. Le repli de chaque clé est sa valeur fr-CH
 *
 * `i18n-repli-divergent-actif.test.ts` ne retient que les clés dont les replis divergent
 * **entre plusieurs sites** ; chacune des clés ci-dessous n'en a qu'un, si bien qu'un repli
 * resté à l'ancien libellé (« Affecter manuellement ») n'y rougirait pas (validation P2 de la
 * fiche, P2-3-3). Cette garde compare, sur une **liste fermée**, le relevé des replis des sources
 * de l'écran au catalogue fr-CH, clé par clé, et asserte que chaque clé est **relevée** (un
 * site renommé ne la ferait pas disparaître en silence).
 *
 * `ecartsAuCatalogue` juge par préfixe et signalerait aussi les voisines sans rapport des
 * préfixes `reconciliation-labels-`, `reconciliation-failed-`, `reconciliation-manual-`
 * (validation P3, P3-3-2) : elle n'est donc pas employée ici.
 *
 * ## 2. Les quatre catalogues nomment le geste rebaptisé
 *
 * Pour les clés qui citent le geste, chaque locale porte le nom neuf (« Écriture manuelle »,
 * « Manuelle Buchung », « Manual entry », « Registrazione manuale ») ou sa forme verbale.
 *
 * Angle mort écrit : la garde ligne à ligne de l'AC-6 (`git grep`) reste la preuve qu'aucun
 * autre texte ne nomme encore l'ancien geste ; ce test ne voit que sa liste.
 */
import { describe, it, expect } from 'vitest';
import {
	catalogueDe,
	releveDesReplis,
	sourcesDesDossiers,
} from '$lib/shared/i18n-repli-catalogue';

const RACINE_FTL = '../crates/kesh-i18n/locales';
const LOCALES = ['fr-CH', 'de-CH', 'en-CH', 'it-CH'] as const;
const DOSSIER_ECRAN = 'src/lib/features/reconciliation';

/** Les clés qui citent le geste (AC-6) : quatre `failed-rule-masked*`, l'avis du lot, trois de F3-2. */
const CLES_QUI_CITENT = [
	'reconciliation-failed-rule-masked-payment-batch',
	'reconciliation-failed-rule-masked-invoice-candidate',
	'reconciliation-failed-rule-masked-overpaid',
	'reconciliation-failed-rule-masked',
	'reconciliation-labels-notice-supplier-invoice-in-batch',
	'reconciliation-labels-more-capped',
	'reconciliation-failed-supplier-invoice-in-batch',
	'reconciliation-failed-supplier-payment-hold',
] as const;

/** Les libellés du geste (AC-1) et les deux avis (AC-4). */
const CLES_DU_DIALOGUE = [
	'reconciliation-manual-button-label',
	'reconciliation-manual-modal-title',
	'reconciliation-manual-submit',
	'reconciliation-manual-claim-notice-receivable',
	'reconciliation-manual-claim-notice-payable',
] as const;

const LISTE_FERMEE = [...CLES_QUI_CITENT, ...CLES_DU_DIALOGUE];

/** Le nom neuf du geste, ou sa forme verbale, par locale. */
const NOM_DU_GESTE: Record<(typeof LOCALES)[number], RegExp> = {
	'fr-CH': /écriture manuelle/i,
	'de-CH': /manuelle Buchung/i,
	'en-CH': /manual entr(y|ies)/i,
	'it-CH': /registrazion[ei] manual[ei]|registrate manualmente/i,
};

describe('Écriture manuelle — replis et catalogues (Story 34-3)', () => {
	const releve = releveDesReplis(sourcesDesDossiers([DOSSIER_ECRAN]));
	const fr = catalogueDe(RACINE_FTL, 'fr-CH');

	it.each(LISTE_FERMEE)('%s : relevée, et son repli égale le catalogue fr-CH', (cle) => {
		const replis = releve.get(cle);
		expect(replis, `${cle} n’est demandée par aucun site littéral`).toBeDefined();
		expect([...replis!]).toEqual([fr.get(cle)]);
	});

	it('libellés exacts du bouton, du titre et de la validation (AC-1)', () => {
		expect(fr.get('reconciliation-manual-button-label')).toBe('Écriture manuelle');
		expect(fr.get('reconciliation-manual-modal-title')).toBe('Écriture manuelle');
		expect(fr.get('reconciliation-manual-submit')).toBe("Passer l'écriture");
		const attendu: Record<string, [string, string, string]> = {
			'de-CH': ['Manuelle Buchung', 'Manuelle Buchung', 'Buchung erfassen'],
			'en-CH': ['Manual entry', 'Manual entry', 'Post entry'],
			'it-CH': ['Registrazione manuale', 'Registrazione manuale', 'Registra'],
		};
		for (const [locale, [bouton, titre, valider]] of Object.entries(attendu)) {
			const c = catalogueDe(RACINE_FTL, locale);
			expect(c.get('reconciliation-manual-button-label'), locale).toBe(bouton);
			expect(c.get('reconciliation-manual-modal-title'), locale).toBe(titre);
			expect(c.get('reconciliation-manual-submit'), locale).toBe(valider);
		}
	});

	it.each(LOCALES)('%s : chaque clé qui cite le geste nomme le geste rebaptisé', (locale) => {
		const c = catalogueDe(RACINE_FTL, locale);
		for (const cle of CLES_QUI_CITENT) {
			const valeur = c.get(cle);
			expect(valeur, `${locale} ${cle}`).toBeDefined();
			expect(valeur, `${locale} ${cle}`).toMatch(NOM_DU_GESTE[locale]);
		}
		for (const cle of CLES_DU_DIALOGUE) expect(c.get(cle), `${locale} ${cle}`).toBeTruthy();
	});
});
