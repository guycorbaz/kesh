<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { onboardingState, i18nMsg, loadI18nMessages } from '$lib/features/onboarding/onboarding.svelte';
	import { modeState } from '$lib/app/stores/mode.svelte';
	import { authState } from '$lib/app/stores/auth.svelte';
	import { isApiError } from '$lib/shared/utils/api-client';
	import { errorMessageOf } from '$lib/shared/utils/error-message';
	import { toast } from 'svelte-sonner';
	import { fetchAccounts } from '$lib/features/accounts/accounts.api';
	import type { AccountResponse } from '$lib/features/accounts/accounts.types';
	import { accountIdsWithRole } from '$lib/features/accounts/account-options';
	import {
		BANK_ACCOUNT_DESCRIPTION_MAX,
		linkableLedgerAccounts,
		preselectedBankLedgerId,
	} from '$lib/features/bank-accounts/format';

	function msg(key: string, fallback: string): string {
		return i18nMsg(key, fallback);
	}

	// Story 31-1b (AC15, #503) — restaurer une sauvegarde au lieu de configurer :
	// aux étapes 0 à 2 seulement (avant le choix démo / production, qui crée des
	// données), et pour l'Administrateur seulement (le compte créé par /setup
	// l'est ; le serveur refuse de toute façon les autres rôles, 403).
	const showRestoreLink = $derived(
		onboardingState.loaded &&
			onboardingState.stepCompleted <= 2 &&
			authState.currentUser?.role === 'Admin'
	);

	// Charger les messages i18n au mount si reprise à étape 2+
	onMount(async () => {
		if (onboardingState.stepCompleted >= 1) {
			await loadI18nMessages();
		}
	});

	// --- Step handlers ---
	//
	// Story v017-2 (#809) : chaque `catch` lie l'erreur et affiche le motif du refus
	// du serveur, par `errorMessageOf(err, <message de l'étape>)` — le message de
	// l'étape ne s'affiche plus que si le serveur ne donne pas de motif lisible (panne
	// réseau, message vide). La garde `onboarding-catch-guard.test.ts` interdit le
	// retour d'un `catch` sans liaison dans ce fichier.

	async function chooseLanguage(lang: string) {
		try {
			await onboardingState.setLanguage(lang);
			await loadI18nMessages();
		} catch (err) {
			toast.error(errorMessageOf(err, msg('onboarding-language-error', 'Erreur lors du choix de langue')));
		}
	}

	async function chooseMode(mode: 'guided' | 'expert') {
		try {
			await onboardingState.setMode(mode);
			modeState.value = mode;
		} catch (err) {
			toast.error(errorMessageOf(err, msg('onboarding-mode-error', 'Erreur lors du choix de mode')));
		}
	}

	async function chooseDemoPath() {
		try {
			await onboardingState.seedDemo();
			goto('/');
		} catch (err) {
			toast.error(
				errorMessageOf(err, msg('onboarding-demo-load-error', 'Erreur lors du chargement des données de démo'))
			);
		}
	}

	async function chooseProductionPath() {
		try {
			await onboardingState.startProduction();
		} catch (err) {
			toast.error(
				errorMessageOf(err, msg('onboarding-setup-start-error', 'Erreur lors du démarrage de la configuration'))
			);
		}
	}

	async function chooseOrgType(orgType: string) {
		try {
			await onboardingState.setOrgType(orgType);
		} catch (err) {
			toast.error(errorMessageOf(err, msg('onboarding-org-type-error', 'Erreur lors du choix du type')));
		}
	}

	async function chooseAccountingLang(lang: string) {
		try {
			await onboardingState.setAccountingLanguage(lang);
		} catch (err) {
			toast.error(
				errorMessageOf(err, msg('onboarding-accounting-lang-error', 'Erreur lors du choix de la langue comptable'))
			);
		}
	}

	// Form state for coordinates (#213 — adresse structurée + prénom/nom optionnels).
	let coordName = $state('');
	let coordFirstName = $state('');
	let coordLastName = $state('');
	let coordStreet = $state('');
	let coordBuilding = $state('');
	let coordPostalCode = $state('');
	let coordCity = $state('');
	let coordCountry = $state('CH');
	let coordIde = $state('');

	async function submitCoordinates() {
		const isPerson = coordFirstName.trim() !== '' || coordLastName.trim() !== '';
		const hasName = isPerson
			? coordFirstName.trim() !== '' && coordLastName.trim() !== ''
			: coordName.trim() !== '';
		if (!hasName) {
			toast.error(msg('onboarding-name-required', 'Nom (ou prénom + nom) obligatoire'));
			return;
		}
		if (!coordPostalCode.trim() || !coordCity.trim()) {
			toast.error(
				msg('onboarding-address-npa-city-required', 'NPA et localité sont obligatoires')
			);
			return;
		}
		try {
			await onboardingState.setCoordinates(
				coordName.trim(),
				coordFirstName.trim() || null,
				coordLastName.trim() || null,
				{
					street: coordStreet.trim(),
					building: coordBuilding.trim(),
					postalCode: coordPostalCode.trim(),
					city: coordCity.trim(),
					country: coordCountry.trim() || 'CH'
				},
				coordIde.trim() || null
			);
		} catch (err) {
			toast.error(
				errorMessageOf(err, msg('onboarding-address-save-error', 'Erreur lors de la sauvegarde des coordonnées'))
			);
		}
	}

	// Form state for bank account
	let bankName = $state('');
	let bankIban = $state('');
	let bankQrIban = $state('');

	// Story 34-1 (#966) — description et compte comptable lié du compte bancaire.
	//
	// Le plan est chargé à l'étape 4→5 : à l'étape 6, la liste des comptes est
	// lue (`GET /api/v1/accounts`, servi à tout utilisateur authentifié) et
	// filtrée par `linkableLedgerAccounts` — la même fonction que l'écran
	// *Comptes bancaires* —, sans les comptes de RÔLE `Receivable` et `Payable` :
	// les réglages de facturation, qui désignent les comptes de créance, ne
	// naissent qu'à la finalisation, et c'est sur ces rôles qu'ils les poseront
	// (C-e341-3 ; le serveur refuse de toute façon, AC-4).
	//
	// ⛔ La présélection du `1020` est un geste d'écran (décision de Guy, Story
	// 14-3a) : le serveur ne connaît aucun numéro. Si la liste ne se charge pas,
	// le champ reste « Non configuré » et l'étape reste utilisable (AC-6).
	let bankDescription = $state('');
	let bankJournalAccountId = $state<number | null>(null);
	let bankLedgerOptions = $state<AccountResponse[]>([]);
	let bankLedgerError = $state<string | null>(null);
	let bankLedgerRequested = false;

	async function loadBankLedgerOptions() {
		try {
			const all = await fetchAccounts(false);
			const roleIds = new Set([
				...accountIdsWithRole(all, 'Receivable'),
				...accountIdsWithRole(all, 'Payable'),
			]);
			bankLedgerOptions = linkableLedgerAccounts(all, roleIds);
			bankJournalAccountId = preselectedBankLedgerId(bankLedgerOptions);
		} catch (err) {
			// Liste indisponible : le compte se crée sans lien, et se lie ensuite
			// dans Administration → Comptes bancaires (AC-6). Le motif s'affiche
			// SOUS le champ, non en toast : l'étape reste utilisable (garde
			// `onboarding-catch-guard.test.ts` : tout `catch` passe par
			// `errorMessageOf`).
			bankLedgerOptions = [];
			bankJournalAccountId = null;
			bankLedgerError = errorMessageOf(
				err,
				msg(
					'onboarding-bank-ledger-load-error',
					'La liste des comptes n’a pas pu être chargée : le compte bancaire peut être lié plus tard.'
				)
			);
		}
	}

	$effect(() => {
		if (onboardingState.stepCompleted === 6 && !bankLedgerRequested) {
			bankLedgerRequested = true;
			void loadBankLedgerOptions();
		}
	});

	// P9: distinguish bank-step errors from finalize() errors so the user gets an
	// actionable message and doesn't end up stuck at /onboarding step 7 with the
	// bank account already saved.
	//
	// Story v017-2 (#809, AC-4) : la branche comparait le code à `VALIDATION`, code
	// que le serveur ne rend jamais (`VALIDATION_ERROR`, `errors.rs`) ; le refus « comptes
	// 1100/3000 manquants » s'affichait donc « Finalisation échouée ». Toute erreur autre
	// que « déjà finalisé » passe désormais par `errorMessageOf` : le motif du serveur,
	// sinon le message de la finalisation.
	function reportFinalizeError(err: unknown) {
		if (isApiError(err) && err.code === 'ONBOARDING_STEP_ALREADY_COMPLETED') {
			toast.error(
				msg(
					'onboarding-already-finalized',
					"L'onboarding est déjà finalisé. Veuillez recharger la page."
				)
			);
			return;
		}
		toast.error(errorMessageOf(err, msg('onboarding-finalize-error', 'Finalisation échouée — réessayez.')));
	}

	// P6-L7: defense-in-depth — never navigate to / unless the backend confirmed
	// stepCompleted == 8. A bug or race that returns a stale snapshot would
	// otherwise drop the user into the main app with onboarding incomplete.
	function navigateHomeIfFinalized() {
		if (onboardingState.stepCompleted === 8) {
			goto('/');
		} else {
			toast.error(
				msg(
					'onboarding-finalize-incomplete',
					"La finalisation n'a pas été confirmée. Veuillez recharger la page."
				)
			);
		}
	}

	async function submitBankAccount() {
		if (!bankName.trim() || !bankIban.trim()) {
			toast.error(msg('onboarding-bank-required', 'Nom de banque et IBAN sont obligatoires'));
			return;
		}
		try {
			// Story 34-1 (AC-1) : description et compte lié toujours envoyés, `null`
			// compris — un champ absent vaudrait « inchangé » côté serveur.
			await onboardingState.setBankAccount({
				bankName: bankName.trim(),
				iban: bankIban.trim(),
				qrIban: bankQrIban.trim() || null,
				description: bankDescription.trim() || null,
				journalAccountId: bankJournalAccountId,
			});
		} catch (err) {
			toast.error(
				errorMessageOf(err, msg('onboarding-bank-save-error', 'Erreur lors de la sauvegarde du compte bancaire'))
			);
			return;
		}
		try {
			await onboardingState.finalize();
			navigateHomeIfFinalized();
		} catch (err) {
			reportFinalizeError(err);
		}
	}

	async function handleSkipBank() {
		try {
			await onboardingState.skipBank();
		} catch (err) {
			toast.error(
				errorMessageOf(err, msg('onboarding-bank-skip-error', 'Erreur lors du saut du compte bancaire'))
			);
			return;
		}
		try {
			await onboardingState.finalize();
			navigateHomeIfFinalized();
		} catch (err) {
			reportFinalizeError(err);
		}
	}

	const cardClass = "rounded-lg border border-border bg-white p-6 text-left shadow-sm transition-colors hover:border-primary hover:bg-primary-light/5";
