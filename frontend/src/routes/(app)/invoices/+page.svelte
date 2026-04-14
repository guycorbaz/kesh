<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import * as Dialog from '$lib/components/ui/dialog';
	import { Plus, Pencil, Trash2, Eye } from '@lucide/svelte';
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { isApiError } from '$lib/shared/utils/api-client';
	import { notifyError, notifySuccess } from '$lib/shared/utils/notify';

	import { listInvoices, deleteInvoice } from '$lib/features/invoices/invoices.api';
	import type {
		InvoiceListItemResponse,
		InvoiceSortBy,
		InvoiceStatus,
		SortDirection,
	} from '$lib/features/invoices/invoices.types';
	import { formatInvoiceTotal } from '$lib/features/invoices/invoice-helpers';
	import ContactPicker from '$lib/components/invoices/ContactPicker.svelte';
	import type { ContactResponse } from '$lib/features/contacts/contacts.types';
	import { getContact } from '$lib/features/contacts/contacts.api';

	let items = $state<InvoiceListItemResponse[]>([]);
	let total = $state(0);
	let loading = $state(false);

	let search = $state('');
	let effectiveSearch = $state('');
	let statusFilter = $state<InvoiceStatus | ''>('');
	let contactFilter = $state<ContactResponse | null>(null);
	let dateFromFilter = $state('');
	let dateToFilter = $state('');
	let sortBy = $state<InvoiceSortBy>('Date');
	let sortDirection = $state<SortDirection>('Desc');
	let limit = $state(20);
	let offset = $state(0);

	let debounceHandle: ReturnType<typeof setTimeout> | null = null;
	let mounted = $state(false);
	let loadSeq = 0;

	let deleteOpen = $state(false);
	let deleteTarget = $state<InvoiceListItemResponse | null>(null);
	let deleteSubmitting = $state(false);
	let deleteError = $state('');

	const VALID_STATUS: InvoiceStatus[] = ['draft', 'validated', 'cancelled'];
	const VALID_SORT_BY: InvoiceSortBy[] = ['Date', 'TotalAmount', 'ContactName', 'CreatedAt'];
	const VALID_SORT_DIR: SortDirection[] = ['Asc', 'Desc'];

	async function initFromUrl() {
		const params = page.url.searchParams;
		search = params.get('search') ?? '';
		effectiveSearch = search;
		const rawStatus = params.get('status') ?? '';
		statusFilter = (VALID_STATUS as string[]).includes(rawStatus)
			? (rawStatus as InvoiceStatus)
			: '';
		dateFromFilter = params.get('dateFrom') ?? '';
		dateToFilter = params.get('dateTo') ?? '';
		const rawContactId = parseInt(params.get('contactId') ?? '', 10);
		// Await avant `mounted = true` pour éviter qu'un premier `load()` parte
		// sans le filtre contact — sinon l'utilisateur voit brièvement une liste
		// non filtrée alors que l'URL réclame le filtre.
		if (Number.isFinite(rawContactId) && rawContactId > 0) {
			try {
				contactFilter = await getContact(rawContactId);
			} catch {
				contactFilter = null;
			}
		}
		const rawSortBy = params.get('sortBy') ?? 'Date';
		sortBy = (VALID_SORT_BY as string[]).includes(rawSortBy)
			? (rawSortBy as InvoiceSortBy)
			: 'Date';
		const rawSortDir = params.get('sortDirection') ?? 'Desc';
		sortDirection = (VALID_SORT_DIR as string[]).includes(rawSortDir)
			? (rawSortDir as SortDirection)
			: 'Desc';
		limit = Math.min(100, Math.max(1, parseInt(params.get('limit') ?? '20', 10) || 20));
		offset = Math.max(0, parseInt(params.get('offset') ?? '0', 10) || 0);
		mounted = true;
	}

	onMount(() => {
		initFromUrl();
		return () => {
			if (debounceHandle) clearTimeout(debounceHandle);
		};
	});

	function syncUrl() {
		const p = new URLSearchParams();
		// Si la plage de dates est invalide, on vide explicitement les deux
		// paramètres pour ne pas bookmarker un état trompeur (l'URL précédente
		// valide devient sinon masquée par une saisie invalide).
		if (dateRangeError) {
			if (effectiveSearch) p.set('search', effectiveSearch);
			if (statusFilter) p.set('status', statusFilter);
			if (contactFilter) p.set('contactId', String(contactFilter.id));
			if (sortBy !== 'Date') p.set('sortBy', sortBy);
			if (sortDirection !== 'Desc') p.set('sortDirection', sortDirection);
			if (limit !== 20) p.set('limit', String(limit));
			if (offset !== 0) p.set('offset', String(offset));
			const s2 = p.toString();
			goto(`/invoices${s2 ? `?${s2}` : ''}`, {
				replaceState: true,
				keepFocus: true,
				noScroll: true,
			});
			return;
		}
		if (effectiveSearch) p.set('search', effectiveSearch);
		if (statusFilter) p.set('status', statusFilter);
		if (contactFilter) p.set('contactId', String(contactFilter.id));
		if (dateFromFilter) p.set('dateFrom', dateFromFilter);
		if (dateToFilter) p.set('dateTo', dateToFilter);
		if (sortBy !== 'Date') p.set('sortBy', sortBy);
		if (sortDirection !== 'Desc') p.set('sortDirection', sortDirection);
		if (limit !== 20) p.set('limit', String(limit));
		if (offset !== 0) p.set('offset', String(offset));
		const s = p.toString();
		goto(`/invoices${s ? `?${s}` : ''}`, { replaceState: true, keepFocus: true, noScroll: true });
	}

	let dateRangeError = $derived(
		dateFromFilter && dateToFilter && dateFromFilter > dateToFilter
			? 'La date « Depuis » doit être antérieure ou égale à « Jusqu\'à »'
			: '',
	);

	async function load() {
		// Guard client : évite l'aller-retour 400 et l'affichage d'une liste vide trompeuse.
		if (dateRangeError) {
			items = [];
			total = 0;
			loading = false;
			return;
		}
		const seq = ++loadSeq;
		loading = true;
		try {
			const res = await listInvoices({
				search: effectiveSearch || undefined,
				status: statusFilter || undefined,
				contactId: contactFilter?.id,
				dateFrom: dateFromFilter || undefined,
				dateTo: dateToFilter || undefined,
				sortBy,
				sortDirection,
				limit,
				offset,
			});
			if (seq !== loadSeq) return;
			items = res.items;
			total = res.total;
		} catch (err) {
			if (seq !== loadSeq) return;
			if (isApiError(err)) notifyError(err.message);
			items = [];
			total = 0;
		} finally {
			if (seq === loadSeq) loading = false;
		}
	}

	$effect(() => {
		if (!mounted) return;
		effectiveSearch;
		statusFilter;
		contactFilter;
		dateFromFilter;
		dateToFilter;
		sortBy;
		sortDirection;
		limit;
		offset;
		syncUrl();
		load();
	});

	function onSearchInput(e: Event) {
		search = (e.target as HTMLInputElement).value;
		if (debounceHandle) clearTimeout(debounceHandle);
		debounceHandle = setTimeout(() => {
			effectiveSearch = search.trim();
			offset = 0;
		}, 300);
	}

	function toggleSort(col: InvoiceSortBy) {
		if (sortBy === col) {
			sortDirection = sortDirection === 'Asc' ? 'Desc' : 'Asc';
		} else {
			sortBy = col;
			sortDirection = col === 'Date' ? 'Desc' : 'Asc';
		}
		offset = 0;
	}

	function openDelete(item: InvoiceListItemResponse) {
		deleteTarget = item;
		deleteOpen = true;
	}

	async function confirmDelete() {
		if (!deleteTarget) return;
		deleteSubmitting = true;
		deleteError = '';
		try {
			await deleteInvoice(deleteTarget.id);
			notifySuccess('Facture supprimée');
			deleteOpen = false;
			deleteTarget = null;
			await load();
		} catch (err) {
			if (isApiError(err)) {
				deleteError = err.message;
				notifyError(err.message);
			} else {
				deleteError = 'Erreur lors de la suppression';
				notifyError(deleteError);
			}
		} finally {
			deleteSubmitting = false;
		}
	}

	function statusLabel(s: InvoiceStatus): string {
		switch (s) {
			case 'draft':
				return 'Brouillon';
			case 'validated':
				return 'Validée';
			case 'cancelled':
				return 'Annulée';
		}
	}
