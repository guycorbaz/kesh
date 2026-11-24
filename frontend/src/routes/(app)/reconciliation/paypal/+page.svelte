<!--
  Story 38-11d (AC-3 à AC-5) — la page PayPal : choix du compte PayPal et de
  l'exercice, import du CSV « Balance affecting transactions » et son compte
  rendu, propositions à cocher et à accepter par paquets de 500, écarts et
  lignes « à examiner » (rattachement par la modale du rapprochement).

  Story 38-12b (AC-1 à AC-3) — à côté de l'import du CSV, « Télécharger depuis
  PayPal » pour un compte dont le libellé a ses identifiants dans
  l'environnement du serveur (`providerApi.configured` de la liste des comptes
  bancaires) : le dialogue générique `ProviderDownloadDialog` lance la tâche, en
  suit la progression et montre le même compte rendu que l'import du CSV
  (`PaypalImportReport`) ; la revue se recharge à la fin. Sans identifiants,
  l'import du CSV reste seul, et l'Administrateur lit une ligne d'aide.

  Kesh PROPOSE, l'utilisateur ACCEPTE : rien n'est écrit à la lecture de la
  revue ; aucune proposition n'est cochée d'office. La revue se recharge après
  l'import et après l'acceptation ; une réponse périmée (autre compte, autre
  exercice, rechargement plus récent) est écartée par une garde de génération.
