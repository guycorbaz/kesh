<script lang="ts">
	// Story 33-3b (#918) — la part des plus gros postes de charges, cinq plus
	// « Autres ». Le tableau tient lieu de légende, et porte les seuls liens
	// (C-e332-16) ; « Autres » n'en a pas (C-e332-18).
	import { goto } from '$app/navigation';
	import Big from 'big.js';
	import { formatSwissAmount } from '$lib/features/journal-entries/balance';
	import { ledgerHref } from '$lib/features/reports/reports.api';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import ChartPatterns from './ChartPatterns.svelte';
	import {
		OTHERS_STYLE,
		PIE_PALETTE,
		chartFill,
		markHref,
		pieSlices,
		type ChartStyle,
	} from './homepage-charts';
	import type { ExpensePost, HomepageSummary } from './homepage-summary.api';

	interface Props {
		summary: Extract<HomepageSummary, { state: 'open' }>;
	}
	let { summary }: Props = $props();

	const PREFIX = 'homepage-pie';
	const GEOMETRY = { cx: 100, cy: 100, r: 96 };
	const STYLES: readonly ChartStyle[] = [...PIE_PALETTE, OTHERS_STYLE];

	let slices = $derived(
		pieSlices(summary.expensePosts, summary.otherExpenses, summary.expensePostsBase, GEOMETRY),
	);
	let hasPosts = $derived(summary.expensePosts.length > 0);
	let creditCount = $derived(summary.creditBalanceExpenseAccounts);

	/** Le poste de cette part, `null` pour « Autres ». */
	function postOf(accountId: number | null): ExpensePost | null {
		return summary.expensePosts.find((p) => p.accountId === accountId) ?? null;
	}

	/** Le grand livre du compte, sur l'exercice entier (C-e332-20). */
	function postHref(accountId: number): string {
		return ledgerHref(accountId, summary.fiscalYear.startDate, summary.fiscalYear.endDate);
	}

	function open(event: MouseEvent) {
		const href = markHref(event.target);
		if (href) void goto(href);
	}

	function amount(raw: string): string {
		return formatSwissAmount(new Big(raw));
	}

	function othersLabel(count: number): string {
		return i18nMsg('homepage-charts-others-label', 'Autres ({ $count })', { count });
	}
</script>

