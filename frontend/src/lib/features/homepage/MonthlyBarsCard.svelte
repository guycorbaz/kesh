<script lang="ts">
	// Story 33-3b (#918) — les revenus et les dépenses de chaque mois de l'exercice.
	// Le SVG est une image (`role="img"`), sans aucun élément focalisable ; le clavier
	// et le lecteur d'écran passent par le vrai tableau voisin (C-e332-16).
	import { goto } from '$app/navigation';
	import { i18nLocale, i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import ChartPatterns from './ChartPatterns.svelte';
	import {
		BAR_EXPENSE,
		BAR_REVENUE,
		FULL_WIDTH_ABOVE_MONTHS,
		axisLabels,
		chartAmount,
		chartFill,
		chartWidth,
		followMark,
		monthHref,
		monthLabel,
		monthYearLabel,
		monthlyBars,
		type ChartBox,
	} from './homepage-charts';
	import type { HomepageSummary } from './homepage-summary.api';

	interface Props {
		summary: Extract<HomepageSummary, { state: 'open' }>;
	}
	let { summary }: Props = $props();

	const PREFIX = 'homepage-bars';
	const expenseFill = chartFill(PREFIX, BAR_EXPENSE);

	let n = $derived(summary.months.length);
	let wide = $derived(n > FULL_WIDTH_ABOVE_MONTHS);
	let width = $derived(chartWidth(n));
	let box = $derived.by((): ChartBox => {
		const base: ChartBox = {
			width,
			height: wide ? 360 : 320,
			marginTop: 12,
			marginBottom: wide ? 34 : 36,
			marginLeft: 0,
			// Une étiquette centrée sur le dernier groupe ne déborde pas.
			marginRight: wide ? 36 : 8,
		};
		// La marge suit la plus longue graduation.
		const longest = Math.max(...monthlyBars(summary.months, base).ticks.map((t) => t.label.length));
		return { ...base, marginLeft: longest * (wide ? 9 : 11) + 14 };
	});
	let chart = $derived(monthlyBars(summary.months, box));
	let labels = $derived(axisLabels(summary.months, i18nLocale()));
</script>

<div
	class="rounded-lg border border-border bg-white p-6 shadow-sm {wide ? 'lg:col-span-2' : ''}"
	data-testid="homepage-card-monthly-bars"
>
	<h2 class="text-lg font-semibold text-text">
		{i18nMsg('homepage-charts-bars-title', 'Revenus et dépenses par mois')}
	</h2>
	<p class="text-sm text-text-muted">{summary.fiscalYear.name}</p>

	<ul class="mt-3 flex flex-wrap gap-4 text-sm text-text">
		<li class="flex items-center gap-2">
			<svg aria-hidden="true" width="14" height="14"><rect width="14" height="14" fill={BAR_REVENUE.color} /></svg>
			{i18nMsg('homepage-charts-legend-revenues', 'Revenus')}
		</li>
		<li class="flex items-center gap-2">
			<svg aria-hidden="true" width="14" height="14"><rect width="14" height="14" fill={expenseFill} stroke={BAR_EXPENSE.color} /></svg>
			{i18nMsg('homepage-charts-legend-expenses', 'Dépenses')}
		</li>
	</ul>

	<!-- Un seul écouteur sur l'image : le clavier passe par le tableau. -->
	<!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_noninteractive_element_interactions -->
	<svg
		class="chart mt-3 h-auto w-full {wide ? 'wide max-w-[1200px]' : 'max-w-3xl'}"
		role="img"
		aria-labelledby="homepage-bars-title homepage-bars-desc"
		viewBox="0 0 {width} {box.height}"
		onclick={(event) => followMark(event, goto)}
	>
		<title id="homepage-bars-title">{i18nMsg('homepage-charts-bars-title', 'Revenus et dépenses par mois')}</title>
		<desc id="homepage-bars-desc">
			{n === 1
				? i18nMsg('homepage-charts-bars-desc-one', '{ $year } : revenus { $revenues }, dépenses { $expenses }, sur un mois.', {
						year: summary.fiscalYear.name,
						revenues: chartAmount(summary.totalRevenues),
						expenses: chartAmount(summary.totalExpenses),
					})
				: i18nMsg('homepage-charts-bars-desc', '{ $year } : revenus { $revenues }, dépenses { $expenses }, sur { $months } mois.', {
						year: summary.fiscalYear.name,
						revenues: chartAmount(summary.totalRevenues),
						expenses: chartAmount(summary.totalExpenses),
						months: n,
					})}
		</desc>
		<ChartPatterns prefix={PREFIX} styles={[BAR_EXPENSE]} size={wide ? 12 : 10} />

		{#each chart.ticks as tick (tick.value.toString())}
			<line x1={box.marginLeft} x2={width - box.marginRight} y1={tick.y} y2={tick.y} stroke="var(--color-border)" />
			<text class="axis" x={box.marginLeft - 8} y={tick.y} text-anchor="end" dominant-baseline="middle">{tick.label}</text>
		{/each}

		{#each chart.groups as group (group.month.month)}
			<g
				class="mark"
				data-href={monthHref(group.month)}
				data-testid="homepage-chart-month-mark"
				data-month={group.month.month}
			>
				<title>
					{i18nMsg('homepage-charts-month-mark-title', '{ $month } : revenus { $revenues }, dépenses { $expenses }', {
						month: monthYearLabel(group.month.month, i18nLocale()),
						revenues: chartAmount(group.month.revenues),
						expenses: chartAmount(group.month.expenses),
					})}
				</title>
				<rect class="hit" x={group.x} y={chart.plotTop} width={group.width} height={chart.plotBottom - chart.plotTop} fill="transparent" pointer-events="all" />
				{#if group.revenue}
					<rect data-bar="revenue" x={group.revenue.x} y={group.revenue.y} width={group.revenue.width} height={group.revenue.height} fill={BAR_REVENUE.color} />
				{/if}
				{#if group.expense}
					<rect data-bar="expense" x={group.expense.x} y={group.expense.y} width={group.expense.width} height={group.expense.height} fill={expenseFill} stroke={BAR_EXPENSE.color} />
				{/if}
			</g>
		{/each}

		<line x1={box.marginLeft} x2={width - box.marginRight} y1={chart.zeroY} y2={chart.zeroY} stroke="var(--color-text-muted)" stroke-width="1.5" />

		{#each labels as label (label.index)}
			<text
				class="axis"
				x={chart.groups[label.index].x + chart.groups[label.index].width / 2}
				y={box.height - 8}
				text-anchor="middle"
			>{label.text}</text>
		{/each}
	</svg>

	{#if chart.isEmpty}
		<p class="mt-2 text-sm text-text-muted" data-testid="homepage-charts-empty">
			{i18nMsg('homepage-charts-empty', 'Aucun mouvement net de produits ou de charges dans cet exercice.')}
		</p>
	{/if}

	<details class="mt-3 text-sm" data-testid="homepage-chart-months-details">
		<summary class="cursor-pointer text-primary">
			{i18nMsg('homepage-charts-months-summary', 'Détail par mois')}
		</summary>
		<table class="mt-2 w-full">
			<caption class="sr-only">{i18nMsg('homepage-charts-bars-title', 'Revenus et dépenses par mois')}</caption>
			<thead>
				<tr class="border-b border-border text-left text-text-muted">
					<th scope="col" class="py-1 font-medium">{i18nMsg('homepage-charts-th-month', 'Mois')}</th>
					<th scope="col" class="py-1 text-right font-medium">{i18nMsg('homepage-charts-legend-revenues', 'Revenus')}</th>
					<th scope="col" class="py-1 text-right font-medium">{i18nMsg('homepage-charts-legend-expenses', 'Dépenses')}</th>
				</tr>
			</thead>
			<tbody>
				{#each summary.months as m (m.month)}
					<tr class="border-b border-border last:border-0" data-month={m.month} data-revenues={m.revenues} data-expenses={m.expenses}>
						<th scope="row" class="py-1 text-left font-normal">
							<a class="text-primary underline" href={monthHref(m)} data-testid="homepage-chart-month-link">{monthYearLabel(m.month, i18nLocale())}</a>
						</th>
						<td class="py-1 text-right font-mono">{chartAmount(m.revenues)}</td>
						<td class="py-1 text-right font-mono">{chartAmount(m.expenses)}</td>
					</tr>
				{/each}
			</tbody>
		</table>
	</details>
</div>

<style>
	/* Corps en unités du `viewBox`, un par largeur, sans adaptation au téléphone. La page
	   ne descend pas sous 1280 px (`min-w-[1280px]`, `(app)/+layout.svelte`) : une fenêtre
	   plus étroite la parcourt de côté. Sous 1024 px, `lg:` repasse la grille sur une
	   colonne : la carte s'élargit, et `max-w-*` borne le dessin (C-e332-30). */
	.chart :global(.axis) {
		fill: var(--color-text-muted);
		font-size: 18px;
	}
	.chart.wide :global(.axis) {
		font-size: 15px;
	}
	.mark {
		cursor: pointer;
	}
	.mark:hover .hit {
		fill: var(--color-surface-alt);
	}
</style>
