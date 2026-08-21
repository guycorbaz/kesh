/**
 * **Preuves du lecteur de littéral — les trois formes qui ont réellement cassé.**
 *
 * Story 23-1a, AC7-bis. Ces fixtures ne sont pas choisies pour la couverture : ce sont
 * les **trois sites du dépôt** sur lesquels une extraction par expression régulière a
 * échoué pendant les passes de revue de cette story. Les inscrire en test est le seul
 * geste qui empêche leur cinquième récidive.
 */

import { describe, it, expect } from 'vitest';
import {
	corpsDeFonction,
	findCallSites,
	findRelays,
	masquerCommentaires,
	readFallback,
	readLiteral
} from './i18n-literal-reader.js';

describe("l'extracteur résiste aux trois formes qui ont cassé", () => {
	it("(a) gabarit dont l'interpolation contient des apostrophes", () => {
		// BankImportUpload.svelte:547 — une classe `[^'"`]*` s'arrête sur l'apostrophe
		// de `'-'` et ne reconnaît jamais l'appel. Ce motif entier avait été MANQUÉ.
		const src = "{i18nMsg(`bank-import-info-${info.replace(/_/g, '-')}`, info)}";
		const sites = findCallSites(src);
		expect(sites).toHaveLength(1);
		expect(sites[0].arg?.kind).toBe('template');
		expect(sites[0].arg?.value).toBe("bank-import-info-${info.replace(/_/g, '-')}");
	});

	it('(b) appel réparti sur plusieurs lignes, en balisage Svelte', () => {
		// supplier-invoices/import/+page.svelte:85-88 — invisible à tout balayage
		// ligne à ligne.
		const src = [
			'notifyWarning(',
			'\ti18nMsg(',
			"\t\t'imported-supplier-invoices-reload-failed',",
			"\t\t'La liste n’a pas pu être rechargée — actualisez la page.',",
			'\t),',
			');'
		].join('\n');
		const sites = findCallSites(src);
		expect(sites[0].arg?.value).toBe('imported-supplier-invoices-reload-failed');
		expect(readFallback(src, sites[0].afterFirstArg)?.value).toBe(
			'La liste n’a pas pu être rechargée — actualisez la page.'
		);
	});

	it('(c) appel multi-ligne en TypeScript pur, dans un ternaire', () => {
		// notify.ts:103-110 — forme sans balisage Svelte. Deux des cinq clés `.ts` en
		// viennent : les perdre ramènerait le compte à 3, sous la borne mesurée de 5.
		const src = [
			'const message = isClosed',
			'\t? i18nMsg(',
			"\t\t\t'error-fiscal-year-closed-for-date',",
			'\t\t\t"L\'exercice qui couvre cette date est clôturé."',
			'\t\t)',
			'\t: i18nMsg(',
			"\t\t\t'error-fiscal-year-missing',",
			'\t\t\t"Créez d\'abord un exercice comptable"',
			'\t\t);'
		].join('\n');
		const sites = findCallSites(src);
		expect(sites.map((s) => s.arg?.value)).toEqual([
			'error-fiscal-year-closed-for-date',
			'error-fiscal-year-missing'
		]);
	});

	it("(d) le repli entre guillemets DOUBLES contenant une apostrophe est lu ENTIER", () => {
		// payment-batches/[id]/+page.svelte:88 — le septième conflit de repli, manqué
		// deux fois, y compris par le script écrit pour le vérifier.
		const src = `i18nMsg('payment-batches-col-date', "Date d'exécution")`;
		const sites = findCallSites(src);
		expect(readFallback(src, sites[0].afterFirstArg)?.value).toBe("Date d'exécution");
	});

	it('(e) un premier argument non littéral rend `arg === null` — il entre à l’inventaire', () => {
		// routes/(app)/+layout.svelte:154 — la clé vit dans une table de données. Six
		// clés de dette, dont quatre entrées du menu principal, étaient dans ce trou.
		const sites = findCallSites('return i18nMsg(item.i18nKey, item.fallback);');
		expect(sites).toHaveLength(1);
		expect(sites[0].arg).toBeNull();
	});
});

