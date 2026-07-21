<script lang="ts">
	// Story 9-1 — Vue Bilan.
	import Big from 'big.js';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { formatReportAmount, formatSwissDate, isReportEmpty } from './reports.api';
	import type { BalanceSheetDto } from './reports.types';

	interface Props {
		dto: BalanceSheetDto;
	}
	let { dto }: Props = $props();

	let empty = $derived(isReportEmpty('balance-sheet', dto));

	const fmt = formatReportAmount;

	let equityBig = $derived(
		(() => {
			try {
				return new Big(dto.equityResult);
			} catch {
				return new Big(0);
			}
		})(),
	);

	let equityClass = $derived(
		equityBig.gt(0)
			? 'text-green-700 font-semibold'
			: equityBig.lt(0)
				? 'text-red-700 font-semibold'
				: 'text-gray-700',
	);

	// P16 — fallback inclut "(avant clôture)" pour cohérence avec la valeur i18n canonique
	// et préserve la nuance OR Art. 960b (résultat pre-clôture vs définitif).
	let equityLabel = $derived(
		equityBig.gt(0)
			? i18nMsg('reports-equity-result-profit', "Bénéfice de l'exercice")
			: equityBig.lt(0)
				? i18nMsg('reports-equity-result-loss', "Perte de l'exercice")
				: i18nMsg('reports-equity-result-section-title', "Résultat de l'exercice (avant clôture)"),
	);

	// Story 14-1 — Résultat reporté (report à-nouveau virtuel) = cumul P&L des exercices
	// antérieurs. Négatif → « Perte reportée ».
	let retainedBig = $derived(
		(() => {
			try {
				return new Big(dto.retainedEarnings);
			} catch {
				return new Big(0);
			}
		})(),
	);

	let retainedClass = $derived(
		retainedBig.gt(0)
			? 'text-green-700 font-semibold'
			: retainedBig.lt(0)
				? 'text-red-700 font-semibold'
				: 'text-gray-700',
	);

	let retainedLabel = $derived(
		retainedBig.lt(0)
			? i18nMsg('reports-retained-earnings-loss', 'Perte reportée')
			: i18nMsg('reports-retained-earnings', 'Résultat reporté'),
	);
</script>

<section class="space-y-4">
	<header class="text-sm text-gray-600">
		<strong>{i18nMsg('reports-filter-period', 'Période')}:</strong>
		{formatSwissDate(dto.period.startDate)} — {formatSwissDate(dto.period.endDate)}
	</header>

	{#if empty}
		<p class="rounded bg-blue-50 p-4 text-blue-900" role="status">
			{i18nMsg('reports-error-no-entries-in-period', 'Aucune écriture dans la période sélectionnée.')}
		</p>
	{:else}
		<div class="grid gap-6 lg:grid-cols-2">
			<div>
				<h3 class="text-lg font-semibold">
					{i18nMsg('reports-section-assets', 'Actifs')}
				</h3>
				<table class="mt-2 w-full border-collapse">
					<thead>
						<tr class="border-b bg-gray-50 text-left text-sm">
							<th class="px-2 py-1">{i18nMsg('reports-column-account-number', 'N° de compte')}</th>
							<th class="px-2 py-1">{i18nMsg('reports-column-account-name', 'Intitulé')}</th>
							<th class="px-2 py-1 text-right">{i18nMsg('reports-column-balance', 'Solde')}</th>
						</tr>
					</thead>
					<tbody>
						{#each dto.assets as a (a.accountId)}
							<tr class:opacity-60={!a.active}>
								<td class="px-2 py-1 font-mono">{a.accountNumber}</td>
								<td class="px-2 py-1">{a.accountName}</td>
								<td class="px-2 py-1 text-right font-mono">{fmt(a.balance)}</td>
							</tr>
						{/each}
					</tbody>
					<tfoot>
						<tr class="border-t font-semibold">
							<td colspan="2" class="px-2 py-1"
								>{i18nMsg('reports-total-assets', 'Total actifs')}</td
							>
							<td class="px-2 py-1 text-right font-mono">{fmt(dto.totalAssets)}</td>
						</tr>
					</tfoot>
				</table>
			</div>

			<div>
				<h3 class="text-lg font-semibold">
					{i18nMsg('reports-section-liabilities', 'Passifs')}
				</h3>
				<table class="mt-2 w-full border-collapse">
					<thead>
						<tr class="border-b bg-gray-50 text-left text-sm">
							<th class="px-2 py-1">{i18nMsg('reports-column-account-number', 'N° de compte')}</th>
							<th class="px-2 py-1">{i18nMsg('reports-column-account-name', 'Intitulé')}</th>
							<th class="px-2 py-1 text-right">{i18nMsg('reports-column-balance', 'Solde')}</th>
						</tr>
					</thead>
					<tbody>
						{#each dto.liabilities as l (l.accountId)}
							<tr class:opacity-60={!l.active}>
								<td class="px-2 py-1 font-mono">{l.accountNumber}</td>
								<td class="px-2 py-1">{l.accountName}</td>
								<td class="px-2 py-1 text-right font-mono">{fmt(l.balance)}</td>
							</tr>
						{/each}
					</tbody>
					<tfoot>
						<tr class="border-t font-semibold">
							<td colspan="2" class="px-2 py-1"
								>{i18nMsg('reports-total-liabilities', 'Total passifs')}</td
							>
							<td class="px-2 py-1 text-right font-mono">{fmt(dto.totalLiabilities)}</td>
						</tr>
						<tr>
							<td colspan="2" class="px-2 py-1 {retainedClass}">{retainedLabel}</td>
							<td class="px-2 py-1 text-right font-mono {retainedClass}"
								>{fmt(dto.retainedEarnings)}</td
							>
						</tr>
						<tr>
							<td colspan="2" class="px-2 py-1 {equityClass}">{equityLabel}</td>
							<td class="px-2 py-1 text-right font-mono {equityClass}">{fmt(dto.equityResult)}</td>
						</tr>
					</tfoot>
				</table>
			</div>
		</div>

		{#if !dto.equationHolds}
			<p class="rounded bg-red-50 p-3 text-sm text-red-900" role="alert">
				{i18nMsg(
					'reports-equation-warning',
					'⚠️ Équation bilan déséquilibrée (vérifier données source).',
				)}
			</p>
		{/if}
	{/if}
</section>
