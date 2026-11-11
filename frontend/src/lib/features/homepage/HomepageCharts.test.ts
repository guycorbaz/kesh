// Story 33-3b (#918) — les deux graphiques de l'accueil, états, tableaux, liens.
// i18nMsg est mocké pour renvoyer le fallback interpolé ; `goto` est moqué.
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { fireEvent, render, screen } from '@testing-library/svelte';

const { goto } = vi.hoisted(() => ({ goto: vi.fn() }));
vi.mock('$app/navigation', () => ({ goto }));
vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string, args?: Record<string, string | number>) =>
		fallback.replace(/\{ \$(\w+) \}/g, (_m, k) => String(args?.[k] ?? '')),
	i18nLocale: () => 'fr-CH',
}));

import HomepageCharts from './HomepageCharts.svelte';
import { BAR_EXPENSE, BAR_REVENUE, OTHERS_STYLE, PIE_PALETTE } from './homepage-charts';
import type { HomepageSummary } from './homepage-summary.api';

function months(amounts: Array<[string, string]>) {
	return amounts.map(([revenues, expenses], i) => {
		const m = String(i + 1).padStart(2, '0');
		return { month: `2026-${m}`, from: `2026-${m}-01`, to: `2026-${m}-28`, revenues, expenses };
	});
}

const zero12: Array<[string, string]> = Array.from({ length: 12 }, () => ['0', '0']);

function summary(over: Partial<HomepageSummary> = {}): HomepageSummary {
	return {
		state: 'open',
		asOf: '2026-11-06',
		fiscalYear: { id: 3, name: 'Exercice 2026', startDate: '2026-01-01', endDate: '2026-12-31' },
		months: months(zero12),
		totalRevenues: '0',
		totalExpenses: '0',
		expensePosts: [],
		otherExpenses: null,
		expensePostsBase: '0',
		creditBalanceExpenseAccounts: 0,
		...over,
	} as HomepageSummary;
}

const post = (accountId: number, name: string, amount: string, share: string) => ({
	accountId,
	number: String(accountId),
	name,
	amount,
	share,
});

const populated = (): HomepageSummary =>
	summary({
		months: months([['1000', '300'], ...zero12.slice(1)]),
		totalRevenues: '1000',
		totalExpenses: '300',
		expensePosts: [post(6000, 'Loyer', '200', '66.7'), post(6100, 'Assurances', '100', '33.3')],
		expensePostsBase: '300',
	});

beforeEach(() => goto.mockReset());

describe('états', () => {
	it('AC16 : le chargement le dit, sans barre fantôme', () => {
		render(HomepageCharts, { state: 'loading', summary: null });
		const el = screen.getByTestId('homepage-charts-loading');
		expect(el.getAttribute('aria-busy')).toBe('true');
		expect(document.querySelector('svg')).toBeNull();
	});

	it('AC17 : l’échec se dit une seule fois, jamais un graphique vide', () => {
		render(HomepageCharts, { state: 'error', summary: null });
		expect(screen.getAllByTestId('homepage-charts-unavailable')).toHaveLength(1);
		expect(document.querySelector('svg')).toBeNull();
		expect(screen.queryByTestId('homepage-charts-empty')).toBeNull();
	});

	it('AC18 : aucun exercice ouvert, une phrase, aucun graphique, aucun lien', () => {
		render(HomepageCharts, {
			state: 'ready',
			summary: summary({ state: 'noOpenFiscalYear', fiscalYear: null, months: [] }),
		});
		const el = screen.getByTestId('homepage-charts-no-open-year');
		expect(el.querySelector('a')).toBeNull();
		expect(document.querySelector('svg')).toBeNull();
		expect(screen.queryByTestId('homepage-card-monthly-bars')).toBeNull();
	});
});

