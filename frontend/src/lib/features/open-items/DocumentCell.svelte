<script lang="ts">
	// Story 15-1c-i (AC2, C-15-1c-6) — la pièce d'une ligne : son texte et, quand
	// une route l'ouvre, son lien. Une transaction bancaire n'a pas de lien (aucune
	// route n'ouvre une transaction), un règlement sans facture non plus.
	import { documentHref } from './open-items';
	import { documentLabel } from './open-items-labels';
	import type { DocumentRef } from './open-items.types';

	interface Props {
		document: DocumentRef | null;
	}
	let { document }: Props = $props();

	let href = $derived(document ? documentHref(document) : null);
</script>

{#if document}
	{#if href}
		<a {href} class="text-primary underline" data-testid="open-items-document-link"
			>{documentLabel(document)}</a
		>
	{:else}
		<span data-testid="open-items-document-text">{documentLabel(document)}</span>
	{/if}
{/if}
