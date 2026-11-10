// Story 33-3b (#918) — le calcul pur des graphiques de l'accueil.
import { readFileSync } from 'node:fs';
import { describe, it, expect, vi } from 'vitest';
import Big from 'big.js';
import { contrastRatio } from '../../../../tests/e2e/helpers/contrast';
import {
	BAR_EXPENSE,
	BAR_REVENUE,
	CENTER_FONT_MAX,
	CENTER_FONT_MIN,
	MIN_SLICE_ANGLE,
	OTHERS_STYLE,
	PIE_GEOMETRY,
	PIE_PALETTE,
	SEPARATED_MIN_ANGLE,
	SEPARATOR_WIDTH,
	axisLabels,
	axisTicks,
	centerFontSize,
	centerLines,
	chartAmount,
	chartWidth,
	differentAmounts,
	drawOrder,
	followMark,
	fullRing,
	markHref,
	monthHref,
	monthLabel,
	monthlyBars,
	pieSlices,
	postStyle,
	tickLabel,
} from './homepage-charts';
import type { MonthSummary } from './homepage-summary.api';

/** Des mois consécutifs à zéro, à partir de `start` (`AAAA-MM`). */
function months(n: number, start: string): MonthSummary[] {
	const [y0, m0] = start.split('-').map(Number);
	return Array.from({ length: n }, (_, i) => {
		const total = y0 * 12 + (m0 - 1) + i;
		const y = Math.floor(total / 12);
		const m = String((total % 12) + 1).padStart(2, '0');
		return {
			month: `${y}-${m}`,
			from: `${y}-${m}-01`,
			to: `${y}-${m}-28`,
			revenues: '0',
			expenses: '0',
		};
	});
}

function withAmounts(list: Array<[string, string]>): MonthSummary[] {
	return months(list.length, '2026-01').map((m, i) => ({
		...m,
		revenues: list[i][0],
		expenses: list[i][1],
	}));
}

const opts = { width: 720, height: 320, marginTop: 10, marginBottom: 40, marginLeft: 60, marginRight: 10 };

describe('monthlyBars', () => {
	it('positifs : les hauteurs sont proportionnelles, un groupe par mois', () => {
		const r = monthlyBars(withAmounts([['1000', '500'], ['250', '0']]), opts);
		expect(r.groups).toHaveLength(2);
		const a = r.groups[0];
		expect(a.revenue!.height / a.expense!.height).toBeCloseTo(2, 6);
		expect(a.revenue!.height / r.groups[1].revenue!.height).toBeCloseTo(4, 6);
		expect(r.groups[1].expense, 'un montant nul : aucune barre').toBeNull();
		expect(a.revenue!.y + a.revenue!.height).toBeCloseTo(r.zeroY, 6);
	});

	it('négatifs : la barre part sous l’axe du zéro, sa hauteur reste positive et proportionnelle', () => {
		const r = monthlyBars(withAmounts([['100', '-50']]), opts);
		const g = r.groups[0];
		expect(g.expense!.height).toBeGreaterThan(0);
		expect(g.expense!.y).toBeCloseTo(r.zeroY, 6);
		expect(g.revenue!.height / g.expense!.height).toBeCloseTo(2, 6);
		expect(g.expense!.y + g.expense!.height).toBeLessThanOrEqual(opts.height - opts.marginBottom + 1e-6);
		expect(g.revenue!.y).toBeGreaterThanOrEqual(opts.marginTop - 1e-6);
	});

	it('un mois seulement négatif : l’axe du zéro reste dans le cadre', () => {
		const r = monthlyBars(withAmounts([['-100', '0']]), opts);
		expect(r.zeroY).toBeCloseTo(opts.marginTop, 6);
		expect(r.groups[0].revenue!.height).toBeCloseTo(opts.height - opts.marginBottom - opts.marginTop, 6);
	});

	it('tout à zéro : aucune division par zéro, aucune barre, le cadre reste', () => {
		const r = monthlyBars(months(12, '2026-01'), opts);
		expect(r.groups).toHaveLength(12);
		expect(r.groups.every((g) => g.revenue === null && g.expense === null)).toBe(true);
		expect(Number.isFinite(r.zeroY)).toBe(true);
		expect(r.isEmpty).toBe(true);
	});

	it('un montant à quatre décimales passe par big.js, sans perte sur le grand montant', () => {
		const r = monthlyBars(withAmounts([['99999999999999.9999', '99999999999999.9998']]), opts);
		const g = r.groups[0];
		expect(g.revenue!.height).toBeGreaterThan(0);
		expect(g.expense!.height).toBeLessThanOrEqual(g.revenue!.height);
	});

	it('AC2-bis : n groupes de largeur égale, dans le cadre, quel que soit n', () => {
		for (const start of ['2026-01', '2026-03']) {
			for (const n of [1, 12, 13, 36, 37, 132]) {
				const r = monthlyBars(months(n, start), { ...opts, width: chartWidth(n) });
				expect(r.groups).toHaveLength(n);
				const w = r.groups[0].width;
				expect(r.groups.every((g) => Math.abs(g.width - w) < 1e-6)).toBe(true);
				const last = r.groups[n - 1];
				expect(last.x + last.width).toBeLessThanOrEqual(chartWidth(n) - opts.marginRight + 1e-6);
			}
		}
	});
});

