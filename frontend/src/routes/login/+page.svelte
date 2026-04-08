<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { authState } from '$lib/app/stores/auth.svelte';
	import { apiClient, isApiError } from '$lib/shared/utils/api-client';
	import { AlertTriangle, Clock, WifiOff, XCircle } from '@lucide/svelte';

	let username = $state('');
	let password = $state('');
	let loading = $state(false);
	let errorMessage = $state('');
	let errorIcon = $state<'credentials' | 'rate' | 'network' | 'server' | 'session' | null>(null);

	// Détection du paramètre ?reason=session_expired
	const sessionExpired = $derived($page.url.searchParams.get('reason') === 'session_expired');

	async function handleLogin(e: SubmitEvent) {
		e.preventDefault();
		if (loading) return;

		loading = true;
		errorMessage = '';
		errorIcon = null;

		try {
			const data = await apiClient.post<{
				accessToken: string;
				refreshToken: string;
				expiresIn: number;
			}>('/api/v1/auth/login', { username, password });
			authState.login(data.accessToken, data.refreshToken, data.expiresIn);
			await goto('/');
		} catch (err) {
			if (isApiError(err)) {
				if (err.code === 'NETWORK_ERROR') {
					errorMessage = 'Impossible de contacter le serveur. Vérifiez votre connexion.';
					errorIcon = 'network';
				} else if (err.status === 401) {
					errorMessage = 'Identifiant ou mot de passe incorrect';
					errorIcon = 'credentials';
				} else if (err.status === 429) {
					errorMessage = 'Trop de tentatives de connexion. Réessayez dans quelques minutes.';
					errorIcon = 'rate';
				} else {
					errorMessage = 'Erreur serveur. Réessayez ultérieurement.';
					errorIcon = 'server';
				}
			} else {
				errorMessage = 'Erreur inattendue. Réessayez ultérieurement.';
				errorIcon = 'server';
			}
		} finally {
			loading = false;
		}
	}
</script>

<svelte:head>
	<title>Connexion - Kesh</title>
</svelte:head>

<main class="flex min-h-screen items-center justify-center bg-surface-alt">
	<div class="w-full max-w-sm rounded-lg border border-border bg-surface p-8 shadow-sm">
		<h1 class="mb-6 text-center text-2xl font-semibold text-text">Kesh</h1>

		<!-- Message session expirée -->
		{#if sessionExpired && !errorMessage}
			<div
				class="mb-4 flex items-center gap-2 rounded-md border border-warning/30 bg-warning/5 p-3 text-sm text-warning"
				role="status"
			>
				<Clock class="h-4 w-4 flex-shrink-0" aria-hidden="true" />
				<span>Votre session a expiré. Veuillez vous reconnecter.</span>
			</div>
		{/if}

		<!-- Zone d'erreur — toujours dans le DOM pour que aria-live fonctionne -->
		<div
			id="login-error"
			class="rounded-md text-sm {errorMessage
				? 'mb-4 flex items-center gap-2 border border-error/30 bg-error/5 p-3 text-error'
				: ''}"
			role="alert"
			aria-live="polite"
		>
			{#if errorMessage}
				{#if errorIcon === 'credentials'}
					<AlertTriangle class="h-4 w-4 flex-shrink-0" aria-hidden="true" />
				{:else if errorIcon === 'rate'}
					<Clock class="h-4 w-4 flex-shrink-0" aria-hidden="true" />
				{:else if errorIcon === 'network'}
					<WifiOff class="h-4 w-4 flex-shrink-0" aria-hidden="true" />
				{:else}
					<XCircle class="h-4 w-4 flex-shrink-0" aria-hidden="true" />
				{/if}
				<span>{errorMessage}</span>
			{/if}
		</div>

		<form onsubmit={handleLogin} class="flex flex-col gap-4">
			<div class="flex flex-col gap-1.5">
				<label for="username" class="text-sm font-medium text-text">
					Identifiant
				</label>
				<Input
					id="username"
					type="text"
					bind:value={username}
					placeholder="Votre identifiant"
					required
					autocomplete="username"
					aria-describedby={errorMessage ? 'login-error' : undefined}
				/>
			</div>

			<div class="flex flex-col gap-1.5">
				<label for="password" class="text-sm font-medium text-text">
					Mot de passe
				</label>
				<Input
					id="password"
					type="password"
					bind:value={password}
					placeholder="Votre mot de passe"
					required
					autocomplete="current-password"
					aria-describedby={errorMessage ? 'login-error' : undefined}
				/>
			</div>

			<Button
				type="submit"
				class="w-full bg-primary text-white hover:bg-primary/90"
				style="min-height: var(--kesh-target-min-height);"
				disabled={loading}
			>
				{#if loading}
					<span class="mr-2 inline-block h-4 w-4 animate-spin rounded-full border-2 border-current border-t-transparent" aria-hidden="true"></span>
				{/if}
				Se connecter
			</Button>
		</form>
	</div>
</main>
