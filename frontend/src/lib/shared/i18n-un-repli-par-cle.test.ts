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
 */
import { describe, it, expect } from 'vitest';
import { readFileSync, readdirSync } from 'node:fs';
import { join } from 'node:path';
import { findCallSites, readFallback, masquerCommentaires } from './i18n-literal-reader.js';
import { dansLePerimetreDeFichier } from './i18n-harvest.js';

const RACINE = 'src';
const PREFIXES = ['supplier-invoices-', 'imported-supplier-invoices-'];

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
	it('le relevé n’est pas vide', () => {
		expect(replisParCle().size).toBeGreaterThanOrEqual(90);
	});
});
