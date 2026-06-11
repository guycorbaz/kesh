<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { resetPassword } from './auth-recovery.api';
	import { isApiError } from '$lib/shared/utils/api-client';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { CheckCircle2, Clock, LinkIcon, WifiOff, XCircle } from '@lucide/svelte';

	// Story 17-4d (T-D3 / AC18) — formulaire de pose du nouveau mot de passe.
	// Le `token` vient du query param `?token=` lu par la page wrapper.
	//
	// `400 INVALID_OR_EXPIRED_TOKEN` est GÉNÉRIQUE par design (DC4 : inconnu /
	// expiré / déjà utilisé / compte désactivé indistincts) → état « lien
	// invalide » avec CTA vers /forgot-password.

	let { token }: { token: string | null } = $props();

	// HTTP-LAN safe (AC31) : IDs DOM via `$props.id()` — pas de crypto.* (#145).
	const uid = $props.id();
	const passwordId = `reset-password-${uid}`;
	const confirmId = `reset-confirm-${uid}`;
	const hintId = `reset-password-hint-${uid}`;
	const errorZoneId = `reset-error-${uid}`;

	// Le backend impose ≥ 12 chars (KESH_PASSWORD_MIN_LENGTH défaut). Hardcode
	// assumé, même limitation documentée que SetupForm (ECH1-4 v011-5, fix v0.2
	// via endpoint config public) — le backend re-valide (400 VALIDATION_ERROR).
	const MIN_PASSWORD = 12;

	let password = $state('');
	let passwordConfirm = $state('');
	let loading = $state(false);
	let success = $state(false);
	// Rejeté par le backend (400 INVALID_OR_EXPIRED_TOKEN).
	let rejectedToken = $state(false);
	let errorMessage = $state('');
	let errorIcon = $state<'validation' | 'rate' | 'network' | 'server' | null>(null);

	// Token absent de l'URL OU rejeté par le backend → même état « lien invalide ».
	let invalidLink = $derived(token === null || token.trim() === '' || rejectedToken);

	// Unicode-safe : code points, pas UTF-16 units (calque SetupForm ECH1-3).
	let passwordValid = $derived([...password].length >= MIN_PASSWORD);
	let passwordMatch = $derived(password === passwordConfirm);
	let formValid = $derived(passwordValid && passwordMatch);

	async function handleSubmit(e: SubmitEvent) {
		e.preventDefault();
		if (loading || !formValid || invalidLink || token === null) return;

		loading = true;
		errorMessage = '';
		errorIcon = null;

		try {
			await resetPassword(token, password);
			success = true;
		} catch (err) {
			if (isApiError(err)) {
				if (err.code === 'INVALID_OR_EXPIRED_TOKEN') {
					rejectedToken = true;
				} else if (err.status === 429 || err.code === 'RATE_LIMITED') {
					errorMessage = i18nMsg(
						'auth-recovery-error-rate-limit',
						'Trop de tentatives. Réessayez dans quelques minutes.',
					);
					errorIcon = 'rate';
				} else if (err.code === 'VALIDATION_ERROR') {
					// Message backend localisé (politique de mot de passe).
					errorMessage =
						err.message ||
						i18nMsg('auth-recovery-password-min', 'Au moins 12 caractères.');
					errorIcon = 'validation';
				} else if (err.code === 'NETWORK_ERROR' || err.code === 'TIMEOUT') {
					errorMessage = i18nMsg(
						'auth-recovery-error-network',
						'Impossible de contacter le serveur. Vérifiez votre connexion.',
					);
					errorIcon = 'network';
				} else {
					errorMessage = i18nMsg(
						'auth-recovery-error-server',
						'Erreur serveur. Réessayez ultérieurement.',
					);
					errorIcon = 'server';
				}
			} else {
				errorMessage = i18nMsg(
					'auth-recovery-error-server',
					'Erreur serveur. Réessayez ultérieurement.',
				);
				errorIcon = 'server';
			}
		} finally {
			loading = false;
		}
	}
</script>

