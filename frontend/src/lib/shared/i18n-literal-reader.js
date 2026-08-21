/**
 * Lecture des littéraux JavaScript/TypeScript — module PARTAGÉ.
 *
 * ⚠️ **Pourquoi ce module existe séparément de ses appelants** (story 23-1a, D1-bis).
 * Deux outils ont besoin du même lecteur : la garde `i18n-keys.test.ts` (vitest) et
 * le moissonneur `scripts/harvest-i18n-fallbacks.mjs` de la story 23-1b, qui est un
 * script `node` nu. Un script `node` ne peut pas importer un symbole d'un `.test.ts`.
 * Sans ce module, la seule issue serait la recopie — et une copie de lecteur qui
 * dérive est exactement le défaut que ce lecteur existe pour empêcher.
 *
 * ⚠️ **Le défaut qu'il remplace, et qui a coûté cinq erreurs à la story 23-1.**
 * Une expression régulière à classe de caractères négative — `[^'"`]*` — ne peut PAS
 * lire un littéral entre guillemets doubles contenant une apostrophe :
 *
 *     i18nMsg('payment-batches-col-date', "Date d'exécution")
 *                                          ↑ la classe s'arrête ici
 *
 * Elle ne traverse pas non plus un gabarit dont l'interpolation contient des quotes :
 *
 *     i18nMsg(`bank-import-info-${info.replace(/_/g, '-')}`, info)
 *
 * D'où un automate caractère par caractère, qui respecte l'échappement et
 * l'imbrication. Il n'y a pas de version « plus simple » de ce lecteur qui soit juste.
 */

/** Les trois délimiteurs de littéral en JS/TS. */
const QUOTES = ["'", '"', '`'];

/**
 * Lit un littéral à partir de la position `i` de `source`.
 *
 * @param {string} source  le texte du fichier
 * @param {number} i       position du délimiteur ouvrant
 * @returns {{kind: 'literal'|'template', value: string, end: number} | null}
 *   - `literal`  : chaîne close, `value` est son contenu déséchappé ;
 *   - `template` : gabarit contenant au moins une interpolation `${…}` ; `value` est
 *     le gabarit **avec** ses `${…}` conservés tels quels, pour que l'appelant puisse
 *     le reconnaître comme motif dynamique ;
 *   - `null`     : la position ne porte pas de littéral, ou il n'est pas clos.
 */
export function readLiteral(source, i) {
	if (i >= source.length || !QUOTES.includes(source[i])) return null;
	const quote = source[i];
	/** @type {string[]} */
	const out = [];
	let depth = 0; // profondeur d'imbrication des `${ … }`
	// ⚠️ Le TYPE se décide sur ce qui s'est réellement passé pendant la lecture, jamais
	// en relisant la valeur : `'a${b}c'` entre guillemets SIMPLES n'est pas un gabarit —
	// une chaîne `'…'` ne peut pas l'être en JS —, et un `\${` ÉCHAPPÉ dans un vrai
	// gabarit est volontairement inerte. Une recherche de sous-chaîne `${` dans le
	// résultat classait les deux comme dynamiques. (Revue de code 23-1a, passe 1.)
	let interpolationVue = false;
	let j = i + 1;

	while (j < source.length) {
		const c = source[j];

		// Échappement : le caractère suivant est littéral, quel qu'il soit.
		if (c === '\\') {
			out.push(source[j + 1] ?? '');
			j += 2;
			continue;
		}

		// Ouverture d'une interpolation, seulement dans un gabarit.
		if (quote === '`' && c === '$' && source[j + 1] === '{') {
			depth += 1;
			interpolationVue = true;
			out.push('${');
			j += 2;
			continue;
		}

		if (depth > 0) {
			// À l'intérieur d'une interpolation : on suit les accolades, et on saute
			// les littéraux imbriqués — c'est là que `info.replace(/_/g, '-')` piège
			// une expression régulière naïve.
			if (c === '/' && estDebutDeRegex(source, j)) {
				// ⚠️ Une regex peut contenir une accolade NON APPARIÉE (`/{/`) ou une
				// quote (`/['"]/`). Sans ce saut, la première désynchronise `depth` et la
				// seconde ouvre un faux littéral : dans les deux cas la lecture du gabarit
				// part à la dérive. Le commentaire ci-dessus promettait cette protection ;
				// il ne la décrivait pas. (Revue de code 23-1a, passe 1.)
				const finRegex = finDeRegex(source, j);
				if (finRegex !== null) {
					out.push(source.slice(j, finRegex));
					j = finRegex;
					continue;
				}
			}
			if (c === '{') depth += 1;
			else if (c === '}') depth -= 1;
			else if (QUOTES.includes(c)) {
				const inner = readLiteral(source, j);
				if (inner) {
					out.push(source.slice(j, inner.end));
					j = inner.end;
					continue;
				}
			}
			out.push(c);
			j += 1;
			continue;
		}

		// Fermeture du littéral.
		if (c === quote) {
			const value = out.join('');
			return {
				kind: interpolationVue ? 'template' : 'literal',
				value,
				end: j + 1
			};
		}

		out.push(c);
		j += 1;
	}

	return null; // littéral non clos — fichier tronqué ou source invalide
}

