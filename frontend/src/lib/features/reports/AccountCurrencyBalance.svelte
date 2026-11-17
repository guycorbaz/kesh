<script lang="ts">
	// Story 37-4 (AC-4) — la devise et le solde en devise d'un compte du bilan ou du
	// compte de résultat, « USD 1'233.57 », à côté du nom. Rien pour un compte en
	// devise principale : un rapport mono-devise ne change pas. Les totaux du
	// rapport restent en devise principale.
	import { formatCurrencyAmount } from './reports.api';
	import type { AccountBalance } from './reports.types';

	interface Props {
		account: AccountBalance;
	}
	let { account }: Props = $props();
</script>

{#if account.currency}
	<span
		class="ml-2 font-mono text-xs text-gray-600"
		data-testid="account-fx-{account.accountNumber}"
		>{formatCurrencyAmount(account.currency, account.balanceCurrency)}</span
	>
{/if}