describe('le recensement des relais', () => {
	it('reconnaît la forme à deux paramètres', () => {
		const src = 'function msg(key: string, fallback: string): string { return i18nMsg(key, fallback); }';
		expect(findRelays(src)).toEqual(['msg']);
	});

	it('reconnaît la forme à TROIS paramètres', () => {
		// routes/(app)/+page.svelte:11 — un motif à deux paramètres n'en recense que 6
		// sur 7, et le réflexe le moins cher serait d'ajuster le nombre attendu.
		const src = [
			'function msg(key: string, fallback: string, args?: Record<string, string | number>): string {',
			'\treturn i18nMsg(key, fallback, args);',
			'}'
		].join('\n');
		expect(findRelays(src)).toEqual(['msg']);
	});

	it('REJETTE une fonction qui construit une clé au lieu de la transmettre', () => {
		// `journalLabel` n'est pas un relais : la clé y est un gabarit, déjà couvert par
		// MOTIFS_DYNAMIQUES. La confondre avec un relais gonflerait la cardinalité.
		const src =
			'function journalLabel(j: string): string { return i18nMsg(`journal-${j.toLowerCase()}`, j); }';
		expect(findRelays(src)).toEqual([]);
	});

	it('les littéraux passés à un relais sont collectés comme les autres', () => {
		const src = [
			'function msg(key: string, fallback: string): string { return i18nMsg(key, fallback); }',
			"const t = msg('onboarding-field-name-hint', 'Nom de votre organisation');"
		].join('\n');
		const cles = findCallSites(src)
			.filter((s) => s.arg?.kind === 'literal')
			.map((s) => s.arg?.value);
		expect(cles).toContain('onboarding-field-name-hint');
	});
});

describe('défauts trouvés en passe 2 de revue', () => {
	it('une regex contenant `//` ne fait pas passer la suite de la ligne pour un commentaire', () => {
		// ⚠️ Cas RÉEL : `routes/(app)/+layout.svelte:241` écrit `href.replace(/^\\//, '')`.
		// Le `\\/` échappé suivi du délimiteur fermant forme deux `/` consécutifs — pris
		// pour un `//`, ils faisaient blanchir la fin de la ligne, et tout appel qui l'y
		// suivait disparaissait. C'est le fichier qui porte les 22 clés du menu.
		const src = "const s = x.replace(/^\\//g, '') + i18nMsg('cle-apres-regex', 'v');";
		expect(findCallSites(src).map((s) => s.arg?.value)).toEqual(['cle-apres-regex']);
	});

	it('une chaîne collée à un mot-clé est bien lue', () => {
		// La règle « une quote précédée d'une lettre n'ouvre pas une chaîne » protège des
		// apostrophes de prose, mais `return'x'` et `typeof'x'` sont du JavaScript valide.
		const src = "function f() { return'x'; }\nconst a = i18nMsg('cle-ok', 'v');";
		expect(findCallSites(src).map((s) => s.arg?.value)).toContain('cle-ok');
	});

	it('un relais dont le corps dépasse quelques centaines de caractères reste vu', () => {
		// ⚠️ Une borne de caractères rendait invisible tout relais au corps long — et
		// l'assertion de cardinalité ne l'aurait PAS rattrapé : elle compte les relais
		// DÉTECTÉS, donc un relais jamais vu laisse le compte inchangé.
		const corps = "\tconst x = 'a'.repeat(350);\n".repeat(20);
		const src = `function msg(key: string, fallback: string): string {\n${corps}\treturn i18nMsg(key, fallback);\n}`;
		expect(findRelays(src)).toEqual(['msg']);
	});

	it('une accolade dans une chaîne du corps ne fausse pas la détection du relais', () => {
		const src =
			"function msg(key: string, fallback: string): string { const c = '}'; return i18nMsg(key, fallback); }";
		expect(findRelays(src)).toEqual(['msg']);
	});
});

