/**
 * Garde contre les **libellés en dur** — l'angle mort #255, story 23-3b (#316).
 *
 * ⚠️ **Une fonction de libellé qui n'appelle jamais `i18nMsg` est invisible de TOUT
 * l'appareil de contrôle de l'epic 23.** Le moissonneur (`i18n-harvest.js`), l'allowlist
 * (`i18n-dette-connue.ts`) et les trois gardes existantes — `loader.rs::parity_between_locales`,
 * `i18n-keys.test.ts`, `i18n-un-repli-par-cle.test.ts` — ne relèvent que les **sites
 * `i18nMsg`**. Ce qui n'appelle pas n'existe pour aucune d'elles.
 *
 * Le défaut a été trouvé **à l'écran**, en passe 4 de revue de la 23-3 :
 * `supplierInvoiceStatusLabel()` retournait `'Ouverte'` / `'Payée'` / `'Annulée'` en dur
 * **sous un en-tête de colonne que la story venait de traduire** — un germanophone lisait
 * « Status » au-dessus de « Payée ». Et ce n'était pas un accident : le même doc-comment
 * *« Libellé FR du statut d'un X (fallback i18n côté composant) »* était copié trois fois.
 *
 * ⚠️ **Elle lit les SOURCES, jamais les catalogues.** Une garde qui s'appuierait sur les
 * catalogues s'éteindrait au moment où la traduction est livrée — c'est-à-dire quand le
 * risque devient réel. C'est le mode d'échec du test muet, déjà payé plusieurs fois ici.
 *
 * ## Ce qu'elle voit, et ce qu'elle NE PEUT PAS voir
 *
 * Elle vise **un patron précis** : *une fonction dont la valeur est affichée retourne un
 * littéral sans passer par `i18nMsg`*. Elle le détecte **par le nom** de la déclaration
 * (`*Label`, `*Text`, `*Display`), puis par l'inspection de son corps.
 *
 * | forme du défaut | vue ? | pourquoi |
 * |---|---|---|
 * | fonction `*Label` → `return '<littéral>'` | ✅ **oui** — c'est son objet | |
 * | valeur affichée sans aucun appel (`{data.company.orgType}`) | ❌ non | ni fonction ni littéral : le défaut est l'*absence* d'appel |
 * | littéraux dans un **tableau de données** (`navGroups`) | ❌ non | pas un corps de fonction, et **indiscernable** d'une table de replis légitime sans analyse de flot |
 * | **nœud de texte** de balisage (`<Dialog.Title>Valider la facture</Dialog.Title>`) | ❌ non | ni fonction, ni littéral JS |
 *
 * ⚠️ **Ces trois formes ont été corrigées À LA MAIN par la story 23-3b** (sites 4, 6 et 8
 * de son tableau). **Ne pas élargir la garde pour les attraper** : l'élargissement « par le
 * contenu » — détecter tout littéral qui *ressemble à du français* — a été **chiffré** sur
 * le dépôt : 1035 littéraux bruts, dont 860 replis légitimes en 2ᵉ argument d'`i18nMsg`,
 * soit **175 à trier** dont ~160 sans rapport ; et cet angle **rate quand même** `Brouillon`
 * et 7 des 9 libellés de navigation, qui n'ont ni accent ni mot-outil. Il coûte 160
 * exemptions pour ne rien gagner, c'est-à-dire une garde qu'on désactive au premier gate
 * bruyant.
 *
 * ## Les trois classes de littéraux légitimes sont des CRITÈRES, jamais des entrées
 *
 * Une **forme** de littéral ne s'énumère pas — sinon la liste enfle à chaque cas et perd son
 * sens. Sont donc écartés par critère : le **cadratin et assimilés** (`'—'`, `'N/A'`), la
 * **chaîne vide** `''`, et les **gabarits interpolés** (`` `${a} — ${b}` `` n'est pas un
 * libellé en dur ; le lecteur les distingue déjà par `kind: 'template'`).
 *
 * ## Pourquoi l'allowlist est VIDE, et ce que ça ne veut pas dire
 *
 * La spec annonçait quatre « faux amis » connus d'avance — `LABEL_FALLBACK`,
 * `FILTER_FALLBACK_FR`, `TYPE_FALLBACK`, `ROLE_FALLBACK`, des `Record` de littéraux passés
 * en 2ᵉ argument d'`i18nMsg`. ⚠️ **Cette garde ne les atteint pas** : ce sont des tables, pas
 * des corps de fonction, et les fonctions qui les consomment (`typeLabel`, `roleLabel`)
 * appellent `i18nMsg` — donc conformes. Les inscrire ici produirait des **entrées mortes qui
 * paraissent fonctionner**, c'est-à-dire l'affirmation d'un contrôle qui n'a pas lieu.
 *
 * ⚠️ Le critère de discrimination reste écrit pour le jour où quelqu'un voudra les exempter :
 * *un `Record` de littéraux est légitime si et seulement si chacun de ses accès apparaît en
 * 2ᵉ argument d'`i18nMsg` ou d'un relais.* Sans lui, on exempte au nom — et `navGroups`, qui
 * a exactement la même forme et porte un vrai défaut, deviendrait légitime par accident.
 */
