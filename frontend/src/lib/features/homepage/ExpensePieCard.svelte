<script lang="ts">
	// Story 33-3b (#918) — la part des plus gros postes de charges, cinq plus
	// « Autres ». Le tableau tient lieu de légende, et porte les seuls liens
	// (C-e332-16) ; « Autres » n'en a pas (C-e332-18).
	// Story 34-10 (#970) : un anneau en couleurs unies, séparées par un liseré de la
	// couleur de la carte, le total des dépenses de l'exercice au centre (C-e342-17).
	import { goto } from '$app/navigation';
	import { ledgerHref } from '$lib/features/reports/reports.api';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import {
		OTHERS_STYLE,
		PIE_GEOMETRY,
		SEPARATOR_WIDTH,
		centerLines,
		centerSizes,
		chartAmount,
		differentAmounts,
		followMark,
		fullRing,
		pieSeparators,
		pieSlices,
		postStyle,
	} from './homepage-charts';
	import type { ExpensePost, HomepageSummary } from './homepage-summary.api';

	interface Props {
		summary: Extract<HomepageSummary, { state: 'open' }>;
	}
	let { summary }: Props = $props();

	const GEOMETRY = PIE_GEOMETRY;
	const RING = fullRing(GEOMETRY);
	const HOLE = 2 * GEOMETRY.innerR;

	let slices = $derived(
		pieSlices(summary.expensePosts, summary.otherExpenses, summary.expensePostsBase, GEOMETRY),
	);
	let hasPosts = $derived(summary.expensePosts.length > 0);
	let creditCount = $derived(summary.creditBalanceExpenseAccounts);

	/** Le total des dépenses de l'exercice, celui des barres et du compte de résultat. */
	let totalLabel = $derived(i18nMsg('homepage-charts-pie-total-label', 'Total des dépenses'));
	let totalAmount = $derived(chartAmount(summary.totalExpenses));
	let sizes = $derived(centerSizes(totalLabel, totalAmount, HOLE));
	let lines = $derived(centerLines(GEOMETRY.cy, sizes.labelSize, sizes.amountSize));
	let separators = $derived(pieSeparators(slices, GEOMETRY));
	/**
	 * Les parts portent sur une autre base que le total du centre : à le dire, et seulement
	 * s'il y a des parts (AC-4, validation P6). Comparaison en valeur (P7-F2).
	 */
	let baseDiffers = $derived(
		hasPosts && differentAmounts(summary.totalExpenses, summary.expensePostsBase),
	);

	/** Le poste de cette part, `null` pour « Autres ». */
	function postOf(accountId: number | null): ExpensePost | null {
		return summary.expensePosts.find((p) => p.accountId === accountId) ?? null;
	}

	/** Le grand livre du compte, sur l'exercice entier (C-e332-20). */
	function postHref(accountId: number): string {
		return ledgerHref(accountId, summary.fiscalYear.startDate, summary.fiscalYear.endDate);
	}

	/** Le nombre de comptes que le diagramme répartit, ceux de « Autres » compris. */
	let accountCount = $derived(
		summary.expensePosts.length + (summary.otherExpenses?.accountCount ?? 0),
	);

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
			onclick={(event) => followMark(event, goto)}
		>
			<title id="homepage-pie-title">{i18nMsg('homepage-charts-pie-title', 'Principaux postes de charges')}</title>
			<desc id="homepage-pie-desc">
				{#if accountCount === 1}
					{i18nMsg(
						'homepage-charts-pie-desc-one',
						'{ $year } : un seul compte de charges, { $top } ; total des dépenses { $total }.',
						{
							year: summary.fiscalYear.name,
							total: totalAmount,
							top: summary.expensePosts[0].name,
						},
					)}
				{:else}
					{i18nMsg(
						'homepage-charts-pie-desc',
						'{ $year } : { $count } comptes de charges, total des dépenses { $total }. Le plus gros : { $top }, { $share } %. Les parts suivent l’ordre du tableau, dans le sens horaire à partir de midi.',
						{
							year: summary.fiscalYear.name,
							count: accountCount,
							total: totalAmount,
							top: summary.expensePosts[0].name,
							share: summary.expensePosts[0].share,
						},
					)}
				{/if}
				{#if baseDiffers}
					{i18nMsg('homepage-charts-pie-desc-base', 'Parts calculées sur { $base }.', {
						base: chartAmount(summary.expensePostsBase),
					})}
				{/if}
			</desc>
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
									amount: chartAmount(post.amount),
									share: post.share,
								})
							: summary.otherExpenses
								? i18nMsg('homepage-charts-post-mark-title', '{ $name } : { $amount }, { $share } %', {
										name: othersLabel(summary.otherExpenses.accountCount),
										amount: chartAmount(summary.otherExpenses.amount),
										share: summary.otherExpenses.share,
									})
								: ''}
					</title>
					{#if slice.full}
						<!-- La part qui fait le tour : un cercle au rayon médian, au trait de l'épaisseur de l'anneau. -->
						<circle cx={GEOMETRY.cx} cy={GEOMETRY.cy} r={RING.r} fill="none" stroke={slice.style.color} stroke-width={RING.width} />
					{:else}
						<path d={slice.path} fill={slice.style.color} />
					{/if}
				</g>
			{/each}
			<!-- Le liseré, de la couleur de la carte : un trait radial par limite, peint au-dessus des parts,
			     qui marque chaque limite quelle que soit la paire de teintes et la largeur des parts
			     (C-e342-16, C-e3410-12). Il ne capte pas le clic : la part dessous reste un lien. -->
			<g data-testid="homepage-chart-pie-separators" pointer-events="none">
				<!-- Clé par l'indice, unique par construction : deux angles égaux ne lèvent pas `each_key_duplicate` (revue P3, B3-1). -->
				{#each separators as line, i (i)}
					<line x1={line.x1} y1={line.y1} x2={line.x2} y2={line.y2} stroke="var(--color-surface)" stroke-width={SEPARATOR_WIDTH} />
				{/each}
			</g>
			<!-- Le centre : ni une part ni un `path`, hors de toute marque (AC-3, AC-4). -->
			<g data-testid="homepage-chart-pie-total" pointer-events="none">
				<text
					x={GEOMETRY.cx}
					y={lines.labelY}
					text-anchor="middle"
					fill="var(--color-text)"
					font-size={sizes.labelSize}
				>{totalLabel}</text>
				<text
					x={GEOMETRY.cx}
					y={lines.amountY}
					text-anchor="middle"
					dominant-baseline="hanging"
					fill="var(--color-text)"
					font-weight="600"
					font-size={sizes.amountSize}
				>{totalAmount}</text>
			</g>
		</svg>

		<table class="mt-3 w-full text-sm">
			<caption class="sr-only">{i18nMsg('homepage-charts-pie-title', 'Principaux postes de charges')}</caption>
			<thead>
				<tr class="border-b border-border text-left text-text-muted">
					<th scope="col" class="py-1 pr-3 font-medium">{i18nMsg('homepage-charts-th-rank', 'Rang')}</th>
					<th scope="col" class="py-1 font-medium">{i18nMsg('homepage-charts-th-account', 'Compte')}</th>
					<th scope="col" class="py-1 text-right font-medium">{i18nMsg('homepage-charts-th-amount', 'Montant')}</th>
					<th scope="col" class="py-1 text-right font-medium">{i18nMsg('homepage-charts-th-share', 'Part')}</th>
				</tr>
			</thead>
			<tbody>
				{#each summary.expensePosts as post, i (post.accountId)}
					<tr class="border-b border-border last:border-0" data-account-id={post.accountId} data-amount={post.amount} data-share={post.share}>
						<td class="whitespace-nowrap py-1 pr-3">
							<svg class="mr-1 inline-block align-[-2px]" aria-hidden="true" width="14" height="14"><rect width="14" height="14" fill={postStyle(i).color} /></svg>{i + 1}
						</td>
						<th scope="row" class="py-1 text-left font-normal [overflow-wrap:anywhere]">
							<a class="text-primary underline" href={postHref(post.accountId)} data-testid="homepage-chart-post-link">{post.number} {post.name}</a>
						</th>
						<td class="py-1 pl-3 text-right font-mono">{chartAmount(post.amount)}</td>
						<td class="whitespace-nowrap py-1 pl-3 text-right font-mono">{post.share} %</td>
					</tr>
				{/each}
				{#if summary.otherExpenses}
					<tr data-testid="homepage-chart-others-row" data-amount={summary.otherExpenses.amount}>
						<td class="whitespace-nowrap py-1 pr-3">
							<svg class="mr-1 inline-block align-[-2px]" aria-hidden="true" width="14" height="14"><rect width="14" height="14" fill={OTHERS_STYLE.color} /></svg>{summary.expensePosts.length + 1}
						</td>
						<th scope="row" class="py-1 text-left font-normal">{othersLabel(summary.otherExpenses.accountCount)}</th>
						<td class="py-1 pl-3 text-right font-mono">{chartAmount(summary.otherExpenses.amount)}</td>
						<td class="whitespace-nowrap py-1 pl-3 text-right font-mono">{summary.otherExpenses.share} %</td>
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
	{#if baseDiffers}
		<!-- Après la note des comptes créditeurs si elle est rendue, seule sinon (AC-4). -->
		<p class="mt-2 text-sm text-text-muted" data-testid="homepage-charts-pie-base-note">
			{i18nMsg('homepage-charts-pie-base-note', 'Les parts portent sur { $base }.', {
				base: chartAmount(summary.expensePostsBase),
			})}
		</p>
	{/if}
</div>

<style>
	.mark {
		cursor: pointer;
	}
</style>
