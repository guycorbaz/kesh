/**
 * Gardes de la variante `destructive` du `Button` partagé — Story 28-1 (#640, AC7 (1)-(3)).
 *
 * Le défaut qu'elles ferment : la variante employait le jeton `destructive`, qui
 * n'était PAS défini dans le `@theme` d'`app.css`. Tailwind v4 n'engendre une
 * classe de couleur que pour un jeton défini : `bg-destructive/10`,
 * `text-destructive`… ne produisaient aucune règle, et les 24 boutons d'action
 * destructive s'affichaient comme du texte. Rien ne le signalait.
 *
 * - (1) lit la variante SEULE (`buttonVariants.variants.variant.destructive`), non
 *   la sortie fusionnée, qui contient la base — et ses `dark:aria-invalid:…`
 *   (C-28-1-18) ; (1-bis) vérifie, sur la sortie fusionnée, que tailwind-merge a
 *   bien remplacé la bordure ET l'anneau de focus de la base (C-28-1-20 ; l'anneau
 *   depuis la revue de code P1, E2 — sa couleur rendue reste vue à l'œil seulement).
 * - (2) chaque jeton de couleur employé par la variante est défini dans `app.css`
 *   — lu dans le `@theme` sans ses commentaires (revue de code P1, E6).
 * - (3) les contrastes, recalculés depuis les valeurs LUES dans `app.css` par le
 *   helper `tests/e2e/helpers/contrast.ts` (seule méthode de calcul, C-28-1-27).
 *
 * Ce qu'elles ne voient pas : un `variant="destructive"` retiré d'un site (angle
 * mort 7) ; le rendu réel (l'E2E `destructive-buttons.spec.ts` le mesure).
 */
// @vitest-environment node
import { describe, it, expect } from 'vitest';
import { readFileSync } from 'node:fs';
import { buttonVariants } from './button.svelte';
import { blend, contrastRatio } from '../../../../../tests/e2e/helpers/contrast';

const VARIANTE: string = (buttonVariants as unknown as { variants: { variant: Record<string, string> } })
	.variants.variant.destructive;
const CSS = readFileSync('src/app.css', 'utf8');

/**
 * Bloc `@theme { … }` d'`app.css` (le premier, le seul du fichier), **commentaires
 * retirés** : un jeton écrit en commentaire (`/* --color-card: #fff; *\/`) n'est
 * pas défini, et les gardes (2) et (3) ne doivent pas le lire (revue de code P1, E6).
 */