/**
 * Vrai si la quote en position `i` ouvre réellement un littéral de chaîne.
 *
 * ⚠️ **Un fichier `.svelte` n'est pas du JavaScript.** Son balisage contient de la prose,
 * et le français y met des apostrophes : `l'exercice`, `d'abord`. Traitées comme des
 * ouvertures de chaîne, elles avalent tout jusqu'à l'apostrophe suivante — plusieurs
 * dizaines de lignes, commentaires compris, qui cessent alors d'être masqués.
 * *C'est arrivé : la première rédaction du masquage laissait passer un commentaire sur
 * trois.* (Revue de code 23-1a, passe 1.)
 *
 * Le critère est net : une chaîne JavaScript n'est **jamais** précédée d'une lettre ou
 * d'un chiffre — elle suit un espace, `(`, `,`, `=`, `[`, `:`, `{`… — tandis qu'une
 * apostrophe de prose l'est presque toujours.
 *
 * @param {string} source
 * @param {number} i
 * @returns {boolean}
 */
function ouvreUnLitteral(source, i) {
	// ⚠️ **Un backtick ouvre TOUJOURS un littéral en JavaScript**, étiqueté ou non
	// (`String.raw\`…\``). Le refuser quand il suit une lettre ou une parenthèse avait
	// une conséquence mesurée : dans un commentaire de prose, le backtick **fermant** de
	// `` - Émet `onConfirm({ paidAt })` ; `` était accepté comme OUVRANT et lançait un
	// gabarit qui avalait jusqu'à **58 lignes** — cinq fichiers `.svelte` réels, dix-neuf
	// commentaires qui échappaient au masquage. (Revue de code 23-1a, passe 3.)
	if (source[i] === '`') return true;
	let k = i - 1;
	while (k >= 0 && ' \t'.includes(source[k])) k -= 1;
	if (k < 0) return true;
	// `\p{L}` couvre TOUT l'Unicode — pas seulement le latin étendu : une apostrophe
	// après un emoji ou un idéogramme rouvrait un faux littéral.
	if (!/[\p{L}\p{N}_$]/u.test(source[k])) return true;
	// …mais un mot-clé suivi d'une quote SANS espace reste une ouverture valide :
	// `return'x'`, `typeof'x'`, `case'x'` sont du JavaScript légitime.
	let d = k;
	while (d >= 0 && /[\p{L}\p{N}_$]/u.test(source[d])) d -= 1;
	return MOTS_CLES_AVANT_LITTERAL.has(source.slice(d + 1, k + 1));
}

/** Mots-clés après lesquels une quote collée ouvre bien une chaîne. */
const MOTS_CLES_AVANT_LITTERAL = new Set([
	'return', 'typeof', 'case', 'in', 'of', 'delete', 'void', 'do', 'else',
	'yield', 'await', 'new', 'throw', 'instanceof',
	// ⚠️ Sans ceux-ci, **1111 littéraux réels du dépôt** étaient refusés — les chemins
	// d'import (`from '@sveltejs/kit'`) en tête. Sans conséquence mesurée, mais c'est le
	// mécanisme de F2 : un refus de trop, et un commentaire cesse d'être masqué.
	'from', 'as', 'import', 'export', 'default', 'satisfies', 'extends'
]);

