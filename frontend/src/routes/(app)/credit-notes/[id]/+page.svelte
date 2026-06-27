<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { ArrowLeft, BookOpen, Printer, FileText } from '@lucide/svelte';
	import { getCreditNote } from '$lib/features/credit-notes/credit-notes.api';
	import {
		creditNoteStatusLabel,
		formatCreditNoteTotal,
	} from '$lib/features/credit-notes/credit-note-helpers';
	import type { CreditNoteResponse } from '$lib/features/credit-notes/credit-notes.types';
	import { apiClient, isApiError } from '$lib/shared/utils/api-client';
	import { notifyError } from '$lib/shared/utils/notify';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';

	let creditNote = $state<CreditNoteResponse | null>(null);
	let loading = $state(true);
	let errorMsg = $state('');
	let pdfDownloading = $state(false);

	let id = $derived(parseInt(page.params.id ?? '', 10));

	onMount(async () => {
		if (!Number.isFinite(id) || id <= 0) {
			errorMsg = 'Identifiant invalide';
			loading = false;
			return;
		}
		try {
			creditNote = await getCreditNote(id);
		} catch (err) {
			if (isApiError(err)) errorMsg = err.message;
		} finally {
			loading = false;
		}
	});

	async function downloadPdf() {
		if (!creditNote) return;
		pdfDownloading = true;
		try {
			const res = await apiClient.getBlob(`/api/v1/credit-notes/${creditNote.id}/pdf`);
			const blob = await res.blob();
			if (blob.size === 0) {
				notifyError(i18nMsg('invoice-pdf-error-empty', 'Le PDF reçu est vide.'));
				return;
			}
			const url = URL.createObjectURL(blob);
			const filename = `avoir-${creditNote.creditNoteNumber ?? creditNote.id}.pdf`;
			const a = document.createElement('a');
			a.href = url;
			a.download = filename;
			a.style.display = 'none';
			document.body.appendChild(a);
			a.click();
			document.body.removeChild(a);
			setTimeout(() => URL.revokeObjectURL(url), 5_000);
		} catch (err) {
			if (isApiError(err)) notifyError(err.message);
			else notifyError(i18nMsg('invoice-pdf-error-generic', 'Échec du téléchargement du PDF.'));
		} finally {
			pdfDownloading = false;
		}
	}
</script>

<svelte:head>
	<title>{i18nMsg('credit-notes-title', 'Avoir')} — Kesh</title>
</svelte:head>

<div class="mb-6 flex items-center justify-between">
	<Button variant="ghost" onclick={() => goto('/credit-notes')}>
		<ArrowLeft class="h-4 w-4" aria-hidden="true" />
		{i18nMsg('common-back', 'Retour')}
	</Button>
	{#if creditNote}
		<div class="flex gap-2">
			<Button onclick={downloadPdf} disabled={pdfDownloading}>
				<Printer class="h-4 w-4" aria-hidden="true" />
				{i18nMsg('credit-notes-download-pdf', 'Imprimer / Télécharger PDF')}
			</Button>
			{#if creditNote.journalEntryId}
				<Button
					variant="outline"
					onclick={() => goto(`/journal-entries/${creditNote!.journalEntryId}`)}
				>
					<BookOpen class="h-4 w-4" aria-hidden="true" />
					{i18nMsg('credit-notes-view-entry', 'Voir l’écriture comptable')}
				</Button>
			{/if}
			<Button variant="outline" onclick={() => goto(`/invoices/${creditNote!.invoiceId}`)}>
				<FileText class="h-4 w-4" aria-hidden="true" />
				{i18nMsg('credit-notes-view-invoice', 'Voir la facture annulée')}
			</Button>
		</div>
	{/if}
</div>

{#if loading}
	<p class="text-sm text-text-muted">Chargement…</p>
{:else if errorMsg}
	<p class="text-sm text-destructive">{errorMsg}</p>
{:else if creditNote}
	<h1 class="mb-1 text-2xl font-semibold" data-testid="credit-note-number">
		{i18nMsg('credit-notes-title', 'Avoir')}
		{creditNote.creditNoteNumber ?? `#${creditNote.id}`}
	</h1>
	<p class="mb-6 text-sm text-text-muted">
		{creditNote.date} · {creditNoteStatusLabel(creditNote.status)}
	</p>

	<table class="w-full border-collapse text-sm">
		<thead>
			<tr class="border-b text-left text-text-muted">
				<th class="py-2">{i18nMsg('credit-notes-col-description', 'Description')}</th>
				<th class="py-2 text-right">{i18nMsg('credit-notes-col-qty', 'Qté')}</th>
				<th class="py-2 text-right">{i18nMsg('credit-notes-col-unit-price', 'Prix unitaire')}</th>
				<th class="py-2 text-right">{i18nMsg('credit-notes-col-vat', 'TVA %')}</th>
				<th class="py-2 text-right">{i18nMsg('credit-notes-col-line-total', 'Total HT')}</th>
			</tr>
		</thead>
		<tbody>
			{#each creditNote.lines as line (line.position)}
				<tr class="border-b">
					<td class="py-2">{line.description}</td>
					<td class="py-2 text-right">{line.quantity}</td>
					<td class="py-2 text-right">{formatCreditNoteTotal(line.unitPrice)}</td>
					<td class="py-2 text-right">{line.vatRate}</td>
					<td class="py-2 text-right">{formatCreditNoteTotal(line.lineTotal)}</td>
				</tr>
			{/each}
		</tbody>
		<tfoot>
			<tr class="font-semibold">
				<td class="py-2" colspan="4">{i18nMsg('credit-notes-col-total', 'Total HT')}</td>
				<td class="py-2 text-right" data-testid="credit-note-total">
					{formatCreditNoteTotal(creditNote.totalAmount)}
				</td>
			</tr>
		</tfoot>
	</table>
{/if}
