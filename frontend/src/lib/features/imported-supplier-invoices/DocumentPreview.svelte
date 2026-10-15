<script lang="ts">
	/**
	 * Aperçu d'un document à traiter — Story 30-2a (AC6, C-30-2-6).
	 *
	 * Le document est lu par la fonction `load` (qui passe par `apiClient.getBlob`, donc
	 * par le rafraîchissement du jeton), puis affiché par une URL `blob:` :
	 * - `image/png`, `image/jpeg` → `<img>` ;
	 * - `application/pdf` → `<iframe>`, titre accessible = nom du fichier ;
	 * - **tout autre type** → aucun cadre, un lien « Télécharger » seulement : un type
	 *   futur n'ouvre pas un cadre de même origine.
	 *
	 * La CSP des pages autorise `blob:` dans `img-src`, `frame-src` et `object-src`
	 * (`crates/kesh-api/src/middleware/csp.rs`). L'URL `blob:` est **révoquée** à la
	 * destruction du composant et à chaque changement de document (`load` neuve).
	 *
	 * ⚠️ Angle mort assumé : un PDF à octets magiques valides mais corrompu entre dans la
	 * liste (`PDF_RENDER_ERROR` est un `qrError`, non un refus) ; son aperçu reste vide,
	 * et le document se retire de la liste.
	 */
	import { isApiError } from '$lib/shared/utils/api-error';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';

	interface Props {
		/** Lit le document et rend son `Blob` (type = `Content-Type` de la réponse). */
		load: () => Promise<Blob>;
		/** Nom du fichier : titre accessible du cadre, texte alternatif, nom téléchargé. */
		fileName: string;
	}

	let { load, fileName }: Props = $props();

	type View =
		| { kind: 'loading' }
		| { kind: 'gone' }
		| { kind: 'error' }
		| { kind: 'ready'; url: string; type: string };

	let view = $state<View>({ kind: 'loading' });

	/** Type MIME sans paramètres, en minuscules (`application/pdf; x=y` → `application/pdf`). */
	function baseType(type: string): string {
		return type.split(';')[0].trim().toLowerCase();
	}

	$effect(() => {
		const read = load;
		let url: string | null = null;
		let cancelled = false;
		view = { kind: 'loading' };
		read()
			.then((blob) => {
				if (cancelled) return;
				url = URL.createObjectURL(blob);
				view = { kind: 'ready', url, type: baseType(blob.type) };
			})
			.catch((err: unknown) => {
				if (cancelled) return;
				view =
					isApiError(err) && err.code === 'SOURCE_DOCUMENT_GONE'
						? { kind: 'gone' }
						: { kind: 'error' };
			});
		// Nettoyage : à la destruction ET avant toute relecture (document changé).
		return () => {
			cancelled = true;
			if (url) URL.revokeObjectURL(url);
		};
	});
</script>

<div class="rounded border border-border bg-surface p-2" data-testid="document-preview">
	{#if view.kind === 'loading'}
		<p class="text-sm text-text-muted" data-testid="document-preview-loading">
			{i18nMsg('imported-supplier-invoices-preview-loading', 'Chargement du document…')}
		</p>
	{:else if view.kind === 'gone'}
		<p class="text-sm text-warning" data-testid="document-preview-gone">
			{i18nMsg('imported-supplier-invoices-doc-gone', 'Le document n’a pas été restauré.')}
		</p>
	{:else if view.kind === 'error'}
		<p class="text-sm text-destructive" data-testid="document-preview-error">
			{i18nMsg('imported-supplier-invoices-preview-error', 'Le document n’a pas pu être affiché.')}
		</p>
	{:else if view.type === 'image/png' || view.type === 'image/jpeg'}
		<img
			src={view.url}
			alt={fileName}
			class="mx-auto max-h-[75vh] max-w-full bg-white"
			data-testid="document-preview-image"
		/>
	{:else if view.type === 'application/pdf'}
		<iframe
			src={view.url}
			title={fileName}
			class="h-[75vh] w-full bg-white"
			data-testid="document-preview-frame"
		></iframe>
	{:else}
		<a
			href={view.url}
			download={fileName}
			class="text-sm text-primary underline"
			data-testid="document-preview-download"
		>
			{i18nMsg('imported-supplier-invoices-preview-download', 'Télécharger')}
		</a>
	{/if}
</div>
