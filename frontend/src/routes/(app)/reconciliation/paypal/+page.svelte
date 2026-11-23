<!--
  Story 38-11d (AC-3 à AC-5) — la page PayPal : choix du compte PayPal et de
  l'exercice, import du CSV « Balance affecting transactions » et son compte
  rendu, propositions à cocher et à accepter par paquets de 500, écarts et
  lignes « à examiner » (rattachement par la modale du rapprochement).

  Story 40-7d — la page est la page commune des prestataires
  (`ProviderReviewPage`, généralisée de celle-ci) ; ce qui reste ici est propre
  à PayPal : ses textes, les libellés des règles, le compte de destination d'un
  retrait, les lignes écartées du compte rendu, les colonnes de ses lignes
  « à examiner ».

  Story 38-12b (AC-1 à AC-3) — à côté de l'import du CSV, « Télécharger depuis
  PayPal » pour un compte dont le libellé a ses identifiants dans
  l'environnement du serveur (`providerApi.configured` de la liste des comptes
  bancaires) : le dialogue générique `ProviderDownloadDialog` lance la tâche, en
  suit la progression et montre le même compte rendu que l'import du CSV
  (`PaypalImportReport`, branché aussi sur la page commune par son fragment
  `importReport`) ; la revue se recharge à la fin. Sans identifiants, l'import
  du CSV reste seul, et l'Administrateur lit une ligne d'aide.
-->
<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { todayInZurich } from '$lib/shared/utils/local-date';
	import { authState } from '$lib/app/stores/auth.svelte';
	import { fetchAccounts } from '$lib/features/accounts/accounts.api';
	import { withoutForeignCurrency, type AccountResponse } from '$lib/features/accounts/accounts.types';
	import type { BankAccountSummary } from '$lib/features/bank-accounts/bank-accounts.api';
	import {
		accountLabelKey,
		paypalAccountLabels,
		paypalApiStatus,
		providerSettingsOf,
	} from '$lib/features/bank-accounts/provider';
	import type { FiscalYearResponse } from '$lib/features/fiscal-years/fiscal-years.types';
	import ProviderReviewPage from '$lib/features/provider/ProviderReviewPage.svelte';
	import ProviderDownloadDialog from '$lib/features/provider-download/ProviderDownloadDialog.svelte';
	import { defaultPeriod } from '$lib/features/provider-download/provider-download';
	import PaypalImportReport from '$lib/features/paypal/PaypalImportReport.svelte';
	import {
		acceptPaypalProposals,
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
		proposalActionLabel,
		proposalKey,
		proposalTransactionId,
		selectedBodies,
		withdrawalAccountCandidates,
	} from '$lib/features/paypal/paypal-review';
	import { listRules } from '$lib/features/reconciliation/rules/rules.api';
	import type {
		PaypalAcceptProposal,
		PaypalDownloadResult,
		PaypalFinding,
		PaypalImportResponse,
		PaypalProposal,
	} from '$lib/features/paypal/paypal.types';

	let accounts = $state<AccountResponse[]>([]);
	// Libellés des règles, par identifiant : la revue ne sert que `ruleId`.
	let ruleLabels = $state<Map<number, string>>(new Map());
	let bankAccounts = $state<BankAccountSummary[]>([]);
	let selectedAccount = $state<string | null>(null);
	let selected = $state<Set<string>>(new Set());
	let accepting = $state(false);
	let withdrawalAccounts = $state<Record<string, number | null>>({});
	let selectedYearId = $state<number | null>(null);
	let fiscalYears = $state<FiscalYearResponse[]>([]);
	// La page commune, pour relire la revue à la fin d’un téléchargement (`reload`, exporté).
	let reviewPage = $state<{ reload: () => Promise<void> } | null>(null);

	// Téléchargement par l'API (38-12b, AC-1) : l'état du compte choisi, le rôle, la période proposée.
	let downloadOpen = $state(false);
	let downloadApi = $derived(selectedAccount === null ? null : paypalApiStatus(bankAccounts, selectedAccount));
	let role = $derived(authState.currentUser?.role);
	let canDownload = $derived(role === 'Admin' || role === 'Comptable');
	let downloadPeriod = $derived(
		defaultPeriod(fiscalYears.find((fy) => fy.id === selectedYearId) ?? null, todayInZurich()),
	);

	// fx-filtre-comptes : la liste des retraits est filtrée en CHF (`withdrawalAccountCandidates`,
	// après `withoutForeignCurrency`) ; les autres comptes ne servent qu'aux libellés.
	// Le compte du plan du compte PayPal choisi est refusé par le serveur comme destination.
	let ownLedgerIds = $derived(
		bankAccounts
			.filter((b) => {
				const settings = providerSettingsOf(b, 'paypal');
				return (
					settings !== null &&
					selectedAccount !== null &&
					accountLabelKey(settings.account) === accountLabelKey(selectedAccount) &&
					b.journalAccountId !== null
				);
			})
			.map((b) => b.journalAccountId as number),
	);
	let withdrawalChoices = $derived(
		withdrawalAccountCandidates(withoutForeignCurrency(accounts), ownLedgerIds),
	);

	/** Lectures propres à PayPal : comptes du plan et libellés des règles (best-effort). */
	async function loadPaypal() {
		const [acc, rules] = await Promise.allSettled([fetchAccounts(false), allRules()]);
		ruleLabels =
			rules.status === 'fulfilled' ? new Map(rules.value.map((r) => [r.id, r.label])) : new Map();
		accounts = acc.status === 'fulfilled' ? acc.value : [];
	}

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

	function needsAccount(p: PaypalProposal): boolean {
		return p.type === 'withdrawal' && p.counterpartyAccountId === null;
	}

	/** Une proposition se coche-t-elle ? Un retrait sans compte de destination, non. */
	function selectable(p: PaypalProposal, key: string): boolean {
		return !needsAccount(p) || (withdrawalAccounts[key] ?? null) !== null;
	}

	function chooseWithdrawal(key: string, value: string) {
		const id = value === '' ? null : Number(value);
		withdrawalAccounts = { ...withdrawalAccounts, [key]: id };
		if (id === null && selected.has(key)) {
			const next = new Set(selected);
			next.delete(key);
			selected = next;
		}
	}