import { describe, it, expect } from 'vitest';
import { readFileSync, readdirSync } from 'node:fs';
import { join } from 'node:path';
import { corpsDeFonction, masquerCommentaires, readLiteral } from './i18n-literal-reader.js';
import { dansLePerimetreDeFichier } from './i18n-harvest.js';

const RACINE = 'src';

/** Les suffixes qui désignent une fonction dont la valeur est **affichée**. */
const SUFFIXES = ['Label', 'Text', 'Display'];

/**
 * Nombre de **déclarations candidates** relevées sur `src/` — conformes, exemptées et en
 * violation confondues.
 *
 * ⚠️ **La borne porte sur une grandeur STABLE, jamais sur le nombre de violations** : un
 * compteur de violations tombe à zéro dès le premier correctif et n'atteste plus rien —
 * c'est un test muet en puissance. Le nombre de fonctions dont le nom promet un libellé,
 * lui, ne descend pas tout seul.
 *
 * ⚠️ **Un écart se recompte, il ne s'ajuste pas.** Si ce nombre rougit : ouvrir le relevé,
 * regarder QUELLES déclarations sont apparues ou ont disparu, et ne changer le chiffre
 * qu'après. Une baisse peut être un libellé qui a cessé d'être une fonction.
 */
const CANDIDATES_ATTENDUES = 41;

/** Les trois délimiteurs de littéral en JS/TS. */
const QUOTES = ["'", '"', '`'];

/**
 * Littéraux qui ne sont **pas** des libellés, écartés par CRITÈRE et non par exemption :
 * le cadratin et ses assimilés, et la chaîne vide.
 */
const NEUTRES = new Set(['', '—', '–', '-', '…', 'N/A']);

/**
 * Exemptions nominatives — **vide, et cf. le préambule**. Toute entrée porte un motif écrit ;
 * ⚠️ *« ce site n'était pas prévu »* n'en est pas un : un site relevé et non prévu se corrige
 * ou s'ouvre en issue, il ne s'inscrit pas ici.
 */
const EXEMPTEES: { nom: string; chemin: string; motif: string }[] = [];

type Retour = { texte: string; ligne: number };
/**
 * `analysee: false` ⇒ **la déclaration a un corps que la garde n'a PAS su lire.**
 *
 * ⚠️ **C'est le correctif de fond de la passe 1 de revue, et il vaut plus que les formes
 * syntaxiques qu'il a fallu ajouter par ailleurs.** Auparavant, toute forme non reconnue
 * ressortait avec `retours: []` — c'est-à-dire **comptée parmi les conformes**. Une fonction
 * jamais inspectée était donc déclarée saine, ce qui est exactement le mode d'échec que cette
 * garde existe pour fermer. Désormais elle rougit : mieux vaut un faux positif bruyant qu'un
 * faux négatif muet.
 *
 * ⚠️ **Une déclaration SANS corps de fonction reste `analysee: true`** — `let x = $state('')`,
 * `$derived(expr)` : il n'y a pas de corps à lire, et l'analyse « littéral en tête » a bien eu
 * lieu. Les classer « non analysées » ferait rougir la garde sur du code conforme.
 */
type Candidate = {
	nom: string;
	chemin: string;
	ligne: number;
	retours: Retour[];
	analysee: boolean;
};

/** Saute espaces et sauts de ligne. Les commentaires sont déjà masqués en amont. */
function sauterBlancs(source: string, i: number): number {
	let j = i;
	while (j < source.length && ' \t\r\n'.includes(source[j])) j += 1;
	return j;
}

/**
 * Apparie la parenthèse ouvrante en `ouvrante` et rend la position de sa fermante, en
 * sautant les littéraux — sans quoi une parenthèse dans une chaîne fausserait le compte.
 */
function fermeParenthese(source: string, ouvrante: number): number | null {
	let profondeur = 0;
	let j = ouvrante;
	while (j < source.length) {
		const c = source[j];
		if (QUOTES.includes(c)) {
			const lu = readLiteral(source, j);
			if (lu) {
				j = lu.end;
				continue;
			}
		}
		if (c === '(') profondeur += 1;
		else if (c === ')') {
			profondeur -= 1;
			if (profondeur === 0) return j;
		}
		j += 1;
	}
	return null;
}

/**
 * Rend la position de l'accolade **du corps** d'une `function`, à partir de la position
 * qui suit son nom.
 *
 * ⚠️ **Le piège, et il est la raison d'être de cette fonction** : chercher « le premier `{`
 * après le nom » casse sur un type inline — `function fooLabel(opts: { x: number }): string {`
 * rendrait le `{` du type. Il faut le `{` qui suit la **fermeture de la signature**, d'où
 * l'appariement de parenthèses. Aucune des déclarations du dépôt n'a ce cas aujourd'hui :
 * il est verrouillé par un test synthétique plutôt que laissé à la chance.
 *
 * ⚠️ Second piège, symétrique : un **type de retour objet** — `function fooLabel(): { a: string } {`
 * — place encore un `{` avant le corps. Il est reconnu à ce qu'un autre `{` le suit.
 */
