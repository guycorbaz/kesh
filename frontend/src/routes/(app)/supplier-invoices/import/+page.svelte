<script lang="ts">
	import { onMount } from 'svelte';
	import Big from 'big.js';
	import { isApiError } from '$lib/shared/utils/api-client';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { notifySuccess, notifyError, notifyWarning } from '$lib/shared/utils/notify';
	/**
	 * « Documents à traiter » — Story 12.5d, refondue par la Story 30-2a (#685).
	 *
	 * Tout PDF ou image déposé dans le dossier d'import y entre, avec ou sans QR-code.
	 * Chaque ligne porte un badge (« QR-facture » avec créancier, montant et référence,
	 * ou « Sans QR-code »), un **aperçu** (`DocumentPreview`), « Créer une facture » et
	 * « Retirer de la liste » (l'ancien *Écarter* : la ligne passe `discarded`, le
	 * fichier reste conservé ; un nouveau dépôt la réactive).
	 * L'URL reste `/supplier-invoices/import` (liens de la préparation de clôture, K5).
	 *
	 * Story 30-2b (AC10, C-30-2-11) : le formulaire de complétion en ligne a disparu.
	 * « Créer une facture » mène à `/supplier-invoices/new?document=<id>`, où la facture
	 * se saisit à côté du document (`SupplierInvoiceForm`, une seule définition).
	 */
	import {
		triggerInboxImport,
		listImported,
		discardImport,
		downloadImportedSourceDocument,
		fetchImportedSourceDocument,
	} from '$lib/features/imported-supplier-invoices/imported-supplier-invoices.api';
	import {
		hasQr,
		importDateOf,
		type ImportedSupplierInvoice,
		type InboxImportReport,
	} from '$lib/features/imported-supplier-invoices/imported-supplier-invoices.types';
	import { importErrorLabel } from '$lib/features/imported-supplier-invoices/error-label';
	import DocumentPreview from '$lib/features/imported-supplier-invoices/DocumentPreview.svelte';
	import { formatSwissAmount } from '$lib/features/journal-entries/balance';

	let toComplete = $state<ImportedSupplierInvoice[]>([]);
	let loading = $state(true);
	let errorMsg = $state('');

	let report = $state<InboxImportReport | null>(null);
	let importing = $state(false);

	// Aperçu : un document à la fois, sous sa ligne.
	let previewId = $state<number | null>(null);
	// Une fonction de lecture STABLE par document : une closure recréée à chaque rendu
	// relancerait la lecture (et révoquerait l'URL affichée) à chaque mise à jour.
	const previewLoaders = new Map<number, () => Promise<Blob>>();
	function previewLoader(id: number): () => Promise<Blob> {
		let loader = previewLoaders.get(id);
		if (!loader) {
			loader = () => fetchImportedSourceDocument(id);
			previewLoaders.set(id, loader);
		}
		return loader;
	}
	function togglePreview(id: number) {
		previewId = previewId === id ? null : id;
	}

	async function reloadList() {
		toComplete = await listImported('to_complete');
	}

	/** Rechargement « best-effort » : ne propage jamais (évite une rejection non
	 * gérée quand appelé depuis un `catch`, BH7/EC6). Notifie sur échec. */
	async function safeReloadList() {
		try {
			await reloadList();
		} catch {
			notifyWarning(
				i18nMsg(
					'imported-supplier-invoices-reload-failed',
					'La liste n’a pas pu être rechargée — actualisez la page.',
				),
			);
		}
	}

	onMount(async () => {
		try {
			await reloadList();
		} catch (err) {
			if (isApiError(err)) errorMsg = err.message;
		} finally {
			loading = false;
		}
	});

	async function runImport() {
		if (importing) return;
		importing = true;
		try {
			report = await triggerInboxImport();
		} catch (err) {
			report = null;
			if (isApiError(err) && err.code === 'INBOX_IMPORT_ALREADY_RUNNING') {
				notifyError(
					i18nMsg(
						'imported-supplier-invoices-import-running',
						'Un import est déjà en cours. Réessayez dans quelques instants.',
					),
				);
			} else {
				notifyError(
					(isApiError(err) && err.message) ||
						i18nMsg('imported-supplier-invoices-import-failed', 'Erreur inattendue lors de l’import.'),
				);
			}
			importing = false;
			return;
		}
		// Rechargement de la liste séparé du déclenchement (BH1/EC2) : un échec de
		// `reloadList` ne doit PAS effacer le rapport d'import qui vient de réussir.
		try {
			await reloadList();
		} catch {
			notifyWarning(
				i18nMsg(
					'imported-supplier-invoices-completed-reload-failed',
					'Import effectué, mais la liste n’a pas pu être rechargée — actualisez la page.',
				),
			);
		} finally {
			importing = false;
		}
	}

	/** « Retirer de la liste » (C-30-2-5) : l'ancien *Écarter*, fichier conservé. */
	async function discard(row: ImportedSupplierInvoice) {
		if (
			!confirm(
				i18nMsg(
					'imported-supplier-invoices-discard-confirm',
					'Retirer ce document de la liste ? Le fichier reste conservé.',
				),
			)
		) {
			return;
		}
		try {
			await discardImport(row.id);
			toComplete = toComplete.filter((r) => r.id !== row.id);
			if (previewId === row.id) previewId = null;
			notifySuccess(i18nMsg('imported-supplier-invoices-discarded', 'Document retiré de la liste.'));
		} catch (err) {
			if (isApiError(err) && err.code === 'IMPORT_NOT_PENDING_COMPLETION') {
				notifyWarning(
					i18nMsg(
						'imported-supplier-invoices-discard-conflict',
						'Ce document a déjà été traité ou retiré de la liste par une autre session.',
					),
				);
				await safeReloadList();
			} else if (isApiError(err) && err.code === 'IMPORTED_INVOICE_NOT_FOUND') {
				notifyWarning(i18nMsg('imported-supplier-invoices-err-not-found', 'Document introuvable.'));
				await safeReloadList();
			} else {
				notifyError(
					(isApiError(err) && err.message) ||
						i18nMsg('imported-supplier-invoices-discard-failed', 'Impossible de retirer le document de la liste.'),
				);
			}
		}
	}

	async function downloadJustif(row: ImportedSupplierInvoice) {
		try {
			await downloadImportedSourceDocument(row.id);
		} catch (err) {
			if (isApiError(err) && err.code === 'SOURCE_DOCUMENT_GONE') {
				notifyWarning(
					i18nMsg('imported-supplier-invoices-doc-gone', 'Le document n’a pas été restauré.'),
				);
			} else if (isApiError(err) && err.code === 'IMPORTED_INVOICE_NOT_FOUND') {
				notifyWarning(i18nMsg('imported-supplier-invoices-err-not-found', 'Document introuvable.'));
			} else {
				notifyError(
					(isApiError(err) && err.message) ||
						i18nMsg('imported-supplier-invoices-doc-failed', 'Téléchargement impossible.'),
				);
			}
		}
	}

	function fmt(amount: string | null): string {
		if (!amount) return '—';
		try {
			return formatSwissAmount(new Big(amount));
		} catch {
			return amount;
		}
	}
