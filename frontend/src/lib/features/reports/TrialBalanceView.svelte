<script lang="ts">
	// Story 9-1 — Vue Balance des comptes.
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { formatReportAmount, formatSwissDate, isReportEmpty } from './reports.api';
	import type { TrialBalanceDto } from './reports.types';

	interface Props {
		dto: TrialBalanceDto;
	}
	let { dto }: Props = $props();
	let empty = $derived(isReportEmpty('trial-balance', dto));

	const fmt = formatReportAmount;
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
		<p class="rounded bg-amber-50 p-3 text-sm text-amber-900" role="note">
			{i18nMsg(
				'reports-trial-balance-period-note',
				'La balance de vérification affiche le mouvement de la période (par exercice). Le total par compte n’est pas comparable au solde cumulé du même compte au bilan (report à-nouveau depuis l’origine).',
			)}
		</p>
		<table class="w-full border-collapse">
			<thead>
				<tr class="border-b bg-gray-50 text-left text-sm">
					<th class="px-2 py-1">{i18nMsg('reports-column-account-number', 'N°')}</th>
					<th class="px-2 py-1">{i18nMsg('reports-column-account-name', 'Intitulé')}</th>
					<th class="px-2 py-1 text-right">{i18nMsg('reports-column-debit', 'Débit')}</th>
					<th class="px-2 py-1 text-right">{i18nMsg('reports-column-credit', 'Crédit')}</th>
					<th class="px-2 py-1 text-right">{i18nMsg('reports-column-balance', 'Solde')}</th>
				</tr>
			</thead>
			<tbody>
				{#each dto.rows as r (r.accountId)}
					<tr class:opacity-60={!r.active}>
						<td class="px-2 py-1 font-mono">{r.accountNumber}</td>
						<td class="px-2 py-1">
							{r.accountName}
							{#if !r.active}<span class="ml-1 rounded bg-gray-200 px-1 text-xs"
									>{i18nMsg('reports-archived-label', 'archivé')}</span
								>{/if}
						</td>
						<td class="px-2 py-1 text-right font-mono">{fmt(r.totalDebit)}</td>
						<td class="px-2 py-1 text-right font-mono">{fmt(r.totalCredit)}</td>
						<td class="px-2 py-1 text-right font-mono">{fmt(r.balance)}</td>
					</tr>
				{/each}
			</tbody>
			<tfoot>
				<tr class="border-t font-semibold">
					<td colspan="2" class="px-2 py-1">{i18nMsg('reports-grand-total', 'Total général')}</td>
					<td class="px-2 py-1 text-right font-mono">{fmt(dto.totalDebit)}</td>
					<td class="px-2 py-1 text-right font-mono">{fmt(dto.totalCredit)}</td>
					<td class="px-2 py-1 text-right" class:text-red-700={!dto.balanced}>
						{dto.balanced ? '✓' : '⚠️'}
					</td>
				</tr>
			</tfoot>
		</table>
	{/if}
</section>