function accoladeDeCorps(source: string, apresNom: number): number | null {
	const ouvrante = sauterBlancs(source, apresNom);
	if (source[ouvrante] !== '(') return null;
	const fermante = fermeParenthese(source, ouvrante);
	if (fermante === null) return null;
	// Avancer jusqu'à la première accolade, en traversant l'annotation de type de retour
	// (`: string`, `: { a: string }`, `: Promise<{…}>`) et en sautant les littéraux.
	// ⚠️ **Sans cette traversée, la garde est MUETTE** : presque toutes les déclarations du
	// dépôt s'écrivent `function xLabel(…): string {`, le caractère qui suit la signature
	// est donc `:` et non `{`. Une première rédaction rendait `null` sur toutes — le relevé
	// était vide, les violations aussi, et le gate VERT. Attrapé par les cas synthétiques
	// ci-dessous, jamais par le relevé du dépôt : c'est précisément ce qu'ils sont là pour
	// faire, un test muet ne se signale pas lui-même.
	let j = fermante + 1;
	while (j < source.length) {
		const c = source[j];
		if (QUOTES.includes(c)) {
			const lu = readLiteral(source, j);
			if (lu) {
				j = lu.end;
				continue;
			}
		}
		if (c === '{') break;
		// Ni corps ni type : signature de surcharge, ou déclaration sans implémentation.
		if (c === ';' || c === ')' || c === '}') return null;
		j += 1;
	}
	while (j < source.length && source[j] === '{') {
		const corps = corpsDeFonction(source, j);
		if (corps === null) return null;
		const apres = sauterBlancs(source, j + corps.length + 2);
		// Un `{` suivi d'un autre `{` était l'annotation de type de retour, pas le corps.
		if (source[apres] !== '{') return j;
		j = apres;
	}
	return null;
}

/**
 * Relève les littéraux retournés par un corps de fonction — ceux qui sont en **position de
 * valeur**, et eux seuls.
 *
 * ⚠️ **La première rédaction n'examinait que le token collé à `return`**, si bien que
 * `return cond ? 'Payée' : 'Ouverte'` — *le patron même du domaine* — passait inaperçu.
 * (Passe 1 de revue.)
 *
 * ⚠️ **Et l'élargissement naïf se retourne contre son objet** : relever TOUT littéral de
 * l'expression ferait de `return i18nMsg('cle', 'repli')` une violation dans **toutes** les
 * fonctions conformes, et de `s === 'paid'` un libellé. D'où deux règles :
 *
 *   1. on ne relève qu'en **position de valeur** — après `return`, `?`, `:`, `||`, `&&`, `??`,
 *      ou une parenthèse de **groupement** ;
 *   2. **rien n'est relevé à l'intérieur d'un appel** : les arguments d'une fonction ne sont
 *      pas des retours. C'est ce qui distingue `i18nMsg('cle', 'repli')` d'un libellé nu.
 */