</script>

<svelte:head>
	<title>Factures — Kesh</title>
</svelte:head>

<div class="mb-6 flex items-center justify-between">
	<h1 class="text-2xl font-semibold text-text">Factures</h1>
	<Button onclick={() => goto('/invoices/new')}>
		<Plus class="h-4 w-4" aria-hidden="true" />
		Nouvelle facture
	</Button>
</div>

<div class="mb-4 flex flex-wrap items-end gap-3">
	<div class="flex-1 min-w-48 max-w-xs">
		<label class="mb-1 block text-xs text-text-muted" for="invoice-search">Recherche</label>
		<Input
			id="invoice-search"
			type="search"
			placeholder="Rechercher…"
			value={search}
			oninput={onSearchInput}
		/>
	</div>
	<div>
		<label class="mb-1 block text-xs text-text-muted" for="invoice-status-filter">Statut</label>
		<select
			id="invoice-status-filter"
			bind:value={statusFilter}
			class="h-9 rounded-md border border-border bg-background px-2 text-sm"
		>
			<option value="">Tous les statuts</option>
			<option value="draft">Brouillon</option>
			<option value="validated">Validée</option>
			<option value="cancelled">Annulée</option>
		</select>
	</div>
	<div class="min-w-56">
		<label class="mb-1 block text-xs text-text-muted" for="invoice-contact-filter">Contact</label>
		<ContactPicker
			selected={contactFilter}
			onSelect={(c) => {
				contactFilter = c;
				offset = 0;
			}}
			placeholder="Tous les contacts"
		/>
	</div>
	<div>
		<label class="mb-1 block text-xs text-text-muted" for="invoice-date-from">Depuis</label>
		<Input
			id="invoice-date-from"
			type="date"
			value={dateFromFilter}
			oninput={(e: Event) => {
				dateFromFilter = (e.target as HTMLInputElement).value;
				offset = 0;
			}}
		/>
	</div>
	<div>
		<label class="mb-1 block text-xs text-text-muted" for="invoice-date-to">Jusqu'à</label>
		<Input
			id="invoice-date-to"
			type="date"
			value={dateToFilter}
			oninput={(e: Event) => {
				dateToFilter = (e.target as HTMLInputElement).value;
				offset = 0;
			}}
		/>
	</div>
	{#if contactFilter || dateFromFilter || dateToFilter}
		<Button
			variant="outline"
			size="sm"
			onclick={() => {
				contactFilter = null;
				dateFromFilter = '';
				dateToFilter = '';
				offset = 0;
			}}
		>
			Réinitialiser
		</Button>
	{/if}
</div>
{#if dateRangeError}
	<div class="mb-4 rounded-md border border-destructive bg-destructive/10 px-3 py-2 text-sm text-destructive">
		{dateRangeError}
	</div>
{/if}

{#if loading}
	<p class="text-sm text-text-muted">Chargement…</p>
{:else if items.length === 0}
	<p class="text-sm text-text-muted">Aucune facture.</p>
{:else}
	<table class="w-full border-collapse text-sm">
		<thead>
			<tr class="border-b border-border text-left">
				<th class="cursor-pointer py-2 pr-2" onclick={() => toggleSort('Date')}>Date</th>
				<th class="cursor-pointer py-2 pr-2" onclick={() => toggleSort('ContactName')}>
					Contact
				</th>
				<th class="py-2 pr-2">N°</th>
				<th class="py-2 pr-2">Statut</th>
				<th class="cursor-pointer py-2 pr-2 text-right" onclick={() => toggleSort('TotalAmount')}>
					Total
				</th>
				<th class="py-2 pr-2 text-right">Actions</th>
			</tr>
		</thead>
		<tbody>
			{#each items as inv (inv.id)}
				<tr class="border-b border-border">
					<td class="py-2 pr-2">{inv.date}</td>
					<td class="py-2 pr-2">{inv.contactName}</td>
					<td class="py-2 pr-2">{inv.invoiceNumber ?? '—'}</td>
					<td class="py-2 pr-2">{statusLabel(inv.status)}</td>
					<td class="py-2 pr-2 text-right font-mono">
						{formatInvoiceTotal(inv.totalAmount)}
					</td>
					<td class="flex justify-end gap-1 py-2 pr-2">
						<Button variant="ghost" size="sm" onclick={() => goto(`/invoices/${inv.id}`)}>
							<Eye class="h-4 w-4" aria-hidden="true" />
						</Button>
						{#if inv.status === 'draft'}
							<Button variant="ghost" size="sm" onclick={() => goto(`/invoices/${inv.id}/edit`)}>
								<Pencil class="h-4 w-4" aria-hidden="true" />
							</Button>
							<Button variant="ghost" size="sm" onclick={() => openDelete(inv)}>
								<Trash2 class="h-4 w-4" aria-hidden="true" />
							</Button>
						{/if}
					</td>
				</tr>
			{/each}
		</tbody>
	</table>
	<div class="mt-4 flex items-center justify-between text-sm text-text-muted">
		<span>{total} facture{total > 1 ? 's' : ''}</span>
		<div class="flex gap-2">
			<Button
				variant="outline"
				size="sm"
				disabled={offset === 0}
				onclick={() => (offset = Math.max(0, offset - limit))}
			>
				Précédent
			</Button>
			<Button
				variant="outline"
				size="sm"
				disabled={offset + limit >= total}
				onclick={() => (offset = offset + limit)}
			>
				Suivant
			</Button>
		</div>
	</div>
{/if}

<Dialog.Root
	open={deleteOpen}
	onOpenChange={(o) => {
		deleteOpen = o;
		if (!o) {
			// Reset les états locaux à la fermeture (Annuler, Escape, overlay).
			// Sans ça, un deleteTarget stale pourrait réapparaître à l'ouverture
			// suivante avant le prochain appel à openDelete.
			deleteError = '';
			if (!deleteSubmitting) deleteTarget = null;
		}
	}}
>
	<Dialog.Content>
		<Dialog.Header>
			<Dialog.Title>Supprimer la facture</Dialog.Title>
		</Dialog.Header>
		<p class="text-sm">
			Confirmer la suppression de la facture du {deleteTarget?.date} pour {deleteTarget?.contactName} ?
		</p>
		{#if deleteError}
			<div class="rounded-md border border-destructive bg-destructive/10 px-3 py-2 text-sm text-destructive">
				{deleteError}
			</div>
		{/if}
		<Dialog.Footer>
			<Button variant="outline" onclick={() => (deleteOpen = false)}>Annuler</Button>
			<Button variant="destructive" onclick={confirmDelete} disabled={deleteSubmitting}>
				Supprimer
			</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>