describe('chartWidth', () => {
	it('720 jusqu’à 12 mois, 1200 au-delà', () => {
		expect(chartWidth(12)).toBe(720);
		expect(chartWidth(13)).toBe(1200);
	});
});

describe('axisLabels (AC2, tiré du mois rendu)', () => {
	const indices = (n: number, start: string) => axisLabels(months(n, start), 'fr-CH').map((l) => l.index);
	const textes = (n: number, start: string) => axisLabels(months(n, start), 'fr-CH').map((l) => l.text);

	it('jusqu’à 12 mois : chaque mois', () => {
		expect(indices(1, '2026-03')).toEqual([0]);
		expect(indices(12, '2026-03')).toEqual([0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);
		expect(textes(12, '2026-01')[0]).toMatch(/^janv/);
	});

	it('de 13 à 36 mois : janvier, avril, juillet, octobre, quel que soit le départ', () => {
		// Départ en janvier : 0, 3, 6…
		expect(indices(13, '2026-01')).toEqual([0, 3, 6, 9, 12]);
		// Départ en mars : avril est l'indice 1, pas un multiple de 3.
		expect(indices(13, '2026-03')).toEqual([1, 4, 7, 10]);
		expect(textes(13, '2026-03')[0]).toMatch(/^avr.* 26$/);
		expect(indices(36, '2026-03').length).toBe(12);
	});

	it('au-delà de 36 mois : chaque janvier, l’année seule', () => {
		// Départ en mars : janvier 2027 est l'indice 10.
		expect(indices(37, '2026-03')).toEqual([10, 22, 34]);
		expect(textes(37, '2026-03')).toEqual(['2027', '2028', '2029']);
		expect(indices(132, '2020-01').length).toBe(11);
		expect(textes(132, '2020-01')[0]).toBe('2020');
	});

	// AC2-bis, les douze combinaisons : le premier et le dernier mois étiquetés, et leur
	// nombre, écrits à la main depuis le calendrier.
	it.each([
		[1, '2026-01', 1, '2026-01', '2026-01'],
		[1, '2026-03', 1, '2026-03', '2026-03'],
		[12, '2026-01', 12, '2026-01', '2026-12'],
		[12, '2026-03', 12, '2026-03', '2027-02'],
		[13, '2026-01', 5, '2026-01', '2027-01'],
		[13, '2026-03', 4, '2026-04', '2027-01'],
		[36, '2026-01', 12, '2026-01', '2028-10'],
		[36, '2026-03', 12, '2026-04', '2029-01'],
		[37, '2026-01', 4, '2026-01', '2029-01'],
		[37, '2026-03', 3, '2027-01', '2029-01'],
		[132, '2020-01', 11, '2020-01', '2030-01'],
		[132, '2020-03', 11, '2021-01', '2031-01'],
	])('%i mois depuis %s : %i étiquettes, de %s à %s', (n, start, count, first, last) => {
		const list = months(n, start);
		const labels = axisLabels(list, 'fr-CH');
		expect(labels).toHaveLength(count);
		expect(list[labels[0].index].month).toBe(first);
		expect(list[labels[count - 1].index].month).toBe(last);
	});
});

describe('axisTicks et tickLabel (AC20)', () => {
	it('tickLabel : sans décimales si la valeur est ronde, à deux sinon', () => {
		expect(tickLabel(new Big('1000'))).toBe('1’000');
		expect(tickLabel(new Big('1250.5'))).toBe('1’250.50');
		expect(tickLabel(new Big('0'))).toBe('0');
		expect(tickLabel(new Big('-2500'))).toBe('-2’500');
	});

	it('des graduations rondes, et le zéro y figure toujours', () => {
		const t = axisTicks(new Big('1234'), new Big('0'));
		expect(t.some((v) => v.eq(0))).toBe(true);
		expect(t.some((v) => v.gte('1234')), 'le maximum est couvert').toBe(true);
		const n = axisTicks(new Big('1000'), new Big('300'));
		expect(n.some((v) => v.lt(0))).toBe(true);
		expect(axisTicks(new Big(0), new Big(0)).map((v) => v.toString())).toEqual(['0']);
	});

	it('un très petit montant ne donne pas une infinité de graduations', () => {
		const t = axisTicks(new Big('0.01'), new Big('0'));
		expect(t.length).toBeLessThan(10);
		expect(t.some((v) => v.gte('0.01'))).toBe(true);
	});

	it('sous dix centimes, aucun pas de 0,025 : chaque graduation est un nombre de centimes', () => {
		const t = axisTicks(new Big('0.10'), new Big('0'));
		expect(t.map((v) => v.toString())).toEqual(['0', '0.05', '0.1']);
		// Au-dessus, le pas de 2,5 reste : 25 centimes s'écrivent.
		expect(axisTicks(new Big('1'), new Big('0')).map((v) => v.toString())).toContain('0.25');
	});
});

describe('chartAmount (E-2)', () => {
	it('deux décimales, quatre seulement pour une fraction de centime', () => {
		expect(chartAmount('1000')).toBe('1’000.00');
		expect(chartAmount('1250.5000')).toBe('1’250.50');
		expect(chartAmount('0.0040')).toBe('0.0040');
		expect(chartAmount('-12.3456')).toBe('-12.3456');
	});
});

describe('pieSlices', () => {
	const post = (accountId: number, amount: string, share: string) => ({
		accountId,
		number: String(accountId),
		name: `Compte ${accountId}`,
		amount,
		share,
	});
	const geo = { cx: 100, cy: 100, r: 90, innerR: 54 };

	it('trois parts : sens horaire depuis midi, angles depuis amount / base et non depuis share', () => {
		// 1/3 : `share` vaut 33.3, ce qui donnerait 119,88° ; le vrai angle est 120°.
		const r = pieSlices(
			[post(1, '100', '33.3'), post(2, '100', '33.3'), post(3, '100', '33.3')],
			null,
			'300',
			geo,
		);
		expect(r).toHaveLength(3);
		expect(r[0].startAngle).toBeCloseTo(0, 6);
		expect(r[0].endAngle).toBeCloseTo((2 * Math.PI) / 3, 6);
		expect(r[2].endAngle).toBeCloseTo(2 * Math.PI, 6);
		// Secteur d'anneau (34-10, AC-3) : arc extérieur depuis midi, segment, arc intérieur
		// à rebours, refermé sur le point intérieur de midi.
		expect(r[0].path).toMatch(
			/^M100 10 A90 90 0 0 1 [\d.]+ [\d.]+ L[\d.]+ [\d.]+ A54 54 0 0 0 100 46 Z$/,
		);
		expect(r.map((s) => s.rank)).toEqual([1, 2, 3]);
	});

	it('un anneau, non un camembert : aucune part ne passe par le centre (34-10, AC-3)', () => {
		const r = pieSlices([post(1, '75', '75.0'), post(2, '25', '25.0')], null, '100', geo);
		for (const s of r) {
			expect(s.path).not.toContain('100 100');
			expect(s.path).toContain('A54 54 ');
		}
	});

	it('la géométrie de l’accueil : rayon intérieur à 60 % du rayon extérieur (AC-3)', () => {
		expect(PIE_GEOMETRY.innerR / PIE_GEOMETRY.r).toBeCloseTo(0.6, 6);
		expect(PIE_GEOMETRY.innerR).toBeGreaterThan(0);
	});

	it('la part pleine : un cercle au rayon médian, trait de l’épaisseur de l’anneau (AC-3)', () => {
		expect(fullRing(geo)).toEqual({ r: 72, width: 36 });
	});

	it('« Autres » vient en dernier, sans compte', () => {
		const r = pieSlices(
			[post(1, '50', '50.0')],
			{ amount: '50', share: '50.0', accountCount: 2 },
			'100',
			geo,
		);
		expect(r).toHaveLength(2);
		expect(r[1].accountId).toBeNull();
		expect(r[1].rank).toBe(2);
		expect(r[0].accountId).toBe(1);
	});

	it('une part de plus de la moitié porte le grand arc', () => {
		const r = pieSlices([post(1, '75', '75.0'), post(2, '25', '25.0')], null, '100', geo);
		expect(r[0].path).toContain('A90 90 0 1 1 ');
		expect(r[0].path).toContain('A54 54 0 1 0 ');
		expect(r[1].path).toContain('A90 90 0 0 1 ');
		expect(r[1].path).toContain('A54 54 0 0 0 ');
	});

	it('une part à 100 % : un cercle, aucun arc de 360°', () => {
		const r = pieSlices([post(1, '42.5', '100.0')], null, '42.5', geo);
		expect(r).toHaveLength(1);
		expect(r[0].full).toBe(true);
		expect(r[0].path).toBeNull();
	});

	/** Les deux extrémités d'un arc du `path` (extérieur ou intérieur), et leur corde. */
	const chord = (path: string | null, which: 'outer' | 'inner'): number => {
		const re =
			which === 'outer'
				? /^M(-?[\d.]+) (-?[\d.]+) A[\d.]+ [\d.]+ 0 [01] 1 (-?[\d.]+) (-?[\d.]+) L/
				: /L(-?[\d.]+) (-?[\d.]+) A[\d.]+ [\d.]+ 0 [01] 0 (-?[\d.]+) (-?[\d.]+) Z$/;
		const m = re.exec(path ?? '');
		expect(m, path ?? 'path nul').not.toBeNull();
		const [x1, y1, x2, y2] = m!.slice(1).map(Number);
		return Math.hypot(x2 - x1, y2 - y1);
	};

	it('une part minuscule reste VISIBLE : au moins MIN_SLICE_ANGLE, prise sur la plus grande (revue P1, B1-1)', () => {
		const r = pieSlices([post(1, '999999', '100.0'), post(2, '0.0001', '0.0')], null, '999999.0001', geo);
		expect(r).toHaveLength(2);
		// Les angles vrais restent ceux du montant ; seuls les angles dessinés changent.
		expect(r[1].endAngle).toBeCloseTo(2 * Math.PI, 6);
		expect(r[1].displayEnd - r[1].displayStart).toBeCloseTo(MIN_SLICE_ANGLE, 9);
		expect(r[0].displayEnd - r[0].displayStart).toBeCloseTo(2 * Math.PI - MIN_SLICE_ANGLE, 9);
		// Plus de cercle ni de rayon blanc dégénéré (E1-3) : deux vrais secteurs.
		expect(r[0].full).toBe(false);
		expect(r[1].full).toBe(false);
		// Visible : au moins une unité de `viewBox` à l'arc intérieur, le plus étroit.
		expect(chord(r[1].path, 'inner')).toBeGreaterThanOrEqual(1);
		expect(chord(r[1].path, 'outer')).toBeGreaterThanOrEqual(1);
	});

	it('une part étroite n’a pas de liseré, et se dessine par-dessus ses voisines (B1-1)', () => {
		const r = pieSlices(
			[post(1, '900', '99.4'), post(2, '2', '0.2')],
			{ amount: '3', share: '0.3', accountCount: 2 },
			'905',
			geo,
		);
		expect(r.map((s) => s.separated)).toEqual([true, false, false]);
		// Ordre de dessin : les parts à liseré d'abord, puis les étroites, rang conservé.
		expect(drawOrder(r).map((s) => s.rank)).toEqual([1, 2, 3]);
		const mixed = pieSlices(
			[post(1, '900', '90.0'), post(2, '2', '0.2')],
			{ amount: '98', share: '9.8', accountCount: 6 },
			'1000',
			geo,
		);
		expect(mixed.map((s) => s.separated)).toEqual([true, false, true]);
		expect(drawOrder(mixed).map((s) => s.rank)).toEqual([1, 3, 2]);
	});

	it('une part assez large pour son liseré garde au moins une unité visible de chaque côté', () => {
		// Le liseré mange SEPARATOR_WIDTH / 2 de chaque côté : à l'arc intérieur, une part
		// séparée doit garder au moins une unité une fois les deux demi-liserés retirés.
		expect(SEPARATED_MIN_ANGLE * PIE_GEOMETRY.innerR - SEPARATOR_WIDTH).toBeGreaterThanOrEqual(1);
		expect(MIN_SLICE_ANGLE * PIE_GEOMETRY.innerR).toBeGreaterThanOrEqual(1);
	});

	it('20’000.00 et 0.01 : la grande part n’est plus un cercle, la petite se voit (E1-3)', () => {
		const r = pieSlices([post(1, '20000.00', '100.0'), post(2, '0.01', '0.0')], null, '20000.01', geo);
		expect(r[0].full).toBe(false);
		expect(r[0].path).not.toBeNull();
		expect(chord(r[1].path, 'inner')).toBeGreaterThanOrEqual(1);
	});

	it('aucun arc intérieur aux extrémités confondues : le trou n’est jamais rempli (E1-2)', () => {
		// Un reste de 2e-12 du tour : sans angle minimal, les extrémités se confondraient. Toute
		// part non nulle fait au moins MIN_SLICE_ANGLE, si bien que les deux arcs de chaque
		// part ont des extrémités distinctes, l'intérieur compris.
		const r = pieSlices([post(1, '1000000', '100.0'), post(2, '0.000002', '0.0')], null, '1000000.000002', geo);
		for (const s of r) {
			expect(s.full).toBe(false);
			expect(chord(s.path, 'inner')).toBeGreaterThanOrEqual(1);
			expect(chord(s.path, 'outer')).toBeGreaterThanOrEqual(1);
		}
	});

	it('99,9 % : un vrai arc, grand, aux extrémités distinctes', () => {
		const r = pieSlices([post(1, '999', '99.9'), post(2, '1', '0.1')], null, '1000', geo);
		expect(r[0].full).toBe(false);
		const m = /^M([\d.]+ [\d.]+) A90 90 0 1 1 ([\d.]+ [\d.]+) L/.exec(r[0].path ?? '');
		expect(m, r[0].path ?? '').not.toBeNull();
		expect(m![1]).not.toBe(m![2]);
	});


	it('aucune part : aucun dessin, et une base nulle ne divise pas', () => {
		expect(pieSlices([], null, '0', geo)).toEqual([]);
	});
});

describe('monthHref, monthLabel, markHref', () => {
	it('monthHref porte les deux bornes rendues par l’API', () => {
		expect(monthHref({ from: '2026-03-15', to: '2026-03-31' })).toBe(
			'/journal-entries?dateFrom=2026-03-15&dateTo=2026-03-31',
		);
	});

	it('monthLabel : le 1er janvier ne glisse pas en décembre, sous aucun fuseau', () => {
		expect(monthLabel('2026-01', 'fr-CH')).toMatch(/^janv/);
		expect(monthLabel('2026-01', 'en-CH')).toMatch(/^Jan/);
		// À l'ouest de Greenwich, minuit UTC du 1er janvier est encore le 31 décembre.
		const before = process.env.TZ;
		process.env.TZ = 'America/New_York';
		try {
			expect(new Date(Date.UTC(2026, 0, 1)).getDate(), 'sonde : le fuseau a pris').toBe(31);
			expect(monthLabel('2026-01', 'fr-CH')).toMatch(/^janv/);
		} finally {
			if (before === undefined) delete process.env.TZ;
			else process.env.TZ = before;
		}
	});

	it('followMark navigue vers la marque, et ne fait rien hors d’une marque', () => {
		const navigate = vi.fn();
		const g = document.createElementNS('http://www.w3.org/2000/svg', 'g');
		g.setAttribute('data-href', '/x');
		g.dispatchEvent(new Event('click'));
		g.addEventListener('click', (e) => followMark(e, navigate));
		g.dispatchEvent(new Event('click'));
		expect(navigate).toHaveBeenCalledWith('/x');
		navigate.mockClear();
		const bare = document.createElementNS('http://www.w3.org/2000/svg', 'g');
		bare.addEventListener('click', (e) => followMark(e, navigate));
		bare.dispatchEvent(new Event('click'));
		expect(navigate).not.toHaveBeenCalled();
	});

	it('markHref lit le data-href de la marque la plus proche, et rien hors d’une marque', () => {
		const svg = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
		const g = document.createElementNS('http://www.w3.org/2000/svg', 'g');
		g.setAttribute('data-href', '/x');
		const rect = document.createElementNS('http://www.w3.org/2000/svg', 'rect');
		g.appendChild(rect);
		svg.appendChild(g);
		expect(markHref(rect)).toBe('/x');
		expect(markHref(svg)).toBeNull();
		expect(markHref(null)).toBeNull();
	});
});

describe('jetons des graphiques (34-10, AC-1)', () => {
	const css = readFileSync('src/app.css', 'utf-8');
	/** La valeur hexadécimale d'un jeton, `undefined` s'il est absent ou écrit en alias. */
	const tokenValue = (name: string): string | undefined =>
		new RegExp(`${name}:\\s*(#[0-9a-f]{6});`, 'i').exec(css)?.[1];

	it('chaque constante égale son jeton, nom par nom, en hexadécimal', () => {
		expect(BAR_REVENUE.token).toBe('--color-chart-revenue');
		expect(BAR_EXPENSE.token).toBe('--color-chart-expense');
		expect(PIE_PALETTE.map((s) => s.token)).toEqual([
			'--color-chart-1',
			'--color-chart-2',
			'--color-chart-3',
			'--color-chart-4',
			'--color-chart-5',
		]);
		expect(OTHERS_STYLE.token).toBe('--color-text-muted');
		for (const s of [BAR_REVENUE, BAR_EXPENSE, ...PIE_PALETTE, OTHERS_STYLE]) {
			expect(css, s.token).toContain(`${s.token}: ${s.color};`);
		}
	});

	it('revenus = succès, dépenses = destructif ; postes rouge, vert, bleu, violet, cyan', () => {
		expect(tokenValue('--color-chart-revenue')).toBe(tokenValue('--color-success'));
		expect(tokenValue('--color-chart-expense')).toBe(tokenValue('--color-destructive'));
		expect(BAR_REVENUE.color).toBe('#16a34a');
		expect(BAR_EXPENSE.color).toBe('#b91c1c');
		expect(PIE_PALETTE.map((s) => s.color)).toEqual(['#b91c1c', '#16a34a', '#1e40af', '#7c3aed', '#0891b2']);
		expect(OTHERS_STYLE.color).toBe('#64748b');
	});
});

describe('palette (34-10, AC-7)', () => {
	const white = '#ffffff';

	it('postStyle : la palette par rang, « Autres » au-delà', () => {
		expect(postStyle(0)).toBe(PIE_PALETTE[0]);
		expect(postStyle(5)).toBe(OTHERS_STYLE);
	});

	it('chaque couleur a un contraste d’au moins 3:1 sur blanc', () => {
		for (const s of [BAR_REVENUE, BAR_EXPENSE, OTHERS_STYLE, ...PIE_PALETTE]) {
			expect(contrastRatio(s.color, white), s.color).toBeGreaterThanOrEqual(3);
		}
	});

	it('revenus et dépenses : deux couleurs, d’au moins 1,5:1 entre elles (la clarté les sépare)', () => {
		expect(BAR_REVENUE.color).not.toBe(BAR_EXPENSE.color);
		expect(contrastRatio(BAR_REVENUE.color, BAR_EXPENSE.color)).toBeGreaterThanOrEqual(1.5);
	});

	it('six couleurs distinctes à l’anneau, « Autres » en gris comprise', () => {
		const all = [...PIE_PALETTE, OTHERS_STYLE];
		expect(PIE_PALETTE).toHaveLength(5);
		expect(new Set(all.map((s) => s.color)).size).toBe(6);
	});

	it('aucun motif dans le module : couleurs unies seulement', () => {
		const source = readFileSync('src/lib/features/homepage/homepage-charts.ts', 'utf-8');
		expect(source).not.toMatch(/ChartPattern|chartFill|patternId|pattern/i);
	});
});

describe('centerFontSize (34-10, AC-4, C-e342-24)', () => {
	const hole = 2 * PIE_GEOMETRY.innerR;

	it('décroît avec la longueur du texte, bornée par un plancher et un plafond', () => {
		const short = centerFontSize('1.00', hole);
		const long = centerFontSize('999’999’999.99', hole);
		expect(short).toBeLessThanOrEqual(CENTER_FONT_MAX);
		expect(long).toBeLessThan(short);
		expect(long).toBeGreaterThanOrEqual(CENTER_FONT_MIN);
		expect(CENTER_FONT_MIN).toBeGreaterThan(0);
	});

	it('le plus long montant plausible reste au-dessus du plancher ; au-delà, le plancher tient (revue P1, B1-2)', () => {
		// La règle (0,6 em, 85 % du trou) n'est pas recopiée ici : ce qui se vérifie sans mise
		// en page, c'est que le montant plausible le plus long n'est PAS écrasé au plancher
		// — la règle s'applique, elle n'est pas bornée — et qu'un texte hors de toute plausibilité
		// (trente caractères) reste au plancher : son débordement est assumé, non silencieux.
		expect(centerFontSize('-999’999’999.9999', hole)).toBeGreaterThan(CENTER_FONT_MIN);
		expect(centerFontSize('x'.repeat(30), hole)).toBe(CENTER_FONT_MIN);
		// Le plafond est atteint par un montant court : il n'y a pas de croissance sans fin.
		expect(centerFontSize('0.00', hole)).toBe(CENTER_FONT_MAX);
	});
});

describe('centerLines (revue P1, B1-4)', () => {
	it('l’écart entre le libellé et le montant suit la taille du libellé', () => {
		const small = centerLines(100, 8, 11);
		const big = centerLines(100, 22, 22);
		// La ligne de base du libellé laisse au moins ses jambages (≈ 0,25 em) et une unité
		// avant le haut du montant (`hanging`).
		for (const [lines, label] of [
			[small, 8],
			[big, 22],
		] as const) {
			expect(lines.amountY - lines.labelY).toBeGreaterThanOrEqual(0.25 * label + 1);
		}
		expect(big.amountY - big.labelY).toBeGreaterThan(small.amountY - small.labelY);
	});

	it('le bloc des deux lignes est centré sur le centre de l’anneau', () => {
		for (const [label, amount] of [
			[9, 12],
			[22, 22],
			[8, 8],
		]) {
			const { labelY, amountY } = centerLines(100, label, amount);
			const top = labelY - 0.75 * label;
			const bottom = amountY + 0.75 * amount;
			expect((top + bottom) / 2).toBeCloseTo(100, 6);
		}
	});
});

describe('differentAmounts (34-10, AC-4, P7-F2)', () => {
	it('compare des valeurs, non des chaînes', () => {
		expect(differentAmounts('30', '30.00')).toBe(false);
		expect(differentAmounts('-50', '30')).toBe(true);
		expect(differentAmounts('300.0000', '300')).toBe(false);
	});
});
