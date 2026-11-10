// Story 33-3b (#918) — le calcul pur des deux graphiques de l'accueil.
//
// ⚠️ Comme dans `homepage.ts` : tout montant reste une chaîne décimale et passe par
// `big.js`, jamais de `Number` sur un montant. `Number` n'apparaît que sur le ratio
// géométrique, une fois le montant devenu une proportion.

import Big from 'big.js';
import { formatSwissAmount } from '$lib/features/journal-entries/balance';
import type { ExpensePost, MonthSummary, OtherExpenses } from './homepage-summary.api';

// --- Palette (Story 34-10, #970, C-e342-15) -----------------------------------
//
// Couleurs unies, sur décision de Guy : les trames de la 33-3b (AC14, C-e332-17) sont
// retirées. Chaque couleur est une constante hexadécimale ÉGALE à son jeton d'`app.css`
// (`token`) : les tests de contraste lisent `#rrggbb`, jsdom ne charge pas `app.css`, et
// un test tient les deux égales nom par nom.
//
// Ce que la couleur ne porte pas, et qui reste : la position des barres (revenus à
// gauche), la légende, le rang écrit au tableau, l'ordre des parts (sens horaire depuis
// midi), les valeurs au survol. Rouge et vert se confondent par la teinte en vision
// deutéranope ou protanope ; leur clarté (1,96:1 entre eux) les sépare encore (C-e342-18).

export interface ChartStyle {
	/** La valeur `#rrggbb` du jeton, écrite en clair dans `fill`. */
	color: string;
	/** Le jeton d'`app.css` dont `color` est la valeur. */
	token: string;
}

/** Revenus : vert, la valeur de `--color-success`. */
export const BAR_REVENUE: ChartStyle = { color: '#16a34a', token: '--color-chart-revenue' };
/** Dépenses : rouge, la valeur de `--color-destructive`. */
export const BAR_EXPENSE: ChartStyle = { color: '#b91c1c', token: '--color-chart-expense' };

/**
 * Les cinq postes nommés, par rang : rouge, vert, bleu, puis violet et cyan (#970).
 * Le rang, non le compte, porte la couleur : un classement qui change repeint les
 * parts, et le tableau voisin dit toujours quelle part est quel compte.
 */
export const PIE_PALETTE: readonly ChartStyle[] = [
	{ color: '#b91c1c', token: '--color-chart-1' },
	{ color: '#16a34a', token: '--color-chart-2' },
	{ color: '#1e40af', token: '--color-chart-3' },
	{ color: '#7c3aed', token: '--color-chart-4' },
	{ color: '#0891b2', token: '--color-chart-5' },
];

/** « Autres » : gris, la valeur de `--color-text-muted`. */
export const OTHERS_STYLE: ChartStyle = { color: '#64748b', token: '--color-text-muted' };

// --- Barres mensuelles ----------------------------------------------------------

/** La carte passe en pleine largeur au-delà de ce nombre de mois (AC2). */
export const FULL_WIDTH_ABOVE_MONTHS = 12;

/** Largeur du `viewBox` des barres pour `n` mois (AC2). */
export function chartWidth(n: number): number {
	return n <= FULL_WIDTH_ABOVE_MONTHS ? 720 : 1200;
}

export interface ChartBox {
	width: number;
	height: number;
	marginTop: number;
	marginBottom: number;
	marginLeft: number;
	marginRight: number;
}

export interface BarRect {
	x: number;
	y: number;
	width: number;
	height: number;
}

export interface BarGroup {
	month: MonthSummary;
	x: number;
	width: number;
	/** `null` si le montant est nul : aucune barre (AC3). */
	revenue: BarRect | null;
	expense: BarRect | null;
}

export interface AxisTick {
	value: Big;
	label: string;
	y: number;
}

export interface MonthlyBars {
	groups: BarGroup[];
	ticks: AxisTick[];
	zeroY: number;
	plotTop: number;
	plotBottom: number;
	/** Tous les montants sont nuls. */
	isEmpty: boolean;
}

/** Le plus petit pas de graduation : le centime, qu'une étiquette sait écrire. */
const MIN_TICK_STEP = new Big('0.01');

