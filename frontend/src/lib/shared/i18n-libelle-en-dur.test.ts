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
const CANDIDATES_ATTENDUES = 39;

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
type Candidate = { nom: string; chemin: string; ligne: number; retours: Retour[] };

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

/** Relève les littéraux en **tête d'un retour** dans un corps de fonction. */
function retoursLitteraux(corps: string, ligneDeBase: number): Retour[] {
	const out: Retour[] = [];
	const motif = /\breturn\b/g;
	let m: RegExpExecArray | null;
	while ((m = motif.exec(corps)) !== null) {
		const i = sauterBlancs(corps, m.index + m[0].length);
		if (!QUOTES.includes(corps[i])) continue;
		const lu = readLiteral(corps, i);
		// ⚠️ Un gabarit interpolé n'est pas un libellé en dur — le lecteur les distingue.
		if (!lu || lu.kind !== 'literal') continue;
		out.push({ texte: lu.value, ligne: ligneDeBase + corps.slice(0, i).split('\n').length - 1 });
	}
	return out;
}

/**
 * Relève toutes les déclarations candidates d'une source et, pour chacune, les littéraux
 * qu'elle retourne.
 *
 * Quatre formes de déclaration sont reconnues — `function nom(…) {…}`, `const nom = (…) => {…}`,
 * `const nom = (…) => <expression>` et `const nom = <expression>` (dont `$derived`,
 * `$derived.by(() => {…})`). ⚠️ **La quatrième compte** : `const statusLabel = 'Ouvert'`
 * doit rougir comme le ferait un `return`.
 */
export function candidatesDe(source: string, chemin = '<mémoire>'): Candidate[] {
	const out: Candidate[] = [];
	const decl = /(?:function\s+(\w+)|(?:const|let|var)\s+(\w+)\s*=)/g;
	let m: RegExpExecArray | null;
	while ((m = decl.exec(source)) !== null) {
		const nom = m[1] ?? m[2];
		if (!SUFFIXES.some((s) => nom.endsWith(s))) continue;
		const ligne = source.slice(0, m.index).split('\n').length;
		const apres = m.index + m[0].length;
		const candidate: Candidate = { nom, chemin, ligne, retours: [] };
		out.push(candidate);

		// (a) `function nom(…) {…}` — l'accolade du corps, jamais celle d'un type.
		if (m[1] !== undefined) {
			const ouvrante = accoladeDeCorps(source, apres);
			if (ouvrante === null) continue;
			const corps = corpsDeFonction(source, ouvrante);
			if (corps === null) continue;
			candidate.retours = retoursLitteraux(corps, source.slice(0, ouvrante).split('\n').length);
			continue;
		}

		// (b) `const nom = <littéral>` — le libellé posé directement sur la déclaration.
		const debut = sauterBlancs(source, apres);
		if (QUOTES.includes(source[debut])) {
			const lu = readLiteral(source, debut);
			if (lu && lu.kind === 'literal') candidate.retours.push({ texte: lu.value, ligne });
			continue;
		}

		// (c) une flèche dans l'expression d'initialisation — `(…) => {…}` ou `() => <expr>`,
		// ce qui couvre aussi `$derived.by(() => {…})`.
		const fin = finInstruction(source, debut);
		const fleche = source.slice(debut, fin).indexOf('=>');
		if (fleche === -1) continue;
		const apresFleche = sauterBlancs(source, debut + fleche + 2);
		if (source[apresFleche] === '{') {
			const corps = corpsDeFonction(source, apresFleche);
			if (corps === null) continue;
			candidate.retours = retoursLitteraux(
				corps,
				source.slice(0, apresFleche).split('\n').length
			);
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

	// ⚠️ **Un total doit être cohérent avec sa propre ventilation.** Sur l'epic 16, le compte
	// rendu est devenu le lieu du défaut plus souvent que le code ; cet invariant met la
	// ventilation sous test au lieu de la laisser vivre dans un Change Log que personne ne
	// recompte. Au 2026-08-21 : 39 = 30 + 4 + 5.
	it('la ventilation se somme au total — conformes + écartées par critère + en violation', () => {
		const candidates = releverLeDepot();
		const avecRetour = candidates.filter((c) => c.retours.length > 0);
		const enViolation = avecRetour.filter((c) =>
			c.retours.some((r) => !NEUTRES.has(r.texte.trim()))
		);
		const conformes = candidates.length - avecRetour.length;
		const ecartees = avecRetour.length - enViolation.length;
		expect(conformes + ecartees + enViolation.length).toBe(candidates.length);
		expect({ conformes, ecartees, enViolation: enViolation.length }).toEqual({
			conformes: 30,
			ecartees: 4,
			enViolation: 0
		});
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
