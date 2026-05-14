<script lang="ts">
	// Story 9-1 — Page rapports comptables.
	// AC #33 (Pass 1 AA-10) : 4 onglets (Bilan, Résultat, Balance, Journaux).

	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import ReportSelector from '$lib/features/reports/ReportSelector.svelte';
	import BalanceSheetView from '$lib/features/reports/BalanceSheetView.svelte';
	import IncomeStatementView from '$lib/features/reports/IncomeStatementView.svelte';
	import TrialBalanceView from '$lib/features/reports/TrialBalanceView.svelte';
	import JournalReportView from '$lib/features/reports/JournalReportView.svelte';
	import {
		getBalanceSheet,
		getIncomeStatement,
		getJournalReport,
		getTrialBalance,
	} from '$lib/features/reports/reports.api';
	import type {
		BalanceSheetDto,
		IncomeStatementDto,
		JournalReportDto,
		ReportType,
		TrialBalanceDto,
	} from '$lib/features/reports/reports.types';
	import type { FiscalYearResponse } from '$lib/features/fiscal-years/fiscal-years.types';

	interface PageData {
		fiscalYears: FiscalYearResponse[];
	}
	let { data }: { data: PageData } = $props();

	let selectedFiscalYearId = $state<number | null>(data.fiscalYears[0]?.id ?? null);
	let periodStart = $state('');
	let periodEnd = $state('');
	let activeTab = $state<ReportType>('balance-sheet');
	let loading = $state(false);
	let errorMsg = $state<string | null>(null);

	let balanceSheet = $state<BalanceSheetDto | null>(null);
	let incomeStatement = $state<IncomeStatementDto | null>(null);
	let trialBalance = $state<TrialBalanceDto | null>(null);
	let journalReport = $state<JournalReportDto | null>(null);

	async function generate(): Promise<void> {
		if (selectedFiscalYearId === null) return;
		loading = true;
		errorMsg = null;
		const query = {
			fiscalYearId: selectedFiscalYearId,
			periodStart: periodStart || undefined,
			periodEnd: periodEnd || undefined,
		};
		try {
			switch (activeTab) {
				case 'balance-sheet':
					balanceSheet = await getBalanceSheet(query);
					break;
				case 'income-statement':
					incomeStatement = await getIncomeStatement(query);
					break;
				case 'trial-balance':
					trialBalance = await getTrialBalance(query);
					break;
				case 'journals':
					journalReport = await getJournalReport(query);
					break;
			}
		} catch (e) {
			errorMsg = (e as Error).message ?? 'Erreur';
		} finally {
			loading = false;
		}
	}

	const tabs: { id: ReportType; labelKey: string; fallback: string }[] = [
		{ id: 'balance-sheet', labelKey: 'reports-balance-sheet', fallback: 'Bilan' },
		{ id: 'income-statement', labelKey: 'reports-income-statement', fallback: 'Compte de résultat' },
		{ id: 'trial-balance', labelKey: 'reports-trial-balance', fallback: 'Balance' },
		{ id: 'journals', labelKey: 'reports-journals', fallback: 'Journaux' },
	];
</script>

<svelte:head>
	<title>Kesh — Rapports</title>
</svelte:head>

<div class="space-y-4 p-4">
	<h1 class="text-2xl font-bold">Rapports comptables</h1>

	<ReportSelector
		fiscalYears={data.fiscalYears}
		bind:selectedFiscalYearId
		bind:periodStart
		bind:periodEnd
		{loading}
		onGenerate={generate}
	/>

	<div role="tablist" class="flex border-b">
		{#each tabs as tab (tab.id)}
			<button
				type="button"
				role="tab"
				aria-selected={activeTab === tab.id}
				class="border-b-2 px-4 py-2 text-sm font-medium {activeTab === tab.id
					? 'border-indigo-600 text-indigo-700'
					: 'border-transparent text-gray-600 hover:text-gray-900'}"
				onclick={() => (activeTab = tab.id)}
			>
				{i18nMsg(tab.labelKey, tab.fallback)}
			</button>
		{/each}
	</div>

	{#if errorMsg}
		<p class="rounded bg-red-50 p-3 text-sm text-red-900" role="alert">{errorMsg}</p>
	{/if}

	<div role="tabpanel">
		{#if activeTab === 'balance-sheet' && balanceSheet}
			<BalanceSheetView dto={balanceSheet} />
		{:else if activeTab === 'income-statement' && incomeStatement}
			<IncomeStatementView dto={incomeStatement} />
		{:else if activeTab === 'trial-balance' && trialBalance}
			<TrialBalanceView dto={trialBalance} />
		{:else if activeTab === 'journals' && journalReport}
			<JournalReportView dto={journalReport} />
		{:else}
			<p class="text-sm italic text-gray-500">
				Sélectionnez un exercice et cliquez sur « {i18nMsg('reports-button-generate', 'Générer')} ».
			</p>
		{/if}
	</div>
</div>
