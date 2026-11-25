<script lang="ts">
	// Story 37-4 (AC-4) — la devise et le solde en devise d'un compte du bilan ou du
	// compte de résultat, « USD 1'233.57 », à côté du nom. Rien pour un compte en
	// devise principale : un rapport mono-devise ne change pas. Les totaux du
	// rapport restent en devise principale.
	// Story 46-2 (AC-4) — sert aussi la ligne d'un groupe dont toute la descendance
	// est dans une même devise (champs `currency` / `balanceCurrency` de la 46-1) ;
	// `testIdPrefix` distingue alors la ligne du groupe de celle d'un compte.
	import { formatCurrencyAmount } from './reports.api';
	import type { AccountBalance } from './reports.types';

	interface Props {
		account: Pick<AccountBalance, 'accountNumber' | 'currency' | 'balanceCurrency'>;
		testIdPrefix?: string;
	}
	let { account, testIdPrefix = 'account-fx' }: Props = $props();
</script>

{#if account.currency}
	<span
		class="ml-2 font-mono text-xs text-gray-600"
		data-testid="{testIdPrefix}-{account.accountNumber}"
		>{formatCurrencyAmount(account.currency, account.balanceCurrency)}</span
	>
{/if}
