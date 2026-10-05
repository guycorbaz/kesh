<script lang="ts">
	// Story 9-1 — Vue Balance des comptes.
	// Story 25-5-b (#385) — ouverture, mouvements, clôture, et la ligne calculée du
	// résultat reporté : la clôture d'un compte de bilan est son solde au bilan.
	import Big from 'big.js';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { formatReportAmount, formatSwissDate, isReportEmpty, ledgerHref } from './reports.api';
	import type { TrialBalanceDto } from './reports.types';

	interface Props {
		dto: TrialBalanceDto;
	}
	let { dto }: Props = $props();
	let empty = $derived(isReportEmpty('trial-balance', dto));

	// « Perte reportée » si négatif, comme au bilan — l'écran seul bascule.
	let retainedLabel = $derived.by(() => {
		try {
			if (new Big(dto.retainedEarnings).lt(0)) {
				return i18nMsg('reports-retained-earnings-loss', 'Perte reportée');
			}
		} catch {
			// Montant illisible : libellé neutre.
		}
		return i18nMsg('reports-retained-earnings-calculated', 'Résultat reporté (calculé)');
	});

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
		<table class="w-full border-collapse">
			<thead>
				<tr class="border-b bg-gray-50 text-left text-sm">
					<th class="px-2 py-1">{i18nMsg('reports-column-account-number', 'N°')}</th>
					<th class="px-2 py-1">{i18nMsg('reports-column-account-name', 'Intitulé')}</th>
					<th class="px-2 py-1 text-right" data-testid="tb-col-opening"
						>{i18nMsg('reports-column-opening', 'Ouverture')}</th
					>
					<th class="px-2 py-1 text-right">{i18nMsg('reports-column-debit', 'Débit')}</th>
					<th class="px-2 py-1 text-right">{i18nMsg('reports-column-credit', 'Crédit')}</th>
					<th class="px-2 py-1 text-right" data-testid="tb-col-closing"
						>{i18nMsg('reports-column-closing', 'Clôture')}</th
					>
				</tr>
			</thead>
			<tbody>
				{#each dto.rows as r (r.accountId)}
					<tr class:opacity-60={!r.active} data-testid="tb-row">
						<td class="px-2 py-1 font-mono">
							<a
								class="text-indigo-700 hover:underline"
								href={ledgerHref(r.accountId, dto.period.startDate, dto.period.endDate)}
								title={i18nMsg('reports-ledger-open-from-balance', 'Voir le détail dans le grand livre')}
							>{r.accountNumber}</a
							>
						</td>
						<td class="px-2 py-1">
							{r.accountName}
							{#if !r.active}<span class="ml-1 rounded bg-gray-200 px-1 text-xs"
									>{i18nMsg('reports-archived-label', 'archivé')}</span
								>{/if}
						</td>
						<td class="px-2 py-1 text-right font-mono">{fmt(r.openingBalance)}</td>
						<td class="px-2 py-1 text-right font-mono">{fmt(r.totalDebit)}</td>
						<td class="px-2 py-1 text-right font-mono">{fmt(r.totalCredit)}</td>
						<td class="px-2 py-1 text-right font-mono">{fmt(r.closingBalance)}</td>
					</tr>
				{/each}
				<tr class="italic" data-testid="tb-retained">
					<td></td>
					<td class="px-2 py-1">{retainedLabel}</td>
					<td class="px-2 py-1 text-right font-mono">{fmt(dto.retainedEarnings)}</td>
					<td></td>
					<td></td>
					<td class="px-2 py-1 text-right font-mono">{fmt(dto.retainedEarnings)}</td>
				</tr>
			</tbody>
			<tfoot>
				<tr class="border-t font-semibold">
					<td colspan="2" class="px-2 py-1">{i18nMsg('reports-grand-total', 'Total général')}</td>
					<td
						class="px-2 py-1 text-right"
						class:text-red-700={!dto.openingBalanced}
						data-testid="tb-opening-check"
					>
						{dto.openingBalanced ? '✓' : '⚠️'}
					</td>
					<td class="px-2 py-1 text-right font-mono">{fmt(dto.totalDebit)}</td>
					<td class="px-2 py-1 text-right font-mono">{fmt(dto.totalCredit)}</td>
					<td
						class="px-2 py-1 text-right"
						class:text-red-700={!dto.balanced}
						data-testid="tb-movements-check"
					>
						{dto.balanced ? '✓' : '⚠️'}
					</td>
				</tr>
			</tfoot>
		</table>
	{/if}
</section>