</script>

<svelte:head>
	<title>{i18nMsg('imported-supplier-invoices-title', 'Documents à traiter')} — Kesh</title>
</svelte:head>

<div class="mb-6 flex items-center justify-between">
	<h1 class="text-2xl font-semibold">
		{i18nMsg('imported-supplier-invoices-title', 'Documents à traiter')}
	</h1>
	<button
		class="rounded bg-primary px-4 py-2 text-sm text-primary-foreground disabled:opacity-60"
		data-testid="inbox-import-trigger"
		onclick={runImport}
		disabled={importing}
	>
		{importing
			? i18nMsg('imported-supplier-invoices-importing', 'Import en cours…')
			: i18nMsg('imported-supplier-invoices-import', 'Importer le dossier')}
	</button>
</div>

{#if report}
	<div class="mb-6 rounded border border-border p-4" data-testid="inbox-import-report">
		<p class="text-sm font-medium">
			{i18nMsg('imported-supplier-invoices-report-accepted', '{$n} document(s) importé(s).', {
				n: report.accepted.length,
			})}
		</p>
		{#if report.accepted.some((a) => a.qr !== 'present')}
			<ul class="mt-1 space-y-1 text-sm">
				{#each report.accepted.filter((a) => a.qr !== 'present') as a (a.importedSupplierInvoiceId)}
					<li data-testid="inbox-import-accepted-row">
						<span class="font-mono">{a.fileName}</span> —
						{#if a.qr === 'unreadable'}
							{i18nMsg('imported-supplier-invoices-report-qr-unreadable', 'QR illisible')}{#if a.qrError}
								({importErrorLabel(a.qrError)}){/if}.
							{i18nMsg(
								'imported-supplier-invoices-qr-unreadable-hint',
								'Document accepté sans les données du QR : en créant la facture, saisissez l’IBAN et la référence à la main.',
							)}
						{:else}
							{i18nMsg('imported-supplier-invoices-report-no-qr', 'sans QR-code')}
						{/if}
					</li>
				{/each}
			</ul>
		{/if}
		{#if report.failed.length > 0}
			<div class="mt-3">
				<p class="text-sm font-medium text-destructive">
					{i18nMsg('imported-supplier-invoices-report-failed', '{$n} échec(s) :', {
						n: report.failed.length,
					})}
				</p>
				<ul class="mt-1 space-y-1 text-sm">
					{#each report.failed as f, fi (fi)}
						<li data-testid="inbox-import-failed-row">
							<span class="font-mono">{f.fileName}</span> — {importErrorLabel(f.errorCode)}
						</li>
					{/each}
				</ul>
			</div>
		{/if}
		{#if report.warnings.length > 0}
			<ul class="mt-3 space-y-1 text-sm text-text-muted">
				{#each report.warnings as w, wi (wi)}
					<li>⚠️ {w}</li>
				{/each}
			</ul>
		{/if}
	</div>
{/if}

<h2 class="mb-3 text-lg font-medium">
	{i18nMsg('imported-supplier-invoices-to-complete', 'Documents à traiter')}
</h2>

{#if loading}
	<p class="text-sm text-text-muted">{i18nMsg('common-loading', 'Chargement…')}</p>
{:else if errorMsg}
	<p class="text-sm text-destructive">{errorMsg}</p>
{:else if toComplete.length === 0}
	<p class="text-sm text-text-muted" data-testid="imported-empty">
		{i18nMsg('imported-supplier-invoices-empty', 'Aucun document à traiter.')}
	</p>
{:else}
	<ul class="space-y-4" data-testid="imported-list">
		{#each toComplete as row (row.id)}
			<li class="rounded border border-border p-4" data-testid="imported-row">
				<div class="flex flex-wrap items-start justify-between gap-3">
					<div class="text-sm">
						<div class="font-medium" data-testid="imported-file-name">{row.originalFilename}</div>
						<div class="text-text-muted">
							{i18nMsg('imported-supplier-invoices-imported-on', 'Importé le {$date}', {
								date: importDateOf(row),
							})}
						</div>
						{#if hasQr(row)}
							<div class="mt-1" data-testid="imported-badge-qr">
								<span class="rounded bg-primary/10 px-2 py-0.5 text-xs font-medium text-primary"
									>{i18nMsg('imported-supplier-invoices-badge-qr', 'QR-facture')}</span
								>
								<span class="ml-1">{row.creditorName}</span>
								<span class="text-text-muted">
									· {i18nMsg('imported-supplier-invoices-amount', 'Montant')}: {fmt(row.amount)}
									{row.currency}
									{#if row.referenceValue}
										· {i18nMsg('imported-supplier-invoices-reference', 'Réf.')}: <span
											class="font-mono">{row.referenceValue}</span
										>
									{/if}
								</span>
							</div>
						{:else}
							<div class="mt-1" data-testid="imported-badge-no-qr">
								<span class="rounded bg-muted px-2 py-0.5 text-xs font-medium"
									>{i18nMsg('imported-supplier-invoices-badge-no-qr', 'Sans QR-code')}</span
								>
							</div>
						{/if}
					</div>
					<div class="flex flex-wrap gap-2">
						<button
							class="rounded border px-3 py-1 text-sm"
							data-testid="imported-preview"
							aria-expanded={previewId === row.id}
							onclick={() => togglePreview(row.id)}
							>{previewId === row.id
								? i18nMsg('imported-supplier-invoices-preview-close', 'Fermer l’aperçu')
								: i18nMsg('imported-supplier-invoices-preview', 'Aperçu')}</button
						>
						<a
							class="rounded bg-primary px-3 py-1 text-sm text-primary-foreground"
							data-testid="imported-create-invoice"
							href={`/supplier-invoices/new?document=${row.id}`}
							>{i18nMsg('imported-supplier-invoices-save', 'Créer une facture')}</a
						>
						<button
							class="rounded border px-3 py-1 text-sm disabled:opacity-60"
							data-testid="imported-discard"
							onclick={() => discard(row)}
							>{i18nMsg('imported-supplier-invoices-discard', 'Retirer de la liste')}</button
						>
						<button
							class="rounded border px-3 py-1 text-sm text-primary"
							data-testid="imported-download"
							onclick={() => downloadJustif(row)}
							>{i18nMsg('imported-supplier-invoices-view-doc', 'Télécharger')}</button
						>
					</div>
				</div>

				{#if previewId === row.id}
					<div class="mt-4" data-testid="imported-preview-panel">
						<DocumentPreview load={previewLoader(row.id)} fileName={row.originalFilename} />
					</div>
				{/if}
			</li>
		{/each}
	</ul>
{/if}
