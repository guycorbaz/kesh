<script lang="ts">
	import { onMount } from 'svelte';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { isApiError } from '$lib/shared/utils/api-client';
	import { notifySuccess } from '$lib/shared/utils/notify';
	import { fetchAccounts } from '$lib/features/accounts/accounts.api';
	import { accountRoleKey, type AccountResponse } from '$lib/features/accounts/accounts.types';
	import {
		generateOpeningBalances,
		getOpeningBalancesStatus
	} from '$lib/features/opening-balances/opening-balances.api';
	import type { OpeningBalancesStatus } from '$lib/features/opening-balances/opening-balances.types';
	import {
		computeBalance,
		formatSwissAmount,
		isValidAmount
	} from '$lib/features/journal-entries/balance';

	// ------------------------------------------------------------------
	// État de chargement (P3-BH3-2) : le statut pilote grille-vs-verrou,
	// donc son échec n'est PAS tolérable — état d'erreur explicite + Réessayer,
	// jamais de grille par défaut.
	// ------------------------------------------------------------------
	let loading = $state(true);
	let statusError = $state(false);
	let status = $state<OpeningBalancesStatus | null>(null);
	let accounts = $state<AccountResponse[]>([]);

	// Grille : une ligne par compte de bilan, montants en string décimale.
	interface GridRow {
		account: AccountResponse;
		debit: string;
		credit: string;
	}
	let rows = $state<GridRow[]>([]);

	let submitting = $state(false);
	let submitError = $state<string | null>(null);

	async function load() {
		loading = true;
		statusError = false;
		const [statusResult, accountsResult] = await Promise.allSettled([
			getOpeningBalancesStatus(),
			fetchAccounts(false)
		]);

		// Tolérance de panne asymétrique : le statut est obligatoire (il
		// décide quoi afficher), les comptes seulement si la grille s'ouvre.
		if (statusResult.status === 'fulfilled') {
			status = statusResult.value;
		} else {
			status = null;
			statusError = true;
		}

		if (accountsResult.status === 'fulfilled') {
			accounts = accountsResult.value;
		} else {
			accounts = [];
			// Sans comptes, la grille serait vide et inutilisable : on traite
			// l'échec comme un échec de chargement global (même bouton Réessayer).
			if (status?.canEnter) {
				statusError = true;
			}
		}

		// Grille = comptes de bilan actifs ET postables (D4). Les capitaux
		// propres sont des comptes de type Liability dans les 3 plans.
		rows = accounts
			.filter(
				(a) =>
					a.active &&
					a.postable &&
					(a.accountType === 'Asset' || a.accountType === 'Liability')
			)
			.map((account) => ({ account, debit: '', credit: '' }));

		loading = false;
	}

	onMount(() => {
		void load();
	});

	// Bandeau de total en direct (D3) — big.js via balance.ts, jamais parseFloat.
	const balance = $derived(computeBalance(rows));

	// Lignes non vides envoyées au POST (les autres sont ignorées).
	const nonEmptyRows = $derived(rows.filter((r) => r.debit !== '' || r.credit !== ''));

	const canGenerate = $derived(!submitting && balance.isBalanced);

	/** Débit/Crédit mutuellement exclusifs par ligne : saisir l'un vide l'autre. */
	function onDebitInput(row: GridRow) {
		if (row.debit !== '') row.credit = '';
	}
	function onCreditInput(row: GridRow) {
		if (row.credit !== '') row.debit = '';
	}

	function roleLabel(account: AccountResponse): string {
		if (!account.role) return '';
		return i18nMsg(accountRoleKey(account.role), account.role);
	}

	async function handleGenerate() {
		if (!canGenerate) return;
		submitting = true;
		submitError = null;
		try {
			await generateOpeningBalances({
				lines: nonEmptyRows.map((r) => ({
					accountId: r.account.id,
					debit: r.debit === '' ? '0' : r.debit.replace(',', '.'),
					credit: r.credit === '' ? '0' : r.credit.replace(',', '.')
				}))
			});
			notifySuccess(i18nMsg('opening-balances-success', 'Écriture d’ouverture générée.'));
			// Comportement déterministe post-génération (P1-M2-BH) : recharger
			// le statut → l'écran repasse en état verrouillé ALREADY_HAS_ENTRIES
			// in-place (liens bilan + journal), pas de redirection.
			await load();
		} catch (err) {
			// Le serveur localise déjà tous les messages : afficher tel quel
			// (AC-E — aucun err.code n'est reformulé côté client).
			submitError = isApiError(err)
				? err.message
				: i18nMsg('opening-balances-status-error', 'Impossible de charger l’état des soldes de départ.');
		} finally {
			submitting = false;
		}
	}

	const formatNumber = formatSwissAmount;
