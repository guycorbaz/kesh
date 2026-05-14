<script lang="ts">
	// Story 9-1 — Sélecteur d'exercice + période + bouton Générer.
	// Pass 1 ECH-12 + AC #34 : si fiscal_years vide → dropdown vide + bouton disabled.

	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { formatSwissDate } from './reports.api';
	import type { FiscalYearResponse } from '$lib/features/fiscal-years/fiscal-years.types';

	interface Props {
		fiscalYears: FiscalYearResponse[];
		selectedFiscalYearId: number | null;
		periodStart: string;
		periodEnd: string;
		loading: boolean;
		onGenerate: () => void;
	}

	let {
		fiscalYears,
		selectedFiscalYearId = $bindable(),
		periodStart = $bindable(),
		periodEnd = $bindable(),
		loading,
		onGenerate,
	}: Props = $props();

	let noFiscalYears = $derived(fiscalYears.length === 0);
	let canGenerate = $derived(!noFiscalYears && selectedFiscalYearId !== null && !loading);
</script>

<div class="space-y-4 rounded border bg-white p-4 shadow-sm">
	<div class="grid gap-4 sm:grid-cols-3">
		<label class="block">
			<span class="text-sm font-medium text-gray-700"
				>{i18nMsg('reports-filter-fiscal-year', 'Exercice')}</span
			>
			<select
				class="mt-1 block w-full rounded border-gray-300 shadow-sm"
				bind:value={selectedFiscalYearId}
				disabled={noFiscalYears || loading}
				aria-label={i18nMsg('reports-filter-fiscal-year', 'Exercice')}
			>
				{#each fiscalYears as fy (fy.id)}
					<option value={fy.id}
						>{fy.name} ({formatSwissDate(fy.startDate)} → {formatSwissDate(fy.endDate)})</option
					>
				{/each}
			</select>
		</label>

		<label class="block">
			<span class="text-sm font-medium text-gray-700"
				>{i18nMsg('reports-filter-period', 'Période')} — début</span
			>
			<input
				type="date"
				class="mt-1 block w-full rounded border-gray-300 shadow-sm"
				bind:value={periodStart}
				disabled={noFiscalYears || loading}
			/>
		</label>

		<label class="block">
			<span class="text-sm font-medium text-gray-700"
				>{i18nMsg('reports-filter-period', 'Période')} — fin</span
			>
			<input
				type="date"
				class="mt-1 block w-full rounded border-gray-300 shadow-sm"
				bind:value={periodEnd}
				disabled={noFiscalYears || loading}
			/>
		</label>
	</div>

	{#if noFiscalYears}
		<p class="rounded bg-amber-50 p-3 text-sm text-amber-900" role="alert">
			{i18nMsg(
				'reports-error-no-fiscal-year-available',
				'Aucun exercice comptable disponible. Créez un exercice avant de générer des rapports.',
			)}
		</p>
	{/if}

	<button
		type="button"
		class="rounded bg-indigo-600 px-4 py-2 text-white shadow-sm hover:bg-indigo-700 disabled:cursor-not-allowed disabled:bg-gray-400"
		disabled={!canGenerate}
		onclick={onGenerate}
	>
		{i18nMsg('reports-button-generate', 'Générer')}
	</button>
</div>
