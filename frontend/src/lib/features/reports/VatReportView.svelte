<script lang="ts">
	// Story 11-2 — Vue Rapport TVA (TVA due / vente).
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { formatReportAmount, formatSwissDate, isReportEmpty } from './reports.api';
	import type { VatReportDto } from './reports.types';

	interface Props {
		dto: VatReportDto;
	}
	let { dto }: Props = $props();
	let empty = $derived(isReportEmpty('vat', dto));

	const fmt = formatReportAmount;

	/** Affiche un taux décimal (ex. "8.10") avec le symbole pourcent. */
	function fmtRate(rate: string): string {
		return `${rate} %`;
	}
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
		<table class="mt-2 w-full border-collapse">
			<thead>
				<tr class="border-b bg-gray-50 text-left text-sm">
					<th class="px-2 py-1">{i18nMsg('reports-vat-column-rate', 'Taux')}</th>
					<th class="px-2 py-1 text-right"
						>{i18nMsg('reports-vat-column-base-ht', "Chiffre d'affaires HT")}</th
					>
					<th class="px-2 py-1 text-right">{i18nMsg('reports-vat-column-vat-due', 'TVA due')}</th>
				</tr>
			</thead>
			<tbody>
				{#each dto.rows as row (row.rate)}
					<tr>
						<td class="px-2 py-1 font-mono">{fmtRate(row.rate)}</td>
						<td class="px-2 py-1 text-right font-mono">{fmt(row.baseHt)}</td>
						<td class="px-2 py-1 text-right font-mono">{fmt(row.vatDue)}</td>
					</tr>
				{/each}
			</tbody>
			<tfoot>
				<tr class="border-t font-semibold">
					<td class="px-2 py-1">{i18nMsg('reports-vat-total-base-ht', 'Total CA HT')}</td>
					<td class="px-2 py-1 text-right font-mono">{fmt(dto.totalBaseHt)}</td>
					<td class="px-2 py-1 text-right font-mono">{fmt(dto.totalVatDue)}</td>
				</tr>
				<tr>
					<td colspan="2" class="px-2 py-1"
						>{i18nMsg('reports-vat-recoverable', 'TVA récupérable')}</td
					>
					<td class="px-2 py-1 text-right font-mono">{fmt(dto.totalVatRecoverable)}</td>
				</tr>
				<tr class="border-t font-semibold">
					<td colspan="2" class="px-2 py-1">{i18nMsg('reports-vat-balance', 'Solde')}</td>
					<td class="px-2 py-1 text-right font-mono">{fmt(dto.vatBalance)}</td>
				</tr>
			</tfoot>
		</table>
	{/if}
</section>
