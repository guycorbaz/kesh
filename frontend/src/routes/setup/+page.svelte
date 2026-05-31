<script lang="ts">
	import SetupForm from '$lib/features/setup/SetupForm.svelte';
</script>

<svelte:head>
	<title>Configuration initiale - Kesh</title>
</svelte:head>

<main class="flex min-h-screen items-center justify-center bg-surface-alt p-4">
	<SetupForm />

	<footer
		class="absolute bottom-4 left-0 right-0 text-center text-xs text-muted-foreground"
		data-testid="app-version"
	>
		Kesh v{__APP_VERSION__}
	</footer>
</main>