describe('AC19 — chaque graphique décide sur sa seule donnée', () => {
	it('exercice vide : les deux messages, le cadre des barres reste', () => {
		render(HomepageCharts, { state: 'ready', summary: summary() });
		expect(screen.getByTestId('homepage-charts-empty')).toBeTruthy();
		expect(screen.getByTestId('homepage-charts-no-posts')).toBeTruthy();
		expect(screen.getAllByTestId('homepage-chart-month-mark')).toHaveLength(12);
		expect(screen.queryAllByTestId('homepage-chart-post-mark')).toHaveLength(0);
		expect(document.querySelector('[data-bar]')).toBeNull();
	});

	it('produits seuls : barres dessinées, pas de poste', () => {
		render(HomepageCharts, {
			state: 'ready',
			summary: summary({ months: months([['500', '0'], ...zero12.slice(1)]), totalRevenues: '500' }),
		});
		expect(screen.queryByTestId('homepage-charts-empty')).toBeNull();
		expect(document.querySelector('[data-bar="revenue"]')).not.toBeNull();
		expect(screen.getByTestId('homepage-charts-no-posts')).toBeTruthy();
	});

	it('charges toutes créditrices : pas de poste, et la note', () => {
		render(HomepageCharts, {
			state: 'ready',
			summary: summary({
				months: months([['0', '-50'], ...zero12.slice(1)]),
				totalExpenses: '-50',
				creditBalanceExpenseAccounts: 1,
			}),
		});
		expect(screen.getByTestId('homepage-charts-no-posts')).toBeTruthy();
		expect(screen.getByTestId('homepage-charts-credit-note').textContent).toContain('1 compte de charges');
	});

	it('reclassification_keeps_the_pie : barres vides, part, tableau et note', () => {
		render(HomepageCharts, {
			state: 'ready',
			summary: summary({
				expensePosts: [post(6000, 'Loyer', '100', '100.0')],
				expensePostsBase: '100',
				creditBalanceExpenseAccounts: 1,
			}),
		});
		expect(screen.getByTestId('homepage-charts-empty')).toBeTruthy();
		expect(screen.queryByTestId('homepage-charts-no-posts')).toBeNull();
		expect(screen.getAllByTestId('homepage-chart-post-mark')).toHaveLength(1);
		expect(document.querySelector('[data-testid="homepage-chart-post-mark"] circle')).not.toBeNull();
		expect(screen.getByTestId('homepage-chart-post-link').textContent).toContain('Loyer');
		expect(screen.getByTestId('homepage-charts-credit-note')).toBeTruthy();
	});
});

describe('AC10 — la note des comptes créditeurs', () => {
	it('au compte 1 : singulier ; au compte 2 : pluriel ; à zéro : absente', () => {
		const a = render(HomepageCharts, {
			state: 'ready',
			summary: { ...populated(), creditBalanceExpenseAccounts: 1 },
		});
		expect(screen.getByTestId('homepage-charts-credit-note').textContent).toBe(
			'1 compte de charges au solde créditeur ne figure pas dans ce diagramme.',
		);
		a.unmount();
		const b = render(HomepageCharts, {
			state: 'ready',
			summary: { ...populated(), creditBalanceExpenseAccounts: 2 },
		});
		expect(screen.getByTestId('homepage-charts-credit-note').textContent).toBe(
			'2 comptes de charges au solde créditeur ne figurent pas dans ce diagramme.',
		);
		b.unmount();
		render(HomepageCharts, { state: 'ready', summary: populated() });
		expect(screen.queryByTestId('homepage-charts-credit-note')).toBeNull();
	});
});