/**
 * Vrai si le `/` en position `i` ouvre un littéral de regex plutôt qu'une division.
 *
 * Heuristique standard : une regex ne peut suivre qu'un opérateur, une ouverture, une
 * virgule ou un début d'expression — jamais un identifiant, un nombre ou une fermeture.
 * Elle suffit ici parce qu'on ne lit que l'intérieur d'une interpolation.
 *
 * @param {string} source
 * @param {number} i
 * @returns {boolean}
 */
function estDebutDeRegex(source, i) {
	let k = i - 1;
	while (k >= 0 && ' \t\r\n'.includes(source[k])) k -= 1;
	if (k < 0) return true;
	// ⚠️ `>` couvre la fin d'une flèche `=>`, et les mots-clés sont indispensables :
	// `return /['"]/.test(s)` était lu comme une division, la quote de la classe ouvrait
	// une fausse chaîne, et le commentaire suivant échappait au masquage. C'est le même
	// besoin que `ouvreUnLitteral`, resté sans réponse chez son jumeau une passe durant.
	// ⚠️ **`}` N'EST PAS dans ce jeu, et son retrait est un correctif.** Il y avait été
	// ajouté en passe 3 sans justification ni preuve, alors qu'il est **ambigu** : il
	// ferme un bloc comme un objet, si bien que `{…} / 2` est une division licite.
	// Prise pour une regex, elle faisait chercher un `/` fermant jusqu'au `//` d'un
	// commentaire de la même ligne — qui échappait alors au masquage.
	// (Revue de code 23-1a, passe 4.)
	if ('(,=:[!&|?{;+-*%~^>'.includes(source[k])) return true;
	let d = k;
	while (d >= 0 && /[\p{L}\p{N}_$]/u.test(source[d])) d -= 1;
	return MOTS_CLES_AVANT_LITTERAL.has(source.slice(d + 1, k + 1));
}

/**
 * Rend la position juste après la regex ouverte en `i`, ou `null` si elle n'est pas
 * close sur la même ligne (auquel cas ce n'était pas une regex).
 *
 * @param {string} source
 * @param {number} i
 * @returns {number | null}
 */
function finDeRegex(source, i) {
	// ⚠️ `//` et `/*` ne sont JAMAIS des regex : ce sont des commentaires. Une regex
	// vide s'écrit `/(?:)/`. Sans ce garde, `//` était lu comme une regex vide et le
	// commentaire échappait au masquage. (Trouvé en écrivant la preuve du correctif de
	// la passe 2 — le correctif avait son propre défaut.)
	if (source[i + 1] === '/' || source[i + 1] === '*') return null;
	let k = i + 1;
	let classe = false; // à l'intérieur d'un `[…]`, un `/` n'est pas une fermeture
	while (k < source.length) {
		const c = source[k];
		if (c === '\n') return null;
		if (c === '\\') {
			k += 2;
			continue;
		}
		// ⚠️ Défense en profondeur : rencontrer un commentaire en cherchant la fermeture
		// prouve que ce `/` n'ouvrait pas une regex. Le garde de la ligne d'ouverture ne
		// couvre que le cas `//` en TÊTE ; celui-ci vaut pour tous les caractères du jeu,
		// pas seulement le `}` qui a révélé le défaut.
		if (!classe && (source.startsWith('//', k) || source.startsWith('/*', k))) return null;
		if (c === '[') classe = true;
		else if (c === ']') classe = false;
		else if (c === '/' && !classe) {
			k += 1;
			while (k < source.length && /[a-z]/.test(source[k])) k += 1; // drapeaux
			return k;
		}
		k += 1;
	}
	return null;
}

