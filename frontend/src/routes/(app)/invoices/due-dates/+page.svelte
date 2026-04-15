<!--
  Story 5.4 — Page échéancier.
  Filtre défaut : paymentStatus=unpaid, sort=dueDate ASC.
  Surlignage lignes overdue (fond orange clair). Summary d'impayées en tête.
-->
<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Download } from '@lucide/svelte';
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';

	import { isApiError } from '$lib/shared/utils/api-client';
	import { authState } from '$lib/app/stores/auth.svelte';
	import { notifyError, notifySuccess } from '$lib/shared/utils/notify';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import {
		listDueDates,
		markInvoicePaid,
		exportDueDatesCsv,
	} from '$lib/features/invoices/invoices.api';
	import type {
		DueDateItem,
		DueDatesSummary,
		PaymentStatusFilter,
		InvoiceSortBy,
		SortDirection,
	} from '$lib/features/invoices/invoices.types';
	import { formatInvoiceTotal } from '$lib/features/invoices/invoice-helpers';
	import ContactPicker from '$lib/components/invoices/ContactPicker.svelte';
	import type { ContactResponse } from '$lib/features/contacts/contacts.types';
	import { getContact } from '$lib/features/contacts/contacts.api';
	import PaymentStatusBadge from '$lib/features/invoices/PaymentStatusBadge.svelte';
	import MarkPaidDialog from '$lib/features/invoices/MarkPaidDialog.svelte';

	const VALID_PAYMENT_STATUS: PaymentStatusFilter[] = ['all', 'unpaid', 'overdue', 'paid'];

	// D2 (review pass 1 G2 D) : map locale-stable des labels filter (le
	// fallback `i18nMsg(key, ps)` exposait raw `'unpaid'` aux UI FR/DE/IT
	// si la clé FTL manquait). Fallbacks FR explicites au lieu du mot-clé.
	const FILTER_FALLBACK_FR: Record<PaymentStatusFilter, string> = {
		all: 'Toutes',
		unpaid: 'Impayées',
		overdue: 'En retard',
		paid: 'Payées',
	};

	// B2 (review pass 1 G2 B) : export CSV gated Comptable+ — masquer le
	// bouton pour Consultation pour éviter un 403 au clic.
	const canExportCsv = $derived(
		authState.currentUser?.role === 'Admin' || authState.currentUser?.role === 'Comptable',
	);
	const VALID_SORT_BY: InvoiceSortBy[] = ['Date', 'DueDate', 'TotalAmount', 'ContactName'];
	const VALID_SORT_DIR: SortDirection[] = ['Asc', 'Desc'];

	let items = $state<DueDateItem[]>([]);
	let total = $state(0);
	let summary = $state<DueDatesSummary>({
		unpaidCount: 0,
		unpaidTotal: '0',
		overdueCount: 0,
		overdueTotal: '0',
	});
	let loading = $state(false);

	let search = $state('');
	let effectiveSearch = $state('');
	let paymentStatus = $state<PaymentStatusFilter>('unpaid');
	let contactFilter = $state<ContactResponse | null>(null);
	let dueBeforeFilter = $state('');
	let sortBy = $state<InvoiceSortBy>('DueDate');
	let sortDirection = $state<SortDirection>('Asc');
	let limit = $state(20);
	let offset = $state(0);

	let debounceHandle: ReturnType<typeof setTimeout> | null = null;
	let mounted = $state(false);
	let loadSeq = 0;

	let markOpen = $state(false);
	let markTarget = $state<DueDateItem | null>(null);
	let markSubmitting = $state(false);
	let markError = $state('');

	async function initFromUrl() {
		const params = page.url.searchParams;
		search = params.get('search') ?? '';
		effectiveSearch = search;
		const rawPs = params.get('paymentStatus') ?? 'unpaid';
		paymentStatus = (VALID_PAYMENT_STATUS as string[]).includes(rawPs)
			? (rawPs as PaymentStatusFilter)
			: 'unpaid';
		dueBeforeFilter = params.get('dueBefore') ?? '';
		const rawContactId = parseInt(params.get('contactId') ?? '', 10);
		if (Number.isFinite(rawContactId) && rawContactId > 0) {
			try {
				contactFilter = await getContact(rawContactId);
			} catch {
				contactFilter = null;
			}
		}
		const rawSortBy = params.get('sortBy') ?? 'DueDate';
		sortBy = (VALID_SORT_BY as string[]).includes(rawSortBy)
			? (rawSortBy as InvoiceSortBy)
			: 'DueDate';
		const rawSortDir = params.get('sortDirection') ?? 'Asc';
		sortDirection = (VALID_SORT_DIR as string[]).includes(rawSortDir)
			? (rawSortDir as SortDirection)
			: 'Asc';
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
		if (effectiveSearch) p.set('search', effectiveSearch);
		// D1 (review pass 1 G2 D) : URL share-friendly — toujours écrire
		// `paymentStatus` même si défaut, pour qu'un lien partagé montre
		// exactement ce que voit le destinataire (sans dépendre du défaut
		// backend qui pourrait évoluer).
		p.set('paymentStatus', paymentStatus);
		if (contactFilter) p.set('contactId', String(contactFilter.id));
		if (dueBeforeFilter) p.set('dueBefore', dueBeforeFilter);
		if (sortBy !== 'DueDate') p.set('sortBy', sortBy);
		if (sortDirection !== 'Asc') p.set('sortDirection', sortDirection);
		if (limit !== 20) p.set('limit', String(limit));
		if (offset !== 0) p.set('offset', String(offset));
		const s = p.toString();
		goto(`/invoices/due-dates${s ? `?${s}` : ''}`, {
			replaceState: true,
			keepFocus: true,
			noScroll: true,
		});
	}

	async function load() {
		const seq = ++loadSeq;
		loading = true;
		try {
			const res = await listDueDates({
				search: effectiveSearch || undefined,
				contactId: contactFilter?.id,
				dueBefore: dueBeforeFilter || undefined,
				paymentStatus,
				sortBy,
				sortDirection,
				limit,
				offset,
			});
			if (seq !== loadSeq) return;
			items = res.items;
			total = res.total;
			summary = res.summary;
		} catch (err) {
			if (seq !== loadSeq) return;
			if (isApiError(err)) notifyError(err.message);
			items = [];
			total = 0;
			// Reset summary à zéro sur erreur — sinon stale KPIs visibles
			// à côté d'une table vide (utilisateur croit que ses filtres
			// ont produit ces chiffres).
			summary = {
				unpaidCount: 0,
				unpaidTotal: '0',
				overdueCount: 0,
				overdueTotal: '0',
			};
		} finally {
			if (seq === loadSeq) loading = false;
		}
	}

	$effect(() => {
		if (!mounted) return;
		effectiveSearch;
		paymentStatus;
		contactFilter;
		dueBeforeFilter;
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
			// Defaults raisonnables : dates ASC (chrono), montants/contacts DESC
			// (plus gros / Z d'abord). Branche morte « Asc : Asc » de pass 1
			// remplacée par un vrai choix par colonne.
			sortDirection = col === 'DueDate' || col === 'Date' || col === 'ContactName' ? 'Asc' : 'Desc';
		}
		offset = 0;
	}

	function openMark(item: DueDateItem) {
		markTarget = item;
		markError = '';
		markOpen = true;
	}

	async function handleMarkConfirm(paidAt: string) {
		if (!markTarget) return;
		markSubmitting = true;
		markError = '';
		try {
			await markInvoicePaid(markTarget.id, { paidAt, version: markTarget.version });
			notifySuccess(i18nMsg('invoice-mark-paid-success', 'Facture marquée payée'));
			markOpen = false;
			markTarget = null;
			await load();
		} catch (err) {
			if (isApiError(err)) {
				markError = err.message;
				notifyError(err.message);
				if (err.code === 'OPTIMISTIC_LOCK_CONFLICT') {
					markOpen = false;
					markTarget = null;
					await load();
				}
			} else {
				markError = i18nMsg('common-error', 'Erreur inattendue');
			}
		} finally {
			markSubmitting = false;
		}
	}

	let exportingCsv = $state(false);
	async function onExportCsv() {
		exportingCsv = true;
		try {
			// Propage tous les filtres + tri visibles à l'écran — le CSV
			// exporté doit refléter exactement la liste affichée (review
			// pass 1 G2 D : auparavant sortBy/sortDirection/dates étaient
			// ignorés → CSV désaligné).
			const blob = await exportDueDatesCsv({
				search: effectiveSearch || undefined,
				contactId: contactFilter?.id,
				dueBefore: dueBeforeFilter || undefined,
				paymentStatus,
				sortBy,
				sortDirection,
			});
			const url = URL.createObjectURL(blob);
			const a = document.createElement('a');
			a.href = url;
			const today = new Date().toISOString().slice(0, 10);
			a.download = `echeancier-${today}.csv`;
			document.body.appendChild(a);
			a.click();
			a.remove();
			setTimeout(() => URL.revokeObjectURL(url), 5_000);
		} catch (err) {
			if (isApiError(err)) notifyError(err.message);
			else notifyError(i18nMsg('common-error', 'Erreur inattendue'));
		} finally {
			exportingCsv = false;
		}
	}

	function statusOf(inv: DueDateItem): 'paid' | 'unpaid' | 'overdue' {
		if (inv.paidAt) return 'paid';
		// `isOverdue` peut être `undefined` lors d'un déploiement échelonné
		// (frontend en avance sur backend). Strict comparison `=== true`
		// évite de classer overdue silencieusement toute valeur truthy
		// inattendue, et vice-versa.
		if (inv.isOverdue === true) return 'overdue';
		return 'unpaid';
	}