function retoursLitteraux(corps: string, ligneDeBase: number): Retour[] {
	const out: Retour[] = [];
	const motif = /\breturn\b/g;
	let m: RegExpExecArray | null;
	while ((m = motif.exec(corps)) !== null) {
		let j = m.index + m[0].length;
		let attendValeur = true;
		let profondeurAppel = 0;
		let profondeurGroupe = 0;
		let profondeurStructure = 0;
		let precedent = '';
		while (j < corps.length) {
			const c = corps[j];
			if (' \t\r\n'.includes(c)) {
				j += 1;
				continue;
			}
			if (QUOTES.includes(c)) {
				const lu = readLiteral(corps, j);
				if (!lu) break;
				if (attendValeur && profondeurAppel === 0 && profondeurStructure === 0 && lu.kind === 'literal') {
					out.push({
						texte: lu.value,
						ligne: ligneDeBase + corps.slice(0, j).split('\n').length - 1
					});
				}
				// ⚠️ Reprendre APRÈS le littéral, gabarit compris : un `return` niché dans une
				// interpolation serait sinon attribué à la fonction englobante.
				j = lu.end;
				motif.lastIndex = Math.max(motif.lastIndex, j);
				attendValeur = false;
				precedent = 'x';
				continue;
			}
			if (c === ';' && profondeurAppel === 0 && profondeurGroupe === 0 && profondeurStructure === 0)
				break;
			// ⚠️ **Une STRUCTURE retournée n'est pas un libellé** — `return { open: 'Ouverte' }[s]`
			// est une table de dispatch, forme légitime que la doctrine de cette garde écarte déjà
			// (« indiscernable d'une table de replis légitime sans analyse de flot »). Sans ce
			// suivi, la rédaction de la passe 1 criait sur du code sain — et un gate bruyant se
			// fait désactiver. ⚠️ **Le tableau était pire que l'objet** : il ne relevait QUE le
			// second élément, parce que la virgule rouvrait une position de valeur que le crochet
			// n'avait jamais fermée. (Passe 2 de revue.)
			if (c === '{' || c === '[') {
				profondeurStructure += 1;
				attendValeur = false;
				precedent = c;
				j += 1;
				continue;
			}
			if (c === '}' || c === ']') {
				if (profondeurStructure > 0) profondeurStructure -= 1;
				else break;
				attendValeur = false;
				// ⚠️ `precedent` devient une FERMETURE : le `[s]` qui suit `{…}` est alors lu comme
				// un accès, pas comme une nouvelle structure — et le `?? 'Aucun'` qui suivrait
				// reste, lui, une position de valeur.
				precedent = ')';
				j += 1;
				continue;
			}
			if (c === '(') {
				// Un `(` collé à un identifiant ou à une fermeture ouvre un APPEL ; sinon c'est un
				// groupement, et `return ('Dur')` reste un retour de littéral.
				if (/[\w$)\]]/.test(precedent)) profondeurAppel += 1;
				else {
					profondeurGroupe += 1;
					attendValeur = true;
				}
				precedent = '(';
				j += 1;
				continue;
			}
			if (c === ')') {
				if (profondeurAppel > 0) profondeurAppel -= 1;
				else if (profondeurGroupe > 0) profondeurGroupe -= 1;
				else break;
				attendValeur = false;
				precedent = ')';
				j += 1;
				continue;
			}
			if (c === '?' || c === ':' || c === '|' || c === '&' || c === ',') {
				// La virgule n'ouvre une position de valeur qu'HORS appel — sinon le 2ᵉ argument
				// d'`i18nMsg` deviendrait une violation.
				attendValeur = c !== ',' || profondeurAppel === 0;
				precedent = c;
				j += 1;
				continue;
			}
			attendValeur = false;
			precedent = c;
			j += 1;
		}
	}
	return out;
}

/**
 * Relève toutes les déclarations candidates d'une source et, pour chacune, les littéraux
 * qu'elle retourne.
 *
 * Formes reconnues — `function nom(…) {…}` (génératrice comprise), `const nom = (…) => {…}`,
 * `const nom = (…) => <expression>`, `const nom = function (…) {…}`, et `const nom = <expression>`
 * (dont `$derived`, `$derived.by(() => {…})`). L'annotation de type est tolérée entre le nom et
 * le `=`. ⚠️ **La dernière forme compte** : `const statusLabel = 'Ouvert'` doit rougir comme le
 * ferait un `return`.
 *
 * ⚠️ **Ce qui n'est PAS reconnu doit être BRUYANT, jamais silencieux** — cf. `analysee`. Restent
 * hors de portée, et c'est écrit plutôt que découvert : les **méthodes de classe**
 * (`class Foo { statusLabel() {…} }`) — aucune classe TypeScript dans `src/` à ce jour —, les
 * propriétés d'objet (`{ label: 'x' }`), indiscernables d'une table de replis légitime, et la
 * **seconde déclaration d'une liste** (`const a = 1, bLabel = '…'`). ⚠️ Cette dernière n'est pas
 * un oubli : élargir le motif à la virgule ferait entrer les paramètres à valeur par défaut
 * comme candidates — un faux négatif échangé contre un faux positif, et c'est le mauvais sens.
 */
