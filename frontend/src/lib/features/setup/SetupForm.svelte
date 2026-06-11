<script lang="ts">
	import { onDestroy } from 'svelte';
	import { goto } from '$app/navigation';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { setupAdmin } from './setup.api';
	import { isApiError } from '$lib/shared/utils/api-client';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { AlertTriangle, Clock, XCircle } from '@lucide/svelte';

	// Story v011-5 (T4 / AC #15-17) — formulaire création 1er admin.
	// Composant isolé dans `src/lib/features/setup/` pour que
	// `lint-i18n-ownership` scanne les clés `setup-*` (Pass 1 BH1-7 mitigation).

	let username = $state('');
	let password = $state('');
	let passwordConfirm = $state('');
	// Story 17-4d (AC21/DD-5) — email de recovery, optionnel mais recommandé.
	let email = $state('');
	let loading = $state(false);
	let errorMessage = $state('');
	let errorIcon = $state<'validation' | 'rate' | 'gone' | 'server' | null>(null);

	// CR Pass 1 BH1-7 — timer ID pour nettoyage via onDestroy. Sinon, si le
	// composant unmount entre le `setTimeout` (410) et son fire 2s plus tard,
	// `goto('/login')` s'exécute hors contexte → navigation surprenante.
	let redirectTimer: ReturnType<typeof setTimeout> | null = null;
	onDestroy(() => {
		if (redirectTimer !== null) {
			clearTimeout(redirectTimer);
		}
	});

	// Le backend impose ≥ 12 chars (cohérent KESH_PASSWORD_MIN_LENGTH défaut).
	// CR Pass 1 ECH1-4 limitation : valeur hardcoded — si l'opérateur set
	// KESH_PASSWORD_MIN_LENGTH ≠ 12, le frontend pourrait diverger. v0.2 fix
	// via endpoint public config (cf. deferred-work.md).
	const MIN_PASSWORD = 12;

	let usernameValid = $derived(username.trim().length > 0);
	// CR Pass 1 ECH1-3 — utiliser `[...password].length` (Unicode code points)
	// au lieu de `password.length` (UTF-16 code units). Pour les passwords
	// composés d'emojis (caractères astraux 4 bytes UTF-16), `.length` compte
	// 2 par emoji alors que le backend Rust `chars().count()` en compte 1 →
	// 11 emojis = false-valide frontend (11×2=22 ≥ 12) puis 400 backend.
	let passwordValid = $derived([...password].length >= MIN_PASSWORD);
	let passwordMatch = $derived(password === passwordConfirm);
	// Story 17-4d (DD-5) : email optionnel — vide OK, sinon présence de `@`
	// minimale (le backend valide le format complet, AC5 17-4a).
	let emailValid = $derived(email.trim() === '' || email.includes('@'));
	let formValid = $derived(usernameValid && passwordValid && passwordMatch && emailValid);

	async function handleSubmit(e: SubmitEvent) {
		e.preventDefault();
		if (loading || !formValid) return;

		loading = true;
		errorMessage = '';
		errorIcon = null;

		try {
			// Story 17-4d : email vide → `undefined` (omis du JSON, backend Option).
			await setupAdmin(username.trim(), password, email.trim() || undefined);
			// Sur succès : cookies HttpOnly set + authState.login broadcaste.
			// Story v011-5 ECH2-3 : await explicite, login() est synchrone après
			// la résolution de setupAdmin → goto sûr.
			await goto('/onboarding');
		} catch (err) {
			if (isApiError(err)) {
				if (err.status === 410 || err.code === 'SETUP_ALREADY_COMPLETE') {
					errorMessage = i18nMsg(
						'setup-error-already-complete',
						'Le compte administrateur a déjà été créé. Vous allez être redirigé vers la page de connexion.',
					);
					errorIcon = 'gone';
					// CR Pass 1 BH1-7 — track timer pour clearTimeout via onDestroy si
					// le composant unmount avant 2s (sinon goto sur contexte démonté).
					redirectTimer = setTimeout(() => {
						redirectTimer = null;
						void goto('/login');
					}, 2000);
				} else if (err.status === 429 || err.code === 'RATE_LIMITED') {
					errorMessage = i18nMsg(
						'setup-error-rate-limit',
						'Trop de tentatives. Réessayez dans quelques minutes.',
					);
					errorIcon = 'rate';
				} else if (err.status === 400 || err.code === 'VALIDATION_ERROR') {
					errorMessage = err.message || 'Validation échouée.';
					errorIcon = 'validation';
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

<div class="w-full max-w-md rounded-lg border border-border bg-surface p-8 shadow-sm">
	<h1 class="mb-2 text-center text-2xl font-semibold text-text">
		{i18nMsg('setup-welcome', 'Bienvenue dans Kesh')}
	</h1>

	<p class="mb-6 text-center text-sm text-muted-foreground">
		{i18nMsg(
			'setup-intro',
			"Pour terminer l'installation, créez le compte administrateur initial. Ce compte aura les droits complets sur votre instance Kesh.",
		)}
	</p>

	<!-- Zone d'erreur (cohérent /login pattern) -->
	<div
		id="setup-error"
		class="rounded-md text-sm {errorMessage
			? 'mb-4 flex items-center gap-2 border border-error/30 bg-error/5 p-3 text-error'
			: ''}"
		role="alert"
		aria-live="polite"
	>
		{#if errorMessage}
			{#if errorIcon === 'rate'}
				<Clock class="h-4 w-4 flex-shrink-0" aria-hidden="true" />
			{:else if errorIcon === 'validation'}
				<AlertTriangle class="h-4 w-4 flex-shrink-0" aria-hidden="true" />
			{:else}
				<XCircle class="h-4 w-4 flex-shrink-0" aria-hidden="true" />
			{/if}
			<span data-testid="setup-error-message">{errorMessage}</span>
		{/if}
	</div>

	<form onsubmit={handleSubmit} class="flex flex-col gap-4">
		<div class="flex flex-col gap-1.5">
			<label for="setup-username" class="text-sm font-medium text-text">
				{i18nMsg('setup-username-label', "Nom d'utilisateur")}
			</label>
			<Input
				id="setup-username"
				data-testid="setup-username"
				type="text"
				bind:value={username}
				placeholder={i18nMsg('setup-username-placeholder', 'admin')}
				required
				autocomplete="username"
				aria-invalid={!usernameValid && username.length > 0}
			/>
			{#if !usernameValid && username.length > 0}
				<span class="text-xs text-error" data-testid="setup-username-error">
					{i18nMsg('setup-username-required', "Le nom d'utilisateur est obligatoire.")}
				</span>
			{/if}
		</div>

		<div class="flex flex-col gap-1.5">
			<label for="setup-password" class="text-sm font-medium text-text">
				{i18nMsg('setup-password-label', 'Mot de passe')}
			</label>
			<Input
				id="setup-password"
				data-testid="setup-password"
				type="password"
				bind:value={password}
				required
				autocomplete="new-password"
				aria-describedby="setup-password-hint"
				aria-invalid={password.length > 0 && !passwordValid}
			/>
			<span id="setup-password-hint" class="text-xs text-muted-foreground">
				{i18nMsg('setup-password-min', 'Au moins 12 caractères.')}
			</span>
		</div>

		<div class="flex flex-col gap-1.5">
			<label for="setup-password-confirm" class="text-sm font-medium text-text">
				{i18nMsg('setup-password-confirm-label', 'Confirmer le mot de passe')}
			</label>
			<Input
				id="setup-password-confirm"
				data-testid="setup-password-confirm"
				type="password"
				bind:value={passwordConfirm}
				required
				autocomplete="new-password"
				aria-invalid={passwordConfirm.length > 0 && !passwordMatch}
			/>
			{#if passwordConfirm.length > 0 && !passwordMatch}
				<span class="text-xs text-error" data-testid="setup-password-mismatch">
					{i18nMsg('setup-password-mismatch', 'Les mots de passe ne correspondent pas.')}
				</span>
			{/if}
		</div>

		<!-- Story 17-4d (AC21/DD-5) — email de recovery, optionnel mais recommandé. -->
		<div class="flex flex-col gap-1.5">
			<label for="setup-email" class="text-sm font-medium text-text">
				{i18nMsg('setup-email-label', 'Email (recommandé)')}
			</label>
			<Input
				id="setup-email"
				data-testid="setup-email"
				type="email"
				bind:value={email}
				autocomplete="email"
				aria-describedby="setup-email-hint"
				aria-invalid={email.trim().length > 0 && !emailValid}
			/>
			<span id="setup-email-hint" class="text-xs text-muted-foreground">
				{i18nMsg(
					'setup-email-hint',
					'Permet la réinitialisation du mot de passe par email en cas d’oubli.',
				)}
			</span>
			{#if email.trim().length > 0 && !emailValid}
				<span class="text-xs text-error" data-testid="setup-email-invalid">
					{i18nMsg('setup-email-invalid', "Format d'email invalide.")}
				</span>
			{/if}
		</div>

		<Button
			type="submit"
			data-testid="setup-submit"
			class="w-full bg-primary text-white hover:bg-primary/90"
			style="min-height: var(--kesh-target-min-height);"
			disabled={loading || !formValid}
		>
			{#if loading}
				<span
					class="mr-2 inline-block h-4 w-4 animate-spin rounded-full border-2 border-current border-t-transparent"
					aria-hidden="true"
				></span>
			{/if}
			{i18nMsg('setup-submit', 'Créer le compte administrateur')}
		</Button>
	</form>
</div>
