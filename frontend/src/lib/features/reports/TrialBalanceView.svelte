<script lang="ts">
	// Story 9-1 — Vue Balance des comptes.
	// Story 25-5-b (#385) — ouverture, mouvements, clôture, et la ligne calculée du
	// résultat reporté : la clôture d'un compte de bilan est son solde au bilan.
	// Story 46-6b — avec `groupedRows` (réponse demandée avec `detailLevel`), le corps
	// rend les lignes groupées par le composant partagé du bilan (quatre colonnes,
	// seconde ligne en devise d'un compte ou d'un groupe en devise unique) ; sans, la
	// vue à plat d'avant, inchangée. En-tête, ligne du résultat reporté (après les
	// groupes, hors d'eux), totaux et contrôles restent ceux de la vue à plat.
	import Big from 'big.js';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import AccountLink from './AccountLink.svelte';
	import GroupedReportRows, { TRIAL_BALANCE_COLUMNS } from './GroupedReportRows.svelte';
	import {
		formatCurrencyFigure,
		formatReportAmount,
		isReportEmpty,
	} from './reports.api';
	import ReportPeriodHeading from './ReportPeriodHeading.svelte';
	import type { ExerciceBornes } from './reports-url';
	import type { TrialBalanceDto, TrialBalanceGroupedRow } from './reports.types';

	/** Ce que la seconde ligne en devise affiche : une ligne de compte ou de groupe. */
	type CurrencyAmounts = Pick<
		TrialBalanceGroupedRow,
		'openingCurrency' | 'totalDebitCurrency' | 'totalCreditCurrency' | 'closingCurrency'
	>;

	interface Props {
		dto: TrialBalanceDto;
		/** Story 46-6b — clic sur l'accordéon d'un groupe (vue groupée seulement). */
		onToggleGroup?: (row: TrialBalanceGroupedRow) => void;
		/** Story 55-1a (AC-17) — les exercices de la société, pour l'en-tête de la période. */
		fiscalYears?: readonly ExerciceBornes[];
	}
	let { dto, onToggleGroup = () => {}, fiscalYears = [] }: Props = $props();
	let empty = $derived(isReportEmpty('trial-balance', dto));

	// Story 46-6b — vue groupée si la réponse en porte les lignes.
	let grouped = $derived(dto.groupedRows && dto.grouping ? dto.groupedRows : null);
	let subtotalPosition = $derived(dto.grouping?.subtotalPosition ?? 'header');

	/** Le texte de la seconde ligne en devise, après le code de la devise (Story 37-4). */
	function fxText(r: CurrencyAmounts): string {
		return i18nMsg(
			'reports-fx-trial-balance',
			'ouverture { $opening } · débit { $debit } · crédit { $credit } · clôture { $closing }',
			{
				opening: formatCurrencyFigure(r.openingCurrency),
				debit: formatCurrencyFigure(r.totalDebitCurrency),
				credit: formatCurrencyFigure(r.totalCreditCurrency),
				closing: formatCurrencyFigure(r.closingCurrency),
			},
		);
	}

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
	<!-- Story 55-1a (AC-17) — la période réelle de la réponse, et l'exercice dont elle est l'entier ou une partie. -->
	<header>
		<ReportPeriodHeading from={dto.period.startDate} to={dto.period.endDate} {fiscalYears} />
	</header>

	{#if empty}
		<p class="rounded bg-blue-50 p-4 text-blue-900" role="status">
			{i18nMsg('reports-error-no-entries-in-period', 'Aucune écriture sur cette période.')}
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
				{#if grouped}
					<GroupedReportRows
						rows={grouped}
						{subtotalPosition}
						columns={TRIAL_BALANCE_COLUMNS}
						period={dto.period}
						onToggle={onToggleGroup}
						archivedBadge
					>
						{#snippet currencyRow(row: TrialBalanceGroupedRow)}
							<!-- Même seconde ligne qu'un compte de la vue à plat ; celle d'un groupe
							     en devise unique se distingue par son identifiant de test. -->
							<tr
								class="text-xs text-gray-600"
								data-testid={row.kind === 'account' || row.kind === 'own'
									? `tb-fx-row-${row.accountNumber}`
									: `tb-group-fx-row-${row.kind}-${row.accountNumber}`}
							>
								<td></td>
								<td class="px-2 pb-1 font-mono" colspan="5"
									>{row.currency}
									{fxText(row)}</td
								>
							</tr>
						{/snippet}
					</GroupedReportRows>
				{:else}
				{#each dto.rows as r (r.accountId)}
					<tr class:opacity-60={!r.active} data-testid="tb-row">
						<td class="px-2 py-1 font-mono">
							<AccountLink accountId={r.accountId} number={r.accountNumber} from={dto.period.startDate} to={dto.period.endDate} />
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
					{#if r.currency}
						<!-- Story 37-4 (AC-3) — seconde ligne, en devise du compte ; aucun total
						     en devise (devises mêlées). -->
						<tr class="text-xs text-gray-600" data-testid="tb-fx-row-{r.accountNumber}">
							<td></td>
							<td class="px-2 pb-1 font-mono" colspan="5"
								>{r.currency}
								{fxText(r)}</td
							>
						</tr>
					{/if}
				{/each}
				{/if}
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
