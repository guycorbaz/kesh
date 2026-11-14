/**
 * Garde de jetons et de contrastes des options d'`AccountAutocomplete` —
 * Story 36-1 (#1165, AC8), sur le patron de `button-destructive.test.ts` (28-1).
 *
 * Le défaut qu'elle ferme : la désignation au clavier (l'option qu'*Entrée*
 * choisit) était rendue par `bg-accent`, jeton que le `@theme` d'`app.css` ne
 * définit pas — Tailwind v4 n'engendrait aucune règle, la désignation ne se
 * voyait nulle part (validation P3, V3-1).
 *
 * Les jetons se lisent DANS LES CONSTANTES (V5-1) : une constante vidée ou
 * changée doit rougir ici, et non rendre les assertions vraies à vide.
 *
 * Ce qu'elle ne voit pas : le rendu réel (captures de la story, AC7) et une
 * classe écrite en dur dans le composant au lieu des constantes (lecture de
 * revue ; AC1 s du composant contrôle les classes rendues).
 */
// @vitest-environment node
import { describe, it, expect } from 'vitest';
import { readFileSync } from 'node:fs';
import { CLASSE_DESIGNEE, CLASSE_SURVOL } from './account-autocomplete-classes';
import { contrastRatio } from '../../../../tests/e2e/helpers/contrast';
import {
	blocTheme,
	declareDans,
	jetonDans,
	jetonsDesClasses,
	motEntier,
} from '../../../../tests/e2e/helpers/theme';

const THEME = blocTheme(readFileSync('src/app.css', 'utf8'));

/** Valeur lue du jeton ; lève si absent (un ratio ne se calcule pas sur rien). */
function lire(nom: string): string {
	const v = jetonDans(nom, THEME);
	if (!v) throw new Error(`--color-${nom} absent du @theme (ou non #rrggbb)`);
	return v;
}

/** Le seul jeton de l'utilité `utilite` dans `classes` ; lève s'il n'y en a pas un et un seul. */
function seulJeton(classes: string, utilite: string): string {
	const trouves = jetonsDesClasses(classes).filter((j) => j.utilite === utilite);
	if (trouves.length !== 1) {
		throw new Error(`« ${classes} » : ${trouves.length} jeton(s) ${utilite}-*, attendu 1`);
	}
	return trouves[0].jeton;
}

/** Fond de la liste d'options (`AccountAutocomplete.svelte`, `<ul … bg-popover>`). */
const FOND_LISTE = 'popover';

describe('AC8 — les constantes nomment les classes attendues (non vides)', () => {
	it('CLASSE_DESIGNEE : fond primary plein, texte primary-foreground', () => {
		expect(motEntier(CLASSE_DESIGNEE, 'bg-primary'), CLASSE_DESIGNEE).toBe(true);
		expect(motEntier(CLASSE_DESIGNEE, 'text-primary-foreground'), CLASSE_DESIGNEE).toBe(true);
	});

	it('CLASSE_SURVOL : hover:bg-border, et rien qui désigne', () => {
		expect(motEntier(CLASSE_SURVOL, 'hover:bg-border'), CLASSE_SURVOL).toBe(true);
		expect(motEntier(CLASSE_SURVOL, 'bg-primary'), CLASSE_SURVOL).toBe(false);
	});
});

describe('AC8 — tout jeton employé est défini dans le @theme', () => {
	it('les jetons lus dans les deux constantes, et le fond de la liste', () => {
		const noms = [
			...jetonsDesClasses(CLASSE_DESIGNEE),
			...jetonsDesClasses(CLASSE_SURVOL),
		].map((j) => j.jeton);
		expect(noms).toEqual(expect.arrayContaining(['primary', 'primary-foreground', 'border']));
		const absents = [...noms, FOND_LISTE].filter((n) => !declareDans(n, THEME));
		expect(absents, `jetons non définis dans @theme : ${absents.join(', ')}`).toEqual([]);
	});

	it('ni accent ni muted (absents du thème : rendu nul)', () => {
		for (const c of [CLASSE_DESIGNEE, CLASSE_SURVOL]) {
			const noms = jetonsDesClasses(c).map((j) => j.jeton);
			expect(noms).not.toContain('accent');
			expect(noms).not.toContain('muted');
		}
	});
});

describe('AC8 — contrastes, sur les jetons extraits des constantes', () => {
	const fondDesigne = () => lire(seulJeton(CLASSE_DESIGNEE, 'bg'));
	const texteDesigne = () => lire(seulJeton(CLASSE_DESIGNEE, 'text'));
	const fondSurvol = () => lire(seulJeton(CLASSE_SURVOL, 'bg'));

	it('fond désigné contre le fond de la liste ≥ 3 (indicateur d’état, 1.4.11)', () => {
		expect(contrastRatio(fondDesigne(), lire(FOND_LISTE))).toBeGreaterThanOrEqual(3);
	});

	it('texte désigné sur son fond ≥ 4,5 (1.4.3)', () => {
		expect(contrastRatio(texteDesigne(), fondDesigne())).toBeGreaterThanOrEqual(4.5);
	});

	it('fond désigné contre le fond de survol ≥ 3 (les deux états se distinguent)', () => {
		expect(contrastRatio(fondDesigne(), fondSurvol())).toBeGreaterThanOrEqual(3);
	});
});