</script>

{#snippet importReport(report: PaypalImportResponse)}
	<PaypalImportReport {report} />
{/snippet}

{#snippet importActions({ busy }: { busy: boolean })}
	{#if canDownload && downloadApi?.configured}
		<div>
			<Button
				variant="outline"
				onclick={() => (downloadOpen = true)}
				disabled={busy}
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
{/snippet}

{#snippet importNotes()}
	{#if role === 'Admin' && downloadApi !== null && !downloadApi.configured}
		<p class="mt-2 text-xs text-text-muted" data-testid="paypal-download-not-configured">
			{i18nMsg(
				'paypal-download-not-configured',
				'Téléchargement depuis PayPal non configuré pour ce compte : voir le manuel d’administration.',
			)}
		</p>
	{/if}
{/snippet}

{#snippet proposalAction(p: PaypalProposal, key: string)}
	{#if needsAccount(p)}
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
{/snippet}

{#snippet toReviewHead()}
	<th class="py-2 pr-4 font-semibold">{i18nMsg('paypal-col-type', 'Type PayPal')}</th>
	<th class="py-2 pr-4 font-semibold">{i18nMsg('paypal-col-name', 'Nom')}</th>
	<th class="py-2 pr-4 font-semibold">{i18nMsg('paypal-col-email', 'E-mail')}</th>
{/snippet}

{#snippet toReviewCells(t: PaypalFinding)}
	{@const tx = t.transaction}
	<td class="py-2 pr-4">{tx?.providerType ?? ''}</td>
	<td class="py-2 pr-4">{tx?.name ?? ''}</td>
	<td class="py-2 pr-4 text-xs">{tx?.counterpartyEmail ?? ''}</td>
{/snippet}

<ProviderReviewPage
	bind:this={reviewPage}
	bind:selected
	bind:selectedYearId
	bind:fiscalYears
	bind:accepting
	bind:selectedAccount
	bind:bankAccounts
	testId="paypal"
	texts={{
		title: i18nMsg('paypal-page-title', 'PayPal'),
		subtitle: i18nMsg(
			'paypal-page-subtitle',
			'Importez le relevé CSV d’un compte PayPal, puis acceptez les propositions de rapprochement. Rien n’est écrit sans votre acceptation.',
		),
		noAccount: i18nMsg(
			'paypal-no-account',
			'Aucun compte PayPal. Créez un compte bancaire par devise, case « Compte PayPal » cochée, dans les comptes bancaires.',
		),
		accountLabel: i18nMsg('paypal-label-account', 'Compte PayPal'),
		importButton: i18nMsg('paypal-import-button', 'Importer le CSV PayPal'),
		idColumn: i18nMsg('paypal-col-txn', 'TXN'),
	}}
	api={{ importFile: importPaypalCsv, getReview: getPaypalReview, accept: acceptPaypalProposals }}
	accountLabels={paypalAccountLabels}
	importErrorLines={paypalImportErrorLines}
	failedLabel={(f) => paypalFailedLabel(f.errorCode, f.details)}
	{proposalKey}
	{proposalTransactionId}
	proposalKind={(p: PaypalProposal) => proposalKindLabel(p, ruleLabels)}
	{selectable}
	acceptBodies={(proposals, sel): PaypalAcceptProposal[] => selectedBodies(proposals, sel, withdrawalAccounts)}
	gapLabel={findingGapLabel}
	gapDetails={findingGapDetails}
	toReviewLabel={(f) => toReviewReasonLabel(f.code)}
	onLoad={loadPaypal}
	onReviewLoaded={() => (withdrawalAccounts = {})}
	{importReport}
	{importActions}
	{importNotes}
	{proposalAction}
	{toReviewHead}
	{toReviewCells}
/>

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
		onDone={() => void reviewPage?.reload()}
	>
		{#snippet report(result: PaypalDownloadResult)}
			<PaypalImportReport report={result} testid="paypal-download-report" />
		{/snippet}
	</ProviderDownloadDialog>
{/if}
