/**
 * Garde lexicale de l'assistant d'installation — Story v017-2 (#809, AC-5, C-v017-2-9).
 *
 * Chaque `catch` de `+page.svelte` doit **lier** l'erreur et passer par
 * `errorMessageOf(` ou `reportFinalizeError(` : c'est ce qui affiche le motif du refus
 * du serveur. Un `catch {` nu jette ce motif — le défaut de #809, à sept étapes.
 *
 * Recueil, après masquage des commentaires : `/(^|[^.\w$:])catch\s*(\(|\{)/g` — le `.`
 * écarte `.catch(`, le `:` écarte `{:catch}` du balisage. Le bloc se lit par
 * `blocEquilibre` / `corpsDeFonction` (`i18n-literal-reader.js`), qui sautent chaînes,
 * gabarits et regex.
 *
 * Plancher : **onze** `catch` (S1 à S11 de la fiche) — un recueil qui passe à vide rougit.
 *
 * ## Ce qu'elle ne voit pas
 *
 * - un `catch` dans un composant extrait de l'assistant : seul `+page.svelte` est lu ;
 * - que le résultat d'`errorMessageOf(` soit affiché, ni que son argument soit la
 *   liaison du `catch` (`catch (err) { … errorMessageOf(autre) }` passe) ;
 * - `{:catch}` et `.catch(` : hors recueil (aucun dans l'assistant à l'écriture).
 */
import { describe, it, expect } from 'vitest';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { blocEquilibre, corpsDeFonction, masquerCommentaires } from '$lib/shared/i18n-literal-reader.js';

const PAGE = join('src', 'routes', 'onboarding', '+page.svelte');
const PLANCHER = 11;

type Releve = { recueillis: number; violations: string[] };

/** Contrôle une source : chaque `catch` lie l'erreur et l'affiche par la règle commune. */
function controlerCatch(source: string): Releve {
	const s = masquerCommentaires(source);
	const motif = /(^|[^.\w$:])catch\s*(\(|\{)/g;
	const violations: string[] = [];
	let recueillis = 0;
	let m: RegExpExecArray | null;
	while ((m = motif.exec(s)) !== null) {
		recueillis += 1;
		const debut = m.index + m[1].length;
		const ligne = s.slice(0, debut).split('\n').length;
		if (m[2] === '{') {
			violations.push(`ligne ${ligne} : catch sans liaison`);
			continue;
		}
		const ouvranteParam = m.index + m[0].length - 1;
		const param = blocEquilibre(s, ouvranteParam, '(', ')');
		if (param === null) {
			violations.push(`ligne ${ligne} : liaison incalculable`);
			continue;
		}
		const apres = ouvranteParam + param.length + 2;
		const accolade = s.indexOf('{', apres);
		const corps = accolade === -1 ? null : corpsDeFonction(s, accolade);
		if (corps === null) {
			violations.push(`ligne ${ligne} : bloc incalculable`);
			continue;
		}
		if (!/\berrorMessageOf\(/.test(corps) && !/\breportFinalizeError\(/.test(corps)) {
			violations.push(`ligne ${ligne} : le bloc n'appelle ni errorMessageOf( ni reportFinalizeError(`);
		}
	}
	return { recueillis, violations };
}

describe('garde — les catch de l’assistant affichent le motif (#809, AC-5)', () => {
	it(`+page.svelte : au moins ${PLANCHER} catch, tous conformes`, () => {
		const { recueillis, violations } = controlerCatch(readFileSync(PAGE, 'utf-8'));
		expect(violations).toEqual([]);
		expect(recueillis).toBeGreaterThanOrEqual(PLANCHER);
	});

	it('témoin : un catch sans liaison rougit', () => {
		const r = controlerCatch('async function f() { try { await g(); } catch { toast.error("x"); } }');
		expect(r.recueillis).toBe(1);
		expect(r.violations).toHaveLength(1);
	});

	it('témoin : un catch lié qui affiche un littéral rougit', () => {
		const r = controlerCatch("try { await g(); } catch (e) { toast.error('x') }");
		expect(r.recueillis).toBe(1);
		expect(r.violations).toHaveLength(1);
	});

	it('témoin : un catch conforme ne rend aucune violation', () => {
		const r = controlerCatch("try { await g(); } catch (e) { toast.error(errorMessageOf(e, 'r')) }");
		expect(r.recueillis).toBe(1);
		expect(r.violations).toEqual([]);
	});

	it('témoin : .catch(, {:catch} et un catch en commentaire ne sont pas recueillis', () => {
		const r = controlerCatch(
			'p.catch((e) => 1);\n{#await p}{:catch e}{/await}\n// catch { }\n/* catch (x) { } */'
		);
		expect(r.recueillis).toBe(0);
	});
});
