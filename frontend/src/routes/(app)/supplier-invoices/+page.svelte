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
	 */
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
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

	let items = $state<SupplierInvoiceListItemResponse[]>([]);
	let loading = $state(true);
	let errorMsg = $state('');
	let showCancelled = $state(false);

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

	async function load() {
		const seq = ++requestSeq;
		errorMsg = '';
		try {
			const res = await listSupplierInvoices(
				showCancelled ? { limit: 100 } : { limit: 100, status: WITHOUT_CANCELLED },
			);
			if (seq !== requestSeq) return;
			items = res.items;
		} catch (err) {
			if (seq !== requestSeq) return;
			if (isApiError(err)) errorMsg = err.message;
		} finally {
			if (seq === requestSeq) loading = false;
		}
	}

	onMount(load);
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

<label class="mb-4 flex items-center gap-2 text-sm">
	<input
		type="checkbox"
		data-testid="supplier-invoices-show-cancelled"
		bind:checked={showCancelled}
		onchange={() => void load()}
	/>
	{i18nMsg('supplier-invoices-show-cancelled', 'Afficher les factures annulées')}
</label>

{#if loading}
	<p class="text-sm text-text-muted">{i18nMsg('common-loading', 'Chargement…')}</p>
{:else if errorMsg}
	<p class="text-sm text-destructive">{errorMsg}</p>
{:else if items.length === 0}
	<p class="text-sm text-text-muted">
		{i18nMsg('supplier-invoices-empty', 'Aucune facture fournisseur enregistrée.')}
	</p>
{:else}
	<table class="w-full border-collapse text-sm" data-testid="supplier-invoices-table">
		<thead>
			<tr class="border-b text-left text-text-muted">
				<th class="py-2">{i18nMsg('supplier-invoices-col-number', 'N°')}</th>
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
					<td class="py-2 font-mono">{inv.supplierInvoiceNumber ?? `#${inv.id}`}</td>
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
{/if}