/**
 * Le pas rond (1, 2, 2,5 ou 5 × 10^k) qui donne environ quatre intervalles. Pas de
 * 2,5 × 0,01 : 0,025 n'est plus un nombre de centimes, et l'étiquette l'arrondirait.
 */
function niceStep(span: Big): Big {
	const raw = span.div(4);
	if (raw.lte(MIN_TICK_STEP)) return MIN_TICK_STEP;
	const magnitude = new Big(10).pow(raw.e);
	const mantissa = raw.div(magnitude);
	const steps = magnitude.lt('0.1') ? ['1', '2', '5', '10'] : ['1', '2', '2.5', '5', '10'];
	const nice = steps.find((m) => mantissa.lte(m)) ?? '10';
	return magnitude.times(nice);
}

/**
 * Les graduations, du bas vers le haut, zéro compris. L'échelle couvre la plus
 * grande valeur de chaque signe (C-e332-19, C-e332-29).
 */
export function axisTicks(maxPositive: Big, maxNegative: Big): Big[] {
	if (maxPositive.eq(0) && maxNegative.eq(0)) return [new Big(0)];
	const step = niceStep(maxPositive.plus(maxNegative));
	const above = maxPositive.div(step).round(0, Big.roundUp).toNumber();
	const below = maxNegative.div(step).round(0, Big.roundUp).toNumber();
	const ticks: Big[] = [];
	for (let i = -below; i <= above; i++) ticks.push(step.times(i));
	return ticks;
}

/** Une graduation : sans décimales si la valeur est ronde, à deux sinon (AC20). */
export function tickLabel(value: Big): string {
	return value.eq(value.round(0)) ? formatSwissAmount(value, 0) : formatSwissAmount(value);
}

/** Les groupes de barres d'un jeu de mois, en coordonnées du `viewBox`. */
export function monthlyBars(months: MonthSummary[], box: ChartBox): MonthlyBars {
	const plotTop = box.marginTop;
	const plotBottom = box.height - box.marginBottom;
	const plotHeight = plotBottom - plotTop;

	let maxPositive = new Big(0);
	let maxNegative = new Big(0);
	for (const m of months) {
		for (const raw of [m.revenues, m.expenses]) {
			const v = new Big(raw);
			if (v.gt(maxPositive)) maxPositive = v;
			if (v.lt(0) && v.abs().gt(maxNegative)) maxNegative = v.abs();
		}
	}
	const isEmpty = maxPositive.eq(0) && maxNegative.eq(0);

	const tickValues = axisTicks(maxPositive, maxNegative);
	const domainMin = tickValues[0];
	const domainMax = tickValues[tickValues.length - 1];
	const domain = domainMax.minus(domainMin);
	// Tout à zéro : l'axe en bas du cadre, et aucun quotient.
	const yOf = (v: Big): number =>
		domain.eq(0) ? plotBottom : plotBottom - domainRatio(v.minus(domainMin), domain) * plotHeight;
	const zeroY = yOf(new Big(0));

	const groupWidth = (box.width - box.marginLeft - box.marginRight) / Math.max(months.length, 1);
	const pad = groupWidth * 0.12;
	const barWidth = (groupWidth - 2 * pad) / 2;

	const bar = (raw: string, x: number): BarRect | null => {
		const v = new Big(raw);
		if (v.eq(0) || domain.eq(0)) return null;
		const height = domainRatio(v.abs(), domain) * plotHeight;
		return { x, width: barWidth, height, y: v.gt(0) ? zeroY - height : zeroY };
	};

	const groups = months.map((month, i) => {
		const x = box.marginLeft + i * groupWidth;
		return {
			month,
			x,
			width: groupWidth,
			revenue: bar(month.revenues, x + pad),
			expense: bar(month.expenses, x + pad + barWidth),
		};
	});

	return {
		groups,
		ticks: tickValues.map((value) => ({ value, label: tickLabel(value), y: yOf(value) })),
		zeroY,
		plotTop,
		plotBottom,
		isEmpty,
	};
}

/** `a / b` en nombre : le seul endroit où un montant devient une proportion. */
function domainRatio(a: Big, b: Big): number {
	return a.div(b).toNumber();
}

// --- Étiquettes et liens --------------------------------------------------------