describe('régressions trouvées en passe 3 de revue', () => {
	it('un relais dont le corps contient une regex à accolade reste vu', () => {
		// ⚠️ **Régression introduite par le correctif de la passe 2** : `corpsDeFonction`
		// savait sauter les chaînes, pas les regex. Une accolade dans `/[{]/` déséquilibrait
		// l'appariement, le corps rendait `null`, et le relais était jeté AVEC TOUTES SES
		// CLÉS — en silence, l'assertion de cardinalité ne comptant que les relais détectés.
		const src =
			'function msg(key: string, fallback: string): string { if (/[{]/.test(key)) return key; return i18nMsg(key, fallback); }';
		expect(findRelays(src)).toEqual(['msg']);
	});

	it('un gabarit étiqueté dans le corps ne fait pas perdre le relais', () => {
		// Un backtick ouvre TOUJOURS un littéral en JavaScript, étiqueté ou non.
		const src =
			'function msg(key: string, fallback: string): string { const r = String.raw`{`; return i18nMsg(key, fallback); }';
		expect(findRelays(src)).toEqual(['msg']);
	});

	it('un backtick de prose dans un commentaire ne lance pas un gabarit fugueur', () => {
		// ⚠️ Ligne RÉELLE de `MarkPaidDialog.svelte`. Le backtick FERMANT, précédé d'une
		// parenthèse, était accepté comme ouvrant : le gabarit avalait **58 lignes**, et
		// les commentaires de la zone cessaient d'être masqués. Cinq fichiers `.svelte`
		// du dépôt étaient dans ce cas, dix-neuf commentaires au total.
		const src = [
			'<!--',
			'  - Émet `onConfirm({ paidAt })` ;',
			'-->',
			'// exemple de doc : i18nMsg(\'cle-fantome\', \'Repli\')',
			"const vrai = i18nMsg('cle-reelle', 'v');"
		].join('\n');
		expect(findCallSites(src).map((s) => s.arg?.value)).toEqual(['cle-reelle']);
	});

	it('une regex après un mot-clé ou une flèche est reconnue comme telle', () => {
		// `estDebutDeRegex` ne connaissait que des opérateurs ASCII : ni `=>`, ni les
		// mots-clés. `return /['"]/.test(s)` était lu comme une division, la quote de la
		// classe ouvrait une fausse chaîne, et le commentaire suivant échappait au masquage.
		const apresMotCle = [
			'function f(s) { return /[\'"]/.test(s); }',
			'// exemple : i18nMsg(\'fantome-a\', \'x\')',
			"const a = i18nMsg('vraie-a', 'v');"
		].join('\n');
		expect(findCallSites(apresMotCle).map((s) => s.arg?.value)).toEqual(['vraie-a']);

		const apresFleche = [
			'const p = (s) => /^\\//.test(s);',
			'// exemple : i18nMsg(\'fantome-b\', \'x\')',
			"const b = i18nMsg('vraie-b', 'v');"
		].join('\n');
		expect(findCallSites(apresFleche).map((s) => s.arg?.value)).toEqual(['vraie-b']);
	});

	it("un chemin d'import n'est pas refusé comme littéral", () => {
		// 1111 littéraux réels du dépôt étaient refusés, les `from '…'` en tête.
		const src = "import { x } from '@sveltejs/kit';\nconst a = i18nMsg('vraie-c', 'v');";
		expect(findCallSites(src).map((s) => s.arg?.value)).toEqual(['vraie-c']);
	});
});

