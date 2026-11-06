<script lang="ts">
	/**
	 * Liste des factures fournisseurs — Story 12.2.
	 *
	 * Story 30-2b (#685, AC10, C-30-2-11) : le formulaire de saisie en ligne de cette page
	 * a disparu au profit de la page `/supplier-invoices/new` (composant
	 * `SupplierInvoiceForm`, une seule définition) ; « Enregistrer une facture » y mène,
	 * pour les rôles qui peuvent créer (Comptable+).
	 *
	 * Story v017-4b (#849, #831 ; AC-21) :
	 * - le statut s'affiche en **badge** — « Brouillon », « Validée » (l'état stocké
	 *   `open`, décision Q4), « Payée », « Annulée » ;
	 * - les factures **annulées** sont masquées par défaut (C-v017-4-14, C-v017-4-21) : la
	 *   liste demande `?status=draft,open,paid` ; la case « Afficher les factures
	 *   annulées » la relit sans filtre. Leur écriture d'achat et sa contre-passation
	 *   restent au journal (limite L6) ;
	 * - le TTC s'affiche aux **décimales exactes** (`formatExactAmount`, #831) : un total
	 *   sous le centime ne s'arrondit plus à deux décimales.
	 *
	 * Story v0172-correctifs (#870, C-v0172b-7, C-v0172b-26) — la liste **se pagine** (50 par
	 * page, *Précédent* / *Suivant*, le total) et **se filtre par statut depuis l'adresse** :
	 * `?status=` (une valeur parmi `draft`, `open`, `paid`, `cancelled`), `?cancelled=1` (la
	 * case), `?offset=`, `?limit=` (borné à [1, 200], comme le serveur). L'adresse est la
	 * **source de vérité**, lue de façon réactive (`page.url`, `$derived`) : un lien vers
	 * `/supplier-invoices` depuis la page filtrée, ou le retour arrière, relit la liste. Les
	 * gestes (boutons, case) **naviguent**, ils ne posent pas d'état local. Le filtre ne porte
	 * que sur le statut : depuis le contrôle de clôture K8, la liste montre les brouillons de
	 * tous les exercices.
	 *
	 * Story 33-1b (#926) — deux colonnes, **N° interne** (triable) et **N° fournisseur** ;
	 * `?sort=internalNumber`, `?order=asc|desc` et `?q=` (numéro interne, numéro et nom du
	 * fournisseur) passent par l'adresse, comme le reste : tri et recherche se font au serveur,
	 * la liste étant paginée, et un geste ramène `offset` à 0.
	 */
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { authState } from '$lib/app/stores/auth.svelte';
	import { listSupplierInvoices } from '$lib/features/supplier-invoices/supplier-invoices.api';
	import { supplierInvoiceStatusLabel } from '$lib/features/supplier-invoices/supplier-invoice-helpers';
	import { formatExactAmount } from '$lib/features/invoices/invoice-helpers';
	import { formatSwissDate } from '$lib/features/reports/reports.api';
	import type {
		SupplierInvoiceListItemResponse,
		SupplierInvoiceStatus,
	} from '$lib/features/supplier-invoices/supplier-invoices.types';
	import { isApiError } from '$lib/shared/utils/api-client';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';

	/** Les statuts listés par défaut : tout, sauf les annulées. */
	const WITHOUT_CANCELLED: SupplierInvoiceStatus[] = ['draft', 'open', 'paid'];
	/** Les valeurs admises de `?status=` ; toute autre est ignorée, jamais envoyée. */
	const STATUSES: readonly SupplierInvoiceStatus[] = ['draft', 'open', 'paid', 'cancelled'];
	/** Taille de page par défaut et bornes de `?limit=` — celles du serveur. */
	const DEFAULT_LIMIT = 50;
	const MAX_LIMIT = 200;

	/** Un entier de l'adresse, ou `null` s'il n'en est pas un. */
	function intParam(raw: string | null): number | null {
		if (raw === null || !/^\d+$/.test(raw)) return null;
		const n = Number(raw);
		return Number.isSafeInteger(n) ? n : null;
	}

	const statusFilter = $derived.by((): SupplierInvoiceStatus | null => {
		const raw = page.url.searchParams.get('status');
		return STATUSES.find((s) => s === raw) ?? null;
	});
	/** La case « annulées » : ignorée quand un statut est imposé. */
	const showCancelled = $derived(
		statusFilter === null && page.url.searchParams.get('cancelled') === '1',
	);
	const offset = $derived(intParam(page.url.searchParams.get('offset')) ?? 0);
	const limit = $derived.by(() => {
		const n = intParam(page.url.searchParams.get('limit'));
		return n === null || n < 1 ? DEFAULT_LIMIT : Math.min(n, MAX_LIMIT);
	});
	/** La recherche de l'adresse ; une valeur blanche ne filtre pas. */
	const searchTerm = $derived((page.url.searchParams.get('q') ?? '').trim());
	/** Le tri de l'adresse : une seule colonne, toute autre valeur est ignorée. */
	const sortedByNumber = $derived(page.url.searchParams.get('sort') === 'internalNumber');
	const sortOrder = $derived.by((): 'asc' | 'desc' =>
		page.url.searchParams.get('order') === 'desc' ? 'desc' : 'asc',
	);

	/** Le champ de recherche ; l'adresse ne suit qu'après la frappe temporisée. */
	let searchInput = $state('');
	let searchTimer: ReturnType<typeof setTimeout> | undefined;
	const SEARCH_DEBOUNCE_MS = 300;

	// L'adresse change sans passer par le champ (lien, retour arrière) : le champ la suit.
	$effect(() => {
		const term = page.url.searchParams.get('q') ?? '';
		if (searchTimer === undefined) searchInput = term;
	});

	function onSearchInput(value: string): void {
		searchInput = value;
		clearTimeout(searchTimer);
		searchTimer = setTimeout(() => {
			searchTimer = undefined;
			navigate({ q: value.trim() === '' ? null : value, offset: null }, { replaceState: true });
		}, SEARCH_DEBOUNCE_MS);
	}

	/** En-tête N° interne : non trié → croissant → décroissant → croissant. */
	function toggleNumberSort(): void {
		const next = sortedByNumber && sortOrder === 'asc' ? 'desc' : 'asc';
		navigate({ sort: 'internalNumber', order: next, offset: null });
	}

	let items = $state<SupplierInvoiceListItemResponse[]>([]);
	let total = $state(0);
	let loading = $state(true);
	let errorMsg = $state('');

	/**
	 * Navigue vers la liste avec les paramètres `changes` appliqués à l'adresse courante
	 * (`null` retire le paramètre). `offset` 0 ne s'écrit pas.
	 */
	function navigate(
		changes: Record<string, string | null>,
		opts: { replaceState?: boolean } = {},
	): void {
		const params = new URLSearchParams(page.url.searchParams);
		for (const [k, v] of Object.entries(changes)) {
			if (v === null || (k === 'offset' && v === '0')) params.delete(k);
			else params.set(k, v);
		}
		const qs = params.toString();
		void goto(`/supplier-invoices${qs ? `?${qs}` : ''}`, {
			keepFocus: true,
			noScroll: true,
			replaceState: opts.replaceState ?? false,
		});
	}

	const canWrite = $derived(
		authState.currentUser?.role === 'Admin' || authState.currentUser?.role === 'Comptable',
	);

	/** Classe du badge de statut : le brouillon se distingue, l'annulée s'efface. */
	function badgeClass(status: SupplierInvoiceStatus): string {
		switch (status) {
			case 'draft':
				// Revue P1 : `bg-warning-soft` n'est défini nulle part (#645) ; le fond
				// pâle se dit par le jeton défini `warning` à 10 %.
				return 'border-warning bg-warning/10';
			case 'cancelled':
				return 'border-border text-text-muted';
			default:
				return 'border-border';
		}
	}

	/**
	 * Revue P1 (B-4 = E-8) — le jeton de la dernière lecture : deux bascules rapprochées de
	 * la case lancent deux lectures ; seule la réponse de la plus récente s'affiche, sans
	 * quoi la case pourrait dire « masquées » au-dessus d'une liste qui les montre.
	 */
	let requestSeq = 0;

	async function load(
		filter: SupplierInvoiceStatus | null,
		cancelled: boolean,
		off: number,
		lim: number,
		q: string,
		sorted: boolean,
		order: 'asc' | 'desc',
	) {
		const seq = ++requestSeq;
		errorMsg = '';
		try {
			const res = await listSupplierInvoices({
				limit: lim,
				offset: off,
				...(filter
					? { status: [filter] }
					: cancelled
						? {}
						: { status: WITHOUT_CANCELLED }),
				...(q ? { q } : {}),
				...(sorted ? { sort: 'internalNumber' as const, order } : {}),
			});
			if (seq !== requestSeq) return;
			// Page au-delà du total (dernière ligne supprimée, signet ancien) : la
			// dernière page, sans nouvelle entrée d'historique.
			if (res.items.length === 0 && res.total > 0 && off > 0) {
				navigate(
					{ offset: String(Math.floor((res.total - 1) / lim) * lim) },
					{ replaceState: true },
				);
				return;
			}
			items = res.items;
			total = res.total;
		} catch (err) {
			if (seq !== requestSeq) return;
			if (isApiError(err)) errorMsg = err.message;
		} finally {
			if (seq === requestSeq) loading = false;
		}
	}

	// L'adresse change (geste, lien, retour arrière) : la liste se relit.
	$effect(() => {
		void load(statusFilter, showCancelled, offset, limit, searchTerm, sortedByNumber, sortOrder);
	});
