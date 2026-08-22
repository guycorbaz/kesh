/**
 * Garde « le catalogue ne ment pas en français » — divergences ACTIVES.
 *
 * ⚠️ **Elle ne garde pas la même chose que `i18n-un-repli-par-cle.test.ts`, et la
 * différence est le mot ACTIVE.** Cette dernière interdit qu'une clé porte deux replis
 * divergents, sur les domaines qu'elle énumère — un défaut alors **latent** : tant que la
 * clé manque des catalogues, `i18nMsg` retombe sur le repli du site appelant et chaque
 * écran affiche le bon libellé. C'est la traduction qui l'activerait.
 *
 * Celle-ci croise le relevé avec le catalogue `fr-CH` et ne retient que les clés qui y
 * **figurent déjà**. Pour celles-là, `i18nMsg` fait `_messages[key] || fallback`
 * (`utils/i18n.svelte.ts:15`) : le catalogue gagne, le repli n'est jamais lu, et l'un des
 * deux sites affiche **aujourd'hui, en français, un message qui n'est pas le sien**.
 * Ce n'est pas une dette de traduction — c'est un défaut d'affichage en production.
 *
 * ⚠️ **Sans filtre de préfixe, délibérément.** La garde par domaines n'aurait rien vu des
 * six cas corrigés le 2026-08-22 : ils portaient sur `error-internal`, `error-validation`
 * et `error-unexpected`, des clés **transverses** qu'aucun préfixe de domaine ne couvre.
 * Le plus lourd — dix `catch` de l'onboarding effondrés sur « Erreur interne » — vivait
 * dans le tout premier écran qu'un nouvel utilisateur voit.
 *
 * ⚠️ **La borne anti-test-muet est l'allowlist elle-même, et c'est voulu.** Chaque entrée
 * de `TOLEREES` doit *encore* être divergente : si le lecteur casse, si l'arbre est
 * déplacé, si `findCallSites` cesse de rendre des sites, le relevé se vide — les douze
 * entrées cessent d'être divergentes et le second test rougit. Un compteur exact du
 * relevé aurait le même effet, mais rougirait aussi à chaque clé ajoutée par une story
 * i18n ordinaire : coût récurrent pour un contrôle qu'on obtient ici gratuitement.
 * *(Arbitrage du 2026-08-22 — régime allégé sur l'i18n : on traduit, on ne fignole pas.)*
 */
import { describe, it, expect } from 'vitest';
import { readFileSync, readdirSync } from 'node:fs';
import { join } from 'node:path';
import { findCallSites, readFallback, masquerCommentaires } from './i18n-literal-reader.js';
import { dansLePerimetreDeFichier } from './i18n-harvest.js';

const RACINE = 'src';
const RACINE_FTL = '../crates/kesh-i18n/locales';

/**
 * Divergences actives tolérées — relevé du 2026-08-22, douze entrées.
 *
 * ⚠️ **Le critère de tolérance est « imprécision » contre « contresens ».** Une entrée
 * n'est admissible ici que si les deux replis désignent la MÊME chose et que la valeur du
 * catalogue reste vraie sur les deux sites, fût-elle mal calibrée (trop longue pour une
 * colonne, trop courte pour un titre). Dès qu'un site affiche une phrase qui *dit autre
 * chose* — nomme la mauvaise opération, perd le diagnostic, désigne un autre objet —, ce
 * n'est plus une imprécision : la clé doit être scindée, comme l'ont été les six du
 * 2026-08-22.
 */
