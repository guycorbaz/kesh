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
			out.push('${');
			j += 2;
			continue;
		}

		if (depth > 0) {
			// À l'intérieur d'une interpolation : on suit les accolades, et on saute
			// les littéraux imbriqués — c'est là que `info.replace(/_/g, '-')` piège
			// une expression régulière naïve.
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
				kind: value.includes('${') ? 'template' : 'literal',
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
	const decl =
		/function\s+(\w+)\s*\(\s*(\w+)\s*:\s*string\s*,\s*(\w+)\s*:\s*string[^)]*\)[^{]*\{\s*return\s+i18nMsg\(\s*(\w+)\s*,\s*(\w+)/g;
	let m;
	while ((m = decl.exec(source)) !== null) {
		const [, nom, pKey, pFallback, aKey, aFallback] = m;
		// Le corps doit TRANSMETTRE les paramètres, pas construire une clé.
		if (aKey === pKey && aFallback === pFallback) noms.push(nom);
	}
	return noms;
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
	const relais = findRelays(source);
	const noms = ['i18nMsg', ...relais];
	/** @type {{fn: string, line: number, arg: {kind: 'literal'|'template', value: string} | null,
	 *          afterFirstArg: number | null}[]} */
	const sites = [];

	for (const nom of new Set(noms)) {
		// `(?<![\w.])` évite de confondre `msg(` avec `errorMsg(` ou `x.msg(`.
		const appel = new RegExp(`(?<![\\w.])${nom}\\(`, 'g');
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
