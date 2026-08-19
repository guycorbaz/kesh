/**
 * **Preuves du moissonneur** — story 23-1b, AC10 (d).
 *
 * ⚠️ **Ce test existe parce que rien n'exécutait le moissonneur.** `vite.config.ts` borne
 * vitest à `src/**` : un script de `scripts/` n'est lancé par aucun gate, et toute la
 * substance d'AC10 — périmètre, exclusion des tests, détection des conflits — n'aurait eu
 * pour preuve que deux nombres recopiés à la main dans un compte rendu. Un script qui
 * imprime deux tableaux en dur aurait passé la revue.
 *
 * Les trois premières fixtures reprennent **les trois défauts que cette story a réellement
 * payés** pendant ses passes de revue. Les inscrire ici est le seul geste qui empêche leur
 * récidive.
 */

import { describe, it, expect } from 'vitest';
import {
	moissonner,
	fragmentFtl,
	estFtlSain,
	estCleFtlSaine,
	dansLePerimetreDeFichier
} from './i18n-harvest.js';

/** Aucune clé n'existe au catalogue, sauf mention contraire. */
const rienAuCatalogue = () => false;

describe('le moissonneur', () => {
	it("(i) le PÉRIMÈTRE DE FICHIER exclut les tests — et la règle est ici, donc testée", () => {
		// ⚠️ Cette règle vivait dans le script, hors de tout gate, alors que le docstring
		// de ce fichier revendiquait l'« exclusion des tests » parmi ce qu'il prouve.
		// `i18n.svelte.test.ts` demande `une-cle` et `compteur` : des clés FICTIVES, qui
		// doivent le rester. (Revue de code 23-1b, passe 1.)
		expect(dansLePerimetreDeFichier('i18n.svelte.test.ts')).toBe(false);
		expect(dansLePerimetreDeFichier('contacts-page.test.ts')).toBe(false);
		expect(dansLePerimetreDeFichier('ContactPersonsManager.svelte')).toBe(true);
		expect(dansLePerimetreDeFichier('notify.ts')).toBe(true);
		expect(dansLePerimetreDeFichier('README.md')).toBe(false);
		expect(dansLePerimetreDeFichier('i18n-literal-reader.js')).toBe(false);

		const moisson = moissonner(
			[{ chemin: 'a.svelte', source: "const x = i18nMsg('vraie-cle', 'V');" }],
			rienAuCatalogue
		);
		expect([...moisson.replis.keys()]).toEqual(['vraie-cle']);
	});

	it("(i-ter) la COUTURE tient : `moissonner` applique le périmètre LUI-MÊME", () => {
		// ⚠️ La preuve qui manquait, et son absence n'était pas visible. Le test (i) prouve le
		// filtre, les autres prouvent le moissonneur — et **la chaîne restait non prouvée** :
		// retirer l'appel à `dansLePerimetreDeFichier` du script laissait les 8 tests VERTS.
		// Vérifié par mutation en passe 2 de revue. On donne donc ici un `.test.ts` DIRECTEMENT
		// au moissonneur : sa clé doit rester dehors sans que l'appelant ait rien filtré.
		const moisson = moissonner(
			[
				{ chemin: 'src/lib/x.svelte', source: "i18nMsg('vraie-cle', 'V');" },
				{ chemin: 'src/lib/i18n.svelte.test.ts', source: "i18nMsg('une-cle', 'mon repli');" },
				{ chemin: 'src/lib/notes.md', source: "i18nMsg('cle-de-doc', 'D');" }
			],
			rienAuCatalogue
		);
		expect([...moisson.replis.keys()]).toEqual(['vraie-cle']);
	});

	it("(i-quater) une CLÉ invalide en Fluent est écartée — l'autre moitié du signe `=`", () => {
		// ⚠️ La passe 1 ne gardait que la VALEUR. Une clé vide, numérique ou pointée produit
		// une ligne que le parseur rejette — et `loader.rs` propageant l'erreur sans tri, elle
		// emporte TOUTE la locale. (Revue de code 23-1b, passe 2.)
		expect(estCleFtlSaine('contact-persons-delete')).toBe(true);
		expect(estCleFtlSaine('')).toBe(false);
		expect(estCleFtlSaine('123')).toBe(false);
		expect(estCleFtlSaine('foo.bar')).toBe(false); // `.` = ATTRIBUT en Fluent, pas une clé
		expect(estCleFtlSaine('_x')).toBe(false);

		const moisson = moissonner(
			[
				{
					chemin: 'a.svelte',
					source: "i18nMsg('bonne-cle', 'B'); i18nMsg('foo.bar', 'M'); i18nMsg('123', 'N');"
				}
			],
			rienAuCatalogue
		);
		const fragment = fragmentFtl(moisson, '2026-08-19');
		expect(fragment).toContain('bonne-cle = B');
		expect(fragment).not.toContain('foo.bar');
		expect(fragment).not.toContain('123 =');
	});

	it("(i-bis) un repli qui casserait le .ftl est ÉCARTÉ du fragment, pas injecté", () => {
		// ⚠️ Un repli invalide ne casse pas une ligne : `loader.rs` propage l'erreur de
		// parse sans tri, donc **toute la locale** cesse de charger.
		expect(estFtlSain('Texte normal.')).toBe(true);
		expect(estFtlSain('Facture #{$id} enregistrée.')).toBe(true);
		expect(estFtlSain('{$n} facture(s) importée(s).')).toBe(true); // le second du Change Log, placeable LÉGITIME
		expect(estFtlSain('Ligne un\nLigne deux')).toBe(false); // retour à la ligne
		expect(estFtlSain('Valeur { non fermée')).toBe(false); // accolade non appariée
		expect(estFtlSain('Fermeture } orpheline')).toBe(false);

		const moisson = moissonner(
			[
				{ chemin: 'a.svelte', source: "i18nMsg('saine', 'Texte {$n} normal');" },
				{ chemin: 'b.svelte', source: "i18nMsg('cassee', 'Valeur { non fermée');" }
			],
			rienAuCatalogue
		);
		expect(moisson.aEchapper.map(([c]) => c)).toEqual(['cassee']);
		const fragment = fragmentFtl(moisson, '2026-08-19');
		expect(fragment).toContain('saine = Texte {$n} normal');
		expect(fragment).not.toContain('cassee');
	});

	it('(ii) signale une clé demandée avec DEUX replis différents', () => {
		// Sept clés du dépôt sont dans ce cas. Un moissonneur qui garde le dernier vu
		// fige silencieusement le mauvais libellé — sur une colonne de tableau vue par
		// l'utilisateur.
		const moisson = moissonner(
			[
				{ chemin: 'a.svelte', source: "i18nMsg('col-total', 'Total');" },
				{ chemin: 'b.svelte', source: "i18nMsg('col-total', 'Montant');" }
			],
			rienAuCatalogue
		);
		expect(moisson.divergents).toHaveLength(1);
		const [cle, parTexte] = moisson.divergents[0];
		expect(cle).toBe('col-total');
		expect([...parTexte.keys()].sort()).toEqual(['Montant', 'Total']);
	});

	it("(iii) lit ENTIER un repli entre guillemets doubles contenant une apostrophe", () => {
		// `payment-batches-col-date` — le septième conflit, manqué deux fois, y compris
		// par le script écrit pour le vérifier. Une classe `[^"]` s'arrête sur l'apostrophe.
		const moisson = moissonner(
			[{ chemin: 'a.svelte', source: `i18nMsg('col-date', "Date d'exécution");` }],
			rienAuCatalogue
		);
		expect([...(moisson.replis.get('col-date') ?? new Map()).keys()]).toEqual([
			"Date d'exécution"
		]);
	});

	it('(iv) classe à part une clé dont le repli est INTERPOLÉ', () => {
		// Les cinq clés de `TransactionSplitModal` : elles demandent une entrée Fluent à
		// variables, que le moissonneur ne peut pas proposer. Elles vont sur la sortie
		// d'erreur, pas dans le fragment.
		const moisson = moissonner(
			[{ chemin: 'a.svelte', source: 'i18nMsg(`ligne-erreur`, `Ligne ${i + 1} : compte requis`);' }],
			rienAuCatalogue
		);
		expect(moisson.replis.has('ligne-erreur')).toBe(false);
		expect([...moisson.sansRepli.keys()]).toEqual(['ligne-erreur']);
	});

	it('(v) une clé qui a un repli littéral AILLEURS n’est pas « sans repli »', () => {
		const moisson = moissonner(
			[
				{ chemin: 'a.svelte', source: 'i18nMsg(`k`, `Ligne ${i}`);' },
				{ chemin: 'b.svelte', source: "i18nMsg('k', 'Libellé net');" }
			],
			rienAuCatalogue
		);
		expect(moisson.sansRepli.size).toBe(0);
		expect([...(moisson.replis.get('k') ?? new Map()).keys()]).toEqual(['Libellé net']);
	});

	it('(vi) ne moissonne QUE les clés absentes des catalogues', () => {
		const moisson = moissonner(
			[{ chemin: 'a.svelte', source: "i18nMsg('deja-la', 'X'); i18nMsg('absente', 'Y');" }],
			(cle) => cle === 'deja-la'
		);
		expect([...moisson.replis.keys()]).toEqual(['absente']);
	});

	it('(vii) le fragment est trié, daté, et signale les conflits', () => {
		const moisson = moissonner(
			[
				{ chemin: 'a.svelte', source: "i18nMsg('zeta', 'Z'); i18nMsg('alpha', 'A');" },
				{ chemin: 'b.svelte', source: "i18nMsg('alpha', 'Autre');" }
			],
			rienAuCatalogue
		);
		const lignes = fragmentFtl(moisson, '2026-08-19').split('\n');
		expect(lignes[0]).toContain('2026-08-19');
		expect(lignes[0]).toContain('À RELIRE');
		expect(lignes.filter((l) => !l.startsWith('#'))).toEqual(['alpha = A', 'zeta = Z']);
		expect(lignes.some((l) => l.startsWith('# CONFLIT'))).toBe(true);
	});
});