/** Le mois civil d'une clé `AAAA-MM`, en date UTC : il ne glisse sous aucun fuseau. */
function monthDate(month: string): Date {
	const [y, m] = month.split('-').map(Number);
	return new Date(Date.UTC(y, m - 1, 1));
}

/** Le nom court d'un mois, dans la locale de l'installation. */
export function monthLabel(month: string, locale: string): string {
	return new Intl.DateTimeFormat(locale, { month: 'short', timeZone: 'UTC' }).format(monthDate(month));
}

/** Le mois et l'année, pour le tableau. */
export function monthYearLabel(month: string, locale: string): string {
	return new Intl.DateTimeFormat(locale, { month: 'short', year: 'numeric', timeZone: 'UTC' }).format(
		monthDate(month),
	);
}

export interface AxisLabel {
	/** Indice du mois dans `months`. */
	index: number;
	text: string;
}

/**
 * Les étiquettes de l'axe, tirées du mois rendu et non de l'indice (AC2, C-e332-23) :
 * jusqu'à 12 mois, chacun ; de 13 à 36, chaque début de trimestre civil ; au-delà,
 * chaque janvier, l'année seule. Le tableau porte tous les mois.
 */
export function axisLabels(months: MonthSummary[], locale: string): AxisLabel[] {
	const n = months.length;
	const labels: AxisLabel[] = [];
	months.forEach(({ month }, index) => {
		const [year, m] = month.split('-');
		const monthNumber = Number(m);
		if (n <= FULL_WIDTH_ABOVE_MONTHS) {
			labels.push({ index, text: monthLabel(month, locale) });
		} else if (n <= 36) {
			if ((monthNumber - 1) % 3 === 0) {
				labels.push({ index, text: `${monthLabel(month, locale)} ${year.slice(2)}` });
			}
		} else if (monthNumber === 1) {
			labels.push({ index, text: year });
		}
	});
	return labels;
}

/** Les écritures d'un mois, bornes rendues par l'API (C-e332-20). */
export function monthHref({ from, to }: { from: string; to: string }): string {
	const qs = new URLSearchParams({ dateFrom: from, dateTo: to });
	return `/journal-entries?${qs.toString()}`;
}

/**
 * Le `data-href` de la marque la plus proche d'une cible de clic, `null` hors d'une
 * marque. Écrit à la main, jamais depuis un texte saisi : identifiants et dates.
 */
export function markHref(target: EventTarget | null): string | null {
	if (!(target instanceof Element)) return null;
	return target.closest('[data-href]')?.getAttribute('data-href') ?? null;
}

/** L'écouteur unique d'un `<svg>` : suit la marque cliquée, s'il y en a une (C-e332-16). */
export function followMark(event: Event, navigate: (href: string) => unknown): void {
	const href = markHref(event.target);
	if (href) void navigate(href);
}

/**
 * Un montant affiché : deux décimales, quatre s'il porte une fraction de centime, comme
 * au règlement (Story 25-4-d2b). Un `0.0040` ne s'écrit pas `0.00` à côté de sa barre.
 */
export function chartAmount(raw: string): string {
	const value = new Big(raw);
	return formatSwissAmount(value, value.eq(value.round(2)) ? 2 : 4);
}

// --- Anneau des postes (Story 34-10, AC-3) -------------------------------------

export interface PieGeometry {
	cx: number;
	cy: number;
	/** Rayon extérieur. */
	r: number;
	/** Rayon intérieur : le trou, où s'écrit le total des dépenses. */
	innerR: number;
}

/** L'anneau de l'accueil, dans un `viewBox` de 200 : trou à 60 % du rayon extérieur. */
export const PIE_GEOMETRY: PieGeometry = { cx: 100, cy: 100, r: 96, innerR: 96 * 0.6 };

/**
 * La part qui fait le tour : un cercle au rayon médian, sans remplissage, dont le trait a
 * l'épaisseur de l'anneau. Un arc de 360° ne se dessine pas.
 */
export function fullRing({ r, innerR }: PieGeometry): { r: number; width: number } {
	return { r: (r + innerR) / 2, width: r - innerR };
}

/** Plafond et plancher de la taille du texte du centre, en unités du `viewBox`. */
export const CENTER_FONT_MAX = 22;
export const CENTER_FONT_MIN = 8;

