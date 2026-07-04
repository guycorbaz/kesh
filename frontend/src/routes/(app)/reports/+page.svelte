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
	import VatReportView from '$lib/features/reports/VatReportView.svelte';
	import {
		buildExportFilename,
		downloadReport,
		downloadProjectReport,
		formatSwissDate,
		getBalanceSheet,
		getIncomeStatement,
		getJournalReport,
		getProjectExpenses,
		getTrialBalance,
		getVatReport,
	} from '$lib/features/reports/reports.api';
	import ProjectExpensesView from '$lib/features/reports/ProjectExpensesView.svelte';
	import { listProjects } from '$lib/features/projects/projects.api';
	import type { ProjectResponse } from '$lib/features/projects/projects.types';
	import type {
		BalanceSheetDto,
		IncomeStatementDto,
		JournalReportDto,
		JournalReportQuery,
		ProjectExpensesDto,
		ProjectReportMode,
		ProjectReportQuery,
		ReportQuery,
		ReportType,
		TrialBalanceDto,
		VatReportDto,
	} from '$lib/features/reports/reports.types';
	import type { FiscalYearResponse } from '$lib/features/fiscal-years/fiscal-years.types';

	/** Onglets = rapports comptables classiques + rapports par projet (Story 19-6a). */
	type TabId = ReportType | 'project-expenses';

	interface PageData {
		fiscalYears: FiscalYearResponse[];
		// Story 9-2a — Nom de la company pour construire les filenames d'export.
		companyName: string;
	}
	let { data }: { data: PageData } = $props();

	let selectedFiscalYearId = $state<number | null>(data.fiscalYears[0]?.id ?? null);
	let periodStart = $state('');
	let periodEnd = $state('');
	let activeTab = $state<TabId>('balance-sheet');
	let loading = $state(false);
	let errorMsg = $state<string | null>(null);

	// Story 19-6a — état du rapport « Dépenses par projet ».
	let projects = $state<ProjectResponse[]>([]);
	let selectedProjectId = $state<number | null>(null);
	let projectMode = $state<ProjectReportMode>('fiscal_year');
	let projectExpenses = $state<ProjectExpensesDto | null>(null);
	$effect(() => {
		listProjects()
			.then((p) => (projects = p))
			.catch(() => (projects = []));
	});
	// Story 9-2a — flag dédié pour l'export (PAS partagé avec `loading` qui contrôle
	// uniquement `generate()`). Pass 1 ECH-H2 + AC #36 + Pass 2 AA2-C1.
	let exporting = $state(false);

	let balanceSheet = $state<BalanceSheetDto | null>(null);
	let incomeStatement = $state<IncomeStatementDto | null>(null);
	let trialBalance = $state<TrialBalanceDto | null>(null);
	let journalReport = $state<JournalReportDto | null>(null);
	let vatReport = $state<VatReportDto | null>(null);

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
			vatReport = null;
			projectExpenses = null;
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

	/** Construit la query du rapport projet actif (null si incomplet). */
	function projectQuery(): ProjectReportQuery | null {
		if (selectedProjectId === null) return null;
		if (projectMode === 'fiscal_year' && selectedFiscalYearId === null) return null;
		return {
			projectId: selectedProjectId,
			mode: projectMode,
			fiscalYearId: projectMode === 'fiscal_year' ? (selectedFiscalYearId ?? undefined) : undefined,
			periodStart: projectMode === 'fiscal_year' ? periodStart || undefined : undefined,
			periodEnd: projectMode === 'fiscal_year' ? periodEnd || undefined : undefined,
		};
	}

	async function generate(): Promise<void> {
		// Story 19-6a — branche projet (query différente des rapports classiques).
		if (activeTab === 'project-expenses') {
			const q = projectQuery();
			if (!q) return;
			const mySeq = ++genSeq;
			loading = true;
			errorMsg = null;
			try {
				const result = await getProjectExpenses(q);
				if (mySeq === genSeq) projectExpenses = result;
			} catch (e) {
				if (mySeq === genSeq) errorMsg = formatError(e);
			} finally {
				if (mySeq === genSeq) loading = false;
			}
			return;
		}
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
				case 'vat': {
					const result = await getVatReport(query);
					if (mySeq === genSeq) vatReport = result;
					break;
				}
			}
		} catch (e) {
			if (mySeq === genSeq) errorMsg = formatError(e);
		} finally {
			if (mySeq === genSeq) loading = false;
		}
	}

	// Story 9-2a — Helpers export PDF/CSV.

	/**
	 * Retourne le DTO actuel pour l'onglet actif, ou `null` si aucun rapport
	 * généré. Sert à `canExport` ET à extraire `period.start/end` pour le filename.
	 */
	function activeReportPeriod(): { startDate: string; endDate: string } | null {
		switch (activeTab) {
			case 'balance-sheet':
				return balanceSheet?.period ?? null;
			case 'income-statement':
				return incomeStatement?.period ?? null;
			case 'trial-balance':
				return trialBalance?.period ?? null;
			case 'journals':
				return journalReport?.period ?? null;
			case 'vat':
				return vatReport?.period ?? null;
			default:
				// 'project-expenses' — géré via son propre chemin (projectExpenses).
				return null;
		}
	}

	let canExport = $derived(
		!loading &&
			!exporting &&
			selectedFiscalYearId !== null &&
			data.fiscalYears.length > 0 &&
			activeReportPeriod() !== null,
	);

	/**
	 * Handler factorisé : déclenche le download d'un rapport au format demandé.
	 * Flag `exporting` toujours reset dans `finally` (AC #36(b)). Pass 1 ECH-H2
	 * closure capture : `query` est snapshot au moment du clic, donc un change de
	 * FY pendant l'export en vol ne casse pas l'opération (AC #36(a)).
	 *
	 * Pass 1 code-review M12 (BH4-F-02) : `exporting = true` est posé AVANT
	 * tout `await` (immédiatement après le early-return guard) pour éviter une
	 * fenêtre de race où deux clics rapides passeraient tous deux le guard
	 * `if (exporting) return` avant que le flag ne soit mis à jour.
	 *
	 * Pass 1 code-review M13 (AA4-F2) : si l'erreur n'expose pas de code
	 * structuré (network, parse error), on retombe sur la clé i18n générique
	 * `reports-export-error-generic` (AC #23) plutôt qu'un message brut.
	 */
	async function exportReport(format: 'pdf' | 'csv'): Promise<void> {
		// Pass 1 code-review M12 : guard re-entrancy avant TOUT await.
		if (exporting) return;
		// Story 19-6a — branche projet.
		if (activeTab === 'project-expenses') {
			const q = projectQuery();
			if (!q || projectExpenses === null) return;
			exporting = true;
			errorMsg = null;
			try {
				const companySlug = data.companyName.replace(/[^a-zA-Z0-9]+/g, '-').slice(0, 20);
				const filename = `kesh-depenses-par-projet-${companySlug}.${format}`;
				await downloadProjectReport('project-expenses', q, format, filename);
			} catch (e) {
				if (isApiError(e) && e.code) errorMsg = formatError(e);
				else
					errorMsg = i18nMsg(
						'reports-export-error-generic',
						"Impossible d'exporter le rapport. Vérifiez votre connexion et réessayez.",
					);
			} finally {
				exporting = false;
			}
			return;
		}
		if (!canExport || selectedFiscalYearId === null) return;
		const period = activeReportPeriod();
		if (!period) return;
		exporting = true;
		errorMsg = null;
		try {
			const query: ReportQuery | JournalReportQuery = {
				fiscalYearId: selectedFiscalYearId,
				periodStart: periodStart || undefined,
				periodEnd: periodEnd || undefined,
			};
			if (activeTab === 'journals') {
				// Aucun filtre journal exposé v0.1 — laisse undefined.
			}
			const filename = buildExportFilename(
				activeTab,
				data.companyName,
				{ start: period.startDate, end: period.endDate },
				format,
			);
			await downloadReport(activeTab, query, format, filename);
		} catch (e) {
			// Pass 1 code-review M13 (AA4-F2) : code structuré → message i18n
			// du backend (formatError) ; sinon fallback générique AC #23.
			if (isApiError(e) && e.code) {
				errorMsg = formatError(e);
			} else {
				errorMsg = i18nMsg(
					'reports-export-error-generic',
					"Impossible d'exporter le rapport. Vérifiez votre connexion et réessayez.",
				);
			}
		} finally {
			exporting = false;
		}
	}

	async function exportPdf(): Promise<void> {
		await exportReport('pdf');
	}

	async function exportCsv(): Promise<void> {
		await exportReport('csv');
	}

	const tabs: { id: TabId; labelKey: string; fallback: string }[] = [
		{ id: 'balance-sheet', labelKey: 'reports-balance-sheet', fallback: 'Bilan' },
		{ id: 'income-statement', labelKey: 'reports-income-statement', fallback: 'Compte de résultat' },
		{ id: 'trial-balance', labelKey: 'reports-trial-balance', fallback: 'Balance' },
		{ id: 'journals', labelKey: 'reports-journals', fallback: 'Journaux' },
		{ id: 'vat', labelKey: 'reports-vat', fallback: 'TVA' },
		{
			id: 'project-expenses',
			labelKey: 'reports-project-expenses',
			fallback: 'Dépenses par projet',
		},
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
			selectTab(tabs[nextIndex].id);
			const btn = document.getElementById(`reports-tab-${tabs[nextIndex].id}`);
			btn?.focus();
		}
	}

	/**
	 * Sélectionne un nouvel onglet et clear `errorMsg` (Pass 1 code-review M10 —
	 * ECH4-M2 : avant ce patch, une erreur d'export sur l'onglet Bilan restait
	 * affichée après bascule vers l'onglet Compte de résultat, faux-positif UX).
	 */
	function selectTab(next: TabId): void {
		if (activeTab !== next) {
			activeTab = next;
			errorMsg = null;
		}
	}