describe('accessibilité (AC11 à AC13)', () => {
	it('chaque svg : role img, titre et description chiffrés, aucun descendant focalisable', () => {
		render(HomepageCharts, { state: 'ready', summary: populated() });
		const svgs = Array.from(document.querySelectorAll('svg[role="img"]'));
		expect(svgs).toHaveLength(2);
		for (const svg of svgs) {
			const ids = (svg.getAttribute('aria-labelledby') ?? '').split(' ');
			expect(ids).toHaveLength(2);
			expect(svg.querySelector(`[id="${ids[0]}"]`)?.tagName.toLowerCase()).toBe('title');
			expect(svg.querySelector(`[id="${ids[1]}"]`)?.tagName.toLowerCase()).toBe('desc');
			expect(svg.querySelectorAll('a, [tabindex], button')).toHaveLength(0);
		}
		const [bars, pie] = svgs;
		expect(bars.querySelector('desc')?.textContent).toContain('1’000.00');
		expect(pie.querySelector('desc')?.textContent).toContain('Loyer');
		expect(pie.querySelector('desc')?.textContent).toContain('66.7');
		expect(pie.querySelector('desc')?.textContent).toContain('2 comptes de charges');
	});

	it('la description compte les comptes de « Autres », et connaît le singulier (B-5)', () => {
		const a = render(HomepageCharts, {
			state: 'ready',
			summary: summary({
				expensePosts: [post(6000, 'Loyer', '50', '50.0')],
				otherExpenses: { amount: '50', share: '50.0', accountCount: 4 },
				expensePostsBase: '100',
			}),
		});
		expect(screen.getByTestId('homepage-card-expense-pie').querySelector('desc')?.textContent).toContain(
			'5 comptes de charges',
		);
		a.unmount();
		const b = render(HomepageCharts, {
			state: 'ready',
			summary: summary({
				months: months([['0', '100']]),
				expensePosts: [post(6000, 'Loyer', '100', '100.0')],
				expensePostsBase: '100',
			}),
		});
		const pieDesc = screen.getByTestId('homepage-card-expense-pie').querySelector('desc')?.textContent;
		expect(pieDesc).toContain('un seul compte de charges');
		expect(pieDesc).toContain('Loyer');
		const barsDesc = screen.getByTestId('homepage-card-monthly-bars').querySelector('desc')?.textContent;
		expect(barsDesc).toContain('sur un mois');
		b.unmount();
	});

	it('le tableau des mois : caption, th scope, vrai lien, dans un details', () => {
		render(HomepageCharts, { state: 'ready', summary: populated() });
		const card = screen.getByTestId('homepage-card-monthly-bars');
		const table = card.querySelector('details table')!;
		expect(table.querySelector('caption')).not.toBeNull();
		expect(table.querySelectorAll('th[scope="col"]').length).toBeGreaterThanOrEqual(3);
		const links = screen.getAllByTestId('homepage-chart-month-link');
		expect(links).toHaveLength(12);
		expect(links[0].getAttribute('href')).toBe('/journal-entries?dateFrom=2026-01-01&dateTo=2026-01-28');
		expect(links[0].closest('tr')?.getAttribute('data-revenues')).toBe('1000');
	});

	it('le tableau des postes : visible, rang, compte, montant, part, vrai lien', () => {
		render(HomepageCharts, { state: 'ready', summary: populated() });
		const card = screen.getByTestId('homepage-card-expense-pie');
		expect(card.querySelector('details')).toBeNull();
		expect(card.querySelector('table caption')).not.toBeNull();
		const links = screen.getAllByTestId('homepage-chart-post-link');
		expect(links[0].getAttribute('href')).toBe(
			'/reports?tab=general-ledger&accountId=6000&from=2026-01-01&to=2026-12-31',
		);
		const row = links[0].closest('tr')!;
		expect(row.textContent).toContain('200.00');
		expect(row.textContent).toContain('66.7 %');
		expect(row.getAttribute('data-amount')).toBe('200');
	});
});

describe('liens et clic de souris (AC5, AC8, C-e332-18)', () => {
	it('clic sur une marque de mois : goto reçoit son data-href', async () => {
		render(HomepageCharts, { state: 'ready', summary: populated() });
		const mark = screen.getAllByTestId('homepage-chart-month-mark')[0];
		await fireEvent.click(mark.querySelector('rect')!);
		expect(goto).toHaveBeenCalledWith('/journal-entries?dateFrom=2026-01-01&dateTo=2026-01-28');
	});

	it('clic sur une part de compte : le grand livre sur tout l’exercice', async () => {
		render(HomepageCharts, { state: 'ready', summary: populated() });
		const mark = screen.getAllByTestId('homepage-chart-post-mark')[1];
		await fireEvent.click(mark.querySelector('path')!);
		expect(goto).toHaveBeenCalledWith(
			'/reports?tab=general-ledger&accountId=6100&from=2026-01-01&to=2026-12-31',
		);
	});

	it('clic hors d’une marque : rien', async () => {
		render(HomepageCharts, { state: 'ready', summary: populated() });
		await fireEvent.click(document.querySelector('svg[role="img"]')!);
		expect(goto).not.toHaveBeenCalled();
	});

	it('« Autres » : ni lien ni data-href, au dessin comme au tableau', () => {
		render(HomepageCharts, {
			state: 'ready',
			summary: summary({
				expensePosts: [post(6000, 'Loyer', '50', '50.0')],
				otherExpenses: { amount: '50', share: '50.0', accountCount: 2 },
				expensePostsBase: '100',
			}),
		});
		const row = screen.getByTestId('homepage-chart-others-row');
		expect(row.querySelector('a')).toBeNull();
		expect(row.textContent).toContain('Autres');
		expect(row.textContent).toContain('2');
		const others = screen.getByTestId('homepage-chart-others-mark');
		expect(others.getAttribute('data-href')).toBeNull();
		expect(others.closest('[data-href]')).toBeNull();
		expect(screen.getAllByTestId('homepage-chart-post-link')).toHaveLength(1);
	});
});

