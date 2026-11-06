// Story 33-3b (#918) — le calcul pur des graphiques de l'accueil.
import { describe, it, expect } from 'vitest';
import Big from 'big.js';
import { contrastRatio } from '../../../../tests/e2e/helpers/contrast';
import {
	BAR_EXPENSE,
	BAR_REVENUE,
	OTHERS_STYLE,
	PIE_PALETTE,
	axisLabels,
	axisTicks,
	chartWidth,
	markHref,
	monthHref,
	monthLabel,
	monthlyBars,
	pieSlices,
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
});

describe('pieSlices', () => {
	const post = (accountId: number, amount: string, share: string) => ({
		accountId,
		number: String(accountId),
		name: `Compte ${accountId}`,
		amount,
		share,
	});
	const geo = { cx: 100, cy: 100, r: 90 };

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
		expect(r[0].path).toMatch(/^M100 100 L100(\.0+)? 10(\.0+)? A90 90 0 0 1 /);
		expect(r.map((s) => s.rank)).toEqual([1, 2, 3]);
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
		expect(r[0].path).toContain(' 0 1 1 ');
		expect(r[1].path).toContain(' 0 0 1 ');
	});

	it('une part à 100 % : un cercle, aucun arc de 360°', () => {
		const r = pieSlices([post(1, '42.5', '100.0')], null, '42.5', geo);
		expect(r).toHaveLength(1);
		expect(r[0].full).toBe(true);
		expect(r[0].path).toBeNull();
	});

	it('une part minuscule reste rendue, sans valeur non finie', () => {
		const r = pieSlices([post(1, '999999', '100.0'), post(2, '0.0001', '0.0')], null, '999999.0001', geo);
		expect(r).toHaveLength(2);
		expect(r[1].path).not.toContain('NaN');
		expect(r[1].full).toBe(false);
		expect(r[1].endAngle).toBeCloseTo(2 * Math.PI, 6);
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

describe('palette (AC14)', () => {
	const white = '#ffffff';
	it('chaque remplissage a un contraste d’au moins 3:1 sur blanc', () => {
		for (const s of [BAR_REVENUE, BAR_EXPENSE, OTHERS_STYLE, ...PIE_PALETTE]) {
			expect(contrastRatio(s.color, white), s.color).toBeGreaterThanOrEqual(3);
		}
	});

	it('revenus pleins, dépenses tramées', () => {
		expect(BAR_REVENUE.pattern).toBe('solid');
		expect(BAR_EXPENSE.pattern).not.toBe('solid');
	});

	it('six motifs distincts au camembert, « Autres » en gris comprise', () => {
		const all = [...PIE_PALETTE, OTHERS_STYLE];
		expect(PIE_PALETTE).toHaveLength(5);
		expect(new Set(all.map((s) => s.pattern)).size).toBe(6);
		expect(new Set(all.map((s) => s.color)).size).toBe(6);
		expect(OTHERS_STYLE.color).toBe('#64748b');
	});
});