describe('régression trouvée en passe 4 de revue', () => {
	it("une division précédée d'une accolade ne masque pas le commentaire qui la suit", () => {
		// ⚠️ **Régression introduite par le correctif de la passe 3**, qui avait ajouté `}`
		// au jeu de caractères annonçant une regex — sans justification ni preuve. Or `}`
		// est ambigu : `{…} / 2` est une division licite. Prise pour une regex, elle
		// cherchait un `/` fermant et le trouvait dans le `//` du commentaire suivant, qui
		// échappait alors au masquage — clé fantôme à la clé.
		const src = "const r = x} / 2; // exemple : i18nMsg('cle-fantome', 'v')";
		expect(findCallSites(src).map((s) => s.arg?.value)).toEqual([]);
	});

	it('une regex non close avant un commentaire est rejetée en cours de balayage', () => {
		// Défense en profondeur : rencontrer un commentaire en cherchant la fermeture
		// prouve que ce `/` n'ouvrait pas une regex. Vaut pour tous les caractères du jeu,
		// pas seulement le `}` qui a révélé le défaut.
		const src = "const a = (b / c); // i18nMsg('cle-fantome-2', 'v')\nconst d = i18nMsg('vraie', 'v');";
		expect(findCallSites(src).map((s) => s.arg?.value)).toEqual(['vraie']);
	});
});

describe('cas limites du lecteur', () => {
	it('lit un littéral échappé sans se laisser fermer trop tôt', () => {
		expect(readLiteral(String.raw`'Saisie d\'écriture'`, 0)?.value).toBe("Saisie d'écriture");
	});

	it('rend null sur un littéral non clos plutôt que de rendre du faux', () => {
		expect(readLiteral("'jamais fermé", 0)).toBeNull();
	});

	it('ne confond pas `msg(` avec un identifiant qui se termine par msg', () => {
		const src = [
			'function msg(key: string, fallback: string): string { return i18nMsg(key, fallback); }',
			"const a = errorMsg('pas-une-cle', 'x');"
		].join('\n');
		expect(findCallSites(src).map((s) => s.arg?.value)).not.toContain('pas-une-cle');
	});

	it('VOIT un appel membre `obj.msg(…)` au lieu de le rendre invisible', () => {
		// ⚠️ Comportement changé en revue de code : le lookbehind excluait le point, si
		// bien qu'un appel `obj.i18nMsg(…)` ou `ctx?.msg(…)` disparaissait de l'inventaire
		// SANS MÊME l'alarme d'un `arg === null` — strictement pire que le défaut que
		// cette garde ferme. Un tel appel doit être vu, quitte à être classé non résolu.
		const src = [
			'function msg(key: string, fallback: string): string { return i18nMsg(key, fallback); }',
			"const b = obj.msg('une-cle-via-membre', 'y');"
		].join('\n');
		expect(findCallSites(src).map((s) => s.arg?.value)).toContain('une-cle-via-membre');
	});

	it('masque les commentaires : une docstring qui cite i18nMsg n’est pas un site', () => {
		// Trois docstrings du dépôt écrivent `i18nMsg(clé, repli)` en exemple. Comptées
		// comme sites, elles polluaient l'inventaire — et permettaient à une régression
		// de se compenser en silence (un commentaire qui part, un vrai site qui arrive).
		const src = [
			'// le caller applique `i18nMsg(key, fallback)`.',
			'/** résolu via `i18nMsg(label, fallback)`. */',
			"const vrai = i18nMsg('vraie-cle', 'v');"
		].join('\n');
		const sites = findCallSites(src);
		expect(sites).toHaveLength(1);
		expect(sites[0].arg?.value).toBe('vraie-cle');
	});

	it("une apostrophe de prose n'ouvre pas un faux littéral qui avalerait les commentaires", () => {
		// ⚠️ Un `.svelte` n'est pas du JavaScript : son balisage contient de la prose, et
		// le français y met des apostrophes. Traitées comme ouvertures de chaîne, elles
		// avalaient des dizaines de lignes — dont des commentaires qui cessaient d'être
		// masqués. La première rédaction du masquage laissait ainsi passer un commentaire
		// sur trois.
		const src = [
			"<p>l'exercice comptable d'abord</p>",
			'// exemple : `i18nMsg(key, fallback)`',
			"const vrai = i18nMsg('cle-apres-prose', 'v');"
		].join('\n');
		const cles = findCallSites(src).map((s) => s.arg?.value);
		expect(cles).toEqual(['cle-apres-prose']);
	});

	it('une regex à accolade non appariée ou à quote ne casse plus la lecture', () => {
		const accolade = 'i18nMsg(`cle-${a.match(/{/)}`, f)';
		expect(findCallSites(accolade)[0].arg?.kind).toBe('template');
		const quote = `i18nMsg(\`x-\${s.replace(/['"]/g, '')}\`, 'y')`;
		expect(findCallSites(quote)[0].arg?.kind).toBe('template');
	});

	it("le TYPE vient du parsing, jamais d'une relecture de la valeur", () => {
		// `'a${b}c'` entre guillemets SIMPLES ne peut pas être un gabarit en JS, et un
		// `\${` échappé dans un vrai gabarit est volontairement inerte. Les classer comme
		// dynamiques les faisait échapper à la vérification par clé exacte.
		expect(readLiteral("'a${b}c'", 0)?.kind).toBe('literal');
		expect(readLiteral('`prix: \\${montant}`', 0)?.kind).toBe('literal');
	});
});

