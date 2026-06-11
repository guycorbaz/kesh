<script lang="ts">
	// Story 17-4d (AC18/DD-2) — page PUBLIQUE, hors groupe `(app)`. Lit le token
	// du query param `?token=` (URL envoyée par l'email 17-4c :
	// `{public_base_url}/reset-password?token={brut}`). Un user authentifié qui
	// suit un lien valide peut l'utiliser (pas de redirect, contrairement à /setup).
	import { page } from '$app/stores';
	import ResetPasswordForm from '$lib/features/auth-recovery/ResetPasswordForm.svelte';
	import { appVersion } from '$lib/shared/utils/app-version.svelte';

	const token = $derived($page.url.searchParams.get('token'));
</script>

<svelte:head>
	<title>Nouveau mot de passe - Kesh</title>
</svelte:head>

<main class="flex min-h-screen items-center justify-center bg-surface-alt p-4">
	<ResetPasswordForm {token} />

	<footer
		class="absolute bottom-4 left-0 right-0 text-center text-xs text-muted-foreground"
		data-testid="app-version"
	>
		Kesh v{appVersion.value}
	</footer>
</main>
