<script lang="ts">
	import InvoiceForm from '$lib/components/invoices/InvoiceForm.svelte';
	import { onMount } from 'svelte';
	import { page } from '$app/state';
	import { getInvoice } from '$lib/features/invoices/invoices.api';
	import type { InvoiceResponse } from '$lib/features/invoices/invoices.types';
	import { isApiError } from '$lib/shared/utils/api-client';

	let invoice = $state<InvoiceResponse | null>(null);
	let loading = $state(true);
	let errorMsg = $state('');

	onMount(async () => {
		const id = parseInt(page.params.id ?? '', 10);
		if (!Number.isFinite(id) || id <= 0) {
			errorMsg = 'Identifiant de facture invalide';
			loading = false;
			return;
		}
		try {
			invoice = await getInvoice(id);
		} catch (err) {
			if (isApiError(err)) errorMsg = err.message;
		} finally {
			loading = false;
		}
	});
</script>

<svelte:head>
	<title>Modifier facture — Kesh</title>
</svelte:head>

<h1 class="mb-6 text-2xl font-semibold text-text">Modifier la facture</h1>
{#if loading}
	<p class="text-sm text-text-muted">Chargement…</p>
{:else if errorMsg}
	<p class="text-sm text-destructive">{errorMsg}</p>
{:else if invoice}
	<InvoiceForm {invoice} />
{/if}
