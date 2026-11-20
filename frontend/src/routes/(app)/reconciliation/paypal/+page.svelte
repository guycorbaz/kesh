<!--
  Story 38-11d (AC-3 à AC-5) — la page PayPal : choix du compte PayPal et de
  l'exercice, import du CSV « Balance affecting transactions » et son compte
  rendu, propositions à cocher et à accepter par paquets de 500, écarts et
  lignes « à examiner » (rattachement par la modale du rapprochement).

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
	import { todayLocal } from '$lib/shared/utils/local-date';
	import { formatSwissDate } from '$lib/features/reports/reports.api';
	import { fetchAccounts } from '$lib/features/accounts/accounts.api';
	import { withoutForeignCurrency, type AccountResponse } from '$lib/features/accounts/accounts.types';
	import { listBankAccounts, type BankAccountSummary } from '$lib/features/bank-accounts/bank-accounts.api';
	import { paypalAccountLabels } from '$lib/features/bank-accounts/provider';
	import { listFiscalYears } from '$lib/features/fiscal-years/fiscal-years.api';
	import type { FiscalYearResponse } from '$lib/features/fiscal-years/fiscal-years.types';
	import LinkSettlementModal from '$lib/features/reconciliation/LinkSettlementModal.svelte';
	import type { LinkTarget } from '$lib/features/reconciliation/reconciliation.types';
	import {
		acceptPaypalProposals,
		chunkProposals,
		getPaypalReview,
		importPaypalCsv,
	} from '$lib/features/paypal/paypal.api';
	import {
		gapKindLabel,
		paypalFailedLabel,
		paypalImportErrorLines,
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
	import type {
		PaypalFailedProposal,
		PaypalImportResponse,
		PaypalReview,
		PaypalToReview,
	} from '$lib/features/paypal/paypal.types';

	let bankAccounts = $state<BankAccountSummary[]>([]);
	let fiscalYears = $state<FiscalYearResponse[]>([]);
	let accounts = $state<AccountResponse[]>([]);
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

	// Rattachement d'une ligne « à examiner »
	let linkOpen = $state(false);
	let linkTarget = $state<LinkTarget | null>(null);

	let paypalAccounts = $derived(paypalAccountLabels(bankAccounts));
	// fx-filtre-comptes : la liste des retraits est filtrée en CHF (`withdrawalAccountCandidates`,
	// après `withoutForeignCurrency`) ; les autres comptes ne servent qu'aux libellés.
	let withdrawalChoices = $derived(withdrawalAccountCandidates(withoutForeignCurrency(accounts)));

	onMount(async () => {
		const [ba, fy, acc] = await Promise.allSettled([
			listBankAccounts(),
			listFiscalYears(),
			fetchAccounts(false),
		]);
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

	/** Relit la revue du compte et de l'exercice choisis ; une réponse périmée est écartée. */
	async function loadReview() {
		const account = selectedAccount;
		const yearId = selectedYearId;
		const myGen = ++reviewGen;
		if (account === null || yearId === null) {
			review = null;
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
		if (!review || selectedAccount === null) return;
		const bodies = selectedBodies(review.proposals, selected, withdrawalAccounts);
		if (bodies.length === 0) return;
		const batches = chunkProposals(bodies);
		accepting = true;
		acceptError = null;
		const report = { accepted: 0, failed: [] as PaypalFailedProposal[] };
		try {
			for (let i = 0; i < batches.length; i++) {
				batchProgress = { current: i + 1, total: batches.length };
				const res = await acceptPaypalProposals(selectedAccount, batches[i]);
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

	function openLink(item: PaypalToReview) {
		linkTarget = {
			bankTransactionId: item.bankTransactionId,
			transaction: {
				bookingDate: item.date,
				valueDate: null,
				amount: item.amount,
				currency: item.currency,
				counterpartyName: item.name,
			},
		};
		linkOpen = true;
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
			<Button onclick={() => fileInput?.click()} disabled={importing} data-testid="paypal-import-button">
				{importing
					? i18nMsg('paypal-import-running', 'Import en cours…')
					: i18nMsg('paypal-import-button', 'Importer le CSV PayPal')}
			</Button>
		</div>
	</div>

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
		<section class="mt-4 rounded border border-border bg-surface-alt p-3 text-sm" data-testid="paypal-import-report">
			<h2 class="font-semibold">{i18nMsg('paypal-import-report-title', 'Compte rendu de l’import')}</h2>
			<ul class="mt-2 list-disc pl-5">
				{#each importReport.currencies as c (c.currency)}
					<li data-testid="paypal-import-report-{c.currency}">
						{i18nMsg('paypal-import-report-currency', '{ $currency } : { $inserted } transaction(s) insérée(s), { $duplicates } doublon(s)', {
							currency: c.currency,
							inserted: String(c.inserted),
							duplicates: String(c.duplicates),
						})}
					</li>
				{/each}
			</ul>
			<p class="mt-2" data-testid="paypal-import-report-skipped">
				{i18nMsg('paypal-import-report-skipped', 'Écartées : { $notCompleted } ligne(s) non terminée(s), { $memo } ligne(s) Memo, { $pairs } paire(s) neutre(s).', {
					notCompleted: String(importReport.skipped.notCompleted.length),
					memo: String(importReport.skipped.memo),
					pairs: String(importReport.skipped.neutralPairs),
				})}
			</p>
			{#if importReport.skipped.notCompleted.length > 0}
				<ul class="mt-1 list-disc pl-5 text-xs text-text-muted" data-testid="paypal-import-report-not-completed">
					{#each importReport.skipped.notCompleted as l (l.transactionId)}
						<li>{l.transactionId} — {l.type} — {l.status}</li>
					{/each}
				</ul>
			{/if}
		</section>
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
					<Button variant="outline" onclick={() => toggleAll(true)} data-testid="paypal-select-all">
						{i18nMsg('paypal-select-all', 'Tout cocher')}
					</Button>
					<Button variant="ghost" onclick={() => toggleAll(false)} data-testid="paypal-select-none">
						{i18nMsg('paypal-select-none', 'Tout décocher')}
					</Button>
					<Button onclick={acceptSelection} disabled={accepting || selected.size === 0} data-testid="paypal-accept">
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
										disabled={!selectable(key, needsAccount)}
										onchange={(e) => toggle(key, (e.currentTarget as HTMLInputElement).checked)}
										aria-label={i18nMsg('paypal-select-one', 'Choisir cette proposition')}
										data-testid="paypal-check-{proposalTransactionId(p)}"
									/>
								</td>
								<td class="py-2 pr-4 whitespace-nowrap">{formatSwissDate(p.date)}</td>
								<td class="py-2 pr-4">{proposalKindLabel(p)}</td>
								<td class="py-2 pr-4 text-right font-mono whitespace-nowrap">{p.amount} {p.currency}</td>
								<td class="py-2 pr-4">{p.name ?? ''}</td>
								<td class="py-2 pr-4 font-mono text-xs">{p.transactionId ?? ''}</td>
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
							<tr class="border-b border-border" data-testid="paypal-gap-{g.kind}">
								<td class="py-2 pr-4 whitespace-nowrap">{g.date ? formatSwissDate(g.date) : g.entryDate ? formatSwissDate(g.entryDate) : ''}</td>
								<td class="py-2 pr-4">{gapKindLabel(g.kind)}</td>
								<td class="py-2 pr-4 text-right font-mono whitespace-nowrap">{g.amount ?? ''} {g.currency ?? ''}</td>
								<td class="py-2 pr-4">{g.name ?? ''}</td>
								<td class="py-2 pr-4 font-mono text-xs">{g.transactionId ?? g.saleTransactionId ?? ''}</td>
								<td class="py-2">
									{#if g.journalEntryId}
										<a class="text-primary underline" href="/journal-entries/{g.journalEntryId}">
											{i18nMsg('paypal-entry-number', 'n° { $number }', {
												number: String(g.entryNumber ?? g.journalEntryId),
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
						{#each review.toReview as t (t.bankTransactionId)}
							<tr class="border-b border-border" data-testid="paypal-to-review-{t.bankTransactionId}">
								<td class="py-2 pr-4 whitespace-nowrap">{formatSwissDate(t.date)}</td>
								<td class="py-2 pr-4">{toReviewReasonLabel(t.reason)}</td>
								<td class="py-2 pr-4 text-right font-mono whitespace-nowrap">{t.amount} {t.currency}</td>
								<td class="py-2 pr-4">{t.providerType ?? ''}</td>
								<td class="py-2 pr-4">{t.name ?? ''}</td>
								<td class="py-2 pr-4 text-xs">{t.counterpartyEmail ?? ''}</td>
								<td class="py-2 whitespace-nowrap">
									<button
										type="button"
										class="rounded border border-border px-2 py-1 text-xs"
										onclick={() => openLink(t)}
										data-testid="paypal-link-{t.bankTransactionId}"
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

<LinkSettlementModal
	open={linkOpen}
	onOpenChange={(v) => (linkOpen = v)}
	proposal={linkTarget}
	onSuccess={() => void loadReview()}
/>
