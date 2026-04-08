<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { Button } from '$lib/components/ui/button';
	import { XCircle } from '@lucide/svelte';
</script>

<svelte:head>
	<title>Erreur {page.status} - Kesh</title>
</svelte:head>

<main class="flex min-h-screen flex-col items-center justify-center gap-4 bg-surface-alt">
	<XCircle class="h-12 w-12 text-error" aria-hidden="true" />
	<h1 class="text-2xl font-semibold text-text">
		Erreur {page.status}
	</h1>
	<p class="text-sm text-text-muted">
		{page.error?.message ?? 'Une erreur inattendue est survenue.'}
	</p>
	<Button variant="outline" onclick={() => goto('/')}>
		Retour à l'accueil
	</Button>
</main>