</script>

<svelte:head>
	<title>{i18nMsg('due-dates-title', 'Échéancier')} — Kesh</title>
</svelte:head>

<div class="mb-6">
	<h1 class="text-2xl font-semibold text-text">{i18nMsg('due-dates-title', 'Échéancier')}</h1>
</div>

<!-- Summary -->
<div class="mb-4 rounded-md border border-border bg-surface-alt px-4 py-3 text-sm">
	<strong>{summary.unpaidCount}</strong>
	{i18nMsg('due-dates-summary-unpaid', 'factures impayées')},
	<strong>CHF {formatInvoiceTotal(summary.unpaidTotal)}</strong>
	{#if summary.overdueCount > 0}
		—
		<span class="text-warning">
			{summary.overdueCount}
			{i18nMsg('due-dates-summary-overdue', 'en retard')}
			(CHF {formatInvoiceTotal(summary.overdueTotal)})
		</span>
	{/if}
</div>

<!-- Filtres -->
<div class="mb-4 flex flex-wrap items-end gap-3">
	<div role="tablist" class="flex gap-1">
		{#each VALID_PAYMENT_STATUS as ps (ps)}
			<button
				role="tab"
				aria-selected={paymentStatus === ps}
				class="rounded-md border px-3 py-1 text-sm"
				class:border-primary={paymentStatus === ps}
				class:bg-primary-light={paymentStatus === ps}
				class:border-border={paymentStatus !== ps}
				onclick={() => {
					paymentStatus = ps;
					offset = 0;
				}}
			>
				{i18nMsg(`due-dates-filter-${ps}`, FILTER_FALLBACK_FR[ps])}
			</button>
		{/each}
	</div>
	<div class="min-w-48 max-w-xs flex-1">
		<label class="mb-1 block text-xs text-text-muted" for="dd-search">
			{i18nMsg('due-dates-search-label', 'Recherche')}
		</label>
		<Input id="dd-search" type="search" value={search} oninput={onSearchInput} />
	</div>
	<div class="min-w-56">
		<label class="mb-1 block text-xs text-text-muted" for="dd-contact">
			{i18nMsg('due-dates-contact-label', 'Contact')}
		</label>
		<ContactPicker
			selected={contactFilter}
			onSelect={(c) => {
				contactFilter = c;
				offset = 0;
			}}
			placeholder={i18nMsg('due-dates-contact-placeholder', 'Tous les contacts')}
		/>
	</div>
	<div>
		<label class="mb-1 block text-xs text-text-muted" for="dd-due-before">
			{i18nMsg('due-dates-due-before-label', 'Échéance avant')}
		</label>
		<Input
			id="dd-due-before"
			type="date"
			value={dueBeforeFilter}
			oninput={(e: Event) => {
				dueBeforeFilter = (e.target as HTMLInputElement).value;
				offset = 0;
			}}
		/>
	</div>
	{#if canExportCsv}
		<Button variant="outline" size="sm" onclick={onExportCsv} disabled={exportingCsv}>
			<Download class="h-4 w-4" aria-hidden="true" />
			{i18nMsg('due-dates-export-button', 'Exporter CSV')}
		</Button>
	{/if}
</div>

{#if loading}
	<p class="text-sm text-text-muted">{i18nMsg('common-loading', 'Chargement…')}</p>
{:else if items.length === 0}
	<p class="text-sm text-text-muted">
		{i18nMsg('due-dates-no-results', 'Aucune facture à afficher.')}
	</p>
{:else}
	<table class="w-full border-collapse text-sm">
		<thead>
			<tr class="border-b border-border text-left">
				<th class="cursor-pointer py-2 pr-2" onclick={() => toggleSort('Date')}>
					{i18nMsg('due-dates-column-date', 'Date')}
				</th>
				<th class="cursor-pointer py-2 pr-2" onclick={() => toggleSort('DueDate')}>
					{i18nMsg('due-dates-column-due-date', 'Échéance')}
				</th>
				<th class="py-2 pr-2">N°</th>
				<th class="cursor-pointer py-2 pr-2" onclick={() => toggleSort('ContactName')}>
					{i18nMsg('due-dates-column-contact', 'Client')}
				</th>
				<th class="cursor-pointer py-2 pr-2 text-right" onclick={() => toggleSort('TotalAmount')}>
					{i18nMsg('due-dates-column-total', 'Total')}
				</th>
				<th class="py-2 pr-2">
					{i18nMsg('due-dates-column-payment-status', 'Statut')}
				</th>
				<th class="py-2 pr-2">
					{i18nMsg('due-dates-column-paid-at', 'Payée le')}
				</th>
				<th class="py-2 pr-2 text-right">Actions</th>
			</tr>
		</thead>
		<tbody>
			{#each items as inv (inv.id)}
				<tr class="border-b border-border" class:row-overdue={inv.isOverdue}>
					<td class="py-2 pr-2">{inv.date}</td>
					<td class="py-2 pr-2">{inv.dueDate ?? '—'}</td>
					<td class="py-2 pr-2">
						<a class="underline" href={`/invoices/${inv.id}`}>{inv.invoiceNumber ?? '—'}</a>
					</td>
					<td class="py-2 pr-2">{inv.contactName}</td>
					<td class="py-2 pr-2 text-right font-mono">{formatInvoiceTotal(inv.totalAmount)}</td>
					<td class="py-2 pr-2">
						<PaymentStatusBadge status={statusOf(inv)} />
					</td>
					<td class="py-2 pr-2">{inv.paidAt ? inv.paidAt.slice(0, 10) : '—'}</td>
					<td class="py-2 pr-2 text-right">
						{#if !inv.paidAt}
							<Button variant="outline" size="sm" onclick={() => openMark(inv)}>
								{i18nMsg('invoice-mark-paid-button', 'Marquer payée')}
							</Button>
						{/if}
					</td>
				</tr>
			{/each}
		</tbody>
	</table>
	<div class="mt-4 flex items-center justify-between text-sm text-text-muted">
		<span>{total} {i18nMsg('due-dates-result-suffix', 'résultat(s)')}</span>
		<div class="flex gap-2">
			<Button
				variant="outline"
				size="sm"
				disabled={offset === 0}
				onclick={() => (offset = Math.max(0, offset - limit))}
			>
				{i18nMsg('common-previous', 'Précédent')}
			</Button>
			<Button
				variant="outline"
				size="sm"
				disabled={offset + limit >= total}
				onclick={() => (offset = offset + limit)}
			>
				{i18nMsg('common-next', 'Suivant')}
			</Button>
		</div>
	</div>
{/if}

{#if markTarget}
	<MarkPaidDialog
		open={markOpen}
		onOpenChange={(o) => {
			markOpen = o;
			if (!o) {
				markError = '';
				if (!markSubmitting) markTarget = null;
			}
		}}
		invoiceDate={markTarget.date}
		submitting={markSubmitting}
		errorMsg={markError}
		onConfirm={handleMarkConfirm}
	/>
{/if}

<style>
	:global(.row-overdue) {
		background-color: color-mix(in srgb, var(--color-warning, #f59e0b) 10%, transparent);
	}
</style>