export function candidatesDe(source: string, chemin = '<mémoire>'): Candidate[] {
	const out: Candidate[] = [];
	// ⚠️ `function\s*\*?\s*` capte les génératrices ; `(?::[^;\n]*?)?=(?![=>])` tolère une
	// annotation de type sans se laisser arrêter par la flèche d'un type fonction
	// (`const x: () => string = …`). Sans elle, `const statusLabel: string = 'Ouvert'` n'était
	// même pas COMPTÉE — ni conforme, ni en violation, simplement hors radar.
	const decl = /(?:function\s*\*?\s*(\w+)|(?:const|let|var)\s+(\w+)\s*(?::[^;\n]*?)?=(?![=>]))/g;
	let m: RegExpExecArray | null;
	while ((m = decl.exec(source)) !== null) {
		const nom = m[1] ?? m[2];
		if (!SUFFIXES.some((s) => nom.endsWith(s))) continue;
		const ligne = source.slice(0, m.index).split('\n').length;
		const apres = m.index + m[0].length;
		const candidate: Candidate = { nom, chemin, ligne, retours: [], analysee: true };
		out.push(candidate);

		/** Lit le corps ouvert en `ouvrante` ; marque la candidate NON ANALYSÉE si c'est illisible. */
		const lireCorps = (ouvrante: number) => {
			const corps = corpsDeFonction(source, ouvrante);
			if (corps === null) {
				candidate.analysee = false;
				return;
			}
			candidate.retours = retoursLitteraux(corps, source.slice(0, ouvrante).split('\n').length);
		};

		// (a) `function nom(…) {…}` — l'accolade du corps, jamais celle d'un type.
		if (m[1] !== undefined) {
			const ouvrante = accoladeDeCorps(source, apres);
			if (ouvrante === null) candidate.analysee = false;
			else lireCorps(ouvrante);
			continue;
		}

		// (b) `const nom = <littéral>` — le libellé posé directement sur la déclaration.
		const debut = sauterBlancs(source, apres);
		if (QUOTES.includes(source[debut])) {
			const lu = readLiteral(source, debut);
			if (lu && lu.kind === 'literal') candidate.retours.push({ texte: lu.value, ligne });
			continue;
		}

		const fin = finInstruction(source, debut);
		const region = source.slice(debut, fin);

		// (c) `const nom = function (…) {…}` — une expression de fonction, anonyme ou nommée.
		//     Elle ressortait « conforme » sans que son corps ait jamais été lu.
		const motFunction = /\bfunction\b/.exec(region);
		const fleche = flecheDeCorps(region);
		if (motFunction !== null && (fleche === -1 || motFunction.index < fleche)) {
			const nomOuParen = sauterBlancs(source, debut + motFunction.index + 'function'.length);
			// `function (…)` anonyme ou `function inner(…)` : sauter le nom s'il y en a un.
			let k = nomOuParen;
			while (k < source.length && /[\w$]/.test(source[k])) k += 1;
			const ouvrante = accoladeDeCorps(source, k);
			if (ouvrante === null) candidate.analysee = false;
			else lireCorps(ouvrante);
			continue;
		}

		// (d) une flèche de corps — `(…) => {…}` ou `() => <expr>`, ce qui couvre `$derived.by`.
		if (fleche === -1) continue; // pas de corps de fonction : rien à lire, cf. `analysee`
		const apresFleche = sauterBlancs(source, debut + fleche + 2);
		if (source[apresFleche] === '{') {
			lireCorps(apresFleche);
		} else if (QUOTES.includes(source[apresFleche])) {
			// Corps d'expression : `const xLabel = (s) => 'Ouvert'`.
			const lu = readLiteral(source, apresFleche);
			if (lu && lu.kind === 'literal') {
				candidate.retours.push({
					texte: lu.value,
					ligne: source.slice(0, apresFleche).split('\n').length
				});
			}
		}
	}
	return out;
}

/**
 * Position de la flèche qui ouvre un CORPS de fonction dans `region`, ou `-1`.
 *
 * ⚠️ **La première occurrence textuelle de `=>` ne convient pas** : dans
 * `const xLabel = (cb: () => void) => 'Ouvert'`, c'est la flèche du TYPE de `cb` qu'on trouve
 * d'abord.
 *
 * ⚠️ **Et exiger la profondeur ZÉRO ne convient pas non plus** — première rédaction du correctif,
 * qui a cassé `$derived.by(() => {…})`, dont la flèche vit à l'intérieur des parenthèses de
 * l'appel. Le critère juste est la profondeur **MINIMALE** rencontrée : elle vaut 0 pour une
 * fléchée nue, 1 pour un `$derived.by`, et écarte dans les deux cas la flèche d'un type de
 * paramètre, qui est toujours plus profonde que le corps qu'elle précède.
 */
function flecheDeCorps(region: string): number {
	let profondeur = 0;
	let meilleure = -1;
	let profondeurMeilleure = Infinity;
	let j = 0;
	while (j < region.length) {
		const c = region[j];
		if (QUOTES.includes(c)) {
			const lu = readLiteral(region, j);
			if (lu) {
				j = lu.end;
				continue;
			}
		}
		if (c === '(' || c === '[' || c === '{') profondeur += 1;
		else if (c === ')' || c === ']' || c === '}') profondeur -= 1;
		else if (c === '=' && region[j + 1] === '>' && profondeur < profondeurMeilleure) {
			meilleure = j;
			profondeurMeilleure = profondeur;
		}
		j += 1;
	}
	return meilleure;
}

/** Les candidates dont le corps existe mais n'a PAS pu être lu — cf. `Candidate.analysee`. */
function nonAnalyseesDe(candidates: Candidate[]): string[] {
	return candidates.filter((c) => !c.analysee).map((c) => `${c.chemin}:${c.ligne} ${c.nom}()`);
}

/**
 * Rend la position de fin de l'instruction commencée en `i` — le `;` de niveau 0, ou la
 * sortie du bloc englobant. ⚠️ **Par appariement, jamais par un nombre de caractères** :
 * une borne en caractères rendrait invisible toute déclaration au corps plus long, et
 * l'invisibilité ne fait rougir personne.
 */
function finInstruction(source: string, i: number): number {
	let profondeur = 0;
	let j = i;
	while (j < source.length) {
		const c = source[j];
		if (QUOTES.includes(c)) {
			const lu = readLiteral(source, j);
			if (lu) {
				j = lu.end;
				continue;
			}
		}
		if ('([{'.includes(c)) profondeur += 1;
		else if (')]}'.includes(c)) {
			profondeur -= 1;
			if (profondeur < 0) return j;
		} else if (c === ';' && profondeur === 0) return j;
		j += 1;
	}
	return source.length;
}

