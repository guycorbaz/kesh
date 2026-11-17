<!--
  Story 37-2 (AC-6) — écran *Cours de change* : tableau annuel des cours d'un
  exercice multi-devises.

  - Une ligne par devise ; une colonne par mois de l'exercice, lue dans `months`
    (aucune constante de douze), puis Ouverture et Clôture selon les genres :
    genre `day`, la case du jour ; genre `month`, la case du mois — la Clôture liée
    à la dernière colonne, l'Ouverture liée à la première quand l'exercice commence
    en cours de mois, sinon une case propre (dernier mois de l'exercice précédent).
  - Une valeur par case (cours retenu par la société), modifiable par
    l'Administrateur et le Comptable ; mention de la source et de la valeur publiée ;
    ↑ (`RateCell`). Exercice clos et rôle Consultation : lecture seule — mais
    « Récupérer les cours manquants » reste offert sur un exercice clos.
  - Choix des deux genres (`PUT /rate-kinds`), avec le dialogue U4 quand un
    exercice suivant existe ; dialogue U3 sous la condition
    `canManageSeveralCompanies` (fausse aujourd'hui : ↑ direct).
  - « Récupérer les cours manquants » et « Revérifier les cours publiés » : un
    `POST /fetch` par colonne, l'un après l'autre ; un échec s'affiche sur les cases
    de sa colonne sans arrêter les suivantes ; puis relecture.
  - Défilement horizontal du seul tableau, colonne des devises figée.
-->
<script lang="ts">
	import { onMount } from 'svelte';
	import * as Dialog from '$lib/components/ui/dialog';
	import { Button } from '$lib/components/ui/button';
	import { authState } from '$lib/app/stores/auth.svelte';
	import { isApiError } from '$lib/shared/utils/api-client';
	import { errorMessageOf } from '$lib/shared/utils/error-message';
	import { i18nLocale, i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { dayBefore, todayInZurich } from '$lib/shared/utils/local-date';
	import { FOREIGN_CURRENCIES } from '$lib/shared/utils/iso4217';
	import { formatSwissDate } from '$lib/features/reports/reports.api';
	import { fetchCompanyCurrent } from '$lib/features/settings/settings.api';
	import { listFiscalYears } from '$lib/features/fiscal-years/fiscal-years.api';
	import type { FiscalYearResponse } from '$lib/features/fiscal-years/fiscal-years.types';
	import {
		fetchRates,
		getFetchStatus,
		getYearGrid,
		putRate,
		putRateKinds,
		resetRate,
	} from './exchange-rates.api';
	import {
		cellAt,
		columnsToFetch,
		columnsToRecheck,
		defaultFiscalYear,
		emptyRow,
		fetchability,
		fetchRequestOf,
		formatRate,
		gridColumns,
		isLinkedToMonth,
		mentionOf,
		monthLabel,
		monthLongLabel,
		multiCurrencyFiscalYears,
		nextFiscalYearOf,
		unitLabel,
		type Column,
	} from './exchange-rates-grid';
	import { fetchFailureText, notFetchableText, statusText } from './exchange-rates-labels';
	import { CAN_MANAGE_SEVERAL_COMPANIES, canEditRates } from './exchange-rates-access';
	import RateCell from './RateCell.svelte';
	import CorrectionDialog from './CorrectionDialog.svelte';
	import type {
		Cell,
		FetchStatusResponse,
		FiscalYearRatesResponse,
		FiscalYearRatesRow,
		RateKind,
	} from './exchange-rates.types';

	interface Props {
		/** Date à couvrir par l'exercice affiché (lien du bandeau des corrections). */
		initialDate?: string | null;
		/** Condition unique du dialogue U3 (AC-6, A23) — `false` aujourd'hui. */
		canManageSeveralCompanies?: boolean;
	}

	let { initialDate = null, canManageSeveralCompanies = CAN_MANAGE_SEVERAL_COMPANIES }: Props = $props();

	let loading = $state(true);
	let multiCurrency = $state(false);
	let companyName = $state('');
	let fiscalYears = $state<FiscalYearResponse[]>([]);
	let selectedId = $state<number | null>(null);
	let grid = $state<FiscalYearRatesResponse | null>(null);
	let gridError = $state('');
	/** Échec du chargement initial (société ou exercices) : rendu à la place des états vides. */
	let loadError = $state('');
	let loadCount = $state(0);
	/** Numéro de la dernière lecture du tableau : une réponse tardive est ignorée. */
	let gridSeq = 0;
	let all = $state(false);
	let extraCurrencies = $state<string[]>([]);
	let currencyToAdd = $state('');
	let status = $state<FetchStatusResponse | null>(null);
	let message = $state<{ kind: 'info' | 'error'; text: string } | null>(null);
	let columnErrors = $state<Record<string, string>>({});
	let fetching = $state<'missing' | 'recheck' | null>(null);
	let progress = $state('');
	let savingKinds = $state(false);

	// Dialogue U4 (genre de clôture).
	let kindsDialogOpen = $state(false);
	let pendingClosing = $state<RateKind | null>(null);
	let applyToNext = $state(true);

	// Dialogue U3 (↑ d'une correction).
	let correctionOpen = $state(false);
	let pendingCorrection = $state<{ currency: string; cell: Cell } | null>(null);

	const locale = $derived(i18nLocale());
	const today = todayInZurich();

	let canEdit = $derived(canEditRates(authState.currentUser?.role));
	let selectedFy = $derived(fiscalYears.find((f) => f.id === selectedId) ?? null);
	let fyClosed = $derived(selectedFy?.status === 'Closed');
	let editable = $derived(canEdit && !fyClosed);
	let multiYears = $derived(multiCurrencyFiscalYears(fiscalYears));
	let nextFy = $derived(selectedFy ? nextFiscalYearOf(selectedFy, fiscalYears) : null);
	let columns = $derived(grid ? gridColumns(grid) : null);
	let rows = $derived.by((): FiscalYearRatesRow[] => {
		if (!grid) return [];
		const g = grid;
		const present = new Set(g.rows.map((r) => r.currency));
		const extra = extraCurrencies.filter((c) => !present.has(c)).map((c) => emptyRow(c, g));
		return [...g.rows, ...extra].sort((a, b) => a.currency.localeCompare(b.currency));
	});
	let addable = $derived(FOREIGN_CURRENCIES.filter((c) => !rows.some((r) => r.currency === c)));
	let status_ = $derived(status ? statusText(status, locale) : null);

	function testKey(currency: string, column: Column): string {
		return `${currency}-${column.slot}-${column.rateDate}`;
	}

	function linkedLabel(column: Column): string | null {
		if (!grid || column.slot === 'month') return null;
		return isLinkedToMonth(column.kind, column.rateDate, grid.months)
			? monthLabel(column.rateDate, locale)
			: null;
	}

	function notFetchable(column: Column): string | null {
		const f = fetchability(column.kind, column.rateDate, today);
		return f.ok ? null : notFetchableText(column.kind, f.reason);
	}

	async function loadGrid() {
		if (selectedId === null) return;
		const mine = ++gridSeq;
		gridError = '';
		try {
			const g = await getYearGrid(selectedId, all);
			// Revue de code P1 (37-2-CR-P1-6) : un changement d'exercice ou de
			// « Toutes les devises » pendant la lecture rend celle-ci périmée.
			if (mine !== gridSeq) return;
			grid = g;
			loadCount += 1;
		} catch (e) {
			if (mine !== gridSeq) return;
			grid = null;
			gridError = errorMessageOf(e);
		}
	}

	async function loadStatus() {
		try {
			status = await getFetchStatus();
		} catch {
			status = null;
		}
	}

	async function reloadFiscalYears() {
		try {
			fiscalYears = await listFiscalYears();
		} catch {
			// La liste reste celle d'avant : le tableau se relit quand même.
		}
	}

	onMount(async () => {
		try {
			const company = await fetchCompanyCurrent();
			multiCurrency = company.company.multiCurrencyEnabled;
			companyName = company.company.name;
			if (!multiCurrency) return;
			fiscalYears = await listFiscalYears();
			selectedId = defaultFiscalYear(fiscalYears, today, initialDate)?.id ?? null;
			await Promise.all([loadGrid(), loadStatus()]);
		} catch (e) {
			// Revue de code P1 (37-2-CR-P1-7) : un échec de lecture de la société
			// ou des exercices ne se fait pas passer pour « mono-devise » ou
			// « aucun exercice ».
			loadError = errorMessageOf(e);
		} finally {
			loading = false;
		}
	});

	async function selectFiscalYear(id: number) {
		selectedId = id;
		columnErrors = {};
		message = null;
		await loadGrid();
	}

	/** Un 409 recharge la vue avec un message ; toute autre erreur est affichée. */
	async function handleWriteError(e: unknown) {
		if (isApiError(e) && e.status === 409) {
			message = {
				kind: 'error',
				text: i18nMsg(
					'exchange-rates-conflict',
					'Ce cours a changé entre-temps : le tableau a été rechargé, vérifiez puis réessayez.',
				),
			};
			await loadGrid();
			return;
		}
		message = { kind: 'error', text: errorMessageOf(e) };
		// Revue de code P1 (37-2-CR-P1-5) : la case ou le choix de genre refusé
		// revient à la valeur enregistrée (les cases et les boutons radio se
		// remontent), au lieu de montrer une valeur non enregistrée.
		loadCount += 1;
	}

	async function savePut(currency: string, cell: Cell, rate: string) {
		message = null;
		try {
			await putRate({
				currency,
				kind: cell.kind,
				rateDate: cell.rateDate,
				rate,
				...(cell.row ? { version: cell.row.version } : {}),
			});
			await loadGrid();
		} catch (e) {
			await handleWriteError(e);
		}
	}

	async function saveReset(currency: string, cell: Cell) {
		message = null;
		try {
			await resetRate({
				currency,
				kind: cell.kind,
				rateDate: cell.rateDate,
				...(cell.row ? { version: cell.row.version } : {}),
				...(cell.published ? { publishedRate: cell.published.rate } : {}),
			});
			await loadGrid();
		} catch (e) {
			await handleWriteError(e);
		}
	}

	/** Le ↑ : direct, sauf le ↑ d'une correction quand le dialogue U3 est offert. */
	async function arrow(currency: string, cell: Cell) {
		if (canManageSeveralCompanies && mentionOf(cell).kind === 'afc-corrected') {
			pendingCorrection = { currency, cell };
			correctionOpen = true;
			return;
		}
		await saveReset(currency, cell);
	}

	async function confirmCorrection() {
		const p = pendingCorrection;
		pendingCorrection = null;
		correctionOpen = false;
		if (p) await saveReset(p.currency, p.cell);
	}

	function cancelCorrection() {
		pendingCorrection = null;
		correctionOpen = false;
	}

	async function runFetch(mode: 'missing' | 'recheck') {
		if (!grid || fetching) return;
		const cols = mode === 'missing' ? columnsToFetch(grid, rows, today) : columnsToRecheck(grid, rows, today);
		fetching = mode;
		message = null;
		columnErrors = {};
		const errors: Record<string, string> = {};
		let i = 0;
		for (const c of cols) {
			i += 1;
			progress = i18nMsg('exchange-rates-fetch-progress', 'Récupération { $n } sur { $total }…', {
				n: i,
				total: cols.length,
			});
			try {
				await fetchRates(fetchRequestOf(c));
			} catch (e) {
				errors[c.key] = fetchFailureText(e);
			}
		}
		progress = '';
		columnErrors = errors;
		if (cols.length === 0) {
			message = {
				kind: 'info',
				text:
					mode === 'missing'
						? i18nMsg('exchange-rates-fetch-nothing-missing', 'Aucun cours récupérable ne manque.')
						: i18nMsg('exchange-rates-recheck-nothing', 'Aucun cours publié à revérifier.'),
			};
		}
		await Promise.all([loadGrid(), loadStatus()]);
		fetching = null;
	}

	async function sendRateKinds(opening: RateKind, closing: RateKind, apply: boolean) {
		if (!selectedFy) return;
		savingKinds = true;
		message = null;
		try {
			const r = await putRateKinds(selectedFy.id, {
				openingRateKind: opening,
				closingRateKind: closing,
				applyToNextOpening: apply,
			});
			if (r.nextFiscalYear && !r.nextFiscalYear.applied) {
				const name = fiscalYears.find((f) => f.id === r.nextFiscalYear?.id)?.name ?? '';
				message = {
					kind: 'info',
					text: i18nMsg(
						'exchange-rates-kinds-next-not-applied',
						'L’exercice suivant ({ $name }) est clos : son cours d’ouverture n’a pas été changé.',
						{ name },
					),
				};
			}
			await reloadFiscalYears();
			await loadGrid();
		} catch (e) {
			await handleWriteError(e);
			await reloadFiscalYears();
		} finally {
			savingKinds = false;
		}
	}

	function changeOpening(kind: RateKind) {
		if (!grid || kind === grid.openingRateKind) return;
		void sendRateKinds(kind, grid.closingRateKind, false);
	}

	function changeClosing(kind: RateKind) {
		if (!grid || kind === grid.closingRateKind) return;
		if (nextFy) {
			pendingClosing = kind;
			applyToNext = nextFy.status !== 'Closed';
			kindsDialogOpen = true;
			return;
		}
		void sendRateKinds(grid.openingRateKind, kind, false);
	}

	function confirmClosing() {
		const kind = pendingClosing;
		pendingClosing = null;
		kindsDialogOpen = false;
		if (!grid || kind === null) return;
		void sendRateKinds(grid.openingRateKind, kind, nextFy?.status === 'Closed' ? false : applyToNext);
	}

	function cancelClosing() {
		pendingClosing = null;
		kindsDialogOpen = false;
		// Les boutons radio reviennent au genre enregistré.
		loadCount += 1;
	}

	function addCurrency() {
		if (!currencyToAdd) return;
		extraCurrencies = [...extraCurrencies, currencyToAdd];
		currencyToAdd = '';
	}

	async function toggleAll(value: boolean) {
		all = value;
		await loadGrid();
	}
</script>

<div class="flex flex-col gap-4" data-testid="exchange-rates-screen">
	<h1 class="text-2xl font-semibold">{i18nMsg('exchange-rates-title', 'Cours de change')}</h1>

	<p class="text-sm text-text-muted" data-testid="exchange-rates-source">
		{i18nMsg(
			'exchange-rates-source-note',
			'Source : cours officiels de l’AFC, publiés par l’Office fédéral de la douane et de la sécurité des frontières (OFDF). Le cours mensuel AFC est publié d’avance et vaut pour tout le mois ; le cours journalier est publié les jours ouvrables. Kesh les récupère une fois par jour jusqu’au cours du mois suivant, puis fait une pause jusqu’au plus tôt de « obtention + 24 jours » et du 25 du mois obtenu, jamais avant le 1er de ce mois. Ces cours sont indicatifs et non contraignants : chaque cours retenu peut être remplacé par le vôtre. Les cours s’entendent en CHF pour une unité de devise.',
		)}
	</p>

	{#if loading}
		<p class="text-sm text-text-muted">{i18nMsg('common-loading', 'Chargement…')}</p>
	{:else if loadError}
		<p class="text-sm text-error" data-testid="exchange-rates-load-error">{loadError}</p>
	{:else if !multiCurrency}
		<p class="text-sm" data-testid="exchange-rates-mono">
			{i18nMsg(
				'exchange-rates-mono',
				'Le mode multi-devises n’est pas activé : aucun cours de change n’est tenu.',
			)}
		</p>
	{:else if multiYears.length === 0}
		<p class="text-sm" data-testid="exchange-rates-no-fiscal-year">
			{i18nMsg('exchange-rates-no-fiscal-year', 'Aucun exercice multi-devises.')}
		</p>
	{:else}
		<div class="flex flex-wrap items-end gap-4">
			<label class="flex flex-col gap-1 text-sm">
				<span class="font-medium">{i18nMsg('exchange-rates-fiscal-year', 'Exercice')}</span>
				<select
					class="rounded border border-border px-2 py-1"
					value={selectedId ?? ''}
					onchange={(e) => void selectFiscalYear(Number((e.currentTarget as HTMLSelectElement).value))}
					data-testid="exchange-rates-fiscal-year-select"
				>
					{#each multiYears as fy (fy.id)}
						<option value={fy.id}>{fy.name}</option>
					{/each}
				</select>
			</label>
			{#if fyClosed}
				<span class="text-sm text-text-muted" data-testid="exchange-rates-closed">
					{i18nMsg('exchange-rates-closed', 'Exercice clos : cours en lecture seule.')}
				</span>
			{/if}
		</div>

		{#if status_}
			<p class="text-sm" data-testid="exchange-rates-status">{status_}</p>
		{:else if status}
			<p class="text-sm text-text-muted" data-testid="exchange-rates-status">
				{i18nMsg('exchange-rates-status-none', 'Aucune récupération des cours n’a encore eu lieu.')}
			</p>
		{/if}

		{#if message}
			<p
				role="status"
				class="text-sm {message.kind === 'error' ? 'text-error' : 'text-text'}"
				data-testid="exchange-rates-message"
			>
				{message.text}
			</p>
		{/if}

		{#if gridError}
			<p class="text-sm text-error" data-testid="exchange-rates-grid-error">{gridError}</p>
		{/if}

		{#if grid && columns}
			{#key loadCount}
				<!-- Genres d'ouverture et de clôture (R6, T4, U4) -->
				<div class="grid gap-4 sm:grid-cols-2" data-testid="exchange-rates-kinds">
					<fieldset class="flex flex-col gap-1 text-sm">
						<legend class="font-medium">{i18nMsg('exchange-rates-opening-title', 'Cours d’ouverture')}</legend>
						{#if canEdit}
							<label class="flex items-start gap-2">
								<input
									type="radio"
									name="exchange-rates-opening"
									checked={grid.openingRateKind === 'day'}
									disabled={fyClosed || savingKinds}
									onchange={() => changeOpening('day')}
									data-testid="exchange-rates-opening-day"
								/>
								{i18nMsg(
									'exchange-rates-opening-day',
									'cours journalier de la veille de l’ouverture ({ $date })',
									{ date: formatSwissDate(dayBefore(grid.startDate)) },
								)}
							</label>
							<label class="flex items-start gap-2">
								<input
									type="radio"
									name="exchange-rates-opening"
									checked={grid.openingRateKind === 'month'}
									disabled={fyClosed || savingKinds}
									onchange={() => changeOpening('month')}
									data-testid="exchange-rates-opening-month"
								/>
								{i18nMsg(
									'exchange-rates-opening-month',
									'cours mensuel du dernier mois de l’exercice précédent ({ $month })',
									{ month: monthLongLabel(dayBefore(grid.startDate), locale) },
								)}
							</label>
						{:else}
							<span data-testid="exchange-rates-opening-text">
								{grid.openingRateKind === 'day'
									? i18nMsg(
											'exchange-rates-opening-day',
											'cours journalier de la veille de l’ouverture ({ $date })',
											{ date: formatSwissDate(dayBefore(grid.startDate)) },
										)
									: i18nMsg(
											'exchange-rates-opening-month',
											'cours mensuel du dernier mois de l’exercice précédent ({ $month })',
											{ month: monthLongLabel(dayBefore(grid.startDate), locale) },
										)}
							</span>
						{/if}
						<span class="text-xs text-text-muted">
							{i18nMsg(
								'exchange-rates-opening-help',
								'Le cours d’ouverture est le cours de clôture de l’exercice précédent : ainsi, les valeurs en CHF concordent d’un exercice à l’autre.',
							)}
						</span>
					</fieldset>
					<fieldset class="flex flex-col gap-1 text-sm">
						<legend class="font-medium">{i18nMsg('exchange-rates-closing-title', 'Cours de clôture')}</legend>
						{#if canEdit}
							<label class="flex items-start gap-2">
								<input
									type="radio"
									name="exchange-rates-closing"
									checked={grid.closingRateKind === 'day'}
									disabled={fyClosed || savingKinds}
									onchange={() => changeClosing('day')}
									data-testid="exchange-rates-closing-day"
								/>
								{i18nMsg('exchange-rates-closing-day', 'cours journalier de la date de clôture ({ $date })', {
									date: formatSwissDate(grid.endDate),
								})}
							</label>
							<label class="flex items-start gap-2">
								<input
									type="radio"
									name="exchange-rates-closing"
									checked={grid.closingRateKind === 'month'}
									disabled={fyClosed || savingKinds}
									onchange={() => changeClosing('month')}
									data-testid="exchange-rates-closing-month"
								/>
								{i18nMsg('exchange-rates-closing-month', 'cours mensuel du dernier mois ({ $month })', {
									month: monthLongLabel(grid.endDate, locale),
								})}
							</label>
						{:else}
							<span data-testid="exchange-rates-closing-text">
								{grid.closingRateKind === 'day'
									? i18nMsg(
											'exchange-rates-closing-day',
											'cours journalier de la date de clôture ({ $date })',
											{ date: formatSwissDate(grid.endDate) },
										)
									: i18nMsg('exchange-rates-closing-month', 'cours mensuel du dernier mois ({ $month })', {
											month: monthLongLabel(grid.endDate, locale),
										})}
							</span>
						{/if}
					</fieldset>
				</div>

				{#if canEdit}
					<div class="flex flex-wrap items-center gap-2">
						<Button
							size="sm"
							disabled={fetching !== null}
							onclick={() => void runFetch('missing')}
							data-testid="exchange-rates-fetch-missing"
						>
							{i18nMsg('exchange-rates-fetch-missing', 'Récupérer les cours manquants')}
						</Button>
						<Button
							size="sm"
							variant="outline"
							disabled={fetching !== null}
							onclick={() => void runFetch('recheck')}
							data-testid="exchange-rates-recheck"
						>
							{i18nMsg('exchange-rates-recheck', 'Revérifier les cours publiés')}
						</Button>
						{#if progress}
							<span class="text-sm text-text-muted" data-testid="exchange-rates-progress">{progress}</span>
						{/if}
					</div>
				{/if}

				<div class="flex flex-wrap items-center gap-4 text-sm">
					{#if editable}
						<label class="flex items-center gap-2">
							<span>{i18nMsg('exchange-rates-add-currency', 'Ajouter une devise')}</span>
							<select
								class="rounded border border-border px-2 py-1"
								bind:value={currencyToAdd}
								data-testid="exchange-rates-add-currency-select"
							>
								<option value="">—</option>
								{#each addable as c (c)}
									<option value={c}>{c}</option>
								{/each}
							</select>
						</label>
						<Button
							size="sm"
							variant="outline"
							disabled={!currencyToAdd}
							onclick={addCurrency}
							data-testid="exchange-rates-add-currency"
						>
							{i18nMsg('exchange-rates-add-currency-button', 'Ajouter')}
						</Button>
					{/if}
					<label class="flex items-center gap-2">
						<input
							type="checkbox"
							checked={all}
							onchange={(e) => void toggleAll((e.currentTarget as HTMLInputElement).checked)}
							data-testid="exchange-rates-all"
						/>
						{i18nMsg('exchange-rates-all', 'Toutes les devises publiées')}
					</label>
				</div>

				<div class="max-w-full overflow-x-auto rounded border border-border" data-testid="exchange-rates-table-scroll">
					<table class="text-sm" data-testid="exchange-rates-table">
						<thead>
							<tr class="bg-surface-alt">
								<th class="sticky left-0 z-10 bg-surface-alt px-2 py-1 text-left">
									{i18nMsg('exchange-rates-col-currency', 'Devise')}
								</th>
								{#each columns.months as c (c.key)}
									<th
										class="px-2 py-1 text-left whitespace-nowrap"
										title={notFetchable(c) ?? undefined}
										data-testid="exchange-rates-header-month-{c.rateDate}"
									>
										{monthLabel(c.rateDate, locale)}
									</th>
								{/each}
								<th
									class="px-2 py-1 text-left whitespace-nowrap"
									title={notFetchable(columns.opening) ?? undefined}
									data-testid="exchange-rates-header-opening"
								>
									{i18nMsg('exchange-rates-col-opening', 'Ouverture')}<br />
									<span class="font-normal">{unitLabel(columns.opening.kind, columns.opening.rateDate, locale)}</span>
								</th>
								<th
									class="px-2 py-1 text-left whitespace-nowrap"
									title={notFetchable(columns.closing) ?? undefined}
									data-testid="exchange-rates-header-closing"
								>
									{i18nMsg('exchange-rates-col-closing', 'Clôture')}<br />
									<span class="font-normal">{unitLabel(columns.closing.kind, columns.closing.rateDate, locale)}</span>
								</th>
							</tr>
						</thead>
						<tbody>
							{#each rows as row (row.currency)}
								<tr class="border-t border-border align-top" data-testid="exchange-rates-row-{row.currency}">
									<th class="sticky left-0 z-10 bg-white px-2 py-1 text-left font-medium">{row.currency}</th>
									{#each [...columns.months, columns.opening, columns.closing] as c (c.slot + c.key)}
										{@const cell = cellAt(row, c)}
										<td class="px-2 py-1">
											<RateCell
												{cell}
												currency={row.currency}
												testKey={testKey(row.currency, c)}
												{editable}
												linkedTo={linkedLabel(c)}
												notFetchable={notFetchable(c)}
												fetchError={columnErrors[c.key] ?? null}
												onPut={(rate) => savePut(row.currency, cell, rate)}
												onReset={() => saveReset(row.currency, cell)}
												onArrow={() => arrow(row.currency, cell)}
												onInvalid={(text) => (message = { kind: 'error', text })}
											/>
										</td>
									{/each}
								</tr>
							{:else}
								<tr>
									<td class="px-2 py-2 text-text-muted" colspan={columns.months.length + 3}>
										{i18nMsg('exchange-rates-no-row', 'Aucune devise : aucun compte ne porte de devise étrangère.')}
									</td>
								</tr>
							{/each}
						</tbody>
					</table>
				</div>
			{/key}
		{/if}
	{/if}
</div>

<!-- Dialogue U4 : appliquer le genre de clôture à l'ouverture de l'exercice suivant -->
<Dialog.Root
	bind:open={kindsDialogOpen}
	onOpenChange={(o) => {
		if (!o && pendingClosing !== null) cancelClosing();
	}}
>
	<Dialog.Content data-testid="exchange-rates-kinds-dialog">
		<Dialog.Header>
			<Dialog.Title>{i18nMsg('exchange-rates-kinds-dialog-title', 'Changer le cours de clôture')}</Dialog.Title>
			<Dialog.Description>
				{i18nMsg(
					'exchange-rates-kinds-dialog-body',
					'Entre deux exercices Kesh, aucune écriture d’ouverture n’existe : les soldes de bilan sont reportés d’un exercice à l’autre. Ce choix ne modifie aucune écriture ; il change le cours proposé et affiché à l’ouverture de l’exercice suivant.',
				)}
			</Dialog.Description>
		</Dialog.Header>
		{#if nextFy}
			<label class="flex items-start gap-2 text-sm">
				<input
					type="checkbox"
					bind:checked={applyToNext}
					disabled={nextFy.status === 'Closed'}
					data-testid="exchange-rates-kinds-apply-next"
				/>
				{i18nMsg(
					'exchange-rates-kinds-apply-next',
					'Appliquer aussi au cours d’ouverture de l’exercice suivant ({ $name })',
					{ name: nextFy.name },
				)}
			</label>
			{#if nextFy.status === 'Closed'}
				<p class="text-xs text-text-muted" data-testid="exchange-rates-kinds-next-closed">
					{i18nMsg(
						'exchange-rates-kinds-next-closed',
						'L’exercice suivant est clos : son cours d’ouverture ne peut plus changer.',
					)}
				</p>
			{/if}
		{/if}
		<Dialog.Footer class="mt-4">
			<Button variant="outline" type="button" onclick={cancelClosing} data-testid="exchange-rates-kinds-cancel">
				{i18nMsg('common-cancel', 'Annuler')}
			</Button>
			<Button type="button" onclick={confirmClosing} data-testid="exchange-rates-kinds-save">
				{i18nMsg('common-save', 'Enregistrer')}
			</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>

{#if canManageSeveralCompanies && pendingCorrection}
	{@const m = mentionOf(pendingCorrection.cell)}
	<CorrectionDialog
		bind:open={correctionOpen}
		{companyName}
		retained={formatRate(pendingCorrection.cell.effectiveRate)}
		corrected={formatRate(m.published)}
		correctedOn={m.correctedAt ? formatSwissDate(m.correctedAt) : ''}
		onConfirm={() => void confirmCorrection()}
		onCancel={cancelCorrection}
	/>
{/if}
