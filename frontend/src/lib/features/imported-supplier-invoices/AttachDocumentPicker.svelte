<script lang="ts">
	/**
	 * Sélecteur « Joindre un document » de la fiche d'une facture fournisseur — Story
	 * 30-2c (AC20, C-30-2-12).
	 *
	 * Liste les documents à traiter (nom, date d'import, badge QR) ; pour un document
	 * avec QR, son **montant** s'affiche à côté du **total de la facture** — aucun refus
	 * sur un écart : l'humain voit les deux et décide (C-30-2-12). « Aperçu » ouvre le
	 * document sous sa ligne (`DocumentPreview`, AC6) ; « Joindre » demande confirmation
	 * puis appelle `onAttach`. Un refus du serveur s'affiche dans le panneau, sans le
	 * fermer ; un refus `404`/`409` (document traité, retiré ou joint ailleurs entre-temps)
	 * fait **relire la liste**, pour ne plus proposer ce qui vient d'échouer (revue P1,
	 * B2 = E5).
	 *
	 * Vit dans `imported-supplier-invoices/` pour partager les libellés de la liste
	 * « Documents à traiter » (badge, aperçu) avec elle.
	 */
	import { onMount } from 'svelte';
	import { isApiError } from '$lib/shared/utils/api-client';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { formatSupplierInvoiceTotal } from '$lib/features/supplier-invoices/supplier-invoice-helpers';
	import { formatExactAmount } from '$lib/features/invoices/invoice-helpers';
	import DocumentPreview from './DocumentPreview.svelte';
	import { fetchImportedSourceDocument, listImported } from './imported-supplier-invoices.api';
	import { hasQr, importDateOf, type ImportedSupplierInvoice } from './imported-supplier-invoices.types';
	import { isStaleDocumentRefusal } from './stale-refusal';

	interface Props {
		/** Total TTC de la facture (décimal sérialisé), affiché à côté du montant du QR. */
		invoiceTotal: string;
		/** Joint le document choisi ; une erreur levée s'affiche dans le panneau. */
		onAttach: (importedSupplierInvoiceId: number) => Promise<void>;
		/** Ferme le panneau sans rien joindre. */
		onClose: () => void;
	}

	let { invoiceTotal, onAttach, onClose }: Props = $props();

	let rows = $state<ImportedSupplierInvoice[]>([]);
	let loading = $state(true);
	let loadError = $state('');
	let previewId = $state<number | null>(null);
	let attaching = $state(false);
	let attachError = $state('');

	// Une fonction de lecture STABLE par document (patron de la liste) : une closure
	// recréée à chaque rendu relancerait la lecture de l'aperçu.
	const previewLoaders = new Map<number, () => Promise<Blob>>();
	function previewLoader(id: number): () => Promise<Blob> {
		let loader = previewLoaders.get(id);
		if (!loader) {
			loader = () => fetchImportedSourceDocument(id);
			previewLoaders.set(id, loader);
		}
		return loader;
	}

	/** Lit (ou relit) les documents à traiter. */
	async function loadRows() {
		try {
			rows = await listImported('to_complete');
			loadError = '';
		} catch (err) {
			loadError = isApiError(err) ? err.message : i18nMsg('common-error', 'Erreur inattendue');
		} finally {
			loading = false;
		}
	}

	onMount(loadRows);

	async function attach(row: ImportedSupplierInvoice) {
		// Garde du double clic : sous jsdom, un second clic est parti malgré
		// `disabled={attaching}` (revue P1) ; la garde ne dépend pas du rendu.
		if (attaching) return;
		attachError = '';
		const ok = confirm(
			i18nMsg(
				'imported-supplier-invoices-attach-confirm',
				'Joindre le document « { $file } » à cette facture ?',
				{ file: row.originalFilename },
			),
		);
		if (!ok) return;
		attaching = true;
		try {
			await onAttach(row.id);
		} catch (err) {
			attachError = isApiError(err) ? err.message : i18nMsg('common-error', 'Erreur inattendue');
			if (isStaleDocumentRefusal(err)) await loadRows();
		} finally {
			attaching = false;
		}
	}
</script>