/**
 * Rend une copie de `source` où le CONTENU des commentaires est remplacé par des
 * espaces — mêmes longueurs, mêmes numéros de ligne, donc mêmes positions.
 *
 * ⚠️ **Pourquoi c'est nécessaire.** Trois docstrings du dépôt écrivent `i18nMsg(clé,
 * repli)` à titre d'exemple. Sans masquage, elles étaient comptées comme des sites
 * d'appel : trois des trente-six « sites non résolus » étaient de la prose. Deux
 * conséquences, la seconde étant la vraie : éditer un commentaire faisait rougir le
 * gate pour une raison sans rapport, et surtout **une régression pouvait se compenser
 * en silence** — un commentaire qui disparaît pendant qu'un vrai site apparaît laisse
 * le compte inchangé. (Revue de code 23-1a, passe 1.)
 *
 * **Trois formes couvertes** : `//`, `/* … *\/` et — depuis la story 23-3b — les
 * commentaires de balisage `<!-- … -->` des `.svelte`.
 *
 * @param {string} source
 * @returns {string}
 */
export function masquerCommentaires(source) {
	const out = source.split('');
	let i = 0;
	while (i < source.length) {
		const c = source[i];
		if (QUOTES.includes(c) && ouvreUnLitteral(source, i)) {
			const lu = readLiteral(source, i);
			// Un littéral est recopié tel quel : un `//` dans une URL n'est pas un commentaire.
			i = lu ? lu.end : i + 1;
			continue;
		}
		// ⚠️ Une regex AVANT le test des commentaires : `href.replace(/^\//, '')` contient
		// deux `/` consécutifs — un `\/` échappé suivi du délimiteur fermant. Pris pour
		// un `//`, il faisait blanchir la fin de la ligne, et tout appel qui l'y suivait
		// disparaissait. Le cas est RÉEL : `routes/(app)/+layout.svelte:241`, le fichier
		// même qui porte les 22 clés du menu. (Revue de code 23-1a, passe 2.)
		if (c === '/' && estDebutDeRegex(source, i)) {
			const fin = finDeRegex(source, i);
			if (fin !== null) {
				i = fin;
				continue;
			}
		}
		if (c === '/' && source[i + 1] === '/') {
			while (i < source.length && source[i] !== '\n') out[i++] = ' ';
			continue;
		}
		if (c === '/' && source[i + 1] === '*') {
			const fin = source.indexOf('*/', i + 2);
			const stop = fin === -1 ? source.length : fin + 2;
			while (i < stop) {
				if (source[i] !== '\n') out[i] = ' ';
				i += 1;
			}
			continue;
		}
		// ⚠️ **Commentaires de BALISAGE** — `.svelte` uniquement en pratique (story 23-3b).
		// Sans eux, la prose française du balisage passe pour des littéraux, et c'est
		// exactement dans les `.svelte` que vivent les sites que la garde doit lire.
		// Mesuré sur `main` au 2026-08-21 : **235 blocs**, dont **49** portent au moins un
		// littéral lisible (majorant — le critère `ouvreUnLitteral` en écarte une part).
		// ⚠️ **Le dépôt en porte 237 à HEAD** : cette story en a ajouté elle-même, par les
		// commentaires qui expliquent ses correctifs. Un nombre absolu inscrit dans le code se
		// périme donc par ses propres commits — d'où la mention de la BORNE de mesure. Ce qui
		// compte n'est pas le nombre mais le geste : `grep -rho "<!--" src --include=*.svelte`.
		// ⚠️ **L'extension ne peut ni retirer un site compté ni toucher un corps candidat** :
		// **zéro** appel `i18nMsg` vit dans un commentaire de balisage, et **zéro**
		// commentaire de balisage vit dans un bloc `<script>` — les deux vérifiés au dépôt.
		// ⚠️ **L'ordre compte** : le test des littéraux est AU-DESSUS, donc un `'<!-- … -->'`
		// écrit dans une chaîne JS reste une chaîne — comme un `//` dans une URL.
		if (c === '<' && source.startsWith('<!--', i)) {
			const fin = source.indexOf('-->', i + 4);
			const stop = fin === -1 ? source.length : fin + 3;
			while (i < stop) {
				if (source[i] !== '\n') out[i] = ' ';
				i += 1;
			}
			continue;
		}
		i += 1;
	}
	return out.join('');
}

