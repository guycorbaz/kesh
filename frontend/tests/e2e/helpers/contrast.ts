/**
 * Contraste WCAG 2.x — fonctions pures (Story 28-1, #640, AC7 (3)).
 *
 * Seule méthode de calcul des ratios cités par la story (C-28-1-27) : un mélange
 * à alpha est pris **non arrondi** (`blend`), puis le ratio est comparé tel quel
 * — l'arrondi au centième n'intervient que pour l'écrire.
 *
 * Ce module ne sert que la garde Vitest `button-destructive.test.ts` : l'E2E ne
 * recalcule aucun contraste (C-28-1-31), elle asserte des valeurs.
 */

/** Couleur sRGB, composantes 0-255, éventuellement non entières (mélange). */
export type Rgb = readonly [number, number, number];

/**
 * Lit une couleur `#rrggbb` (six chiffres hexadécimaux, casse indifférente).
 *
 * Toute autre forme — `#fff` court, `rgb(…)`, `oklab(…)`, nom de couleur — lève
 * une erreur qui la nomme : une valeur mal lue ne doit pas devenir un ratio.
 */
export function parseHex(couleur: string): Rgb {
	const m = /^#([0-9a-f]{2})([0-9a-f]{2})([0-9a-f]{2})$/i.exec(couleur.trim());
	if (!m) {
		throw new Error(`parseHex : couleur non prise en charge « ${couleur} » (attendu #rrggbb)`);
	}
	return [parseInt(m[1], 16), parseInt(m[2], 16), parseInt(m[3], 16)];
}

function canal(c: number): number {
	const s = c / 255;
	return s <= 0.04045 ? s / 12.92 : ((s + 0.055) / 1.055) ** 2.4;
}

/** Luminance relative WCAG 2.x d'une couleur sRGB. */
export function relativeLuminance([r, g, b]: Rgb): number {
	return 0.2126 * canal(r) + 0.7152 * canal(g) + 0.0722 * canal(b);
}

/**
 * Ratio de contraste WCAG entre deux couleurs, **dans n'importe quel ordre** :
 * `(L_clair + 0,05) / (L_foncé + 0,05)`, entre 1 et 21.
 */
export function contrastRatio(a: Rgb | string, b: Rgb | string): number {
	const la = relativeLuminance(typeof a === 'string' ? parseHex(a) : a);
	const lb = relativeLuminance(typeof b === 'string' ? parseHex(b) : b);
	const [clair, fonce] = la >= lb ? [la, lb] : [lb, la];
	return (clair + 0.05) / (fonce + 0.05);
}

/**
 * Couleur `couleur` posée à l'opacité `alpha` (0-1) sur `fond` — mélange sRGB
 * **non arrondi**, comme le compositeur avant quantification (C-28-1-27).
 */
export function blend(couleur: Rgb | string, fond: Rgb | string, alpha: number): Rgb {
	const c = typeof couleur === 'string' ? parseHex(couleur) : couleur;
	const f = typeof fond === 'string' ? parseHex(fond) : fond;
	return [
		c[0] * alpha + f[0] * (1 - alpha),
		c[1] * alpha + f[1] * (1 - alpha),
		c[2] * alpha + f[2] * (1 - alpha),
	];
}