</script>

<svelte:head>
	<title>{i18nMsg('supplier-invoices-title', 'Factures fournisseurs')} — Kesh</title>
</svelte:head>

<div class="mb-6 flex items-center justify-between">
	<h1 class="text-2xl font-semibold">
		{i18nMsg('supplier-invoices-title', 'Factures fournisseurs')}
	</h1>
	{#if canWrite}
		<a
			class="rounded bg-primary px-4 py-2 text-sm text-primary-foreground"
			data-testid="supplier-invoice-new"
			href="/supplier-invoices/new"
		>
			{i18nMsg('supplier-invoices-new', 'Enregistrer une facture')}
		</a>
	{/if}
</div>

{#if statusFilter}
	<div
		class="mb-4 flex flex-wrap items-center gap-3 rounded border border-border bg-surface-alt px-3 py-2 text-sm"
		data-testid="supplier-invoices-filter-banner"
	>
		<span>
			{i18nMsg(
				'supplier-invoices-filter-status',
				'Seules les factures « { $status } » sont affichées, tous exercices confondus.',
				{ status: supplierInvoiceStatusLabel(statusFilter) },
			)}
		</span>
		<!-- Revue de code P1 (E-5, C-v0172b-35) : « toutes » comprend les annulées. -->
		<a class="text-primary underline" href="/supplier-invoices?cancelled=1" data-testid="supplier-invoices-filter-clear">
			{i18nMsg('supplier-invoices-filter-show-all', 'Afficher toutes les factures')}
		</a>
	</div>
{:else}
	<label class="mb-4 flex items-center gap-2 text-sm">
		<input
			type="checkbox"
			data-testid="supplier-invoices-show-cancelled"
			checked={showCancelled}
			onchange={(e) =>
				navigate({ cancelled: e.currentTarget.checked ? '1' : null, offset: null })}
		/>
		{i18nMsg('supplier-invoices-show-cancelled', 'Afficher les factures annulées')}
	</label>
{/if}

<label class="mb-4 block max-w-md text-sm">
	<span class="mb-1 block">
		{i18nMsg('supplier-invoices-search-label', 'Rechercher un numéro ou un fournisseur')}
	</span>
	<input
		type="search"
		class="w-full rounded border px-2 py-1"
		data-testid="supplier-invoices-search"
		value={searchInput}
		oninput={(e) => onSearchInput(e.currentTarget.value)}
		maxlength="128"
	/>
</label>

{#if loading}
	<p class="text-sm text-text-muted">{i18nMsg('common-loading', 'Chargement…')}</p>
{:else if errorMsg}
	<p class="text-sm text-destructive">{errorMsg}</p>
{:else if items.length === 0}
	<!-- Revue de code P1 (B-4 = E-4) : le message dit ce qui est vide — le filtre, la liste
	     sans les annulées, ou toute la liste. -->
	<p class="text-sm text-text-muted" data-testid="supplier-invoices-empty">
		{#if searchTerm}
			{i18nMsg('supplier-invoices-empty-search', 'Aucune facture ne correspond à « { $term } ».', {
				term: searchTerm,
			})}
		{:else if statusFilter}
			{i18nMsg('supplier-invoices-empty-filtered', 'Aucune facture « { $status } ».', {
				status: supplierInvoiceStatusLabel(statusFilter),
			})}
		{:else if !showCancelled}
			{i18nMsg(
				'supplier-invoices-empty-active',
				'Aucune facture fournisseur à afficher : les factures annulées sont masquées.',
			)}
		{:else}
			{i18nMsg('supplier-invoices-empty', 'Aucune facture fournisseur enregistrée.')}
		{/if}
	</p>
{:else}
	<table class="w-full border-collapse text-sm" data-testid="supplier-invoices-table">
		<thead>
			<tr class="border-b text-left text-text-muted">
				<th
					class="py-2"
					aria-sort={sortedByNumber ? (sortOrder === 'asc' ? 'ascending' : 'descending') : 'none'}
				>
					<button
						type="button"
						class="underline-offset-2 hover:underline"
						data-testid="supplier-invoices-sort-internal-number"
						onclick={toggleNumberSort}
					>
						{i18nMsg('supplier-invoices-col-internal-number', 'N° interne')}
						{#if sortedByNumber}<span aria-hidden="true">{sortOrder === 'asc' ? '▲' : '▼'}</span>{/if}
					</button>
				</th>
				<th class="py-2">{i18nMsg('supplier-invoices-col-number', 'N° fournisseur')}</th>
				<th class="py-2">{i18nMsg('supplier-invoices-col-supplier', 'Fournisseur')}</th>
				<th class="py-2">{i18nMsg('supplier-invoices-col-date', 'Date')}</th>
				<th class="py-2">{i18nMsg('supplier-invoices-col-due', 'Échéance')}</th>
				<th class="py-2">{i18nMsg('supplier-invoices-col-status', 'Statut')}</th>
				<th class="py-2 text-right">{i18nMsg('supplier-invoices-col-total', 'TTC')}</th>
			</tr>
		</thead>
		<tbody>
			{#each items as inv (inv.id)}
				<tr
					class="cursor-pointer border-b hover:bg-surface-hover"
					data-testid="supplier-invoice-row"
					onclick={() => goto(`/supplier-invoices/${inv.id}`)}
				>
					<td class="py-2 font-mono" data-testid="supplier-invoice-row-internal-number"
						>{inv.internalNumber ?? '—'}</td
					>
					<td class="py-2 font-mono" data-testid="supplier-invoice-row-supplier-number"
						>{inv.supplierInvoiceNumber ?? '—'}</td
					>
					<td class="py-2">{inv.contactName}</td>
					<!-- Story v0171 (#871) : dates en forme suisse. -->
					<td class="py-2" data-testid="supplier-invoice-row-date">{formatSwissDate(inv.invoiceDate)}</td>
					<td class="py-2" data-testid="supplier-invoice-row-due">
						{inv.dueDate ? formatSwissDate(inv.dueDate) : '—'}
					</td>
					<td class="py-2">
						<span
							class="inline-block rounded border px-2 py-0.5 text-xs {badgeClass(inv.status)}"
							data-testid="supplier-invoice-status-badge"
							data-status={inv.status}>{supplierInvoiceStatusLabel(inv.status)}</span
						>
					</td>
					<td class="py-2 text-right" data-testid="supplier-invoice-row-total"
						>{formatExactAmount(inv.totalAmount)}</td
					>
				</tr>
			{/each}
		</tbody>
	</table>
	<div class="mt-4 flex items-center justify-between text-sm text-text-muted">
		<span data-testid="supplier-invoices-count">
			{total > 1
				? i18nMsg('supplier-invoices-count-other', '{ $count } factures', { count: total })
				: i18nMsg('supplier-invoices-count-one', '{ $count } facture', { count: total })}
		</span>
		<div class="flex gap-2">
			<button
				type="button"
				class="rounded border border-border px-3 py-1 disabled:opacity-50"
				data-testid="supplier-invoices-prev"
				disabled={offset === 0}
				onclick={() => navigate({ offset: String(Math.max(0, offset - limit)) })}
			>
				{i18nMsg('common-previous', 'Précédent')}
			</button>
			<button
				type="button"
				class="rounded border border-border px-3 py-1 disabled:opacity-50"
				data-testid="supplier-invoices-next"
				disabled={offset + limit >= total}
				onclick={() => navigate({ offset: String(offset + limit) })}
			>
				{i18nMsg('common-next', 'Suivant')}
			</button>
		</div>
	</div>
{/if}
