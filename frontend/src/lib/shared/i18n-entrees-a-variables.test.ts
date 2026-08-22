/**
 * Garde « les entrées Fluent à variables sont cohérentes de bout en bout ».
 *
 * ⚠️ **Une erreur de nom de variable est SILENCIEUSE à la compilation, et invisible à
 * l'écran tant que le cas d'erreur ne survient pas.** `i18nMsg` remplace `{ $x }` par
 * `args[x] ?? ''` : un nom qui ne correspond pas ne lève rien — il rend la **chaîne vide**.
 * L'utilisateur lit alors « Ligne  : compte requis », et aucun test de rendu ordinaire ne
 * passe par là, ces messages n'apparaissant qu'en cas de saisie invalide.
 *
 * C'est le risque **R4** du plan de l'Epic 23, écrit avant que la 23-5 ne les livre : les
 * cinq entrées de `TransactionSplitModal` sont « les seules non mécaniques » du rollout.
 *
 * ⚠️ **Trois maillons, trois tests — parce qu'un seul ne suffit pas.** La chaîne va du site
 * d'appel au catalogue en passant par le repli, et chaque jonction peut casser seule :
 *
 *   1. **repli ↔ catalogue** — le repli dit `{ $min }`, une locale dit `{ $minimum }` : la
 *      clé manquante retomberait sur le repli et marcherait, la clé présente échouerait ;
 *   2. **entre locales** — `fr-CH` déclare `{ $line }`, `de-CH` l'oublie : le message
 *      allemand perd son numéro de ligne, en silence ;
 *   3. **site ↔ repli** — le site passe `{ ligne: i + 1 }` pour un repli qui dit
 *      `{ $line }` : **c'est le cas que R4 vise**, et ni 1 ni 2 ne le voient, le repli et
 *      les quatre locales pouvant être parfaitement d'accord entre eux.
 *
 * ⚠️ **Le repli porte lui aussi les `{ $x }`, et ce n'est pas un détail de style.**
 * `i18nMsg` interpole `raw`, qui vaut le catalogue **ou** le repli. Un repli écrit en
 * gabarit JS (`` `Au moins ${MIN} lignes` ``) serait déjà rendu et ne passerait pas par
 * l'interpolation : les deux chemins divergeraient, et le repli — celui qu'on voit le moins,
 * puisqu'il ne sert que quand la clé manque — serait le seul correct.
 */
import { describe, it, expect } from 'vitest';
import { readFileSync, readdirSync } from 'node:fs';
import { join } from 'node:path';
import { findCallSites, readFallback, masquerCommentaires } from './i18n-literal-reader.js';
import { dansLePerimetreDeFichier } from './i18n-harvest.js';

const RACINE = 'src';
const RACINE_FTL = '../crates/kesh-i18n/locales';
const LOCALES = ['fr-CH', 'de-CH', 'it-CH', 'en-CH'] as const;

/** Les variables Fluent d'un texte : `{ $nom }`, marques d'isolation tolérées. */
function variables(texte: string): Set<string> {
	const out = new Set<string>();
	for (const m of texte.matchAll(/⁨?\{\s*\$(\w+)\s*\}⁩?/g)) out.add(m[1]);
	return out;
}

function catalogue(locale: string): Map<string, string> {
	const texte = readFileSync(join(RACINE_FTL, locale, 'messages.ftl'), 'utf-8');
	const m = new Map<string, string>();
	for (const ligne of texte.split('\n')) {
		const r = /^([a-zA-Z][\w-]*)\s*=\s*(.*)$/.exec(ligne);
		if (r) m.set(r[1], r[2]);
	}
	return m;
}

/**
 * Lit les noms de propriétés du **troisième** argument d'un appel — l'objet d'arguments.
 *
 * ⚠️ Ne descend pas dans les objets imbriqués, et c'est voulu : Fluent ne consomme que des
 * variables plates. Une propriété imbriquée serait de toute façon inutilisable comme `$x`.
 */
function argsDuSite(source: string, apresRepli: number): Set<string> | null {
	let j = apresRepli;
	while (j < source.length && /\s/.test(source[j])) j++;
	if (source[j] !== ',') return null;
	j++;
	while (j < source.length && /\s/.test(source[j])) j++;
	if (source[j] !== '{') return null;
	let profondeur = 0;
	const debut = j;
	for (; j < source.length; j++) {
		if (source[j] === '{') profondeur++;
		else if (source[j] === '}' && --profondeur === 0) break;
	}
	const corps = source.slice(debut + 1, j);
	// Découpage au PREMIER niveau : une virgule imbriquée (objet, appel, littéral de tableau)
	// ne sépare pas deux propriétés.
	const fragments: string[] = [];
	let prof = 0;
	let debutFragment = 0;
	for (let k = 0; k < corps.length; k++) {
		const c = corps[k];
		if (c === '{' || c === '(' || c === '[') prof++;
		else if (c === '}' || c === ')' || c === ']') prof--;
		else if (c === ',' && prof === 0) {
			fragments.push(corps.slice(debutFragment, k));
			debutFragment = k + 1;
		}
	}
	fragments.push(corps.slice(debutFragment));
	const out = new Set<string>();
	for (const f of fragments) {
		const t = f.trim();
		if (!t) continue;
		// ⚠️ **La forme ABRÉGÉE `{ code }` est la plus courante du dépôt**, et une première
		// rédaction de ce parseur ne cherchait que `nom:` — d'où trois faux positifs sur des
		// sites parfaitement corrects (`error-label.ts`, `reminder-error-label.ts`). Une garde
		// qui crie au loup se fait désarmer, et c'est pire que pas de garde.
		const avecDeuxPoints = /^([A-Za-z_$][\w$]*)\s*:/.exec(t);
		if (avecDeuxPoints) {
			out.add(avecDeuxPoints[1]);
			continue;
		}
		const abrege = /^([A-Za-z_$][\w$]*)$/.exec(t);
		if (abrege) out.add(abrege[1]);
	}
	return out;
}

