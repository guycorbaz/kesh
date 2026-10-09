<!--
  Doublure de test (Story 15-6b, test 18) — hôte de `SettleInvoiceDialog` qui lui passe ses props
  UNE À UNE, comme un vrai parent, et ne change que `accounts` (bouton `host-load-accounts`).

  ⚠️ Pourquoi pas `rerender` : celui de @testing-library/svelte remplace l'objet de props entier
  (`$state.raw`), si bien que CHAQUE prop paraît changée et que l'effet de réinitialisation du
  dialogue se rejoue — ce qu'un parent réel ne provoque pas. Le test serait rouge sur un
  composant juste. Le suffixe `.test.` tient ce fichier hors des relevés i18n et hors du build.
-->
<script lang="ts">
	import SettleInvoiceDialog from './SettleInvoiceDialog.svelte';
	import type { AccountResponse } from '$lib/features/accounts/accounts.types';
	import type { BankAccountSummary } from '$lib/features/bank-accounts/bank-accounts.api';

	type Props = {
		lateAccounts: AccountResponse[];
		bankAccounts: BankAccountSummary[];
	};
	let { lateAccounts, bankAccounts }: Props = $props();
	let accounts = $state<AccountResponse[]>([]);
</script>

<button type="button" data-testid="host-load-accounts" onclick={() => (accounts = lateAccounts)}>
	charger
</button>
<SettleInvoiceDialog
	open={true}
	onOpenChange={() => {}}
	invoiceDate="2026-01-01"
	amountDue="100.00"
	{accounts}
	{bankAccounts}
	onConfirm={() => {}}
/>
