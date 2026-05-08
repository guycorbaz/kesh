<!--
  Story 8-5a-zero — Form pour lier/délier un bank_account au plan comptable.

  Filtre client-side classe 1/2 actifs Asset|Liability (cohérent avec
  validation serveur qui rejette Revenue/Expense). Le bouton submit est
  désactivé si la sélection courante est identique à la valeur initiale
  (no-op KF-004 cohérent — court-circuit serveur-side aussi).
-->
<script lang="ts">
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { isApiError } from '$lib/shared/utils/api-client';
	import { notifySuccess } from '$lib/shared/utils/notify';
	import type { AccountResponse } from '$lib/features/accounts/accounts.types';
	import {
		updateBankAccountJournalLink,
		type BankAccountSummary,
	} from './bank-accounts.api';

	interface Props {
		bankAccount: BankAccountSummary;
		accounts: AccountResponse[];
		onSuccess: (updated: BankAccountSummary) => void;
		onCancel: () => void;
	}

	let { bankAccount, accounts, onSuccess, onCancel }: Props = $props();

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

	// Filtre client-side : Asset|Liability ET (number startsWith '1' ou '2')
	// — pattern §validation-account-type + §frontend-flow Pass 3 Opus L1'''.
	const eligibleAccounts = $derived(
		accounts
			.filter(
				(a) =>
					a.active &&
					(a.accountType === 'Asset' || a.accountType === 'Liability') &&
					(a.number.startsWith('1') || a.number.startsWith('2')),
			)
			.sort((a, b) => a.number.localeCompare(b.number)),
	);

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
			if (isApiError(err)) {
				errorMessage = err.message;
			} else {
				errorMessage = String(err);
			}
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
			if (isApiError(err)) {
				errorMessage = err.message;
			} else {
				errorMessage = String(err);
			}
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
		<div class="text-sm font-medium text-text">
			{bankAccount.bankName} — {bankAccount.iban}
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
			{#each eligibleAccounts as acc (acc.id)}
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
		{#if bankAccount.journalAccountId !== null}
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
