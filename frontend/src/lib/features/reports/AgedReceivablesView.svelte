<script lang="ts">
	// Story 21-7 — Vue Balance âgée débiteurs.
	// Namespace i18n : clés `reports-*` uniquement (composant sous features/reports/).
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { formatReportAmount, formatSwissDate } from './reports.api';
	import type { AgedReceivablesDto } from './reports.types';

	interface Props {
		dto: AgedReceivablesDto;
	}
	let { dto }: Props = $props();

	const fmt = formatReportAmount;
	let empty = $derived(dto.rows.length === 0);
</script>

<section class="space-y-4">
	<header class="flex items-center justify-between text-sm text-gray-600">
		<div>
			<strong>{i18nMsg('reports-aged-as-of', 'Arrêté au { $date }', { date: formatSwissDate(dto.asOf) })}</strong>
		</div>
		<!-- Lien croisé : balance âgée → échéancier (namespace reports-*). -->
		<a class="text-primary underline" href="/invoices/due-dates" data-testid="aged-link-due-dates">
			{i18nMsg('reports-aged-link-due-dates', "Voir l'échéancier")}
		</a>
	</header>

	{#if empty}
		<p class="rounded bg-blue-50 p-4 text-blue-900" role="status">
			{i18nMsg('reports-aged-empty', 'Aucune créance client ouverte.')}
		</p>
	{:else}
		<table class="w-full border-collapse" data-testid="aged-receivables-table">
			<thead>
				<tr class="border-b bg-gray-50 text-left text-sm">
					<th class="px-2 py-1">{i18nMsg('reports-aged-col-contact', 'Client')}</th>
					<th class="px-2 py-1 text-right">{i18nMsg('reports-aged-col-not-due', 'Non échu')}</th>
					<th class="px-2 py-1 text-right">{i18nMsg('reports-aged-col-1-30', '1-30 j')}</th>
					<th class="px-2 py-1 text-right">{i18nMsg('reports-aged-col-31-60', '31-60 j')}</th>
					<th class="px-2 py-1 text-right">{i18nMsg('reports-aged-col-61-90', '61-90 j')}</th>
					<th class="px-2 py-1 text-right">{i18nMsg('reports-aged-col-over-90', '90+ j')}</th>
					<th class="px-2 py-1 text-right">{i18nMsg('reports-aged-col-total', 'Total')}</th>
				</tr>
			</thead>
			<tbody>
				{#each dto.rows as row (row.contactId)}
					<tr class="border-b" data-testid="aged-receivables-row">
						<td class="px-2 py-1">
							<!-- Drill-down : créances du contact → liste factures filtrée (?contactId=). -->
							<a class="text-primary underline" href="/invoices?contactId={row.contactId}">
								{row.contactName}
							</a>
						</td>
						<td class="px-2 py-1 text-right font-mono">{fmt(row.notDue)}</td>
						<td class="px-2 py-1 text-right font-mono">{fmt(row.days1To30)}</td>
						<td class="px-2 py-1 text-right font-mono">{fmt(row.days31To60)}</td>
						<td class="px-2 py-1 text-right font-mono">{fmt(row.days61To90)}</td>
						<td class="px-2 py-1 text-right font-mono">{fmt(row.daysOver90)}</td>
						<td class="px-2 py-1 text-right font-mono font-semibold">{fmt(row.total)}</td>
					</tr>
				{/each}
			</tbody>
			<tfoot>
				<tr class="border-t font-semibold" data-testid="aged-receivables-total">
					<td class="px-2 py-1">{i18nMsg('reports-aged-total-row', 'Total général')}</td>
					<td class="px-2 py-1 text-right font-mono">{fmt(dto.totals.notDue)}</td>
					<td class="px-2 py-1 text-right font-mono">{fmt(dto.totals.days1To30)}</td>
					<td class="px-2 py-1 text-right font-mono">{fmt(dto.totals.days31To60)}</td>
					<td class="px-2 py-1 text-right font-mono">{fmt(dto.totals.days61To90)}</td>
					<td class="px-2 py-1 text-right font-mono">{fmt(dto.totals.daysOver90)}</td>
					<td class="px-2 py-1 text-right font-mono">{fmt(dto.totals.total)}</td>
				</tr>
			</tfoot>
		</table>
	{/if}
</section>
