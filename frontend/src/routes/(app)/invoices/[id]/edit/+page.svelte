<script lang="ts">
	import InvoiceForm from '$lib/components/invoices/InvoiceForm.svelte';
	import { onMount } from 'svelte';
	import { page } from '$app/state';
	import { getInvoice } from '$lib/features/invoices/invoices.api';
	import type { InvoiceResponse } from '$lib/features/invoices/invoices.types';
	import { isApiError } from '$lib/shared/utils/api-client';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { authState } from '$lib/app/stores/auth.svelte';
	import { LINK_UNDERLINE_CLASS } from '$lib/shared/navigation/link-class';

	let invoice = $state<InvoiceResponse | null>(null);
	let loading = $state(true);
	let errorMsg = $state('');

	// Story 64-1 (#1758) : `PUT /invoices/{id}` est réservé à l'Administrateur et au
	// Comptable (`comptable_routes`) ; un autre rôle ne voit jamais le formulaire.
	const canWrite = $derived(
		authState.currentUser?.role === 'Admin' || authState.currentUser?.role === 'Comptable',
	);

	onMount(async () => {
		if (!canWrite) {
			loading = false;
			return;
		}
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
{#if !canWrite}
	<p class="text-sm text-text-muted" data-testid="invoice-edit-forbidden">
		{i18nMsg('invoice-edit-forbidden', 'Votre rôle ne permet pas de modifier une facture.')}
	</p>
	<a
		class="mt-3 inline-block text-sm text-primary {LINK_UNDERLINE_CLASS}"
		href="/invoices/{page.params.id}"
		data-testid="invoice-edit-back"
	>
		{i18nMsg('invoice-edit-back', 'Revenir à la facture')}
	</a>
{:else if loading}
	<p class="text-sm text-text-muted">Chargement…</p>
{:else if errorMsg}
	<p class="text-sm text-destructive">{errorMsg}</p>
{:else if invoice && invoice.status !== 'draft'}
	<!-- Seul un brouillon se modifie ; le 409 du serveur reste la garde de fond. -->
	<p class="text-sm text-text-muted" data-testid="invoice-edit-not-draft">
		{invoice.status === 'cancelled'
			? i18nMsg('invoice-edit-not-draft-cancelled', 'Cette facture est annulée : elle ne se modifie plus.')
			: i18nMsg('invoice-edit-not-draft-validated', 'Cette facture est validée : elle ne se modifie plus.')}
	</p>
	<a
		class="mt-3 inline-block text-sm text-primary {LINK_UNDERLINE_CLASS}"
		href="/invoices/{invoice.id}"
		data-testid="invoice-edit-back"
	>
		{i18nMsg('invoice-edit-back', 'Revenir à la facture')}
	</a>
{:else if invoice}
	<InvoiceForm {invoice} />
{/if}
