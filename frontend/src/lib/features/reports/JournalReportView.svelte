<script lang="ts">
	// Story 9-1 — Vue Journaux.
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { formatRate } from '$lib/features/exchange-rates/exchange-rates-grid';
	import {
		formatCurrencyAmount,
		formatReportAmount,
		formatSwissDate,
		isReportEmpty,
	} from './reports.api';
	import type { JournalEntryLineRow, JournalReportDto } from './reports.types';

	interface Props {
		dto: JournalReportDto;
	}
	let { dto }: Props = $props();
	let empty = $derived(isReportEmpty('journals', dto));

	const fmt = formatReportAmount;

	/**
	 * Story 37-4 (AC-7) — la partie en devise d'une ligne : « USD 31.35 × 0.8808 »,
	 * ou « Réévaluation au cours X », sans montant (comme au grand livre).
	 */
	function fxLine(currency: string, l: JournalEntryLineRow): string {
		const rate = formatRate(l.exchangeRate);
		if (l.rateSource === 'revaluation') {
			return i18nMsg('reports-fx-revaluation', 'Réévaluation au cours { $rate }', { rate });
		}
		return i18nMsg('reports-fx-line', '{ $amount } × { $rate }', {
			amount: formatCurrencyAmount(currency, l.amountCurrency),
			rate,
		});
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
											{#if l.currency}
												<!-- Story 37-4 (AC-7) — la partie en devise, écran seulement (le
												     PDF et le CSV du journal restent inchangés). -->
												<tr class="text-xs text-gray-600" data-testid="journal-fx-line">
													<td></td>
													<td class="font-mono" colspan="3">{fxLine(l.currency, l)}</td>
												</tr>
											{/if}
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
