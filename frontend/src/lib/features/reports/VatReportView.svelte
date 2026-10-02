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

	<!-- Story 18-1e : bandeau de réconciliation (INFO non bloquant). Indépendant du
	     flag `empty` ; visible uniquement si la TVA due dérivée ne correspond pas au
	     solde du compte TVA due au grand livre (écriture validée modifiée à la main). -->
	{#if dto.reconciliationStatus === 'delta'}
		<p class="rounded bg-amber-50 p-3 text-sm text-amber-900" role="alert">
			{i18nMsg(
				'reports-vat-reconciliation-warning',
				'Le décompte ne correspond pas aux écritures comptables (écart : { $delta }). Vérifiez les écritures validées modifiées manuellement.',
				{ delta: fmt(dto.reconciliationDelta) }
			)}
		</p>
	{/if}

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
					<td class="px-2 py-1 text-right font-mono" data-testid="vat-total-vat-due">
						{fmt(dto.totalVatDue)}
					</td>
				</tr>
				<!-- Story 25-4-d2c : les diminutions de contre-prestation (soldes) —
				     seulement s'il y en a dans la période. -->
				{#if dto.writeOffRows.length > 0}
					<tr class="border-t" data-testid="vat-write-off-section">
						<td colspan="3" class="px-2 py-1 font-semibold">
							{i18nMsg(
								'reports-vat-write-off-title',
								'Diminutions de contre-prestation (soldes)',
							)}
						</td>
					</tr>
					{#each dto.writeOffRows as row (row.rate)}
						<tr data-testid="vat-write-off-row">
							<td class="px-2 py-1 font-mono">{fmtRate(row.rate)}</td>
							<td class="px-2 py-1 text-right font-mono">{fmt(row.baseHt)}</td>
							<td class="px-2 py-1 text-right font-mono">{fmt(row.vat)}</td>
						</tr>
					{/each}
					<tr>
						<td colspan="2" class="px-2 py-1"
							>{i18nMsg('reports-vat-total-write-off', 'Total TVA des soldes')}</td
						>
						<td class="px-2 py-1 text-right font-mono">{fmt(dto.totalVatWriteOff)}</td>
					</tr>
					<tr class="font-semibold">
						<td colspan="2" class="px-2 py-1"
							>{i18nMsg('reports-vat-due-net', 'TVA due nette')}</td
						>
						<td class="px-2 py-1 text-right font-mono" data-testid="vat-total-vat-due-net">
							{fmt(dto.totalVatDueNet)}
						</td>
					</tr>
				{/if}
				<tr>
					<td colspan="2" class="px-2 py-1"
						>{i18nMsg('reports-vat-recoverable', 'TVA récupérable')}</td
					>
					<td class="px-2 py-1 text-right font-mono">{fmt(dto.totalVatRecoverable)}</td>
				</tr>
				<tr class="border-t font-semibold">
					<td colspan="2" class="px-2 py-1">{i18nMsg('reports-vat-balance', 'Solde')}</td>
					<td class="px-2 py-1 text-right font-mono" data-testid="vat-balance">
						{fmt(dto.vatBalance)}
					</td>
				</tr>
			</tfoot>
		</table>
	{/if}
</section>
