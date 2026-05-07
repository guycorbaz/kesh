<!--
  Story 8-5a-zero — Liste des bank_accounts avec leur compte comptable lié.
  Click sur « Lier » / « Délier » ouvre `BankAccountJournalLinkForm` inline.
-->
<script lang="ts">
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import type { AccountResponse } from '$lib/features/accounts/accounts.types';
	import BankAccountJournalLinkForm from './BankAccountJournalLinkForm.svelte';
	import type { BankAccountSummary } from './bank-accounts.api';

	interface Props {
		bankAccounts: BankAccountSummary[];
		accounts: AccountResponse[];
		onUpdated: (updated: BankAccountSummary) => void;
	}

	let { bankAccounts, accounts, onUpdated }: Props = $props();

	let editingId = $state<number | null>(null);

	function accountLabel(journalAccountId: number | null): string {
		if (journalAccountId === null) {
			return i18nMsg('bank-accounts-labels-not-configured', 'Non configuré');
		}
		const acc = accounts.find((a) => a.id === journalAccountId);
		if (!acc) return `#${journalAccountId}`;
		return `${acc.number} — ${acc.name}`;
	}

	function handleSuccess(updated: BankAccountSummary) {
		editingId = null;
		onUpdated(updated);
	}
</script>

{#if bankAccounts.length === 0}
	<p class="text-text-muted" data-testid="bank-accounts-empty">
		{i18nMsg('bank-accounts-labels-empty', 'Aucun compte bancaire configuré.')}
	</p>
{:else}
	<table class="w-full text-sm" data-testid="bank-accounts-list">
		<thead>
			<tr class="border-b border-border text-left">
				<th class="py-2 pr-4 font-semibold"
					>{i18nMsg('bank-accounts-labels-bank-name', 'Banque')}</th
				>
				<th class="py-2 pr-4 font-semibold"
					>{i18nMsg('bank-accounts-labels-iban', 'IBAN')}</th
				>
				<th class="py-2 pr-4 font-semibold"
					>{i18nMsg(
						'bank-accounts-labels-journal-account-id',
						'Compte comptable lié',
					)}</th
				>
				<th class="py-2"></th>
			</tr>
		</thead>
		<tbody>
			{#each bankAccounts as ba (ba.id)}
				<tr class="border-b border-border" data-testid="bank-account-row-{ba.id}">
					<td class="py-2 pr-4">{ba.bankName}</td>
					<td class="py-2 pr-4 font-mono text-xs">{ba.iban}</td>
					<td class="py-2 pr-4" data-testid="journal-account-cell-{ba.id}">
						{accountLabel(ba.journalAccountId)}
					</td>
					<td class="py-2">
						{#if editingId !== ba.id}
							<button
								type="button"
								class="rounded border border-border px-3 py-1 text-xs"
								onclick={() => (editingId = ba.id)}
								data-testid="link-button-{ba.id}"
							>
								{i18nMsg(
									'bank-accounts-actions-link-account',
									'Lier au plan comptable',
								)}
							</button>
						{/if}
					</td>
				</tr>
				{#if editingId === ba.id}
					<tr>
						<td colspan="4" class="py-3">
							<BankAccountJournalLinkForm
								bankAccount={ba}
								{accounts}
								onSuccess={handleSuccess}
								onCancel={() => (editingId = null)}
							/>
						</td>
					</tr>
				{/if}
			{/each}
		</tbody>
	</table>
{/if}
