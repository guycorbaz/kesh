<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { Button } from '$lib/components/ui/button';
	import { onboardingState, i18nMsg, loadI18nMessages } from '$lib/features/onboarding/onboarding.svelte';
	import { modeState } from '$lib/app/stores/mode.svelte';
	import { toast } from 'svelte-sonner';

	function msg(key: string, fallback: string): string {
		return i18nMsg(key, fallback);
	}

	// P8 — Charger les messages i18n au mount si reprise à étape 2 ou 3
	onMount(async () => {
		if (onboardingState.stepCompleted >= 1) {
			await loadI18nMessages();
		}
	});

	async function chooseLanguage(lang: string) {
		try {
			await onboardingState.setLanguage(lang);
			await loadI18nMessages();
		} catch {
			toast.error('Erreur lors du choix de langue');
		}
	}

	async function chooseMode(mode: 'guided' | 'expert') {
		try {
			await onboardingState.setMode(mode);
			modeState.value = mode;
		} catch {
			toast.error(msg('error-internal', 'Erreur lors du choix de mode'));
		}
	}

	async function chooseDemoPath() {
		try {
			await onboardingState.seedDemo();
			goto('/');
		} catch {
			toast.error(msg('error-internal', 'Erreur lors du chargement des données de démo'));
		}
	}

	function chooseProductionPath() {
		toast.info('À venir — Story 2-3');
	}
</script>

{#if onboardingState.loading && !onboardingState.loaded}
	<div class="flex justify-center p-8">
		<p class="text-text-muted">Chargement...</p>
	</div>
{:else if onboardingState.stepCompleted === 0}
	<!-- Étape 1 : Choix de langue -->
	<div class="grid grid-cols-2 gap-4">
		<button
			class="rounded-lg border border-border bg-white p-6 text-center text-lg font-medium shadow-sm transition-colors hover:border-primary hover:bg-primary-light/5"
			onclick={() => chooseLanguage('FR')}
			disabled={onboardingState.loading}
		>
			Fran&ccedil;ais
		</button>
		<button
			class="rounded-lg border border-border bg-white p-6 text-center text-lg font-medium shadow-sm transition-colors hover:border-primary hover:bg-primary-light/5"
			onclick={() => chooseLanguage('DE')}
			disabled={onboardingState.loading}
		>
			Deutsch
		</button>
		<button
			class="rounded-lg border border-border bg-white p-6 text-center text-lg font-medium shadow-sm transition-colors hover:border-primary hover:bg-primary-light/5"
			onclick={() => chooseLanguage('IT')}
			disabled={onboardingState.loading}
		>
			Italiano
		</button>
		<button
			class="rounded-lg border border-border bg-white p-6 text-center text-lg font-medium shadow-sm transition-colors hover:border-primary hover:bg-primary-light/5"
			onclick={() => chooseLanguage('EN')}
			disabled={onboardingState.loading}
		>
			English
		</button>
	</div>
{:else if onboardingState.stepCompleted === 1}
	<!-- Étape 2 : Choix du mode -->
	<h2 class="mb-6 text-center text-xl font-semibold">
		{msg('onboarding-choose-mode', 'Choisissez votre mode d\'utilisation')}
	</h2>
	<div class="flex flex-col gap-4">
		<button
			class="rounded-lg border border-border bg-white p-6 text-left shadow-sm transition-colors hover:border-primary hover:bg-primary-light/5"
			onclick={() => chooseMode('guided')}
			disabled={onboardingState.loading}
		>
			<div class="text-lg font-medium">{msg('onboarding-mode-guided', 'Guidé')}</div>
			<div class="mt-1 text-sm text-text-muted">
				{msg('onboarding-mode-guided-desc', 'Espacements généreux, aide contextuelle, confirmations avant actions')}
			</div>
		</button>
		<button
			class="rounded-lg border border-border bg-white p-6 text-left shadow-sm transition-colors hover:border-primary hover:bg-primary-light/5"
			onclick={() => chooseMode('expert')}
			disabled={onboardingState.loading}
		>
			<div class="text-lg font-medium">{msg('onboarding-mode-expert', 'Expert')}</div>
			<div class="mt-1 text-sm text-text-muted">
				{msg('onboarding-mode-expert-desc', 'Interface compacte, raccourcis clavier, actions directes')}
			</div>
		</button>
	</div>
{:else if onboardingState.stepCompleted === 2}
	<!-- Étape 3 : Choix du chemin -->
	<h2 class="mb-6 text-center text-xl font-semibold">
		{msg('onboarding-choose-path', 'Comment souhaitez-vous commencer ?')}
	</h2>
	<div class="flex flex-col gap-4">
		<button
			class="rounded-lg border border-border bg-white p-6 text-left shadow-sm transition-colors hover:border-primary hover:bg-primary-light/5"
			onclick={chooseDemoPath}
			disabled={onboardingState.loading}
		>
			<div class="text-lg font-medium">
				{msg('onboarding-path-demo', 'Explorer avec des données de démo')}
			</div>
			<div class="mt-1 text-sm text-text-muted">
				{msg('onboarding-path-demo-desc', 'Découvrez Kesh avec des données fictives réalistes')}
			</div>
		</button>
		<button
			class="rounded-lg border border-border bg-white p-6 text-left shadow-sm transition-colors hover:border-primary hover:bg-primary-light/5"
			onclick={chooseProductionPath}
			disabled={onboardingState.loading}
		>
			<div class="text-lg font-medium">
				{msg('onboarding-path-production', 'Configurer pour la production')}
			</div>
			<div class="mt-1 text-sm text-text-muted">
				{msg('onboarding-path-production-desc', 'Configurez votre organisation pour commencer à travailler')}
			</div>
		</button>
	</div>
{/if}
