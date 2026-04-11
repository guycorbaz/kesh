<script lang="ts">
	import { onMount } from 'svelte';
	import { untrack } from 'svelte';
	import { page } from '$app/state';
	import { goto } from '$app/navigation';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import * as Select from '$lib/components/ui/select';
	import { Pencil, Plus, Trash2 } from '@lucide/svelte';
	import { toast } from 'svelte-sonner';
	import Big from 'big.js';
	import { i18nMsg } from '$lib/features/onboarding/onboarding.svelte';
	import { isApiError } from '$lib/shared/utils/api-client';
	import { fetchAccounts } from '$lib/features/accounts/accounts.api';
	import type { AccountResponse } from '$lib/features/accounts/accounts.types';
	import {
		deleteJournalEntry,
		fetchJournalEntries
	} from '$lib/features/journal-entries/journal-entries.api';
	import type {
		Journal,
		JournalEntryListQuery,
		JournalEntryResponse,
		SortBy,
		SortDirection
	} from '$lib/features/journal-entries/journal-entries.types';
	import JournalEntryForm from '$lib/features/journal-entries/JournalEntryForm.svelte';
	import { formatSwissAmount } from '$lib/features/journal-entries/balance';
	import {
		parseQueryFromUrl,
		serializeQuery
	} from '$lib/features/journal-entries/query-helpers';
	import { debounce } from '$lib/features/journal-entries/debounce';

	type Mode = 'list' | 'create' | 'edit';

	let mode = $state<Mode>('list');
	let entries = $state<JournalEntryResponse[]>([]);
	let accounts = $state<AccountResponse[]>([]);
	let accountsLoadError = $state(false);
	let loading = $state(false);

	// Story 3.4 — pagination + filtres.
	let total = $state(0);
	let offset = $state(0);
	let limit = $state(50);

	// État des filtres (initialisé depuis l'URL au mount).
	let description = $state('');
	let amountMin = $state('');
	let amountMax = $state('');
	let dateFrom = $state('');
	let dateTo = $state('');
	let journalFilter = $state<Journal | ''>('');
	let sortBy = $state<SortBy>('EntryDate');
	let sortDir = $state<SortDirection>('Desc');

	// Édition (story 3.3)
	let editingEntry = $state<JournalEntryResponse | null>(null);

	// Suppression (story 3.3)
	let deleteTarget = $state<JournalEntryResponse | null>(null);
	let deleting = $state(false);

	const rowActionsDisabled = $derived(deleting || deleteTarget !== null);

	const JOURNALS: Journal[] = ['Achats', 'Ventes', 'Banque', 'Caisse', 'OD'];
	const PAGE_SIZES = [25, 50, 100];

	function buildQuery(): JournalEntryListQuery {
		return {
			description: description || undefined,
			amountMin: amountMin || undefined,
			amountMax: amountMax || undefined,
			dateFrom: dateFrom || undefined,
			dateTo: dateTo || undefined,
			journal: journalFilter || undefined,
			sortBy,
			sortDir,
			offset,
			limit
		};
	}

	async function loadFiltered() {
		loading = true;
		try {
			const query = buildQuery();
			const [entriesResult, accountsResult] = await Promise.allSettled([
				fetchJournalEntries(query),
				fetchAccounts(false)
			]);

			if (entriesResult.status === 'fulfilled') {
				entries = entriesResult.value.items;
				total = entriesResult.value.total;
				// P5 : ne PAS écraser offset/limit depuis la réponse serveur.
				// Le client reste source de vérité — si le serveur a clampé
				// silencieusement, on conserve la valeur saisie côté UI pour
				// éviter une divergence state/UI. Le serveur retourne ses
				// valeurs pour information mais on les ignore ici.
			} else {
				entries = [];
				total = 0;
			}

			if (accountsResult.status === 'fulfilled') {
				accounts = accountsResult.value;
				accountsLoadError = false;
			} else {
				accounts = [];
				accountsLoadError = true;
			}
		} finally {
			loading = false;
		}
	}

	// Debounced loader pour les inputs text (description, montants).
	const debouncedLoad = debounce(() => {
		offset = 0; // reset pagination sur changement de filtre
		void loadFiltered();
	}, 300);

	// Loader immédiat pour les filtres non-text (dates, journal, tri, pagination).
	function immediateLoad(resetOffset = true) {
		if (resetOffset) offset = 0;
		void loadFiltered();
	}

	onMount(() => {
		// Initialiser l'état depuis l'URL.
		const initial = parseQueryFromUrl(page.url.searchParams);
		if (initial.description) description = initial.description;
		if (initial.amountMin) amountMin = initial.amountMin;
		if (initial.amountMax) amountMax = initial.amountMax;
		if (initial.dateFrom) dateFrom = initial.dateFrom;
		if (initial.dateTo) dateTo = initial.dateTo;
		if (initial.journal) journalFilter = initial.journal;
		if (initial.sortBy) sortBy = initial.sortBy;
		if (initial.sortDir) sortDir = initial.sortDir;
		if (initial.offset !== undefined) offset = initial.offset;
		if (initial.limit !== undefined) limit = initial.limit;

		void loadFiltered();

		return () => {
			debouncedLoad.cancel();
		};
	});

	// Sync URL ↔ query state.
	// P1 : `new URL(page.url)` est une lecture réactive de `page.url`. Le `goto`
	// avec `replaceState` peut mettre à jour `page.url`, ce qui re-déclencherait
	// ce `$effect` en boucle. On enveloppe TOUTE la section qui touche à
	// `page.url` dans `untrack(...)` — seules les dépendances sur `query` et
	// ses sous-state (`description`, `offset`, etc.) doivent déclencher l'effet.
	$effect(() => {
		// Déclencheurs réactifs : buildQuery lit les state vars.
		const q = buildQuery();
		const params = serializeQuery(q);
		untrack(() => {
			const url = new URL(page.url);
			url.search = params.toString();
			goto(url, { replaceState: true, noScroll: true, keepFocus: true });
		});
	});

	// Reset des filtres.
	function resetFilters() {
		description = '';
		amountMin = '';
		amountMax = '';
		dateFrom = '';
		dateTo = '';
		journalFilter = '';
		sortBy = 'EntryDate';
		sortDir = 'Desc';
		offset = 0;
		void loadFiltered();
	}

	// Handlers filtres text (debounced).
	function onDescriptionInput() {
		debouncedLoad();
	}
	function onAmountMinInput() {
		debouncedLoad();
	}
	function onAmountMaxInput() {
		debouncedLoad();
	}

	// Handlers filtres immédiats.
	function onDateFromChange() {
		immediateLoad();
	}
	function onDateToChange() {
		immediateLoad();
	}
	function onJournalChange(value: string) {
		journalFilter = (value as Journal) || '';
		immediateLoad();
	}

	// Tri par header cliquable.
	function onSortClick(col: SortBy) {
		if (sortBy === col) {
			sortDir = sortDir === 'Desc' ? 'Asc' : 'Desc';
		} else {
			sortBy = col;
			sortDir = 'Desc';
		}
		immediateLoad();
	}

	// Pagination. P3 : guards `if (loading) return` pour éviter les fetchs
	// concurrents (race last-write-wins qui désynchroniserait UI/état).
	function onPrevPage() {
		if (loading) return;
		if (offset === 0) return;
		offset = Math.max(0, offset - limit);
		void loadFiltered();
	}
	function onNextPage() {
		if (loading) return;
		if (offset + limit >= total) return;
		offset = offset + limit;
		void loadFiltered();
	}
	function onPageSizeChange(value: string) {
		if (loading) return;
		const n = Number(value);
		if (Number.isFinite(n) && n > 0) {
			limit = n;
			offset = 0;
			void loadFiltered();
		}
	}

	// --- Édition/suppression (story 3.3, inchangé) ---
	function openCreate() {
		editingEntry = null;
		mode = 'create';
	}

	function openEdit(entry: JournalEntryResponse) {
		editingEntry = entry;
		mode = 'edit';
	}

	function handleSuccess() {
		mode = 'list';
		editingEntry = null;
		void loadFiltered();
	}

	function handleCancel() {
		mode = 'list';
		editingEntry = null;
	}

	function handleConflictReload() {
		mode = 'list';
		editingEntry = null;
		void loadFiltered();
		toast.info(
			i18nMsg(
				'journal-entry-conflict-reloaded',
				'Liste rechargée — cliquez à nouveau sur modifier pour reprendre'
			)
		);
	}

	function openDeleteConfirm(entry: JournalEntryResponse) {
		if (deleting || deleteTarget) return;
		deleteTarget = entry;
	}

	function cancelDelete() {
		if (deleting) return;
		deleteTarget = null;
	}

	async function confirmDelete() {
		if (!deleteTarget) return;
		deleting = true;
		const targetId = deleteTarget.id;
		try {
			await deleteJournalEntry(targetId);
			toast.success(i18nMsg('journal-entry-deleted', 'Écriture supprimée'));
			await loadFiltered();
		} catch (err) {
			if (isApiError(err)) {
				const code = err.code ?? '';
				switch (code) {
					case 'FISCAL_YEAR_CLOSED':
					case 'NOT_FOUND':
					case 'VALIDATION_ERROR':
						toast.error(err.message);
						break;
					default:
						toast.error(err.message || 'Erreur lors de la suppression');
				}
			} else {
				toast.error('Erreur lors de la suppression');
			}
		} finally {
			deleteTarget = null;
			deleting = false;
		}
	}

	function handleKeydown(e: KeyboardEvent) {
		if ((e.ctrlKey || e.metaKey) && e.key === 'n' && mode === 'list') {
			e.preventDefault();
			openCreate();
		}
		if (e.key === 'Escape' && deleteTarget && !deleting) {
			cancelDelete();
		}
	}

	// --- Rendu helpers ---
	function totalOf(entry: JournalEntryResponse): string {
		try {
			const total = entry.lines.reduce(
				(acc, l) => acc.plus(new Big(l.debit || '0')),
				new Big(0)
			);
			return formatSwissAmount(total);
		} catch {
			return '—';
		}
	}

	function journalLabel(j: string): string {
		return i18nMsg(`journal-${j.toLowerCase()}`, j);
	}

	function sortIndicator(col: SortBy): string {
		if (sortBy !== col) return '';
		return sortDir === 'Asc' ? '↑' : '↓';
	}

	// Pagination display : "X-Y sur N" (1-indexed côté affichage).
	const paginationRange = $derived.by(() => {
		if (total === 0) return '0';
		const from = offset + 1;
		const to = Math.min(offset + limit, total);
		return `${from}-${to}`;
	});

	const canPrev = $derived(offset > 0);
	const canNext = $derived(offset + limit < total);