-->
<script lang="ts">
	import { onMount } from 'svelte';
	import { Button } from '$lib/components/ui/button';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { errorMessageOf } from '$lib/shared/utils/error-message';
	import { todayInZurich, todayLocal } from '$lib/shared/utils/local-date';
	import { authState } from '$lib/app/stores/auth.svelte';
	import { formatSwissDate } from '$lib/features/reports/reports.api';
	import { fetchAccounts } from '$lib/features/accounts/accounts.api';
	import { withoutForeignCurrency, type AccountResponse } from '$lib/features/accounts/accounts.types';
	import { listBankAccounts, type BankAccountSummary } from '$lib/features/bank-accounts/bank-accounts.api';
	import { accountLabelKey, paypalAccountLabels, paypalApiStatus } from '$lib/features/bank-accounts/provider';
	import { listFiscalYears } from '$lib/features/fiscal-years/fiscal-years.api';
	import type { FiscalYearResponse } from '$lib/features/fiscal-years/fiscal-years.types';
	import LinkSettlementModal from '$lib/features/reconciliation/LinkSettlementModal.svelte';
	import ProviderDownloadDialog from '$lib/features/provider-download/ProviderDownloadDialog.svelte';
	import { defaultPeriod } from '$lib/features/provider-download/provider-download';
	import PaypalImportReport from '$lib/features/paypal/PaypalImportReport.svelte';
	import type { LinkTarget } from '$lib/features/reconciliation/reconciliation.types';
	import {
		acceptPaypalProposals,
		chunkProposals,
		getPaypalReview,
		importPaypalCsv,
	} from '$lib/features/paypal/paypal.api';
	import {
		findingGapDetails,
		findingGapLabel,
		paypalFailedLabel,
		paypalImportErrorLines,
		paypalLineErrorLabel,
		proposalKindLabel,
		toReviewReasonLabel,
	} from '$lib/features/paypal/paypal-labels';
	import {
		defaultFiscalYearId,
		proposalActionLabel,
		proposalKey,
		proposalTransactionId,
		selectedBodies,
		withdrawalAccountCandidates,
	} from '$lib/features/paypal/paypal-review';
	import { listRules } from '$lib/features/reconciliation/rules/rules.api';
	import type {
		PaypalFailedProposal,
		PaypalFinding,
		PaypalDownloadResult,
		PaypalImportResponse,
		PaypalReview,
	} from '$lib/features/paypal/paypal.types';

	let bankAccounts = $state<BankAccountSummary[]>([]);
	let fiscalYears = $state<FiscalYearResponse[]>([]);
	let accounts = $state<AccountResponse[]>([]);
	// Libellés des règles, par identifiant : la revue ne sert que `ruleId`.
	let ruleLabels = $state<Map<number, string>>(new Map());
	let loading = $state(true);
	let loadError = $state<string | null>(null);

	let selectedAccount = $state<string | null>(null);
	let selectedYearId = $state<number | null>(null);

	// Import
	let fileInput = $state<HTMLInputElement | null>(null);
	let importing = $state(false);
	let importReport = $state<PaypalImportResponse | null>(null);
	let importErrors = $state<string[]>([]);

	// Revue
	let review = $state<PaypalReview | null>(null);
	let reviewLoading = $state(false);
	let reviewError = $state<string | null>(null);
	let reviewGen = 0;
	let selected = $state<Set<string>>(new Set());
	let withdrawalAccounts = $state<Record<string, number | null>>({});

	// Acceptation
	let accepting = $state(false);
	let batchProgress = $state<{ current: number; total: number } | null>(null);
	let acceptReport = $state<{ accepted: number; failed: PaypalFailedProposal[] } | null>(null);
	let acceptError = $state<string | null>(null);

	// Téléchargement par l'API (38-12b)
	let downloadOpen = $state(false);

	// Rattachement d'une ligne « à examiner »
	let linkOpen = $state(false);
	let linkTarget = $state<LinkTarget | null>(null);

	let paypalAccounts = $derived(paypalAccountLabels(bankAccounts));
	// fx-filtre-comptes : la liste des retraits est filtrée en CHF (`withdrawalAccountCandidates`,
	// après `withoutForeignCurrency`) ; les autres comptes ne servent qu'aux libellés.
	// Le compte du plan du compte PayPal choisi est refusé par le serveur comme destination.
	let ownLedgerIds = $derived(
		bankAccounts
			.filter(
				(b) =>
					b.provider === 'paypal' &&
					b.paypal !== null &&
					selectedAccount !== null &&
					accountLabelKey(b.paypal.account) === accountLabelKey(selectedAccount) &&
					b.journalAccountId !== null,
			)
			.map((b) => b.journalAccountId as number),
	);
	let withdrawalChoices = $derived(
		withdrawalAccountCandidates(withoutForeignCurrency(accounts), ownLedgerIds),
	);

	// Téléchargement par l'API (38-12b, AC-1) : l'état du compte choisi, le rôle, la période proposée.
	let downloadApi = $derived(selectedAccount === null ? null : paypalApiStatus(bankAccounts, selectedAccount));
	let role = $derived(authState.currentUser?.role);
	let canDownload = $derived(role === 'Admin' || role === 'Comptable');
	let downloadPeriod = $derived(
		defaultPeriod(fiscalYears.find((fy) => fy.id === selectedYearId) ?? null, todayInZurich()),
	);

	onMount(async () => {
		const [ba, fy, acc, rules] = await Promise.allSettled([
			listBankAccounts(),
			listFiscalYears(),
			fetchAccounts(false),
			allRules(),
		]);
		// Libellés des règles : best-effort, « Règle #id » sinon.
		ruleLabels =
			rules.status === 'fulfilled' ? new Map(rules.value.map((r) => [r.id, r.label])) : new Map();
		if (ba.status === 'fulfilled') bankAccounts = ba.value;
		else loadError = errorMessageOf(ba.reason);
		if (fy.status === 'fulfilled') fiscalYears = fy.value;
		else loadError = errorMessageOf(fy.reason);
		accounts = acc.status === 'fulfilled' ? acc.value : [];
		selectedAccount = paypalAccounts[0] ?? null;
		selectedYearId = defaultFiscalYearId(fiscalYears, todayLocal());
		loading = false;
		await loadReview();
	});

	/**
	 * Toutes les règles, page par page (le serveur borne une page à 200) : les libellés des
	 * propositions « Règle … » au-delà de la première page (revue P1, L9).
	 */
	async function allRules(): Promise<{ id: number; label: string }[]> {
		const out: { id: number; label: string }[] = [];
		for (let page = 1; page <= 50; page++) {
			const res = await listRules(page, 200);
			out.push(...res.items);
			if (res.items.length === 0 || out.length >= res.total) break;
		}
		return out;
	}

	/** Relit la revue du compte et de l'exercice choisis ; une réponse périmée est écartée. */
	async function loadReview() {
		const account = selectedAccount;
		const yearId = selectedYearId;
		const myGen = ++reviewGen;
		if (account === null || yearId === null) {
			review = null;
			reviewLoading = false;
			return;
		}
		reviewLoading = true;
		reviewError = null;
		try {
			const r = await getPaypalReview(account, yearId);
			if (myGen !== reviewGen) return;
			review = r;
			selected = new Set();
			withdrawalAccounts = {};
		} catch (err) {
			if (myGen !== reviewGen) return;
			review = null;
			reviewError = errorMessageOf(err);
		} finally {
			if (myGen === reviewGen) reviewLoading = false;
		}
	}

	async function onSelectionChange() {
		importReport = null;
		importErrors = [];
		acceptReport = null;
		await loadReview();
	}

	async function onFileChosen(e: Event) {
		const input = e.currentTarget as HTMLInputElement;
		const file = input.files?.[0];
		input.value = '';
		if (!file || selectedAccount === null) return;
		importing = true;
		importErrors = [];
		try {
			importReport = await importPaypalCsv(file, selectedAccount);
			await loadReview();
		} catch (err) {
			importReport = null;
			importErrors = paypalImportErrorLines(err);
		} finally {
			importing = false;
		}
	}

	function toggle(key: string, on: boolean) {
		const next = new Set(selected);
		if (on) next.add(key);
		else next.delete(key);
		selected = next;
	}

	/** Une proposition se coche-t-elle ? Un retrait sans compte de destination, non. */
	function selectable(key: string, needsAccount: boolean): boolean {
		return !needsAccount || (withdrawalAccounts[key] ?? null) !== null;
	}

	function toggleAll(on: boolean) {
		if (!review) return;
		selected = on
			? new Set(
					review.proposals
						.filter((p) => selectable(proposalKey(p), p.type === 'withdrawal' && p.counterpartyAccountId === null))
						.map(proposalKey),
				)
			: new Set();
	}

	function chooseWithdrawal(key: string, value: string) {
		const id = value === '' ? null : Number(value);
		withdrawalAccounts = { ...withdrawalAccounts, [key]: id };
		if (id === null) toggle(key, false);
	}

	/** Accepte la sélection, par paquets de 500, un à la fois, dans l'ordre des dates. */
	async function acceptSelection() {
		// Le compte est figé avant la boucle (revue P1, L1) : les sélecteurs sont désactivés
		// pendant l'acceptation, et chaque paquet part au même compte.
		const account = selectedAccount;
		if (!review || account === null) return;
		const bodies = selectedBodies(review.proposals, selected, withdrawalAccounts);
		if (bodies.length === 0) return;
		const batches = chunkProposals(bodies);
		accepting = true;
		acceptError = null;
		const report = { accepted: 0, failed: [] as PaypalFailedProposal[] };
		try {
			for (let i = 0; i < batches.length; i++) {
				batchProgress = { current: i + 1, total: batches.length };
				const res = await acceptPaypalProposals(account, batches[i]);
				report.accepted += res.accepted.length;
				report.failed.push(...res.failed);
			}
		} catch (err) {
			acceptError = errorMessageOf(err);
		} finally {
			acceptReport = report;
			batchProgress = null;
			accepting = false;
			await loadReview();
		}
	}

	/** Ouvre la modale de rattachement sur la transaction d'une ligne « à examiner ». */
	function openLink(item: PaypalFinding) {
		const t = item.transaction;
		if (!t) return;
		linkTarget = {
			bankTransactionId: t.id,
			transaction: {
				bookingDate: t.date,
				valueDate: null,
				amount: t.amount,
				currency: t.currency,
				counterpartyName: t.name,
			},
		};
		linkOpen = true;
	}

	/** La date d'un écart : celle de sa transaction, sinon de sa première écriture. */
	function findingDate(f: PaypalFinding): string {
		const d = f.transaction?.date ?? f.entries?.[0]?.entryDate;
		return d ? formatSwissDate(d) : '';
	}

	function failedLabel(f: PaypalFailedProposal): string {
		return paypalFailedLabel(f.errorCode, f.details);
	}
