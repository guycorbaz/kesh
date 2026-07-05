<!--
  Story 19-6a — Vue « Dépenses par projet ».
  Tableau groupé par section (sous-projet) → compte, lignes de compte
  expandables vers les écritures contributrices (drill-down).
-->
<script lang="ts">
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { formatReportAmount, formatSwissDate } from './reports.api';
	import type { ProjectExpensesDto } from './reports.types';

	interface Props {
		report: ProjectExpensesDto;
	}
	let { report }: Props = $props();

	// Clés de lignes dépliées : `${sectionProjectId}:${accountId}`.
	let expanded = $state<Set<string>>(new Set());

	function toggle(key: string): void {
		const next = new Set(expanded);
		if (next.has(key)) next.delete(key);
		else next.add(key);
		expanded = next;
	}
</script>

<div data-testid="project-expenses-view">
	<h2 class="text-lg font-semibold">
		{i18nMsg('reports-project-expenses-title', 'Dépenses par projet')} — {report.project.code}
		{report.project.name}
	</h2>
	<p class="mb-3 text-sm text-text-muted" data-testid="project-expenses-period">
		{report.periodLabel}
	</p>

	{#if report.sections.length === 0}
		<p class="rounded border border-border bg-surface px-3 py-2 text-sm" data-testid="project-expenses-empty">
			{i18nMsg('reports-project-expenses-empty', 'Aucune dépense taguée sur ce projet pour la période.')}
		</p>
	{:else}
		<table class="w-full text-sm" data-testid="project-expenses-table">
			<thead>
				<tr class="border-b border-border text-left text-xs text-text-muted">
					<th class="py-1">{i18nMsg('reports-project-expenses-col-account', 'Compte')}</th>
					<th class="py-1 text-right">{i18nMsg('reports-project-expenses-col-amount', 'Montant')}</th>
				</tr>
			</thead>
			<tbody>
				{#each report.sections as section (section.project.id)}
					<tr class="bg-surface">
						<td colspan="2" class="py-1 font-medium" data-testid={`project-expenses-section-${section.project.id}`}>
							{#if section.isRoot}
								{section.project.code} — {section.project.name}
							{:else}
								&nbsp;&nbsp;↳ {section.project.code} — {section.project.name}
							{/if}
						</td>
					</tr>
					{#each section.rows as row (row.accountId)}
						{@const key = `${section.project.id}:${row.accountId}`}
						<tr class="border-b border-border/50">
							<td class="py-1">
								<button
									type="button"
									class="text-left underline-offset-2 hover:underline"
									onclick={() => toggle(key)}
									data-testid={`project-expenses-row-${section.project.id}-${row.accountId}`}
								>
									{expanded.has(key) ? '▾' : '▸'}
									{row.accountNumber} — {row.accountName}
								</button>
							</td>
							<td class="py-1 text-right tabular-nums">{formatReportAmount(row.amount)}</td>
						</tr>
						{#if expanded.has(key)}
							{#each row.entries as entry (entry.entryId)}
								<tr class="text-xs text-text-muted">
									<td class="py-0.5 pl-6" data-testid={`project-expenses-entry-${entry.entryId}`}>
										#{entry.entryNumber} — {formatSwissDate(entry.entryDate)} — {entry.description}
									</td>
									<td class="py-0.5 text-right tabular-nums">{formatReportAmount(entry.amount)}</td>
								</tr>
							{/each}
						{/if}
					{/each}
					<tr class="border-b border-border font-medium">
						<td class="py-1 text-right">{i18nMsg('reports-project-expenses-subtotal', 'Sous-total')}</td>
						<td class="py-1 text-right tabular-nums">{formatReportAmount(section.subtotal)}</td>
					</tr>
				{/each}
				<tr class="font-semibold">
					<td class="py-2 text-right">{i18nMsg('reports-project-expenses-total', 'Total dépenses')}</td>
					<td class="py-2 text-right tabular-nums" data-testid="project-expenses-grand-total">
						{formatReportAmount(report.grandTotal)}
					</td>
				</tr>
			</tbody>
		</table>
	{/if}
</div>
