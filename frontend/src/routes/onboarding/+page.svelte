<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { onboardingState, i18nMsg, loadI18nMessages } from '$lib/features/onboarding/onboarding.svelte';
	import { modeState } from '$lib/app/stores/mode.svelte';
	import { toast } from 'svelte-sonner';

	function msg(key: string, fallback: string): string {
		return i18nMsg(key, fallback);
	}

	// Charger les messages i18n au mount si reprise à étape 2+
	onMount(async () => {
		if (onboardingState.stepCompleted >= 1) {
			await loadI18nMessages();
		}
	});

	// --- Step handlers ---

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

	async function chooseProductionPath() {
		try {
			await onboardingState.startProduction();
		} catch {
			toast.error(msg('error-internal', 'Erreur lors du démarrage de la configuration'));
		}
	}

	async function chooseOrgType(orgType: string) {
		try {
			await onboardingState.setOrgType(orgType);
		} catch {
			toast.error(msg('error-internal', 'Erreur lors du choix du type'));
		}
	}

	async function chooseAccountingLang(lang: string) {
		try {
			await onboardingState.setAccountingLanguage(lang);
		} catch {
			toast.error(msg('error-internal', 'Erreur lors du choix de la langue comptable'));
		}
	}

	// Form state for coordinates
	let coordName = $state('');
	let coordAddress = $state('');
	let coordIde = $state('');

	async function submitCoordinates() {
		if (!coordName.trim() || !coordAddress.trim()) {
			toast.error(msg('error-validation', 'Nom et adresse sont obligatoires'));
			return;
		}
		try {
			await onboardingState.setCoordinates(
				coordName.trim(),
				coordAddress.trim(),
				coordIde.trim() || null
			);
		} catch {
			toast.error(msg('error-internal', 'Erreur lors de la sauvegarde des coordonnées'));
		}
	}

	// Form state for bank account
	let bankName = $state('');
	let bankIban = $state('');
	let bankQrIban = $state('');

	async function submitBankAccount() {
		if (!bankName.trim() || !bankIban.trim()) {
			toast.error(msg('error-validation', 'Nom de banque et IBAN sont obligatoires'));
			return;
		}
		try {
			await onboardingState.setBankAccount(
				bankName.trim(),
				bankIban.trim(),
				bankQrIban.trim() || null
			);
			goto('/');
		} catch {
			toast.error(msg('error-internal', 'Erreur lors de la sauvegarde du compte bancaire'));
		}
	}

	async function handleSkipBank() {
		try {
			await onboardingState.skipBank();
			goto('/');
		} catch {
			toast.error(msg('error-internal', 'Erreur'));
		}
	}

	const cardClass = "rounded-lg border border-border bg-white p-6 text-left shadow-sm transition-colors hover:border-primary hover:bg-primary-light/5";
</script>