<div class="rounded-lg border border-border bg-white p-6 shadow-sm" data-testid="homepage-card-expense-pie">
	<h2 class="text-lg font-semibold text-text">
		{i18nMsg('homepage-charts-pie-title', 'Principaux postes de charges')}
	</h2>
	<p class="text-sm text-text-muted">{summary.fiscalYear.name}</p>

	{#if hasPosts}
		<!-- Un seul écouteur sur l'image : le clavier passe par le tableau. -->
		<!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_noninteractive_element_interactions -->
		<svg
			class="mx-auto mt-3 h-auto w-full max-w-60"
			role="img"
			aria-labelledby="homepage-pie-title homepage-pie-desc"
			viewBox="0 0 200 200"
			onclick={open}
		>
			<title id="homepage-pie-title">{i18nMsg('homepage-charts-pie-title', 'Principaux postes de charges')}</title>
			<desc id="homepage-pie-desc">
				{i18nMsg(
					'homepage-charts-pie-desc',
					'{ $year } : { $count } postes de charges, pour un total de { $base }. Le plus gros : { $top }, { $share } %.',
					{
						year: summary.fiscalYear.name,
						count: slices.length,
						base: amount(summary.expensePostsBase),
						top: summary.expensePosts[0].name,
						share: summary.expensePosts[0].share,
					},
				)}
			</desc>
			<ChartPatterns prefix={PREFIX} styles={STYLES} />
			{#each slices as slice (slice.rank)}
				{@const post = postOf(slice.accountId)}
				<g
					class={post ? 'mark' : ''}
					data-href={post ? postHref(post.accountId) : undefined}
					data-testid={post ? 'homepage-chart-post-mark' : 'homepage-chart-others-mark'}
					data-rank={slice.rank}
				>
					<title>
						{post
							? i18nMsg('homepage-charts-post-mark-title', '{ $name } : { $amount }, { $share } %', {
									name: post.name,
									amount: amount(post.amount),
									share: post.share,
								})
							: summary.otherExpenses
								? i18nMsg('homepage-charts-post-mark-title', '{ $name } : { $amount }, { $share } %', {
										name: othersLabel(summary.otherExpenses.accountCount),
										amount: amount(summary.otherExpenses.amount),
										share: summary.otherExpenses.share,
									})
								: ''}
					</title>
					{#if slice.full}
						<circle cx={GEOMETRY.cx} cy={GEOMETRY.cy} r={GEOMETRY.r} fill={chartFill(PREFIX, slice.style)} stroke={slice.style.color} />
					{:else}
						<path d={slice.path} fill={chartFill(PREFIX, slice.style)} stroke="#ffffff" stroke-width="2" />
					{/if}
				</g>
			{/each}
		</svg>

		<table class="mt-3 w-full text-sm">
			<caption class="sr-only">{i18nMsg('homepage-charts-pie-title', 'Principaux postes de charges')}</caption>
			<thead>
				<tr class="border-b border-border text-left text-text-muted">
					<th scope="col" class="py-1 font-medium">{i18nMsg('homepage-charts-th-rank', 'Rang')}</th>
					<th scope="col" class="py-1 font-medium">{i18nMsg('homepage-charts-th-account', 'Compte')}</th>
					<th scope="col" class="py-1 text-right font-medium">{i18nMsg('homepage-charts-th-amount', 'Montant')}</th>
					<th scope="col" class="py-1 text-right font-medium">{i18nMsg('homepage-charts-th-share', 'Part')}</th>
				</tr>
			</thead>
			<tbody>
				{#each summary.expensePosts as post, i (post.accountId)}
					<tr class="border-b border-border last:border-0" data-account-id={post.accountId} data-amount={post.amount} data-share={post.share}>
						<td class="whitespace-nowrap py-1">
							<svg class="mr-1 inline-block align-[-2px]" aria-hidden="true" width="14" height="14"><rect width="14" height="14" fill={chartFill(PREFIX, PIE_PALETTE[i])} stroke={PIE_PALETTE[i].color} /></svg>{i + 1}
						</td>
						<th scope="row" class="py-1 text-left font-normal break-words">
							<a class="text-primary underline" href={postHref(post.accountId)} data-testid="homepage-chart-post-link">{post.number} {post.name}</a>
						</th>
						<td class="py-1 text-right font-mono">{amount(post.amount)}</td>
						<td class="whitespace-nowrap py-1 text-right font-mono">{post.share} %</td>
					</tr>
				{/each}
				{#if summary.otherExpenses}
					<tr data-testid="homepage-chart-others-row" data-amount={summary.otherExpenses.amount}>
						<td class="whitespace-nowrap py-1">
							<svg class="mr-1 inline-block align-[-2px]" aria-hidden="true" width="14" height="14"><rect width="14" height="14" fill={chartFill(PREFIX, OTHERS_STYLE)} stroke={OTHERS_STYLE.color} /></svg>{summary.expensePosts.length + 1}
						</td>
						<th scope="row" class="py-1 text-left font-normal">{othersLabel(summary.otherExpenses.accountCount)}</th>
						<td class="py-1 text-right font-mono">{amount(summary.otherExpenses.amount)}</td>
						<td class="whitespace-nowrap py-1 text-right font-mono">{summary.otherExpenses.share} %</td>
					</tr>
				{/if}
			</tbody>
		</table>
	{:else}
		<p class="mt-2 text-sm text-text-muted" data-testid="homepage-charts-no-posts">
			{i18nMsg('homepage-charts-no-posts', 'Aucun poste de charges à répartir dans cet exercice.')}
		</p>
	{/if}

	{#if creditCount === 1}
		<p class="mt-2 text-sm text-text-muted" data-testid="homepage-charts-credit-note">
			{i18nMsg('homepage-charts-credit-balance-note-one', '1 compte de charges au solde créditeur ne figure pas dans ce diagramme.')}
		</p>
	{:else if creditCount > 1}
		<p class="mt-2 text-sm text-text-muted" data-testid="homepage-charts-credit-note">
			{i18nMsg('homepage-charts-credit-balance-note-other', '{ $count } comptes de charges au solde créditeur ne figurent pas dans ce diagramme.', { count: creditCount })}
		</p>
	{/if}
</div>

<style>
	.mark {
		cursor: pointer;
	}
</style>
