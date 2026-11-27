<script lang="ts">
	// Story 55-3b — hôte de test : une fonction `load` stable, et une `errorMessage` que le test change.
	// Sans hôte, `rerender` de testing-library renotifie toutes les propriétés (relecture artificielle).
	import DocumentPreview from './DocumentPreview.svelte';

	interface Props {
		load: () => Promise<Blob>;
		errorMessage?: (err: unknown) => string | undefined;
	}
	let { load, errorMessage }: Props = $props();
	// La fonction lue au montage seulement : seul `errorMessage` peut changer d'un rendu à l'autre.
	const stable = (() => {
		const f = load;
		return () => f();
	})();
</script>

<DocumentPreview load={stable} fileName="f.pdf" {errorMessage} />