<div class="mb-6 rounded border border-border p-4" data-testid="attach-document-picker">
	<div class="mb-3 flex items-center justify-between">
		<h2 class="font-medium">
			{i18nMsg('imported-supplier-invoices-attach-title', 'Joindre un document à traiter')}
		</h2>
		<button class="text-sm text-primary" data-testid="attach-document-close" onclick={onClose}>
			{i18nMsg('common-cancel', 'Annuler')}
		</button>
	</div>

	{#if attachError}
		<p class="mb-3 text-sm text-destructive" role="alert" data-testid="attach-document-error">
			{attachError}
		</p>
	{/if}

	{#if loading}
		<p class="text-sm text-text-muted">{i18nMsg('common-loading', 'Chargement…')}</p>
	{:else if loadError}
		<p class="text-sm text-destructive">{loadError}</p>
	{:else if rows.length === 0}
		<p class="text-sm text-text-muted" data-testid="attach-document-empty">
			{i18nMsg(
				'imported-supplier-invoices-attach-empty',
				'Aucun document à traiter — déposez-le dans le dossier d’import puis importez-le.',
			)}
		</p>
	{:else}
		<ul class="space-y-3">
			{#each rows as row (row.id)}
				<li class="rounded border border-border p-3" data-testid="attach-document-row">
					<div class="flex flex-wrap items-start justify-between gap-3">
						<div class="text-sm">
							<div class="font-medium" data-testid="attach-document-name">
								{row.originalFilename}
							</div>
							<div class="text-text-muted">
								{i18nMsg('imported-supplier-invoices-imported-on', 'Importé le {$date}', {
									date: importDateOf(row),
								})}
							</div>
							{#if hasQr(row)}
								<div class="mt-1">
									<span class="rounded bg-primary/10 px-2 py-0.5 text-xs font-medium text-primary"
										>{i18nMsg('imported-supplier-invoices-badge-qr', 'QR-facture')}</span
									>
									<span class="ml-1">{row.creditorName}</span>
								</div>
							{:else}
								<div class="mt-1">
									<span class="rounded bg-muted px-2 py-0.5 text-xs font-medium"
										>{i18nMsg('imported-supplier-invoices-badge-no-qr', 'Sans QR-code')}</span
									>
								</div>
							{/if}
							<div class="mt-1" data-testid="attach-document-amounts">
								{#if hasQr(row) && row.amount}
									<span data-testid="attach-document-qr-amount"
										>{i18nMsg('imported-supplier-invoices-attach-qr-amount', 'Montant du QR : { $amount }', {
											amount: `${formatSupplierInvoiceTotal(row.amount)} ${row.currency ?? ''}`.trim(),
										})}</span
									>
									·
								{/if}
								<span data-testid="attach-document-invoice-total"
									>{i18nMsg(
										'imported-supplier-invoices-attach-invoice-total',
										'Total de la facture : { $total }',
										// Story v017-3b (#687, C-v017-3-27) — le total de la facture
										// affichée, exact : une facture existante sous le centime s'y
										// lit comme sur sa fiche. Le montant du QR, au centime par le
										// contrôle d'import, garde son format.
										{ total: formatExactAmount(invoiceTotal) },
									)}</span
								>
							</div>
						</div>
						<div class="flex flex-wrap gap-2">
							<button
								class="rounded border px-3 py-1 text-sm"
								data-testid="attach-document-preview"
								aria-expanded={previewId === row.id}
								onclick={() => (previewId = previewId === row.id ? null : row.id)}
								>{previewId === row.id
									? i18nMsg('imported-supplier-invoices-preview-close', 'Fermer l’aperçu')
									: i18nMsg('imported-supplier-invoices-preview', 'Aperçu')}</button
							>
							<button
								class="rounded bg-primary px-3 py-1 text-sm text-primary-foreground disabled:opacity-60"
								data-testid="attach-document-submit"
								disabled={attaching}
								onclick={() => attach(row)}
								>{i18nMsg('imported-supplier-invoices-attach-submit', 'Joindre')}</button
							>
						</div>
					</div>
					{#if previewId === row.id}
						<div class="mt-3" data-testid="attach-document-preview-panel">
							<DocumentPreview load={previewLoader(row.id)} fileName={row.originalFilename} />
						</div>
					{/if}
				</li>
			{/each}
		</ul>
	{/if}
</div>