/** Parcourt `src/` et rend toutes les déclarations candidates du dépôt. */
function releverLeDepot(): Candidate[] {
	const out: Candidate[] = [];
	const parcourir = (rep: string) => {
		for (const e of readdirSync(rep, { withFileTypes: true })) {
			const chemin = join(rep, e.name);
			if (e.isDirectory()) {
				parcourir(chemin);
				continue;
			}
			if (!dansLePerimetreDeFichier(e.name)) continue;
			const source = masquerCommentaires(readFileSync(chemin, 'utf-8'));
			out.push(...candidatesDe(source, chemin));
		}
	};
	parcourir(RACINE);
	return out;
}

/** Les retours en dur qui ne sont écartés ni par critère ni par exemption. */
function violationsDe(candidates: Candidate[]): string[] {
	return candidates
		.filter((c) => !EXEMPTEES.some((e) => e.nom === c.nom && e.chemin === c.chemin))
		.flatMap((c) =>
			c.retours
				.filter((r) => !NEUTRES.has(r.texte.trim()))
				.map((r) => `${c.chemin}:${r.ligne} ${c.nom}() → « ${r.texte} »`)
		);
}

describe('libellés en dur — l’angle mort #255', () => {
	it('aucune fonction de libellé ne retourne un littéral sans passer par i18nMsg', () => {
		expect(violationsDe(releverLeDepot())).toEqual([]);
	});

	// Borne anti-test-muet : si le relevé rendait un ensemble vide — lecteur cassé, arbre
	// déplacé, suffixes renommés —, la preuve ci-dessus serait verte à vide.
	it('le relevé porte exactement le nombre attendu de déclarations candidates', () => {
		expect(releverLeDepot().length).toBe(CANDIDATES_ATTENDUES);
	});

	it('toute exemption porte un motif écrit', () => {
		expect(EXEMPTEES.filter((e) => e.motif.trim().length === 0)).toEqual([]);
	});

	// ⚠️ **La rédaction précédente de ce test était une TAUTOLOGIE, et elle a été relevée en
	// passe 1 de revue** : `conformes` et `ecartees` y étaient définis par SOUSTRACTION à partir
	// du total, si bien que `conformes + ecartees + violations === total` se réduisait à `N = N`.
	// Elle ne pouvait pas rougir — un test muet, écrit dans le test qui dénonce les tests muets.
	// Chaque candidate est désormais classée **une seule fois**, et c'est la partition qui est
	// vérifiée : une régression de la logique de classification la casse pour de bon.
	it('la ventilation partitionne le relevé — chaque candidate dans une classe et une seule', () => {
		const candidates = releverLeDepot();
		const classes = { nonAnalysee: 0, enViolation: 0, ecartee: 0, conforme: 0 };
		for (const c of candidates) {
			if (!c.analysee) classes.nonAnalysee += 1;
			else if (c.retours.some((r) => !NEUTRES.has(r.texte.trim()))) classes.enViolation += 1;
			else if (c.retours.length > 0) classes.ecartee += 1;
			else classes.conforme += 1;
		}
		expect(classes).toEqual({ nonAnalysee: 0, enViolation: 0, ecartee: 4, conforme: 37 });
		// La somme est recalculée depuis les classes, jamais depuis le total qu'elle contrôle.
		const somme = Object.values(classes).reduce((a, b) => a + b, 0);
		expect(somme).toBe(CANDIDATES_ATTENDUES);
	});

	// ⚠️ **Le garde-fou le plus important de cette garde, et il est né d'une revue.** Une
	// déclaration dont le corps existe et n'a pas pu être lu ressortait auparavant « conforme » :
	// la garde déclarait saine une fonction qu'elle n'avait jamais inspectée. Ce test rend cet
	// état IMPOSSIBLE à ignorer — si une forme syntaxique nouvelle entre dans le dépôt, elle
	// rougit ici plutôt que de disparaître dans le compte des conformes.
	it('aucune déclaration candidate n’échappe à l’analyse de son corps', () => {
		expect(nonAnalyseesDe(releverLeDepot())).toEqual([]);
	});
});

