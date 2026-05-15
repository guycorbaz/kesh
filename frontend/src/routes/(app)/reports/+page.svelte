<script lang="ts">
	// Story 9-1 — Page rapports comptables.
	// AC #33 (Pass 1 AA-10) : 4 onglets (Bilan, Résultat, Balance, Journaux).
	// Code review Pass 1 patches : P5 (error handler isApiError + Fluent), P6 (ARIA tabs),
	// P17 (reset dates on FY change), P19 (loading indicator), P20 (race guard).

	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { isApiError } from '$lib/shared/utils/api-client';
	import ReportSelector from '$lib/features/reports/ReportSelector.svelte';
	import BalanceSheetView from '$lib/features/reports/BalanceSheetView.svelte';
	import IncomeStatementView from '$lib/features/reports/IncomeStatementView.svelte';
	import TrialBalanceView from '$lib/features/reports/TrialBalanceView.svelte';
	import JournalReportView from '$lib/features/reports/JournalReportView.svelte';
	import {
		formatSwissDate,
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

	// P20 — race guard : sequence counter pattern (cf. ReconciliationProposals).
	let genSeq = 0;

	// P17 — reset period dates when fiscal year changes (avoid silent cross-FY 400).
	let lastFyId: number | null = selectedFiscalYearId;
	$effect(() => {
		if (selectedFiscalYearId !== lastFyId) {
			periodStart = '';
			periodEnd = '';
			balanceSheet = null;
			incomeStatement = null;
			trialBalance = null;
			journalReport = null;
			errorMsg = null;
			lastFyId = selectedFiscalYearId;
		}
	});

	// P5 — error handler : ApiError structured fields + Fluent interpolation.
	function formatError(err: unknown): string {
		if (isApiError(err)) {
			if (
				err.code === 'REPORT_PERIOD_OUT_OF_FISCAL_YEAR' &&
				err.details &&
				typeof err.details.fyStart === 'string' &&
				typeof err.details.fyEnd === 'string'
			) {
				return i18nMsg(
					'reports-error-period-out-of-fiscal-year',
					`La période sélectionnée dépasse les bornes de l'exercice. Choisissez une période entre ${formatSwissDate(err.details.fyStart)} et ${formatSwissDate(err.details.fyEnd)}.`,
					{
						fyStart: formatSwissDate(err.details.fyStart),
						fyEnd: formatSwissDate(err.details.fyEnd),
					},
				);
			}
			return err.message;
		}
		return err instanceof Error ? err.message : String(err);
	}

	async function generate(): Promise<void> {
		if (selectedFiscalYearId === null) return;
		const mySeq = ++genSeq;
		loading = true;
		errorMsg = null;
		const query = {
			fiscalYearId: selectedFiscalYearId,
			periodStart: periodStart || undefined,
			periodEnd: periodEnd || undefined,
		};
		try {
			switch (activeTab) {
				case 'balance-sheet': {
					const result = await getBalanceSheet(query);
					if (mySeq === genSeq) balanceSheet = result;
					break;
				}
				case 'income-statement': {
					const result = await getIncomeStatement(query);
					if (mySeq === genSeq) incomeStatement = result;
					break;
				}
				case 'trial-balance': {
					const result = await getTrialBalance(query);
					if (mySeq === genSeq) trialBalance = result;
					break;
				}
				case 'journals': {
					const result = await getJournalReport(query);
					if (mySeq === genSeq) journalReport = result;
					break;
				}
			}
		} catch (e) {
			if (mySeq === genSeq) errorMsg = formatError(e);
		} finally {
			if (mySeq === genSeq) loading = false;
		}
	}

	const tabs: { id: ReportType; labelKey: string; fallback: string }[] = [
		{ id: 'balance-sheet', labelKey: 'reports-balance-sheet', fallback: 'Bilan' },
		{ id: 'income-statement', labelKey: 'reports-income-statement', fallback: 'Compte de résultat' },
		{ id: 'trial-balance', labelKey: 'reports-trial-balance', fallback: 'Balance' },
		{ id: 'journals', labelKey: 'reports-journals', fallback: 'Journaux' },
	];

	// P6 — ARIA tabs : keyboard navigation (ArrowLeft/Right/Home/End).
	function handleTabKeydown(event: KeyboardEvent, index: number): void {
		let nextIndex: number | null = null;
		if (event.key === 'ArrowRight') nextIndex = (index + 1) % tabs.length;
		else if (event.key === 'ArrowLeft') nextIndex = (index - 1 + tabs.length) % tabs.length;
		else if (event.key === 'Home') nextIndex = 0;
		else if (event.key === 'End') nextIndex = tabs.length - 1;
		if (nextIndex !== null) {
			event.preventDefault();
			activeTab = tabs[nextIndex].id;
			const btn = document.getElementById(`reports-tab-${tabs[nextIndex].id}`);
			btn?.focus();
		}
	}
</script>

<svelte:head>
	<title>Kesh — Rapports</title>
</svelte:head>

<div class="space-y-4 p-4">
	<h1 class="text-2xl font-bold">{i18nMsg('reports-page-title', 'Rapports comptables')}</h1>

	<ReportSelector
		fiscalYears={data.fiscalYears}
		bind:selectedFiscalYearId
		bind:periodStart
		bind:periodEnd
		{loading}
		onGenerate={generate}
	/>

	<div role="tablist" class="flex border-b" aria-label={i18nMsg('reports-page-title', 'Rapports comptables')}>
		{#each tabs as tab, idx (tab.id)}
			<button
				type="button"
				role="tab"
				id="reports-tab-{tab.id}"
				aria-selected={activeTab === tab.id}
				aria-controls="reports-tabpanel-{tab.id}"
				tabindex={activeTab === tab.id ? 0 : -1}
				class="border-b-2 px-4 py-2 text-sm font-medium {activeTab === tab.id
					? 'border-indigo-600 text-indigo-700'
					: 'border-transparent text-gray-600 hover:text-gray-900'}"
				onclick={() => (activeTab = tab.id)}
				onkeydown={(e) => handleTabKeydown(e, idx)}
			>
				{i18nMsg(tab.labelKey, tab.fallback)}
			</button>
		{/each}
	</div>

	{#if errorMsg}
		<p class="rounded bg-red-50 p-3 text-sm text-red-900" role="alert">{errorMsg}</p>
	{/if}

	<div
		role="tabpanel"
		id="reports-tabpanel-{activeTab}"
		aria-labelledby="reports-tab-{activeTab}"
	>
		{#if loading}
			<p class="text-sm italic text-gray-500" role="status">
				{i18nMsg('reports-loading', 'Génération du rapport en cours…')}
			</p>
		{:else if activeTab === 'balance-sheet' && balanceSheet}
			<BalanceSheetView dto={balanceSheet} />
		{:else if activeTab === 'income-statement' && incomeStatement}
			<IncomeStatementView dto={incomeStatement} />
		{:else if activeTab === 'trial-balance' && trialBalance}
			<TrialBalanceView dto={trialBalance} />
		{:else if activeTab === 'journals' && journalReport}
			<JournalReportView dto={journalReport} />
		{:else}
			<p class="text-sm italic text-gray-500">
				{i18nMsg(
					'reports-instruction-select-and-generate',
					'Sélectionnez un exercice et cliquez sur Générer.',
				)}
			</p>
		{/if}
	</div>
</div>
