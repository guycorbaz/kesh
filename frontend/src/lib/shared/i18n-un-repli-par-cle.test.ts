/**
 * Garde « une clé, un repli » — sur le domaine `supplier-invoices`.
 *
 * ⚠️ **Cette garde existe parce qu'un défaut RÉEL l'a rendue nécessaire, et il n'était
 * visible ni au test, ni à l'écran.** `supplier-invoices-col-total` servait TROIS sites
 * avec deux replis : « TTC » sur le total de la facture (`invoice.totalAmount`, documenté
 * *TTC (Σ HT + Σ TVA)* en `supplier_invoice.rs:29`) et « Total HT » sur le total d'une
 * ligne (`line.lineTotal`, documenté *quantity × unit_price (HT)* en `:89`). Deux
 * grandeurs différentes sous une clé unique.
 *
 * ⚠️ **Le défaut était LATENT, et c'est la TRADUCTION qui l'aurait activé.** Tant que la
 * clé manque des quatre catalogues, `i18nMsg` retombe sur le repli du site appelant :
 * chaque écran affiche donc le bon libellé, par accident. Entrer une valeur unique au
 * catalogue l'impose aux trois sites — « TTC » au-dessus d'une colonne de montants hors
 * taxe, sur une facture fournisseur. (Story 23-3.)
 *
 * ⚠️ **Pourquoi cette garde et non le moissonneur** : le moissonneur signale bien les
 * replis divergents, mais il ne voit QUE les clés absentes des catalogues. Une fois la
 * traduction livrée, il cesserait de les voir et se tairait. Une garde qui s'éteint au
 * moment où le risque devient réel n'en est pas une — c'est le mode d'échec du test muet,
 * déjà payé plusieurs fois sur ce dépôt. Celle-ci lit les SOURCES, pas les catalogues.
 *
 * ⚠️ **CE QU'ELLE NE VOIT PAS, et il faut le savoir avant de s'y fier.** Les deux filtres
 * `kind !== 'literal'` ci-dessous écartent, par construction, deux familles :
 *
 *   1. **les clés construites par gabarit** — `i18nMsg(`…-error-${code}`, …)` : la clé n'est
 *      pas connue statiquement, donc deux sites bâtissant la MÊME clé dynamique avec des
 *      replis différents passeraient inaperçus. Au 2026-08-20, un seul site construit
 *      `imported-supplier-invoices-error-*` — `lib/features/imported-supplier-invoices/
 *      error-label.ts`, vérifié au grep. ⚠️ Cette référence a cité `import/+page.svelte:68`
 *      jusqu'à la passe 5, alors que la passe 4 avait déplacé le code : un lecteur suivant
 *      l'invitation à vérifier « au grep » n'aurait rien trouvé là où on l'envoyait —
 *      la divergence est donc impossible aujourd'hui, non parce que la garde l'interdit,
 *      **mais parce qu'il n'y a rien à faire diverger** ;
 *   2. **les replis non littéraux** — un repli calculé (gabarit, ternaire, variable) n'est
 *      pas comparable textuellement et n'entre pas au relevé.
 *
 * C'est la même famille d'angle mort que #255 (chaîne en dur qui n'appelle jamais
 * `i18nMsg`) : une garde qui ne dit pas ce qu'elle ne couvre pas se fait lire comme si elle
 * couvrait tout. Un second site sur une clé dynamique du domaine devra être gardé
 * autrement — par l'énumération explicite des suffixes, comme le fait la carte de
 * `import/+page.svelte`. *(Écrit en passe 2 de revue, story 23-3.)*
 */
import { describe, it, expect } from 'vitest';
import { readFileSync, readdirSync } from 'node:fs';
import { join } from 'node:path';
import { findCallSites, readFallback, masquerCommentaires } from './i18n-literal-reader.js';
import { dansLePerimetreDeFichier } from './i18n-harvest.js';

const RACINE = 'src';
const PREFIXES = ['supplier-invoices-', 'imported-supplier-invoices-'];

/** Nombre de clés du domaine relevées dans les sources. Recompté, jamais ajusté. */
const CLES_RELEVEES = 110;


/** Relève, pour chaque clé du domaine, l'ensemble de ses replis littéraux distincts. */
function replisParCle(): Map<string, Map<string, string[]>> {
	/** @type {Map<string, Map<string, string[]>>} */
	const trouves = new Map<string, Map<string, string[]>>();
	const parcourir = (rep: string) => {
		for (const e of readdirSync(rep, { withFileTypes: true })) {
			const chemin = join(rep, e.name);
			if (e.isDirectory()) {
				parcourir(chemin);
				continue;
			}
			if (!dansLePerimetreDeFichier(e.name)) continue;
			const source = masquerCommentaires(readFileSync(chemin, 'utf-8'));
			for (const site of findCallSites(source)) {
				if (site.arg?.kind !== 'literal') continue;
				const cle = site.arg.value;
				if (!PREFIXES.some((p) => cle.startsWith(p))) continue;
				const repli = readFallback(source, site.afterFirstArg);
				if (repli === null || repli.kind !== 'literal') continue;
				if (!trouves.has(cle)) trouves.set(cle, new Map());
				const parTexte = trouves.get(cle)!;
				if (!parTexte.has(repli.value)) parTexte.set(repli.value, []);
				parTexte.get(repli.value)!.push(`${chemin}:${site.line}`);
			}
		}
	};
	parcourir(RACINE);
	return trouves;
}

describe('une clé, un repli — domaine supplier-invoices', () => {
	it('aucune clé ne porte deux replis différents', () => {
		const divergents = [...replisParCle()]
			.filter(([, parTexte]) => parTexte.size > 1)
			.map(([cle, parTexte]) => `${cle} → ${[...parTexte.keys()].map((t) => `« ${t} »`).join(' / ')}`);
		expect(divergents).toEqual([]);
	});

	it('les deux totaux restent DEUX clés — la fusion ne peut pas revenir', () => {
		const releve = replisParCle();
		// ⚠️ L'assertion porte sur les DEUX sens, pas seulement sur l'existence des clés :
		// une fusion qui ferait disparaître `-line-total` passerait un simple `toBeDefined`.
		const colTotal = releve.get('supplier-invoices-col-total');
		const ligneTotal = releve.get('supplier-invoices-line-total');
		expect([...(colTotal?.keys() ?? [])]).toEqual(['TTC']);
		expect([...(ligneTotal?.keys() ?? [])]).toEqual(['Total HT']);
	});

	// Borne anti-test-muet : si le relevé rendait un ensemble vide — lecteur cassé, arbre
	// déplacé, préfixes renommés —, les deux preuves ci-dessus seraient vertes à vide.
	//
	// ⚠️ **EXACTE, et non « au moins ».** La première rédaction posait `>= 90` pour un relevé
	// réel d'une centaine : une régression effaçant douze clés serait passée. C'est la doctrine
	// inverse de `i18n-keys.test.ts`, écrit dans le même commit, qui pose un `sitesTotal` exact
	// et dit pourquoi : **un écart se recompte, il ne s'ajuste pas.** Deux disciplines opposées
	// dans un même patch — relevé en passe 4 de revue. Si ce nombre rougit, recompter la cause
	// avant de le changer : une baisse est une clé qui a cessé d'être traduite.
	it('le relevé porte exactement les clés attendues du domaine', () => {
		expect(replisParCle().size).toBe(CLES_RELEVEES);
	});
});