describe('ce que la garde reconnaît — cas synthétiques', () => {
	it('voit un `return` de littéral dans une fonction candidate', () => {
		const src = "function statusLabel(s) {\n\tif (s) return 'Ouverte';\n\treturn s;\n}";
		expect(violationsDe(candidatesDe(src))).toEqual(['<mémoire>:2 statusLabel() → « Ouverte »']);
	});

	it('ne dit rien d’une fonction qui passe par i18nMsg', () => {
		const src =
			"function statusLabel(s) {\n\tswitch (s) {\n\t\tcase 'open':\n\t\t\treturn i18nMsg('x-open', 'Ouverte');\n\t}\n\treturn s;\n}";
		expect(violationsDe(candidatesDe(src))).toEqual([]);
	});

	it('⚠️ un `case` de code technique n’est PAS un retour — sinon la garde crie sur les conformes', () => {
		const src = "function typeLabel(t) {\n\tif (t === 'open') return i18nMsg('k', 'Ouverte');\n\treturn t;\n}";
		expect(violationsDe(candidatesDe(src))).toEqual([]);
	});

	it('⚠️ le `{` d’un TYPE INLINE dans la signature ne passe pas pour le corps', () => {
		const src = "function fooLabel(opts: { x: number }): string {\n\treturn 'Dur';\n}";
		expect(violationsDe(candidatesDe(src))).toEqual(['<mémoire>:2 fooLabel() → « Dur »']);
	});

	it('⚠️ un TYPE DE RETOUR objet ne passe pas non plus pour le corps', () => {
		const src = "function fooLabel(): { a: string } {\n\treturn 'Dur';\n}";
		expect(violationsDe(candidatesDe(src))).toEqual(['<mémoire>:2 fooLabel() → « Dur »']);
	});

	it('voit un libellé posé directement sur la déclaration', () => {
		expect(violationsDe(candidatesDe("const statusLabel = 'Ouvert';"))).toEqual([
			'<mémoire>:1 statusLabel() → « Ouvert »'
		]);
	});

	it('voit une flèche à corps de bloc, `$derived.by` comprise', () => {
		const src = "const statusLabel = $derived.by(() => {\n\treturn 'Ouvert';\n});";
		expect(violationsDe(candidatesDe(src))).toEqual(['<mémoire>:2 statusLabel() → « Ouvert »']);
	});

	it('voit une flèche à corps d’expression', () => {
		const src = "const statusLabel = (s) => 'Ouvert';";
		expect(violationsDe(candidatesDe(src))).toEqual(['<mémoire>:1 statusLabel() → « Ouvert »']);
	});

	it('écarte par CRITÈRE le cadratin et la chaîne vide — ils sont relevés, puis triés', () => {
		const src = "function aLabel() {\n\treturn '—';\n}\nfunction bLabel() {\n\treturn '';\n}";
		expect(candidatesDe(src).flatMap((c) => c.retours).length).toBe(2);
		expect(violationsDe(candidatesDe(src))).toEqual([]);
	});

	it('écarte par CRITÈRE un gabarit interpolé — ce n’est pas un libellé en dur', () => {
		const src = 'function aLabel(a, b) {\n\treturn `${a} — ${b}`;\n}';
		expect(violationsDe(candidatesDe(src))).toEqual([]);
	});

	it('une accolade dans une chaîne du corps ne tronque pas l’inspection', () => {
		const src = "function aLabel() {\n\tconst x = '}';\n\treturn 'Dur';\n}";
		expect(violationsDe(candidatesDe(src))).toEqual(['<mémoire>:3 aLabel() → « Dur »']);
	});

	it('ne relève que les noms qui promettent un libellé', () => {
		expect(candidatesDe("function total() {\n\treturn 'Dur';\n}")).toEqual([]);
	});
});