</script>

{#if onboardingState.loading && !onboardingState.loaded}
	<div class="flex justify-center p-8">
		<p class="text-text-muted">{msg('common-loading', 'Chargement…')}</p>
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
		<button class={cardClass} onclick={() => chooseOrgType('Independant')} disabled={onboardingState.loading} data-testid="onboarding-org-type-independant">
			<div class="text-lg font-medium">{msg('onboarding-org-independant', 'Indépendant')}</div>
			<div class="mt-1 text-sm text-text-muted">{msg('onboarding-org-independant-desc', 'Travailleur indépendant, freelance')}</div>
		</button>
		<button class={cardClass} onclick={() => chooseOrgType('Association')} disabled={onboardingState.loading} data-testid="onboarding-org-type-association">
			<div class="text-lg font-medium">{msg('onboarding-org-association', 'Association')}</div>
			<div class="mt-1 text-sm text-text-muted">{msg('onboarding-org-association-desc', 'Association à but non lucratif')}</div>
		</button>
		<button class={cardClass} onclick={() => chooseOrgType('Pme')} disabled={onboardingState.loading} data-testid="onboarding-org-type-pme">
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
				{msg('onboarding-field-name', 'Raison sociale')}
				<span class="font-normal text-text-muted"> ({msg('onboarding-field-name-hint', 'entreprise — sinon remplir prénom + nom')})</span>
			</label>
			<Input id="coord-name" bind:value={coordName} />
		</div>
		<div class="grid grid-cols-2 gap-3">
			<div>
				<label for="coord-firstname" class="mb-1 block text-sm font-medium">
					{msg('field-first-name', 'Prénom')}
				</label>
				<Input id="coord-firstname" bind:value={coordFirstName} />
			</div>
			<div>
				<label for="coord-lastname" class="mb-1 block text-sm font-medium">
					{msg('field-last-name', 'Nom')}
				</label>
				<Input id="coord-lastname" bind:value={coordLastName} />
			</div>
		</div>
		<fieldset class="rounded-md border border-border p-3">
			<legend class="px-1 text-sm font-medium">{msg('field-address', 'Adresse')} *</legend>
			<div class="grid grid-cols-3 gap-3">
				<div class="col-span-2">
					<label for="coord-street" class="mb-1 block text-xs text-text-muted">{msg('field-street', 'Rue')}</label>
					<Input id="coord-street" bind:value={coordStreet} />
				</div>
				<div>
					<label for="coord-building" class="mb-1 block text-xs text-text-muted">{msg('field-building', 'N°')}</label>
					<Input id="coord-building" bind:value={coordBuilding} />
				</div>
				<div>
					<label for="coord-npa" class="mb-1 block text-xs text-text-muted">{msg('field-postal-code', 'NPA')} *</label>
					<Input id="coord-npa" bind:value={coordPostalCode} required />
				</div>
				<div class="col-span-2">
					<label for="coord-city" class="mb-1 block text-xs text-text-muted">{msg('field-city', 'Localité')} *</label>
					<Input id="coord-city" bind:value={coordCity} required />
				</div>
				<div>
					<label for="coord-country" class="mb-1 block text-xs text-text-muted">{msg('field-country', 'Pays')}</label>
					<Input
						id="coord-country"
						bind:value={coordCountry}
						placeholder="CH"
						aria-describedby="coord-country-hint"
					/>
					<!-- Story v017-2 (#809, AC-6) : la forme attendue, sous le champ et non dans le
					     libellé (cellule d'un tiers de grille). Pas de maxlength : « Suisse » doit
					     atteindre le serveur et revenir expliqué, non être tronqué en « Su », accepté. -->
					<p id="coord-country-hint" class="mt-1 text-xs text-text-muted">
						{msg('onboarding-field-country-hint', 'code ISO à deux lettres, ex. CH, FR, DE')}
					</p>
				</div>
			</div>
		</fieldset>
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
				<!-- ⚠️ « (optionnel) » vivait ICI en dur, à côté d'un libellé traduit. La convention du
				     dépôt le met DANS le libellé (`account-field-parent-optional`,
				     `vat-rates-field-to`…), et `bank-accounts-labels-qr-iban` porte déjà
				     « QR-IBAN (optionnel) » traduit dans les quatre locales, pour ce champ exact.
				     Réutilisée plutôt que dupliquée. (Story 23-4, AC6-bis.) -->
				{msg('bank-accounts-labels-qr-iban', 'QR-IBAN (optionnel)')}
			</label>
			<Input id="bank-qr-iban" bind:value={bankQrIban} placeholder="CH44 3199 9123 0008 8901 2" />
		</div>
		<!-- Story 34-1 (#966, AC-6) : les libellés, la limite et l'aide sont ceux de
		     la page Comptes bancaires (clés réutilisées, mêmes locales). -->
		<div>
			<label for="bank-description" class="mb-1 block text-sm font-medium">
				{msg('bank-accounts-labels-description', 'Description (optionnelle)')}
			</label>
			<Input
				id="bank-description"
				bind:value={bankDescription}
				maxlength={BANK_ACCOUNT_DESCRIPTION_MAX}
				aria-describedby="bank-description-help"
			/>
			<p id="bank-description-help" class="mt-1 text-xs text-text-muted">
				{msg(
					'bank-accounts-help-description',
					'Par exemple « Personnel » ou « Épargne » : pour distinguer deux comptes de la même banque. 60 caractères au plus.'
				)}
			</p>
		</div>
		<div>
			<label for="bank-journal-account" class="mb-1 block text-sm font-medium">
				{msg('bank-accounts-labels-journal-account-id', 'Compte comptable lié')}
			</label>
			<select
				id="bank-journal-account"
				bind:value={bankJournalAccountId}
				class="w-full rounded border border-border px-3 py-2 text-sm"
				data-testid="onboarding-bank-journal-account"
			>
				<option value={null}>{msg('bank-accounts-labels-not-configured', 'Non configuré')}</option>
				{#each bankLedgerOptions as acc (acc.id)}
					<option value={acc.id}>{acc.number} — {acc.name}</option>
				{/each}
			</select>
			{#if bankLedgerError}
				<p class="mt-1 text-xs text-text-muted" data-testid="onboarding-bank-ledger-error">
					{bankLedgerError}
				</p>
			{/if}
		</div>
		<!-- Story 34-1 (AC-7) : le choix n'est pas définitif (décision de Guy). -->
		<p class="text-sm text-text-muted" data-testid="onboarding-bank-editable-later">
			{msg(
				'onboarding-bank-editable-later',
				'La description et le compte comptable lié se modifient ensuite dans Administration → Comptes bancaires.'
			)}
		</p>
		<div class="flex gap-3">
			<Button type="button" variant="outline" onclick={handleSkipBank} disabled={onboardingState.loading} class="flex-1">
				{msg('onboarding-skip-bank', 'Configurer plus tard')}
			</Button>
			<Button type="submit" disabled={onboardingState.loading} class="flex-1">
				{msg('onboarding-save', 'Enregistrer')}
			</Button>
		</div>
	</form>
{/if}

{#if showRestoreLink}
	<!-- Story 31-1b (AC15) : lien secondaire, sous le choix de l'étape. -->
	<div class="mt-8 border-t border-border pt-4 text-center text-sm text-text-muted">
		<p>{msg('onboarding-restore-hint', 'Vous avez une sauvegarde de Kesh (.keshbackup) ?')}</p>
		<a
			href="/onboarding/restore"
			class="font-medium text-primary underline hover:no-underline"
			data-testid="onboarding-restore-link"
		>
			{msg('onboarding-restore-link', 'Restaurer une sauvegarde')}
		</a>
	</div>
{/if}