</script>

<svelte:head>
	<title>{i18nMsg('opening-balances-title', 'Soldes de départ')} — Kesh</title>
</svelte:head>

<div class="mx-auto max-w-4xl space-y-6">
	<h1 class="text-2xl font-semibold">
		{i18nMsg('opening-balances-title', 'Soldes de départ')}
	</h1>

	{#if loading}
		<!-- (a) Chargement : jamais de grille tant que le statut n'est pas résolu. -->
		<p class="text-text-muted" data-testid="opening-balances-loading" role="status">…</p>
	{:else if statusError}
		<!-- (b) Échec du fetch statut : message + Réessayer, PAS de grille. -->
		<div
			class="rounded-lg border border-destructive bg-red-50 p-6 dark:bg-red-950/30"
			data-testid="opening-balances-status-error"
		>
			<p class="text-sm text-destructive">
				{i18nMsg('opening-balances-status-error', 'Impossible de charger l’état des soldes de départ.')}
			</p>
			<Button
				variant="outline"
				size="sm"
				class="mt-3"
				onclick={() => void load()}
				data-testid="opening-balances-retry"
			>
				{i18nMsg('opening-balances-retry', 'Réessayer')}
			</Button>
		</div>
	{:else if status && !status.canEnter}
		<!-- État verrouillé : message explicite selon reason (D6). Pas de grille. -->
		<div
			class="rounded-lg border border-border bg-surface p-6"
			data-testid="opening-balances-locked"
			data-reason={status.reason}
		>
			<p class="text-sm text-text">
				{#if status.reason === 'NO_FISCAL_YEAR'}
					{i18nMsg(
						'opening-balances-locked-no-fiscal-year',
						'Aucun exercice comptable : créez d’abord un exercice (Paramètres → Exercices) pour saisir vos soldes de départ.'
					)}
				{:else if status.reason === 'FIRST_YEAR_CLOSED'}
					{i18nMsg(
						'opening-balances-locked-first-year-closed',
						'Le premier exercice « { $name } » est clôturé : un administrateur doit le rouvrir avant la saisie des soldes de départ.',
						{ name: status.fiscalYear?.name ?? '' }
					)}
				{:else}
					{i18nMsg(
						'opening-balances-locked-already-has-entries',
						'La société contient déjà des écritures : le bilan d’ouverture est verrouillé. Corrigez l’écriture d’ouverture directement dans le journal, ou supprimez toutes les écritures pour recommencer.'
					)}
				{/if}
			</p>
			<div class="mt-4 flex gap-3">
				{#if status.reason === 'ALREADY_HAS_ENTRIES'}
					<Button
						variant="outline"
						size="sm"
						href="/reports"
						data-testid="opening-balances-goto-balance-sheet"
					>
						{i18nMsg('opening-balances-goto-balance-sheet', 'Voir le bilan')}
					</Button>
					<Button
						variant="outline"
						size="sm"
						href="/journal-entries"
						data-testid="opening-balances-goto-journal"
					>
						{i18nMsg('opening-balances-goto-journal', 'Ouvrir le journal')}
					</Button>
				{:else if status.reason === 'NO_FISCAL_YEAR' || status.reason === 'FIRST_YEAR_CLOSED'}
					<Button variant="outline" size="sm" href="/settings/fiscal-years">
						{i18nMsg('nav-fiscal-years', 'Exercices comptables')}
					</Button>
				{/if}
			</div>
		</div>
	{:else if status && status.canEnter}
		<!-- Grille de saisie (statut READY). -->
		<p class="text-sm text-text-muted" data-testid="opening-balances-intro">
			{i18nMsg(
				'opening-balances-intro',
				'Saisissez les soldes de vos comptes de bilan repris de votre ancienne comptabilité. Une écriture d’ouverture équilibrée sera générée au { $date } (premier jour de l’exercice « { $name } »). Posez votre report à-nouveau accumulé sur votre compte de report pour équilibrer l’écriture.',
				{
					date: status.fiscalYear?.startDate ?? '',
					name: status.fiscalYear?.name ?? ''
				}
			)}
		</p>

		<table class="w-full border-collapse text-sm" data-testid="opening-balances-grid">
			<thead>
				<tr class="border-b border-border text-left text-xs uppercase tracking-wider text-text-muted">
					<th class="py-2 pr-2">{i18nMsg('opening-balances-account', 'Compte')}</th>
					<th class="w-40 py-2 pr-2 text-right">{i18nMsg('opening-balances-debit', 'Débit')}</th>
					<th class="w-40 py-2 text-right">{i18nMsg('opening-balances-credit', 'Crédit')}</th>
				</tr>
			</thead>
			<tbody>
				{#each rows as row (row.account.id)}
					<tr
						class="border-b border-border/50"
						data-testid="opening-balances-row-{row.account.number}"
					>
						<td class="py-1.5 pr-2">
							<span class="tabular-nums font-medium">{row.account.number}</span>
							<span class="ml-2">{row.account.name}</span>
							{#if row.account.role}
								<span
									class="ml-2 rounded bg-primary-light/20 px-1.5 py-0.5 text-xs text-primary"
									data-testid="opening-balances-row-{row.account.number}-role-badge"
								>
									{roleLabel(row.account)}
								</span>
							{/if}
						</td>
						<td class="py-1.5 pr-2">
							<Input
								type="text"
								inputmode="decimal"
								class="text-right tabular-nums"
								bind:value={row.debit}
								oninput={() => onDebitInput(row)}
								aria-invalid={!isValidAmount(row.debit)}
								data-testid="opening-balances-debit-{row.account.number}"
							/>
						</td>
						<td class="py-1.5">
							<Input
								type="text"
								inputmode="decimal"
								class="text-right tabular-nums"
								bind:value={row.credit}
								oninput={() => onCreditInput(row)}
								aria-invalid={!isValidAmount(row.credit)}
								data-testid="opening-balances-credit-{row.account.number}"
							/>
						</td>
					</tr>
				{/each}
			</tbody>
		</table>

		<!-- Bandeau de total en direct (D3), miroir JournalEntryForm. -->
		<div
			class="flex items-center justify-between rounded-md border p-4 tabular-nums {balance.isBalanced
				? 'border-green-600 bg-green-50 dark:bg-green-950/30'
				: balance.totalDebit.gt(0) || balance.totalCredit.gt(0)
					? 'border-destructive bg-red-50 dark:bg-red-950/30'
					: 'border-border'}"
			data-testid="opening-balances-totals"
		>
			<div class="space-x-4 text-sm">
				<span>
					<strong>{i18nMsg('opening-balances-total-debit', 'Total débits')} :</strong>
					<span data-testid="opening-balances-total-debit">{formatNumber(balance.totalDebit)}</span>
				</span>
				<span>
					<strong>{i18nMsg('opening-balances-total-credit', 'Total crédits')} :</strong>
					<span data-testid="opening-balances-total-credit">{formatNumber(balance.totalCredit)}</span>
				</span>
				<span>
					<strong>{i18nMsg('opening-balances-diff', 'Différence')} :</strong>
					<span data-testid="opening-balances-diff">{formatNumber(balance.diff)}</span>
				</span>
			</div>
			<div class="text-sm font-medium">
				{#if balance.isBalanced}
					<span class="text-green-700 dark:text-green-400">
						✓ {i18nMsg('journal-entry-form-balanced', 'Équilibré')}
					</span>
				{:else if balance.totalDebit.gt(0) || balance.totalCredit.gt(0)}
					<span class="text-destructive">
						✗ {i18nMsg('journal-entry-form-unbalanced', 'Déséquilibré')}
					</span>
				{/if}
			</div>
		</div>

		{#if submitError}
			<p class="text-sm text-destructive" data-testid="opening-balances-submit-error" role="alert">
				{submitError}
			</p>
		{/if}

		<div class="flex justify-end">
			<Button
				onclick={handleGenerate}
				disabled={!canGenerate}
				data-testid="opening-balances-generate"
			>
				{submitting
					? i18nMsg('opening-balances-generating', 'Génération…')
					: i18nMsg('opening-balances-generate', 'Générer l’écriture d’ouverture')}
			</Button>
		</div>
	{/if}
</div>
