<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { authState } from '$lib/app/stores/auth.svelte';
	import { apiClient, isApiError } from '$lib/shared/utils/api-client';
	import { appVersion } from '$lib/shared/utils/app-version.svelte';
	import { featureFlags } from '$lib/shared/utils/feature-flags.svelte';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { AlertTriangle, Clock, WifiOff, XCircle } from '@lucide/svelte';

	let username = $state('');
	let password = $state('');
	let loading = $state(false);
	let errorMessage = $state('');
	let errorIcon = $state<'credentials' | 'rate' | 'network' | 'server' | 'session' | null>(null);

	// Story v0172-session (0.17.3, #876) — fermeture pour inactivité :
	// `?reason=idle&minutes=N` (N entier). Sans N valable, le message de session
	// expirée. ⚠️ Sur `/login`, les messages i18n ne sont pas chargés (endpoint
	// authentifié) : le repli français s'affiche quelle que soit la langue.
	const idleMinutes = $derived.by(() => {
		if ($page.url.searchParams.get('reason') !== 'idle') return null;
		const raw = $page.url.searchParams.get('minutes') ?? '';
		if (!/^[0-9]{1,4}$/.test(raw)) return null;
		const n = Number(raw);
		// Bornes du réglage (revue de code P1, N-B4/N-E6) : hors de 1 à 1440,
		// le message de session expirée, sans durée.
		return n >= 1 && n <= 1440 ? n : null;
	});
	const idleClosed = $derived(idleMinutes !== null);

	// Détection du paramètre ?reason=session_expired (ou d'une fermeture pour
	// inactivité sans durée lisible).
	const sessionExpired = $derived(
		$page.url.searchParams.get('reason') === 'session_expired' ||
			($page.url.searchParams.get('reason') === 'idle' && idleMinutes === null),
	);

	async function handleLogin(e: SubmitEvent) {
		e.preventDefault();
		if (loading) return;

		loading = true;
		errorMessage = '';
		errorIcon = null;

		try {
			// Story 10-5 (T6.4bis Pass 3 F-LOGIN-PAGE-DECODES-JWT-P3-4) :
			// signature `authState.login()` change pour un objet payload
			// (sans tokens — ils sont dans les cookies HttpOnly émis par le
			// backend via Set-Cookie). Les champs `userId/username/role` du
			// body (D6 acté Pass 2) permettent de peupler `_currentUser`
			// sans décoder le JWT côté JS.
			const data = await apiClient.post<{
				accessToken: string;
				refreshToken: string;
				expiresIn: number;
				userId: number;
				username: string;
				role: string;
				idleTimeoutMinutes?: number;
				idleExpiresIn?: number | null;
			}>('/api/v1/auth/login', { username, password });
			authState.login({
				userId: String(data.userId),
				username: data.username,
				role: data.role,
				expiresIn: data.expiresIn,
				idleTimeoutMinutes: data.idleTimeoutMinutes,
				idleExpiresIn: data.idleExpiresIn,
			});
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

		<!-- Story v0172-session — session fermée pour inactivité -->
		{#if idleClosed && !errorMessage}
			<div
				class="mb-4 flex items-center gap-2 rounded-md border border-warning/30 bg-warning/5 p-3 text-sm text-warning"
				role="status"
				data-testid="login-session-idle"
			>
				<Clock class="h-4 w-4 flex-shrink-0" aria-hidden="true" />
				<!-- Singulier par une clé `-one` (précédent `supplier-invoices-count-one`) :
				     `i18nMsg` n'évalue pas les sélecteurs Fluent (choix C-v0172s-60). -->
				<span
					>{idleMinutes === 1
						? i18nMsg('login-session-idle-one', "Session fermée après 1 minute d'inactivité.")
						: i18nMsg(
								'login-session-idle',
								"Session fermée après { $minutes } minutes d'inactivité.",
								{ minutes: idleMinutes ?? 0 },
							)}</span
				>
			</div>
		{/if}

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
				<!-- Story 17-4d (AC20/DC9) : lien recovery conditionné sur le flag
				     backend lu via /health (défaut false → absent du DOM tant que
				     /health n'a pas répondu OU si KESH_FEATURE_FORGOT_PASSWORD=false). -->
				{#if featureFlags.forgotPasswordEnabled}
					<a
						href="/forgot-password"
						class="self-end text-sm text-primary hover:underline"
						data-testid="forgot-password-link"
					>
						Mot de passe oublié ?
					</a>
				{/if}
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

	<footer
		class="absolute bottom-4 left-0 right-0 text-center text-xs text-muted-foreground"
		data-testid="app-version"
	>
		Kesh v{appVersion.value}
	</footer>
</main>
