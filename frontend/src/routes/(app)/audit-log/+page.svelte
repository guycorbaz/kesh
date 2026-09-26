<script lang="ts">
	/**
	 * Le journal d'audit — Story 25-1c-b1 (#378).
	 *
	 * Consulter la piste de contrôle de la société : filtrer, déplier le
	 * détail d'une entrée, exporter. ⛔ L'écran ne traduit RIEN du vocabulaire :
	 * la route rend `actionLabel`, `entityTypeLabel` et le vocabulaire traduit
	 * (Story 25-1c-a). ⚠️ Les dates filtrent sur des jours **UTC** (contrat de la
	 * route) ; le tableau affiche l'heure **locale** — d'où les libellés « jour
	 * UTC » des filtres.
	 */
	import { onMount, untrack } from 'svelte';
	import { page } from '$app/state';
	import { goto } from '$app/navigation';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { i18nLocale, i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { isApiError } from '$lib/shared/utils/api-client';
	import { notifyError } from '$lib/shared/utils/notify';
	import {
		exportAuditLogCsv,
		getAuditLogVocabulary,
		listAuditLog,
	} from '$lib/features/audit-log/audit-log.api';
	import type {
		AuditLogEntry,
		AuditLogQuery,
		AuditLogVocabulary,
	} from '$lib/features/audit-log/audit-log.types';
	import {
		DEFAULT_LIMIT,
		parseQueryFromUrl,
		serializeQuery,
	} from '$lib/features/audit-log/query-helpers';
	import { toSelectOptions } from '$lib/features/audit-log/vocabulary-options';

	let vocabulary = $state<AuditLogVocabulary | null>(null);
	let vocabularyError = $state('');
	let entries = $state<AuditLogEntry[]>([]);
	let total = $state(0);
	let offset = $state(0);
	let limit = $state(DEFAULT_LIMIT);
	let loading = $state(true);
	let listError = $state('');
	let exporting = $state(false);
	let exportError = $state('');
	let expanded = $state<Set<number>>(new Set());

	// Les filtres affichés.
	let dateFrom = $state('');
	let dateTo = $state('');
	let entityType = $state('');
	let entityIdValue = $state('');
	let action = $state('');

	let entityTypeOptions = $derived(
		toSelectOptions(vocabulary?.entityTypes ?? [], i18nLocale(), entityType || undefined),
	);
	let actionOptions = $derived(
		toSelectOptions(vocabulary?.actions ?? [], i18nLocale(), action || undefined),
	);
	let dateFormat = $derived(
		new Intl.DateTimeFormat(i18nLocale(), { dateStyle: 'medium', timeStyle: 'short' }),
	);

	/** La requête courante : les filtres affichés et la pagination. */
	function buildQuery(): AuditLogQuery {
		const q: AuditLogQuery = { offset, limit };
		if (dateFrom) q.dateFrom = dateFrom;
		if (dateTo) q.dateTo = dateTo;
		if (entityType) {
			q.entityType = entityType;
			const n = Number(entityIdValue);
			if (entityIdValue !== '' && Number.isInteger(n) && n > 0) q.entityId = n;
		}
		if (action) q.action = action;
		return q;
	}

	async function loadList() {
		loading = true;
		listError = '';
		try {
			const res = await listAuditLog(buildQuery());
			entries = res.items;
			total = res.total;
		} catch (err) {
			entries = [];
			total = 0;
			listError = isApiError(err)
				? err.message
				: i18nMsg('audit-log-error', "Le journal d'audit n'a pas pu être chargé.");
		} finally {
			loading = false;
		}
	}

	onMount(() => {
		const initial = parseQueryFromUrl(page.url.searchParams);
		dateFrom = initial.dateFrom ?? '';
		dateTo = initial.dateTo ?? '';
		entityType = initial.entityType ?? '';
		entityIdValue = initial.entityId !== undefined ? String(initial.entityId) : '';
		action = initial.action ?? '';
		offset = initial.offset ?? 0;
		limit = initial.limit ?? DEFAULT_LIMIT;

		// ⛔ Un échec du vocabulaire est l'état d'ERREUR de la page : des filtres
		// vides laisseraient croire qu'il n'y a rien à filtrer.
		getAuditLogVocabulary()
			.then((v) => (vocabulary = v))
			.catch((err) => {
				vocabularyError = isApiError(err)
					? err.message
					: i18nMsg('audit-log-error', "Le journal d'audit n'a pas pu être chargé.");
			});
		void loadList();
	});

	// L'URL suit les filtres et la pagination (patron `journal-entries`).
	$effect(() => {
		const params = serializeQuery(buildQuery());
		untrack(() => {
			const url = new URL(page.url);
			url.search = params.toString();
			goto(url, { replaceState: true, noScroll: true, keepFocus: true });
		});
	});

	/** Un filtre a changé : retour à la première page. */
	function applyFilters() {
		offset = 0;
		void loadList();
	}

	function onEntityTypeChange() {
		// ⛔ Revenir à « Tous » VIDE l'identifiant : désactiver le champ ne
		// suffit pas, sa valeur partirait à la route, qui répondrait 400.
		if (!entityType) entityIdValue = '';
		applyFilters();
	}

	function resetFilters() {
		dateFrom = '';
		dateTo = '';
		entityType = '';
		entityIdValue = '';
		action = '';
		applyFilters();
	}

	function onPrev() {
		if (loading || offset === 0) return;
		offset = Math.max(0, offset - limit);
		void loadList();
	}

	function onNext() {
		if (loading || offset + limit >= total) return;
		offset = offset + limit;
		void loadList();
	}

	function toggleDetails(id: number) {
		const next = new Set(expanded);
		if (next.has(id)) next.delete(id);
		else next.add(id);
		expanded = next;
	}

	async function exportCsv() {
		if (exporting) return;
		exporting = true;
		exportError = '';
		try {
			await exportAuditLogCsv(buildQuery());
		} catch (err) {
			// Le message du serveur est déjà traduit (`audit-log-export-error-too-large`).
			if (isApiError(err) && err.code === 'RESULT_TOO_LARGE') {
				exportError = err.message;
			} else {
				notifyError(
					isApiError(err) ? err.message : i18nMsg('common-error', 'Erreur inattendue'),
				);
			}
		} finally {
			exporting = false;
		}
	}
</script>

<svelte:head>
	<title>{i18nMsg('audit-log-title', "Journal d'audit")} — Kesh</title>
</svelte:head>

<h1 class="mb-1 text-2xl font-semibold">{i18nMsg('audit-log-title', "Journal d'audit")}</h1>
<p class="mb-6 text-sm text-text-muted">
	{i18nMsg(
		'audit-log-subtitle',
		'La trace de chaque opération qui modifie les livres de votre société : qui, quoi, quand.',
	)}
</p>

{#if vocabularyError}
	<p class="text-sm text-destructive" role="alert" data-testid="audit-log-error">
		{vocabularyError}
	</p>
{:else}
	<div class="mb-4 flex flex-wrap items-end gap-3">
		<div>
			<label class="mb-1 block text-xs text-text-muted" for="audit-log-filter-date-from">
				{i18nMsg('audit-log-filter-date-from', 'Du (jour UTC)')}
			</label>
			<Input
				id="audit-log-filter-date-from"
				data-testid="audit-log-filter-date-from"
				type="date"
				bind:value={dateFrom}
				onchange={applyFilters}
			/>
		</div>
		<div>
			<label class="mb-1 block text-xs text-text-muted" for="audit-log-filter-date-to">
				{i18nMsg('audit-log-filter-date-to', 'Au (jour UTC)')}
			</label>
			<Input
				id="audit-log-filter-date-to"
				data-testid="audit-log-filter-date-to"
				type="date"
				bind:value={dateTo}
				onchange={applyFilters}
			/>
		</div>
		<div>
			<label class="mb-1 block text-xs text-text-muted" for="audit-log-filter-entity-type">
				{i18nMsg('audit-log-filter-entity-type', "Type d'entité")}
			</label>
			<select
				id="audit-log-filter-entity-type"
				data-testid="audit-log-filter-entity-type"
				bind:value={entityType}
				onchange={onEntityTypeChange}
				class="h-9 rounded-md border border-border bg-background px-2 text-sm"
			>
				<option value="">{i18nMsg('audit-log-filter-entity-type-all', 'Tous')}</option>
				{#each entityTypeOptions as o (o.value)}
					<option value={o.value}>{o.label}</option>
				{/each}
			</select>
		</div>
		<div>
			<label class="mb-1 block text-xs text-text-muted" for="audit-log-filter-entity-id">
				{i18nMsg('audit-log-filter-entity-id', "N° d'entité")}
			</label>
			<Input
				id="audit-log-filter-entity-id"
				data-testid="audit-log-filter-entity-id"
				type="number"
				min="1"
				class="w-28"
				disabled={!entityType}
				bind:value={entityIdValue}
				onchange={applyFilters}
			/>
		</div>
		<div>
			<label class="mb-1 block text-xs text-text-muted" for="audit-log-filter-action">
				{i18nMsg('audit-log-filter-action', 'Action')}
			</label>
			<select
				id="audit-log-filter-action"
				data-testid="audit-log-filter-action"
				bind:value={action}
				onchange={applyFilters}
				class="h-9 rounded-md border border-border bg-background px-2 text-sm"
			>
				<option value="">{i18nMsg('audit-log-filter-action-all', 'Toutes')}</option>
				{#each actionOptions as o (o.value)}
					<option value={o.value}>{o.label}</option>
				{/each}
			</select>
		</div>
		<Button variant="outline" data-testid="audit-log-filter-reset" onclick={resetFilters}>
			{i18nMsg('audit-log-filter-reset', 'Réinitialiser')}
		</Button>
		<Button data-testid="audit-log-export" onclick={exportCsv} disabled={exporting}>
			{i18nMsg('audit-log-export', 'Exporter en CSV')}
		</Button>
	</div>

	{#if exportError}
		<p class="mb-4 text-sm text-destructive" role="alert" data-testid="audit-log-export-error">
			{exportError}
		</p>
	{/if}

	<!-- ⚠️ L'erreur de la LISTE remplace le tableau seul : les filtres et
	     « Réinitialiser » restent visibles, pour sortir d'une erreur de filtre. -->
	{#if loading}
		<p class="text-sm text-text-muted" data-testid="audit-log-loading">
			{i18nMsg('common-loading', 'Chargement…')}
		</p>
	{:else if listError}
		<p class="text-sm text-destructive" role="alert" data-testid="audit-log-error">{listError}</p>
	{:else if entries.length === 0}
		<p class="text-sm text-text-muted" data-testid="audit-log-empty">
			{i18nMsg('audit-log-empty', 'Aucune entrée ne correspond à ces filtres.')}
		</p>
	{:else}
		<table class="w-full border-collapse text-sm" data-testid="audit-log-table">
			<thead>
				<tr class="border-b text-left text-text-muted">
					<th class="py-2">{i18nMsg('audit-log-col-date', 'Date')}</th>
					<th class="py-2">{i18nMsg('audit-log-col-actor', 'Auteur')}</th>
					<th class="py-2">{i18nMsg('audit-log-col-action', 'Action')}</th>
					<th class="py-2">{i18nMsg('audit-log-col-entity-type', "Type d'entité")}</th>
					<th class="py-2 text-right">{i18nMsg('audit-log-col-entity-id', 'N°')}</th>
					<th class="py-2">{i18nMsg('audit-log-col-details', 'Détails')}</th>
				</tr>
			</thead>
			<tbody>
				{#each entries as e (e.id)}
					<tr
						class="border-b align-top"
						data-testid="audit-log-row"
						data-action={e.action}
						data-entity-type={e.entityType}
					>
						<td class="py-2 whitespace-nowrap">{dateFormat.format(new Date(e.createdAt))}</td>
						<td class="py-2">
							{e.actorLabel}
							{#if e.actorType === 'api_key'}
								<span class="text-xs text-text-muted">
									({i18nMsg('audit-log-api-key', 'clé API')})
								</span>
							{/if}
						</td>
						<td class="py-2" data-testid="audit-log-row-action">{e.actionLabel}</td>
						<td class="py-2">{e.entityTypeLabel}</td>
						<td class="py-2 text-right">{e.entityId === 0 ? '—' : e.entityId}</td>
						<td class="py-2">
							{#if e.details !== null && e.details !== undefined}
								<button
									class="text-xs text-primary underline"
									data-testid="audit-log-details-toggle"
									onclick={() => toggleDetails(e.id)}
								>
									{expanded.has(e.id)
										? i18nMsg('audit-log-details-hide', 'Masquer')
										: i18nMsg('audit-log-details-show', 'Afficher')}
								</button>
								{#if expanded.has(e.id)}
									<pre
										class="mt-1 max-w-xl overflow-x-auto rounded bg-muted p-2 text-xs"
										data-testid="audit-log-details">{JSON.stringify(e.details, null, 2)}</pre>
								{/if}
							{/if}
						</td>
					</tr>
				{/each}
			</tbody>
		</table>

		<div class="mt-4 flex items-center gap-3 text-sm">
			<Button
				variant="outline"
				data-testid="audit-log-prev"
				onclick={onPrev}
				disabled={offset === 0}
			>
				{i18nMsg('audit-log-prev', 'Précédent')}
			</Button>
			<span>
				{i18nMsg('audit-log-range', '{ $from }–{ $to } sur { $total }', {
					from: offset + 1,
					to: Math.min(offset + limit, total),
					total,
				})}
			</span>
			<Button
				variant="outline"
				data-testid="audit-log-next"
				onclick={onNext}
				disabled={offset + limit >= total}
			>
				{i18nMsg('audit-log-next', 'Suivant')}
			</Button>
		</div>
	{/if}
{/if}