describe('AC2 — un exercice long', () => {
	it('au-delà de 12 mois : la carte des barres passe en pleine largeur, viewBox de 1200', () => {
		const long = Array.from({ length: 13 }, (): [string, string] => ['0', '0']);
		const m = long.map(([revenues, expenses], i) => {
			const y = 2026 + Math.floor(i / 12);
			const mm = String((i % 12) + 1).padStart(2, '0');
			return { month: `${y}-${mm}`, from: `${y}-${mm}-01`, to: `${y}-${mm}-28`, revenues, expenses };
		});
		render(HomepageCharts, { state: 'ready', summary: summary({ months: m }) });
		const card = screen.getByTestId('homepage-card-monthly-bars');
		expect(card.className).toContain('lg:col-span-2');
		expect(card.querySelector('svg[role="img"]')?.getAttribute('viewBox')).toMatch(/^0 0 1200 /);
	});

	it('jusqu’à 12 mois : une colonne, viewBox de 720', () => {
		render(HomepageCharts, { state: 'ready', summary: summary() });
		const card = screen.getByTestId('homepage-card-monthly-bars');
		expect(card.className).not.toContain('lg:col-span-2');
		expect(card.querySelector('svg[role="img"]')?.getAttribute('viewBox')).toMatch(/^0 0 720 /);
	});
});