</script>

<svelte:head>
	<title>Kesh — Rapports</title>
</svelte:head>

<div class="space-y-4 p-4">
	<h1 class="text-2xl font-bold">{i18nMsg('reports-page-title', 'Rapports comptables')}</h1>

	{#if activeTab !== 'project-expenses'}
		<ReportSelector
			fiscalYears={data.fiscalYears}
			bind:selectedFiscalYearId
			bind:periodStart
			bind:periodEnd
			{loading}
			onGenerate={generate}
			onExportPdf={exportPdf}
			onExportCsv={exportCsv}
			{canExport}
			{exporting}
		/>
	{:else}
		<!-- Story 19-6a — contrôles dédiés du rapport « Dépenses par projet ». -->
		<div class="flex flex-wrap items-end gap-3 rounded border border-border bg-surface p-3" data-testid="project-report-controls">
			<label class="text-sm">
				{i18nMsg('reports-project-selector-label', 'Projet')}
				<select
					class="mt-1 block rounded border border-border px-2 py-1 text-sm"
					bind:value={selectedProjectId}
					data-testid="project-report-project"
				>
					<option value={null}>{i18nMsg('reports-project-selector-placeholder', '— Choisir un projet')}</option>
					{#each projects.filter((p) => p.parentId === null) as root (root.id)}
						<option value={root.id}>{root.code} — {root.name}</option>
						{#each projects.filter((c) => c.parentId === root.id) as child (child.id)}
							<option value={child.id}>&nbsp;&nbsp;↳ {child.code} — {child.name}</option>
						{/each}
					{/each}
				</select>
			</label>

			<label class="text-sm">
				{i18nMsg('reports-project-mode-label', 'Période')}
				<select
					class="mt-1 block rounded border border-border px-2 py-1 text-sm"
					bind:value={projectMode}
					data-testid="project-report-mode"
				>
					<option value="fiscal_year">{i18nMsg('reports-project-mode-fiscal-year', 'Exercice')}</option>
					<option value="cumulative">{i18nMsg('reports-project-mode-cumulative', 'Cumulé depuis l’origine')}</option>
				</select>
			</label>

			{#if projectMode === 'fiscal_year'}
				<label class="text-sm">
					{i18nMsg('reports-fiscal-year-label', 'Exercice')}
					<select
						class="mt-1 block rounded border border-border px-2 py-1 text-sm"
						bind:value={selectedFiscalYearId}
						data-testid="project-report-fiscal-year"
					>
						{#each data.fiscalYears as fy (fy.id)}
							<option value={fy.id}>{fy.name}</option>
						{/each}
					</select>
				</label>
			{/if}

			<button
				type="button"
				class="rounded bg-primary px-3 py-1 text-sm text-white disabled:opacity-50"
				onclick={generate}
				disabled={loading || selectedProjectId === null}
				data-testid="project-report-generate"
			>
				{i18nMsg('reports-generate', 'Générer')}
			</button>
			<button
				type="button"
				class="rounded border border-border px-3 py-1 text-sm disabled:opacity-50"
				onclick={exportPdf}
				disabled={exporting || projectExpenses === null}
				data-testid="project-report-export-pdf"
			>
				PDF
			</button>
			<button
				type="button"
				class="rounded border border-border px-3 py-1 text-sm disabled:opacity-50"
				onclick={exportCsv}
				disabled={exporting || projectExpenses === null}
				data-testid="project-report-export-csv"
			>
				CSV
			</button>
		</div>
	{/if}

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
				onclick={() => selectTab(tab.id)}
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
		{:else if activeTab === 'vat' && vatReport}
			<VatReportView dto={vatReport} />
		{:else if activeTab === 'project-expenses' && projectExpenses}
			<ProjectExpensesView report={projectExpenses} />
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
