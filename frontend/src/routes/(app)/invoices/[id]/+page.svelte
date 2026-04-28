<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import * as Dialog from '$lib/components/ui/dialog';
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { Pencil, Trash2, ArrowLeft, CheckCircle2, BookOpen, Download } from '@lucide/svelte';

	import {
		getInvoice,
		deleteInvoice,
		validateInvoice,
		markInvoicePaid,
		unmarkInvoicePaid,
	} from '$lib/features/invoices/invoices.api';
	import PaymentStatusBadge from '$lib/features/invoices/PaymentStatusBadge.svelte';
	import MarkPaidDialog from '$lib/features/invoices/MarkPaidDialog.svelte';
	import type { InvoiceResponse } from '$lib/features/invoices/invoices.types';
	import { formatInvoiceTotal } from '$lib/features/invoices/invoice-helpers';
	import { apiClient, isApiError } from '$lib/shared/utils/api-client';
	import {
		notifyError,
		notifyMissingFiscalYearOrFallback,
		notifySuccess
	} from '$lib/shared/utils/notify';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { authState } from '$lib/app/stores/auth.svelte';

	let invoice = $state<InvoiceResponse | null>(null);
	let loading = $state(true);
	let errorMsg = $state('');
	let deleteOpen = $state(false);
	let deleteSubmitting = $state(false);
	let deleteError = $state('');
	let validateOpen = $state(false);
	let validateSubmitting = $state(false);
	let validateError = $state('');

	let id = $derived(parseInt(page.params.id ?? '', 10));

	onMount(async () => {
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

	async function confirmDelete() {
		if (!invoice) return;
		deleteSubmitting = true;
		deleteError = '';
		try {
			await deleteInvoice(invoice.id);
			notifySuccess('Facture supprimée');
			goto('/invoices');
		} catch (err) {
			// Cohérence avec la modale de conflit (InvoiceForm) : erreur visible
			// dans la modale en plus du toast.
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

	async function confirmValidate() {
		if (!invoice) return;
		validateSubmitting = true;
		validateError = '';
		try {
			const updated = await validateInvoice(invoice.id);
			invoice = updated;
			// Review edge #13 : fallback si invoiceNumber null (ne devrait pas arriver mais défensif).
			notifySuccess(
				updated.invoiceNumber ? `Facture validée — ${updated.invoiceNumber}` : 'Facture validée',
			);
			validateOpen = false;
		} catch (err) {
			if (isApiError(err)) {
				validateError = err.message;
				// Story 3.7 AC #22 — fallback toast actionnable pour FISCAL_YEAR_INVALID.
				if (notifyMissingFiscalYearOrFallback(err)) {
					validateOpen = false;
					return;
				}
				if (err.code === 'CONFIGURATION_REQUIRED') {
					if (authState.currentUser?.role === 'Admin') {
						validateError = `${err.message} — Configurez les comptes par défaut dans Paramètres > Facturation.`;
					} else {
						validateError =
							'Demandez à votre administrateur de configurer les comptes par défaut de facturation.';
					}
				}
				notifyError(validateError);
				// Si conflit d'état (409) : la facture a peut-être changé — reload.
				if (err.code === 'ILLEGAL_STATE_TRANSITION' || err.code === 'OPTIMISTIC_LOCK_CONFLICT') {
					try {
						invoice = await getInvoice(invoice.id);
					} catch {
						// laisser l'erreur affichée
					}
					validateOpen = false;
				}
				// Review P13 : fermer le dialog sur erreurs non-retryables
				// (l'utilisateur doit corriger la config ou l'exercice avant de réessayer).
				if (
					err.code === 'FISCAL_YEAR_INVALID' ||
					err.code === 'CONFIGURATION_REQUIRED'
				) {
					validateOpen = false;
				}
			} else {
				validateError = 'Erreur lors de la validation';
				notifyError(validateError);
			}
		} finally {
			validateSubmitting = false;
		}
	}

	// Story 5.4 — mark/unmark paid
	let markOpen = $state(false);
	let markSubmitting = $state(false);
	let markError = $state('');
	let unmarkOpen = $state(false);
	let unmarkSubmitting = $state(false);
	let unmarkError = $state('');

	function paymentStatus(inv: InvoiceResponse): 'paid' | 'unpaid' | 'overdue' {
		if (inv.paidAt) return 'paid';
		// P6 (review pass 2) : `isOverdue` est calculé backend → single source
		// of truth pour « aujourd'hui » (évite la désync TZ client/serveur).
		// Pass 2 G2 D : strict `=== true` symétrique avec la page échéancier
		// (defensive contre `undefined` en rollout échelonné).
		return inv.isOverdue === true ? 'overdue' : 'unpaid';
	}

	async function handleMarkConfirm(paidAt: string) {
		if (!invoice) return;
		markSubmitting = true;
		markError = '';
		try {
			invoice = await markInvoicePaid(invoice.id, { paidAt, version: invoice.version });
			notifySuccess(i18nMsg('invoice-mark-paid-success', 'Facture marquée payée'));
			markOpen = false;
		} catch (err) {
			if (isApiError(err)) {
				markError = err.message;
				notifyError(err.message);
				if (err.code === 'OPTIMISTIC_LOCK_CONFLICT') {
					try {
						invoice = await getInvoice(invoice.id);
					} catch {
						// noop
					}
					markOpen = false;
				}
			} else {
				markError = i18nMsg('common-error', 'Erreur inattendue');
			}
		} finally {
			markSubmitting = false;
		}
	}

	async function confirmUnmark() {
		if (!invoice) return;
		unmarkSubmitting = true;
		unmarkError = '';
		try {
			invoice = await unmarkInvoicePaid(invoice.id, { version: invoice.version });
			notifySuccess(i18nMsg('invoice-unmark-paid-success', 'Marquage paiement annulé'));
			unmarkOpen = false;
		} catch (err) {
			if (isApiError(err)) {
				unmarkError = err.message;
				notifyError(err.message);
				if (err.code === 'OPTIMISTIC_LOCK_CONFLICT') {
					try {
						invoice = await getInvoice(invoice.id);
					} catch {
						// noop
					}
					unmarkOpen = false;
				}
			} else {
				unmarkError = i18nMsg('common-error', 'Erreur inattendue');
			}
		} finally {
			unmarkSubmitting = false;
		}
	}

	let pdfDownloading = $state(false);

	// D2 (review pass 1 G2 D) : whitelist explicite des codes d'erreur PDF
	// — empêche la construction dynamique de clés FTL depuis err.code
	// (potentiel mismatch silencieux si le backend renvoie un nouveau code).
	const PDF_ERROR_KEYS: Record<string, string> = {
		INVOICE_NOT_VALIDATED: 'invoice-pdf-error-invoice-not-validated',
		INVOICE_NOT_PDF_READY: 'invoice-pdf-error-invoice-not-pdf-ready',
		INVOICE_TOO_MANY_LINES_FOR_PDF: 'error-invoice-too-many-lines-for-pdf',
		PDF_GENERATION_FAILED: 'invoice-pdf-error-pdf-generation-failed',
		NOT_FOUND: 'invoice-pdf-error-not-found',
	};

	async function downloadPdf() {
		if (!invoice) return;
		pdfDownloading = true;
		try {
			const res = await apiClient.getBlob(`/api/v1/invoices/${invoice.id}/pdf`);
			const blob = await res.blob();
			if (blob.size === 0) {
				notifyError(i18nMsg('invoice-pdf-error-empty', 'Le PDF reçu est vide.'));
				return;
			}
			const url = URL.createObjectURL(blob);
			// D2 (review pass 1 G2 D) : pattern <a download> hidden — aligné
			// sur le download CSV échéancier, évite popup-blocker (Firefox
			// strict / Safari mobile) et garantit semantics de téléchargement.
			const filename = `facture-${invoice.invoiceNumber ?? invoice.id}.pdf`;
			const a = document.createElement('a');
			a.href = url;
			a.download = filename;
			a.style.display = 'none';
			document.body.appendChild(a);
			a.click();
			document.body.removeChild(a);
			// Revoke différé pour laisser le navigateur récupérer le blob.
			setTimeout(() => URL.revokeObjectURL(url), 5_000);
		} catch (err) {
			if (isApiError(err)) {
				// Pass 2 : remappage vers les clés FTL réellement présentes —
		// `INVOICE_TOO_MANY_LINES_FOR_PDF` utilise la clé legacy
		// `error-invoice-too-many-lines-for-pdf` (existante FR/DE/IT/EN
		// avec les arguments {count}/{max}).
		const key = PDF_ERROR_KEYS[err.code] ?? 'invoice-pdf-error-generic';
				notifyError(i18nMsg(key, err.message));
			} else {
				notifyError(i18nMsg('invoice-pdf-error-generic', 'Erreur lors du téléchargement du PDF'));
			}
		} finally {
			pdfDownloading = false;
		}
	}

	function statusLabel(s: string): string {
		if (s === 'draft') return 'Brouillon';
		if (s === 'validated') return 'Validée';
		return 'Annulée';
	}
</script>

<svelte:head>
	<title>Facture — Kesh</title>
</svelte:head>

<div class="mb-6 flex items-center justify-between">
	<Button variant="ghost" onclick={() => goto('/invoices')}>
		<ArrowLeft class="h-4 w-4" aria-hidden="true" />
		Retour
	</Button>
	{#if invoice?.status === 'draft'}
		<div class="flex gap-2">
			<Button onclick={() => (validateOpen = true)}>
				<CheckCircle2 class="h-4 w-4" aria-hidden="true" />
				Valider
			</Button>
			<Button variant="outline" onclick={() => goto(`/invoices/${invoice!.id}/edit`)}>
				<Pencil class="h-4 w-4" aria-hidden="true" />
				Modifier
			</Button>
			<Button variant="destructive" onclick={() => (deleteOpen = true)}>
				<Trash2 class="h-4 w-4" aria-hidden="true" />
				Supprimer
			</Button>
		</div>
	{:else if invoice?.status === 'validated'}
		<div class="flex gap-2">
			{#if !invoice.paidAt}
				<Button variant="outline" onclick={() => (markOpen = true)}>
					{i18nMsg('invoice-mark-paid-button', 'Marquer payée')}
				</Button>
			{:else}
				<Button variant="outline" onclick={() => (unmarkOpen = true)}>
					{i18nMsg('invoice-unmark-paid-button', 'Dé-marquer payée')}
				</Button>
			{/if}
			<Button
				onclick={downloadPdf}
				disabled={pdfDownloading}
				aria-label={i18nMsg(
					'invoices-download-pdf-aria-label',
					`Télécharger la facture ${invoice.invoiceNumber ?? ''} au format PDF`,
					{ number: invoice.invoiceNumber ?? '' },
				)}
			>
				<Download class="h-4 w-4" aria-hidden="true" />
				{i18nMsg('invoices-download-pdf', 'Télécharger PDF')}
			</Button>
			{#if invoice.journalEntryId}
				<Button
					variant="outline"
					onclick={() => goto(`/journal-entries/${invoice!.journalEntryId}`)}
				>
					<BookOpen class="h-4 w-4" aria-hidden="true" />
					Voir l'écriture comptable
				</Button>
			{/if}
		</div>
	{/if}
</div>

{#if loading}
	<p class="text-sm text-text-muted">Chargement…</p>
{:else if errorMsg}
	<p class="text-sm text-destructive">{errorMsg}</p>
{:else if invoice}
	<div class="space-y-6">
		<div>
			<h1 class="text-2xl font-semibold">Facture</h1>
			<p class="text-sm text-text-muted">
				{invoice.invoiceNumber ?? 'Brouillon'} — {statusLabel(invoice.status)}
				{#if invoice.status === 'validated'}
					<span class="ml-2">
						<PaymentStatusBadge status={paymentStatus(invoice)} />
					</span>
				{/if}
			</p>
		</div>

		<div class="grid grid-cols-2 gap-4 text-sm">
			<div>
				<div class="text-text-muted">Date</div>
				<div>{invoice.date}</div>
			</div>
			<div>
				<div class="text-text-muted">Échéance</div>
				<div>{invoice.dueDate ?? '—'}</div>
			</div>
			<div>
				<div class="text-text-muted">Conditions de paiement</div>
				<div>{invoice.paymentTerms ?? '—'}</div>
			</div>
			{#if invoice.paidAt}
				<div>
					<div class="text-text-muted">{i18nMsg('invoice-detail-paid-at-label', 'Payée le')}</div>
					<div>{invoice.paidAt.slice(0, 10)}</div>
				</div>
			{/if}
		</div>

		<table class="w-full border-collapse text-sm">
			<thead>
				<tr class="border-b border-border text-left">
					<th class="py-2 pr-2">Description</th>
					<th class="py-2 pr-2">Quantité</th>
					<th class="py-2 pr-2">Prix unitaire</th>
					<th class="py-2 pr-2">TVA %</th>
					<th class="py-2 pr-2 text-right">Total</th>
				</tr>
			</thead>
			<tbody>
				{#each invoice.lines as l (l.id)}
					<tr class="border-b border-border">
						<td class="py-2 pr-2">{l.description}</td>
						<td class="py-2 pr-2">{l.quantity}</td>
						<td class="py-2 pr-2 font-mono">{formatInvoiceTotal(l.unitPrice)}</td>
						<td class="py-2 pr-2">{l.vatRate}%</td>
						<td class="py-2 pr-2 text-right font-mono">
							{formatInvoiceTotal(l.lineTotal)}
						</td>
					</tr>
				{/each}
			</tbody>
			<tfoot>
				<tr>
					<td colspan="4" class="py-3 text-right font-semibold">Total</td>
					<td class="py-3 text-right font-mono text-lg font-semibold">
						{formatInvoiceTotal(invoice.totalAmount)}
					</td>
				</tr>
			</tfoot>
		</table>
	</div>

	<Dialog.Root
		open={deleteOpen}
		onOpenChange={(o) => {
			deleteOpen = o;
			if (!o) deleteError = '';
		}}
	>
		<Dialog.Content>
			<Dialog.Header>
				<Dialog.Title>Supprimer la facture</Dialog.Title>
			</Dialog.Header>
			<p class="text-sm">Confirmer la suppression définitive de cette facture brouillon ?</p>
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

	<Dialog.Root
		open={validateOpen}
		onOpenChange={(o) => {
			validateOpen = o;
			if (!o) validateError = '';
		}}
	>
		<Dialog.Content>
			<Dialog.Header>
				<Dialog.Title>Valider la facture</Dialog.Title>
			</Dialog.Header>
			<p class="text-sm">
				Une fois validée, cette facture sera immuable, recevra un numéro définitif et
				générera une écriture comptable. Continuer&nbsp;?
			</p>
			{#if validateError}
				<div class="rounded-md border border-destructive bg-destructive/10 px-3 py-2 text-sm text-destructive">
					{validateError}
				</div>
			{/if}
			<Dialog.Footer>
				<Button variant="outline" onclick={() => (validateOpen = false)}>Annuler</Button>
				<Button onclick={confirmValidate} disabled={validateSubmitting}>Valider</Button>
			</Dialog.Footer>
		</Dialog.Content>
	</Dialog.Root>

	<MarkPaidDialog
		open={markOpen}
		onOpenChange={(o) => {
			markOpen = o;
			if (!o) markError = '';
		}}
		invoiceDate={invoice.date}
		submitting={markSubmitting}
		errorMsg={markError}
		onConfirm={handleMarkConfirm}
	/>

	<Dialog.Root
		open={unmarkOpen}
		onOpenChange={(o) => {
			unmarkOpen = o;
			if (!o) unmarkError = '';
		}}
	>
		<Dialog.Content>
			<Dialog.Header>
				<Dialog.Title>
					{i18nMsg('invoice-unmark-paid-dialog-title', 'Dé-marquer payée')}
				</Dialog.Title>
			</Dialog.Header>
			<p class="text-sm">
				{i18nMsg(
					'invoice-unmark-paid-dialog-body',
					'Cette facture sera à nouveau considérée comme impayée. Utile pour corriger une erreur. Continuer ?',
				)}
			</p>
			{#if unmarkError}
				<div class="rounded-md border border-destructive bg-destructive/10 px-3 py-2 text-sm text-destructive">
					{unmarkError}
				</div>
			{/if}
			<Dialog.Footer>
				<Button variant="outline" onclick={() => (unmarkOpen = false)} disabled={unmarkSubmitting}>
					{i18nMsg('common-cancel', 'Annuler')}
				</Button>
				<Button variant="destructive" onclick={confirmUnmark} disabled={unmarkSubmitting}>
					{i18nMsg('invoice-unmark-paid-confirm', 'Dé-marquer')}
				</Button>
			</Dialog.Footer>
		</Dialog.Content>
	</Dialog.Root>
{/if}
