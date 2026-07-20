<script lang="ts">
	import { onMount } from 'svelte';
	import { Button } from '$lib/components/ui/button';
	import { modeState } from '$lib/app/stores/mode.svelte';
	import { i18nMsg } from '$lib/features/onboarding/onboarding.svelte';
	import { listBankAccounts, type BankAccountSummary } from '$lib/features/bank-accounts/bank-accounts.api';
	import { shortIban, formatChfBalance } from '$lib/features/bank-accounts/format';
	import { authState } from '$lib/app/stores/auth.svelte';
	import { listReminders } from '$lib/features/reminders/reminders.api';

	function msg(key: string, fallback: string, args?: Record<string, string | number>): string {
		return i18nMsg(key, fallback, args);
	}

	// Story 21-6c (D-c2) — compteur « à rappeler » dans le widget Factures ouvertes.
	// `/dunning/reminders` est Comptable+ : un Consultation prendrait un 403 → ne
	// PAS fetcher pour ce rôle. Le compteur est un NOMBRE DE FACTURES (somme des
	// `invoices.length` des groupes), jamais un montant (L21-8 — aucun cumul).
	let canManage = $derived(
		authState.currentUser?.role === 'Admin' || authState.currentUser?.role === 'Comptable',
	);
	let reminderCount = $state(0);
	let reminderLoaded = $state(false);

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

		// Compteur « à rappeler » : seulement pour Comptable+ (RBAC endpoint).
		if (canManage) {
			try {
				const res = await listReminders();
				reminderCount = res.groups.reduce((n, g) => n + g.invoices.length, 0);
			} catch {
				// Widget silencieux : un échec ne pollue pas l'accueil (pas de toast).
			} finally {
				reminderLoaded = true;
			}
		}
	});

	let isGuided = $derived(modeState.value === 'guided');

	function formatBalance(balance: number | null): string {
		if (balance === null) return '—';
		return formatChfBalance(balance);
	}

	// F9 Pass 1 code review : afficher le total inconditionnellement (même avec
	// 1 compte) si au moins un solde est calculé, avec note si certains
	// comptes ont currentBalance=null (somme partielle).
	let accountsWithBalance = $derived(bankAccounts.filter((ba) => ba.currentBalance !== null));
	let totalLiquidity = $derived(
		accountsWithBalance.reduce((acc, ba) => acc + (ba.currentBalance ?? 0), 0),
	);
	let hasAnyBalance = $derived(accountsWithBalance.length > 0);
	let totalIsPartial = $derived(
		accountsWithBalance.length > 0 && accountsWithBalance.length < bankAccounts.length,
	);
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
		{#if canManage && reminderLoaded && reminderCount > 0}
			<!-- Story 21-6c (D-c2) : N factures à rappeler → lien vers la page Rappels. -->
			<p class="mt-2 text-sm">
				<a class="font-medium text-primary underline" href="/invoices/reminders">
					<span data-testid="homepage-reminders-count">
						{msg('homepage-reminders-count', '{ $n } facture(s) à rappeler', {
							n: reminderCount,
						})}
					</span>
				</a>
			</p>
		{:else if isGuided}
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
								{#if account.lastTransactionDate}
									<p class="text-xs text-text-muted" data-testid="homepage-bank-last-tx-{account.id}">
										{msg('homepage-bank-last-transaction', 'Dernière transaction')} : {account.lastTransactionDate}
									</p>
								{/if}
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
				{#if hasAnyBalance}
					<div class="mt-1 flex items-center justify-between border-t border-border pt-2">
						<p class="text-sm font-semibold">
							{msg('homepage-bank-total-liquidity', 'Total liquidités')}
							{#if totalIsPartial}
								<span class="ml-1 text-xs font-normal text-text-muted" data-testid="homepage-bank-total-partial">
									{msg('homepage-bank-total-partial', '(comptes liés uniquement)')}
								</span>
							{/if}
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