</script>

<svelte:head>
	<title>{i18nMsg('journal-entries-title', 'Écritures comptables')} - Kesh</title>
</svelte:head>

<svelte:window onkeydown={handleKeydown} />

<div class="space-y-6">
	<div class="flex items-center justify-between">
		<h1 class="text-2xl font-semibold">
			{i18nMsg('journal-entries-title', 'Écritures comptables')}
		</h1>
		{#if mode === 'list'}
			<Button onclick={openCreate}>
				<Plus class="w-4 h-4 mr-1" />
				{i18nMsg('journal-entries-new', 'Nouvelle écriture')}
			</Button>
		{/if}
	</div>

	{#if mode === 'create'}
		<JournalEntryForm
			{accounts}
			{accountsLoadError}
			onSuccess={handleSuccess}
			onCancel={handleCancel}
		/>
	{:else if mode === 'edit' && editingEntry}
		<JournalEntryForm
			{accounts}
			{accountsLoadError}
			initialEntry={editingEntry}
			onSuccess={handleSuccess}
			onCancel={handleCancel}
			onConflictReload={handleConflictReload}
		/>
	{:else}
		<!-- Barre de filtres (Story 3.4) -->
		<div class="grid grid-cols-1 md:grid-cols-3 lg:grid-cols-6 gap-2 p-4 bg-muted/30 rounded-md">
			<div>
				<label for="filter-description" class="block text-xs font-medium mb-1">
					{i18nMsg('journal-entries-filter-description', 'Libellé')}
				</label>
				<Input
					id="filter-description"
					type="text"
					bind:value={description}
					oninput={onDescriptionInput}
					placeholder={i18nMsg('journal-entries-filter-description', 'Libellé')}
				/>
			</div>
			<div>
				<label for="filter-amount-min" class="block text-xs font-medium mb-1">
					{i18nMsg('journal-entries-filter-amount-min', 'Montant min')}
				</label>
				<Input
					id="filter-amount-min"
					type="text"
					inputmode="decimal"
					bind:value={amountMin}
					oninput={onAmountMinInput}
					placeholder="0.00"
				/>
			</div>
			<div>
				<label for="filter-amount-max" class="block text-xs font-medium mb-1">
					{i18nMsg('journal-entries-filter-amount-max', 'Montant max')}
				</label>
				<Input
					id="filter-amount-max"
					type="text"
					inputmode="decimal"
					bind:value={amountMax}
					oninput={onAmountMaxInput}
					placeholder="0.00"
				/>
			</div>
			<div>
				<label for="filter-date-from" class="block text-xs font-medium mb-1">
					{i18nMsg('journal-entries-filter-date-from', 'Date début')}
				</label>
				<Input
					id="filter-date-from"
					type="date"
					bind:value={dateFrom}
					onchange={onDateFromChange}
				/>
			</div>
			<div>
				<label for="filter-date-to" class="block text-xs font-medium mb-1">
					{i18nMsg('journal-entries-filter-date-to', 'Date fin')}
				</label>
				<Input
					id="filter-date-to"
					type="date"
					bind:value={dateTo}
					onchange={onDateToChange}
				/>
			</div>
			<div>
				<label for="filter-journal" class="block text-xs font-medium mb-1">
					{i18nMsg('journal-entries-filter-journal', 'Journal')}
				</label>
				<Select.Root
					type="single"
					value={journalFilter}
					onValueChange={onJournalChange}
				>
					<Select.Trigger id="filter-journal">
						{journalFilter
							? journalLabel(journalFilter)
							: i18nMsg('journal-entries-filter-journal-all', 'Tous')}
					</Select.Trigger>
					<Select.Content>
						<Select.Item value="">
							{i18nMsg('journal-entries-filter-journal-all', 'Tous')}
						</Select.Item>
						{#each JOURNALS as j (j)}
							<Select.Item value={j}>{journalLabel(j)}</Select.Item>
						{/each}
					</Select.Content>
				</Select.Root>
			</div>
		</div>

		<div class="flex justify-end">
			<Button type="button" variant="outline" size="sm" onclick={resetFilters}>
				{i18nMsg('journal-entries-filter-reset', 'Réinitialiser')}
			</Button>
		</div>

		{#if loading}
			<p class="text-sm text-muted-foreground">
				{i18nMsg('journal-entries-loading', 'Chargement…')}
			</p>
		{:else if entries.length === 0}
			<p class="text-sm text-muted-foreground">
				{i18nMsg('journal-entries-empty-list', "Aucune écriture saisie pour l'instant")}
			</p>
		{:else}
			<table class="w-full border-collapse">
				<thead>
					<tr class="border-b border-border">
						<th class="text-left py-2 text-sm font-medium w-16">
							<button
								type="button"
								class="hover:underline font-medium"
								onclick={() => onSortClick('EntryNumber')}
							>
								{i18nMsg('journal-entries-col-number', 'N°')}
								{sortIndicator('EntryNumber')}
							</button>
						</th>
						<th class="text-left py-2 text-sm font-medium w-28">
							<button
								type="button"
								class="hover:underline font-medium"
								onclick={() => onSortClick('EntryDate')}
							>
								{i18nMsg('journal-entries-col-date', 'Date')}
								{sortIndicator('EntryDate')}
							</button>
						</th>
						<th class="text-left py-2 text-sm font-medium w-28">
							<button
								type="button"
								class="hover:underline font-medium"
								onclick={() => onSortClick('Journal')}
							>
								{i18nMsg('journal-entries-col-journal', 'Journal')}
								{sortIndicator('Journal')}
							</button>
						</th>
						<th class="text-left py-2 text-sm font-medium">
							<button
								type="button"
								class="hover:underline font-medium"
								onclick={() => onSortClick('Description')}
							>
								{i18nMsg('journal-entries-col-description', 'Libellé')}
								{sortIndicator('Description')}
							</button>
						</th>
						<th class="text-right py-2 text-sm font-medium w-32">
							{i18nMsg('journal-entries-col-total', 'Total')}
						</th>
						<th class="py-2 w-20"></th>
					</tr>
				</thead>
				<tbody>
					{#each entries as entry (entry.id)}
						<tr class="border-b border-border/50">
							<td class="py-2 tabular-nums">{entry.entryNumber}</td>
							<td class="py-2">{entry.entryDate}</td>
							<td class="py-2">{journalLabel(entry.journal)}</td>
							<td class="py-2">{entry.description}</td>
							<td class="py-2 text-right tabular-nums">{totalOf(entry)}</td>
							<td class="py-2 text-right">
								<button
									type="button"
									class="text-text-muted hover:text-foreground p-1 disabled:opacity-40 disabled:cursor-not-allowed"
									onclick={() => openEdit(entry)}
									disabled={rowActionsDisabled}
									aria-label={i18nMsg('journal-entry-edit', 'Modifier')}
								>
									<Pencil class="w-4 h-4" />
								</button>
								<button
									type="button"
									class="text-text-muted hover:text-destructive p-1 disabled:opacity-40 disabled:cursor-not-allowed"
									onclick={() => openDeleteConfirm(entry)}
									disabled={rowActionsDisabled}
									aria-label={i18nMsg('journal-entry-delete', 'Supprimer')}
								>
									<Trash2 class="w-4 h-4" />
								</button>
							</td>
						</tr>
					{/each}
				</tbody>
			</table>

			<!-- Pied de tableau : pagination -->
			<div class="flex items-center justify-between text-sm">
				<div class="text-text-muted">
					{paginationRange} {i18nMsg('journal-entries-pagination-on', 'sur')} {total}
				</div>
				<div class="flex items-center gap-2">
					<Button
						type="button"
						variant="outline"
						size="sm"
						onclick={onPrevPage}
						disabled={!canPrev}
					>
						{i18nMsg('journal-entries-pagination-prev', 'Précédent')}
					</Button>
					<Button
						type="button"
						variant="outline"
						size="sm"
						onclick={onNextPage}
						disabled={!canNext}
					>
						{i18nMsg('journal-entries-pagination-next', 'Suivant')}
					</Button>
					<Select.Root
						type="single"
						value={String(limit)}
						onValueChange={onPageSizeChange}
					>
						<Select.Trigger>
							{limit}
						</Select.Trigger>
						<Select.Content>
							{#each PAGE_SIZES as size (size)}
								<Select.Item value={String(size)}>{size}</Select.Item>
							{/each}
						</Select.Content>
					</Select.Root>
				</div>
			</div>
		{/if}
	{/if}
</div>

<!-- Dialog de confirmation de suppression (story 3.3) -->
{#if deleteTarget}
	{@const deleteTitle = i18nMsg(
		'journal-entry-delete-confirm-title',
		"Supprimer l'écriture N°{ $number } ?"
	).replace(/\{\s*\$number\s*\}/g, String(deleteTarget.entryNumber))}
	<div
		class="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
		role="dialog"
		aria-modal="true"
		aria-labelledby="delete-confirm-title"
		aria-describedby="delete-confirm-desc"
	>
		<div class="bg-card border border-border rounded-lg p-6 max-w-md mx-4 shadow-lg">
			<h2 id="delete-confirm-title" class="text-lg font-semibold mb-2">
				{deleteTitle}
			</h2>
			<p id="delete-confirm-desc" class="text-sm text-text-muted mb-4">
				{i18nMsg(
					'journal-entry-delete-confirm-message',
					"Cette action est irréversible. L'action sera enregistrée dans le journal d'audit."
				)}
			</p>
			<div class="flex justify-end gap-2">
				<!-- svelte-ignore a11y_autofocus -->
				<Button
					type="button"
					variant="outline"
					onclick={cancelDelete}
					disabled={deleting}
					autofocus
				>
					{i18nMsg('journal-entry-delete-confirm-cancel', 'Annuler')}
				</Button>
				<Button
					type="button"
					variant="destructive"
					onclick={confirmDelete}
					disabled={deleting}
				>
					{i18nMsg('journal-entry-delete-confirm-delete', 'Supprimer')}
				</Button>
			</div>
		</div>
	</div>
{/if}
