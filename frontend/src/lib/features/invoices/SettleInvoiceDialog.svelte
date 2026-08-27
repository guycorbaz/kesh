<!--
  Story 24-3 (#372) — Dialogue « Enregistrer un règlement ».

  ⛔ REMPLACE `MarkPaidDialog` (Story 5.4), qui ne demandait qu'une date parce
  que le marquage n'écrivait rien. Un règlement PRODUIT son écriture : il lui
  faut donc une contrepartie et un montant.

  ⚠️ Le mode de règlement est indifférent au traitement comptable — espèces,
  poste, compensation, virement : seule change la contrepartie. Le dialogue les
  présente donc sur le même plan, sans hiérarchie.

  Émet `onConfirm({ settlementType, bankAccountId, accountId, amount, settledOn })` ;
  le parent gère l'appel API et ses erreurs.
-->
<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import * as Dialog from '$lib/components/ui/dialog';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import type { AccountResponse } from '$lib/features/accounts/accounts.types';
	import type { BankAccountSummary } from '$lib/features/bank-accounts/bank-accounts.api';

	export type SettlementPayload = {
		settlementType: 'bank_transfer' | 'internal_account';
		bankAccountId?: number;
		accountId?: number;
		amount: string;
		settledOn: string;
	};

	type Props = {
		open: boolean;
		onOpenChange: (v: boolean) => void;
		/** Date facture (YYYY-MM-DD) — borne basse de `settledOn`. */
		invoiceDate: string;
		/** Résiduel, qui pré-remplit le montant. `null` = non calculé. */
		amountDue: string | null;
		accounts: AccountResponse[];
		bankAccounts: BankAccountSummary[];
		submitting?: boolean;
		errorMsg?: string;
		onConfirm: (payload: SettlementPayload) => void;
	};

	let {
		open,
		onOpenChange,
		invoiceDate,
		amountDue,
		accounts,
		bankAccounts,
		submitting = false,
		errorMsg = '',
		onConfirm,
	}: Props = $props();

	function todayIso(): string {
		return new Date().toISOString().slice(0, 10);
	}

	let settlementType = $state<'bank_transfer' | 'internal_account'>('bank_transfer');
	let bankAccountId = $state<number | null>(null);
	let accountId = $state<number | null>(null);
	let amount = $state('');
	let settledOn = $state(todayIso());

	// Réinitialisation à l'ouverture — sans quoi le dialogue rouvrirait sur la
	// saisie précédente, ce qui est particulièrement trompeur pour un montant.
	$effect(() => {
		if (open) {
			settledOn = todayIso();
			amount = amountDue ?? '';
			settlementType = 'bank_transfer';
			bankAccountId = bankAccounts.find((b) => b.isPrimary)?.id ?? bankAccounts[0]?.id ?? null;
			accountId = null;
		}
	});

	// ⚠️ Seuls les comptes ACTIFS : le backend refuse un compte archivé, et
	// proposer ce qu'il refusera est une erreur qu'on fait commettre.
	let selectableAccounts = $derived(accounts.filter((a) => a.active));

	let clientError = $derived.by(() => {
		if (!settledOn) {
			return i18nMsg('invoice-error-settled-on-required', 'Date de règlement obligatoire');
		}
		// Pas de borne haute : `settledOn` est une date de valeur, qui peut être
		// future (ordre programmé, décalage week-end ou jour férié).
		if (invoiceDate && settledOn < invoiceDate.slice(0, 10)) {
			return i18nMsg(
				'invoice-error-settled-on-before-invoice-date',
				'La date de règlement ne peut être antérieure à la date de facture',
			);
		}
		const n = Number(amount);
		if (!amount || !Number.isFinite(n) || n <= 0) {
			return i18nMsg('invoice-error-amount-positive', 'Le montant doit être supérieur à zéro');
		}
		if (amountDue !== null && n > Number(amountDue)) {
			return i18nMsg(
				'invoice-error-amount-over-due',
				'Le montant dépasse ce qui reste dû sur cette facture',
			);
		}
		if (settlementType === 'bank_transfer' && bankAccountId === null) {
			return i18nMsg('invoice-error-bank-account-required', 'Choisissez un compte bancaire');
		}
		if (settlementType === 'internal_account' && accountId === null) {
			return i18nMsg('invoice-error-account-required', 'Choisissez un compte');
		}
		return '';
	});

	function handleConfirm() {
		if (clientError) return;
		onConfirm({
			settlementType,
			bankAccountId: settlementType === 'bank_transfer' ? (bankAccountId ?? undefined) : undefined,
			accountId: settlementType === 'internal_account' ? (accountId ?? undefined) : undefined,
			amount,
			settledOn,
		});
	}
</script>

<Dialog.Root {open} {onOpenChange}>
	<Dialog.Content>
		<Dialog.Header>
			<Dialog.Title>
				{i18nMsg('invoice-settle-dialog-title', 'Enregistrer un règlement')}
			</Dialog.Title>
		</Dialog.Header>
		<p class="text-sm">
			{i18nMsg(
				'invoice-settle-dialog-body',
				'Le règlement produit son écriture comptable, quel que soit le mode.',
			)}
		</p>

		<div class="mt-2">
			<label class="mb-1 block text-xs text-text-muted" for="settle-type">
				{i18nMsg('invoice-settle-type-label', 'Mode de règlement')}
			</label>
			<select
				id="settle-type"
				class="w-full rounded border border-border px-2 py-1 text-sm"
				bind:value={settlementType}
				data-testid="settle-type"
			>
				<option value="bank_transfer">
					{i18nMsg('invoice-settle-type-bank', 'Virement bancaire')}
				</option>
				<option value="internal_account">
					{i18nMsg('invoice-settle-type-internal', 'Espèces ou autre compte')}
				</option>
			</select>
		</div>

		{#if settlementType === 'bank_transfer'}
			<div class="mt-2">
				<label class="mb-1 block text-xs text-text-muted" for="settle-bank">
					{i18nMsg('invoice-settle-bank-label', 'Compte bancaire')}
				</label>
				<select
					id="settle-bank"
					class="w-full rounded border border-border px-2 py-1 text-sm"
					bind:value={bankAccountId}
					data-testid="settle-bank"
				>
					{#each bankAccounts as b (b.id)}
						<option value={b.id}>{b.bankName} — {b.iban}</option>
					{/each}
				</select>
			</div>
		{:else}
			<div class="mt-2">
				<label class="mb-1 block text-xs text-text-muted" for="settle-account">
					{i18nMsg('invoice-settle-account-label', 'Compte')}
				</label>
				<select
					id="settle-account"
					class="w-full rounded border border-border px-2 py-1 text-sm"
					bind:value={accountId}
					data-testid="settle-account"
				>
					<option value={null}>
						{i18nMsg('invoice-settle-account-placeholder', '— Choisir un compte')}
					</option>
					{#each selectableAccounts as a (a.id)}
						<option value={a.id}>{a.number} — {a.name}</option>
					{/each}
				</select>
			</div>
		{/if}

		<div class="mt-2 grid grid-cols-2 gap-2">
			<div>
				<label class="mb-1 block text-xs text-text-muted" for="settle-amount">
					{i18nMsg('invoice-settle-amount-label', 'Montant')}
				</label>
				<Input id="settle-amount" type="text" inputmode="decimal" bind:value={amount} />
			</div>
			<div>
				<label class="mb-1 block text-xs text-text-muted" for="settle-date">
					{i18nMsg('invoice-settle-date-label', 'Date de règlement')}
				</label>
				<Input id="settle-date" type="date" bind:value={settledOn} min={invoiceDate} />
			</div>
		</div>

		{#if clientError}
			<div
				class="rounded-md border border-destructive bg-destructive/10 px-3 py-2 text-sm text-destructive"
			>
				{clientError}
			</div>
		{:else if errorMsg}
			<div
				class="rounded-md border border-destructive bg-destructive/10 px-3 py-2 text-sm text-destructive"
			>
				{errorMsg}
			</div>
		{/if}

		<Dialog.Footer>
			<Button variant="outline" onclick={() => onOpenChange(false)} disabled={submitting}>
				{i18nMsg('common-cancel', 'Annuler')}
			</Button>
			<Button onclick={handleConfirm} disabled={submitting || !!clientError} data-testid="settle-confirm">
				{i18nMsg('invoice-settle-confirm', 'Enregistrer le règlement')}
			</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>
