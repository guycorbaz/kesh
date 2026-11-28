<!--
  Story 55-3b (D11, AC-1, AC-2, AC-7) — les justificatifs d'une écriture, depuis le Journal et le Grand livre.

  **Une instance par vue** : le dialogue est rendu une fois ; chaque ligne qui porte `hasDocuments` montre
  l'icône (`entryDocumentsButton`, snippet exporté ci-dessous), qui ouvre cette instance par
  `openFor(entryId, titre, élément)`. Au clic seulement, la liste est lue
  (`GET /journal-entries/{id}/documents`) ; elle est **relue à chaque ouverture**, et la réponse d'une
  ouverture antérieure (fermé puis rouvert en vol) est ignorée — numéro d'ouverture, sans annulation réseau.

  Le dialogue ne touche pas l'URL (aucun `goto`, 55-1a D2) ; il se ferme par « Fermer », un clic à côté ou
  Échap (comportement par défaut du dialogue de Kesh, bits-ui), et rend le focus à l'icône qui l'a ouvert.
-->
<script lang="ts" module>
	import FileText from '@lucide/svelte/icons/file-text';

	/** Les propriétés de l'icône d'une ligne. */
	export interface EntryDocumentsButtonProps {
		/** Libellé accessible : « Justificatifs de l'écriture n° N ». */
		label: string;
		testId: string;
		/** Ouvre le dialogue ; reçoit le bouton, qui reprendra le focus à la fermeture. */
		onOpen: (trigger: HTMLElement) => void;
	}

	export { entryDocumentsButton };
</script>

<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import * as Dialog from '$lib/components/ui/dialog';
	import { errorMessageOf } from '$lib/shared/utils/error-message';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import EntryDocumentsViewer from './EntryDocumentsViewer.svelte';
	import { fetchEntryDocuments } from './reports.api';
	import type { EntryDocument } from './reports.types';

	type Phase =
		| { kind: 'loading' }
		| { kind: 'ready'; documents: EntryDocument[] }
		| { kind: 'error'; message: string };

	let open = $state(false);
	/** Le titre du dialogue (l'écriture ouverte), copié au clic : un rapport rechargé ne le change pas. */
	let title = $state('');
	let phase = $state<Phase>({ kind: 'loading' });
	/** L'icône qui a ouvert le dialogue : elle reprend le focus à la fermeture. */
	let trigger: HTMLElement | null = null;
	/** Numéro d'ouverture : une liste arrivée pour une ouverture antérieure est ignorée. */
	let openSeq = 0;

	/**
	 * Ouvre le dialogue sur l'écriture `entryId` (titre `dialogTitle`, `entryDocumentsTitle`) et lit sa
	 * liste de justificatifs ; `from` reprendra le focus à la fermeture.
	 */
	export function openFor(entryId: number, dialogTitle: string, from: HTMLElement): void {
		const mySeq = ++openSeq;
		trigger = from;
		title = dialogTitle;
		phase = { kind: 'loading' };
		open = true;
		fetchEntryDocuments(entryId).then(
			(documents) => {
				if (mySeq === openSeq) phase = { kind: 'ready', documents };
			},
			(err: unknown) => {
				if (mySeq === openSeq) phase = { kind: 'error', message: errorMessageOf(err) };
			},
		);
	}

	function onOpenChange(value: boolean) {
		// Échap ou clic à côté : la lecture en vol devient périmée.
		if (!value) openSeq++;
	}

	function close() {
		openSeq++;
		open = false;
	}

	function restoreFocus(e: Event) {
		// bits-ui rend le focus à l'élément actif avant l'ouverture, qui n'est pas toujours l'icône (un clic
		// de souris ne la focalise pas partout) : le focus est rendu explicitement à l'icône d'origine.
		e.preventDefault();
		trigger?.focus();
	}

	const withList = $derived(phase.kind === 'ready' && phase.documents.length >= 2);
</script>

{#snippet entryDocumentsButton(p: EntryDocumentsButtonProps)}
	<!-- AC-1 — monochrome, de la couleur du texte (aucune classe de couleur), un peu plus grande qu'une icône
	     de texte, sans remplissage vertical : la ligne garde sa hauteur. -->
	<button
		type="button"
		class="inline-flex items-center rounded-sm p-0 align-middle leading-none focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2"
		aria-label={p.label}
		title={p.label}
		data-testid={p.testId}
		onclick={(e) => p.onOpen(e.currentTarget)}
	>
		<FileText class="size-[1.15em]" aria-hidden="true" />
	</button>
{/snippet}

<Dialog.Root bind:open {onOpenChange}>
	<Dialog.Content
		showCloseButton={false}
		class="max-h-[85vh] overflow-y-auto sm:max-w-4xl {withList
			? '[&_iframe]:h-[calc(85vh-17rem)]'
			: '[&_iframe]:h-[calc(85vh-10rem)]'}"
		onCloseAutoFocus={restoreFocus}
		data-testid="entry-documents-dialog"
	>
		<Dialog.Header>
			<Dialog.Title>{title}</Dialog.Title>
		</Dialog.Header>

		{#if phase.kind === 'loading'}
			<p class="text-sm text-text-muted" role="status" data-testid="entry-documents-loading">
				{i18nMsg('reports-nav-documents-loading', 'Chargement des justificatifs…')}
			</p>
		{:else if phase.kind === 'error'}
			<p class="text-sm text-destructive" role="alert" data-testid="entry-documents-error">
				{phase.message}
			</p>
		{:else if phase.documents.length === 0}
			<p class="text-sm text-text-muted" role="status" data-testid="entry-documents-none">
				{i18nMsg('reports-nav-documents-none', "Cette écriture n'a plus de justificatif.")}
			</p>
		{:else}
			<EntryDocumentsViewer documents={phase.documents} />
		{/if}

		<Dialog.Footer>
			<Button variant="outline" onclick={close} data-testid="entry-documents-close">
				{i18nMsg('reports-nav-documents-close', 'Fermer')}
			</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>
