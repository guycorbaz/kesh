<script lang="ts">
	// Story 9-1 — Vue Journaux.
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { formatReportAmount, formatSwissDate, isReportEmpty } from './reports.api';
	import type { JournalReportDto } from './reports.types';

	interface Props {
		dto: JournalReportDto;
	}
	let { dto }: Props = $props();
	let empty = $derived(isReportEmpty('journals', dto));

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
		{#each dto.journals as section (section.journal)}
			<details open={section.entries.length > 0} class="rounded border bg-white">
				<summary class="cursor-pointer border-b bg-gray-50 px-3 py-2 font-semibold">
					{section.journal} ({section.entries.length})
				</summary>
				<div class="p-3">
					{#if section.entries.length === 0}
						<p class="text-sm italic text-gray-500">—</p>
					{:else}
						{#each section.entries as entry (entry.entryId)}
							<div class="mb-3 border-b pb-2 last:border-b-0">
								<div class="flex justify-between text-sm">
									<span class="font-mono">#{entry.entryNumber}</span>
									<span>{formatSwissDate(entry.entryDate)}</span>
								</div>
								<div class="text-sm text-gray-700">{entry.description}</div>
								<table class="mt-1 w-full text-sm">
									<tbody>
										{#each entry.lines as l (l.accountId + '_' + l.lineOrder)}
											<tr>
												<td class="font-mono">{l.accountNumber}</td>
												<td>{l.accountName}</td>
												<td class="text-right font-mono">{fmt(l.debit)}</td>
												<td class="text-right font-mono">{fmt(l.credit)}</td>
											</tr>
										{/each}
									</tbody>
								</table>
							</div>
						{/each}
					{/if}
					<div class="mt-2 flex justify-end gap-4 text-sm font-semibold">
						<span
							>{i18nMsg('reports-total-debit', 'Total débit')}: {fmt(
								section.sectionTotalDebit,
							)}</span
						>
						<span
							>{i18nMsg('reports-total-credit', 'Total crédit')}: {fmt(
								section.sectionTotalCredit,
							)}</span
						>
					</div>
				</div>
			</details>
		{/each}

		<p class="rounded border bg-gray-50 p-3 text-right font-semibold">
			{i18nMsg('reports-grand-total', 'Total général')}:
			{i18nMsg('reports-total-debit', 'Total débit')} = {fmt(dto.grandTotalDebit)} |
			{i18nMsg('reports-total-credit', 'Total crédit')} = {fmt(dto.grandTotalCredit)}
		</p>
	{/if}
</section>
