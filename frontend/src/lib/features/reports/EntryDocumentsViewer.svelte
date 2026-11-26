<script lang="ts">
	/**
	 * Story 55-3b (D11, AC-2, AC-3, AC-5) — la visionneuse des justificatifs d'une écriture : la liste
	 * **en haut**, à partir de deux justificatifs seulement, et `DocumentPreview` dessous sur l'élément
	 * choisi (le premier par défaut) ; une facture pas encore émise est signalée « PDF pas encore émis »
	 * à la place de `DocumentPreview`, **sans rien lire** (D12).
	 *
	 * ⚠️ **Lecture unique** : le justificatif affiché est tenu par deux primitives (`kind`, `id`), et la
	 * fonction `load` n'est dérivée que d'elles. Une liste remplacée par des objets égaux (rapport rechargé,
	 * 55-1a AC-24) ne change ni l'une ni l'autre : `DocumentPreview` ne relit pas le document.
	 */
	import DocumentPreview from '$lib/features/imported-supplier-invoices/DocumentPreview.svelte';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import {
		entryDocumentErrorMessage,
		entryDocumentFileName,
		entryDocumentItemLabel,
		entryDocumentLoader,
		isNotIssuedInvoice,
	} from './entry-documents';
	import type { EntryDocument, EntryDocumentKind } from './reports.types';

	interface Props {
		/** Les justificatifs de l'écriture, lus à l'ouverture (au moins un). */
		documents: readonly EntryDocument[];
	}
	let { documents }: Props = $props();

	// Le choix de l'utilisateur, par primitives ; `null` = le premier de la liste.
	let chosenKind = $state<EntryDocumentKind | null>(null);
	let chosenId = $state<number | null>(null);

	const current = $derived(
		documents.find((d) => d.kind === chosenKind && d.id === chosenId) ?? documents[0],
	);
	// Primitives dérivées : leur valeur seule compte (égalité stricte), un objet égal ne les change pas.
	const kind = $derived(current.kind);
	const id = $derived(current.id);
	const pdfStatus = $derived(current.pdfStatus);
	const fileName = $derived(entryDocumentFileName(current));
	const notIssued = $derived(isNotIssuedInvoice(kind, pdfStatus));
	const load = $derived(entryDocumentLoader(kind, id));
	const errorMessage = (err: unknown) => entryDocumentErrorMessage(kind, err);

	function choose(doc: EntryDocument) {
		chosenKind = doc.kind;
		chosenId = doc.id;
	}
</script>

<div class="space-y-3" data-testid="entry-documents-viewer">
	{#if documents.length >= 2}
		<ul
			class="max-h-24 space-y-1 overflow-y-auto"
			aria-label={i18nMsg('reports-nav-documents-list', "Justificatifs de l'écriture")}
			data-testid="entry-documents-list"
		>
			{#each documents as doc (doc.kind + '-' + doc.id)}
				{@const selected = doc.kind === kind && doc.id === id}
				<li>
					<button
						type="button"
						class="w-full rounded px-2 py-1 text-left text-sm hover:bg-gray-50"
						class:font-semibold={selected}
						class:bg-gray-100={selected}
						aria-current={selected ? 'true' : undefined}
						data-testid="entry-documents-item-{doc.kind}-{doc.id}"
						onclick={() => choose(doc)}>{entryDocumentItemLabel(doc)}</button
					>
				</li>
			{/each}
		</ul>
	{/if}

	{#if notIssued}
		<p class="rounded border border-border bg-surface p-3 text-sm" role="status" data-testid="entry-documents-not-issued">
			{i18nMsg(
				'reports-nav-documents-not-issued',
				"PDF pas encore émis : cette facture n'a pas encore de PDF figé ; son prochain téléchargement, ou son prochain envoi par e-mail, le figera.",
			)}
		</p>
	{:else}
		<DocumentPreview {load} {fileName} {errorMessage} />
	{/if}
</div>
