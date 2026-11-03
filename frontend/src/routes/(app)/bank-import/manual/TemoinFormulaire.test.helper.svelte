<!-- Témoin de test de `bank-import-manual-page.test.ts` : remplace le formulaire et
     expose le rappel `onSaved` que la page lui passe. Jamais rendu hors des tests. -->
<script lang="ts">
	type Props = { onSaved?: (i: unknown) => void };
	let { onSaved }: Props = $props();
	const g = globalThis as { __temoinSaved?: { onSaved: null | ((i: unknown) => void) } };
	$effect(() => {
		if (g.__temoinSaved) g.__temoinSaved.onSaved = onSaved ?? null;
	});
</script>

<div data-testid="temoin-formulaire"></div>