/**
 * Avance depuis `i` en sautant blancs et commentaires, et rend la position du
 * prochain caractère significatif.
 *
 * @param {string} source
 * @param {number} i
 * @returns {number}
 */
function skipTrivia(source, i) {
	let j = i;
	while (j < source.length) {
		const c = source[j];
		if (c === ' ' || c === '\t' || c === '\r' || c === '\n') {
			j += 1;
		} else if (c === '/' && source[j + 1] === '/') {
			const nl = source.indexOf('\n', j);
			j = nl === -1 ? source.length : nl + 1;
		} else if (c === '/' && source[j + 1] === '*') {
			const close = source.indexOf('*/', j);
			j = close === -1 ? source.length : close + 2;
		} else {
			return j;
		}
	}
	return j;
}

/**
 * Recense les **relais locaux** d'un fichier — les fonctions qui transmettent leurs
 * arguments à `i18nMsg` :
 *
 *     function msg(key: string, fallback: string): string { return i18nMsg(key, fallback); }
 *     function msg(key: string, fallback: string, args?: …): string { return i18nMsg(key, fallback, args); }
 *
 * ⚠️ **Le littéral vit alors au site `msg(`, jamais au site `i18nMsg(`.** Ignorer les
 * relais coûtait 29 clés manquantes et un dossier entier (story 23-1a, D4-bis) —
 * et la forme à TROIS paramètres est réelle (`routes/(app)/+page.svelte`).
 *
 * @param {string} source
 * @returns {string[]} les noms des fonctions-relais déclarées dans ce fichier
 */
