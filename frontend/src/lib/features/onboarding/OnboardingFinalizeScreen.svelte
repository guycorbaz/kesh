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
	 *   refus `VALIDATION_ERROR`, une phrase de remède **conditionnelle** précède le motif
	 *   du serveur : la ligne de réglages de facturation a pu être créée sans comptes par un
	 *   écran ouvert avant la fin de la configuration (limite connue, #1200) ;
	 * - `resume` — après un rechargement, ni appel ni échec : la configuration n'est pas
	 *   terminée, *Terminer la configuration*. **Aucune finalisation d'office.**
	 *
	 * Présentationnel : la finalisation et son `catch` vivent dans `+page.svelte`.
	 */
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
		/** Le refus est un `VALIDATION_ERROR` : la phrase de remède précède le motif. */
		remedy?: boolean;
		/** Un appel est en cours, ou le verrou d'un tour du store : le bouton ne répond pas. */
		disabled: boolean;
		/** *Réessayer* ou *Terminer la configuration* : relance la finalisation. */
		onFinalize: () => void;
	} = $props();
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
			{#if remedy}
				<p class="mb-2" data-testid="onboarding-finalize-remedy">
					{i18nMsg(
						'onboarding-finalize-remedy',
						"Si les comptes 1100 et 3000 figurent dans votre plan comptable, ce refus vient d'un écran de Kesh ouvert avant la fin de la configuration. Dans Administration → Facturation, désignez le compte débiteurs et le compte de produit par défaut, vérifiez aussi les autres comptes par défaut (créanciers, arrondi, escompte, frais bancaires, pertes), enregistrez, puis réessayez."
					)}
				</p>
			{/if}
			<p class="font-medium" data-testid="onboarding-finalize-motif">{motif}</p>
		</div>
		<Button type="button" onclick={onFinalize} {disabled} data-testid="onboarding-finalize-retry">
			{i18nMsg('onboarding-retry', 'Réessayer')}
		</Button>
	</div>
{:else}
	<div class="flex flex-col gap-4 text-center" data-testid="onboarding-screen-resume">
		<p class="font-medium">{resumeText}</p>
		<Button type="button" onclick={onFinalize} {disabled} data-testid="onboarding-resume-finish">
			{resumeCta}
		</Button>
	</div>
{/if}