describe('`corpsDeFonction`, exportée pour la garde des libellés en dur (23-3b)', () => {
	// ⚠️ Elle prend en entrée la position d'une accolade **déjà connue** : localiser la
	// déclaration reste à la charge de l'appelant. Ce que ces cas verrouillent, c'est
	// l'appariement lui-même — trois passes de durcissement y sont déjà investies (relais).
	it("rend le corps délimité par appariement, pas par la première accolade fermante", () => {
		const src = 'function f() { if (x) { return 1; } return 2; }';
		expect(corpsDeFonction(src, src.indexOf('{'))).toBe(' if (x) { return 1; } return 2; ');
	});

	it('une accolade dans une chaîne du corps ne fausse pas le compte', () => {
		const src = "function f() { return '}'; }";
		expect(corpsDeFonction(src, src.indexOf('{'))).toBe(" return '}'; ");
	});

	it('une accolade dans une regex du corps ne fausse pas le compte', () => {
		const src = 'function f() { return /[{]/.test(k); }';
		expect(corpsDeFonction(src, src.indexOf('{'))).toBe(' return /[{]/.test(k); ');
	});

	it('rend null sur un corps non clos plutôt que de rendre du faux', () => {
		const src = 'function f() { return 1;';
		expect(corpsDeFonction(src, src.indexOf('{'))).toBeNull();
	});
});

describe('masquage des commentaires de BALISAGE — `<!-- -->` (23-3b)', () => {
	// ⚠️ Sans cette extension, la prose française des commentaires Svelte passe pour des
	// littéraux : 21 blocs, précisément là où vivent les sites que la garde doit lire.
	it('un commentaire de balisage est blanchi, sa prose ne compte plus', () => {
		const src = "<!-- Statut : 'Ouverte' -->\n<span>{x}</span>";
		const masque = masquerCommentaires(src);
		expect(masque).not.toContain('Ouverte');
		expect(masque).toContain('<span>{x}</span>');
	});

	it('les positions sont conservées — le masque a la même longueur et garde les sauts de ligne', () => {
		const src = '<!-- a\nb -->\nX';
		const masque = masquerCommentaires(src);
		expect(masque.length).toBe(src.length);
		expect(masque.split('\n').length).toBe(src.split('\n').length);
	});

	it("un `<!--` À L'INTÉRIEUR d'une chaîne JS n'est pas pris pour un commentaire", () => {
		const src = "const s = '<!-- x -->'; i18nMsg('cle', 'Repli');";
		expect(findCallSites(masquerCommentaires(src)).length).toBe(1);
	});

	it('un commentaire de balisage non clos blanchit jusqu’à la fin, sans boucler', () => {
		const src = '<span>{x}</span>\n<!-- fin tronquée';
		const masque = masquerCommentaires(src);
		expect(masque).toContain('<span>{x}</span>');
		expect(masque).not.toContain('tronquée');
	});
});
