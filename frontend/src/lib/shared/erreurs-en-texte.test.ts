/**
 * Garde contre les **erreurs converties en texte affiché** — Story 28-2 (#520).
 *
 * ⚠️ **`ApiError` est une interface, pas une sous-classe d'`Error`.** Le client HTTP lève
 * des objets simples `{ code, message, status }` ; le motif
 * `e instanceof Error ? e.message : String(e)` les affichait donc « [object Object] ».
 * Quatre écrans le faisaient encore à l'ouverture de la story (verrou de période, profils
 * bancaires), seize autres le gardaient en repli. Tous passent désormais par
 * `errorMessageOf` (`$lib/shared/utils/error-message`) ; cette garde interdit le retour du
 * motif, en lisant les **sources**.
 *
 * ## Ce qu'elle voit, et ce qu'elle NE PEUT PAS voir
 *
 * Elle recueille les **liaisons d'erreur** de chaque fichier — paramètre de `catch (x)` /
 * `catch (x: T)`, paramètre d'un rappel `.catch((x) => …)`, `.catch(x => …)`,
 * `.catch(function (x) …)`, typé ou `async` — et cherche dans leur **portée** :
 *
 * | règle | forme interdite | portée |
 * |---|---|---|
 * | G1 | `String(x)` | la liaison |
 * | G2 | `String(<expr>.reason)` (résultat d'`allSettled`) | tout le fichier |
 * | G3 | `instanceof Error`, **tout usage**, classification comprise | tout le fichier |
 * | G4 | `${x}` dans un gabarit | la liaison |
 * | G5 | `x` opérande entier d'un `+` / `+=` : `x + ''`, `'Erreur : ' + x`, `m += x` | la liaison |
 * | G6 | `x.toString()` | la liaison |
 * | G7 | `JSON.stringify(x` | la liaison |
 *
 * **Exemption par critère, jamais par entrée** : une occurrence placée dans la liste
 * d'arguments d'un appel `console.(log|info|warn|error|debug)(` est permise — journaliser
 * n'est pas afficher. Trois journalisations du dépôt l'exercent au parcours réel
 * (`hooks.client.ts`, `auth.svelte.ts`, `api-client.ts`, dont deux multi-lignes).
 *
 * ⛔ **Une portée incalculable fait ÉCHOUER la garde**, sur ses trois chemins (bloc d'un
 * `catch` ou d'un rappel, corps-expression d'un `.catch(`, arguments d'un `console.*`) :
 * un `null` qui faisait jeter un site **en silence** est exactement le précédent de la
 * 23-1a. Une liaison dont la portée n'a pas été inspectée n'est pas une liaison saine.
 *
 * | forme du défaut | vue ? | pourquoi |
 * |---|---|---|
 * | conversion par une **fonction intermédiaire** (`const f = (x: unknown) => String(x)`) | ❌ non | la garde connaît les liaisons, pas le flot ; seule G3 voit une telle fonction si elle emploie `instanceof Error` — `formatError` (`reports`) et `errorMessageOf` lui-même sont tenus par leurs tests |
 * | erreur **copiée** dans une variable (`const err = r.reason;`, `const x = e`) puis convertie | ❌ non | ni liaison (G1), ni `String(<expr>.reason)` (G2) |
 * | conversion **dans le balisage** (`{String(err)}`) d'un état qui contiendrait une erreur brute | ❌ non | pas une liaison |
 * | liaison **déstructurée** (`catch ({ message })`) | ❌ non | aucune à l'écriture de la garde |
 * | bloc Svelte `{:catch err}` | ❌ non | aucun à l'écriture de la garde |
 * | rejet **sans** `catch` du tout | ❌ non | rien à recueillir (le cas de `handleDelete` des profils bancaires, corrigé par la story) |
 * | affichage réservé aux `ApiError`, **rien** pour le reste (classe D) | ❌ non | un silence n'est pas une conversion ; suivi par #644 |
 * | `String(x ?? '')`, `String(x as T)` | ❌ non | G1 ne reconnaît que `String(<liaison>)` seul |
 * | `x?.toString()` | ❌ non | G6 exige `x.toString()` |
 * | `(x as Error).message` sans garde | ❌ non | aucune règle : ce n'est pas une conversion, c'est un champ (`undefined` pour un objet sans `message`) |
 * | `String(f(y).reason)`, `String(r.at(0).reason)` | ❌ non | G2 n'admet pas de parenthèse avant `.reason` |
 * | rappel de rejet en **second argument** de `.then(ok, (x) => …)` | ❌ non | seul `.catch(` est recueilli |
 * | `(x) + ''`, `'a' + (x)`, `(x as T) + ''`, `x as T + ''` | ❌ non | G5 exige la liaison nue, adjacente au `+` |
 * | `'a'.concat(x)`, `[x].join(…)` | ❌ non | G5 ne voit que l'opérateur `+` |
 *
 * Aucune de ces formes n'est appliquée à une liaison d'erreur dans `src/` à l'écriture de la garde (revue P1, E-L4 ;
 * revue P2 ciblée, P2-3 et P2-4). `String(x ?? '')` y existe, mais sur des valeurs qui ne sont pas des erreurs.
 *
 * **Faux positif bruyant, donc sans danger** : un `+` unaire (`+x`) ou une addition
 * numérique (`x + 1`) sur la liaison relève de G5 ; aucune n'a de sens sur une erreur.
 * G3 interdit **toute** occurrence d'`instanceof Error`, classification comprise
 * (C-28-2-27) : c'est un coût de maintenance assumé, le message d'échec indique
 * l'alternative (revue P1, B-3).
 *
 * ## Anti-muet
 *
 * Des **planchers par forme** de liaison (C-28-2-17, C-28-2-28) : un plancher global
 * laissait passer un recueil qui perdrait toutes les liaisons d'une petite forme. Les deux
 * petites formes portent un plancher **égal à leur mesure** — sur 4 ou 5 occurrences, une
 * marge vaut absence de contrôle. Si une liaison est retirée légitimement, le plancher
 * rougit et se baisse **avec la raison écrite** au-dessus de lui.
 */
