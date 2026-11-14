<!--
  Doublure de test (Story 35-7, AC3) — hôte de `ManualMatchModal` ou de `TransactionSplitModal`
  qui leur passe leurs props UNE À UNE, comme `ReconciliationProposals`, et ne change que
  `bankLedgerAccountId` (bouton `host-resolve-bank`) : l'identifiant du compte de la banque qui
  arrive après que l'utilisateur a choisi une contrepartie (#1000, chemin 1).

  ⚠️ Pourquoi pas `rerender` : celui de @testing-library/svelte remplace l'objet de props entier
  (`$state.raw`), si bien que CHAQUE prop paraît changée et que l'effet de réinitialisation du
  dialogue (`open && proposal`) se rejoue et vide la sélection — ce qu'un parent réel ne provoque
  pas. Le test passerait à vide (même piège que `SettleInvoiceDialogHost.test.svelte`).
-->
<script lang="ts">
	import ManualMatchModal from './ManualMatchModal.svelte';
	import TransactionSplitModal from './TransactionSplitModal.svelte';
	import type { AccountResponse } from '$lib/features/accounts/accounts.types';
	import type { ReconciliationProposal } from './reconciliation.types';

	type Props = {
		modal: 'manual' | 'split';
		proposal: ReconciliationProposal;
		accounts: AccountResponse[];
		/** L'identifiant que le bouton `host-resolve-bank` pose. */
		lateBankLedgerAccountId: number;
	};
	let { modal, proposal, accounts, lateBankLedgerAccountId }: Props = $props();
	let bankLedgerAccountId = $state<number | null>(null);
</script>

<button
	type="button"
	data-testid="host-resolve-bank"
	onclick={() => (bankLedgerAccountId = lateBankLedgerAccountId)}
>
	résoudre
</button>
{#if modal === 'manual'}
	<ManualMatchModal
		open={true}
		onOpenChange={() => {}}
		bankAccountId={17}
		{proposal}
		{accounts}
		{bankLedgerAccountId}
		onSuccess={() => {}}
	/>
{:else}
	<TransactionSplitModal
		open={true}
		onOpenChange={() => {}}
		bankAccountId={17}
		{proposal}
		{accounts}
		{bankLedgerAccountId}
		onSuccess={() => {}}
	/>
{/if}