export function findRelays(source) {
	/** @type {string[]} */
	const noms = [];
	// Deux formes de déclaration — `function nom(…)` et `const nom = (…) =>`.
	const decl =
		/(?:function\s+(\w+)\s*\(|(?:const|let)\s+(\w+)\s*=\s*\()\s*(\w+)\s*:\s*string\s*,\s*(\w+)\s*:\s*string[^)]*\)[^{]*\{/g;
	let m;
	while ((m = decl.exec(source)) !== null) {
		const [, nomFn, nomConst, pKey, pFallback] = m;
		const nom = nomFn ?? nomConst;
		// ⚠️ Le corps est délimité par APPARIEMENT D'ACCOLADES, jamais par un nombre de
		// caractères. Une borne `{0,400}` rendait invisible tout relais au corps plus
		// long — et l'assertion de cardinalité ne l'aurait PAS rattrapé : elle compte les
		// relais DÉTECTÉS, donc un relais jamais vu laisse le compte inchangé.
		// (Revue de code 23-1a, passe 2.)
		const corps = corpsDeFonction(source, m.index + m[0].length - 1);
		if (corps === null) continue;
		const transmet = new RegExp(
			`return\\s+i18nMsg\\(\\s*${pKey}\\s*,\\s*${pFallback}\\s*[,)]`
		);
		if (transmet.test(corps) && !noms.includes(nom)) noms.push(nom);
	}
	return noms;
}

/**
 * Rend le corps de la fonction dont l'accolade ouvrante est en `ouvrante`, en
 * appariant les accolades et en sautant les littéraux — sans quoi une accolade dans
 * une chaîne fausserait le compte.
 *
 * ⚠️ **Exportée pour la garde des libellés en dur** (`i18n-libelle-en-dur.test.ts`,
 * story 23-3b) : elle a besoin du même appariement, et le doc-comment ci-dessus raconte
 * trois passes de durcissement — la réécrire, c'est repayer trois régressions.
 * ⚠️ **Elle ne suffit PAS à elle seule** : elle prend en entrée la position d'une
 * accolade **déjà connue**. Localiser la déclaration (nom, puis la vraie accolade de
 * corps — celle qui suit la FERMETURE de la signature, et non le `{` d'un type inline)
 * reste à la charge de l'appelant.
 *
 * @param {string} source
 * @param {number} ouvrante
 * @returns {string | null}
 */
export function corpsDeFonction(source, ouvrante) {
	let profondeur = 0;
	let j = ouvrante;
	while (j < source.length) {
		const c = source[j];
		// ⚠️ **Troisième consommateur de `ouvreUnLitteral`, et il lui manquait le saut des
		// regex** que `readLiteral` et `masquerCommentaires` faisaient déjà. Une accolade
		// dans une regex (`if (/[{]/.test(key))`) déséquilibrait l'appariement, le corps
		// rendait `null`, et **le relais était jeté avec toutes ses clés** — en silence,
		// puisque l'assertion de cardinalité compte les relais DÉTECTÉS. Régression
		// introduite par le correctif de la passe 2. (Revue de code 23-1a, passe 3.)
		if (c === '/' && estDebutDeRegex(source, j)) {
			const fin = finDeRegex(source, j);
			if (fin !== null) {
				j = fin;
				continue;
			}
		}
		if (QUOTES.includes(c) && ouvreUnLitteral(source, j)) {
			const lu = readLiteral(source, j);
			j = lu ? lu.end : j + 1;
			continue;
		}
		if (c === '{') profondeur += 1;
		else if (c === '}') {
			profondeur -= 1;
			if (profondeur === 0) return source.slice(ouvrante + 1, j);
		}
		j += 1;
	}
	return null;
}


/**
 * Recense tous les sites d'appel de `i18nMsg` et des relais du fichier, et lit le
 * premier argument de chacun.
 *
 * @param {string} source
 * @returns {{fn: string, line: number, arg: {kind: 'literal'|'template', value: string} | null,
 *            afterFirstArg: number | null}[]}
 *   `arg === null` ⇒ le premier argument n'est **ni littéral ni gabarit** : le site
 *   entre à l'inventaire des sites non résolus (23-1a, D4-ter / AC7-quinquies).
 *   `afterFirstArg` est la position juste après le premier argument, d'où le
 *   moissonneur de la 23-1b lit le repli.
 */
export function findCallSites(source) {
	// ⚠️ Les commentaires sont masqués AVANT la recherche : sans cela, une docstring
	// qui écrit `i18nMsg(clé, repli)` en exemple est comptée comme un site d'appel.
	source = masquerCommentaires(source);
	const relais = findRelays(source);
	const noms = ['i18nMsg', ...relais];
	/** @type {{fn: string, line: number, arg: {kind: 'literal'|'template', value: string} | null,
	 *          afterFirstArg: number | null}[]} */
	const sites = [];

	for (const nom of new Set(noms)) {
		// `(?<![\w$])` évite de confondre `msg(` avec `errorMsg(`.
		// ⚠️ Le point N'EST PLUS exclu : `obj.i18nMsg(…)` ou `ctx?.msg(…)` doivent rester
		// VISIBLES. Les exclure les faisait disparaître de l'inventaire sans même
		// l'alarme d'un `arg === null` — strictement pire que le défaut que cette garde
		// ferme. (Revue de code 23-1a, passe 1.)
		const appel = new RegExp(`(?<![\\w$])${nom}\\(`, 'g');
		let m;
		while ((m = appel.exec(source)) !== null) {
			const i = skipTrivia(source, m.index + m[0].length);
			const lu = readLiteral(source, i);
			sites.push({
				fn: nom,
				line: source.slice(0, m.index).split('\n').length,
				arg: lu ? { kind: lu.kind, value: lu.value } : null,
				afterFirstArg: lu ? lu.end : null
			});
		}
	}
	return sites.sort((a, b) => a.line - b.line);
}

/**
 * Lit le **second** argument (le repli) d'un site d'appel dont le premier a été lu.
 * Employé par le moissonneur de la 23-1b — même lecteur, donc mêmes garanties.
 *
 * @param {string} source
 * @param {number | null} afterFirstArg
 * @returns {{kind: 'literal'|'template', value: string, end: number} | null}
 */
export function readFallback(source, afterFirstArg) {
	if (afterFirstArg === null) return null;
	let j = skipTrivia(source, afterFirstArg);
	if (source[j] !== ',') return null;
	j = skipTrivia(source, j + 1);
	return readLiteral(source, j);
}
