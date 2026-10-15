<script lang="ts">
	/**
	 * Liste des factures fournisseurs — Story 12.2.
	 *
	 * Story 30-2b (#685, AC10, C-30-2-11) : le formulaire de saisie en ligne de cette page
	 * a disparu au profit de la page `/supplier-invoices/new` (composant
	 * `SupplierInvoiceForm`, une seule définition) ; « Enregistrer une facture » y mène,
	 * pour les rôles qui peuvent créer (Comptable+).
	 */
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { authState } from '$lib/app/stores/auth.svelte';
	import { listSupplierInvoices } from '$lib/features/supplier-invoices/supplier-invoices.api';
	import {
		formatSupplierInvoiceTotal,
		supplierInvoiceStatusLabel,
	} from '$lib/features/supplier-invoices/supplier-invoice-helpers';
	import type { SupplierInvoiceListItemResponse } from '$lib/features/supplier-invoices/supplier-invoices.types';
	import { isApiError } from '$lib/shared/utils/api-client';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';

	let items = $state<SupplierInvoiceListItemResponse[]>([]);
	let loading = $state(true);
	let errorMsg = $state('');

	const canWrite = $derived(
		authState.currentUser?.role === 'Admin' || authState.currentUser?.role === 'Comptable',
	);

	onMount(async () => {
		try {
			const res = await listSupplierInvoices({ limit: 100 });
			items = res.items;
		} catch (err) {
			if (isApiError(err)) errorMsg = err.message;
		} finally {
			loading = false;
		}
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
					<td class="py-2">{inv.invoiceDate}</td>
					<td class="py-2">{inv.dueDate ?? '—'}</td>
					<td class="py-2">{supplierInvoiceStatusLabel(inv.status)}</td>
					<td class="py-2 text-right">{formatSupplierInvoiceTotal(inv.totalAmount)}</td>
				</tr>
			{/each}
		</tbody>
	</table>
{/if}
