<!--
  Story 40-7d (AC-2, AC-3) — la page **commune** des prestataires (PayPal,
  Stripe ; Wise depuis la 39-11d, plusieurs fichiers ou un zip d'un geste, et
  sa revue depuis la 39-11d2), tirée de la page PayPal de la 38-11d : choix du compte et de
  l'exercice, import du fichier et son compte rendu, propositions à cocher et
  à accepter par paquets de 500, écarts et lignes « à examiner » (rattachement
  par la modale du rapprochement, écriture manuelle). Ce qui est
  propre à un prestataire — ses textes, son client d'API, ses libellés, la
  forme de son compte rendu, l'action d'une proposition, les colonnes de ses
  lignes « à examiner » — lui est passé par ses propriétés et ses fragments ;
  les identifiants de test se préfixent par `testId` (« paypal », « stripe », « wise »).

  Story 39-11d2 (AC-2) — les lignes d'une proposition ou d'une ligne « à
  examiner » qui en réunit plusieurs (le paiement par carte groupé de Wise) :
  `proposalLines` et `findingLines` rendent ses lignes hors la principale,
  affichées en sous-lignes (identifiants de test propres, jamais ceux d'une
  ligne) ; chaque sous-ligne d'un « à examiner » a son propre bouton
  « Rattacher à une écriture », sur sa transaction ; `lineLabel` dit ce qui
  suit une sous-ligne (« frais ») ; `toReviewExtra` ajoute un fragment sous le
  libellé d'un « à examiner », comme `gapExtra` pour un écart. Sans ces
  propriétés, une page n'affiche aucune sous-ligne. Une page **sans revue**
  (`api.getReview` absent) n'a ni exercice ni propositions : le mode reste,
  générique (aucun prestataire ne s'en sert aujourd'hui).

  Fusion de l'Epic 38 (38-12b) : à côté de l'import du fichier, des actions
  propres au prestataire (`importActions` : « Télécharger depuis PayPal ») et
  leurs notes (`importNotes`) ; un compte rendu propre au prestataire
  (`importReport`) peut remplacer le compte rendu commun ; l'exercice choisi et
  les exercices chargés se lisent (période proposée au téléchargement), et
  `reload()` relit la revue à la fin d'un téléchargement.

  Kesh PROPOSE, l'utilisateur ACCEPTE : rien n'est écrit à la lecture de la
  revue ; aucune proposition n'est cochée d'office. La revue se recharge après
  l'import et après l'acceptation ; une réponse périmée (autre compte, autre
  exercice, rechargement plus récent) est écartée par une garde de génération.
-->
<script
	lang="ts"
	generics="P extends ProviderProposalBase, B, R extends ProviderImportResponseBase"
