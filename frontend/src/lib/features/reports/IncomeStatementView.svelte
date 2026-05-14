<script lang="ts">
	// Story 9-1 — Vue Compte de résultat.
	import Big from 'big.js';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { formatReportAmount, formatSwissDate, isReportEmpty } from './reports.api';
	import type { IncomeStatementDto } from './reports.types';

	interface Props {
		dto: IncomeStatementDto;
	}
	let { dto }: Props = $props();
	let empty = $derived(isReportEmpty('income-statement', dto));

	const fmt = formatReportAmount;

	let netBig = $derived(
		(() => {
			try {
				return new Big(dto.netResult);
			} catch {
				return new Big(0);
			}
		})(),
	);
	let netClass = $derived(
		netBig.gt(0)
			? 'text-green-700 font-semibold'
			: netBig.lt(0)
				? 'text-red-700 font-semibold'
				: 'text-gray-700',
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
				<h3 class="text-lg font-semibold">{i18nMsg('reports-section-revenues', 'Produits')}</h3>
				<table class="mt-2 w-full border-collapse">
					<thead>
						<tr class="border-b bg-gray-50 text-left text-sm">
							<th class="px-2 py-1">{i18nMsg('reports-column-account-number', 'N°')}</th>
							<th class="px-2 py-1">{i18nMsg('reports-column-account-name', 'Intitulé')}</th>
							<th class="px-2 py-1 text-right">{i18nMsg('reports-column-balance', 'Solde')}</th>
						</tr>
					</thead>
					<tbody>
						{#each dto.revenues as r (r.accountId)}
							<tr>
								<td class="px-2 py-1 font-mono">{r.accountNumber}</td>
								<td class="px-2 py-1">{r.accountName}</td>
								<td class="px-2 py-1 text-right font-mono">{fmt(r.balance)}</td>
							</tr>
						{/each}
					</tbody>
					<tfoot>
						<tr class="border-t font-semibold">
							<td colspan="2" class="px-2 py-1"
								>{i18nMsg('reports-total-revenues', 'Total produits')}</td
							>
							<td class="px-2 py-1 text-right font-mono">{fmt(dto.totalRevenues)}</td>
						</tr>
					</tfoot>
				</table>
			</div>

			<div>
				<h3 class="text-lg font-semibold">{i18nMsg('reports-section-expenses', 'Charges')}</h3>
				<table class="mt-2 w-full border-collapse">
					<thead>
						<tr class="border-b bg-gray-50 text-left text-sm">
							<th class="px-2 py-1">{i18nMsg('reports-column-account-number', 'N°')}</th>
							<th class="px-2 py-1">{i18nMsg('reports-column-account-name', 'Intitulé')}</th>
							<th class="px-2 py-1 text-right">{i18nMsg('reports-column-balance', 'Solde')}</th>
						</tr>
					</thead>
					<tbody>
						{#each dto.expenses as e (e.accountId)}
							<tr>
								<td class="px-2 py-1 font-mono">{e.accountNumber}</td>
								<td class="px-2 py-1">{e.accountName}</td>
								<td class="px-2 py-1 text-right font-mono">{fmt(e.balance)}</td>
							</tr>
						{/each}
					</tbody>
					<tfoot>
						<tr class="border-t font-semibold">
							<td colspan="2" class="px-2 py-1"
								>{i18nMsg('reports-total-expenses', 'Total charges')}</td
							>
							<td class="px-2 py-1 text-right font-mono">{fmt(dto.totalExpenses)}</td>
						</tr>
					</tfoot>
				</table>
			</div>
		</div>

		<p class="rounded border bg-gray-50 p-3">
			<span class="font-semibold">{i18nMsg('reports-net-result', 'Résultat net')}:</span>
			<span class="ml-2 font-mono {netClass}">{fmt(dto.netResult)}</span>
		</p>
	{/if}
</section>
