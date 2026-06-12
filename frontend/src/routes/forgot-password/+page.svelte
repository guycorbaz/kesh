<script lang="ts">
	// Story 17-4d (AC17/DD-2) — page PUBLIQUE, hors groupe `(app)` (pas de guard
	// auth, pattern /login). Pas de redirect des authentifiés (contrairement à
	// /setup) : la page est inoffensive et l'URL peut être suivie depuis l'email.
	import ForgotPasswordForm from '$lib/features/auth-recovery/ForgotPasswordForm.svelte';
	import { appVersion } from '$lib/shared/utils/app-version.svelte';
</script>

<svelte:head>
	<title>Mot de passe oublié - Kesh</title>
</svelte:head>

<main class="flex min-h-screen items-center justify-center bg-surface-alt p-4">
	<ForgotPasswordForm />

	<footer
		class="absolute bottom-4 left-0 right-0 text-center text-xs text-muted-foreground"
		data-testid="app-version"
	>
		Kesh v{appVersion.value}
	</footer>
</main>
