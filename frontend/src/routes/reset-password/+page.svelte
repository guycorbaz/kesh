<script lang="ts">
	// Story 17-4d (AC18/DD-2) — page PUBLIQUE, hors groupe `(app)`. Lit le token
	// du query param `?token=` (URL envoyée par l'email 17-4c :
	// `{public_base_url}/reset-password?token={brut}`). Un user authentifié qui
	// suit un lien valide peut l'utiliser (pas de redirect, contrairement à /setup).
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import { replaceState } from '$app/navigation';
	import ResetPasswordForm from '$lib/features/auth-recovery/ResetPasswordForm.svelte';
	import { appVersion } from '$lib/shared/utils/app-version.svelte';

	// Pass 1 PD1 — capture ONE-SHOT du token (PAS `$derived` : l'URL est
	// nettoyée juste après, un dérivé redeviendrait `null` et basculerait le
	// formulaire en « lien invalide »). Évalué une fois au chargement de la page.
	const token = $page.url.searchParams.get('token');

	onMount(() => {
		// Pass 1 PD1 — retire le token brut de la barre d'adresse et de
		// l'historique navigateur (défense en profondeur : le secret ne survit
		// pas à l'écran ; il reste de toute façon usage-unique + TTL 30 min).
		if (token !== null) {
			replaceState('/reset-password', {});
		}
	});
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