import { describe, it, expect } from 'vitest';
import { readFileSync, readdirSync } from 'node:fs';
import { join } from 'node:path';
import { blocEquilibre, corpsDeFonction, masquerCommentaires } from './i18n-literal-reader.js';
import { dansLePerimetreDeFichier } from './i18n-harvest.js';

const RACINE = 'src';

/**
 * Planchers du recueil, mesurés à `5aad8b35` + la story (commentaires masqués) :
 * 319 fichiers, 160 `catch (x)`, 4 `catch (x: T)`, 5 rappels `.catch` liés.
 */
const PLANCHERS = {
	fichiers: 310,
	catchNonType: 150,
	// = la mesure : `RulesList.svelte` ×2, `reconciliation/rules/+page.svelte`,
	// `RuleFormModal.svelte`.
	catchType: 4,
	// = la mesure : `mode.svelte.ts`, `InvoiceForm.svelte`, `vat-rates.store.svelte.ts`,
	// `audit-log/+page.svelte`, `bank-import/profiles/[id]/+page.svelte`.
	rappel: 5,
};

type Forme = 'catchNonType' | 'catchType' | 'rappel';

interface Liaison {
	nom: string;
	forme: Forme;
	ligne: number;
}

interface Constat {
	regle: string;
	ligne: number;
	extrait: string;
}

interface Releve {
	liaisons: Liaison[];
	violations: Constat[];
	/** Portées incalculables : chacune fait échouer la garde. */
	incalculables: Constat[];
}

/** Numéro de ligne (base 1) de la position `i`. */
function ligneDe(source: string, i: number): number {
	let n = 1;
	for (let j = 0; j < i && j < source.length; j += 1) if (source[j] === '\n') n += 1;
	return n;
}

/** Le texte de la ligne qui contient `i`, rogné — pour le message d'échec. */
function extraitDe(source: string, i: number): string {
	const debut = source.lastIndexOf('\n', i - 1) + 1;
	const fin = source.indexOf('\n', i);
	return source.slice(debut, fin === -1 ? source.length : fin).trim();
}

