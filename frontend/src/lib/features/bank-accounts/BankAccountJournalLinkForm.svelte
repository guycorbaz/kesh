<!--
  Story 8-5a-zero — Form pour lier/délier un bank_account au plan comptable.

  Filtre client-side : comptes actifs, imputables, Asset|Liability, hors
  comptes de créance (`linkableLedgerAccounts`, Story 34-1 ; cohérent avec la
  validation serveur qui rejette Revenue/Expense). Le bouton submit est
  désactivé si la sélection courante est identique à la valeur initiale
  (no-op KF-004 cohérent — court-circuit serveur-side aussi).
-->
<script lang="ts">
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { errorMessageOf } from '$lib/shared/utils/error-message';
	import { notifySuccess } from '$lib/shared/utils/notify';
	import type { AccountResponse } from '$lib/features/accounts/accounts.types';
	import { withCurrentAccount } from '$lib/features/accounts/account-options';
	import {
		bankAccountOptionLabel,
		linkableLedgerAccounts,
	} from '$lib/features/bank-accounts/format';
	import { admitsLedgerType } from './credit-card';
	import {
		updateBankAccountJournalLink,
		type BankAccountSummary,
	} from './bank-accounts.api';

	interface Props {
		bankAccount: BankAccountSummary;
		accounts: AccountResponse[];
		/**
		 * Story 15-6c (#474, AC7) : le compte débiteurs et le compte créanciers
		 * désignés dans les réglages, écartés des options AVANT
		 * `withCurrentAccount`. Obligatoire : un appelant qui l'oublierait
		 * proposerait le lien que le serveur refuse. Ensemble vide = pas de filtre.
		 */
		claimAccountIds: Set<number>;
		onSuccess: (updated: BankAccountSummary) => void;
		onCancel: () => void;
	}

	let { bankAccount, accounts, claimAccountIds, onSuccess, onCancel }: Props = $props();

	// P-L1 Pass 1 code review Sonnet 4.6 : `selectedAccountId` synchronisé
	// avec la prop `bankAccount.journalAccountId` via `$effect` — dissipe le
	// warning Svelte `state_referenced_locally` et garantit la sync si le
	// parent re-binde la prop sans démonter (le composant est habituellement
	// démonté/remonté, mais le pattern est plus robuste). On initialise à
	// `null` puis on hydrate via `$effect` : Svelte ne capture pas la prop
	// au moment de l'init du `$state`, donc plus de warning.
	let selectedAccountId = $state<number | null>(null);
	$effect(() => {
		selectedAccountId = bankAccount.journalAccountId;
	});
	let submitting = $state(false);
	let errorMessage = $state<string | null>(null);

	// Story 15-6c (AC7) : sans les comptes de créance désignés.
	//
	// Story 34-1 (#966, C-e341-5, C-e3401-1) : le filtre est celui de
	// `linkableLedgerAccounts`, source unique des deux écrans. Le filtre propre à
	// ce formulaire sur le NUMÉRO (« commence par 1 ou 2 ») est retiré : aucun
	// sens ne se déduit d'un numéro (Story 14-3a), le serveur ne le contrôle pas,
	// et le formulaire de création de la même page ne l'appliquait pas — un
	// compte d'actif numéroté hors des classes 1 et 2 se liait à la création,
	// mais n'était pas proposé ici. Le tri par numéro est fait par la fonction
	// commune (revue P1, B-5).
	// Story 34-11a2 (#972, C-e341-33) — passifs seulement pour une carte.
	const eligibleAccounts = $derived(
		linkableLedgerAccounts(accounts, claimAccountIds).filter((a) =>
			admitsLedgerType(bankAccount.accountKind, a.accountType),
		),
	);

	// Issue #271 : un compte lié AVANT de devenir non-postable — ou d'être
	// archivé — disparaîtrait de ces options, laissant le champ vide alors que
	// la liaison existe, et se laisserait effacer au premier `change`.
	const accountOptions = $derived(withCurrentAccount(eligibleAccounts, selectedAccountId, accounts));

	// Désactiver submit si pas de changement (KF-004 no-op cohérent UX).
	const isNoOp = $derived(selectedAccountId === bankAccount.journalAccountId);

	async function handleSubmit(event: SubmitEvent) {
		event.preventDefault();
		if (submitting || isNoOp) return;
		submitting = true;
		errorMessage = null;
		try {
			const updated = await updateBankAccountJournalLink(
				bankAccount.id,
				selectedAccountId,
				bankAccount.version,
			);
			// P-M7 Pass 1 code review Sonnet 4.6 : toast succès câblé.
			notifySuccess(
				i18nMsg(
					'bank-accounts-toast-link-success',
					'Compte bancaire lié avec succès au plan comptable.',
				),
			);
			onSuccess(updated);
		} catch (err) {
			errorMessage = errorMessageOf(err);
		} finally {
			submitting = false;
		}
	}

	async function handleUnlink() {
		if (submitting) return;
		submitting = true;
		errorMessage = null;
		try {
			const updated = await updateBankAccountJournalLink(
				bankAccount.id,
				null,
				bankAccount.version,
			);
			// P-M7 Pass 1 code review Sonnet 4.6 : toast succès câblé.
			notifySuccess(
				i18nMsg(
					'bank-accounts-toast-unlink-success',
					'Compte bancaire délié du plan comptable.',
				),
			);
			onSuccess(updated);
		} catch (err) {
			errorMessage = errorMessageOf(err);
		} finally {
			submitting = false;
		}
	}
