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