/**
 * La taille du texte du centre (AC-4, C-e342-24) : une règle pure, jsdom n'ayant pas de
 * mise en page. Un caractère compte pour 0,6 em (chiffres d'Inter, large), et le texte
 * occupe au plus 85 % du diamètre du trou ; le résultat est borné.
 */
export function centerFontSize(text: string, holeDiameter: number): number {
	const fitting = (holeDiameter * 0.85) / (Math.max(text.length, 1) * 0.6);
	return Math.max(CENTER_FONT_MIN, Math.min(CENTER_FONT_MAX, fitting));
}

/**
 * Deux montants décimaux diffèrent-ils en VALEUR ? `"30"` et `"30.00"` sont égaux : le
 * serveur ne promet pas la même écriture pour deux champs (P7-F2).
 */
export function differentAmounts(a: string, b: string): boolean {
	return !new Big(a).eq(new Big(b));
}

export interface PieSlice {
	/** Rang, à partir de 1, comme à la légende. */
	rank: number;
	/** `null` pour « Autres », qui n'a pas de compte unique. */
	accountId: number | null;
	startAngle: number;
	endAngle: number;
	/** `null` pour une part pleine (`full`) : un arc de 360° ne se dessine pas. */
	path: string | null;
	/**
	 * La part fait le tour : ses deux extrémités, une fois écrites, se confondent. Vrai à
	 * 100 %, et dès qu'il ne reste qu'une fraction infime pour les autres : un arc dont
	 * les extrémités sont égales est omis par le SVG, et l'anneau sortirait blanc.
	 */
	full: boolean;
	style: ChartStyle;
}

function pointAt({ cx, cy }: PieGeometry, r: number, angle: number): string {
	// Sens horaire depuis midi. Arrondi à trois décimales : assez pour le dessin.
	const x = cx + r * Math.sin(angle);
	const y = cy - r * Math.cos(angle);
	return `${Number(x.toFixed(3))} ${Number(y.toFixed(3))}`;
}

/** Le style du poste de rang `index` (à partir de 0) : « Autres » au-delà de la palette. */
export function postStyle(index: number): ChartStyle {
	return PIE_PALETTE[index] ?? OTHERS_STYLE;
}

/**
 * Les parts, dans l'ordre, sens horaire depuis midi, puis « Autres ». Les angles
 * viennent de `amount / base`, jamais de `share`, qui est arrondi ; le cumul se
 * fait en `Big`, si bien que la dernière part se referme exactement.
 */
export function pieSlices(
	posts: ExpensePost[],
	others: OtherExpenses | null,
	base: string,
	geometry: PieGeometry,
): PieSlice[] {
	const baseBig = new Big(base);
	if (baseBig.lte(0)) return [];
	const items = [
		...posts.map((p, i) => ({ amount: p.amount, accountId: p.accountId, style: postStyle(i) })),
		...(others ? [{ amount: others.amount, accountId: null, style: OTHERS_STYLE }] : []),
	];
	const turn = 2 * Math.PI;
	let cumulated = new Big(0);
	return items.map((item, i) => {
		const start = cumulated.div(baseBig).toNumber() * turn;
		cumulated = cumulated.plus(item.amount);
		const end = cumulated.div(baseBig).toNumber() * turn;
		const large = end - start > Math.PI ? 1 : 0;
		const { r, innerR } = geometry;
		const from = pointAt(geometry, r, start);
		const to = pointAt(geometry, r, end);
		const full = large === 1 && from === to;
		// Secteur d'anneau : arc extérieur dans le sens horaire, segment vers l'intérieur,
		// arc intérieur à rebours, puis fermeture sur le point de départ.
		const innerFrom = pointAt(geometry, innerR, start);
		const innerTo = pointAt(geometry, innerR, end);
		return {
			rank: i + 1,
			accountId: item.accountId,
			startAngle: start,
			endAngle: end,
			full,
			path: full
				? null
				: `M${from} A${r} ${r} 0 ${large} 1 ${to} L${innerTo} A${innerR} ${innerR} 0 ${large} 0 ${innerFrom} Z`,
			style: item.style,
		};
	});
}