<div class="w-full max-w-sm rounded-lg border border-border bg-surface p-8 shadow-sm">
	<h1 class="mb-2 text-center text-2xl font-semibold text-text">
		{i18nMsg('auth-recovery-reset-title', 'Nouveau mot de passe')}
	</h1>

	{#if success}
		<div
			class="mt-4 flex items-start gap-2 rounded-md border border-success/30 bg-success/5 p-3 text-sm text-text"
			role="status"
			data-testid="reset-success"
		>
			<CheckCircle2 class="mt-0.5 h-4 w-4 flex-shrink-0 text-success" aria-hidden="true" />
			<span>
				{i18nMsg(
					'auth-recovery-reset-success',
					'Votre mot de passe a été réinitialisé. Vous pouvez maintenant vous connecter.',
				)}
			</span>
		</div>
		<p class="mt-6 text-center">
			<Button
				href="/login"
				data-testid="reset-login-cta"
				class="w-full bg-primary text-white hover:bg-primary/90"
				style="min-height: var(--kesh-target-min-height);"
			>
				{i18nMsg('auth-recovery-login-cta', 'Se connecter')}
			</Button>
		</p>
	{:else if invalidLink}
		<div
			class="mt-4 flex items-start gap-2 rounded-md border border-error/30 bg-error/5 p-3 text-sm text-error"
			role="alert"
			data-testid="reset-invalid-link"
		>
			<LinkIcon class="mt-0.5 h-4 w-4 flex-shrink-0" aria-hidden="true" />
			<span>
				{i18nMsg(
					'auth-recovery-invalid-link',
					'Ce lien de réinitialisation est invalide ou expiré. Refaites une demande pour recevoir un nouveau lien.',
				)}
			</span>
		</div>
		<p class="mt-6 text-center">
			<Button
				href="/forgot-password"
				variant="outline"
				data-testid="reset-request-new-link"
				class="w-full"
				style="min-height: var(--kesh-target-min-height);"
			>
				{i18nMsg('auth-recovery-request-new-link', 'Refaire une demande')}
			</Button>
		</p>
	{:else}
		<p class="mb-6 text-center text-sm text-muted-foreground">
			{i18nMsg(
				'auth-recovery-reset-intro',
				'Choisissez votre nouveau mot de passe.',
			)}
		</p>

		<!-- Zone d'erreur — toujours dans le DOM pour que aria-live fonctionne. -->
		<div
			id={errorZoneId}
			class="rounded-md text-sm {errorMessage
				? 'mb-4 flex items-center gap-2 border border-error/30 bg-error/5 p-3 text-error'
				: ''}"
			role="alert"
			aria-live="polite"
		>
			{#if errorMessage}
				{#if errorIcon === 'rate'}
					<Clock class="h-4 w-4 flex-shrink-0" aria-hidden="true" />
				{:else if errorIcon === 'network'}
					<WifiOff class="h-4 w-4 flex-shrink-0" aria-hidden="true" />
				{:else}
					<XCircle class="h-4 w-4 flex-shrink-0" aria-hidden="true" />
				{/if}
				<span data-testid="reset-error">{errorMessage}</span>
			{/if}
		</div>

		<form onsubmit={handleSubmit} class="flex flex-col gap-4">
			<div class="flex flex-col gap-1.5">
				<label for={passwordId} class="text-sm font-medium text-text">
					{i18nMsg('auth-recovery-new-password-label', 'Nouveau mot de passe')}
				</label>
				<Input
					id={passwordId}
					data-testid="reset-password"
					type="password"
					bind:value={password}
					required
					autocomplete="new-password"
					aria-describedby={hintId}
					aria-invalid={password.length > 0 && !passwordValid}
				/>
				<span id={hintId} class="text-xs text-muted-foreground">
					{i18nMsg('auth-recovery-password-min', 'Au moins 12 caractères.')}
				</span>
			</div>

			<div class="flex flex-col gap-1.5">
				<label for={confirmId} class="text-sm font-medium text-text">
					{i18nMsg('auth-recovery-password-confirm-label', 'Confirmer le mot de passe')}
				</label>
				<Input
					id={confirmId}
					data-testid="reset-password-confirm"
					type="password"
					bind:value={passwordConfirm}
					required
					autocomplete="new-password"
					aria-invalid={passwordConfirm.length > 0 && !passwordMatch}
				/>
				{#if passwordConfirm.length > 0 && !passwordMatch}
					<span class="text-xs text-error" data-testid="reset-password-mismatch">
						{i18nMsg('auth-recovery-password-mismatch', 'Les mots de passe ne correspondent pas.')}
					</span>
				{/if}
			</div>

			<Button
				type="submit"
				data-testid="reset-submit"
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
				{i18nMsg('auth-recovery-reset-submit', 'Réinitialiser le mot de passe')}
			</Button>
		</form>
	{/if}

	{#if !success}
		<p class="mt-6 text-center text-sm">
			<a href="/login" class="text-primary hover:underline" data-testid="reset-back-to-login">
				{i18nMsg('auth-recovery-back-to-login', 'Retour à la connexion')}
			</a>
		</p>
	{/if}
</div>
