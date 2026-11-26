<script lang="ts">
	// Story 9-1 — Sélecteur d'exercice + période. Story 9-2a — boutons d'export.
	// Story 55-1a (D15, D16) — le même composant sur tous les onglets à période :
	//   - un seul libellé « Exercice », par un vrai `<label for>` (plus d'`aria-label` en double) ;
	//   - un menu natif entièrement en gras (`font-semibold`, options `font-normal`), qui montre l'exercice
	//     seul (années ou nom saisi, `libellesExercices`) ; « Plusieurs » et « Hors exercice » en options
	//     désactivées, présentes seulement quand les dates sortent d'un exercice ;
	//   - « Du » et « Au » dans un groupe « Période » (`<fieldset>`) : quitter le groupe (`focusout` hors du
	//     groupe) ou Entrée dans une date confirme la période (`onPeriodCommit`) ;
	//   - « Exercice entier » quand la période n'est qu'une partie de l'exercice ;
	//   - rien n'est désactivé pendant un chargement (R-18) : la dernière demande l'emporte.
	// Story 55-2 (AC-11) — le Grand livre s'y branche aussi (`unSeulExercice={false}`) : il franchit la borne,
	// donc ni message « un seul exercice », et, sans exercice, des dates libres (non désactivées).
	// Le composant ne tient aucun état : l'état du menu (`menu`) et les dates viennent de la page, qui les
	// déduit (`etatDuMenu`) et décide du chargement.

	import type { Snippet } from 'svelte';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { libellesExercices, type EtatMenu } from './fiscal-year-menu';
	import { LINK_CLASS } from '$lib/shared/navigation/link-class';
	import type { FiscalYearResponse } from '$lib/features/fiscal-years/fiscal-years.types';

	interface Props {
		fiscalYears: FiscalYearResponse[];
		/** L'état du menu, déduit des dates par la page. */
		menu: EtatMenu;
		periodStart: string;
		periodEnd: string;
		/** Faux en mode « Cumulé » des rapports par compte analytique : menu et dates masqués. */
		showPeriod?: boolean;
		/** L'onglet ne porte que sur un exercice : « Plusieurs » ou « Hors exercice » disent pourquoi rien ne se charge. */
		unSeulExercice?: boolean;
		/** Story 55-2 — préfixe des `data-testid` des dates (`<préfixe>-from`, `<préfixe>-to`) ; le Grand livre garde `ledger`. */
		datesTestIdPrefix?: string;
		/** La période saisie est invalide (début après fin) : message sous les dates. */
		periodInvalid?: boolean;
		/** Choix d'un exercice dans le menu. */
		onFiscalYearSelect: (id: number) => void;
		/** Saisie d'une date (chaque frappe) : la page met ses champs et le menu à jour. */
		onDatesInput: (from: string, to: string) => void;
		/** Le focus quitte le groupe des dates (`next` : l'élément qui le reçoit, ou `null`), ou Entrée (`next` absent). */
		onPeriodCommit: (via: 'leave' | 'enter', next?: EventTarget | null) => void;
		/** « Exercice entier » : remettre les bornes de l'exercice du menu. */
		onWholeFiscalYear: () => void;
		/** Contrôles propres à l'onglet, avant le menu (compte analytique, mode). */
		controls?: Snippet;
		/** Actions propres à l'onglet (Générer, exports) ; à défaut, les exports PDF et CSV ci-dessous. */
		actions?: Snippet;
		onExportPdf?: () => void;
		onExportCsv?: () => void;
		canExport?: boolean;
		exporting?: boolean;
	}

	let {
		fiscalYears,
		menu,
		periodStart,
		periodEnd,
		showPeriod = true,
		unSeulExercice = true,
		datesTestIdPrefix = 'report',
		periodInvalid = false,
		onFiscalYearSelect,
		onDatesInput,
		onPeriodCommit,
		onWholeFiscalYear,
		controls,
		actions,
		onExportPdf,
		onExportCsv,
		canExport = false,
		exporting = false,
	}: Props = $props();

	const noFiscalYears = $derived(fiscalYears.length === 0);
	const libelles = $derived(libellesExercices(fiscalYears));
	const valeurMenu = $derived(menu.kind === 'fy' ? String(menu.id) : `__${menu.kind}`);
	const needsOneFy = $derived(!noFiscalYears && unSeulExercice && menu.kind !== 'fy');
	// Story 55-2 (AC-11) — sans exercice, un rapport qui franchit la borne (le Grand livre) garde ses dates.
	const datesDisabled = $derived(noFiscalYears && unSeulExercice);

	let groupe = $state<HTMLFieldSetElement | null>(null);

	function surChoix(e: Event): void {
		const v = (e.currentTarget as HTMLSelectElement).value;
		if (/^[0-9]+$/.test(v)) onFiscalYearSelect(Number(v));
	}

	function surSaisie(champ: 'from' | 'to', e: Event): void {
		const v = (e.currentTarget as HTMLInputElement).value;
		if (champ === 'from') onDatesInput(v, periodEnd);
		else onDatesInput(periodStart, v);
	}

	function surSortie(e: FocusEvent): void {
		const next = e.relatedTarget;
		// « Du » → « Au » (et retour), ou vers « Exercice entier » : on reste dans le groupe.
		if (next instanceof Node && groupe?.contains(next)) return;
		onPeriodCommit('leave', next);
	}

	function surTouche(e: KeyboardEvent): void {
		if (e.key === 'Enter') {
			e.preventDefault();
			onPeriodCommit('enter');
		}
	}