</script>

<form
	onsubmit={handleSubmit}
	class="rounded border border-border bg-background p-4"
	data-testid="bank-account-journal-link-form"
	aria-label={i18nMsg('bank-accounts-actions-link-account', 'Lier au plan comptable')}
>
	<div class="mb-3">
		<div class="text-sm font-medium text-text" data-testid="bank-account-journal-link-heading">
			{bankAccountOptionLabel(bankAccount)}
		</div>
	</div>

	<div class="mb-3">
		<label class="mb-1 block text-sm font-medium text-text" for="journal-account-select">
			{i18nMsg('bank-accounts-labels-journal-account-id', 'Compte comptable lié')}
		</label>
		<select
			id="journal-account-select"
			class="w-full rounded border border-border bg-background px-3 py-2 text-sm"
			bind:value={selectedAccountId}
			disabled={submitting}
			data-testid="journal-account-select"
		>
			<option value={null}
				>{i18nMsg('bank-accounts-labels-not-configured', 'Non configuré')}</option
			>
			{#each accountOptions as acc (acc.id)}
				<option value={acc.id}>
					{acc.number} — {acc.name}
				</option>
			{/each}
		</select>
	</div>

	{#if errorMessage}
		<p class="mb-3 text-sm text-red-600" role="alert" data-testid="form-error">
			{errorMessage}
		</p>
	{/if}

	<div class="flex gap-2">
		<button
			type="submit"
			class="rounded bg-primary px-4 py-2 text-sm font-medium text-primary-foreground disabled:opacity-50"
			disabled={submitting || isNoOp}
			data-testid="submit-link"
		>
			{i18nMsg('bank-accounts-actions-submit', 'Lier')}
		</button>
		<!-- Story 34-11a2 (C-e341-16) — une carte ne se délie pas (refus serveur). -->
		{#if bankAccount.journalAccountId !== null && bankAccount.accountKind !== 'credit_card'}
			<button
				type="button"
				class="rounded border border-border px-4 py-2 text-sm font-medium text-text disabled:opacity-50"
				onclick={handleUnlink}
				disabled={submitting}
				data-testid="unlink-button"
			>
				{i18nMsg('bank-accounts-actions-unlink-account', 'Délier')}
			</button>
		{/if}
		<button
			type="button"
			class="rounded border border-border px-4 py-2 text-sm font-medium text-text disabled:opacity-50"
			onclick={onCancel}
			disabled={submitting}
			data-testid="cancel-button"
		>
			{i18nMsg('bank-accounts-actions-cancel', 'Annuler')}
		</button>
	</div>
</form>
