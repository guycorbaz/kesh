<!--
  Story 8-5a-base FR45 — Modal réconciliation manuelle.

  Permet à un Comptable de réconcilier une bank_transaction `pending`
  sans candidate auto-matchée en sélectionnant un compte de
  contrepartie + description + valueDate optionnel. Le compte ledger
  banque est résolu **serveur-side** via `bank_account.journal_account_id`
  (foundation 8-5a-zero) — donc PAS de field `bankLedgerAccountId`
  dans le body API.

  Pré-filtrage client-side classes 5/6/7 (Pass 5 patch + Pass 4 Sonnet
  pattern — pas de wrapper dédié, on filtre en amont) avant passage à
  `AccountAutocomplete`.

  Gestion erreur 412 BANK_ACCOUNT_NOT_CONFIGURED : affiche un message
  + lien vers `/bank-accounts` pour configurer le journal_account_id
  manquant.
-->
<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import * as Dialog from '$lib/components/ui/dialog';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import AccountAutocomplete from '$lib/features/journal-entries/AccountAutocomplete.svelte';
	import type { AccountResponse } from '$lib/features/accounts/accounts.types';
	import type { ReconciliationProposal } from './reconciliation.types';
	import { manualMatchTransaction } from './reconciliation.api';

	type Props = {
		open: boolean;
		onOpenChange: (v: boolean) => void;
		bankAccountId: number;
		proposal: ReconciliationProposal | null;
		/** Pré-chargé par le parent (pas de fetch dans la modal pour
		 *  éviter une double-requête à chaque ouverture). */
		accounts: AccountResponse[];
		onSuccess: () => void;
	};

	let { open, onOpenChange, bankAccountId, proposal, accounts, onSuccess }: Props = $props();

	const MAX_DESCRIPTION_LEN = 200;

	let counterpartyId = $state<number | null>(null);
	let description = $state('');
	let valueDate = $state<string>('');
	let submitting = $state(false);
	let errorMsg = $state<string>('');
	let bankNotConfigured = $state(false);

	// Pré-filtrage client-side classes 5/6/7 (Pass 5 patch — pattern
	// compatible AccountAutocomplete sans wrapper).
	const filteredAccounts = $derived(
		accounts.filter((a) =>
			['5', '6', '7'].some((c) => a.number.startsWith(c)),
		),
	);

	// Reset à l'ouverture + pré-remplissage valueDate depuis tx.
	$effect(() => {
		if (open && proposal) {
			counterpartyId = null;
			description = '';
			errorMsg = '';
			bankNotConfigured = false;
			// Pré-remplir avec value_date s'il existe sinon booking_date.
			// L'API renvoie `transaction.bookingDate` (camelCase). value_date
			// n'est pas exposé dans ReconciliationProposal v0.1 — fallback
			// booking_date qui couvre 95% des cas.
			valueDate = proposal.transaction.bookingDate;
		}
	});

	const clientError = $derived.by(() => {
		if (!proposal) return i18nMsg('reconciliation-manual-error-no-proposal', 'Aucune transaction sélectionnée');
		if (counterpartyId === null) {
			return i18nMsg(
				'reconciliation-manual-error-counterparty-required',
				'Compte de contrepartie obligatoire',
			);
		}
		if (description.length > MAX_DESCRIPTION_LEN) {
			return i18nMsg(
				'reconciliation-manual-error-description-too-long',
				`Description trop longue (max ${MAX_DESCRIPTION_LEN} caractères)`,
			);
		}
		return '';
	});

	async function handleConfirm() {
		if (clientError || !proposal || counterpartyId === null) return;
		submitting = true;
		errorMsg = '';
		bankNotConfigured = false;
		try {
			await manualMatchTransaction(
				bankAccountId,
				proposal.bankTransactionId,
				counterpartyId,
				description || undefined,
				valueDate || undefined,
			);
			onSuccess();
			onOpenChange(false);
		} catch (e) {
			const msg = e instanceof Error ? e.message : String(e);
			// Detect 412 BANK_ACCOUNT_NOT_CONFIGURED via message string
			// (apiClient throw une Error stringifiée incluant le code).
			if (msg.includes('BANK_ACCOUNT_NOT_CONFIGURED')) {
				bankNotConfigured = true;
			} else {
				errorMsg = msg;
			}
		} finally {
			submitting = false;
		}
	}
</script>

<Dialog.Root {open} {onOpenChange}>
	<Dialog.Content data-testid="manual-match-modal">
		<Dialog.Header>
			<Dialog.Title>
				{i18nMsg('reconciliation-manual-modal-title', 'Réconciliation manuelle')}
			</Dialog.Title>
		</Dialog.Header>

		{#if proposal}
			<div class="text-sm text-text-muted">
				<span data-testid="manual-match-tx-summary">
					{proposal.transaction.bookingDate} — {proposal.transaction.amount}
					{proposal.transaction.currency}
				</span>
			</div>

			<div class="mt-2">
				<label class="mb-1 block text-xs text-text-muted" for="manual-match-counterparty">
					{i18nMsg('reconciliation-manual-counterparty-label', 'Compte de contrepartie')}
				</label>
				<AccountAutocomplete
					accounts={filteredAccounts}
					value={counterpartyId}
					onSelect={(id) => (counterpartyId = id)}
					disabled={submitting}
				/>
			</div>

			<div class="mt-2">
				<label class="mb-1 block text-xs text-text-muted" for="manual-match-description">
					{i18nMsg('reconciliation-manual-description-label', 'Description')}
				</label>
				<Input
					id="manual-match-description"
					data-testid="manual-match-description-input"
					bind:value={description}
					maxlength={MAX_DESCRIPTION_LEN}
					placeholder={i18nMsg(
						'reconciliation-manual-description-placeholder',
						'Frais bancaires mai',
					)}
					disabled={submitting}
				/>
			</div>

			<div class="mt-2">
				<label class="mb-1 block text-xs text-text-muted" for="manual-match-value-date">
					{i18nMsg('reconciliation-manual-value-date-label', 'Date de valeur')}
				</label>
				<Input
					id="manual-match-value-date"
					data-testid="manual-match-value-date-input"
					type="date"
					bind:value={valueDate}
					disabled={submitting}
				/>
			</div>

			{#if bankNotConfigured}
				<div
					class="rounded-md border border-destructive bg-destructive/10 px-3 py-2 text-sm text-destructive"
					data-testid="manual-match-bank-not-configured"
				>
					{i18nMsg(
						'reconciliation-manual-bank-account-not-configured',
						"Le compte bancaire n'est pas configuré. Configurer le compte comptable lié dans /bank-accounts.",
					)}
					<a class="underline" href="/bank-accounts">/bank-accounts</a>
				</div>
			{:else if clientError}
				<div class="rounded-md border border-destructive bg-destructive/10 px-3 py-2 text-sm text-destructive">
					{clientError}
				</div>
			{:else if errorMsg}
				<div
					class="rounded-md border border-destructive bg-destructive/10 px-3 py-2 text-sm text-destructive"
					data-testid="manual-match-error"
				>
					{errorMsg}
				</div>
			{/if}
		{/if}

		<Dialog.Footer>
			<Button
				variant="outline"
				onclick={() => onOpenChange(false)}
				disabled={submitting}
				data-testid="manual-match-cancel"
			>
				{i18nMsg('common-cancel', 'Annuler')}
			</Button>
			<Button
				onclick={handleConfirm}
				disabled={submitting || !!clientError}
				data-testid="manual-match-submit"
			>
				{i18nMsg('reconciliation-manual-submit', 'Affecter')}
			</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>
