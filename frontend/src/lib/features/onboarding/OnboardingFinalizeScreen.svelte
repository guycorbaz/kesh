<script lang="ts">
	/**
	 * Story 36-17 (#1187, D2, D3, D5) — l'écran de l'étape 7 de l'assistant : la
	 * finalisation (réglages de facturation, taux de TVA, premier exercice).
	 *
	 * Trois états, choisis par la page (`onboardingScreenFor`, précédence
	 * échec > en cours > reprise) :
	 *
	 * - `finalizing` — « Configuration en cours… » sur toute la zone (`role="status"`) ;
	 * - `finalize-error` — le motif de l'échec (`role="alert"`), et *Réessayer*. Sous un
	 *   refus `VALIDATION_ERROR`, un lien vers *Administration → Facturation* suit le motif
	 *   du serveur, qui nomme le réglage en défaut et son remède (36-19, #1200) ;
	 * - `resume` — après un rechargement, ni appel ni échec : la configuration n'est pas
	 *   terminée, *Terminer la configuration*. **Aucune finalisation d'office.**
	 *
	 * Sur `finalize-error` et `resume`, la section « Premier exercice » de la page
	 * (`fiscalYear`) précède le bouton, dans un formulaire : les dates sont sous les yeux
	 * de l'utilisateur et modifiables avant toute finalisation, et la touche Entrée dans un
	 * champ de date relance la finalisation par le même chemin que le bouton, sous la même
	 * garde (`disabled`, qui couvre les dates invalides) — revue P1 de l'intégration de la
	 * v0.18.0 (H1).
	 *
	 * Présentationnel : la finalisation et son `catch` vivent dans `+page.svelte`.
	 */
	import type { Snippet } from 'svelte';
	import { Button } from '$lib/components/ui/button';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';

	let {
		mode,
		motif = null,
		remedy = false,
		disabled,
		onFinalize,
		resumeText,
		resumeCta,
		fiscalYear,
	}: {
		/**
		 * La phrase et le bouton de l'écran de reprise : ceux de la bannière de l'application
		 * (clés `incomplete-banner-text` et `incomplete-banner-cta`, D3). La page les résout et
		 * les passe : `lint-i18n-ownership` réserve aux clés `onboarding-*` les composants de
		 * ce dossier.
		 */
		resumeText: string;
		resumeCta: string;
		mode: 'finalizing' | 'finalize-error' | 'resume';
		/** Le motif de l'échec (écran `finalize-error`). */
		motif?: string | null;
		/** Le refus est un `VALIDATION_ERROR` : le lien vers *Facturation* suit le motif. */
		remedy?: boolean;
		/** Un appel est en cours, ou le verrou d'un tour du store : le bouton ne répond pas. */
		disabled: boolean;
		/** *Réessayer* ou *Terminer la configuration* : relance la finalisation. */
		onFinalize: () => void;
		/** La section « Premier exercice » de la page, rendue au-dessus du bouton. */
		fiscalYear?: Snippet;
	} = $props();

	function submit(e: SubmitEvent) {
		e.preventDefault();
		if (!disabled) onFinalize();
	}
</script>

{#if mode === 'finalizing'}
	<div
		class="flex flex-col items-center gap-2 p-8 text-center"
		role="status"
		aria-busy="true"
		data-testid="onboarding-screen-finalizing"
	>
		<p class="text-lg font-medium">{i18nMsg('onboarding-finalizing', 'Configuration en cours…')}</p>
		<p class="text-sm text-text-muted">
			{i18nMsg(
				'onboarding-finalizing-desc',
				"Kesh met en place les réglages de facturation, les taux de TVA et le premier exercice. Ne fermez pas cette page."
			)}
		</p>
	</div>
{:else if mode === 'finalize-error'}
	<div class="flex flex-col gap-4" data-testid="onboarding-screen-finalize-error">
		<div role="alert" class="rounded-md border border-border bg-white p-4 text-sm">
			<p class="font-medium" data-testid="onboarding-finalize-motif">{motif}</p>
			{#if remedy}
				<!-- Revue P1 (E10) : le chemin du remède que le motif nomme, d'un clic. -->
				<p class="mt-2">
					<a
						href="/settings/invoicing"
						class="font-medium text-primary underline hover:no-underline"
						data-testid="onboarding-finalize-remedy-link"
					>
						{i18nMsg('onboarding-finalize-remedy-link', 'Ouvrir Administration → Facturation')}
					</a>
				</p>
			{/if}
		</div>
		<form class="flex flex-col gap-4" onsubmit={submit} data-testid="onboarding-finalize-form">
			{@render fiscalYear?.()}
			<Button type="submit" {disabled} data-testid="onboarding-finalize-retry">
				{i18nMsg('onboarding-retry', 'Réessayer')}
			</Button>
		</form>
	</div>
{:else}
	<div class="flex flex-col gap-4 text-center" data-testid="onboarding-screen-resume">
		<p class="font-medium">{resumeText}</p>
		<form class="flex flex-col gap-4 text-left" onsubmit={submit} data-testid="onboarding-resume-form">
			{@render fiscalYear?.()}
			<Button type="submit" {disabled} data-testid="onboarding-resume-finish">
				{resumeCta}
			</Button>
		</form>
	</div>
{/if}
