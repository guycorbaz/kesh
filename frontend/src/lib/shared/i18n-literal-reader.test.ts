/**
 * **Preuves du lecteur de littéral — les trois formes qui ont réellement cassé.**
 *
 * Story 23-1a, AC7-bis. Ces fixtures ne sont pas choisies pour la couverture : ce sont
 * les **trois sites du dépôt** sur lesquels une extraction par expression régulière a
 * échoué pendant les passes de revue de cette story. Les inscrire en test est le seul
 * geste qui empêche leur cinquième récidive.
 */

import { describe, it, expect } from 'vitest';
import { findCallSites, findRelays, readFallback, readLiteral } from './i18n-literal-reader.js';

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
