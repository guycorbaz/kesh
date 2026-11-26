<script lang="ts">
	// Story 9-1 — Vue Compte de résultat.
	// Story 55-2 (AC-2) — à plat aussi, le numéro de chaque compte ouvre son grand livre (`AccountLink`).
	// Story 46-2 — avec `groupedRows`, Produits et Charges rendent leurs lignes
	// groupées (lien vers le grand livre sur les comptes, AC-4) ; sans, la vue à
	// plat d'avant, inchangée.
	import Big from 'big.js';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import AccountCurrencyBalance from './AccountCurrencyBalance.svelte';
	import AccountLink from './AccountLink.svelte';
	import GroupedReportRows, { BALANCE_COLUMNS } from './GroupedReportRows.svelte';
	import { formatReportAmount, isReportEmpty } from './reports.api';
	import ReportPeriodHeading from './ReportPeriodHeading.svelte';
	import type { ExerciceBornes } from './reports-url';
	import type { GroupedRow, IncomeStatementDto } from './reports.types';

	interface Props {
		dto: IncomeStatementDto;
		/** Story 46-2 — clic sur l'accordéon d'un groupe (vue groupée seulement). */
		onToggleGroup?: (row: GroupedRow) => void;
		/** Story 55-1a (AC-17) — les exercices de la société, pour l'en-tête de la période. */
		fiscalYears?: readonly ExerciceBornes[];
	}
	let { dto, onToggleGroup = () => {}, fiscalYears = [] }: Props = $props();

	let grouped = $derived(dto.groupedRows && dto.grouping ? dto.groupedRows : null);
	let subtotalPosition = $derived(dto.grouping?.subtotalPosition ?? 'header');
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
	<!-- Story 55-1a (AC-17) — la période réelle de la réponse, et l'exercice dont elle est l'entier ou une partie. -->
	<header>
		<ReportPeriodHeading from={dto.period.startDate} to={dto.period.endDate} {fiscalYears} />
	</header>

	{#if empty}
		<p class="rounded bg-blue-50 p-4 text-blue-900" role="status">
			{i18nMsg('reports-error-no-entries-in-period', 'Aucune écriture sur cette période.')}
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
						{#if grouped}
							<GroupedReportRows
								rows={grouped.revenues}
								{subtotalPosition}
								columns={BALANCE_COLUMNS}
								period={dto.period}
								onToggle={onToggleGroup}
							/>
						{:else}
						{#each dto.revenues as r (r.accountId)}
							<tr>
								<td class="px-2 py-1 font-mono"
									><AccountLink accountId={r.accountId} number={r.accountNumber} from={dto.period.startDate} to={dto.period.endDate} /></td
								>
								<td class="px-2 py-1">{r.accountName}<AccountCurrencyBalance account={r} /></td>
								<td class="px-2 py-1 text-right font-mono">{fmt(r.balance)}</td>
							</tr>
						{/each}
						{/if}
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
						{#if grouped}
							<GroupedReportRows
								rows={grouped.expenses}
								{subtotalPosition}
								columns={BALANCE_COLUMNS}
								period={dto.period}
								onToggle={onToggleGroup}
							/>
						{:else}
						{#each dto.expenses as e (e.accountId)}
							<tr>
								<td class="px-2 py-1 font-mono"
									><AccountLink accountId={e.accountId} number={e.accountNumber} from={dto.period.startDate} to={dto.period.endDate} /></td
								>
								<td class="px-2 py-1">{e.accountName}<AccountCurrencyBalance account={e} /></td>
								<td class="px-2 py-1 text-right font-mono">{fmt(e.balance)}</td>
							</tr>
						{/each}
						{/if}
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