const TOLEREES: readonly { cle: string; motif: string }[] = [
	{
		cle: 'common-loading',
		motif: "typographie seule — « Chargement... » contre « Chargement… » (U+2026) ; le catalogue porte la forme correcte",
	},
	{
		cle: 'admin-restore-error-invalid',
		motif: "typographie seule — apostrophe droite contre courbe, même phrase, dans le même fichier",
	},
	{
		cle: 'bank-accounts-actions-link-account',
		motif: "« Lier » en colonne d'actions contre « Lier au plan comptable » en titre de formulaire ; le catalogue impose la forme longue dans une colonne étroite — gêne de largeur, pas contresens",
	},
	{
		cle: 'bank-import-profile-labels-bank-name',
		motif: "en-tête de colonne « Banque » contre étiquette de champ « Nom de la banque » — même grandeur, calibrage différent",
	},
	{
		cle: 'bank-import-profile-labels-filename-pattern',
		motif: "même cause que `-bank-name` : en-tête de colonne contre étiquette de champ",
	},
	{
		cle: 'bank-import-profile-labels-encoding',
		motif: "même cause que `-bank-name` : en-tête de colonne contre étiquette de champ",
	},
	{
		cle: 'reports-column-account-number',
		motif: "« N° » dans trois tableaux serrés contre « N° de compte » dans le bilan — même colonne, calibrage différent",
	},
	{
		cle: 'reports-export-csv-button',
		motif: "« Export CSV » contre « Exporter CSV » — synonymes, même action",
	},
	{
		cle: 'contact-filter-search-placeholder',
		motif: "placeholder détaillé du champ principal contre « Rechercher… » d'un second champ du même écran — même fonction",
	},
	{
		cle: 'product-filter-search',
		motif: "même cause que `contact-filter-search-placeholder`",
	},
	{
		cle: 'invoice-pdf-error-generic',
		motif: "« Échec du téléchargement du PDF. » contre « Erreur lors du téléchargement du PDF » — synonymes, même échec",
	},
	{
		cle: 'dunning-edit',
		motif: "« Modifier » en bouton de ligne contre « Modifier le niveau » en titre de formulaire — le catalogue affiche la forme courte, perte de précision sans ambiguïté",
	},
];

// ─────────────────────────────────────────────────────────────────────────────

function clesDuCatalogueFr(): Set<string> {
	const texte = readFileSync(join(RACINE_FTL, 'fr-CH', 'messages.ftl'), 'utf-8');
	const cles = new Set<string>();
	for (const ligne of texte.split('\n')) {
		const m = /^([a-zA-Z][\w-]*)\s*=/.exec(ligne);
		if (m) cles.add(m[1]);
	}
	return cles;
}

/** Relève, pour chaque clé, l'ensemble de ses replis littéraux distincts et leurs sites. */
function replisParCle(): Map<string, Map<string, string[]>> {
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
				const repli = readFallback(source, site.afterFirstArg);
				if (repli === null || repli.kind !== 'literal') continue;
				const cle = site.arg.value;
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

/** Les clés dont les replis divergent ET qui sont déjà servies par le catalogue fr-CH. */
function divergencesActives(): string[] {
	const auCatalogue = clesDuCatalogueFr();
	return [...replisParCle()]
		.filter(([cle, parTexte]) => parTexte.size > 1 && auCatalogue.has(cle))
		.map(([cle]) => cle);
}

describe('le catalogue fr-CH ne ment sur aucun site', () => {
	it('aucune clé déjà traduite ne porte deux replis divergents, hors tolérées', () => {
		const tolerees = new Set(TOLEREES.map((t) => t.cle));
		const actives = divergencesActives().filter((c) => !tolerees.has(c));
		expect(actives).toEqual([]);
	});

	// ⚠️ Ce test EST la borne anti-test-muet — cf. l'en-tête du fichier. Il rougit dans les
	// deux sens : une entrée dont la divergence a été corrigée doit sortir de la liste
	// (sinon l'allowlist devient un cimetière qui ne garde plus rien), et un relevé qui se
	// viderait pour cause de lecteur cassé les ferait toutes sortir d'un coup.
	it("aucune entrée tolérée n'est périmée — chacune est encore divergente", () => {
		const actives = new Set(divergencesActives());
		const perimees = TOLEREES.map((t) => t.cle).filter((c) => !actives.has(c));
		expect(perimees).toEqual([]);
	});

	it('chaque tolérance porte un motif écrit', () => {
		expect(TOLEREES.filter((t) => t.motif.trim().length < 20).map((t) => t.cle)).toEqual([]);
	});
});