/** Position du premier caractère non blanc à partir de `i`. */
function sauterBlancs(source: string, i: number): number {
	while (i < source.length && /\s/.test(source[i])) i += 1;
	return i;
}

interface Intervalle {
	debut: number;
	fin: number;
}

/**
 * Les listes d'arguments des appels `console.*` — l'exemption. Une liste non équilibrée
 * est une portée incalculable : elle n'exempte rien, elle fait échouer.
 */
function appelsConsole(source: string, incalculables: Constat[]): Intervalle[] {
	const out: Intervalle[] = [];
	const motif = /\bconsole\.(?:log|info|warn|error|debug)\s*\(/g;
	let m: RegExpExecArray | null;
	while ((m = motif.exec(source)) !== null) {
		const ouvrante = m.index + m[0].length - 1;
		const contenu = blocEquilibre(source, ouvrante, '(', ')');
		if (contenu === null) {
			incalculables.push({
				regle: 'arguments de console.* non équilibrés',
				ligne: ligneDe(source, m.index),
				extrait: extraitDe(source, m.index),
			});
			continue;
		}
		out.push({ debut: ouvrante + 1, fin: ouvrante + 1 + contenu.length });
	}
	return out;
}

interface Portee {
	liaison: Liaison;
	debut: number;
	texte: string;
}

/**
 * Recueille les liaisons d'erreur et leur portée. Un `null` de portée est consigné en
 * `incalculables`, jamais sauté.
 */
function recueillir(source: string, incalculables: Constat[]): Portee[] {
	const out: Portee[] = [];
	const incalculable = (i: number, regle: string) =>
		incalculables.push({ regle, ligne: ligneDe(source, i), extrait: extraitDe(source, i) });

	// 1. `catch (x) {` / `catch (x: T) {` — et non `.catch(` (rappel, ci-dessous).
	const motifCatch = /(^|[^.\w$])catch\s*\(/g;
	let m: RegExpExecArray | null;
	while ((m = motifCatch.exec(source)) !== null) {
		const debutMot = m.index + m[1].length;
		const ouvrante = m.index + m[0].length - 1;
		const signature = blocEquilibre(source, ouvrante, '(', ')');
		if (signature === null) {
			incalculable(debutMot, 'liaison de catch non fermée');
			continue;
		}
		const lie = /^\s*([A-Za-z_$][\w$]*)\s*(:[\s\S]*)?$/.exec(signature);
		if (!lie) continue; // liaison déstructurée : angle mort écrit au préambule
		const accolade = sauterBlancs(source, ouvrante + signature.length + 2);
		const corps = source[accolade] === '{' ? corpsDeFonction(source, accolade) : null;
		const liaison: Liaison = {
			nom: lie[1],
			forme: lie[2] ? 'catchType' : 'catchNonType',
			ligne: ligneDe(source, debutMot),
		};
		if (corps === null) {
			incalculable(debutMot, 'bloc de catch non équilibré');
			continue;
		}
		out.push({ liaison, debut: accolade + 1, texte: corps });
	}

	// 2. Rappels `.catch(…)` liés : flèche parenthésée ou nue, `function`, `async` ou non.
	const motifRappel = /\.catch\s*\(/g;
	while ((m = motifRappel.exec(source)) !== null) {
		const ouvrante = m.index + m[0].length - 1;
		const apres = source.slice(ouvrante + 1);
		const forme =
			/^\s*(?:async\s+)?function\b[^(]*\(\s*([A-Za-z_$][\w$]*)\s*(?::[^)]*)?\)\s*/.exec(apres) ??
			/^\s*(?:async\s*)?\(\s*([A-Za-z_$][\w$]*)\s*(?::[^)]*)?\)\s*=>\s*/.exec(apres) ??
			/^\s*(?:async\s+)?([A-Za-z_$][\w$]*)\s*=>\s*/.exec(apres);
		if (!forme) continue; // `.catch(() => …)`, `.catch(handler)` : aucune liaison
		const liaison: Liaison = { nom: forme[1], forme: 'rappel', ligne: ligneDe(source, m.index) };
		const debutCorps = ouvrante + 1 + forme[0].length;
		if (source[debutCorps] === '{') {
			const corps = corpsDeFonction(source, debutCorps);
			if (corps === null) {
				incalculable(m.index, 'bloc de rappel .catch non équilibré');
				continue;
			}
			out.push({ liaison, debut: debutCorps + 1, texte: corps });
			continue;
		}
		// Corps-expression : jusqu'à la parenthèse qui ferme `.catch(`.
		const appel = blocEquilibre(source, ouvrante, '(', ')');
		if (appel === null) {
			incalculable(m.index, 'corps-expression de .catch non équilibré');
			continue;
		}
		const decalage = debutCorps - (ouvrante + 1);
		out.push({ liaison, debut: debutCorps, texte: appel.slice(decalage) });
	}
	return out;
}

/** Les règles cherchées dans la portée d'une liaison, à partir de son nom. */
function reglesDeLiaison(nom: string): [string, RegExp][] {
	const x = `(?<![\\w$.])${nom.replace(/\$/g, '\\$')}(?![\\w$])`;
	return [
		['G1 String(<liaison>)', new RegExp(`\\bString\\(\\s*${x}\\s*\\)`, 'g')],
		[
			'G4 ${<liaison>} dans un gabarit',
			new RegExp(`\\$\\{\\s*(?:${x}|String\\(\\s*${x}\\s*\\))\\s*\\}`, 'g'),
		],
		[
			// Revue P1 E-M1 : toute concaténation, pas seulement avec la chaîne vide —
			// `'Erreur : ' + err` rend « Erreur : [object Object] ». La liaison doit être
			// l'opérande ENTIER : `err.message + ' …'`, `'… ' + err.code` ne relèvent pas.
			'G5 <liaison> opérande d’un + (concaténation)',
			new RegExp(`${x}\\s*\\+(?!\\+)|(?<!\\+)\\+=?\\s*${x}(?!\\s*(?:\\??\\.|\\[|\\())`, 'g'),
		],
		['G6 <liaison>.toString()', new RegExp(`${x}\\.toString\\(\\s*\\)`, 'g')],
		['G7 JSON.stringify(<liaison>', new RegExp(`\\bJSON\\.stringify\\(\\s*${x}`, 'g')],
	];
}

/** Les règles cherchées dans tout le fichier. */
const REGLES_DE_FICHIER: [string, RegExp][] = [
	['G2 String(<expr>.reason)', /\bString\(\s*[^()]*\.reason\s*\)/g],
	[
		'G3 instanceof Error — pour classer, préférer err.name, instanceof DOMException ou instanceof TypeError',
		/\binstanceof\s+Error\b/g,
	],
];

/**
 * Détecte, dans une source **brute** (les commentaires sont masqués ici), les liaisons
 * d'erreur, les conversions interdites et les portées incalculables.
 */
function detecter(brute: string): Releve {
	const source = masquerCommentaires(brute);
	const incalculables: Constat[] = [];
	const exemptions = appelsConsole(source, incalculables);
	const exempte = (i: number) => exemptions.some((e) => i >= e.debut && i < e.fin);
	const violations: Constat[] = [];
	const constater = (regle: string, i: number) => {
		if (exempte(i)) return;
		violations.push({ regle, ligne: ligneDe(source, i), extrait: extraitDe(source, i) });
	};

	const portees = recueillir(source, incalculables);
	for (const { liaison, debut, texte } of portees) {
		for (const [regle, motif] of reglesDeLiaison(liaison.nom)) {
			let m: RegExpExecArray | null;
			while ((m = motif.exec(texte)) !== null) constater(regle, debut + m.index);
		}
	}
	for (const [regle, motif] of REGLES_DE_FICHIER) {
		motif.lastIndex = 0;
		let m: RegExpExecArray | null;
		while ((m = motif.exec(source)) !== null) constater(regle, m.index);
	}
	return { liaisons: portees.map((p) => p.liaison), violations, incalculables };
}

/** Parcourt `src/` : un relevé par fichier du périmètre. */
function releverLeDepot(racine: string): { chemin: string; releve: Releve }[] {
	const out: { chemin: string; releve: Releve }[] = [];
	const parcourir = (rep: string) => {
		for (const e of readdirSync(rep, { withFileTypes: true })) {
			const chemin = join(rep, e.name);
			if (e.isDirectory()) {
				parcourir(chemin);
				continue;
			}
			if (!dansLePerimetreDeFichier(e.name)) continue;
			out.push({ chemin, releve: detecter(readFileSync(chemin, 'utf-8')) });
		}
	};
	parcourir(racine);
	return out;
}

describe('garde — une erreur attrapée ne se convertit pas en texte affiché (Story 28-2)', () => {
	const depot = releverLeDepot(RACINE);
	const compte = (forme: Forme) =>
		depot.reduce((n, f) => n + f.releve.liaisons.filter((l) => l.forme === forme).length, 0);

	it('aucune conversion interdite dans les sources', () => {
		const constats = depot.flatMap(({ chemin, releve }) =>
			releve.violations.map((v) => `${chemin}:${v.ligne} — ${v.regle} — ${v.extrait}`)
		);
		expect(
			constats,
			'Passer par errorMessageOf ($lib/shared/utils/error-message) : ApiError → son message, tout le reste → le générique traduit.'
		).toEqual([]);
	});

	it('aucune portée incalculable', () => {
		const constats = depot.flatMap(({ chemin, releve }) =>
			releve.incalculables.map((v) => `${chemin}:${v.ligne} — ${v.regle} — ${v.extrait}`)
		);
		expect(constats).toEqual([]);
	});

	it('le recueil ne passe pas à vide — planchers par forme', () => {
		const releve = {
			fichiers: depot.length,
			catchNonType: compte('catchNonType'),
			catchType: compte('catchType'),
			rappel: compte('rappel'),
		};
		// Informatif (C-28-2-28) : le total ne porte pas de plancher, la somme des trois
		// planchers de forme le borne déjà.
		console.info(
			`[erreurs-en-texte] ${releve.fichiers} fichiers, ${releve.catchNonType + releve.catchType + releve.rappel} liaisons`,
			releve
		);
		expect(releve.fichiers).toBeGreaterThanOrEqual(PLANCHERS.fichiers);
		expect(releve.catchNonType).toBeGreaterThanOrEqual(PLANCHERS.catchNonType);
		expect(releve.catchType).toBeGreaterThanOrEqual(PLANCHERS.catchType);
		expect(releve.rappel).toBeGreaterThanOrEqual(PLANCHERS.rappel);
	});
});

describe('garde — auto-tests du détecteur, sur sources fictives', () => {
	const regles = (src: string) => detecter(src).violations.map((v) => v.regle.slice(0, 2));

	describe('chaque règle a sa fictive positive', () => {
		it('G1 String(<liaison>)', () => {
			expect(regles('try { f(); } catch (e) { msg = String(e); }')).toEqual(['G1']);
		});
		it('G2 String(<expr>.reason)', () => {
			expect(regles('const [r] = await Promise.allSettled([p]); m = String(r.reason);')).toEqual(['G2']);
		});
		it('G3 en ternaire (et G1 sur la même ligne)', () => {
			expect(regles('try {} catch (e) { m = e instanceof Error ? e.message : String(e); }')).toEqual([
				'G1',
				'G3',
			]);
		});
		it('G3 en if/else, sans String', () => {
			expect(
				regles('try {} catch (e) { if (e instanceof Error) msg = e.message; else msg = repli; }')
			).toEqual(['G3']);
		});
		it('G3 aussi pour une classification (C-28-2-27)', () => {
			expect(regles("if (err instanceof Error && err.name === 'AbortError') return;")).toEqual(['G3']);
		});
		it('G3 respecte les bornes de mot : instanceof ErrorEvent n’en relève pas', () => {
			expect(regles('if (ev instanceof ErrorEvent) log(ev);')).toEqual([]);
		});
		it('G4 ${<liaison>} dans un gabarit', () => {
			expect(regles('try {} catch (err) { m = `Erreur : ${err}`; }')).toEqual(['G4']);
		});
		it('G4 et G1 pour ${String(<liaison>)}', () => {
			expect(regles('try {} catch (err) { m = `Erreur : ${String(err)}`; }')).toEqual(['G1', 'G4']);
		});
		it("G5 <liaison> + '' et '' + <liaison>", () => {
			expect(regles("try {} catch (err) { a = err + ''; b = \"\" + err; }")).toEqual(['G5', 'G5']);
		});
		it('G5 concaténation avec une chaîne non vide, des deux côtés (revue P1 E-M1)', () => {
			expect(regles("try {} catch (err) { a = 'Erreur : ' + err; }")).toEqual(['G5']);
			expect(regles("try {} catch (err) { a = err + ' (détail)'; }")).toEqual(['G5']);
			expect(regles('try {} catch (err) { a = `Erreur : ` + err; }')).toEqual(['G5']);
			// Encadrée, sans blancs : un seul constat, la liaison est consommée par le premier `+`.
			expect(regles("try {} catch (err) { a = 'Erreur : '+err+'.'; }")).toEqual(['G5']);
		});
		it('G5 multi-ligne et += (revue P1 E-M1)', () => {
			expect(regles("try {} catch (err) {\n\tmsg = 'Erreur : ' +\n\t\terr;\n}")).toEqual(['G5']);
			expect(regles("try {} catch (err) { msg += err; }")).toEqual(['G5']);
		});
		it('G5 ne relève pas un champ de la liaison, ni ++, ni une autre variable (revue P1 E-M1)', () => {
			expect(
				regles(
					"try {} catch (err) { a = 'Erreur : ' + err.message; b = err.code + ' x'; c = 'x' + err?.code; d = 'x' + err[0]; e = 'x' + err2; f = 'x' + myerr; }"
				)
			).toEqual([]);
			expect(regles('try {} catch (e) { n++; m = n + 1; }')).toEqual([]);
		});
		it('G6 <liaison>.toString()', () => {
			expect(regles('try {} catch (err) { m = err.toString(); }')).toEqual(['G6']);
		});
		it('G7 JSON.stringify(<liaison>', () => {
			expect(regles('try {} catch (err) { m = JSON.stringify(err); }')).toEqual(['G7']);
		});
	});

	describe('exemption `console`, par critère', () => {
		it('mono-ligne : une conversion dans console.error est permise', () => {
			expect(regles("try {} catch (e) { console.error('x', e instanceof Error ? e.message : String(e)); }")).toEqual(
				[]
			);
		});
		it('multi-ligne : idem', () => {
			const src = [
				'try {} catch (error) {',
				'\tconsole.warn(',
				"\t\t'[x] échec :',",
				'\t\terror instanceof Error ? error.message : String(error),',
				'\t);',
				'}',
			].join('\n');
			expect(regles(src)).toEqual([]);
		});
		it('la même conversion HORS de l’appel, dans un fichier qui journalise, est signalée (M16)', () => {
			const src = [
				'try {} catch (e) {',
				"\tconsole.error('x', String(e));",
				'\tmsg = String(e);',
				'}',
			].join('\n');
			const r = detecter(src).violations;
			expect(r.map((v) => [v.regle.slice(0, 2), v.ligne])).toEqual([['G1', 3]]);
		});
		it('étendue : une conversion après l’appel, SUR LA MÊME LIGNE, est signalée (revue P1 E-L3)', () => {
			expect(regles("try {} catch (e) { console.error('a'); m = String(e); }")).toEqual(['G1']);
		});
		it('borne basse : une conversion en tout premier argument est exemptée (revue P1 E-L3)', () => {
			expect(regles('try {} catch (e) { console.error(String(e)); }')).toEqual([]);
		});
	});

	describe('chaque forme de liaison est recueillie, et sa portée inspectée', () => {
		const formes = (src: string) => detecter(src).liaisons.map((l) => [l.nom, l.forme]);
		it('catch (caught) — nom quelconque', () => {
			const src = 'try {} catch (caught) { m = String(caught); }';
			expect(formes(src)).toEqual([['caught', 'catchNonType']]);
			expect(regles(src)).toEqual(['G1']);
		});
		it('catch (e: unknown) — typée', () => {
			const src = 'try {} catch (e: unknown) { m = String(e); }';
			expect(formes(src)).toEqual([['e', 'catchType']]);
			expect(regles(src)).toEqual(['G1']);
		});
		it('.catch((err) => …${String(err)}…) — corps-expression', () => {
			const src = 'p.catch((err) => notifier(`Échec : ${String(err)}`));';
			expect(formes(src)).toEqual([['err', 'rappel']]);
			expect(regles(src)).toEqual(['G1', 'G4']);
		});
		it('.catch(err => String(err)) — flèche nue', () => {
			const src = 'p.catch(err => String(err));';
			expect(formes(src)).toEqual([['err', 'rappel']]);
			expect(regles(src)).toEqual(['G1']);
		});
		it('.catch(function (x) { … String(x) … })', () => {
			const src = 'p.catch(function (x) { m = String(x); });';
			expect(formes(src)).toEqual([['x', 'rappel']]);
			expect(regles(src)).toEqual(['G1']);
		});
		it('.catch((err: unknown) => String(err)) — typée', () => {
			const src = 'p.catch((err: unknown) => String(err));';
			expect(formes(src)).toEqual([['err', 'rappel']]);
			expect(regles(src)).toEqual(['G1']);
		});
		it('.catch(async (err) => { … String(err) … }) — async', () => {
			const src = 'p.catch(async (err) => { await f(); m = String(err); });';
			expect(formes(src)).toEqual([['err', 'rappel']]);
			expect(regles(src)).toEqual(['G1']);
		});
		it('.catch(async err => …) et .catch(async function (x) {…})', () => {
			expect(formes('p.catch(async err => String(err));')).toEqual([['err', 'rappel']]);
			expect(formes('p.catch(async function (x) { String(x); });')).toEqual([['x', 'rappel']]);
		});
		it('catch sans liaison, .catch(() => …) et .catch(handler) : aucune liaison', () => {
			expect(formes('try {} catch { m = 1; } p.catch(() => 0); q.catch(handler);')).toEqual([]);
		});
	});

	describe('portée et littéraux', () => {
		it("accolades dans une chaîne, un gabarit imbriqué et une regex ne coupent pas la portée", () => {
			const src = [
				'try {} catch (err) {',
				"\tconst a = '{';",
				'\tconst b = "}";',
				'\tconst c = `a ${`b ${n} }`} {`;',
				'\tconst d = /[{]/;',
				'\tmsg = String(err);',
				'}',
			].join('\n');
			expect(detecter(src).violations.map((v) => [v.regle.slice(0, 2), v.ligne])).toEqual([['G1', 6]]);
		});
		it('un String(err) après la fermeture du catch, sur une autre variable, n’est pas signalé', () => {
			const src = 'try {} catch (err) { log(1); }\nconst err = 3;\nm = String(err);';
			expect(regles(src)).toEqual([]);
		});
	});

	describe('portée incalculable : un échec nommé sur les trois chemins (M24, M27)', () => {
		const lignes = (src: string) => detecter(src).incalculables.map((c) => c.ligne);
		it('catch (err) { jamais fermé', () => {
			expect(lignes('f();\ntry {} catch (err) {\n\tm = String(err);\n')).toEqual([2]);
		});
		it('.catch((err) => String(err) — parenthèse jamais fermée', () => {
			expect(lignes('f();\np.catch((err) => String(err);\n')).toEqual([2]);
		});
		it('console.error( non fermé suivi d’un String(err)', () => {
			const r = detecter('try {} catch (err) { console.error(String(err); }');
			expect(r.incalculables.map((c) => c.ligne)).toEqual([1]);
			// La conversion n'est pas exemptée en silence par un appel non localisé.
			expect(r.violations.map((v) => v.regle.slice(0, 2))).toEqual(['G1']);
		});
	});

	describe('ce qui n’est pas signalé', () => {
		it('String(x) sur une variable qui n’est pas une liaison d’erreur', () => {
			expect(regles('const n = 3; m = String(n); try {} catch (e) { log(1); }')).toEqual([]);
		});
		it('un commentaire contenant String(err)', () => {
			expect(regles('try {} catch (err) {\n\t// m = String(err);\n\t/* String(err) */\n}')).toEqual([]);
		});
	});
});