</script>

<svelte:head>
	<title>{i18nMsg('paypal-page-title', 'PayPal')} - Kesh</title>
</svelte:head>

<h1 class="text-2xl font-semibold text-text" data-testid="paypal-page-title">
	{i18nMsg('paypal-page-title', 'PayPal')}
</h1>
<p class="mt-2 text-sm text-text-muted">
	{i18nMsg(
		'paypal-page-subtitle',
		'Importez le relevé CSV d’un compte PayPal, puis acceptez les propositions de rapprochement. Rien n’est écrit sans votre acceptation.',
	)}
</p>

{#if loading}
	<p class="mt-6 text-text-muted">{i18nMsg('paypal-loading', 'Chargement…')}</p>
{:else if loadError}
	<p class="mt-6 text-red-600" role="alert" data-testid="paypal-load-error">{loadError}</p>
{:else if paypalAccounts.length === 0}
	<p class="mt-6 text-text-muted" data-testid="paypal-no-account">
		{i18nMsg(
			'paypal-no-account',
			'Aucun compte PayPal. Créez un compte bancaire par devise, case « Compte PayPal » cochée, dans les comptes bancaires.',
		)}
		<a href="/bank-accounts" class="ml-1 text-primary underline" data-testid="paypal-no-account-link">
			{i18nMsg('paypal-no-account-link', 'Comptes bancaires')}
		</a>
	</p>
{:else}
	<div class="mt-6 flex flex-wrap items-end gap-4">
		<div>
			<label for="paypal-account" class="block text-sm font-medium text-text mb-1">
				{i18nMsg('paypal-label-account', 'Compte PayPal')}
			</label>
			<select
				id="paypal-account"
				bind:value={selectedAccount}
				onchange={onSelectionChange}
				disabled={accepting || importing}
				class="rounded border border-border px-3 py-2 text-sm"
				data-testid="paypal-account"
			>
				{#each paypalAccounts as label (label)}
					<option value={label}>{label}</option>
				{/each}
			</select>
		</div>
		<div>
			<label for="paypal-fiscal-year" class="block text-sm font-medium text-text mb-1">
				{i18nMsg('paypal-label-fiscal-year', 'Exercice')}
			</label>
			<select
				id="paypal-fiscal-year"
				bind:value={selectedYearId}
				onchange={onSelectionChange}
				disabled={accepting || importing}
				class="rounded border border-border px-3 py-2 text-sm"
				data-testid="paypal-fiscal-year"
			>
				{#each fiscalYears as fy (fy.id)}
					<option value={fy.id}>{fy.name}</option>
				{/each}
			</select>
		</div>
		<div>
			<input
				bind:this={fileInput}
				type="file"
				accept=".csv,text/csv"
				class="hidden"
				onchange={onFileChosen}
				data-testid="paypal-file-input"
			/>
			<Button onclick={() => fileInput?.click()} disabled={importing || accepting} data-testid="paypal-import-button">
				{importing
					? i18nMsg('paypal-import-running', 'Import en cours…')
					: i18nMsg('paypal-import-button', 'Importer le CSV PayPal')}
			</Button>
		</div>
		{#if canDownload && downloadApi?.configured}
			<div>
				<Button
					variant="outline"
					onclick={() => (downloadOpen = true)}
					disabled={importing || accepting}
					data-testid="paypal-download-button"
				>
					{i18nMsg('paypal-download-button', 'Télécharger depuis PayPal')}
				</Button>
				{#if downloadApi.mode === 'sandbox'}
					<span class="ml-2 text-xs text-text-muted" data-testid="paypal-download-sandbox">
						{i18nMsg('paypal-download-sandbox', 'bac à sable')}
					</span>
				{/if}
			</div>
		{/if}
	</div>
	{#if role === 'Admin' && downloadApi !== null && !downloadApi.configured}
		<p class="mt-2 text-xs text-text-muted" data-testid="paypal-download-not-configured">
			{i18nMsg(
				'paypal-download-not-configured',
				'Téléchargement depuis PayPal non configuré pour ce compte : voir le manuel d’administration.',
			)}
		</p>
	{/if}

	{#if importErrors.length > 0}
		<div class="mt-4 rounded border border-red-200 bg-red-50 p-3 text-sm" role="alert" data-testid="paypal-import-error">
			{#each importErrors as line, i (i)}
				<p data-testid="paypal-import-error-line">{line}</p>
			{/each}
			<a href="/bank-accounts" class="text-primary underline">
				{i18nMsg('paypal-no-account-link', 'Comptes bancaires')}
			</a>
		</div>
	{/if}

	{#if importReport}
		<PaypalImportReport report={importReport} />
	{/if}

	{#if reviewError}
		<p class="mt-4 text-red-600" role="alert" data-testid="paypal-review-error">{reviewError}</p>
	{/if}
	{#if reviewLoading && !review}
		<p class="mt-4 text-text-muted">{i18nMsg('paypal-loading', 'Chargement…')}</p>
	{/if}

	{#if acceptReport}
		<section class="mt-4 rounded border border-border bg-surface-alt p-3 text-sm" data-testid="paypal-accept-report">
			<p>
				{i18nMsg('paypal-accept-report', '{ $accepted } proposition(s) acceptée(s), { $failed } refusée(s).', {
					accepted: String(acceptReport.accepted),
					failed: String(acceptReport.failed.length),
				})}
			</p>
			{#if acceptReport.failed.length > 0}
				<ul class="mt-1 list-disc pl-5" data-testid="paypal-accept-failed">
					{#each acceptReport.failed as f, i (i)}
						<li>#{f.bankTransactionId} — {failedLabel(f)}</li>
					{/each}
				</ul>
			{/if}
		</section>
	{/if}
	{#if acceptError}
		<p class="mt-2 text-red-600" role="alert" data-testid="paypal-accept-error">{acceptError}</p>
	{/if}

	{#if review}
		<!-- Propositions -->
		<section class="mt-6" data-testid="paypal-proposals">
			<h2 class="text-lg font-semibold">
				{i18nMsg('paypal-proposals-title', 'Propositions ({ $count })', {
					count: String(review.proposals.length),
				})}
			</h2>
			{#if review.proposals.length === 0}
				<p class="text-text-muted" data-testid="paypal-proposals-empty">
					{i18nMsg('paypal-proposals-empty', 'Aucune proposition.')}
				</p>
			{:else}
				<div class="mt-2 flex items-center gap-3">
					<Button variant="outline" onclick={() => toggleAll(true)} disabled={accepting} data-testid="paypal-select-all">
						{i18nMsg('paypal-select-all', 'Tout cocher')}
					</Button>
					<Button variant="ghost" onclick={() => toggleAll(false)} disabled={accepting} data-testid="paypal-select-none">
						{i18nMsg('paypal-select-none', 'Tout décocher')}
					</Button>
					<Button onclick={acceptSelection} disabled={accepting || importing || reviewLoading || selected.size === 0} data-testid="paypal-accept">
						{i18nMsg('paypal-accept', 'Accepter la sélection')}
					</Button>
					{#if batchProgress}
						<span class="text-sm text-text-muted" data-testid="paypal-accept-progress">
							{i18nMsg('paypal-accept-progress', 'Lot { $current } sur { $total }', {
								current: String(batchProgress.current),
								total: String(batchProgress.total),
							})}
						</span>
					{/if}
				</div>
				<table class="mt-2 w-full text-sm" data-testid="paypal-proposals-table">
					<thead>
						<tr class="border-b border-border text-left">
							<th class="py-2 pr-2"></th>
							<th class="py-2 pr-4 font-semibold">{i18nMsg('paypal-col-date', 'Date')}</th>
							<th class="py-2 pr-4 font-semibold">{i18nMsg('paypal-col-kind', 'Genre')}</th>
							<th class="py-2 pr-4 text-right font-semibold">{i18nMsg('paypal-col-amount', 'Montant')}</th>
							<th class="py-2 pr-4 font-semibold">{i18nMsg('paypal-col-name', 'Nom')}</th>
							<th class="py-2 pr-4 font-semibold">{i18nMsg('paypal-col-txn', 'TXN')}</th>
							<th class="py-2 font-semibold">{i18nMsg('paypal-col-action', 'Ce qui sera fait')}</th>
						</tr>
					</thead>
					<tbody>
						{#each review.proposals as p (proposalKey(p))}
							{@const key = proposalKey(p)}
							{@const needsAccount = p.type === 'withdrawal' && p.counterpartyAccountId === null}
							<tr class="border-b border-border" data-testid="paypal-proposal-{proposalTransactionId(p)}">
								<td class="py-2 pr-2">
									<input
										type="checkbox"
										checked={selected.has(key)}
										disabled={accepting || !selectable(key, needsAccount)}
										onchange={(e) => toggle(key, (e.currentTarget as HTMLInputElement).checked)}
										aria-label={i18nMsg('paypal-select-one', 'Choisir cette proposition')}
										data-testid="paypal-check-{proposalTransactionId(p)}"
									/>
								</td>
								<td class="py-2 pr-4 whitespace-nowrap">{formatSwissDate(p.transaction.date)}</td>
								<td class="py-2 pr-4">{proposalKindLabel(p, ruleLabels)}</td>
								<td class="py-2 pr-4 text-right font-mono whitespace-nowrap">
									{p.transaction.amount} {p.transaction.currency}
									{#if p.otherTransaction}
										<span class="block text-xs text-text-muted">{p.otherTransaction.amount} {p.otherTransaction.currency}</span>
									{/if}
								</td>
								<td class="py-2 pr-4">{p.transaction.name ?? ''}</td>
								<td class="py-2 pr-4 font-mono text-xs">{p.transaction.transactionId ?? ''}</td>
								<td class="py-2">
									{#if needsAccount}
										<label class="sr-only" for="paypal-withdrawal-{key}">
											{i18nMsg('paypal-withdrawal-account', 'Compte de destination')}
										</label>
										<select
											id="paypal-withdrawal-{key}"
											class="rounded border border-border px-2 py-1 text-xs"
											value={withdrawalAccounts[key] ?? ''}
											onchange={(e) => chooseWithdrawal(key, (e.currentTarget as HTMLSelectElement).value)}
											data-testid="paypal-withdrawal-{proposalTransactionId(p)}"
										>
											<option value="">{i18nMsg('paypal-withdrawal-choose', '— Compte de destination')}</option>
											{#each withdrawalChoices as a (a.id)}
												<option value={String(a.id)}>{a.number} — {a.name}</option>
											{/each}
										</select>
									{:else}
										{proposalActionLabel(p, accounts)}
									{/if}
								</td>
							</tr>
						{/each}
					</tbody>
				</table>
			{/if}
		</section>

		<!-- Écarts -->
		<section class="mt-8" data-testid="paypal-gaps">
			<h2 class="text-lg font-semibold">
				{i18nMsg('paypal-gaps-title', 'Écarts ({ $count })', { count: String(review.gaps.length) })}
			</h2>
			{#if review.gaps.length === 0}
				<p class="text-text-muted" data-testid="paypal-gaps-empty">{i18nMsg('paypal-gaps-empty', 'Aucun écart.')}</p>
			{:else}
				<table class="mt-2 w-full text-sm" data-testid="paypal-gaps-table">
					<thead>
						<tr class="border-b border-border text-left">
							<th class="py-2 pr-4 font-semibold">{i18nMsg('paypal-col-date', 'Date')}</th>
							<th class="py-2 pr-4 font-semibold">{i18nMsg('paypal-col-gap', 'Écart')}</th>
							<th class="py-2 pr-4 text-right font-semibold">{i18nMsg('paypal-col-amount', 'Montant')}</th>
							<th class="py-2 pr-4 font-semibold">{i18nMsg('paypal-col-name', 'Nom')}</th>
							<th class="py-2 pr-4 font-semibold">{i18nMsg('paypal-col-txn', 'TXN')}</th>
							<th class="py-2 font-semibold">{i18nMsg('paypal-col-entry', 'Écriture')}</th>
						</tr>
					</thead>
					<tbody>
						{#each review.gaps as g, i (i)}
							{@const entry = g.entries?.[0]}
							<tr class="border-b border-border" data-testid="paypal-gap-{g.code}">
								<td class="py-2 pr-4 whitespace-nowrap">{findingDate(g)}</td>
								<td class="py-2 pr-4">
									{findingGapLabel(g)}
									{#if findingGapDetails(g) !== null}
										<span class="block text-xs text-text-muted" data-testid="paypal-gap-details">{findingGapDetails(g)}</span>
									{/if}
								</td>
								<td class="py-2 pr-4 text-right font-mono whitespace-nowrap">
									{#if g.transaction}{g.transaction.amount} {g.transaction.currency}{/if}
								</td>
								<td class="py-2 pr-4">{g.transaction?.name ?? entry?.description ?? ''}</td>
								<td class="py-2 pr-4 font-mono text-xs">{g.transaction?.transactionId ?? entry?.ids.join(', ') ?? ''}</td>
								<td class="py-2">
									{#if entry}
										<a class="text-primary underline" href="/journal-entries/{entry.journalEntryId}">
											{i18nMsg('paypal-entry-number', 'n° { $number }', {
												number: String(entry.entryNumber),
											})}
										</a>
									{/if}
								</td>
							</tr>
						{/each}
					</tbody>
				</table>
			{/if}
		</section>

		<!-- À examiner -->
		<section class="mt-8" data-testid="paypal-to-review">
			<h2 class="text-lg font-semibold">
				{i18nMsg('paypal-to-review-title', 'À examiner ({ $count })', {
					count: String(review.toReview.length),
				})}
			</h2>
			{#if review.toReview.length === 0}
				<p class="text-text-muted" data-testid="paypal-to-review-empty">
					{i18nMsg('paypal-to-review-empty', 'Rien à examiner.')}
				</p>
			{:else}
				<table class="mt-2 w-full text-sm" data-testid="paypal-to-review-table">
					<thead>
						<tr class="border-b border-border text-left">
							<th class="py-2 pr-4 font-semibold">{i18nMsg('paypal-col-date', 'Date')}</th>
							<th class="py-2 pr-4 font-semibold">{i18nMsg('paypal-col-reason', 'Motif')}</th>
							<th class="py-2 pr-4 text-right font-semibold">{i18nMsg('paypal-col-amount', 'Montant')}</th>
							<th class="py-2 pr-4 font-semibold">{i18nMsg('paypal-col-type', 'Type PayPal')}</th>
							<th class="py-2 pr-4 font-semibold">{i18nMsg('paypal-col-name', 'Nom')}</th>
							<th class="py-2 pr-4 font-semibold">{i18nMsg('paypal-col-email', 'E-mail')}</th>
							<th class="py-2"></th>
						</tr>
					</thead>
					<tbody>
						{#each review.toReview as t, i (i)}
							{@const tx = t.transaction}
							<tr class="border-b border-border" data-testid="paypal-to-review-{tx?.id ?? `e${i}`}">
								<td class="py-2 pr-4 whitespace-nowrap">{findingDate(t)}</td>
								<td class="py-2 pr-4">{toReviewReasonLabel(t.code)}</td>
								<td class="py-2 pr-4 text-right font-mono whitespace-nowrap">{#if tx}{tx.amount} {tx.currency}{/if}</td>
								<td class="py-2 pr-4">{tx?.providerType ?? ''}</td>
								<td class="py-2 pr-4">{tx?.name ?? ''}</td>
								<td class="py-2 pr-4 text-xs">{tx?.counterpartyEmail ?? ''}</td>
								<td class="py-2 whitespace-nowrap">
									<button
										type="button"
										class="rounded border border-border px-2 py-1 text-xs"
										onclick={() => openLink(t)}
										disabled={!tx}
										data-testid="paypal-link-{tx?.id ?? `e${i}`}"
									>
										{i18nMsg('paypal-link-entry', 'Rattacher à une écriture')}
									</button>
									<a href="/journal-entries" class="ml-1 text-xs text-primary underline">
										{i18nMsg('paypal-manual-entry', 'Écriture manuelle')}
									</a>
								</td>
							</tr>
						{/each}
					</tbody>
				</table>
			{/if}
		</section>
	{/if}
{/if}

{#if selectedAccount !== null}
	<ProviderDownloadDialog
		open={downloadOpen}
		onOpenChange={(v) => (downloadOpen = v)}
		provider="paypal"
		account={selectedAccount}
		title={i18nMsg('paypal-download-title', 'Télécharger depuis PayPal')}
		stepLabel={i18nMsg('paypal-download-step', 'fenêtre')}
		notice={i18nMsg(
			'paypal-download-notice',
			'PayPal peut mettre jusqu’à trois heures à publier une transaction ; l’historique remonte à trois ans ; le téléchargement se fait par tranches de 30 jours.',
		)}
		sandbox={downloadApi?.mode === 'sandbox'}
		defaultPeriod={downloadPeriod}
		lineLabel={paypalLineErrorLabel}
		onDone={() => void loadReview()}
	>
		{#snippet report(result: PaypalDownloadResult)}
			<PaypalImportReport report={result} testid="paypal-download-report" />
		{/snippet}
	</ProviderDownloadDialog>
{/if}

<LinkSettlementModal
	open={linkOpen}
	onOpenChange={(v) => (linkOpen = v)}
	proposal={linkTarget}
	onSuccess={() => void loadReview()}
/>
