<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { requestPasswordReset } from './auth-recovery.api';
	import { isApiError } from '$lib/shared/utils/api-client';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { Clock, MailCheck, WifiOff, XCircle } from '@lucide/svelte';

	// Story 17-4d (T-D2 / AC17) — formulaire « mot de passe oublié ».
	// Composant isolé dans `src/lib/features/auth-recovery/` pour que
	// `lint-i18n-ownership` scanne les clés `auth-recovery-*` (pattern SetupForm).
	//
	// Anti-énumération (DC4) : sur un 200 — TOUJOURS le même message générique,
	// que le compte existe ou non. Ne jamais écho-er l'identifiant saisi dans le
	// message (pas de « un email a été envoyé à X »).

	// HTTP-LAN safe (AC31) : IDs DOM dérivés de `$props.id()` — pas de
	// `crypto.randomUUID()` (absent en contexte non-sécurisé, bug #145).
	const uid = $props.id();
	const identifierId = `forgot-identifier-${uid}`;
	const errorZoneId = `forgot-error-${uid}`;

	let identifier = $state('');
	let loading = $state(false);
	let submitted = $state(false);
	let errorMessage = $state('');
	let errorIcon = $state<'rate' | 'network' | 'unavailable' | null>(null);

	let identifierValid = $derived(identifier.trim().length > 0);

	async function handleSubmit(e: SubmitEvent) {
		e.preventDefault();
		if (loading || !identifierValid) return;

		loading = true;
		errorMessage = '';
		errorIcon = null;

		try {
			await requestPasswordReset(identifier.trim());
			// DC4 — message générique unique, sans écho de l'identifiant.
			submitted = true;
		} catch (err) {
			if (isApiError(err)) {
				if (err.status === 429 || err.code === 'RATE_LIMITED') {
					errorMessage = i18nMsg(
						'auth-recovery-error-rate-limit',
						'Trop de tentatives. Réessayez dans quelques minutes.',
					);
					errorIcon = 'rate';
				} else if (err.code === 'NETWORK_ERROR' || err.code === 'TIMEOUT') {
					errorMessage = i18nMsg(
						'auth-recovery-error-network',
						'Impossible de contacter le serveur. Vérifiez votre connexion.',
					);
					errorIcon = 'network';
				} else {
					// 404 (feature désactivé côté serveur) ou 5xx → message générique
					// d'indisponibilité, sans détail (le lien login n'affiche cette
					// page que si le flag est actif, mais l'URL reste accessible).
					errorMessage = i18nMsg(
						'auth-recovery-error-unavailable',
						"La réinitialisation par email n'est pas disponible. Contactez votre administrateur.",
					);
					errorIcon = 'unavailable';
				}
			} else {
				errorMessage = i18nMsg(
					'auth-recovery-error-unavailable',
					"La réinitialisation par email n'est pas disponible. Contactez votre administrateur.",
				);
				errorIcon = 'unavailable';
			}
		} finally {
			loading = false;
		}
	}
</script>

<div class="w-full max-w-sm rounded-lg border border-border bg-surface p-8 shadow-sm">
	<h1 class="mb-2 text-center text-2xl font-semibold text-text">
		{i18nMsg('auth-recovery-forgot-title', 'Mot de passe oublié')}
	</h1>

	{#if submitted}
		<!-- DC4 : message générique unique — identique que le compte existe ou non. -->
		<div
			class="mt-4 flex items-start gap-2 rounded-md border border-success/30 bg-success/5 p-3 text-sm text-text"
			role="status"
			data-testid="forgot-success"
		>
			<MailCheck class="mt-0.5 h-4 w-4 flex-shrink-0 text-success" aria-hidden="true" />
			<span>
				{i18nMsg(
					'auth-recovery-success-generic',
					'Si un compte correspond à cet identifiant, un email contenant un lien de réinitialisation vient de lui être envoyé. Le lien est valable 30 minutes.',
				)}
			</span>
		</div>
	{:else}
		<p class="mb-6 text-center text-sm text-muted-foreground">
			{i18nMsg(
				'auth-recovery-forgot-intro',
				'Saisissez votre nom d’utilisateur ou votre adresse email. Si un compte correspond, vous recevrez un lien de réinitialisation.',
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
				<span data-testid="forgot-error">{errorMessage}</span>
			{/if}
		</div>

		<form onsubmit={handleSubmit} class="flex flex-col gap-4">
			<div class="flex flex-col gap-1.5">
				<label for={identifierId} class="text-sm font-medium text-text">
					{i18nMsg('auth-recovery-identifier-label', "Nom d'utilisateur ou email")}
				</label>
				<Input
					id={identifierId}
					data-testid="forgot-identifier"
					type="text"
					bind:value={identifier}
					required
					autocomplete="username"
					aria-describedby={errorMessage ? errorZoneId : undefined}
				/>
			</div>

			<Button
				type="submit"
				data-testid="forgot-submit"
				class="w-full bg-primary text-white hover:bg-primary/90"
				style="min-height: var(--kesh-target-min-height);"
				disabled={loading || !identifierValid}
			>
				{#if loading}
					<span
						class="mr-2 inline-block h-4 w-4 animate-spin rounded-full border-2 border-current border-t-transparent"
						aria-hidden="true"
					></span>
				{/if}
				{i18nMsg('auth-recovery-submit', 'Envoyer le lien de réinitialisation')}
			</Button>
		</form>
	{/if}

	<p class="mt-6 text-center text-sm">
		<a href="/login" class="text-primary hover:underline" data-testid="forgot-back-to-login">
			{i18nMsg('auth-recovery-back-to-login', 'Retour à la connexion')}
		</a>
	</p>
</div>
