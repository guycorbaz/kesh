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
		completeOpeningBalances,
		generateOpeningBalances,
		getOpeningBalancesStatus
	} from '$lib/features/opening-balances/opening-balances.api';
	import type {
		CompletableAccount,
		OpeningBalancesStatus
	} from '$lib/features/opening-balances/opening-balances.types';
	import {
		complementCounterpart,
		computeOpeningTotals,
		retainedEarningsWarning
	} from '$lib/features/opening-balances/opening-balances-totals';
	import {
		computeBalance,
		formatSwissAmount,
		isValidAmount,
		parseAmount
	} from '$lib/features/journal-entries/balance';
	import { formatSwissDate } from '$lib/features/reports/reports.api';
	import type Big from 'big.js';

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

	// Mode « compléter » (Story 25-7, AC 6) : une ligne par compte complétable
	// que rend le STATUS — la liste n'est jamais recalculée côté client.
	interface ComplementRow {
		account: CompletableAccount;
		debit: string;
		credit: string;
	}
	let complementRows = $state<ComplementRow[]>([]);
	let completing = $state(false);
	let completeError = $state<string | null>(null);

	// Jeton de génération (Pass 3 review, ECH3-1) : rend la course
	// last-writer-wins impossible PAR CONSTRUCTION — un load() périmé (lancé
	// avant, résolu après) ne peut plus écraser l'état posé par un load() plus
	// récent (patron leçon 21-6b : fermer la fenêtre, pas ajouter un garde).
	let loadGen = 0;

	async function load() {
		const gen = ++loadGen;
		loading = true;
		statusError = false;
		const [statusResult, accountsResult] = await Promise.allSettled([
			getOpeningBalancesStatus(),
			fetchAccounts(false)
		]);
		// Réponse périmée : un load() plus récent a pris la main — aucune
		// écriture d'état.
		if (gen !== loadGen) return;

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
		// Au rechargement (après un refus métier), la saisie des comptes ENCORE
		// proposés est conservée : seul le compte fautif disparaît (revue de code
		// P3, F-1 — reconstruire à vide faisait tout ressaisir).
		const previous = new Map(complementRows.map((r) => [r.account.id, r]));
		// Un rechargement du status qui ÉCHOUE ne touche pas à la saisie : elle
		// survit à « Réessayer » (revue de code P4, R4-4).
		if (statusResult.status === 'fulfilled') {
			complementRows = (status?.completableAccounts ?? []).map((account) => ({
				account,
				debit: previous.get(account.id)?.debit ?? '',
				credit: previous.get(account.id)?.credit ?? ''
			}));
		}

		loading = false;
	}

	onMount(() => {
		void load();
	});

	// Bandeau de total en direct (D3) — big.js via balance.ts, jamais parseFloat.
	const balance = $derived(computeBalance(rows));

	// Lignes non vides envoyées au POST (les autres sont ignorées). Une ligne
	// dont les deux côtés valent 0 — vide OU « 0 »/« 0.00 » tapé explicitement —
	// est traitée comme vide et N'EST PAS envoyée (Pass 2 review, ECH2-2) :
	// le serveur la rejetterait en `EntryLineDebitCreditExclusive` (XOR strict).
	// Même cas que la classification « partial » de `JournalEntryForm`, mais
	// remède différent (Pass 3 AA-LOW-3) : le formulaire d'écriture BLOQUE le
	// submit, ici la ligne nulle est simplement retirée du payload — plus doux,
	// sans risque (elle n'apporte rien à l'écriture). Les montants invalides
	// restent inclus par prudence — `balance.hasInvalidAmount` fait retomber
	// `isBalanced` (balance.ts:65) et désactive le bouton en amont.
	const nonEmptyRows = $derived(
		rows.filter((r) => {
			if (r.debit === '' && r.credit === '') return false;
			if (!isValidAmount(r.debit) || !isValidAmount(r.credit)) return true;
			return parseAmount(r.debit).gt(0) || parseAmount(r.credit).gt(0);
		})
	);

	const canGenerate = $derived(!submitting && balance.isBalanced);

	// Totaux actif / passif et montant à porter (AC 2), avertissement du report
	// à-nouveau (AC 1) — il signale, il ne bloque pas : `canGenerate` l'ignore.
	const totals = $derived(computeOpeningTotals(rows));
	const warning = $derived(retainedEarningsWarning(accounts, rows, balance.isBalanced));

	// Complément : lignes saisies, contrepartie en direct (arbitrage 3).
	const complementLines = $derived(
		complementRows.filter((r) => {
			if (r.debit === '' && r.credit === '') return false;
			if (!isValidAmount(r.debit) || !isValidAmount(r.credit)) return true;
			return parseAmount(r.debit).gt(0) || parseAmount(r.credit).gt(0);
		})
	);
	const complementInvalid = $derived(
		complementRows.some((r) => !isValidAmount(r.debit) || !isValidAmount(r.credit))
	);
	const counterpart = $derived(complementCounterpart(complementLines));
	const canSubmitComplement = $derived(
		!completing && complementLines.length > 0 && !complementInvalid
	);

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
			// Course perdue (409 : un concurrent a rendu la company non-vierge) :
			// recharger le statut pour verrouiller l'écran in-place, comme le
			// chemin succès — sinon la grille reste active et chaque nouvel
			// essai re-échoue à l'identique (Pass 3 review, BH3-LOW).
			if (isApiError(err) && err.code === 'ILLEGAL_STATE_TRANSITION') {
				await load();
			}
		} finally {
			submitting = false;
		}
	}

	async function handleComplete() {
		if (!canSubmitComplement) return;
		// La date est dans la confirmation : elle peut avoir changé depuis la
		// saisie (verrou de période posé, rechargement après un refus — revue de
		// code P4, R4-2), et c'est le dernier écran avant l'écriture.
		const ok = window.confirm(
			i18nMsg(
				'opening-balances-complete-confirm',
				'Enregistrer cette écriture de complément, datée du { $date } ? Elle reste modifiable depuis sa fiche tant que l’exercice est ouvert — sauf si l’une de ses lignes est lettrée : délettrez-la d’abord.',
				{ date: formatSwissDate(status?.complementDate ?? '') }
			)
		);
		if (!ok) return;
		completing = true;
		completeError = null;
		try {
			await completeOpeningBalances({
				lines: complementLines.map((r) => ({
					accountId: r.account.id,
					debit: r.debit === '' ? '0' : r.debit.replace(',', '.'),
					credit: r.credit === '' ? '0' : r.credit.replace(',', '.')
				}))
			});
			notifySuccess(i18nMsg('opening-balances-complete-success', 'Complément enregistré.'));
			// Le compte complété disparaît de la liste : c'est le status qui le dit.
			await load();
		} catch (err) {
			// Le serveur localise ses refus (codes OPENING_COMPLEMENT_*) : tel quel.
			completeError = isApiError(err)
				? err.message
				: i18nMsg('opening-balances-complete-error', 'Le complément n’a pas pu être enregistré. Réessayez.');
			// Un refus métier (compte mouvementé entre-temps, date, report…) rend
			// la liste périmée : recharger le status, sinon chaque nouvel essai
			// échoue à l'identique (revue de code P1, E-F1). Le message, posé
			// avant, survit au rechargement — il est affiché hors de la grille.
			if (
				isApiError(err) &&
				err.code.startsWith('OPENING_COMPLEMENT_') &&
				!SHAPE_REFUSALS.has(err.code)
			) {
				await load();
			}
		} finally {
			completing = false;
		}
	}

	/**
	 * Refus de FORME du handler : la liste n'est pas périmée, la saisie se
	 * corrige — on ne recharge pas, ce qui la viderait (revue de code P2, R-L2).
	 */
	const SHAPE_REFUSALS = new Set([
		'OPENING_COMPLEMENT_NO_LINES',
		'OPENING_COMPLEMENT_TOO_MANY_LINES',
		'OPENING_COMPLEMENT_INVALID_AMOUNT',
		'OPENING_COMPLEMENT_DUPLICATE_ACCOUNT'
	]);

	/** Message du mode « compléter » indisponible, par `completeReason`. */
	function unavailableMessage(reason: string): string {
		switch (reason) {
			case 'NO_ENTRIES':
				return i18nMsg(
					'opening-balances-complete-unavailable-no-entries',
					'La société n’a encore aucune écriture : utilisez la génération du bilan d’ouverture.'
				);
			case 'NO_OPEN_FISCAL_YEAR':
				return i18nMsg(
					'opening-balances-complete-unavailable-no-open-fiscal-year',
					'Aucun exercice ouvert ne couvre la date du jour : le complément ne peut pas être daté.'
				);
			case 'DATE_LOCKED':
				return i18nMsg(
					'opening-balances-complete-unavailable-date-locked',
					'La date du jour tombe dans la période verrouillée : le complément ne peut pas être daté.'
				);
			case 'NO_RETAINED_EARNINGS':
				return i18nMsg(
					'opening-balances-complete-unavailable-no-retained-earnings',
					'Aucun compte en service ne porte le rôle « Bénéfice/perte reporté » : attribuez-le dans le plan comptable pour compléter un compte oublié.'
				);
			case 'RETAINED_EARNINGS_NOT_POSTABLE':
				return i18nMsg(
					'opening-balances-complete-unavailable-retained-earnings-not-postable',
					'Le compte de bénéfice reporté n’est pas imputable : rendez-le imputable pour compléter un compte oublié.'
				);
			case 'NO_COMPLETABLE_ACCOUNT':
				return i18nMsg(
					'opening-balances-complete-unavailable-no-completable-account',
					'Tous les comptes de bilan proposables — en service, imputables, hors compte de report — ont déjà des mouvements : il ne reste aucun compte à compléter. Un montant faux se corrige dans le journal.'
				);
			default:
				return i18nMsg('opening-balances-status-error', 'Impossible de charger l’état des soldes de départ.');
		}
	}

	const formatNumber = formatSwissAmount;
	/** Deux décimales, ou quatre si le montant en porte davantage (saisie jusqu'à 4). */
	function formatExact(amount: Big): string {
		return formatSwissAmount(amount, amount.round(2).eq(amount) ? 2 : 4);
	}
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
			<!-- Anti double-clic (Pass 4 BH4-LOW) : la protection vient du
			     DÉMONTAGE synchrone de cette branche (`loading = true` → Svelte 5
			     flushe → le bouton disparaît avant qu'un 2e clic ne porte), pas
			     d'un attribut disabled — inerte ici car la branche d'erreur est
			     mutuellement exclusive avec l'état loading. `loadGen` reste la
			     défense en profondeur pour tout futur appelant concurrent. -->
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
						'Le premier exercice « { $name } » est clôturé : avant la saisie des soldes de départ, un administrateur doit rouvrir les exercices clôturés jusqu’à celui-ci, en commençant par le plus récent.',
						{ name: status.fiscalYear?.name ?? '' }
					)}
				{:else if status.reason === 'ALREADY_HAS_ENTRIES'}
					{i18nMsg(
						'opening-balances-locked-already-has-entries',
						'La société contient déjà des écritures : le bilan d’ouverture a été généré et ne se régénère plus. Un compte de bilan oublié se complète ci-dessous ; un montant faux sur un compte déjà saisi se corrige dans le journal, en modifiant l’écriture d’ouverture tant que l’exercice est ouvert (après l’avoir délettrée si l’une de ses lignes est lettrée), ou par une contre-passation ou une écriture de correction.'
					)}
				{:else}
					<!-- reason inconnue (skew de version, évolution future) : pas de
					     message trompeur — état neutre invitant à recharger (Pass 3
					     review, ECH3-LOW : l'ancien fourre-tout affichait le message
					     ALREADY_HAS_ENTRIES pour toute reason inconnue). -->
					{i18nMsg('opening-balances-status-error', 'Impossible de charger l’état des soldes de départ.')}
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

		{#if status.reason === 'ALREADY_HAS_ENTRIES'}
			<!-- Mode « compléter » (Story 25-7, AC 6) : seulement sous ce bandeau-là —
			     sous NO_FISCAL_YEAR / FIRST_YEAR_CLOSED, il conseillerait une
			     génération impossible. -->
			{#if status.canComplete && status.retainedEarningsAccount}
				<section class="space-y-4" data-testid="opening-balances-complete">
					<h2 class="text-lg font-semibold">
						{i18nMsg('opening-balances-complete-title', 'Compléter un compte oublié')}
					</h2>
					<p class="text-sm text-text-muted">
						{i18nMsg(
							'opening-balances-complete-intro',
							'Saisissez le solde des seuls comptes de bilan oubliés à l’ouverture. Kesh porte la contrepartie sur le compte de report { $number } « { $name } ».',
							{
								number: status.retainedEarningsAccount.number,
								name: status.retainedEarningsAccount.name
							}
						)}
					</p>
					<p class="text-sm text-text-muted" data-testid="opening-balances-complete-date" data-kind={status.complementDateKind}>
						{#if status.complementDateKind === 'TODAY'}
							{i18nMsg(
								'opening-balances-complete-date-today',
								'L’écriture sera datée du { $date }, dans l’exercice « { $name } » : une régularisation, le premier jour de l’ouverture n’acceptant plus d’écriture (exercice clôturé ou période verrouillée).',
								{
									date: formatSwissDate(status.complementDate ?? ''),
									name: status.complementFiscalYear?.name ?? ''
								}
							)}
						{:else}
							{i18nMsg(
								'opening-balances-complete-date-opening',
								'L’écriture sera datée du { $date }, premier jour de l’exercice « { $name } » : le compte oublié faisait partie de l’ouverture.',
								{
									date: formatSwissDate(status.complementDate ?? ''),
									name: status.complementFiscalYear?.name ?? ''
								}
							)}
						{/if}
						{i18nMsg('opening-balances-complete-date-note', 'La date est confirmée à l’enregistrement.')}
					</p>
					<table class="w-full border-collapse text-sm">
						<thead>
							<tr class="border-b border-border text-left text-xs uppercase tracking-wider text-text-muted">
								<th scope="col" class="py-2 pr-2">{i18nMsg('opening-balances-account', 'Compte')}</th>
								<th scope="col" class="w-40 py-2 pr-2 text-right">{i18nMsg('opening-balances-debit', 'Débit')}</th>
								<th scope="col" class="w-40 py-2 text-right">{i18nMsg('opening-balances-credit', 'Crédit')}</th>
							</tr>
						</thead>
						<tbody>
							{#each complementRows as row (row.account.id)}
								<tr class="border-b border-border/50" data-testid="opening-balances-complete-row-{row.account.number}">
									<td class="py-1.5 pr-2">
										<span class="tabular-nums font-medium">{row.account.number}</span>
										<span class="ml-2">{row.account.name}</span>
									</td>
									<td class="py-1.5 pr-2">
										<Input
											type="text"
											inputmode="decimal"
											placeholder="0.00"
											class="text-right tabular-nums"
											bind:value={row.debit}
											oninput={() => {
												if (row.debit !== '') row.credit = '';
											}}
											aria-invalid={!isValidAmount(row.debit)}
											aria-label={i18nMsg('opening-balances-debit-for', 'Débit du compte { $number } « { $name } »', { number: row.account.number, name: row.account.name })}
											data-testid="opening-balances-complete-debit-{row.account.number}"
										/>
									</td>
									<td class="py-1.5">
										<Input
											type="text"
											inputmode="decimal"
											placeholder="0.00"
											class="text-right tabular-nums"
											bind:value={row.credit}
											oninput={() => {
												if (row.credit !== '') row.debit = '';
											}}
											aria-invalid={!isValidAmount(row.credit)}
											aria-label={i18nMsg('opening-balances-credit-for', 'Crédit du compte { $number } « { $name } »', { number: row.account.number, name: row.account.name })}
											data-testid="opening-balances-complete-credit-{row.account.number}"
										/>
									</td>
								</tr>
							{/each}
						</tbody>
					</table>
					<p class="text-sm" data-testid="opening-balances-complete-counterpart" data-side={counterpart.side}>
						{#if counterpart.side === 'credit'}
							{i18nMsg(
								'opening-balances-complete-counterpart-credit',
								'Contrepartie : { $amount } au crédit du compte { $number }.',
								{ amount: formatExact(counterpart.amount), number: status.retainedEarningsAccount.number }
							)}
						{:else if counterpart.side === 'debit'}
							{i18nMsg(
								'opening-balances-complete-counterpart-debit',
								'Contrepartie : { $amount } au débit du compte { $number }.',
								{ amount: formatExact(counterpart.amount), number: status.retainedEarningsAccount.number }
							)}
						{:else if complementLines.length > 0}
							{i18nMsg(
								'opening-balances-complete-counterpart-none',
								'Les lignes s’équilibrent entre elles : aucune contrepartie.'
							)}
						{/if}
					</p>
					<div class="flex justify-end">
						<Button
							onclick={handleComplete}
							disabled={!canSubmitComplement}
							data-testid="opening-balances-complete-submit"
						>
							{completing
								? i18nMsg('opening-balances-complete-submitting', 'Enregistrement…')
								: i18nMsg('opening-balances-complete-submit', 'Compléter')}
						</Button>
					</div>
				</section>
			{:else}
				<p
					class="text-sm text-text-muted"
					data-testid="opening-balances-complete-unavailable"
					data-reason={status.completeReason}
				>
					{unavailableMessage(status.completeReason)}
				</p>
			{/if}
			{#if completeError}
				<!-- Hors de la grille : il survit au rechargement qui suit un refus. -->
				<p class="text-sm text-destructive" data-testid="opening-balances-complete-error" role="alert">
					{completeError}
				</p>
			{/if}
		{/if}
	{:else if status && status.canEnter}
		<!-- Grille de saisie (statut READY). -->
		<p class="text-sm text-text-muted" data-testid="opening-balances-intro">
			{i18nMsg(
				'opening-balances-intro',
				'Saisissez les soldes de vos comptes de bilan repris de votre ancienne comptabilité. Une écriture d’ouverture équilibrée sera générée au { $date } (premier jour de l’exercice « { $name } »). Posez votre report à-nouveau accumulé sur votre compte de report pour équilibrer l’écriture.',
				{
					// Format suisse dd.mm.yyyy (Pass 3 review, BH3-LOW) — pas l'ISO brut.
					date: formatSwissDate(status.fiscalYear?.startDate ?? ''),
					name: status.fiscalYear?.name ?? ''
				}
			)}
		</p>

		{#if rows.length === 0}
			<!-- Plan comptable atypique : aucun compte de bilan actif+postable —
			     sans ce message, la grille serait un en-tête nu avec un bouton
			     durablement désactivé, sans explication (Pass 1 review, ECH-LOW). -->
			<p class="text-sm text-text-muted" data-testid="opening-balances-empty-grid">
				{i18nMsg(
					'opening-balances-empty-grid',
					'Aucun compte de bilan actif et postable dans votre plan comptable — créez ou réactivez vos comptes d’actifs et de passifs (Plan comptable) avant de saisir les soldes de départ.'
				)}
			</p>
		{:else}
		<table class="w-full border-collapse text-sm" data-testid="opening-balances-grid">
			<thead>
				<tr class="border-b border-border text-left text-xs uppercase tracking-wider text-text-muted">
					<th scope="col" class="py-2 pr-2">{i18nMsg('opening-balances-account', 'Compte')}</th>
					<th scope="col" class="w-40 py-2 pr-2 text-right">{i18nMsg('opening-balances-debit', 'Débit')}</th>
					<th scope="col" class="w-40 py-2 text-right">{i18nMsg('opening-balances-credit', 'Crédit')}</th>
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
								placeholder="0.00"
								class="text-right tabular-nums"
								bind:value={row.debit}
								oninput={() => onDebitInput(row)}
								aria-invalid={!isValidAmount(row.debit)}
								aria-label={i18nMsg('opening-balances-debit-for', 'Débit du compte { $number } « { $name } »', { number: row.account.number, name: row.account.name })}
								data-testid="opening-balances-debit-{row.account.number}"
							/>
						</td>
						<td class="py-1.5">
							<Input
								type="text"
								inputmode="decimal"
								placeholder="0.00"
								class="text-right tabular-nums"
								bind:value={row.credit}
								oninput={() => onCreditInput(row)}
								aria-invalid={!isValidAmount(row.credit)}
								aria-label={i18nMsg('opening-balances-credit-for', 'Crédit du compte { $number } « { $name } »', { number: row.account.number, name: row.account.name })}
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
					<span data-testid="opening-balances-total-debit">{formatExact(balance.totalDebit)}</span>
				</span>
				<span>
					<strong>{i18nMsg('opening-balances-total-credit', 'Total crédits')} :</strong>
					<span data-testid="opening-balances-total-credit">{formatExact(balance.totalCredit)}</span>
				</span>
				<span>
					<strong>{i18nMsg('opening-balances-diff', 'Différence')} :</strong>
					<span data-testid="opening-balances-diff">{formatExact(balance.diff)}</span>
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

		<!-- Totaux actif / passif et montant à porter (AC 2). Il aide la saisie ; le
		     contrôle, c'est la comparaison avec l'ancien bilan (dite à côté). -->
		<div class="rounded-md border border-border p-4 text-sm tabular-nums" data-testid="opening-balances-bilan-totals">
			<dl class="grid grid-cols-2 gap-x-4 gap-y-1">
				<dt>{i18nMsg('opening-balances-total-assets', 'Actifs')}</dt>
				<dd class="text-right" data-testid="opening-balances-total-assets">{formatExact(totals.assets)}</dd>
				<dt>{i18nMsg('opening-balances-total-liabilities', 'Passifs et capitaux (hors report)')}</dt>
				<dd class="text-right" data-testid="opening-balances-total-liabilities">{formatExact(totals.liabilities)}</dd>
				<dt>{i18nMsg('opening-balances-amount-to-carry', 'Montant à porter au compte de report')}</dt>
				<dd class="text-right" data-testid="opening-balances-amount-to-carry" data-side={totals.amountToCarry.gt(0) ? 'credit' : totals.amountToCarry.lt(0) ? 'debit' : 'none'}>
					{formatExact(totals.amountToCarry.abs())}
					{#if totals.amountToCarry.gt(0)}
						{i18nMsg('opening-balances-side-credit', 'au crédit')}
					{:else if totals.amountToCarry.lt(0)}
						{i18nMsg('opening-balances-side-debit', 'au débit')}
					{/if}
				</dd>
				<dt>{i18nMsg('opening-balances-retained-entered', 'Report saisi')}</dt>
				<dd class="text-right" data-testid="opening-balances-retained-entered">{formatExact(totals.retainedEntered)}</dd>
				<dt>{i18nMsg('opening-balances-remaining-gap', 'Écart restant')}</dt>
				<dd class="text-right" data-testid="opening-balances-remaining-gap">{formatExact(totals.remainingGap)}</dd>
			</dl>
			<p class="mt-2 text-text-muted">
				{i18nMsg(
					'opening-balances-compare-hint',
					'Comparez les totaux Actifs et Passifs à ceux du bilan de votre ancienne comptabilité : une saisie équilibrée peut encore être fausse si un montant a été porté sur le mauvais compte.'
				)}
			</p>
		</div>

		{#if warning}
			<!-- Avertissement du report à-nouveau (AC 1) : il signale, il ne bloque
			     pas — le bouton « Générer » reste actif. -->
			<div
				class="rounded-md border border-amber-500 bg-amber-50 p-4 text-sm dark:bg-amber-950/30"
				data-testid="opening-balances-no-retained-earnings"
				data-kind={warning.kind}
				role="status"
			>
				{#if warning.kind === 'NO_ROLE'}
					{i18nMsg(
						'opening-balances-warning-no-retained-role',
						'Aucun compte en service ne porte le rôle « Bénéfice/perte reporté » : Kesh ne peut pas vérifier où le report à-nouveau est porté. Attribuez ce rôle dans le plan comptable, et comparez les totaux Actifs et Passifs à ceux de l’ancien bilan.'
					)}
				{:else if warning.kind === 'NOT_POSTABLE'}
					{i18nMsg(
						'opening-balances-warning-retained-not-postable',
						'Le compte de report { $number } « { $name } » n’est pas imputable : il n’apparaît pas dans la grille, et le report à-nouveau ne peut pas y être porté. Rendez-le imputable dans le plan comptable.',
						{ number: warning.account.number, name: warning.account.name }
					)}
				{:else}
					{i18nMsg(
						'opening-balances-warning-no-retained-earnings',
						'La saisie est équilibrée sans rien sur le compte de report { $number } « { $name } ». Le report à-nouveau — la différence entre les actifs et les passifs de votre ancien bilan — doit y être porté : soit l’écart a été mis sur un autre compte, soit le report vaut réellement zéro. Comparez les totaux Actifs et Passifs à ceux de l’ancien bilan.',
						{ number: warning.account.number, name: warning.account.name }
					)}
				{/if}
			</div>
		{/if}

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
	{/if}
</div>