>
	import { onMount, type Snippet } from 'svelte';
	import { Button } from '$lib/components/ui/button';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { errorMessageOf } from '$lib/shared/utils/error-message';
	import { todayInZurich } from '$lib/shared/utils/local-date';
	import { formatSwissDate } from '$lib/features/reports/reports.api';
	import { listBankAccounts, type BankAccountSummary } from '$lib/features/bank-accounts/bank-accounts.api';
	import { listFiscalYears } from '$lib/features/fiscal-years/fiscal-years.api';
	import type { FiscalYearResponse } from '$lib/features/fiscal-years/fiscal-years.types';
	import LinkSettlementModal from '$lib/features/reconciliation/LinkSettlementModal.svelte';
	import ProviderImportReport from './ProviderImportReport.svelte';
	import type { LinkTarget } from '$lib/features/reconciliation/reconciliation.types';
	import { chunkProposals } from './provider-review.api';
	import {
		defaultFiscalYearId,
		findingDate,
		importErrorCode,
		importErrorConcernsAccounts,
	} from './provider-review';
	import type {
		ProviderFailedProposal,
		ProviderFinding,
		ProviderImportResponseBase,
		ProviderPageApi,
		ProviderPageTexts,
		ProviderProposalBase,
		ProviderReview,
		ProviderReviewTransaction,
	} from './provider-review.types';

	interface Props {
		/** Préfixe des identifiants de test (`paypal`, `stripe`, `wise`). */
		testId: string;
		/** Story 39-11d — les types de fichier offerts au choix (défaut : CSV). */
		fileAccept?: string;
		/** Story 39-11d — plusieurs fichiers d'un geste (`api.importFiles`). */
		multiple?: boolean;
		/** Story 39-11d — le compte rendu entier, à la place des lignes par devise (Wise : par fichier). */
		importReportMain?: Snippet<[R]>;
		texts: ProviderPageTexts;
		api: ProviderPageApi<P, B, R>;
		/** Les libellés de compte du prestataire, tirés des comptes bancaires chargés. */
		accountLabels: (bankAccounts: BankAccountSummary[]) => string[];
		/** Les phrases d'un refus de l'import. */
		importErrorLines: (err: unknown) => string[];
		// Story 39-11d — les libellés et formes de la revue qui suivent sont facultatifs : une page
		// sans revue (`api.getReview` absent) ne les lit jamais ; les défauts sont les codes bruts.
		/** Libellé d'un refus de l'acceptation. */
		failedLabel?: (f: ProviderFailedProposal) => string;
		/** Clé stable d'une proposition et la transaction qui l'identifie. */
		proposalKey?: (p: P) => string;
		proposalTransactionId?: (p: P) => number;
		/** Genre d'une proposition (colonne « Genre »). */
		proposalKind?: (p: P) => string;
		/** Une proposition se coche-t-elle ? (défaut : oui) */
		selectable?: (p: P, key: string) => boolean;
		/** Les corps à envoyer pour la sélection, dans l'ordre des dates. */
		acceptBodies?: (proposals: readonly P[], selected: ReadonlySet<string>) => B[];
		/** Libellé et détails d'un écart. */
		gapLabel?: (f: ProviderFinding) => string;
		gapDetails?: (f: ProviderFinding) => string | null;
		/** Libellé d'un motif « à examiner ». */
		toReviewLabel?: (f: ProviderFinding) => string;
		/**
		 * Story 39-11d2 — les lignes d'une proposition hors sa transaction (sous-lignes ; défaut :
		 * aucune). La page Wise y rend les lignes d'un paiement par carte groupé.
		 */
		proposalLines?: (p: P) => ProviderReviewTransaction[];
		/**
		 * Story 39-11d2 — les lignes d'une ligne « à examiner » hors sa transaction (sous-lignes,
		 * chacune avec son bouton « Rattacher à une écriture » ; défaut : aucune). Un prestataire
		 * qui a des colonnes propres (`toReviewHead`) passe aussi `toReviewLineCells`.
		 */
		findingLines?: (f: ProviderFinding) => ProviderReviewTransaction[];
		/** Story 39-11d2 — le texte après une sous-ligne (« frais »), ou `null`. */
		lineLabel?: (t: ProviderReviewTransaction) => string | null;
		/** Story 39-11d2 — ce qui suit le libellé d'un « à examiner » (liens), facultatif. */
		toReviewExtra?: Snippet<[ProviderFinding]>;
		/**
		 * Story 39-11d2 — les cellules propres au prestataire d'une sous-ligne « à examiner »
		 * (mêmes colonnes que `toReviewCells`) ; données, la cellule du motif ne répète pas
		 * l'identifiant de la sous-ligne.
		 */
		toReviewLineCells?: Snippet<[ProviderReviewTransaction]>;
		/** Le compte rendu propre au prestataire, sous les lignes par devise. */
		importReportExtra?: Snippet<[R]>;
		/**
		 * Le compte rendu entier, propre au prestataire, à la place du compte rendu commun
		 * (PayPal : `PaypalImportReport`, partagé avec le dialogue de téléchargement, 38-12b).
		 */
		importReport?: Snippet<[R]>;
		/** Des actions à côté de l'import du fichier ; `busy` : import ou acceptation en cours (38-12b). */
		importActions?: Snippet<[{ busy: boolean }]>;
		/** Des notes sous la ligne de l'import (aide de l'Administrateur, 38-12b). */
		importNotes?: Snippet;
		/** La colonne « Ce qui sera fait » d'une proposition (sans objet sans revue). */
		proposalAction?: Snippet<[P, string]>;
		/** Ce qui suit le libellé d'un écart (un lien, par exemple), facultatif. */
		gapExtra?: Snippet<[ProviderFinding]>;
		/** Les en-têtes et les cellules propres au prestataire d'une ligne « à examiner ». */
		toReviewHead?: Snippet;
		toReviewCells?: Snippet<[ProviderFinding, number]>;
		/** Lectures propres au prestataire, au chargement de la page (libellés des règles…). */
		onLoad?: () => Promise<void>;
		/** Appelé à chaque revue relue (pour vider un état propre aux propositions). */
		onReviewLoaded?: () => void;
		/** Les propositions cochées (liables, pour qu'un choix propre au prestataire puisse en retirer une). */
		selected?: Set<string>;
		/** Vrai pendant une acceptation (lisible par les fragments). */
		accepting?: boolean;
		/** Le libellé du compte choisi (lisible par la page du prestataire). */
		selectedAccount?: string | null;
		/** Les comptes bancaires chargés (lisibles par la page du prestataire). */
		bankAccounts?: BankAccountSummary[];
		/** L'exercice choisi et les exercices chargés (lisibles par la page du prestataire, 38-12b). */
		selectedYearId?: number | null;
		fiscalYears?: FiscalYearResponse[];
	}

	let {
		testId,
		fileAccept = '.csv,text/csv',
		multiple = false,
		importReportMain,
		texts,
		api,
		accountLabels,
		importErrorLines,
		failedLabel = (f) => f.errorCode,
		proposalKey = (p) => String(p.transaction.id),
		proposalTransactionId = (p) => p.transaction.id,
		proposalKind = (p) => p.type,
		selectable = () => true,
		acceptBodies = () => [],
		gapLabel = (f) => f.code,
		gapDetails = () => null,
		toReviewLabel = (f) => f.code,
		proposalLines = () => [],
		findingLines = () => [],
		lineLabel = () => null,
		toReviewExtra,
		toReviewLineCells,
		importReportExtra,
		importReport: importReportView,
		importActions,
		importNotes,
		proposalAction,
		gapExtra,
		toReviewHead,
		toReviewCells,
		onLoad,
		onReviewLoaded,
		selected = $bindable(new Set<string>()),
		accepting = $bindable(false),
		selectedAccount = $bindable(null),
		bankAccounts = $bindable([]),
		selectedYearId = $bindable(null),
		fiscalYears = $bindable([]),
	}: Props = $props();

	let loading = $state(true);
	let loadError = $state<string | null>(null);

	// Import
	let fileInput = $state<HTMLInputElement | null>(null);
	let importing = $state(false);
	let importReport = $state<R | null>(null);
	let importErrors = $state<string[]>([]);
	// Story 40-9 — le code du refus d'import : le lien « Comptes bancaires » n'est
	// proposé que pour un refus qui se règle à cet écran.
	let importErrorCodeValue = $state<string | null>(null);

	// Revue
	let review = $state<ProviderReview<P> | null>(null);
	let reviewLoading = $state(false);
	let reviewError = $state<string | null>(null);
	let reviewGen = 0;

	// Acceptation
	let batchProgress = $state<{ current: number; total: number } | null>(null);
	let acceptReport = $state<{ accepted: number; failed: ProviderFailedProposal[] } | null>(null);
	let acceptError = $state<string | null>(null);

	// Rattachement d'une ligne « à examiner » (modale du rapprochement)
	let linkOpen = $state(false);
	let linkTarget = $state<LinkTarget | null>(null);

	let providerAccounts = $derived(accountLabels(bankAccounts));
	// Story 39-11d — une page sans revue (`api.getReview` absent) n'a ni exercice ni propositions.
	let hasReview = $derived(api.getReview !== undefined);

	onMount(async () => {
		// Revue P1 de la 39-11d (L7) — une page sans revue ne lit pas les exercices : un échec de
		// cette lecture, dont elle n'a que faire, ne doit pas masquer son import.
		const years = hasReview ? listFiscalYears() : Promise.resolve<FiscalYearResponse[]>([]);
		const [ba, fy] = await Promise.allSettled([listBankAccounts(), years, onLoad?.()]);
		if (ba.status === 'fulfilled') bankAccounts = ba.value;
		else loadError = errorMessageOf(ba.reason);
		if (fy.status === 'fulfilled') fiscalYears = fy.value;
		else loadError = errorMessageOf(fy.reason);
		selectedAccount = providerAccounts[0] ?? null;
		// Story 40-9 — le jour de Zurich, comme la période proposée au téléchargement.
		selectedYearId = defaultFiscalYearId(fiscalYears, todayInZurich());
		loading = false;
		await reload();
	});

	/** Relit la revue du compte et de l'exercice choisis ; une réponse périmée est écartée. */
	export async function reload() {
		const account = selectedAccount;
		const yearId = selectedYearId;
		const myGen = ++reviewGen;
		const getReview = api.getReview;
		if (account === null || yearId === null || getReview === undefined) {
			review = null;
			reviewLoading = false;
			return;
		}
		reviewLoading = true;
		reviewError = null;
		try {
			const r = await getReview(account, yearId);
			if (myGen !== reviewGen) return;
			review = r;
			selected = new Set();
			onReviewLoaded?.();
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
		importErrorCodeValue = null;
		acceptReport = null;
		await reload();
	}

	async function onFileChosen(e: Event) {
		const input = e.currentTarget as HTMLInputElement;
		const files = Array.from(input.files ?? []);
		input.value = '';
		if (files.length === 0 || selectedAccount === null) return;
		importing = true;
		importErrors = [];
		importErrorCodeValue = null;
		try {
			// Story 39-11d — plusieurs fichiers d'un geste quand la page les prend (Wise).
			if (api.importFiles) importReport = await api.importFiles(files, selectedAccount);
			else if (api.importFile) importReport = await api.importFile(files[0], selectedAccount);
			else return;
			await reload();
		} catch (err) {
			importReport = null;
			importErrors = importErrorLines(err);
			importErrorCodeValue = importErrorCode(err);
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

	function toggleAll(on: boolean) {
		if (!review) return;
		selected = on
			? new Set(review.proposals.filter((p) => selectable(p, proposalKey(p))).map(proposalKey))
			: new Set();
	}

	/**
	 * Ouvre la modale de rattachement sur une transaction : celle d'une ligne « à examiner »,
	 * ou celle d'une de ses sous-lignes (39-11d2).
	 */
	function openLink(t: ProviderReviewTransaction | undefined) {
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

	/** Accepte la sélection, par paquets de 500, un à la fois, dans l'ordre des dates. */
	async function acceptSelection() {
		// Le compte est figé avant la boucle (revue P1 de la 38-11d, L1) : les sélecteurs sont
		// désactivés pendant l'acceptation, et chaque paquet part au même compte.
		const account = selectedAccount;
		const accept = api.accept;
		if (!review || account === null || accept === undefined) return;
		const bodies = acceptBodies(review.proposals, selected);
		if (bodies.length === 0) return;
		const batches = chunkProposals(bodies);
		accepting = true;
		acceptError = null;
		const report = { accepted: 0, failed: [] as ProviderFailedProposal[] };
		try {
			for (let i = 0; i < batches.length; i++) {
				batchProgress = { current: i + 1, total: batches.length };
				const res = await accept(account, batches[i]);
				report.accepted += res.accepted.length;
				report.failed.push(...res.failed);
			}
		} catch (err) {
			acceptError = errorMessageOf(err);
		} finally {
			acceptReport = report;
			batchProgress = null;
			accepting = false;
			await reload();
		}
	}
</script>

<!-- Le compte rendu propre au prestataire à la place des lignes par devise (Wise, 39-11d), dans le compte rendu commun. -->
{#snippet importReportMainView()}
	{#if importReport}{@render importReportMain?.(importReport)}{/if}
{/snippet}

<!-- Story 39-11d2 — l'identifiant d'une sous-ligne, et le texte qui le suit (« frais »). -->
{#snippet subLineId(line: ProviderReviewTransaction)}
	{@const label = lineLabel(line)}
	<span class="font-mono text-xs whitespace-nowrap">↳ {line.transactionId ?? ''}</span>
	{#if label !== null}<span class="ml-1 text-xs" data-testid="{testId}-line-label-{line.id}">{label}</span>{/if}
{/snippet}

<!-- Story 39-11d2 — une sous-ligne d'une proposition : sans case à cocher ni action (la proposition les porte). -->
{#snippet subLine(line: ProviderReviewTransaction, rowTestId: string)}
	<tr class="border-b border-border text-text-muted" data-testid={rowTestId}>
		<td class="py-1 pr-2"></td>
		<td class="py-1 pr-4 whitespace-nowrap">{formatSwissDate(line.date)}</td>
		<td class="py-1 pr-4"></td>
		<td class="py-1 pr-4 text-right font-mono whitespace-nowrap">{line.amount} {line.currency}</td>
		<td class="py-1 pr-4">{line.name ?? ''}</td>
		<td class="py-1 pr-4">{@render subLineId(line)}</td>
		<td class="py-1"></td>
	</tr>
{/snippet}

<svelte:head>
	<title>{texts.title} - Kesh</title>
</svelte:head>

<h1 class="text-2xl font-semibold text-text" data-testid="{testId}-page-title">{texts.title}</h1>
<p class="mt-2 text-sm text-text-muted">{texts.subtitle}</p>

{#if loading}
	<p class="mt-6 text-text-muted">{i18nMsg('provider-loading', 'Chargement…')}</p>
{:else if loadError}
	<p class="mt-6 text-red-600" role="alert" data-testid="{testId}-load-error">{loadError}</p>
{:else if providerAccounts.length === 0}
	<p class="mt-6 text-text-muted" data-testid="{testId}-no-account">
		{texts.noAccount}
		<a href="/bank-accounts" class="ml-1 text-primary underline" data-testid="{testId}-no-account-link">
			{i18nMsg('provider-bank-accounts-link', 'Comptes bancaires')}
		</a>
	</p>
{:else}
	<div class="mt-6 flex flex-wrap items-end gap-4">
		<div>
			<label for="{testId}-account" class="block text-sm font-medium text-text mb-1">{texts.accountLabel}</label>
			<select
				id="{testId}-account"
				bind:value={selectedAccount}
				onchange={onSelectionChange}
				disabled={accepting || importing}
				class="rounded border border-border px-3 py-2 text-sm"
				data-testid="{testId}-account"
			>
				{#each providerAccounts as label (label)}
					<option value={label}>{label}</option>
				{/each}
			</select>
		</div>
		{#if hasReview}
		<div>
			<label for="{testId}-fiscal-year" class="block text-sm font-medium text-text mb-1">
				{i18nMsg('provider-label-fiscal-year', 'Exercice')}
			</label>
			<select
				id="{testId}-fiscal-year"
				bind:value={selectedYearId}
				onchange={onSelectionChange}
				disabled={accepting || importing}
				class="rounded border border-border px-3 py-2 text-sm"
				data-testid="{testId}-fiscal-year"
			>
				{#each fiscalYears as fy (fy.id)}
					<option value={fy.id}>{fy.name}</option>
				{/each}
			</select>
		</div>
		{/if}
		<div>
			<input
				bind:this={fileInput}
				type="file"
				accept={fileAccept}
				{multiple}
				class="hidden"
				onchange={onFileChosen}
				data-testid="{testId}-file-input"
			/>
			<Button onclick={() => fileInput?.click()} disabled={importing || accepting} data-testid="{testId}-import-button">
				{importing ? i18nMsg('provider-import-running', 'Import en cours…') : texts.importButton}
			</Button>
		</div>
		{@render importActions?.({ busy: importing || accepting })}
	</div>
	{#if texts.importHelp}
		<p class="mt-2 text-sm text-text" data-testid="{testId}-import-help">{texts.importHelp}</p>
	{/if}
	{@render importNotes?.()}
	{#if texts.hint}
		<p class="mt-2 text-sm text-text-muted" data-testid="{testId}-hint">{texts.hint}</p>
	{/if}

	{#if importErrors.length > 0}
		<div class="mt-4 rounded border border-red-200 bg-red-50 p-3 text-sm" role="alert" data-testid="{testId}-import-error">
			{#each importErrors as line, i (i)}
				<p data-testid="{testId}-import-error-line">{line}</p>
			{/each}
			{#if importErrorConcernsAccounts(importErrorCodeValue)}
				<a href="/bank-accounts" class="text-primary underline" data-testid="{testId}-import-error-accounts-link">
					{i18nMsg('provider-bank-accounts-link', 'Comptes bancaires')}
				</a>
			{/if}
		</div>
	{/if}

	{#if importReport}
		{#if importReportView}
			{@render importReportView(importReport)}
		{:else}
			<ProviderImportReport
				report={importReport}
				testid="{testId}-import-report"
				main={importReportMain ? importReportMainView : undefined}
			>
				{@render importReportExtra?.(importReport)}
			</ProviderImportReport>
		{/if}
	{/if}

	{#if reviewError}
		<p class="mt-4 text-red-600" role="alert" data-testid="{testId}-review-error">{reviewError}</p>
	{/if}
	{#if reviewLoading && !review}
		<p class="mt-4 text-text-muted">{i18nMsg('provider-loading', 'Chargement…')}</p>
	{/if}

	{#if acceptReport}
		<section class="mt-4 rounded border border-border bg-surface-alt p-3 text-sm" data-testid="{testId}-accept-report">
			<p>
				{i18nMsg('provider-accept-report', '{ $accepted } proposition(s) acceptée(s), { $failed } refusée(s).', {
					accepted: String(acceptReport.accepted),
					failed: String(acceptReport.failed.length),
				})}
			</p>
			{#if acceptReport.failed.length > 0}
				<ul class="mt-1 list-disc pl-5" data-testid="{testId}-accept-failed">
					{#each acceptReport.failed as f, i (i)}
						<li>#{f.bankTransactionId} — {failedLabel(f)}</li>
					{/each}
				</ul>
			{/if}
		</section>
	{/if}
	{#if acceptError}
		<p class="mt-2 text-red-600" role="alert" data-testid="{testId}-accept-error">{acceptError}</p>
	{/if}

	{#if review}
		<!-- Propositions -->
		<section class="mt-6" data-testid="{testId}-proposals">
			<h2 class="text-lg font-semibold">
				{i18nMsg('provider-proposals-title', 'Propositions ({ $count })', {
					count: String(review.proposals.length),
				})}
			</h2>
			{#if review.proposals.length === 0}
				<p class="text-text-muted" data-testid="{testId}-proposals-empty">
					{i18nMsg('provider-proposals-empty', 'Aucune proposition.')}
				</p>
			{:else}
				<div class="mt-2 flex items-center gap-3">
					<Button variant="outline" onclick={() => toggleAll(true)} disabled={accepting} data-testid="{testId}-select-all">
						{i18nMsg('provider-select-all', 'Tout cocher')}
					</Button>
					<Button variant="ghost" onclick={() => toggleAll(false)} disabled={accepting} data-testid="{testId}-select-none">
						{i18nMsg('provider-select-none', 'Tout décocher')}
					</Button>
					<Button onclick={acceptSelection} disabled={accepting || importing || reviewLoading || selected.size === 0} data-testid="{testId}-accept">
						{i18nMsg('provider-accept', 'Accepter la sélection')}
					</Button>
					{#if batchProgress}
						<span class="text-sm text-text-muted" data-testid="{testId}-accept-progress">
							{i18nMsg('provider-accept-progress', 'Lot { $current } sur { $total }', {
								current: String(batchProgress.current),
								total: String(batchProgress.total),
							})}
						</span>
					{/if}
				</div>
				<table class="mt-2 w-full text-sm" data-testid="{testId}-proposals-table">
					<thead>
						<tr class="border-b border-border text-left">
							<th class="py-2 pr-2"></th>
							<th class="py-2 pr-4 font-semibold">{i18nMsg('provider-col-date', 'Date')}</th>
							<th class="py-2 pr-4 font-semibold">{i18nMsg('provider-col-kind', 'Genre')}</th>
							<th class="py-2 pr-4 text-right font-semibold">{i18nMsg('provider-col-amount', 'Montant')}</th>
							<th class="py-2 pr-4 font-semibold">{i18nMsg('provider-col-name', 'Nom')}</th>
							<th class="py-2 pr-4 font-semibold">{texts.idColumn}</th>
							<th class="py-2 font-semibold">{i18nMsg('provider-col-action', 'Ce qui sera fait')}</th>
						</tr>
					</thead>
					<tbody>
						{#each review.proposals as p (proposalKey(p))}
							{@const key = proposalKey(p)}
							<tr class="border-b border-border" data-testid="{testId}-proposal-{proposalTransactionId(p)}">
								<td class="py-2 pr-2">
									<input
										type="checkbox"
										checked={selected.has(key)}
										disabled={accepting || !selectable(p, key)}
										onchange={(e) => toggle(key, (e.currentTarget as HTMLInputElement).checked)}
										aria-label={i18nMsg('provider-select-one', 'Choisir cette proposition')}
										data-testid="{testId}-check-{proposalTransactionId(p)}"
									/>
								</td>
								<td class="py-2 pr-4 whitespace-nowrap">{formatSwissDate(p.transaction.date)}</td>
								<td class="py-2 pr-4">{proposalKind(p)}</td>
								<td class="py-2 pr-4 text-right font-mono whitespace-nowrap">
									{p.transaction.amount} {p.transaction.currency}
									{#if p.otherTransaction}
										<span class="block text-xs text-text-muted">{p.otherTransaction.amount} {p.otherTransaction.currency}</span>
									{/if}
								</td>
								<td class="py-2 pr-4">{p.transaction.name ?? ''}</td>
								<td class="py-2 pr-4 font-mono text-xs">{p.transaction.transactionId ?? ''}</td>
								<td class="py-2">{@render proposalAction?.(p, key)}</td>
							</tr>
							{#each proposalLines(p) as line (line.id)}
								{@render subLine(line, `${testId}-proposal-line-${line.id}`)}
							{/each}
						{/each}
					</tbody>
				</table>
			{/if}
		</section>

		<!-- Écarts -->
		<section class="mt-8" data-testid="{testId}-gaps">
			<h2 class="text-lg font-semibold">
				{i18nMsg('provider-gaps-title', 'Écarts ({ $count })', { count: String(review.gaps.length) })}
			</h2>
			{#if review.gaps.length === 0}
				<p class="text-text-muted" data-testid="{testId}-gaps-empty">{i18nMsg('provider-gaps-empty', 'Aucun écart.')}</p>
			{:else}
				<table class="mt-2 w-full text-sm" data-testid="{testId}-gaps-table">
					<thead>
						<tr class="border-b border-border text-left">
							<th class="py-2 pr-4 font-semibold">{i18nMsg('provider-col-date', 'Date')}</th>
							<th class="py-2 pr-4 font-semibold">{i18nMsg('provider-col-gap', 'Écart')}</th>
							<th class="py-2 pr-4 text-right font-semibold">{i18nMsg('provider-col-amount', 'Montant')}</th>
							<th class="py-2 pr-4 font-semibold">{i18nMsg('provider-col-name', 'Nom')}</th>
							<th class="py-2 pr-4 font-semibold">{texts.idColumn}</th>
							<th class="py-2 font-semibold">{i18nMsg('provider-col-entry', 'Écriture')}</th>
						</tr>
					</thead>
					<tbody>
						{#each review.gaps as g, i (i)}
							{@const entry = g.entries?.[0]}
							{@const details = gapDetails(g)}
							<tr class="border-b border-border" data-testid="{testId}-gap-{g.code}">
								<td class="py-2 pr-4 whitespace-nowrap">{findingDate(g)}</td>
								<td class="py-2 pr-4">
									{gapLabel(g)}
									{#if details !== null}
										<span class="block text-xs text-text-muted" data-testid="{testId}-gap-details">{details}</span>
									{/if}
									{@render gapExtra?.(g)}
								</td>
								<td class="py-2 pr-4 text-right font-mono whitespace-nowrap">
									{#if g.transaction}{g.transaction.amount} {g.transaction.currency}{/if}
								</td>
								<td class="py-2 pr-4">{g.transaction?.name ?? entry?.description ?? ''}</td>
								<td class="py-2 pr-4 font-mono text-xs">{g.transaction?.transactionId ?? entry?.ids.join(', ') ?? ''}</td>
								<td class="py-2">
									{#if entry}
										<a class="text-primary underline" href="/journal-entries/{entry.journalEntryId}">
											{i18nMsg('provider-entry-number', 'n° { $number }', {
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
		<section class="mt-8" data-testid="{testId}-to-review">
			<h2 class="text-lg font-semibold">
				{i18nMsg('provider-to-review-title', 'À examiner ({ $count })', {
					count: String(review.toReview.length),
				})}
			</h2>
			{#if review.toReview.length === 0}
				<p class="text-text-muted" data-testid="{testId}-to-review-empty">
					{i18nMsg('provider-to-review-empty', 'Rien à examiner.')}
				</p>
			{:else}
				<table class="mt-2 w-full text-sm" data-testid="{testId}-to-review-table">
					<thead>
						<tr class="border-b border-border text-left">
							<th class="py-2 pr-4 font-semibold">{i18nMsg('provider-col-date', 'Date')}</th>
							<th class="py-2 pr-4 font-semibold">{i18nMsg('provider-col-reason', 'Motif')}</th>
							<th class="py-2 pr-4 text-right font-semibold">{i18nMsg('provider-col-amount', 'Montant')}</th>
							{@render toReviewHead?.()}
							<th class="py-2"></th>
						</tr>
					</thead>
					<tbody>
						{#each review.toReview as t, i (i)}
							{@const tx = t.transaction}
							<tr class="border-b border-border" data-testid="{testId}-to-review-{tx?.id ?? `e${i}`}">
								<td class="py-2 pr-4 whitespace-nowrap">{findingDate(t)}</td>
								<td class="py-2 pr-4">
									{toReviewLabel(t)}
									{@render toReviewExtra?.(t)}
								</td>
								<td class="py-2 pr-4 text-right font-mono whitespace-nowrap">{#if tx}{tx.amount} {tx.currency}{/if}</td>
								{@render toReviewCells?.(t, i)}
								<td class="py-2 whitespace-nowrap">
									<button
										type="button"
										class="rounded border border-border px-2 py-1 text-xs"
										onclick={() => openLink(tx)}
										disabled={!tx}
										data-testid="{testId}-link-{tx?.id ?? `e${i}`}"
									>
										{i18nMsg('provider-link-entry', 'Rattacher à une écriture')}
									</button>
									<a href="/journal-entries" class="ml-1 text-xs text-primary underline">
										{i18nMsg('provider-manual-entry', 'Écriture manuelle')}
									</a>
								</td>
							</tr>
							{#each findingLines(t) as line (line.id)}
								<tr class="border-b border-border text-text-muted" data-testid="{testId}-to-review-line-{line.id}">
									<td class="py-1 pr-4 pl-4 whitespace-nowrap">{formatSwissDate(line.date)}</td>
									<td class="py-1 pr-4">
										{#if toReviewLineCells}
											{@const label = lineLabel(line)}
											<span class="text-xs">↳ {label ?? ''}</span>
										{:else}
											{@render subLineId(line)}
										{/if}
									</td>
									<td class="py-1 pr-4 text-right font-mono whitespace-nowrap">{line.amount} {line.currency}</td>
									{@render toReviewLineCells?.(line)}
									<td class="py-1 whitespace-nowrap">
										<button
											type="button"
											class="rounded border border-border px-2 py-1 text-xs"
											onclick={() => openLink(line)}
											data-testid="{testId}-link-line-{line.id}"
										>
											{i18nMsg('provider-link-entry', 'Rattacher à une écriture')}
										</button>
									</td>
								</tr>
							{/each}
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
	onSuccess={() => void reload()}
/>