{#if onboardingState.loading && !onboardingState.loaded}
	<div class="flex justify-center p-8">
		<p class="text-text-muted">Chargement...</p>
	</div>

{:else if onboardingState.stepCompleted === 0}
	<!-- Step 1 : Langue -->
	<div class="grid grid-cols-2 gap-4">
		{#each [['FR', 'Français'], ['DE', 'Deutsch'], ['IT', 'Italiano'], ['EN', 'English']] as [code, label]}
			<button class={cardClass + " text-center text-lg font-medium"} onclick={() => chooseLanguage(code)} disabled={onboardingState.loading}>
				{label}
			</button>
		{/each}
	</div>

{:else if onboardingState.stepCompleted === 1}
	<!-- Step 2 : Mode -->
	<h2 class="mb-6 text-center text-xl font-semibold">
		{msg('onboarding-choose-mode', 'Choisissez votre mode d\'utilisation')}
	</h2>
	<div class="flex flex-col gap-4">
		<button class={cardClass} onclick={() => chooseMode('guided')} disabled={onboardingState.loading}>
			<div class="text-lg font-medium">{msg('onboarding-mode-guided', 'Guidé')}</div>
			<div class="mt-1 text-sm text-text-muted">{msg('onboarding-mode-guided-desc', 'Espacements généreux, aide contextuelle, confirmations avant actions')}</div>
		</button>
		<button class={cardClass} onclick={() => chooseMode('expert')} disabled={onboardingState.loading}>
			<div class="text-lg font-medium">{msg('onboarding-mode-expert', 'Expert')}</div>
			<div class="mt-1 text-sm text-text-muted">{msg('onboarding-mode-expert-desc', 'Interface compacte, raccourcis clavier, actions directes')}</div>
		</button>
	</div>

{:else if onboardingState.stepCompleted === 2}
	<!-- Step 3 : Chemin A/B -->
	<h2 class="mb-6 text-center text-xl font-semibold">
		{msg('onboarding-choose-path', 'Comment souhaitez-vous commencer ?')}
	</h2>
	<div class="flex flex-col gap-4">
		<button class={cardClass} onclick={chooseDemoPath} disabled={onboardingState.loading}>
			<div class="text-lg font-medium">{msg('onboarding-path-demo', 'Explorer avec des données de démo')}</div>
			<div class="mt-1 text-sm text-text-muted">{msg('onboarding-path-demo-desc', 'Découvrez Kesh avec des données fictives réalistes')}</div>
		</button>
		<button class={cardClass} onclick={chooseProductionPath} disabled={onboardingState.loading}>
			<div class="text-lg font-medium">{msg('onboarding-path-production', 'Configurer pour la production')}</div>
			<div class="mt-1 text-sm text-text-muted">{msg('onboarding-path-production-desc', 'Configurez votre organisation pour commencer à travailler')}</div>
		</button>
	</div>

{:else if onboardingState.stepCompleted === 3}
	<!-- Step 4 : Type d'organisation -->
	<h2 class="mb-6 text-center text-xl font-semibold">
		{msg('onboarding-choose-org-type', 'Type d\'organisation')}
	</h2>
	<div class="flex flex-col gap-4">
		<button class={cardClass} onclick={() => chooseOrgType('Independant')} disabled={onboardingState.loading}>
			<div class="text-lg font-medium">{msg('onboarding-org-independant', 'Indépendant')}</div>
			<div class="mt-1 text-sm text-text-muted">{msg('onboarding-org-independant-desc', 'Travailleur indépendant, freelance')}</div>
		</button>
		<button class={cardClass} onclick={() => chooseOrgType('Association')} disabled={onboardingState.loading}>
			<div class="text-lg font-medium">{msg('onboarding-org-association', 'Association')}</div>
			<div class="mt-1 text-sm text-text-muted">{msg('onboarding-org-association-desc', 'Association à but non lucratif')}</div>
		</button>
		<button class={cardClass} onclick={() => chooseOrgType('Pme')} disabled={onboardingState.loading}>
			<div class="text-lg font-medium">{msg('onboarding-org-pme', 'PME')}</div>
			<div class="mt-1 text-sm text-text-muted">{msg('onboarding-org-pme-desc', 'Petite et moyenne entreprise (SA, Sàrl)')}</div>
		</button>
	</div>

{:else if onboardingState.stepCompleted === 4}
	<!-- Step 5 : Langue comptable -->
	<h2 class="mb-6 text-center text-xl font-semibold">
		{msg('onboarding-choose-accounting-lang', 'Langue comptable')}
	</h2>
	<p class="mb-4 text-center text-sm text-text-muted">
		{msg('onboarding-accounting-lang-desc', 'Langue des libellés du plan comptable (découplée de la langue de l\'interface)')}
	</p>
	<div class="grid grid-cols-2 gap-4">
		{#each [['FR', 'Français'], ['DE', 'Deutsch'], ['IT', 'Italiano'], ['EN', 'English']] as [code, label]}
			<button class={cardClass + " text-center text-lg font-medium"} onclick={() => chooseAccountingLang(code)} disabled={onboardingState.loading}>
				{label}
			</button>
		{/each}
	</div>

{:else if onboardingState.stepCompleted === 5}
	<!-- Step 6 : Coordonnées -->
	<h2 class="mb-6 text-center text-xl font-semibold">
		{msg('onboarding-coordinates-title', 'Coordonnées de votre organisation')}
	</h2>
	<form class="flex flex-col gap-4" onsubmit={(e) => { e.preventDefault(); submitCoordinates(); }}>
		<div>
			<label for="coord-name" class="mb-1 block text-sm font-medium">
				{msg('onboarding-field-name', 'Nom / Raison sociale')} *
			</label>
			<Input id="coord-name" bind:value={coordName} required />
		</div>
		<div>
			<label for="coord-address" class="mb-1 block text-sm font-medium">
				{msg('onboarding-field-address', 'Adresse')} *
			</label>
			<textarea
				id="coord-address"
				bind:value={coordAddress}
				required
				rows="3"
				class="w-full rounded-md border border-border bg-white px-3 py-2 text-sm"
			></textarea>
		</div>
		<div>
			<label for="coord-ide" class="mb-1 block text-sm font-medium">
				{msg('onboarding-field-ide', 'Numéro IDE')}
				<span class="font-normal text-text-muted"> ({msg('onboarding-field-ide-hint', 'optionnel, format CHE-xxx.xxx.xxx')})</span>
			</label>
			<Input id="coord-ide" bind:value={coordIde} placeholder="CHE-123.456.789" />
		</div>
		<Button type="submit" disabled={onboardingState.loading}>
			{msg('onboarding-next', 'Continuer')}
		</Button>
	</form>

{:else if onboardingState.stepCompleted === 6}
	<!-- Step 7 : Compte bancaire -->
	<h2 class="mb-6 text-center text-xl font-semibold">
		{msg('onboarding-bank-title', 'Compte bancaire principal')}
	</h2>
	<form class="flex flex-col gap-4" onsubmit={(e) => { e.preventDefault(); submitBankAccount(); }}>
		<div>
			<label for="bank-name" class="mb-1 block text-sm font-medium">
				{msg('onboarding-field-bank-name', 'Nom de la banque')} *
			</label>
			<Input id="bank-name" bind:value={bankName} required />
		</div>
		<div>
			<label for="bank-iban" class="mb-1 block text-sm font-medium">
				{msg('onboarding-field-iban', 'IBAN')} *
			</label>
			<Input id="bank-iban" bind:value={bankIban} required placeholder="CH93 0076 2011 6238 5295 7" />
		</div>
		<div>
			<label for="bank-qr-iban" class="mb-1 block text-sm font-medium">
				{msg('onboarding-field-qr-iban', 'QR-IBAN')}
				<span class="font-normal text-text-muted"> (optionnel)</span>
			</label>
			<Input id="bank-qr-iban" bind:value={bankQrIban} placeholder="CH44 3199 9123 0008 8901 2" />
		</div>
		<div class="flex gap-3">
			<Button type="button" variant="outline" onclick={handleSkipBank} disabled={onboardingState.loading} class="flex-1">
				{msg('onboarding-skip-bank', 'Configurer plus tard')}
			</Button>
			<Button type="submit" disabled={onboardingState.loading} class="flex-1">
				{msg('onboarding-next', 'Enregistrer')}
			</Button>
		</div>
	</form>
{/if}
