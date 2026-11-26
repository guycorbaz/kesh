<script lang="ts">
	// Story 9-1 — Page rapports comptables.
	// AC #33 (Pass 1 AA-10) : 4 onglets (Bilan, Résultat, Balance, Journaux).
	// Code review Pass 1 patches : P5 (error handler isApiError + Fluent), P6 (ARIA tabs),
	// P17 (reset dates on FY change), P19 (loading indicator), P20 (race guard).

	import { untrack } from 'svelte';
	import { page } from '$app/state';
	import { goto } from '$app/navigation';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { isApiError } from '$lib/shared/utils/api-client';
	import { errorMessageOf } from '$lib/shared/utils/error-message';
	import { estDateStockable } from '$lib/shared/utils/date-filtre';
	import { authState } from '$lib/app/stores/auth.svelte';
	import ReportSelector from '$lib/features/reports/ReportSelector.svelte';
	import BalanceSheetView from '$lib/features/reports/BalanceSheetView.svelte';
	import AgedReceivablesView from '$lib/features/reports/AgedReceivablesView.svelte';
	import IncomeStatementView from '$lib/features/reports/IncomeStatementView.svelte';
	import TrialBalanceView from '$lib/features/reports/TrialBalanceView.svelte';
	import GeneralLedgerView from '$lib/features/reports/GeneralLedgerView.svelte';
	import JournalReportView from '$lib/features/reports/JournalReportView.svelte';
	import VatReportView from '$lib/features/reports/VatReportView.svelte';
	import GroupingControls from '$lib/features/reports/GroupingControls.svelte';
	import {
		groupingParamsOf,
		readGroupingView,
		sameGroupingParams,
		toggleGroup,
		withEcho,
		writeGroupingView,
		type GroupedReport,
		type GroupingView,
	} from '$lib/features/reports/reports-grouping';
	import {
		buildExportFilename,
		downloadReport,
		downloadProjectReport,
		formatSwissDate,
		getBalanceSheet,
		getIncomeStatement,
		getJournalReport,
		getProjectExpenses,
		getProjectReturn,
		getTrialBalance,
		getVatReport,
		getGeneralLedger,
		downloadGeneralLedger,
		getAgedReceivables,
		downloadAgedReceivables,
	} from '$lib/features/reports/reports.api';
	import {
		buildReportSearch,
		isClassicTab,
		parseReportUrl,
		reportPart,
		resolveClassicPeriod,
		type ClassicTab,
		type Presentation,
		type TabId,
	} from '$lib/features/reports/reports-url';
	import { etatDuMenu, type EtatMenu } from '$lib/features/reports/fiscal-year-menu';
	import ProjectExpensesView from '$lib/features/reports/ProjectExpensesView.svelte';
	import ProjectReturnView from '$lib/features/reports/ProjectReturnView.svelte';
	import { listProjects } from '$lib/features/projects/projects.api';
	import { fetchAccounts } from '$lib/features/accounts/accounts.api';
	// fx-filtre-comptes : exempté — rapports (consultation, comptes en devise compris).
	import { letterableAccountIds as letterableIdsOf } from '$lib/features/open-items/open-items';
	import type { AccountResponse } from '$lib/features/accounts/accounts.types';
	import type { ProjectResponse } from '$lib/features/projects/projects.types';
	import type {
		BalanceSheetDto,
		IncomeStatementDto,
		JournalReportDto,
		JournalReportQuery,
		ProjectExpensesDto,
		ProjectReportMode,
		ProjectReportQuery,
		ProjectReportType,
		ProjectReturnDto,
		ReportQuery,
		ReportType,
		TrialBalanceDto,
		VatReportDto,
		GeneralLedgerDto,
		LedgerQuery,
		AgedReceivablesDto,
		GroupedRowBase,
		GroupingParams,
	} from '$lib/features/reports/reports.types';
	import type { FiscalYearResponse } from '$lib/features/fiscal-years/fiscal-years.types';

	// Onglets = rapports comptables classiques + rapports par projet (19-6a/b) + balance âgée (21-7) +
	// grand livre (24-1) : `TabId` et la liste des identifiants vivent dans `reports-url.ts` (55-1a).

	interface PageData {
		fiscalYears: FiscalYearResponse[];
		// Story 9-2a — Nom de la company pour construire les filenames d'export.
		companyName: string;
		// Story 46-2 (AC-10) — clé de la vue groupée mémorisée ; `null` si inconnue.
		companyId?: number | null;
	}
	let { data }: { data: PageData } = $props();

	// Story 55-1a (D15, AC-15) — les dates sont toujours remplies : à l'ouverture, les bornes du premier
	// exercice ; l'exercice (`selectedFiscalYearId`) est DÉDUIT des dates (`etatDuMenu`), et garde sa
	// dernière valeur quand elles sortent d'un exercice (« Plusieurs », « Hors exercice »).
	const premierExercice = untrack(() => data.fiscalYears[0] ?? null);
	let selectedFiscalYearId = $state<number | null>(premierExercice?.id ?? null);
	let periodStart = $state(premierExercice?.startDate ?? '');
	let periodEnd = $state(premierExercice?.endDate ?? '');
	let menu = $state<EtatMenu>(
		premierExercice ? { kind: 'fy', id: premierExercice.id, whole: true } : { kind: 'outside' },
	);
	// AC-20 — début après fin : rien n'est chargé, un message le dit sous les dates.
	let periodInvalid = $state(false);
	let activeTab = $state<TabId>('balance-sheet');
	let loading = $state(false);
	let errorMsg = $state<string | null>(null);

	// Story 19-6a — état du rapport « Dépenses par projet ».
	let projects = $state<ProjectResponse[]>([]);
	let selectedProjectId = $state<number | null>(null);
	let projectMode = $state<ProjectReportMode>('fiscal_year');
	let projectExpenses = $state<ProjectExpensesDto | null>(null);
	let projectReturn = $state<ProjectReturnDto | null>(null);
	// True quand l'onglet actif est un rapport par projet (contrôles dédiés).
	const isProjectTab = $derived(
		activeTab === 'project-expenses' || activeTab === 'project-return',
	);
	// Story 21-7 — balance âgée : onglet à contrôles dédiés (pas de FY/période).
	const isAgedTab = $derived(activeTab === 'aged-receivables');
	let agedReceivables = $state<AgedReceivablesDto | null>(null);

	// Story 24-1 — grand livre : onglet à contrôles dédiés lui aussi, mais pour
	// une raison différente de la balance âgée — il prend bien une période, ce
	// qu'il ne prend PAS c'est un exercice. Il franchit la borne, sans quoi il ne
	// concorderait pas avec le bilan, qui est cumulatif depuis l'origine.
	const isLedgerTab = $derived(activeTab === 'general-ledger');
	let generalLedger = $state<GeneralLedgerDto | null>(null);
	let ledgerFrom = $state('');
	let ledgerTo = $state('');
	let ledgerAccountId = $state<number | null>(null);
	let ledgerIncludeZero = $state(false);
	let accounts = $state<AccountResponse[]>([]);
	// Export CSV réservé Comptable+ (D24) — inutile d'afficher un bouton qui prendra 403.
	const canManage = $derived(
		authState.currentUser?.role === 'Admin' || authState.currentUser?.role === 'Comptable',
	);
	// Un rapport projet a-t-il été généré pour l'onglet actif (gouverne l'export).
	const hasProjectReport = $derived(
		activeTab === 'project-return' ? projectReturn !== null : projectExpenses !== null,
	);
	$effect(() => {
		listProjects()
			.then((p) => (projects = p))
			.catch(() => (projects = []));
	});
	// Grand livre : les comptes ARCHIVÉS sont inclus — un compte archivé garde
	// son historique, et c'est précisément le grand livre qui le donne à lire.
	$effect(() => {
		fetchAccounts(true)
			.then((a) => (accounts = a))
			.catch(() => (accounts = []));
	});
	// Story 15-1c-ii (AC9, C-15-1c-8) — les comptes lettrables, lus dans la même
	// liste : le Grand livre n'offre « Postes ouverts de ce compte » que pour eux.
	// Liste en échec → ensemble vide → aucun lien.
	const letterableAccountIds = $derived(letterableIdsOf(accounts));
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
	// L'onglet dont une demande est en vol, `null` sinon (Story 55-1a : tous les onglets).
	let inFlightTab = $state<TabId | null>(null);

	// Story 55-1a (AC-13, point 4) — les clés d'un onglet : la clé AFFICHÉE est la recherche canonique
	// de la réponse qu'il affiche ; la clé EN VOL, celle de la demande partie. Une clé affichée égale
	// à la clé des champs : rien à demander. Les rapports par compte analytique n'en ont pas (C-55-1-7).
	let displayedKeys = $state<Partial<Record<TabId, string>>>({});
	let inFlightKey: string | null = null;
	// La requête de la réponse affichée de chaque rapport : l'export la reprend (AC-25), la règle de la
	// 46-2 la redemande (une présentation changée ne change pas la période du rapport affiché).
	let displayedClassic: Partial<Record<ClassicTab, ReportQuery>> = {};
	let inFlightClassic: ReportQuery | null = null;
	let displayedLedger: LedgerQuery | null = null;
	let displayedProject: Partial<Record<ProjectReportType, ProjectReportQuery>> = {};

	/**
	 * Périme toute demande en vol, de quelque onglet qu'elle soit, et rabaisse ses drapeaux (`loading`,
	 * `refreshing`) ; rend le numéro de la demande suivante.
	 *
	 * ⚠️ **Le seul endroit qui incrémente `genSeq`** (Story 46-2, revue P1 M1) : une réponse périmée est
	 * jetée par `mySeq !== genSeq`, donc son `finally` ne rabaisse plus rien — c'est à celui qui la
	 * périme de le faire. Story 55-1a (AC-21, R-16) : c'est aussi ce qui fait que **la dernière demande
	 * l'emporte** — aucune annulation réseau (`api-client.ts` n'en propose pas), la réponse d'une demande
	 * plus ancienne que la dernière est ignorée, sans erreur ni message.
	 */
	function invalidateInFlight(): number {
		genSeq++;
		inFlightTab = null;
		inFlightKey = null;
		inFlightClassic = null;
		loading = false;
		refreshing = false;
		return genSeq;
	}

	// Story 46-2 — vue groupée du bilan et du compte de résultat (AC-1 à AC-6, AC-10) ;
	// Story 46-6b — et de la balance, avec sa propre mémoire.
	// Relue de la mémoire du navigateur AVANT la première requête : la première
	// demande part avec la vue retenue, sans passer par la vue par défaut.
	const companyId = untrack(() => data.companyId ?? null);
	let groupingViews = $state<Record<GroupedReport, GroupingView>>({
		'balance-sheet': readGroupingView(companyId, 'balance-sheet'),
		'income-statement': readGroupingView(companyId, 'income-statement'),
		'trial-balance': readGroupingView(companyId, 'trial-balance'),
	});
	// Paramètres de la dernière réponse AFFICHÉE de chaque rapport : l'export les
	// reprend tels quels (AC-6), et non la vue en cours de redemande.
	let displayedGrouping: Record<GroupedReport, GroupingParams | undefined> = {
		'balance-sheet': undefined,
		'income-statement': undefined,
		'trial-balance': undefined,
	};
	// Redemande après un changement de présentation : l'écran garde l'affichage
	// précédent (sans clignotement), marqué `aria-busy` (AC-3).
	let refreshing = $state(false);
	/** L'onglet est-il un rapport qui a une vue groupée ? */
	function isGroupedReport(tab: TabId): tab is GroupedReport {
		return tab === 'balance-sheet' || tab === 'income-statement' || tab === 'trial-balance';
	}
	const isGroupedTab = $derived(isGroupedReport(activeTab));
	const activeGroupingView = $derived(
		isGroupedTab ? groupingViews[activeTab as GroupedReport] : null,
	);
	/** Le rapport groupé affiché d'un onglet, ou `null`. */
	function displayedReport(
		report: GroupedReport,
	): BalanceSheetDto | IncomeStatementDto | TrialBalanceDto | null {
		switch (report) {
			case 'balance-sheet':
				return balanceSheet;
			case 'income-statement':
				return incomeStatement;
			case 'trial-balance':
				return trialBalance;
		}
	}
	const activeMaxDepth = $derived(
		isGroupedTab ? (displayedReport(activeTab as GroupedReport)?.grouping?.maxDepth ?? null) : null,
	);

	/** Retient la vue d'un rapport, à l'écran et dans la mémoire du navigateur. */
	function setGroupingView(report: GroupedReport, view: GroupingView): void {
		groupingViews[report] = view;
		writeGroupingView(companyId, report, view);
	}

	/**
	 * Un changement de présentation (contrôles ou accordéon) : la vue est retenue,
	 * et le rapport affiché est redemandé à l'API, seule à connaître la règle
	 * d'ouverture (C-46-2-1). Sans rapport affiché, rien n'est demandé.
	 */
	function changeGrouping(next: GroupingView): void {
		if (!isGroupedTab) return;
		const report = activeTab as GroupedReport;
		setGroupingView(report, next);
		const displayed = displayedReport(report);
		// Revue P1 M2 (a) — pendant une demande de ce rapport, elle est partie avec
		// l'ancienne vue : la redemander, sans quoi les contrôles diraient une vue
		// que l'écran n'affiche pas. Story 55-1b (T0) : chargement **ou rechargement**
		// (`refreshing`, d'autres paramètres en vol) — la requête en vol est reprise,
		// non celle du rapport affiché, sans quoi la période demandée serait perdue.
		const inFlightThis = inFlightTab === report && inFlightClassic !== null;
		// Story 55-1a — la redemande porte sur la période du rapport affiché (ou de la demande en vol),
		// non sur des champs modifiés depuis : seule la présentation change.
		if (inFlightThis) {
			void generate({ keepDisplay: displayed !== null, classicQuery: inFlightClassic ?? undefined });
		} else if (displayed) {
			void generate({ keepDisplay: true, classicQuery: displayedClassic[report] });
		}
	}

	/**
	 * Revue P1 M2 (b) — le rapport affiché l'a-t-il été avec la vue en cours ?
	 * Sinon (redemande jetée par une génération sur un autre onglet), il faut le
	 * redemander : les contrôles ne montrent jamais une vue que l'écran n'affiche pas.
	 */
	function displayedMatchesView(report: GroupedReport): boolean {
		return sameGroupingParams(displayedGrouping[report], groupingParamsOf(groupingViews[report]));
	}

	/**
	 * Remet l'onglet actif d'accord avec ses contrôles : si c'est un rapport groupé (bilan,
	 * compte de résultat, balance), affiché avec une autre vue que celle des contrôles, et
	 * qu'aucune génération n'est en vol, il est redemandé (affichage gardé).
	 *
	 * Appelée au retour sur l'onglet (`selectTab`) et à la fin d'une génération
	 * d'un AUTRE onglet (revue P2 M1) : un retour pendant ce chargement-là ne
	 * redemande rien — il périmerait la génération demandée sur l'autre onglet —,
	 * la redemande attend donc qu'elle finisse.
	 */
	function resyncActiveGroupedTab(): void {
		if (!isGroupedReport(activeTab)) return;
		const displayed = displayedReport(activeTab);
		// Story 55-1b — ni pendant une demande de CET onglet (redemande comprise, `refreshing`) : elle
		// est déjà partie avec la vue des contrôles, ou `loadActiveIfNeeded` l'a jugée (V6-5).
		if (displayed && !loading && inFlightTab !== activeTab && !displayedMatchesView(activeTab)) {
			void generate({ keepDisplay: true, classicQuery: displayedClassic[activeTab] });
		}
	}

	/**
	 * Fin d'une génération non périmée, lancée sur `finishedTab` : si l'utilisateur
	 * a changé d'onglet entre-temps, l'onglet actif est remis d'accord avec ses
	 * contrôles. La génération de l'onglet actif lui-même ne redemande jamais
	 * (une redemande en erreur ne boucle pas).
	 */
	function afterGeneration(finishedTab: TabId): void {
		if (finishedTab !== activeTab) resyncActiveGroupedTab();
	}

	function onToggleGroup(row: GroupedRowBase): void {
		if (!activeGroupingView) return;
		changeGrouping(toggleGroup(activeGroupingView, row.accountId, row.open === true, row.depth));
	}

	/**
	 * Une réponse d'un rapport groupé (bilan, compte de résultat, balance) est affichée : ses paramètres
	 * deviennent ceux de l'export, et la mémoire reprend les listes de l'écho (les
	 * groupes disparus du plan, ignorés par l'API, en sortent — AC-10).
	 */
	function acceptGrouped(
		report: GroupedReport,
		params: GroupingParams | undefined,
		dto: BalanceSheetDto | IncomeStatementDto | TrialBalanceDto,
	): void {
		displayedGrouping[report] = params;
		// Revue P1 M2 — l'écho n'est repris que s'il répond à la vue en cours :
		// il ne doit jamais remplacer un choix fait après le départ de la demande.
		// ⚠️ Garde défensive, inatteignable aujourd'hui : tout changement de vue
		// redemande le rapport (`changeGrouping`), ce qui périme la réponse
		// partie avec l'ancienne ; et le retour sur l'onglet remet l'écran
		// d'accord avec ses contrôles (`selectTab`).
		if (!displayedMatchesView(report)) return;
		if (params && dto.grouping) {
			const view = withEcho(groupingViews[report], dto.grouping);
			setGroupingView(report, view);
			// Les groupes que l'API a ignorés ne changent rien à l'affichage :
			// l'export part avec les listes de l'écho, égales à la vue retenue.
			displayedGrouping[report] = groupingParamsOf(view);
		}
	}

	// Story 55-1a (D15, R-8, R-11) — P17 (l'effet qui vidait dates et rapports au changement d'exercice)
	// est RETIRÉ : choisir un exercice se fait dans le gestionnaire du menu (`selectFiscalYear`), et
	// l'exercice suit les dates. Quand il change, seuls les rapports par compte analytique, qui lisent le
	// même exercice et n'ont pas de clé, sont vidés ; les rapports classiques restent (chacun sa clé).

	/** L'exercice a changé (menu, adresse, date saisie) : rapports par compte analytique vidés. */
	function fiscalYearChanged(): void {
		projectExpenses = null;
		projectReturn = null;
		displayedProject = {};
		if (inFlightTab === 'project-expenses' || inFlightTab === 'project-return') invalidateInFlight();
	}

	/** Remet l'état du menu d'accord avec les dates ; l'exercice suit (AC-15). */
	function syncMenu(): void {
		menu = etatDuMenu(periodStart, periodEnd, data.fiscalYears, menu);
		if (menu.kind === 'fy' && menu.id !== selectedFiscalYearId) {
			selectedFiscalYearId = menu.id;
			fiscalYearChanged();
		}
	}

	/** Pose la période des onglets classiques (et des rapports par compte analytique). */
	function setPeriod(from: string, to: string): void {
		periodStart = from;
		periodEnd = to;
		periodInvalid = false;
		syncMenu();
	}

	/** Vrai si la période des champs est demandable : deux dates complètes, début au plus tard à la fin. */
	function periodValid(): boolean {
		return estDateStockable(periodStart) && estDateStockable(periodEnd) && periodStart <= periodEnd;
	}

	// Story 19-6a code-review Pass 1 (MEDIUM) + Pass 2 (race) — invalide le rapport
	// projet affiché dès que le projet ou le mode change (sinon la vue montre le
	// projet A tandis que l'export rebâtit la requête sur la sélection courante B →
	// mismatch). `genSeq++` invalide AUSSI toute génération en vol : si l'utilisateur
	// change de projet pendant qu'une requête est en cours, son résultat (mySeq ≠
	// genSeq) ne re-peuplera pas `projectExpenses` (Pass 2 Haiku race — un résultat
	// du projet A ne doit pas s'afficher alors que B est sélectionné).
	let lastProjectSel: string | null = null;
	$effect(() => {
		const key = `${selectedProjectId}:${projectMode}`;
		if (key !== lastProjectSel) {
			if (lastProjectSel !== null) {
				projectExpenses = null;
				projectReturn = null;
				errorMsg = null;
				invalidateInFlight();
			}
			lastProjectSel = key;
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
		}
		// Story 28-2 (C-28-2-33 révisé, revue P1 B-1/A-4) : hors du code spécifique,
		// errorMessageOf rend le message d'une ApiError, et le générique s'il est vide.
		return errorMessageOf(err);
	}

	/** Construit la query du rapport projet actif (null si incomplet). */
	function projectQuery(): ProjectReportQuery | null {
		if (selectedProjectId === null) return null;
		// Story 55-1a (AC-15) — en mode exercice, la période doit tenir dans un seul exercice.
		if (projectMode === 'fiscal_year' && (menu.kind !== 'fy' || !periodValid())) return null;
		return {
			projectId: selectedProjectId,
			mode: projectMode,
			fiscalYearId: projectMode === 'fiscal_year' ? (selectedFiscalYearId ?? undefined) : undefined,
			periodStart: projectMode === 'fiscal_year' ? periodStart || undefined : undefined,
			periodEnd: projectMode === 'fiscal_year' ? periodEnd || undefined : undefined,
		};
	}

	/** Construit la query du grand livre (null si la période est incomplète). */
	function ledgerQuery(): LedgerQuery | null {
		if (!ledgerFrom || !ledgerTo) return null;
		return {
			from: ledgerFrom,
			to: ledgerTo,
			accountIds: ledgerAccountId === null ? undefined : [ledgerAccountId],
			includeZero: ledgerIncludeZero,
		};
	}

	/** La requête d'un onglet classique d'après les champs, `null` si la période n'est pas demandable. */
	function classicQueryOfFields(): ReportQuery | null {
		if (menu.kind !== 'fy' || !periodValid()) return null;
		return { fiscalYearId: menu.id, periodStart, periodEnd };
	}

	/**
	 * La clé (recherche canonique) d'un rapport classique pour une requête. Story 55-1b (D4) : sur un
	 * onglet groupé, la présentation en fait partie (`presentation`), sans quoi deux présentations du même
	 * rapport auraient la même clé.
	 */
	function classicKey(tab: ClassicTab, q: ReportQuery, presentation?: Presentation): string {
		return buildReportSearch({ tab, fy: q.fiscalYearId, from: q.periodStart, to: q.periodEnd, presentation });
	}

	/** La présentation de paramètres de regroupement : `'flat'` quand aucun n'est envoyé (hiérarchie masquée). */
	function presentationOfParams(params: GroupingParams | undefined): Presentation {
		return params ?? 'flat';
	}

	/** La présentation des contrôles d'un onglet groupé (celle qu'une demande enverrait), sinon rien. */
	function controlsPresentation(tab: TabId): Presentation | undefined {
		return isGroupedReport(tab) ? presentationOfParams(groupingParamsOf(groupingViews[tab])) : undefined;
	}

	/** La clé du grand livre pour une requête. */
	function ledgerKey(q: LedgerQuery): string {
		return buildReportSearch({
			tab: 'general-ledger',
			accountId: q.accountIds?.[0] ?? null,
			from: q.from,
			to: q.to,
			includeZero: q.includeZero,
		});
	}

	/**
	 * La clé du rapport que les champs d'un onglet demandent, `null` s'il n'est pas demandable
	 * (période invalide ou hors d'un exercice, grand livre sans dates, rapports par compte analytique).
	 */
	function fieldsKey(tab: TabId): string | null {
		if (isClassicTab(tab)) {
			const q = classicQueryOfFields();
			return q ? classicKey(tab, q, controlsPresentation(tab)) : null;
		}
		if (tab === 'general-ledger') {
			const q = ledgerQuery();
			return q ? ledgerKey(q) : null;
		}
		if (tab === 'aged-receivables') return buildReportSearch({ tab });
		return null;
	}

	/**
	 * L'adresse qu'un geste sur un onglet écrit : la clé des champs ; à défaut, l'onglet et ses dates
	 * (« Plusieurs », « Hors exercice »). Le grand livre, qui ne se charge pas encore seul (55-2),
	 * écrit la clé de son rapport affiché, ou son onglet seul.
	 */
	function urlForTab(tab: TabId): string {
		if (tab === 'general-ledger') return displayedKeys[tab] ?? buildReportSearch({ tab });
		const key = fieldsKey(tab);
		if (key === null) return buildReportSearch({ tab, from: periodStart, to: periodEnd });
		// Story 55-1b (D4, C-55-1b-2) — la présentation n'entre dans l'URL que par la réponse : si l'onglet
		// affiche ce rapport (mêmes paramètres) et que la demande en vol n'est pas celle des champs, l'URL
		// est la clé AFFICHÉE ; la réponse d'une redemande l'écrira.
		const shown = displayedKeys[tab];
		const champsEnVol = inFlightTab === tab && inFlightKey === key;
		if (isGroupedReport(tab) && !champsEnVol && shown !== undefined && reportPart(shown) === reportPart(key)) {
			return shown;
		}
		return key;
	}

	/**
	 * Story 55-1b (D4, R-1) — la réponse acceptée d'un onglet groupé écrit sa clé affichée (écho compris),
	 * en remplaçant l'entrée : seulement pour l'onglet actif, et seulement si la dernière adresse écrite
	 * ou appliquée par la page désigne encore **le même rapport** que la requête (présentation exceptée).
	 * Une navigation entre-temps vers un autre onglet, d'autres dates ou d'autres paramètres a changé
	 * cette adresse : elle n'est pas réécrite. ⚠️ Comparer à `lastHandledKey` et non à `page.url` relevée
	 * au départ : un geste lance sa demande avant d'écrire son adresse (`selectTab`, `commitPeriod`).
	 */
	function writeUrlAtResponse(tab: TabId, requestKey: string): void {
		const shown = displayedKeys[tab];
		if (tab !== activeTab || shown === undefined || lastHandledKey === null) return;
		if (reportPart(lastHandledKey) !== reportPart(requestKey)) return;
		writeUrl(shown, true);
	}

	/**
	 * Une demande pour l'onglet actif. La dernière demande l'emporte (AC-21) : elle périme celle qui
	 * est en vol, de quelque onglet qu'elle soit. Elle n'écrit jamais l'URL (ce sont les gestionnaires
	 * des gestes, `writeUrl`) : elle sert aussi la présentation (46-2) et l'application d'une adresse.
	 *
	 * @param options.keepDisplay — Story 46-2 : l'affichage précédent reste jusqu'à la réponse, grisé
	 *   (AC-3 ; 55-1a AC-24). Par défaut : vrai si l'onglet affiche déjà un rapport.
	 * @param options.classicQuery — Story 55-1a : la requête d'un rapport classique ; par défaut, les champs.
	 */
	async function generate(
		options: { keepDisplay?: boolean; classicQuery?: ReportQuery } = {},
	): Promise<void> {
		const myTab = activeTab;
		// Story 24-1 — grand livre : période libre, sans exercice.
		if (isLedgerTab) {
			const q = ledgerQuery();
			if (!q) return;
			const mySeq = invalidateInFlight();
			inFlightTab = myTab;
			inFlightKey = ledgerKey(q);
			loading = true;
			errorMsg = null;
			try {
				const result = await getGeneralLedger(q);
				if (mySeq === genSeq) {
					generalLedger = result;
					displayedLedger = q;
					displayedKeys[myTab] = ledgerKey(q);
				}
			} catch (e) {
				if (mySeq === genSeq) errorMsg = formatError(e);
			} finally {
				if (mySeq === genSeq) {
					loading = false;
					inFlightTab = null;
					inFlightKey = null;
					afterGeneration(myTab);
				}
			}
			return;
		}
		// Story 21-7 — balance âgée : pas de FY/période (arrêté à aujourd'hui).
		if (isAgedTab) {
			const mySeq = invalidateInFlight();
			inFlightTab = myTab;
			inFlightKey = buildReportSearch({ tab: myTab });
			loading = true;
			errorMsg = null;
			try {
				const result = await getAgedReceivables();
				if (mySeq === genSeq) {
					agedReceivables = result;
					displayedKeys[myTab] = buildReportSearch({ tab: myTab });
				}
			} catch (e) {
				if (mySeq === genSeq) errorMsg = formatError(e);
			} finally {
				if (mySeq === genSeq) {
					loading = false;
					inFlightTab = null;
					inFlightKey = null;
					afterGeneration(myTab);
				}
			}
			return;
		}
		// Story 19-6a/b — branche projet (query différente des rapports classiques).
		if (isProjectTab) {
			const q = projectQuery();
			if (!q) return;
			const mySeq = invalidateInFlight();
			inFlightTab = myTab;
			loading = true;
			errorMsg = null;
			try {
				if (activeTab === 'project-expenses') {
					const result = await getProjectExpenses(q);
					if (mySeq === genSeq) {
						projectExpenses = result;
						displayedProject['project-expenses'] = q;
					}
				} else {
					const result = await getProjectReturn(q);
					if (mySeq === genSeq) {
						projectReturn = result;
						displayedProject['project-return'] = q;
					}
				}
			} catch (e) {
				if (mySeq === genSeq) errorMsg = formatError(e);
			} finally {
				if (mySeq === genSeq) {
					loading = false;
					inFlightTab = null;
					afterGeneration(myTab);
				}
			}
			return;
		}
		if (!isClassicTab(myTab)) return;
		const query = options.classicQuery ?? classicQueryOfFields();
		if (!query) return;
		const keepDisplay = options.keepDisplay ?? isDisplayed(myTab);
		const mySeq = invalidateInFlight();
		// Story 55-1b — la présentation envoyée fait partie de la clé en vol (onglets groupés).
		const sentGrouping = isGroupedReport(myTab) ? groupingParamsOf(groupingViews[myTab]) : undefined;
		const requestKey = classicKey(
			myTab,
			query,
			isGroupedReport(myTab) ? presentationOfParams(sentGrouping) : undefined,
		);
		inFlightTab = myTab;
		inFlightKey = requestKey;
		inFlightClassic = query;
		if (keepDisplay) refreshing = true;
		else loading = true;
		errorMsg = null;
		const accept = () => {
			displayedKeys[myTab] = classicKey(myTab, query);
			displayedClassic[myTab] = query;
		};
		/** Story 55-1b — un rapport groupé accepté : sa clé porte la présentation de la réponse (écho compris). */
		const acceptGroupedKey = (report: GroupedReport) => {
			displayedKeys[report] = classicKey(report, query, presentationOfParams(displayedGrouping[report]));
			writeUrlAtResponse(report, requestKey);
		};
		try {
			switch (myTab) {
				case 'balance-sheet': {
					const params = sentGrouping;
					const result = await getBalanceSheet(query, params);
					if (mySeq === genSeq) {
						balanceSheet = result;
						accept();
						acceptGrouped('balance-sheet', params, result);
						acceptGroupedKey('balance-sheet');
					}
					break;
				}
				case 'income-statement': {
					const params = sentGrouping;
					const result = await getIncomeStatement(query, params);
					if (mySeq === genSeq) {
						incomeStatement = result;
						accept();
						acceptGrouped('income-statement', params, result);
						acceptGroupedKey('income-statement');
					}
					break;
				}
				case 'trial-balance': {
					const params = sentGrouping;
					const result = await getTrialBalance(query, params);
					if (mySeq === genSeq) {
						trialBalance = result;
						accept();
						acceptGrouped('trial-balance', params, result);
						acceptGroupedKey('trial-balance');
					}
					break;
				}
				case 'journals': {
					const result = await getJournalReport(query);
					if (mySeq === genSeq) {
						journalReport = result;
						accept();
					}
					break;
				}
				case 'vat': {
					const result = await getVatReport(query);
					if (mySeq === genSeq) {
						vatReport = result;
						accept();
					}
					break;
				}
			}
		} catch (e) {
			if (mySeq === genSeq) errorMsg = formatError(e);
		} finally {
			if (mySeq === genSeq) {
				loading = false;
				refreshing = false;
				inFlightTab = null;
				inFlightKey = null;
				inFlightClassic = null;
				afterGeneration(myTab);
			}
		}
	}

	/** L'onglet affiche-t-il un rapport (en mémoire) ? */
	function isDisplayed(tab: TabId): boolean {
		switch (tab) {
			case 'balance-sheet':
				return balanceSheet !== null;
			case 'income-statement':
				return incomeStatement !== null;
			case 'trial-balance':
				return trialBalance !== null;
			case 'journals':
				return journalReport !== null;
			case 'vat':
				return vatReport !== null;
			case 'general-ledger':
				return generalLedger !== null;
			case 'aged-receivables':
				return agedReceivables !== null;
			case 'project-expenses':
				return projectExpenses !== null;
			case 'project-return':
				return projectReturn !== null;
		}
	}

	/**
	 * Story 55-1a (D16, AC-20, AC-22) — montre le rapport de l'onglet actif POUR LES CHAMPS : rien si
	 * les champs ne le demandent pas, ou si c'est déjà la demande en vol ; repris de la mémoire si c'est
	 * le rapport affiché — et une demande de CET onglet en vol avec une autre clé est alors périmée
	 * (l'écran garde le rapport des champs, R-1, R-17) ; demandé sinon. Rend vrai si une requête est partie.
	 *
	 * @param options.byClick — Story 55-1b (R-3, AC-5) : au CLIC sur un onglet groupé, un rapport affiché qui
	 *   ne diffère des champs que par la présentation est repris de la mémoire, sans requête : la règle de
	 *   la 46-2 (`resyncActiveGroupedTab`) le redemande, tout de suite ou à la fin de la génération d'un
	 *   autre onglet, qu'elle n'interrompt pas. Une adresse (Retour, lien) le demande, elle (55-1a, AC-13).
	 */
	function loadActiveIfNeeded(options: { byClick?: boolean } = {}): boolean {
		const tab = activeTab;
		const key = fieldsKey(tab);
		if (key === null) return false;
		if (inFlightTab === tab && inFlightKey === key) return false;
		const shown = displayedKeys[tab];
		if (shown === key) {
			if (inFlightTab === tab) invalidateInFlight();
			return false;
		}
		if (
			options.byClick &&
			isGroupedReport(tab) &&
			inFlightTab !== tab &&
			shown !== undefined &&
			reportPart(shown) === reportPart(key)
		) {
			return false;
		}
		void generate();
		return true;
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

	// Story 55-1a (AC-25, R-20) — les exports portent sur le rapport AFFICHÉ (sa requête), et sont
	// désactivés pendant sa demande et tant que les champs diffèrent de lui : un clic sur un export après
	// des dates tapées (qui fait quitter les dates, donc recharge) n'exporte jamais l'ancienne période.
	const activeBusy = $derived((loading || refreshing) && inFlightTab === activeTab);
	// AC-24 — un rechargement (affichage gardé) de l'onglet actif : le rapport est grisé.
	const reloadingActive = $derived(refreshing && inFlightTab === activeTab);
	// AC-26 — « Plusieurs » ou « Hors exercice » sur un onglet à un exercice : rapport retiré de l'écran.
	const hiddenByMenu = $derived(
		isClassicTab(activeTab) && data.fiscalYears.length > 0 && menu.kind !== 'fy',
	);
	// Story 55-1b — la présentation n'y entre pas : l'export reprend celle de la réponse affichée
	// (`displayedGrouping`, 46-2 AC-6), même quand les contrôles en demandent une autre (redemande en erreur).
	let canExport = $derived.by(() => {
		const shown = displayedKeys[activeTab];
		const fields = fieldsKey(activeTab);
		return (
			!activeBusy &&
			!exporting &&
			isClassicTab(activeTab) &&
			activeReportPeriod() !== null &&
			shown !== undefined &&
			fields !== null &&
			reportPart(shown) === reportPart(fields)
		);
	});

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
		// Story 24-1 — branche grand livre (query from/to, pas d'exercice).
		if (isLedgerTab) {
			// Story 55-1a (AC-25) — la requête du rapport affiché, non les champs.
			const q = displayedLedger;
			if (!q || generalLedger === null) return;
			exporting = true;
			errorMsg = null;
			try {
				const typeSlug = i18nMsg('reports-filename-general-ledger', 'grand-livre');
				const companySlug = data.companyName.replace(/[^a-zA-Z0-9]+/g, '-').slice(0, 20);
				const filename = `kesh-${typeSlug}-${companySlug}-${q.from}_${q.to}.${format}`;
				await downloadGeneralLedger(q, format, filename);
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
		// Story 19-6a/b — branche projet.
		if (isProjectTab) {
			// Story 55-1a (AC-25, R-10) — la requête du rapport affiché, non les champs.
			const q = displayedProject[activeTab as ProjectReportType] ?? null;
			const hasReport =
				activeTab === 'project-expenses' ? projectExpenses !== null : projectReturn !== null;
			if (!q || !hasReport) return;
			exporting = true;
			errorMsg = null;
			try {
				// i18n slug, valeur entière du catalogue (non coupée). ⚠️ C'est le SEUL nom
				// que reçoit le navigateur : `triggerDownload` nomme le blob lui-même et le
				// `Content-Disposition` du serveur (`kesh-{type}-{société}-{début}_{fin}`)
				// n'est pas lu ; les deux noms diffèrent (période, forme de la société).
				// Story 36-15, revue P1 E1-3 — l'alignement est tracé à part.
				const slugKey =
					activeTab === 'project-expenses'
						? 'reports-filename-project-expenses'
						: 'reports-filename-project-return';
				const slugFallback =
					activeTab === 'project-expenses'
						? 'depenses-par-compte-analytique'
						: 'rendement-par-compte-analytique';
				const typeSlug = i18nMsg(slugKey, slugFallback);
				const companySlug = data.companyName.replace(/[^a-zA-Z0-9]+/g, '-').slice(0, 20);
				const filename = `kesh-${typeSlug}-${companySlug}.${format}`;
				await downloadProjectReport(activeTab as ProjectReportType, q, format, filename);
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
		if (!canExport || !isClassicTab(activeTab)) return;
		const period = activeReportPeriod();
		// Story 55-1a (AC-25) — la requête du rapport affiché, non les champs.
		const displayedQuery = displayedClassic[activeTab];
		if (!period || !displayedQuery) return;
		exporting = true;
		errorMsg = null;
		try {
			const query: ReportQuery | JournalReportQuery = { ...displayedQuery };
			if (activeTab === 'journals') {
				// Aucun filtre journal exposé v0.1 — laisse undefined.
			}
			const filename = buildExportFilename(
				activeTab as ReportType,
				data.companyName,
				{ start: period.startDate, end: period.endDate },
				format,
			);
			// Story 46-2 (AC-6) — les paramètres de la dernière réponse affichée.
			const grouping = isGroupedTab
				? displayedGrouping[activeTab as GroupedReport]
				: undefined;
			await downloadReport(activeTab as ReportType, query, format, filename, grouping);
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

	/**
	 * Story 21-7 — export CSV dédié de la balance âgée (endpoint distinct,
	 * Comptable+). Guard `exporting` re-entrant avant tout await (patron
	 * `exportReport`).
	 */
	async function exportAgedCsv(): Promise<void> {
		if (exporting) return;
		exporting = true;
		errorMsg = null;
		try {
			await downloadAgedReceivables();
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
			fallback: 'Dépenses par compte analytique',
		},
		{
			id: 'project-return',
			labelKey: 'reports-project-return',
			fallback: 'Rendement par compte analytique',
		},
		{ id: 'aged-receivables', labelKey: 'reports-aged-balance', fallback: 'Balance âgée' },
		{ id: 'general-ledger', labelKey: 'reports-ledger-tab', fallback: 'Grand livre' },
	];

	// =====================================================================================
	// Story 55-1a (D1 à D5) — l'URL décrit, pour l'onglet affiché, le rapport demandé.
	//
	// Deux sens, jamais mêlés : la page ÉCRIT l'URL à ses propres gestes (`writeUrl`, après avoir
	// changé son état), et enregistre la recherche écrite comme « déjà traitée » ; elle APPLIQUE
	// l'URL (`applyUrl`) seulement quand `page.url` porte une recherche qu'elle n'a pas écrite :
	// chargement, Retour, Suivant, lien (de compte ou d'ailleurs). Remplace l'`onMount` de la 21-7 (qui
	// ne lisait `?tab=` qu'une fois) et l'effet du lien du grand livre de la 24-1 (qui n'agissait que pour
	// le grand livre : *Retour* changeait l'adresse sans changer l'écran).
	// =====================================================================================

	/** La dernière recherche écrite ou appliquée par la page ; jamais consommée. */
	let lastHandledKey: string | null = null;

	/**
	 * Écrit l'URL d'un geste : `replace` faux empile une entrée d'historique (clic sur un onglet), vrai la
	 * remplace (chargement, flèches, normalisation). Aucun `goto` si l'adresse est déjà celle-là.
	 */
	function writeUrl(search: string, replace: boolean): void {
		lastHandledKey = search;
		if (page.url.search.replace(/^\?/, '') === search) return;
		void goto(`?${search}`, { replaceState: replace, keepFocus: true, noScroll: true });
	}

	/**
	 * Applique une adresse que la page n'a pas écrite : l'onglet ; les champs de cet onglet (une URL
	 * incomplète n'en modifie aucun) ; puis le rapport des champs, repris de la mémoire ou demandé
	 * (`loadActiveIfNeeded`) ; enfin, l'adresse normalisée (D5 : valeurs invalides retirées, sans entrée
	 * d'historique). Les rapports par compte analytique ne sont jamais touchés (C-55-1-7).
	 */
	function applyUrl(sp: URLSearchParams): void {
		const p = parseReportUrl(
			sp,
			data.fiscalYears.map((f) => f.id),
		);
		const tab: TabId = p.tab ?? 'balance-sheet';
		activeTab = tab;
		errorMsg = null;
		let requested = false;
		if (isClassicTab(tab)) {
			const r = resolveClassicPeriod(p, data.fiscalYears);
			if (r) setPeriod(r.from, r.to);
			// Story 55-1b (D4) — une adresse complète d'un onglet groupé pose sa présentation (URL d'abord) ;
			// une adresse incomplète ne change pas la vue.
			if (r?.fy != null && isGroupedReport(tab)) applyPresentation(tab, p.presentation);
			requested = loadActiveIfNeeded();
		} else if (tab === 'general-ledger') {
			// Le grand livre ne se charge seul que sur une adresse complète (sa sortie de la 55-2).
			if (p.from && p.to) {
				ledgerFrom = p.from;
				ledgerTo = p.to;
				ledgerAccountId = p.accountId;
				ledgerIncludeZero = p.includeZero;
				requested = loadActiveIfNeeded();
			}
		} else if (tab === 'aged-receivables') {
			requested = loadActiveIfNeeded();
		}
		const canonical =
			tab === 'general-ledger'
				? p.from && p.to
					? (fieldsKey(tab) ?? buildReportSearch({ ...p, tab }))
					: buildReportSearch({ ...p, tab })
				: urlForTab(tab);
		// Règle de la 46-2 (V8-1) : un rapport groupé repris de la mémoire est remis d'accord.
		if (!requested) resyncActiveGroupedTab();
		writeUrl(canonical, true);
	}

	/**
	 * Story 55-1b (D4, C-55-1b-1) — la présentation d'une adresse complète devient celle des contrôles (et
	 * de la mémoire du navigateur, `setGroupingView`) : `view=flat` masque la hiérarchie en gardant les
	 * autres choix ; une présentation valide est prise tout entière (champs absents : vue par défaut) ;
	 * sans présentation valide, la vue des contrôles reste (elle porte déjà la mémoire).
	 */
	function applyPresentation(tab: GroupedReport, presentation: Presentation | null): void {
		if (presentation === null) return;
		const current = groupingViews[tab];
		const next: GroupingView =
			presentation === 'flat'
				? { ...current, shown: false }
				: {
						shown: true,
						detailLevel: presentation.detailLevel,
						subtotalPosition: presentation.subtotalPosition,
						open: [...presentation.open],
						closed: [...presentation.closed],
					};
		if (JSON.stringify(next) !== JSON.stringify(current)) setGroupingView(tab, next);
	}

	$effect(() => {
		const search = page.url.search.replace(/^\?/, '');
		untrack(() => {
			if (search === lastHandledKey) return;
			lastHandledKey = search;
			applyUrl(new URLSearchParams(search));
		});
	});

	// P6 — ARIA tabs : keyboard navigation (ArrowLeft/Right/Home/End).
	function handleTabKeydown(event: KeyboardEvent, index: number): void {
		let nextIndex: number | null = null;
		if (event.key === 'ArrowRight') nextIndex = (index + 1) % tabs.length;
		else if (event.key === 'ArrowLeft') nextIndex = (index - 1 + tabs.length) % tabs.length;
		else if (event.key === 'Home') nextIndex = 0;
		else if (event.key === 'End') nextIndex = tabs.length - 1;
		if (nextIndex !== null) {
			event.preventDefault();
			// Story 55-1a (C-55-1-5) — les flèches REMPLACENT l'entrée : une par flèche rendrait Retour inutilisable.
			selectTab(tabs[nextIndex].id, true);
			const btn = document.getElementById(`reports-tab-${tabs[nextIndex].id}`);
			btn?.focus();
		}
	}

	/**
	 * Sélectionne un onglet et efface `errorMsg` (Pass 1 code-review M10). Story 55-1a (D2, AC-22) : le
	 * rapport de l'onglet pour les champs s'affiche — repris de la mémoire, ou chargé (onglets classiques
	 * et balance âgée ; le grand livre et les rapports par compte analytique gardent *Générer*) ; puis
	 * l'URL est écrite, **en dernier** : empilée pour un clic (renverse D-7c de la 21-7, demande de
	 * l'utilisateur), remplacée pour les flèches.
	 */
	function selectTab(next: TabId, replace = false): void {
		if (activeTab === next) return;
		activeTab = next;
		errorMsg = null;
		const requested =
			(isClassicTab(next) || next === 'aged-receivables') && loadActiveIfNeeded({ byClick: true });
		// Story 55-1b — l'adresse est calculée AVANT la règle de la 46-2 : une redemande de présentation
		// n'entre dans l'URL que par sa réponse (le clic empile la clé affichée).
		const url = urlForTab(next);
		// Story 46-2, revue P1 M2 (b) — retour sur un rapport groupé dont la redemande a été jetée :
		// l'écran est remis d'accord avec ses contrôles (ou, si une génération est en vol, à sa fin).
		if (!requested) resyncActiveGroupedTab();
		writeUrl(url, replace);
	}

	// ----- Gestes de la période (D15, D16) -------------------------------------------------

	/** Un geste sur la période (menu, sortie des dates, Entrée, « Exercice entier ») : charger si demandable. */
	function commitPeriod(): void {
		periodInvalid =
			estDateStockable(periodStart) && estDateStockable(periodEnd) && periodStart > periodEnd;
		if (!isClassicTab(activeTab)) return;
		if (loadActiveIfNeeded()) writeUrl(urlForTab(activeTab), true);
	}

	/** Choix d'un exercice dans le menu : ses bornes, puis le chargement (AC-15, AC-20). */
	function selectFiscalYear(id: number): void {
		const fy = data.fiscalYears.find((f) => f.id === id);
		if (!fy) return;
		setPeriod(fy.startDate, fy.endDate);
		commitPeriod();
	}

	/** « Exercice entier » : les bornes de l'exercice du menu. */
	function wholeFiscalYear(): void {
		if (menu.kind === 'fy') selectFiscalYear(menu.id);
	}

	/** Une date saisie : champs et menu mis à jour, rien n'est chargé avant la sortie ou Entrée. */
	function onDatesInput(from: string, to: string): void {
		periodStart = from;
		periodEnd = to;
		periodInvalid = false;
		syncMenu();
	}

	// R-19 (DE-4) — sous Safari, un clic ne donne pas le focus à un bouton : la sortie des dates vers un
	// onglet arrive alors avec un `relatedTarget` nul. Le drapeau, posé au `pointerdown` sur la liste
	// des onglets, dit que le clic va à un onglet, qui chargera lui-même avec les dates tapées.
	let pointerOnTablist = false;
	let tablistEl = $state<HTMLElement | null>(null);

	/** Sortie du groupe des dates, ou Entrée. Vers un onglet : rien, l'onglet visé se charge (AC-22). */
	function onPeriodCommit(via: 'leave' | 'enter', next?: EventTarget | null): void {
		if (via === 'leave') {
			if (pointerOnTablist) return;
			if (next instanceof Node && tablistEl?.contains(next)) return;
		}
		commitPeriod();
	}

	/** Grand livre (24-1) — *Générer* : la demande, puis l'URL de ses champs, remplacée. */
	function generateLedger(): void {
		void generate();
		const key = fieldsKey('general-ledger');
		if (key) writeUrl(key, true);
	}
</script>

<!-- R-19 : le drapeau du `pointerdown` sur la liste des onglets tombe au relâchement, voir `onPeriodCommit`. -->
<svelte:window
	onpointerup={() => (pointerOnTablist = false)}
	onpointercancel={() => (pointerOnTablist = false)}
/>

<svelte:head>
	<title>Kesh — Rapports</title>
</svelte:head>

<div class="space-y-4 p-4">
	<h1 class="text-2xl font-bold">{i18nMsg('reports-page-title', 'Rapports comptables')}</h1>

	{#if !isProjectTab && !isAgedTab && !isLedgerTab}
		<ReportSelector
			fiscalYears={data.fiscalYears}
			{menu}
			{periodStart}
			{periodEnd}
			{periodInvalid}
			onFiscalYearSelect={selectFiscalYear}
			{onDatesInput}
			{onPeriodCommit}
			onWholeFiscalYear={wholeFiscalYear}
			onExportPdf={exportPdf}
			onExportCsv={exportCsv}
			{canExport}
			{exporting}
		/>
	{:else if isLedgerTab}
		<!-- Story 24-1 — contrôles du grand livre. PAS de sélecteur d'exercice :
		     ce rapport franchit la borne, c'est ce qui le rend concordant avec le
		     bilan. La période est donc libre. -->
		<div
			class="flex flex-wrap items-end gap-3 rounded border border-border bg-surface p-3"
			data-testid="ledger-controls"
		>
			<label class="text-sm">
				{i18nMsg('reports-ledger-from', 'Du')}
				<input
					type="date"
					class="mt-1 block rounded border border-border px-2 py-1 text-sm"
					bind:value={ledgerFrom}
					data-testid="ledger-from"
				/>
			</label>
			<label class="text-sm">
				{i18nMsg('reports-ledger-to', 'Au')}
				<input
					type="date"
					class="mt-1 block rounded border border-border px-2 py-1 text-sm"
					bind:value={ledgerTo}
					data-testid="ledger-to"
				/>
			</label>
			<label class="text-sm">
				{i18nMsg('reports-ledger-account-label', 'Compte')}
				<select
					class="mt-1 block rounded border border-border px-2 py-1 text-sm"
					bind:value={ledgerAccountId}
					data-testid="ledger-account"
				>
					<option value={null}
						>{i18nMsg('reports-ledger-all-accounts', 'Tous les comptes mouvementés')}</option
					>
					{#each accounts as a (a.id)}
						<option value={a.id}
							>{a.number} — {a.name}{a.active
								? ''
								: ` (${i18nMsg('reports-ledger-archived', 'archivé')})`}</option
						>
					{/each}
				</select>
			</label>
			<label class="flex items-center gap-2 text-sm">
				<input type="checkbox" bind:checked={ledgerIncludeZero} data-testid="ledger-include-zero" />
				{i18nMsg('reports-ledger-include-zero', 'Inclure les comptes sans mouvement')}
			</label>
			<button
				type="button"
				class="rounded bg-primary px-3 py-1 text-sm text-white disabled:opacity-50"
				onclick={generateLedger}
				disabled={!ledgerFrom || !ledgerTo}
				data-testid="ledger-generate"
			>
				{i18nMsg('reports-button-generate', 'Générer')}
			</button>
			<button
				type="button"
				class="rounded border border-border px-3 py-1 text-sm disabled:opacity-50"
				onclick={exportPdf}
				disabled={exporting || generalLedger === null}
				data-testid="ledger-export-pdf"
			>
				PDF
			</button>
			<button
				type="button"
				class="rounded border border-border px-3 py-1 text-sm disabled:opacity-50"
				onclick={exportCsv}
				disabled={exporting || generalLedger === null}
				data-testid="ledger-export-csv"
			>
				CSV
			</button>
		</div>
	{:else if isProjectTab}
		<!-- Story 19-6a — contrôles dédiés des rapports par compte analytique. Story 55-1a (AC-15) : l'exercice
		     et les dates sont ceux de ReportSelector (la requête lit la période des rapports classiques) ;
		     en mode « Cumulé », menu et dates sont masqués. Ces rapports gardent *Générer* (R-15 c). -->
		<div data-testid="project-report-controls">
			<ReportSelector
				fiscalYears={data.fiscalYears}
				{menu}
				{periodStart}
				{periodEnd}
				{periodInvalid}
				showPeriod={projectMode === 'fiscal_year'}
				onFiscalYearSelect={selectFiscalYear}
				{onDatesInput}
				{onPeriodCommit}
				onWholeFiscalYear={wholeFiscalYear}
			>
				{#snippet controls()}
					<label class="block text-sm">
						{i18nMsg('reports-project-selector-label', 'Compte analytique')}
						<select
							class="mt-1 block w-full rounded border border-border px-2 py-1 text-sm"
							bind:value={selectedProjectId}
							data-testid="project-report-project"
						>
							<option value={null}>{i18nMsg('reports-project-selector-placeholder', '— Choisir un compte analytique')}</option>
							{#each projects.filter((p) => p.parentId === null) as root (root.id)}
								<option value={root.id}>{root.code} — {root.name}</option>
								{#each projects.filter((c) => c.parentId === root.id) as child (child.id)}
									<option value={child.id}>&nbsp;&nbsp;↳ {child.code} — {child.name}</option>
								{/each}
							{/each}
						</select>
					</label>

					<label class="block text-sm sm:col-span-2">
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
				{/snippet}
				{#snippet actions()}
					<button
						type="button"
						class="rounded bg-primary px-3 py-1 text-sm text-white disabled:opacity-50"
						onclick={() => generate()}
						disabled={projectQuery() === null}
						data-testid="project-report-generate"
					>
						{i18nMsg('reports-generate', 'Générer')}
					</button>
					<button
						type="button"
						class="rounded border border-border px-3 py-1 text-sm disabled:opacity-50"
						onclick={exportPdf}
						disabled={exporting || !hasProjectReport || activeBusy}
						data-testid="project-report-export-pdf"
					>
						PDF
					</button>
					<button
						type="button"
						class="rounded border border-border px-3 py-1 text-sm disabled:opacity-50"
						onclick={exportCsv}
						disabled={exporting || !hasProjectReport || activeBusy}
						data-testid="project-report-export-csv"
					>
						CSV
					</button>
				{/snippet}
			</ReportSelector>
		</div>
	{:else}
		<!-- Story 21-7 — contrôles balance âgée : pas de FY/période (arrêté à aujourd'hui).
		     Ne branche NI ReportSelector NI les exports génériques (activeReportPeriod null).
		     Story 55-1a (D16, R-15 b) : chargée à l'arrivée sur l'onglet, plus de bouton Générer. -->
		<div class="flex flex-wrap items-center gap-3 rounded border border-border bg-surface p-3" data-testid="aged-report-controls">
			<span class="text-sm text-text-muted">
				{i18nMsg('reports-aged-instruction', 'Balance âgée arrêtée à ce jour.')}
			</span>
			{#if canManage}
				<button
					type="button"
					class="rounded border border-border px-3 py-1 text-sm disabled:opacity-50"
					onclick={exportAgedCsv}
					disabled={exporting || agedReceivables === null}
					data-testid="aged-report-export-csv"
				>
					{i18nMsg('reports-export-csv-button', 'Exporter CSV')}
				</button>
			{/if}
		</div>
	{/if}

	<div
		role="tablist"
		class="flex border-b"
		aria-label={i18nMsg('reports-page-title', 'Rapports comptables')}
		bind:this={tablistEl}
	>
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
				onpointerdown={() => (pointerOnTablist = true)}
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
		aria-busy={activeBusy}
		class="space-y-3"
	>
		{#if activeGroupingView}
			<GroupingControls
				view={activeGroupingView}
				maxDepth={activeMaxDepth}
				onChange={changeGrouping}
				levelHint={activeTab === 'trial-balance'
					? i18nMsg(
							'reports-groups-level-hint-trial-balance',
							"Le total des débits et celui des crédits restent ceux de tous les comptes. Un groupe dont les comptes non nuls sont de natures contraires (débitrice et créditrice) n'est pas totalisé : ses sous-comptes paraissent à sa place. Un affichage replié est un affichage de travail.",
						)
					: undefined}
			/>
		{/if}
		{#if loading && inFlightTab === activeTab}
			<p class="text-sm italic text-gray-500" role="status" data-testid="report-loading">
				{i18nMsg('reports-loading', 'Chargement…')}
			</p>
		{:else if hiddenByMenu}
			<!-- AC-26 : « Plusieurs » ou « Hors exercice » sur un onglet à un exercice — le rapport, gardé en
			     mémoire, est retiré de l'écran ; le message est sous les dates (ReportSelector). -->
		{:else}
			<!-- AC-24 : pendant un rechargement, l'ancien rapport reste, grisé ; ses liens restent utilisables. -->
			<div class={reloadingActive ? 'opacity-60' : ''} data-testid="report-content">
				{#if activeTab === 'balance-sheet' && balanceSheet}
					<BalanceSheetView dto={balanceSheet} {onToggleGroup} fiscalYears={data.fiscalYears} />
				{:else if activeTab === 'income-statement' && incomeStatement}
					<IncomeStatementView dto={incomeStatement} {onToggleGroup} fiscalYears={data.fiscalYears} />
				{:else if activeTab === 'trial-balance' && trialBalance}
					<TrialBalanceView dto={trialBalance} {onToggleGroup} fiscalYears={data.fiscalYears} />
				{:else if activeTab === 'journals' && journalReport}
					<JournalReportView dto={journalReport} fiscalYears={data.fiscalYears} />
				{:else if activeTab === 'vat' && vatReport}
					<VatReportView dto={vatReport} fiscalYears={data.fiscalYears} />
				{:else if activeTab === 'project-expenses' && projectExpenses}
					<ProjectExpensesView report={projectExpenses} />
				{:else if activeTab === 'project-return' && projectReturn}
					<ProjectReturnView report={projectReturn} />
				{:else if activeTab === 'aged-receivables' && agedReceivables}
					<AgedReceivablesView dto={agedReceivables} />
				{:else if activeTab === 'general-ledger' && generalLedger}
					<GeneralLedgerView dto={generalLedger} {letterableAccountIds} fiscalYears={data.fiscalYears} />
				{:else if isLedgerTab}
					<p class="text-sm italic text-gray-500">
						{i18nMsg(
							'reports-ledger-instruction',
							'Choisissez une période, puis cliquez sur Générer. Le compte est facultatif.',
						)}
					</p>
				{/if}
			</div>
		{/if}
	</div>
</div>