describe('34-10 (#970) — couleurs unies et anneau', () => {
	/** Un poste à 30, un compte au solde créditeur à −80 : total −50, base 30 (AC-4). */
	const creditCase = (): HomepageSummary =>
		summary({
			months: months([['0', '-50'], ...zero12.slice(1)]),
			totalExpenses: '-50',
			expensePosts: [post(6000, 'Loyer', '30', '100.0')],
			expensePostsBase: '30',
			creditBalanceExpenseAccounts: 1,
		});

	const pieCard = () => screen.getByTestId('homepage-card-expense-pie');
	const pieDesc = () => pieCard().querySelector('desc')?.textContent ?? '';

	it('AC-8 : aucun motif, barres et légende aux couleurs de leurs jetons', () => {
		render(HomepageCharts, { state: 'ready', summary: populated() });
		expect(document.querySelector('pattern')).toBeNull();
		expect(document.querySelector('[data-bar="revenue"]')?.getAttribute('fill')).toBe(BAR_REVENUE.color);
		expect(document.querySelector('[data-bar="expense"]')?.getAttribute('fill')).toBe(BAR_EXPENSE.color);
		const legend = Array.from(
			screen.getByTestId('homepage-card-monthly-bars').querySelectorAll('ul svg rect'),
		).map((r) => r.getAttribute('fill'));
		expect(legend).toEqual([BAR_REVENUE.color, BAR_EXPENSE.color]);
	});

	it('AC-5 : la part et la pastille de son rang portent la même couleur unie', () => {
		render(HomepageCharts, {
			state: 'ready',
			summary: summary({
				expensePosts: [post(6000, 'Loyer', '50', '50.0'), post(6100, 'Assurances', '30', '30.0')],
				otherExpenses: { amount: '20', share: '20.0', accountCount: 3 },
				expensePostsBase: '100',
				totalExpenses: '100',
			}),
		});
		const marks = screen.getAllByTestId('homepage-chart-post-mark');
		expect(marks[0].querySelector('path')?.getAttribute('fill')).toBe(PIE_PALETTE[0].color);
		expect(marks[1].querySelector('path')?.getAttribute('fill')).toBe(PIE_PALETTE[1].color);
		expect(
			screen.getByTestId('homepage-chart-others-mark').querySelector('path')?.getAttribute('fill'),
		).toBe(OTHERS_STYLE.color);
		const swatches = Array.from(pieCard().querySelectorAll('tbody svg rect')).map((r) => r.getAttribute('fill'));
		expect(swatches).toEqual([PIE_PALETTE[0].color, PIE_PALETTE[1].color, OTHERS_STYLE.color]);
		// Le rang reste écrit : il relie la part à son compte sans la couleur.
		expect(pieCard().querySelector('tbody tr td')?.textContent?.trim()).toBe('1');
		// Le centre n'est ni une part ni un `path` : trois parts, trois `path`.
		expect(pieCard().querySelectorAll('svg[role="img"] path')).toHaveLength(3);
		expect(
			screen.getByTestId('homepage-chart-pie-total').closest('[data-testid$="-mark"]'),
		).toBeNull();
	});

	it('AC-4 : le centre porte le total des dépenses de l’exercice', () => {
		render(HomepageCharts, { state: 'ready', summary: populated() });
		const total = screen.getByTestId('homepage-chart-pie-total');
		expect(total.textContent).toContain('Total des dépenses');
		expect(total.textContent).toContain('300.00');
	});

	it('AC-4 : un poste plein reste un cercle, et le centre s’y écrit', () => {
		render(HomepageCharts, {
			state: 'ready',
			summary: summary({
				expensePosts: [post(6000, 'Loyer', '100', '100.0')],
				expensePostsBase: '100',
				totalExpenses: '100',
			}),
		});
		const circle = document.querySelector('[data-testid="homepage-chart-post-mark"] circle');
		expect(circle?.getAttribute('fill')).toBe('none');
		expect(circle?.getAttribute('stroke')).toBe(PIE_PALETTE[0].color);
		expect(screen.getByTestId('homepage-chart-pie-total').textContent).toContain('100.00');
	});

	it('AC-4 : un compte, totaux différents — centre −50, phrase de la base après la note', () => {
		render(HomepageCharts, { state: 'ready', summary: creditCase() });
		expect(screen.getByTestId('homepage-chart-pie-total').textContent).toContain('-50.00');
		const note = screen.getByTestId('homepage-charts-credit-note');
		const base = screen.getByTestId('homepage-charts-pie-base-note');
		expect(base.textContent).toBe('Les parts portent sur 30.00.');
		expect(note.compareDocumentPosition(base) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
		expect(pieDesc()).toContain('un seul compte de charges');
		expect(pieDesc()).toContain('total des dépenses -50.00');
		expect(pieDesc()).toContain('Parts calculées sur 30.00.');
		expect(pieDesc()).not.toContain('pour un total de');
	});

	it('AC-4 : un compte, totaux égaux en valeur (« 100.00 » et « 100 ») — ni phrase ni base au desc', () => {
		render(HomepageCharts, {
			state: 'ready',
			summary: summary({
				expensePosts: [post(6000, 'Loyer', '100', '100.0')],
				expensePostsBase: '100',
				totalExpenses: '100.00',
			}),
		});
		expect(screen.queryByTestId('homepage-charts-pie-base-note')).toBeNull();
		expect(pieDesc()).toContain('total des dépenses 100.00');
		expect(pieDesc()).not.toContain('Parts calculées');
	});

	it('AC-4 : plusieurs comptes, totaux différents — total au desc, base jointe', () => {
		render(HomepageCharts, {
			state: 'ready',
			summary: { ...populated(), totalExpenses: '250', creditBalanceExpenseAccounts: 1 },
		});
		expect(screen.getByTestId('homepage-chart-pie-total').textContent).toContain('250.00');
		expect(screen.getByTestId('homepage-charts-pie-base-note').textContent).toBe(
			'Les parts portent sur 300.00.',
		);
		expect(pieDesc()).toContain('2 comptes de charges');
		expect(pieDesc()).toContain('total des dépenses 250.00');
		expect(pieDesc()).toContain('Parts calculées sur 300.00.');
		expect(pieDesc()).not.toContain('pour un total de');
	});

	it('AC-4, AC-6 : plusieurs comptes, totaux égaux — pas de base, l’ordre des parts au desc', () => {
		render(HomepageCharts, { state: 'ready', summary: populated() });
		expect(screen.queryByTestId('homepage-charts-pie-base-note')).toBeNull();
		expect(pieDesc()).toContain('total des dépenses 300.00');
		expect(pieDesc()).not.toContain('Parts calculées');
		expect(pieDesc()).toContain('dans le sens horaire à partir de midi');
	});

	it('AC-4 : sans poste et totaux différents — ni anneau, ni centre, ni phrase de la base', () => {
		render(HomepageCharts, {
			state: 'ready',
			summary: summary({
				months: months([['0', '-50'], ...zero12.slice(1)]),
				totalExpenses: '-50',
				creditBalanceExpenseAccounts: 1,
			}),
		});
		expect(screen.getByTestId('homepage-charts-no-posts')).toBeTruthy();
		expect(screen.getByTestId('homepage-charts-credit-note')).toBeTruthy();
		expect(screen.queryByTestId('homepage-chart-pie-total')).toBeNull();
		expect(screen.queryByTestId('homepage-charts-pie-base-note')).toBeNull();
	});

	it('revue P4, C4-3 : des postes, mais aucune part dessinable — l’état vide, jamais un anneau vide', () => {
		render(HomepageCharts, {
			state: 'ready',
			summary: summary({
				expensePosts: [post(6000, 'Loyer', '0', '0.0')],
				expensePostsBase: '100',
				// Total différent de la base : l'absence de la phrase de la base prouve alors que
				// `baseDiffers` lit `hasPosts` (revue P5, C5-1).
				totalExpenses: '150',
			}),
		});
		expect(screen.getByTestId('homepage-charts-no-posts')).toBeTruthy();
		expect(pieCard().querySelector('svg[role="img"]')).toBeNull();
		expect(screen.queryByTestId('homepage-chart-pie-total')).toBeNull();
		expect(screen.queryByTestId('homepage-charts-pie-base-note')).toBeNull();
	});

	it('revues P1 B1-1, P2 B2-1 : deux parts étroites voisines, visibles, séparées par un trait radial', () => {
		render(HomepageCharts, {
			state: 'ready',
			summary: summary({
				// Deux parts minuscules VOISINES (rang 2 et « Autres »), le cas de la capture P1.
				expensePosts: [post(6000, 'Loyer', '998', '99.8'), post(6100, 'Timbres', '1', '0.1')],
				otherExpenses: { amount: '1', share: '0.1', accountCount: 2 },
				expensePostsBase: '1000',
				totalExpenses: '1000',
			}),
		});
		const svg = pieCard().querySelector('svg[role="img"]')!;
		const paths = Array.from(svg.querySelectorAll('path'));
		// Les parts n'ont aucun contour : rien ne mord sur une part étroite par son propre trait.
		for (const p of paths) expect(p.getAttribute('stroke')).toBeNull();
		// Les séparateurs, un par limite, sont peints APRÈS toutes les parts.
		const group = screen.getByTestId('homepage-chart-pie-separators');
		const lines = Array.from(group.querySelectorAll('line'));
		expect(lines).toHaveLength(3);
		for (const p of paths) {
			expect(p.compareDocumentPosition(group) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
		}
		for (const l of lines) {
			expect(l.getAttribute('stroke')).toBe('var(--color-surface)');
			expect(l.getAttribute('stroke-width')).toBe('1.5');
		}
		// Un séparateur tombe sur la limite entre les deux parts étroites : le point intérieur de
		// fin de la part « Timbres » est celui de début de « Autres », et un trait y part.
		const tiny = screen.getAllByTestId('homepage-chart-post-mark')[1].querySelector('path')!;
		const inner = /L(-?[\d.]+) (-?[\d.]+) A[\d.]+ [\d.]+ 0 [01] 0 (-?[\d.]+) (-?[\d.]+) Z$/.exec(
			tiny.getAttribute('d') ?? '',
		);
		expect(inner).not.toBeNull();
		const [ex, ey, sx, sy] = inner!.slice(1).map(Number);
		const at = (x: number, y: number) =>
			lines.some(
				(l) => Math.hypot(Number(l.getAttribute('x1')) - x, Number(l.getAttribute('y1')) - y) < 0.01,
			);
		expect(at(ex, ey), 'limite Timbres / Autres marquée').toBe(true);
		expect(at(sx, sy), 'limite Loyer / Timbres marquée').toBe(true);
		// Visible : la largeur intérieure moins les deux demi-séparateurs fait au moins une unité.
		expect(Math.hypot(ex - sx, ey - sy) - 1.5).toBeGreaterThanOrEqual(1);
		// Les séparateurs ne captent pas le clic : la part reste un lien vers son grand livre.
		expect(group.getAttribute('pointer-events')).toBe('none');
		expect(tiny.closest('[data-href]')?.getAttribute('data-href')).toContain('accountId=6100');
	});
});
