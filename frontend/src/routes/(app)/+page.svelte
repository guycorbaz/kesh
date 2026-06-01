<script lang="ts">
	import { onMount } from 'svelte';
	import { Button } from '$lib/components/ui/button';
	import { modeState } from '$lib/app/stores/mode.svelte';
	import { i18nMsg } from '$lib/features/onboarding/onboarding.svelte';
	import { listBankAccounts, type BankAccountSummary } from '$lib/features/bank-accounts/bank-accounts.api';

	function msg(key: string, fallback: string): string {
		return i18nMsg(key, fallback);
	}

	// Story v014-1 (F5 Pass 1) — basculer vers listBankAccounts() qui retourne
	// le payload étendu avec `currentBalance` calculé serveur-side.
	let bankAccounts = $state<BankAccountSummary[]>([]);
	let bankLoaded = $state(false);

	onMount(async () => {
		try {
			bankAccounts = await listBankAccounts(false);
		} catch {
			// API not reachable — graceful degradation (widget caché si vide).
		} finally {
			bankLoaded = true;
		}
	});

	let isGuided = $derived(modeState.value === 'guided');

	function shortIban(iban: string): string {
		if (iban.length < 4) return iban;
		return `••••${iban.slice(-4)}`;
	}

	function formatBalance(balance: number | null): string {
		if (balance === null) return '—';
		return new Intl.NumberFormat('fr-CH', {
			style: 'currency',
			currency: 'CHF',
			minimumFractionDigits: 2,
		}).format(balance);
	}

	let totalLiquidity = $derived(
		bankAccounts.reduce((acc, ba) => acc + (ba.currentBalance ?? 0), 0),
	);
	let hasAnyBalance = $derived(bankAccounts.some((ba) => ba.currentBalance !== null));
</script>

<svelte:head>
	<title>Accueil - Kesh</title>
</svelte:head>

<h1 class="mb-6 text-2xl font-semibold text-text">
	{msg('homepage-title', 'Tableau de bord')}
</h1>

<div class="grid grid-cols-1 gap-6 lg:grid-cols-3">
	<!-- Widget : Dernières écritures -->
	<div class="rounded-lg border border-border bg-white p-6 shadow-sm" data-testid="homepage-card-recent-entries">
		<h2 class="text-lg font-semibold text-text">
			{msg('homepage-entries-title', 'Dernières écritures')}
		</h2>
		{#if isGuided}
			<p class="mt-2 text-sm text-text-muted">
				{msg('homepage-entries-empty-guided', 'Aucune écriture pour le moment. Commencez par saisir votre première écriture comptable.')}
			</p>
		{:else}
			<p class="mt-2 text-sm text-text-muted">
				{msg('homepage-entries-empty', 'Aucune écriture.')}
			</p>
		{/if}
		<Button variant="outline" class="mt-4" href="/journal-entries">
			{msg('homepage-entries-action', 'Saisir une écriture')}
		</Button>
	</div>

	<!-- Widget : Factures ouvertes -->
	<div class="rounded-lg border border-border bg-white p-6 shadow-sm" data-testid="homepage-card-open-invoices">
		<h2 class="text-lg font-semibold text-text">
			{msg('homepage-invoices-title', 'Factures ouvertes')}
		</h2>
		{#if isGuided}
			<p class="mt-2 text-sm text-text-muted">
				{msg('homepage-invoices-empty-guided', 'Aucune facture ouverte. Créez votre première facture pour facturer vos clients.')}
			</p>
		{:else}
			<p class="mt-2 text-sm text-text-muted">
				{msg('homepage-invoices-empty', 'Aucune facture ouverte.')}
			</p>
		{/if}
		<Button variant="outline" class="mt-4" href="/invoices">
			{msg('homepage-invoices-action', 'Créer une facture')}
		</Button>
	</div>

	<!-- Story v014-1 (AC#29) — widget Comptes bancaires retiré si aucun compte.
	     Affiche les soldes calculés sinon (currentBalance peut être null si
	     journal_account_id non-configuré → indication UX + lien). -->
	{#if bankLoaded && bankAccounts.length > 0}
		<div class="rounded-lg border border-border bg-white p-6 shadow-sm" data-testid="homepage-card-bank-accounts">
			<h2 class="text-lg font-semibold text-text">
				{msg('homepage-bank-title', 'Comptes bancaires')}
			</h2>
			<div class="mt-3 flex flex-col gap-3">
				{#each bankAccounts as account (account.id)}
					<div class="flex items-center justify-between border-b border-border pb-2 last:border-b-0 last:pb-0">
						<div class="min-w-0 flex-1">
							<p class="font-medium">{account.bankName}</p>
							<p class="font-mono text-xs text-text-muted">{shortIban(account.iban)}</p>
						</div>
						<div class="text-right">
							{#if account.currentBalance !== null}
								<p class="font-mono text-sm" data-testid="homepage-bank-balance-{account.id}">
									{formatBalance(account.currentBalance)}
								</p>
							{:else}
								<p class="text-xs text-text-muted" data-testid="homepage-bank-balance-unavailable-{account.id}">
									<a class="underline" href="/bank-accounts">
										{msg('homepage-bank-balance-unavailable', 'Solde non disponible — lier au plan comptable')}
									</a>
								</p>
							{/if}
						</div>
					</div>
				{/each}
				{#if hasAnyBalance && bankAccounts.length > 1}
					<div class="mt-1 flex items-center justify-between border-t border-border pt-2">
						<p class="text-sm font-semibold">
							{msg('homepage-bank-total-liquidity', 'Total liquidités')}
						</p>
						<p class="font-mono text-sm font-semibold" data-testid="homepage-bank-total-liquidity">
							{formatBalance(totalLiquidity)}
						</p>
					</div>
				{/if}
			</div>
		</div>
	{/if}
</div>