function blocTheme(css: string): string {
	const sansCommentaires = css.replace(/\/\*[\s\S]*?\*\//g, '');
	const debut = sansCommentaires.indexOf('@theme');
	if (debut < 0) throw new Error('app.css : bloc @theme introuvable');
	const ouverture = sansCommentaires.indexOf('{', debut);
	const fermeture = sansCommentaires.indexOf('\n}', ouverture);
	return sansCommentaires.slice(ouverture + 1, fermeture);
}
const THEME = blocTheme(CSS);

/** Valeur `#rrggbb` d'un jeton `--color-<nom>` du `@theme`, ou `undefined`. */
function jeton(nom: string, theme: string = THEME): string | undefined {
	const m = new RegExp(`--color-${nom}:\\s*(#[0-9a-f]{6})\\s*;`, 'i').exec(theme);
	return m?.[1];
}

/** Le jeton `--color-<nom>` est-il déclaré dans le bloc `theme` (toute valeur) ? */
function declare(nom: string, theme: string = THEME): boolean {
	return new RegExp(`--color-${nom}\\s*:`).test(theme);
}

function motEntier(classes: string, mot: string): boolean {
	const echappe = mot.replace(/[.*+?^${}()|[\]\\/]/g, '\\$&');
	return new RegExp(`(^|\\s)${echappe}(\\s|$)`).test(classes);
}

describe('AC7 (1) — la variante destructive a un fond plein, sans dark:', () => {
	it('est lue sur la variante seule (tailwind-variants expose .variants)', () => {
		expect(typeof VARIANTE).toBe('string');
		expect(VARIANTE.length).toBeGreaterThan(0);
	});

	for (const mot of [
		'bg-destructive',
		'text-destructive-foreground',
		'hover:bg-destructive-hover',
		'focus-visible:border-destructive',
		// L'anneau de focus : sans lui, la base laisse `focus-visible:ring-ring/50`, jeton non
		// défini (#645) — l'anneau retombe sur `currentcolor` (revue de code P1, E2).
		'focus-visible:ring-destructive/40',
	]) {
		it(`contient « ${mot} » comme mot entier`, () => {
			expect(motEntier(VARIANTE, mot), VARIANTE).toBe(true);
		});
	}

	it('ne contient aucun jeton dark:', () => {
		expect(VARIANTE.split(/\s+/).filter((c) => c.startsWith('dark:'))).toEqual([]);
	});

	it('ne contient ni bg-transparent ni bg-destructive/<n>', () => {
		const classes = VARIANTE.split(/\s+/);
		expect(classes.filter((c) => /(^|:)bg-transparent$/.test(c))).toEqual([]);
		expect(classes.filter((c) => /(^|:)bg-destructive\/\d+$/.test(c))).toEqual([]);
	});

	it("(1-bis) la sortie fusionnée remplace la bordure et l'anneau de focus de la base", () => {
		const fusion = buttonVariants({ variant: 'destructive' });
		expect(motEntier(fusion, 'bg-destructive'), fusion).toBe(true);
		expect(motEntier(fusion, 'focus-visible:border-destructive'), fusion).toBe(true);
		expect(motEntier(fusion, 'focus-visible:border-ring'), fusion).toBe(false);
		expect(motEntier(fusion, 'focus-visible:ring-destructive/40'), fusion).toBe(true);
		expect(motEntier(fusion, 'focus-visible:ring-ring/50'), fusion).toBe(false);
	});
});

describe('lecture du @theme — auto-tests sur source synthétique', () => {
	it('un jeton écrit en commentaire n’est pas lu comme défini', () => {
		const theme = blocTheme('@theme {\n  /* --color-card: #ffffff; */\n  --color-x: #000000;\n}\n');
		expect(jeton('card', theme)).toBeUndefined();
		expect(declare('card', theme)).toBe(false);
		expect(jeton('x', theme)).toBe('#000000');
	});
});

describe("AC7 (2) — les jetons de la variante sont définis dans app.css", () => {
	for (const nom of ['destructive', 'destructive-foreground', 'destructive-hover']) {
		it(`--color-${nom} est défini (#rrggbb) dans @theme`, () => {
			expect(jeton(nom), `--color-${nom} absent du @theme`).toMatch(/^#[0-9a-f]{6}$/i);
		});
	}

	it('tout jeton de couleur employé par la variante est défini', () => {
		const noms = VARIANTE.split(/\s+/)
			.filter(Boolean)
			.map((c) => c.slice(c.lastIndexOf(':') + 1).replace(/\/\d+$/, ''))
			.map((c) => /^(bg|text|border|ring)-(.+)$/.exec(c)?.[2])
			.filter((n): n is string => n !== undefined);
		expect(noms.length).toBeGreaterThan(0);
		const absents = noms.filter((n) => !declare(n));
		expect(absents, `jetons non définis dans @theme : ${absents.join(', ')}`).toEqual([]);
	});
});

describe('AC7 (3) — contrastes, depuis les valeurs lues dans app.css', () => {
	const lire = (nom: string): string => {
		const v = jeton(nom);
		if (!v) throw new Error(`--color-${nom} absent du @theme`);
		return v;
	};

	it('texte du bouton sur son fond ≥ 4,5 (1.4.3)', () => {
		expect(contrastRatio(lire('destructive-foreground'), lire('destructive'))).toBeGreaterThanOrEqual(4.5);
	});

	it('texte du bouton sur son fond de survol ≥ 4,5 (1.4.3)', () => {
		expect(
			contrastRatio(lire('destructive-foreground'), lire('destructive-hover')),
		).toBeGreaterThanOrEqual(4.5);
	});

	it("texte d'erreur destructive sur blanc ≥ 4,5 (1.4.3)", () => {
		expect(contrastRatio(lire('destructive'), '#ffffff')).toBeGreaterThanOrEqual(4.5);
	});

	it("texte destructive sur bandeau d'erreur (destructive à 10 % sur blanc) ≥ 4,5", () => {
		const bandeau = blend(lire('destructive'), '#ffffff', 0.1);
		expect(contrastRatio(lire('destructive'), bandeau)).toBeGreaterThanOrEqual(4.5);
	});

	it('limite du bouton contre la page blanche ≥ 3 (1.4.11)', () => {
		expect(contrastRatio(lire('destructive'), '#ffffff')).toBeGreaterThanOrEqual(3);
	});
});