type Site = { cle: string; chemin: string; ligne: number; repli: string; args: Set<string> | null };

function sitesAVariables(): Site[] {
	const out: Site[] = [];
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
				if (variables(repli.value).size === 0) continue;
				out.push({
					cle: site.arg.value,
					chemin,
					ligne: site.line,
					repli: repli.value,
					args: argsDuSite(source, repli.end),
				});
			}
		}
	};
	parcourir(RACINE);
	return out;
}

describe('entrées Fluent à variables', () => {
	// Maillon 3 — le seul que R4 vise nommément, et le seul qu'aucun autre test ne voit.
	it("chaque site fournit les arguments que son repli réclame", () => {
		const manquants = sitesAVariables()
			.map((s) => {
				const attendues = [...variables(s.repli)];
				const fournis = s.args ?? new Set<string>();
				const absents = attendues.filter((v) => !fournis.has(v));
				return absents.length
					? `${s.chemin}:${s.ligne} ${s.cle} → manque { ${absents.join(', ')} }`
					: null;
			})
			.filter(Boolean);
		expect(manquants).toEqual([]);
	});

	// Maillon 1 — le repli et le catalogue doivent réclamer EXACTEMENT les mêmes variables.
	it('le repli et le catalogue fr-CH réclament les mêmes variables', () => {
		const fr = catalogue('fr-CH');
		const ecarts = sitesAVariables()
			.filter((s) => fr.has(s.cle))
			.map((s) => {
				const a = [...variables(s.repli)].sort();
				const b = [...variables(fr.get(s.cle)!)].sort();
				return a.join(',') === b.join(',')
					? null
					: `${s.cle} → repli { ${a.join(', ')} } contre catalogue { ${b.join(', ')} }`;
			})
			.filter(Boolean);
		expect(ecarts).toEqual([]);
	});

	// Maillon 2 — une locale qui perd une variable perd l'information, sans rien signaler.
	it('les quatre locales déclarent le même jeu de variables pour chaque clé', () => {
		const cats = LOCALES.map((l) => catalogue(l));
		const ecarts: string[] = [];
		for (const [cle, valeur] of cats[0]) {
			const ref = [...variables(valeur)].sort().join(',');
			if (!ref) continue;
			for (let i = 1; i < LOCALES.length; i++) {
				const autre = [...variables(cats[i].get(cle) ?? '')].sort().join(',');
				if (autre !== ref) ecarts.push(`${cle} : fr-CH { ${ref} } contre ${LOCALES[i]} { ${autre} }`);
			}
		}
		expect(ecarts).toEqual([]);
	});

	// Borne anti-test-muet : les trois preuves ci-dessus sont vertes à vide si le relevé
	// ne rend rien. La 23-5 livre les cinq entrées de `TransactionSplitModal` ; elles sont
	// nommées, parce qu'un simple `> 0` laisserait passer la disparition de quatre d'entre elles.
	it('les cinq entrées à variables de la story 23-5 sont bien relevées', () => {
		const vues = new Set(sitesAVariables().map((s) => s.cle));
		const attendues = [
			'reconciliation-split-error-min-lines',
			'reconciliation-split-error-max-lines',
			'reconciliation-split-error-account-required',
			'reconciliation-split-error-amount-positive',
			'reconciliation-split-error-description-too-long',
		];
		expect(attendues.filter((c) => !vues.has(c))).toEqual([]);
	});

	// Maillon 1+2+3 réunis : le rendu réel, celui que l'utilisateur lit.
	it('aucun placeholder ne survit au rendu, dans les quatre locales', () => {
		const rendre = (raw: string, args: Record<string, string | number>) =>
			raw.replace(/⁨?\{\s*\$(\w+)\s*\}⁩?/g, (_, k) => String(args[k] ?? ''));
		const cas: [string, Record<string, number>][] = [
			['reconciliation-split-error-min-lines', { min: 2 }],
			['reconciliation-split-error-max-lines', { max: 20 }],
			['reconciliation-split-error-account-required', { line: 3 }],
			['reconciliation-split-error-amount-positive', { line: 3 }],
			['reconciliation-split-error-description-too-long', { line: 3, max: 200 }],
		];
		const survivants: string[] = [];
		for (const locale of LOCALES) {
			const cat = catalogue(locale);
			for (const [cle, args] of cas) {
				const brut = cat.get(cle);
				expect(brut, `${cle} absente de ${locale}`).toBeDefined();
				const rendu = rendre(brut!, args);
				if (/\{\s*\$\w+\s*\}/.test(rendu)) survivants.push(`${locale}/${cle} → « ${rendu} »`);
				// ⚠️ Un rendu qui contient «   » (double espace) trahit un argument résolu en
				// chaîne vide — le mode d'échec exact que ce fichier existe pour attraper.
				if (/\s\s/.test(rendu)) survivants.push(`${locale}/${cle} → trou : « ${rendu} »`);
			}
		}
		expect(survivants).toEqual([]);
	});
});
