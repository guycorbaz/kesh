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
import { moissonner, fragmentFtl } from './i18n-harvest.js';

/** Aucune clé n'existe au catalogue, sauf mention contraire. */
const rienAuCatalogue = () => false;

describe('le moissonneur', () => {
	it("(i) ne moissonne PAS les fichiers de test — leurs clés sont fictives", () => {
		// `i18n.svelte.test.ts` demande `une-cle` et `compteur`, qui doivent le rester.
		// L'exclusion est faite par l'appelant (D5-bis) : ce test vérifie que la fonction
		// ne collecte QUE ce qu'on lui donne, donc que le filtre est le bon endroit.
		const moisson = moissonner(
			[{ chemin: 'a.svelte', source: "const x = i18nMsg('vraie-cle', 'V');" }],
			rienAuCatalogue
		);
		expect([...moisson.replis.keys()]).toEqual(['vraie-cle']);
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
