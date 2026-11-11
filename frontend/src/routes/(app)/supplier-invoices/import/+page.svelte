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
	 * ou « Aucune QR-facture exploitable », Story v017-5), un **aperçu** (`DocumentPreview`), « Créer une facture » et
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
		duplicateOf,
		hasQr,
		importDateOf,
		movedToOf,
		reportCounts,
		reportRows,
		type ExistingDocument,
		type ImportedSupplierInvoice,
		type InboxImportReport,
		type ReportRow,
	} from '$lib/features/imported-supplier-invoices/imported-supplier-invoices.types';
	import { importErrorLabel } from '$lib/features/imported-supplier-invoices/error-label';
	import DocumentPreview from '$lib/features/imported-supplier-invoices/DocumentPreview.svelte';
	import { formatSwissAmount } from '$lib/features/journal-entries/balance';

	let toComplete = $state<ImportedSupplierInvoice[]>([]);
	let loading = $state(true);
	let errorMsg = $state('');

	let report = $state<InboxImportReport | null>(null);
	let importing = $state(false);

	// Story 34-9 (#969, #974, AC-4) — une ligne par fichier, dans l'ordre des noms. Le
	// rapport n'est pas conservé : il vit le temps de la page (décision de Guy).
	const rows = $derived(report ? reportRows(report) : []);
	// Revue de code P1, B-3 : le cadre suit la notification — la limite atteinte compte aussi.
	const reportHasProblem = $derived(
		!!report &&
			(report.failed.length > 0 ||
				(report.skipped ?? []).length > 0 ||
				report.truncatedAt != null),
	);

	/**
	 * Story 34-9 (AC-4, F9-1, C-e342-21) — le résumé, composé ici segment par segment : le
	 * client reçoit les messages pré-résolus, un sélecteur Fluent y arriverait figé. Les
	 * segments nuls sont omis ; un import vide garde « 0 document(s) importé(s). ».
	 */
	function summaryOf(r: InboxImportReport): string {
		const c = reportCounts(r);
		const segments: string[] = [];
		if (c.imported > 0 || c.duplicates + c.refused + c.notProcessed === 0) {
			segments.push(
				i18nMsg('imported-supplier-invoices-report-summary-imported', '{$n} document(s) importé(s)', {
					n: c.imported,
				}),
			);
		}
		if (c.duplicates > 0) {
			segments.push(
				i18nMsg('imported-supplier-invoices-report-summary-duplicates', '{$n} déjà importé(s)', {
					n: c.duplicates,
				}),
			);
		}
		if (c.refused > 0) {
			segments.push(
				i18nMsg('imported-supplier-invoices-report-summary-refused', '{$n} refusé(s)', {
					n: c.refused,
				}),
			);
		}
		if (c.notProcessed > 0) {
			segments.push(
				i18nMsg('imported-supplier-invoices-report-summary-not-processed', '{$n} non traité(s)', {
					n: c.notProcessed,
				}),
			);
		}
		return `${segments.join(', ')}.`;
	}

	/**
	 * Story 34-9 (AC-5, C-e342-13) — la notification qui suit un import : un avertissement
	 * dès qu'un fichier n'est pas importé (`failed` ou `skipped`) ou que la limite est
	 * atteinte ; un succès sinon ; rien pour un import sans aucun fichier. Un document
	 * importé sans les données du QR ne déclenche pas l'avertissement : sa ligne le dit.
	 */
	function noticeOf(r: InboxImportReport): { warning: boolean; message: string } | null {
		const n = r.failed.length + (r.skipped ?? []).length;
		// Revue de code P1, E-2 : des fichiers non importés ET la limite atteinte — le
		// toast dit les deux.
		if (n > 0 && r.truncatedAt != null) {
			return {
				warning: true,
				message: i18nMsg(
					'imported-supplier-invoices-notice-not-imported-truncated',
					'Import terminé : {$n} fichier(s) non importé(s), et la limite par import est atteinte. Voir le rapport.',
					{ n },
				),
			};
		}
		if (n > 0) {
			return {
				warning: true,
				message: i18nMsg(
					'imported-supplier-invoices-notice-not-imported',
					'Import terminé : {$n} fichier(s) non importé(s). Voir le rapport.',
					{ n },
				),
			};
		}
		if (r.truncatedAt != null) {
			return {
				warning: true,
				message: i18nMsg(
					'imported-supplier-invoices-notice-truncated',
					'Import terminé : la limite par import est atteinte. Voir le rapport.',
				),
			};
		}
		if (r.accepted.length > 0) {
			return {
				warning: false,
				message: i18nMsg(
					'imported-supplier-invoices-notice-imported',
					'Import terminé : {$n} document(s) importé(s).',
					{ n: r.accepted.length },
				),
			};
		}
		return null;
	}

	/** Le document existant d'une ligne « déjà importé », s'il est connu. */
	function existingOf(row: ReportRow): ExistingDocument | null {
		return row.state === 'duplicate' ? duplicateOf(row.failed) : null;
	}

	/** Le nom rangé dans `failed/` d'une ligne « déjà importé » ou « refusé ». */
	function movedOf(row: ReportRow): string | null {
		return row.state === 'duplicate' || row.state === 'refused' ? movedToOf(row.failed) : null;
	}

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
		let fresh: InboxImportReport;
		try {
			fresh = await triggerInboxImport();
			report = fresh;
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
		const notice = noticeOf(fresh);
		// Rechargement de la liste séparé du déclenchement (BH1/EC2) : un échec de
		// `reloadList` ne doit PAS effacer le rapport d'import qui vient de réussir.
		let reloadFailed = false;
		try {
			await reloadList();
		} catch {
			reloadFailed = true;
		} finally {
			importing = false;
		}
		// Story 34-9 (AC-5, F9-8) — une seule notification : si le rechargement échoue,
		// un avertissement qui dit les deux.
		if (reloadFailed && notice) {
			notifyWarning(
				notice.message,
				i18nMsg(
					'imported-supplier-invoices-reload-failed',
					'La liste n’a pas pu être rechargée — actualisez la page.',
				),
			);
		} else if (reloadFailed) {
			notifyWarning(
				i18nMsg(
					'imported-supplier-invoices-completed-reload-failed',
					'Import effectué, mais la liste n’a pas pu être rechargée — actualisez la page.',
				),
			);
		} else if (notice?.warning) {
			notifyWarning(notice.message);
		} else if (notice) {
			notifySuccess(notice.message);
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
	<!-- Story 34-9 (#969, #974, AC-4, AC-5) — un résumé, puis une ligne par fichier dans
	     l'ordre des noms. Les anciens testids (`inbox-import-accepted-row`,
	     `inbox-import-failed-row`) restent, portés par le contenu de la ligne. -->
	<div
		class="mb-6 rounded border p-4 {reportHasProblem ? 'border-2 border-warning' : 'border-border'}"
		data-testid="inbox-import-report"
	>
		<p class="text-sm font-medium" data-testid="inbox-import-summary">{summaryOf(report)}</p>
		{#if report.truncatedAt != null}
			<p class="mt-1 text-sm font-medium" data-testid="inbox-import-truncated">
				{i18nMsg(
					'imported-supplier-invoices-report-truncated',
					'La limite de {$n} fichiers par import est atteinte : relancez l’import pour traiter le reste.',
					{ n: report.truncatedAt },
				)}
			</p>
		{/if}
		{#if rows.length > 0}
			<ul class="mt-2 space-y-1 text-sm">
				{#each rows as row, ri (ri)}
					{@const existing = existingOf(row)}
					{@const moved = movedOf(row)}
					<li data-testid="inbox-import-file-row" data-state={row.state}>
						{#if row.state === 'imported' || row.state === 'imported-no-qr'}
							<div data-testid="inbox-import-accepted-row">
								<span class="font-mono">{row.fileName}</span> —
								{#if row.state === 'imported'}
									{i18nMsg('imported-supplier-invoices-report-state-imported', 'Importé.')}
								{:else}
									{i18nMsg(
										'imported-supplier-invoices-report-state-imported-no-qr',
										'Importé sans les données du QR :',
									)}
									{#if row.accepted.qr === 'unreadable'}
										{i18nMsg('imported-supplier-invoices-report-qr-unreadable', 'QR illisible')}{#if row.accepted.qrError}
											({importErrorLabel(row.accepted.qrError)}){/if}.
										{i18nMsg(
											'imported-supplier-invoices-qr-unreadable-hint',
											'Document accepté sans les données du QR : en créant la facture, saisissez l’IBAN et la référence à la main.',
										)}
									{:else}
										{i18nMsg('imported-supplier-invoices-report-no-qr', 'aucune QR-facture exploitable')}
									{/if}
								{/if}
							</div>
						{:else if row.state === 'duplicate' || row.state === 'refused'}
							<div data-testid="inbox-import-failed-row">
								<span class="font-mono">{row.fileName}</span> —
								{#if row.state === 'refused'}
									<span class="font-medium text-destructive"
										>{i18nMsg('imported-supplier-invoices-report-refused', 'Refusé :')}</span
									>
									{importErrorLabel(row.failed.errorCode)}.
								{:else if existing}
									<span class="font-medium"
										>{i18nMsg('imported-supplier-invoices-report-duplicate', 'Déjà importé :')}</span
									>
									{#if existing.existingStatus === 'to_complete' && !toComplete.some((r) => r.id === existing.existingDocumentId)}
										<!-- Revue de code P1, E-1 ≈ B-2 : la ligne visée n'est pas dans la liste
										     affichée (liste non rechargée, ou document retiré ou transformé
										     depuis) — aucun lien mort, la ligne le dit. -->
										{i18nMsg(
											'imported-supplier-invoices-report-duplicate-not-in-list',
											'absent de la liste ci-dessous : actualisez la page',
										)}.
									{:else if existing.existingStatus === 'to_complete'}
										<a
											class="text-primary underline"
											data-testid="inbox-import-duplicate-link"
											href={`#imported-${existing.existingDocumentId}`}
											>{i18nMsg(
												'imported-supplier-invoices-report-duplicate-pending',
												'en attente dans Documents à traiter',
											)}</a
										>.
									{:else if existing.existingStatus === 'completed' && existing.supplierInvoiceId != null}
										<a
											class="text-primary underline"
											data-testid="inbox-import-duplicate-link"
											href={`/supplier-invoices/${existing.supplierInvoiceId}`}
											>{i18nMsg(
												'imported-supplier-invoices-report-duplicate-invoice',
												'transformé en facture',
											)}</a
										>.
									{:else if existing.existingStatus === 'discarded'}
										{i18nMsg('imported-supplier-invoices-report-duplicate-discarded', 'retiré de la liste')}.
									{:else}
										{i18nMsg('imported-supplier-invoices-report-duplicate-processed', 'déjà traité')}.
									{/if}
								{:else}
									<span class="font-medium"
										>{i18nMsg('imported-supplier-invoices-report-duplicate-unknown', 'Déjà importé.')}</span
									>
								{/if}
								{#if moved}
									<span class="text-text-muted">
										{i18nMsg(
											'imported-supplier-invoices-report-moved-to',
											'Le fichier a été déplacé dans failed/ sous le nom {$movedTo} ; supprimez-le à la main.',
											{ movedTo: moved },
										)}
									</span>
								{/if}
							</div>
						{:else if row.state === 'not-processed'}
							<span class="font-mono">{row.fileName}</span> —
							<span>
								<span class="font-medium"
									>{i18nMsg('imported-supplier-invoices-report-state-not-processed', 'Non traité :')}</span
								>
								{row.skipped.reason === 'UNSTABLE'
									? i18nMsg(
											'imported-supplier-invoices-report-skipped-unstable',
											'en cours d’écriture, laissé dans le dossier : relancez l’import',
										)
									: i18nMsg(
											'imported-supplier-invoices-report-skipped-vanished',
											'disparu pendant l’import',
										)}.
							</span>
						{/if}
					</li>
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
			<!-- Story 34-9 (AC-4, C-e342-14) — l'ancre que vise le lien d'un doublon en attente. -->
			<li
				id={`imported-${row.id}`}
				class="scroll-mt-4 rounded border border-border p-4 target:ring-2 target:ring-primary"
				data-testid="imported-row"
			>
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
									>{i18nMsg(
										'imported-supplier-invoices-badge-no-qr',
										'Aucune QR-facture exploitable',
									)}</span
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
