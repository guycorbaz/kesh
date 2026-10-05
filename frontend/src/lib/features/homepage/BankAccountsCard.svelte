<script lang="ts">
	// Story 25-6-a (#389) — la tuile « Comptes bancaires » dit ce qu'est son
	// chiffre : le SOLDE COMPTABLE (le grand livre), pas l'argent en banque. Le
	// dernier relevé importé et l'écart, corrigé des dates de valeur côté
	// serveur, le confrontent à la banque.
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { shortIban, formatChfBalance } from '$lib/features/bank-accounts/format';
	import { formatSwissDate } from '$lib/features/reports/reports.api';
	import type { BankAccountSummary } from '$lib/features/bank-accounts/bank-accounts.api';
	import { ledgerTotal, statementGap } from './homepage';

	interface Props {
		state?: 'ready' | 'error';
		accounts: BankAccountSummary[];
	}
	let { state = 'ready', accounts }: Props = $props();

	let total = $derived(ledgerTotal(accounts));
</script>

<div class="rounded-lg border border-border bg-white p-6 shadow-sm" data-testid="homepage-card-bank-accounts">
	<h2 class="text-lg font-semibold text-text">
		{i18nMsg('homepage-bank-title', 'Comptes bancaires')}
	</h2>
	{#if state === 'error'}
		<p class="mt-2 text-sm text-text-muted" data-testid="homepage-bank-unavailable">
			{i18nMsg('homepage-bank-unavailable', 'Comptes bancaires indisponibles pour le moment.')}
		</p>
	{:else}
	<p class="mt-1 text-xs text-text-muted">
		{i18nMsg('homepage-bank-ledger-help', 'Calculé depuis le grand livre, pas depuis la banque.')}
	</p>
	<div class="mt-3 flex flex-col gap-3">
		{#each accounts as account (account.id)}
			{@const gap = statementGap(account)}
			<div class="flex items-start justify-between border-b border-border pb-2 last:border-b-0 last:pb-0">
				<div class="min-w-0 flex-1">
					<p class="font-medium">{account.bankName}</p>
					<p class="font-mono text-xs text-text-muted">{shortIban(account.iban)}</p>
				</div>
				<div class="text-right">
					{#if account.currentBalance !== null}
						<p class="text-xs text-text-muted">
							{i18nMsg('homepage-bank-ledger-balance', 'Solde comptable')}
						</p>
						<p class="font-mono text-sm" data-testid="homepage-bank-balance-{account.id}">
							{formatChfBalance(account.currentBalance)}
						</p>
						{#if account.lastTransactionDate}
							<p class="text-xs text-text-muted" data-testid="homepage-bank-last-tx-{account.id}">
								{i18nMsg('homepage-bank-last-transaction', 'Dernière transaction')} : {formatSwissDate(
									account.lastTransactionDate,
								)}
							</p>
						{/if}
					{:else}
						<p class="text-xs text-text-muted" data-testid="homepage-bank-balance-unavailable-{account.id}">
							<a class="underline" href="/bank-accounts">
								{i18nMsg('homepage-bank-balance-unavailable', 'Solde non disponible — lier au plan comptable')}
							</a>
						</p>
					{/if}
					{#if account.statementClosingBalance !== null && account.statementDate !== null}
						<p class="text-xs text-text-muted" data-testid="homepage-bank-statement-{account.id}">
							{i18nMsg('homepage-bank-statement', 'Relevé du { $date } : { $amount }', {
								date: formatSwissDate(account.statementDate),
								amount: formatChfBalance(account.statementClosingBalance),
							})}
						</p>
					{/if}
					{#if gap === null && account.statementClosingBalance !== null && account.currentBalance !== null && account.ledgerBalanceAtStatement === null}
						<!-- Compte du grand livre partagé : l'écart ne s'attribue à aucun compte. -->
						<p class="text-xs text-text-muted" data-testid="homepage-bank-gap-unavailable-{account.id}">
							{i18nMsg(
								'homepage-bank-gap-unavailable',
								'Écart non calculable : compte du grand livre partagé',
							)}
						</p>
					{/if}
					{#if gap !== null}
						<p class="text-xs font-semibold text-red-700" data-testid="homepage-bank-gap-{account.id}">
							{i18nMsg('homepage-bank-gap', 'Écart : { $amount }', {
								amount: formatChfBalance(gap),
							})}
						</p>
					{/if}
				</div>
			</div>
		{/each}
		{#if total.any}
			<div class="mt-1 flex items-center justify-between border-t border-border pt-2">
				<p class="text-sm font-semibold">
					{i18nMsg('homepage-bank-total-ledger', 'Total (solde comptable)')}
					{#if total.partial}
						<span class="ml-1 text-xs font-normal text-text-muted" data-testid="homepage-bank-total-partial">
							{i18nMsg('homepage-bank-total-partial', '(comptes liés uniquement)')}
						</span>
					{/if}
					{#if total.shared}
						<span class="ml-1 text-xs font-normal text-text-muted" data-testid="homepage-bank-total-shared">
							{i18nMsg(
								'homepage-bank-shared-ledger-note',
								"(un compte du grand livre partagé n'est compté qu'une fois)",
							)}
						</span>
					{/if}
				</p>
				<p class="font-mono text-sm font-semibold" data-testid="homepage-bank-total">
					{formatChfBalance(total.total)}
				</p>
			</div>
		{/if}
	</div>
	{/if}
</div>
