<!--
  Story 19-6b — Vue « Rendement par projet ».
  Tableau par section (sous-projet) : coût investi / revenus / résultat net /
  rendement %, avec ligne total.
-->
<script lang="ts">
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { formatReportAmount } from './reports.api';
	import type { ProjectReturnDto } from './reports.types';

	interface Props {
		report: ProjectReturnDto;
	}
	let { report }: Props = $props();

	function fmtPct(pct: string | null): string {
		return pct === null ? '—' : `${Number(pct).toFixed(2)}%`;
	}
</script>

<div data-testid="project-return-view">
	<h2 class="text-lg font-semibold">
		{i18nMsg('reports-project-return-title', 'Rendement par projet')} — {report.project.code}
		{report.project.name}
	</h2>
	<p class="mb-3 text-sm text-text-muted" data-testid="project-return-period">{report.periodLabel}</p>

	{#if report.sections.length === 0}
		<p class="rounded border border-border bg-surface px-3 py-2 text-sm" data-testid="project-return-empty">
			{i18nMsg('reports-project-return-empty', 'Aucun mouvement tagué sur ce projet pour la période.')}
		</p>
	{:else}
		<div class="overflow-x-auto">
			<table class="w-full text-sm" data-testid="project-return-table">
				<thead>
					<tr class="border-b border-border text-left text-xs text-text-muted">
						<th class="py-1">{i18nMsg('reports-project-return-col-project', 'Projet')}</th>
						<th class="py-1 text-right">{i18nMsg('reports-project-return-col-cost', 'Coût investi')}</th>
						<th class="py-1 text-right">{i18nMsg('reports-project-return-col-revenue', 'Revenus')}</th>
						<th class="py-1 text-right">{i18nMsg('reports-project-return-col-net', 'Résultat net')}</th>
						<th class="py-1 text-right">{i18nMsg('reports-project-return-col-return', 'Rendement')}</th>
					</tr>
				</thead>
				<tbody>
					{#each report.sections as section (section.project.id)}
						<tr class="border-b border-border/50" data-testid={`project-return-section-${section.project.id}`}>
							<td class="py-1">
								{#if section.isRoot}
									{section.project.code} — {section.project.name}
								{:else}
									&nbsp;&nbsp;↳ {section.project.code} — {section.project.name}
								{/if}
							</td>
							<td class="py-1 text-right tabular-nums">{formatReportAmount(section.coutInvesti)}</td>
							<td class="py-1 text-right tabular-nums">{formatReportAmount(section.revenus)}</td>
							<td class="py-1 text-right tabular-nums">{formatReportAmount(section.resultatNet)}</td>
							<td class="py-1 text-right tabular-nums">{fmtPct(section.rendementPct)}</td>
						</tr>
					{/each}
					<tr class="font-semibold">
						<td class="py-2 text-right">{i18nMsg('reports-project-return-total', 'Total')}</td>
						<td class="py-2 text-right tabular-nums" data-testid="project-return-total-cost">{formatReportAmount(report.totals.coutInvesti)}</td>
						<td class="py-2 text-right tabular-nums">{formatReportAmount(report.totals.revenus)}</td>
						<td class="py-2 text-right tabular-nums">{formatReportAmount(report.totals.resultatNet)}</td>
						<td class="py-2 text-right tabular-nums" data-testid="project-return-total-pct">{fmtPct(report.totals.rendementPct)}</td>
					</tr>
				</tbody>
			</table>
		</div>
	{/if}
</div>
