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
	import { fetchCompanyCurrent } from '$lib/features/settings/settings.api';
	import { listFiscalYears } from '$lib/features/fiscal-years/fiscal-years.api';
	import { ONBOARDING_LANGUAGES, onboardingScreenFor } from '$lib/features/onboarding/onboarding-screens';
	import OnboardingUnloaded from '$lib/features/onboarding/OnboardingUnloaded.svelte';
	import OnboardingFinalizeScreen from '$lib/features/onboarding/OnboardingFinalizeScreen.svelte';
	import OnboardingDone, { type OnboardingSummary } from '$lib/features/onboarding/OnboardingDone.svelte';
	import OnboardingDemo from '$lib/features/onboarding/OnboardingDemo.svelte';
	import OnboardingFallback from '$lib/features/onboarding/OnboardingFallback.svelte';

	function msg(key: string, fallback: string): string {
		return i18nMsg(key, fallback);
	}

	// Story 36-17 (#1187) — les drapeaux locaux de la fin de l'assistant (fiche, D3, D7).
	//
	// `finalizing` est posé AVANT l'appel de l'étape bancaire et avant tout rappel de
	// `finalize()`, remis à faux dans le `finally` de la finalisation (ou à l'échec de
	// l'étape bancaire). Il ne se lit pas sur `onboardingState.loading` : entre la fin de
	// `skipBank()` (son `finally`) et le `_loading = true` de `finalize()`, l'état
	// « étape 7, chargé, au repos » existe, et l'écran de reprise s'y rendrait un instant.
	let finalizing = $state(false);
	/** L'échec de la dernière finalisation : son motif, et s'il appelle la phrase de remède. */
	let finalizeError = $state<{ motif: string; remedy: boolean } | null>(null);
	let demoLoading = $state(false);
	let demoMotif = $state<string | null>(null);
	let unloadedMotif = $state<string | null>(null);

	/** L'écran de l'état (fiche, D8) : chaque combinaison en a un. */
	const screen = $derived(
		onboardingScreenFor({
			loaded: onboardingState.loaded,
			stepCompleted: onboardingState.stepCompleted,
			isDemo: onboardingState.isDemo,
			finalizing,
			finalizeFailed: finalizeError !== null,
			demoLoading,
		})
	);

	// Story 31-1b (AC15, #503) — restaurer une sauvegarde au lieu de configurer :
	// aux étapes 0 à 2 seulement (avant le choix démo / production, qui crée des
	// données), et pour l'Administrateur seulement (le compte créé par /setup
	// l'est ; le serveur refuse de toute façon les autres rôles, 403). Story 36-17 :
	// pas pendant le chargement des données de démonstration.
	const showRestoreLink = $derived(
		onboardingState.loaded &&
			onboardingState.stepCompleted <= 2 &&
			!demoLoading &&
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

	// Story 36-17 (#1187, D7) : un indicateur pendant le chargement, puis une navigation
	// ATTENDUE ; si elle échoue, l'écran « démo prête » reste, avec son motif et
	// *Commencer*. Un refus du chargement lui-même reste un toast sur l'écran du choix.
	async function chooseDemoPath() {
		demoLoading = true;
		try {
			await onboardingState.seedDemo();
		} catch (err) {
			toast.error(
				errorMessageOf(err, msg('onboarding-demo-load-error', 'Erreur lors du chargement des données de démo'))
			);
			return;
		} finally {
			demoLoading = false;
		}
		try {
			await goto('/');
		} catch (err) {
			demoMotif = errorMessageOf(
				err,
				msg('onboarding-demo-navigate-error', "Kesh n'a pas pu s'ouvrir : cliquez sur Commencer.")
			);
		}
	}

	// Story 36-17 (#1187, D9) : *Réessayer* de l'écran « état non lu ».
	async function retryFetchState() {
		unloadedMotif = null;
		try {
			await onboardingState.fetchState();
		} catch (err) {
			// Le constat est déjà à l'écran : le motif ne le suit que s'il en dit plus.
			const constat = msg('onboarding-state-load-error', "L'état de la configuration n'a pas pu être lu.");
			const motif = errorMessageOf(err, constat);
			unloadedMotif = motif === constat ? null : motif;
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
	// Revue P1 (E-1) : en cas d'échec, la phrase de l'étape s'affiche TOUJOURS ;
	// le motif du client ou du serveur (« Impossible de contacter le serveur… »)
	// la suit s'il en dit davantage.
	let bankLedgerFailed = $state(false);
	let bankLedgerMotif = $state<string | null>(null);
	// Revue P1 (A-1, B-2) : tant que la liste se charge, l'étape ne s'envoie pas
	// (le corps partirait sans le lien que l'écran va proposer) ; et un choix fait
	// dans le sélecteur avant la fin du chargement n'est pas écrasé par la
	// présélection (drapeau non réactif, posé par `onchange`).
	let bankLedgerLoading = $state(false);
	let bankLedgerRequested = false;
	let bankLedgerTouched = false;

	function bankLedgerLoadErrorText(): string {
		return msg(
			'onboarding-bank-ledger-load-error',
			'La liste des comptes n’a pas pu être chargée : le compte bancaire peut être lié plus tard.'
		);
	}

	async function loadBankLedgerOptions() {
		bankLedgerLoading = true;
		try {
			const all = await fetchAccounts(false);
			const roleIds = new Set([
				...accountIdsWithRole(all, 'Receivable'),
				...accountIdsWithRole(all, 'Payable'),
			]);
			bankLedgerOptions = linkableLedgerAccounts(all, roleIds);
			if (!bankLedgerTouched) {
				bankJournalAccountId = preselectedBankLedgerId(bankLedgerOptions);
			}
		} catch (err) {
			// Liste indisponible : le compte se crée sans lien, et se lie ensuite
			// dans Administration → Comptes bancaires (AC-6). Le motif s'affiche
			// SOUS le champ, non en toast : l'étape reste utilisable (garde
			// `onboarding-catch-guard.test.ts` : tout `catch` passe par
			// `errorMessageOf`).
			bankLedgerOptions = [];
			bankJournalAccountId = null;
			bankLedgerFailed = true;
			const motif = errorMessageOf(err, bankLedgerLoadErrorText());
			bankLedgerMotif = motif === bankLedgerLoadErrorText() ? null : motif;
		} finally {
			bankLedgerLoading = false;
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
	//
	// Story 36-17 (#1187, D2, D5) : l'échec s'affiche À L'ÉCRAN de l'étape 7, avec
	// *Réessayer* — plus de toast passager sur une page vide. Sous `VALIDATION_ERROR` (le
	// seul refus de validation de la finalisation, reconnu au code et non au texte), la
	// phrase de remède précède le motif (limite connue, #1200). « Déjà finalisé » relit
	// l'état et montre son écran ; une relecture en échec s'affiche de même.
	async function reportFinalizeError(err: unknown) {
		if (isApiError(err) && err.code === 'ONBOARDING_STEP_ALREADY_COMPLETED') {
			try {
				await onboardingState.fetchState();
			} catch (relecture) {
				finalizeError = {
					motif: errorMessageOf(
						relecture,
						msg('onboarding-state-load-error', "L'état de la configuration n'a pas pu être lu.")
					),
					remedy: false,
				};
			}
			return;
		}
		finalizeError = {
			motif: errorMessageOf(err, msg('onboarding-finalize-error', 'Finalisation échouée — réessayez.')),
			remedy: isApiError(err) && err.code === 'VALIDATION_ERROR',
		};
	}

	// Story 36-17 (#1187, D1, D3) : la finalisation, après l'étape bancaire ou par
	// *Réessayer* / *Terminer la configuration*. Ordre du geste : effacer l'erreur, poser
	// `finalizing`, appeler — pendant une relance, « Configuration en cours… » s'affiche,
	// non l'écran d'échec. Aucune navigation d'office : l'étape 8 rend la page
	// « Configuration terminée », dont *Commencer* recharge l'application.
	async function runFinalize() {
		finalizeError = null;
		finalizing = true;
		try {
			await onboardingState.finalize();
		} catch (err) {
			await reportFinalizeError(err);
		} finally {
			finalizing = false;
		}
	}

	/** *Réessayer* et *Terminer la configuration* ne répondent pas pendant un appel (D3). */
	const finalizeDisabled = $derived(finalizing || onboardingState.loading);

	// Story 36-17 (#1187, D6) : le récapitulatif de la page terminée, lu une fois à son
	// affichage. Seuls le nom de la banque et la description des comptes bancaires sont
	// recopiés : aucun IBAN n'atteint l'écran.
	let summary = $state<OnboardingSummary | null>(null);
	let summaryError = $state<string | null>(null);
	let summaryRequested = false;

	async function loadSummary() {
		try {
			const [current, years] = await Promise.all([fetchCompanyCurrent(), listFiscalYears()]);
			summary = {
				companyName: current.company.name,
				orgType: current.company.orgType,
				accountingLanguage: current.company.accountingLanguage,
				fiscalYears: [...years]
					.sort((a, b) => a.startDate.localeCompare(b.startDate))
					.map((y) => ({ name: y.name, startDate: y.startDate, endDate: y.endDate })),
				bankAccounts: current.bankAccounts.map((b) => ({
					bankName: b.bankName,
					description: b.description,
				})),
			};
		} catch (err) {
			summaryError = errorMessageOf(
				err,
				msg('onboarding-done-summary-error', "Le récapitulatif n'a pas pu être lu.")
			);
		}
	}

	$effect(() => {
		if (screen === 'done' && !summaryRequested) {
			summaryRequested = true;
			void loadSummary();
		}
	});

	async function submitBankAccount() {
		// Revue P1 (B-2) : pas d'envoi avant la fin du chargement de la liste (le
		// bouton est désactivé ; ce garde couvre la touche Entrée).
		if (bankLedgerLoading || finalizing) return;
		if (!bankName.trim() || !bankIban.trim()) {
			toast.error(msg('onboarding-bank-required', 'Nom de banque et IBAN sont obligatoires'));
			return;
		}
		// Story 36-17 (D3) : posé AVANT l'étape bancaire, pour qu'aucun instant de l'étape 7
		// ne rende l'écran de reprise avant la finalisation.
		finalizing = true;
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
			finalizing = false;
			toast.error(
				errorMessageOf(err, msg('onboarding-bank-save-error', 'Erreur lors de la sauvegarde du compte bancaire'))
			);
			return;
		}
		await runFinalize();
	}

	async function handleSkipBank() {
		finalizing = true;
		try {
			await onboardingState.skipBank();
		} catch (err) {
			finalizing = false;
			toast.error(
				errorMessageOf(err, msg('onboarding-bank-skip-error', 'Erreur lors du saut du compte bancaire'))
			);
			return;
		}
		await runFinalize();
	}

	const cardClass = "rounded-lg border border-border bg-white p-6 text-left shadow-sm transition-colors hover:border-primary hover:bg-primary-light/5";
</script>

<!--
	Story 36-17 (#1187, D8) : un écran pour chaque état, choisi par `onboardingScreenFor`.
	Chaque écran porte `data-testid="onboarding-screen-<id>"` (les étapes 0 à 6 par une
	enveloppe, sans autre changement de leur contenu) ; le détecteur
	`onboarding-screens.test.ts` exige exactement un écran par état.
-->
{#if screen === 'unloaded'}
	<OnboardingUnloaded loading={onboardingState.loading} motif={unloadedMotif} onRetry={retryFetchState} />

{:else if screen === '0'}
	<!-- Step 1 : Langue -->
	<div data-testid="onboarding-screen-0">
	<div class="grid grid-cols-2 gap-4">
		{#each ONBOARDING_LANGUAGES as [code, label]}
			<button class={cardClass + " text-center text-lg font-medium"} onclick={() => chooseLanguage(code)} disabled={onboardingState.loading} data-testid={`onboarding-language-${code}`}>
				{label}
			</button>
		{/each}
	</div>
	</div>

{:else if screen === '1'}
	<!-- Step 2 : Mode -->
	<div data-testid="onboarding-screen-1">
	<h2 class="mb-6 text-center text-xl font-semibold">
		{msg('onboarding-choose-mode', 'Choisissez votre mode d\'utilisation')}
	</h2>
	<div class="flex flex-col gap-4">
		<button class={cardClass} onclick={() => chooseMode('guided')} disabled={onboardingState.loading} data-testid="onboarding-mode-guided">
			<div class="text-lg font-medium">{msg('onboarding-mode-guided', 'Guidé')}</div>
			<div class="mt-1 text-sm text-text-muted">{msg('onboarding-mode-guided-desc', 'Espacements généreux, aide contextuelle, confirmations avant actions')}</div>
		</button>
		<button class={cardClass} onclick={() => chooseMode('expert')} disabled={onboardingState.loading} data-testid="onboarding-mode-expert">
			<div class="text-lg font-medium">{msg('onboarding-mode-expert', 'Expert')}</div>
			<div class="mt-1 text-sm text-text-muted">{msg('onboarding-mode-expert-desc', 'Interface compacte, raccourcis clavier, actions directes')}</div>
		</button>
	</div>
	</div>

{:else if screen === '2'}
	<!-- Step 3 : Chemin A/B -->
	<div data-testid="onboarding-screen-2">
	<h2 class="mb-6 text-center text-xl font-semibold">
		{msg('onboarding-choose-path', 'Comment souhaitez-vous commencer ?')}
	</h2>
	<div class="flex flex-col gap-4">
		<button class={cardClass} onclick={chooseDemoPath} disabled={onboardingState.loading} data-testid="onboarding-path-demo">
			<div class="text-lg font-medium">{msg('onboarding-path-demo', 'Explorer avec des données de démo')}</div>
			<div class="mt-1 text-sm text-text-muted">{msg('onboarding-path-demo-desc', 'Découvrez Kesh avec des données fictives réalistes')}</div>
		</button>
		<button class={cardClass} onclick={chooseProductionPath} disabled={onboardingState.loading} data-testid="onboarding-path-production">
			<div class="text-lg font-medium">{msg('onboarding-path-production', 'Configurer pour la production')}</div>
			<div class="mt-1 text-sm text-text-muted">{msg('onboarding-path-production-desc', 'Configurez votre organisation pour commencer à travailler')}</div>
		</button>
	</div>
	</div>

{:else if screen === '3'}
	<!-- Step 4 : Type d'organisation (chemin production seulement : la démo à l'étape 3
	     a son propre écran, `demo-ready`) -->
	<div data-testid="onboarding-screen-3">
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
	</div>

{:else if screen === '4'}
	<!-- Step 5 : Langue comptable -->
	<div data-testid="onboarding-screen-4">
	<h2 class="mb-6 text-center text-xl font-semibold">
		{msg('onboarding-choose-accounting-lang', 'Langue comptable')}
	</h2>
	<p class="mb-4 text-center text-sm text-text-muted">
		{msg('onboarding-accounting-lang-desc', 'Langue des libellés du plan comptable (découplée de la langue de l\'interface)')}
	</p>
	<div class="grid grid-cols-2 gap-4">
		{#each ONBOARDING_LANGUAGES as [code, label]}
			<button class={cardClass + " text-center text-lg font-medium"} onclick={() => chooseAccountingLang(code)} disabled={onboardingState.loading} data-testid={`onboarding-accounting-lang-${code}`}>
				{label}
			</button>
		{/each}
	</div>
	</div>

{:else if screen === '5'}
	<!-- Step 6 : Coordonnées -->
	<div data-testid="onboarding-screen-5">
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
		<Button type="submit" disabled={onboardingState.loading} data-testid="onboarding-coordinates-submit">
			{msg('onboarding-next', 'Continuer')}
		</Button>
	</form>
	</div>

{:else if screen === '6'}
	<!-- Step 7 : Compte bancaire (reste affiché pendant l'appel de l'étape bancaire,
	     boutons désactivés, jusqu'au passage à l'étape 7 — Story 36-17, D3) -->
	<div data-testid="onboarding-screen-6">
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
				onchange={() => (bankLedgerTouched = true)}
				class="w-full rounded border border-border px-3 py-2 text-sm"
				data-testid="onboarding-bank-journal-account"
			>
				<option value={null}>{msg('bank-accounts-labels-not-configured', 'Non configuré')}</option>
				{#each bankLedgerOptions as acc (acc.id)}
					<option value={acc.id}>{acc.number} — {acc.name}</option>
				{/each}
			</select>
			{#if bankLedgerFailed}
				<p class="mt-1 text-xs text-text-muted" data-testid="onboarding-bank-ledger-error">
					{bankLedgerLoadErrorText()}
					{#if bankLedgerMotif}
						<span data-testid="onboarding-bank-ledger-motif">({bankLedgerMotif})</span>
					{/if}
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
			<Button type="button" variant="outline" onclick={handleSkipBank} disabled={onboardingState.loading || finalizing} class="flex-1" data-testid="onboarding-skip-bank">
				{msg('onboarding-skip-bank', 'Configurer plus tard')}
			</Button>
			<Button type="submit" disabled={onboardingState.loading || bankLedgerLoading || finalizing} class="flex-1" data-testid="onboarding-bank-save">
				{msg('onboarding-save', 'Enregistrer')}
			</Button>
		</div>
	</form>
	</div>

{:else if screen === 'finalizing' || screen === 'finalize-error' || screen === 'resume'}
	<OnboardingFinalizeScreen
		mode={screen}
		motif={finalizeError?.motif ?? null}
		remedy={finalizeError?.remedy ?? false}
		disabled={finalizeDisabled}
		onFinalize={runFinalize}
		resumeText={msg('incomplete-banner-text', "Configuration incomplète : terminez-la avant d'utiliser Kesh.")}
		resumeCta={msg('incomplete-banner-cta', 'Terminer la configuration')}
	/>

{:else if screen === 'done'}
	<OnboardingDone {summary} {summaryError} />

{:else if screen === 'demo-loading' || screen === 'demo-ready'}
	<OnboardingDemo mode={screen} motif={demoMotif} />

{:else}
	<OnboardingFallback />
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