</script>

<div class="space-y-4 rounded border bg-white p-4 shadow-sm">
	<div class="grid gap-4 sm:grid-cols-3">
		{#if controls}
			{@render controls()}
		{/if}
		{#if showPeriod}
			<div class="block">
				<label for="report-fiscal-year" class="text-sm font-medium text-gray-700"
					>{i18nMsg('reports-filter-fiscal-year', 'Exercice')}</label
				>
				<select
					id="report-fiscal-year"
					class="mt-1 block w-full rounded border-gray-300 font-semibold shadow-sm"
					value={valeurMenu}
					onchange={surChoix}
					disabled={noFiscalYears}
					data-testid="report-fiscal-year"
				>
					{#each fiscalYears as fy (fy.id)}
						<option class="font-normal" value={String(fy.id)} selected={valeurMenu === String(fy.id)}
							>{libelles.get(fy.id) ?? fy.name}</option
						>
					{/each}
					{#if menu.kind === 'multiple' && !noFiscalYears}
						<option class="font-normal" value="__multiple" disabled selected
							>{i18nMsg('reports-nav-fy-multiple', 'Plusieurs')}</option
						>
					{:else if menu.kind === 'outside' && !noFiscalYears}
						<option class="font-normal" value="__outside" disabled selected
							>{i18nMsg('reports-nav-fy-outside', 'Hors exercice')}</option
						>
					{/if}
				</select>
			</div>

			<fieldset
				class="m-0 min-w-0 border-0 p-0 sm:col-span-2"
				bind:this={groupe}
				onfocusout={surSortie}
				data-testid="report-period"
			>
				<legend class="sr-only">{i18nMsg('reports-filter-period', 'Période')}</legend>
				<div class="flex flex-wrap items-end gap-4">
					<label class="block">
						<span class="text-sm font-medium text-gray-700">{i18nMsg('reports-ledger-from', 'Du')}</span>
						<input
							type="date"
							class="mt-1 block rounded border-gray-300 shadow-sm"
							value={periodStart}
							oninput={(e) => surSaisie('from', e)}
							onkeydown={surTouche}
							disabled={datesDisabled}
							data-testid="{datesTestIdPrefix}-from"
						/>
					</label>
					<label class="block">
						<span class="text-sm font-medium text-gray-700">{i18nMsg('reports-ledger-to', 'Au')}</span>
						<input
							type="date"
							class="mt-1 block rounded border-gray-300 shadow-sm"
							value={periodEnd}
							oninput={(e) => surSaisie('to', e)}
							onkeydown={surTouche}
							disabled={datesDisabled}
							data-testid="{datesTestIdPrefix}-to"
						/>
					</label>
					{#if menu.kind === 'fy' && !menu.whole}
						<button
							type="button"
							class="pb-2 text-sm {LINK_CLASS}"
							onclick={onWholeFiscalYear}
							data-testid="report-fy-whole"
						>
							{i18nMsg('reports-nav-fy-whole', 'Exercice entier')}
						</button>
					{/if}
				</div>
				{#if periodInvalid}
					<p class="mt-2 text-sm text-amber-900" role="status" data-testid="report-period-invalid">
						{i18nMsg(
							'reports-nav-period-invalid',
							'La date de début doit précéder la date de fin, ou lui être égale.',
						)}
					</p>
				{/if}
			</fieldset>
		{/if}
	</div>

	{#if noFiscalYears && unSeulExercice}
		<p class="rounded bg-amber-50 p-3 text-sm text-amber-900" role="alert">
			{i18nMsg(
				'reports-error-no-fiscal-year-available',
				'Aucun exercice comptable disponible. Créez un exercice avant de générer des rapports.',
			)}
		</p>
	{:else if needsOneFy && showPeriod}
		<p class="rounded bg-amber-50 p-3 text-sm text-amber-900" role="status" data-testid="report-needs-one-fy">
			{i18nMsg(
				'reports-nav-needs-one-fy',
				'Ce rapport porte sur un seul exercice : choisissez deux dates du même exercice.',
			)}
		</p>
	{/if}

	<div class="flex flex-wrap items-center gap-2">
		{#if actions}
			{@render actions()}
		{:else}
			{#if onExportPdf}
				<button
					type="button"
					class="rounded border border-gray-300 bg-white px-4 py-2 text-sm font-medium text-gray-700 shadow-sm hover:bg-gray-50 disabled:cursor-not-allowed disabled:bg-gray-100 disabled:text-gray-400"
					disabled={!canExport || exporting}
					onclick={onExportPdf}
				>
					{i18nMsg('reports-export-pdf-button', 'Export PDF')}
				</button>
			{/if}
			{#if onExportCsv}
				<button
					type="button"
					class="rounded border border-gray-300 bg-white px-4 py-2 text-sm font-medium text-gray-700 shadow-sm hover:bg-gray-50 disabled:cursor-not-allowed disabled:bg-gray-100 disabled:text-gray-400"
					disabled={!canExport || exporting}
					onclick={onExportCsv}
				>
					{i18nMsg('reports-export-csv-button', 'Export CSV')}
				</button>
			{/if}
		{/if}

		{#if exporting}
			<span class="text-sm italic text-gray-500" role="status">
				{i18nMsg('reports-export-loading', 'Génération du fichier…')}
			</span>
		{/if}
	</div>
</div>