describe('régressions trouvées en passe 1 de revue (Sonnet ×3, 2026-08-21)', () => {
	// ⚠️ **Les deux premiers cas disent la même chose** : la garde comptait comme CONFORMES des
	// fonctions dont elle n'avait jamais lu le corps. C'est le mode d'échec que cette garde
	// existe pour fermer, reproduit dans la garde elle-même.
	it('voit un littéral retourné par un TERNAIRE — le patron même du domaine', () => {
		const src = "function statusLabel(s) {\n\treturn s === 'paid' ? 'Payée' : 'Ouverte';\n}";
		expect(violationsDe(candidatesDe(src))).toEqual([
			'<mémoire>:2 statusLabel() → « Payée »',
			'<mémoire>:2 statusLabel() → « Ouverte »'
		]);
	});

	it('⚠️ mais un littéral de COMPARAISON reste un code technique, pas un libellé', () => {
		// Sans cette borne, `i18nMsg('cle', 'repli')` deviendrait une violation dans TOUTES les
		// fonctions conformes : le correctif du ternaire se retournerait contre son objet.
		const src =
			"function statusLabel(s) {\n\treturn s === 'paid' ? i18nMsg('k-paid', 'Payée') : i18nMsg('k-open', 'Ouverte');\n}";
		expect(violationsDe(candidatesDe(src))).toEqual([]);
	});

	it('voit un littéral retourné entre PARENTHÈSES', () => {
		const src = "function aLabel() {\n\treturn ('Dur');\n}";
		expect(violationsDe(candidatesDe(src))).toEqual(['<mémoire>:2 aLabel() → « Dur »']);
	});

	it('voit un repli littéral derrière `||` ou `??`', () => {
		const src = "function aLabel(x) {\n\treturn x ?? 'Aucun';\n}";
		expect(violationsDe(candidatesDe(src))).toEqual(['<mémoire>:2 aLabel() → « Aucun »']);
	});

	it('analyse une EXPRESSION DE FONCTION assignée', () => {
		const src = "const statusLabel = function (s) {\n\treturn 'Ouvert';\n};";
		expect(violationsDe(candidatesDe(src))).toEqual(['<mémoire>:2 statusLabel() → « Ouvert »']);
	});

	it('relève une déclaration portant une ANNOTATION DE TYPE', () => {
		// Le motif exigeait `=` juste après le nom : `const x: string = …` n'était même pas
		// comptée — ni conforme, ni en violation, simplement hors radar.
		expect(candidatesDe("const statusLabel: string = 'Ouvert';").length).toBe(1);
		expect(violationsDe(candidatesDe("const statusLabel: string = 'Ouvert';"))).toEqual([
			'<mémoire>:1 statusLabel() → « Ouvert »'
		]);
	});

	it('relève une fonction GÉNÉRATRICE', () => {
		expect(candidatesDe("function* aLabel() {\n\treturn 'Dur';\n}").length).toBe(1);
	});

	// ⚠️ **Le correctif de FOND, et il vaut plus que les cas ci-dessus** : plutôt que de courir
	// après chaque forme syntaxique, ce qui n'a pas pu être lu doit être BRUYANT. Une candidate
	// dont le corps existe et n'a pas été analysé fait rougir la garde au lieu de gonfler le
	// compte des conformes.
	it('⚠️ une fonction dont le corps n’a PAS pu être lu fait rougir — jamais « conforme »', () => {
		const tronque = 'function aLabel(): string {\n\treturn 1;';
		const c = candidatesDe(tronque);
		expect(c.length).toBe(1);
		expect(c[0].analysee).toBe(false);
		expect(nonAnalyseesDe(c)).toEqual(['<mémoire>:1 aLabel()']);
	});

	it('une déclaration SANS corps de fonction reste analysée — elle n’a pas de corps à lire', () => {
		// `let x = $state('')` ou `$derived(expr)` : l'analyse « littéral en tête » a bien eu
		// lieu. Les classer « non analysées » ferait rougir la garde sur du code conforme.
		for (const src of ["let fLabel = $state('');", 'let equityLabel = $derived(cond ? a : b);']) {
			expect(candidatesDe(src)[0].analysee).toBe(true);
		}
	});

	// ⚠️ **Passe 2 : les FAUX POSITIFS, et ils sont plus dangereux que les faux négatifs.** Un
	// faux négatif laisse passer un défaut ; un faux positif fait rougir le gate sur du code sain,
	// et la réaction naturelle devant un gate bruyant est de DÉSACTIVER la garde. Les trois cas
	// ci-dessous crient à tort dans la rédaction de la passe 1.
	it('⚠️ une table de dispatch retournée ne crie PAS — c’est un objet, pas un libellé', () => {
		// Forme réelle et légitime : `return { open: 'Ouverte' }[s]`. La doctrine de cette garde
		// écarte déjà les tables de littéraux, « indiscernables d'une table de replis légitime ».
		const src = "function statusLabel(s) {\n\treturn { open: 'Ouverte', paid: 'Payée' }[s];\n}";
		expect(violationsDe(candidatesDe(src))).toEqual([]);
	});

	it('⚠️ un tableau retourné ne crie pas non plus', () => {
		const src = "function aLabel() {\n\treturn ['A', 'B'];\n}";
		expect(violationsDe(candidatesDe(src))).toEqual([]);
	});

	it('mais le littéral qui SUIT la structure reste vu', () => {
		const src = "function aLabel(s) {\n\treturn { a: 'x' }[s] ?? 'Aucun';\n}";
		expect(violationsDe(candidatesDe(src))).toEqual(['<mémoire>:2 aLabel() → « Aucun »']);
	});

	// ⚠️ **Limite CONNUE et assumée, verrouillée par ce test plutôt que découverte plus tard.**
	// `const a = 1, statusLabel = 'Ouvert';` — la seconde déclaration d'une liste n'est pas
	// relevée : le motif exige `const`/`let`/`var` en tête. L'élargir à la virgule ferait entrer
	// les paramètres à valeur par défaut (`function f(x, yLabel = 'a')`) comme candidates, donc
	// échangerait un faux négatif contre un faux positif — mauvais marché, cf. le préambule.
	// Aucune occurrence dans `src/` au 2026-08-21. Si ce test rougit, c'est que la forme est
	// apparue : la traiter alors pour de bon. (Passe 2 de revue.)
	it('limite assumée — la 2ᵉ déclaration d’une liste échappe au relevé', () => {
		expect(candidatesDe("const a = 1, statusLabel = 'Ouvert';")).toEqual([]);
	});

	it('un `return` imbriqué dans un gabarit n’est pas attribué à la fonction englobante', () => {
		const src = "function aLabel() {\n\treturn `${(function () { return 'Dur'; })()}`;\n}";
		expect(violationsDe(candidatesDe(src))).toEqual([]);
	});
});
