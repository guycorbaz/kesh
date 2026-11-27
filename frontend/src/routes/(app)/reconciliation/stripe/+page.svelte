<!--
  Story 40-7d (AC-2, AC-3) — la page Stripe, sur la page commune des
  prestataires (`ProviderReviewPage`, généralisée de la page PayPal de la
  38-11d) : choix du compte Stripe et de l'exercice, import du rapport
  « Itemized payout reconciliation » et son compte rendu (par devise, et les
  versements lus), propositions à cocher et à accepter (rattachement d'une
  charge à sa vente, frais, versement), écarts et lignes « à examiner ».

  Ce qui reste ici est propre à Stripe : ses textes, son client d'API, ses
  libellés, le rappel des ventes des boutiques, les versements du compte rendu,
  et, pour un versement sans réception, les liens vers son écriture et vers la
  réconciliation, où sa réception se rattache (compte de destination ordinaire ;
  vers un compte Wise : le texte qui dit d'importer son relevé).

  Story 40-8b — à côté de l'import du rapport, « Télécharger depuis Stripe »
  pour un compte dont le libellé a sa clé restreinte dans l'environnement du
  serveur (`providerApi.configured` de la liste des comptes bancaires) : le
  dialogue générique de la 38-12b (`ProviderDownloadDialog`) lance la tâche, en
  suit la progression par versement et montre le même compte rendu que l'import
  du rapport (`StripeImportReport`), plus les versements manuels non lus ; la
  revue se recharge à la fin. Sans clé, l'import du rapport reste seul, et
  l'Administrateur lit une ligne d'aide. Le mode d'essai (clé `rk_test_`) se
  montre « bac à sable ».
-->
<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { todayInZurich } from '$lib/shared/utils/local-date';
	import { authState } from '$lib/app/stores/auth.svelte';
	import { fetchAccounts } from '$lib/features/accounts/accounts.api';
	import type { AccountResponse } from '$lib/features/accounts/accounts.types';
	import type { BankAccountSummary } from '$lib/features/bank-accounts/bank-accounts.api';
	import { listBankAccounts } from '$lib/features/bank-accounts/bank-accounts.api';
	import { stripeAccountLabels, stripeApiStatus } from '$lib/features/bank-accounts/provider';
	import type { FiscalYearResponse } from '$lib/features/fiscal-years/fiscal-years.types';
	import ProviderReviewPage from '$lib/features/provider/ProviderReviewPage.svelte';
	import ProviderDownloadDialog from '$lib/features/provider-download/ProviderDownloadDialog.svelte';
	import { defaultPeriod } from '$lib/features/provider-download/provider-download';
	import StripeImportReport from '$lib/features/stripe/StripeImportReport.svelte';
	import type { ProviderFinding } from '$lib/features/provider/provider-review.types';
	import { acceptStripeProposals, getStripeReview, importStripeReport } from '$lib/features/stripe/stripe.api';
	import {
		stripeFailedLabel,
		stripeGapDetails,
		stripeGapLabel,
		stripeImportErrorLines,
		stripeLineErrorLabel,
		stripeProposalActionLabel,
		stripeProposalKindLabel,
		stripeToReviewLabel,
	} from '$lib/features/stripe/stripe-labels';
	import {
		stripeProposalKey,
		stripeProposalTransactionId,
		stripeSelectedBodies,
	} from '$lib/features/stripe/stripe-review';
	import type {
		StripeDownloadResult,
		StripeImportResponse,
		StripeProposal,
	} from '$lib/features/stripe/stripe.types';

	// Comptes du plan : libellé du compte de frais d'une proposition (best-effort).
	// fx-filtre-comptes : exempté — aucun sélecteur de compte : la liste ne sert qu'à nommer le
	// compte de frais que la revue a choisi (en CHF ou dans la devise du solde).
	let accounts = $state<AccountResponse[]>([]);
	// Tous les comptes bancaires, archivés et clos compris (le serveur ne les écarte pas d'une
	// destination de versement) : de quoi savoir où un versement arrive.
	let allBankAccounts = $state<BankAccountSummary[]>([]);

	// Téléchargement par l'API (40-8b) : la page commune lit les comptes et l'exercice, la page
	// Stripe en tire l'état du compte choisi, le rôle et la période proposée.
	let bankAccounts = $state<BankAccountSummary[]>([]);
	let selectedAccount = $state<string | null>(null);
	let selectedYearId = $state<number | null>(null);
	let fiscalYears = $state<FiscalYearResponse[]>([]);
	let reviewPage = $state<{ reload: () => Promise<void> } | null>(null);
	let downloadOpen = $state(false);
	let downloadApi = $derived(selectedAccount === null ? null : stripeApiStatus(bankAccounts, selectedAccount));
	let role = $derived(authState.currentUser?.role);
	let canDownload = $derived(role === 'Admin' || role === 'Comptable');
	let downloadPeriod = $derived(
		defaultPeriod(fiscalYears.find((fy) => fy.id === selectedYearId) ?? null, todayInZurich()),
	);

	async function loadStripe() {
		try {
			accounts = await fetchAccounts(false);
		} catch {
			accounts = [];
		}
		try {
			allBankAccounts = await listBankAccounts(true, true);
		} catch {
			allBankAccounts = [];
		}
	}

	/** L'écriture d'un versement sans réception (`details.journalEntryId`), `null` sinon. */
	function payoutEntryId(g: ProviderFinding): number | null {
		const id = g.details?.journalEntryId;
		return g.code === 'payout_without_receipt' && typeof id === 'number' ? id : null;
	}

	/**
	 * Revue P1 de 40-7d (M3) — le compte de destination d'un versement sans réception est-il un
	 * compte de prestataire (Wise) ? La réconciliation bancaire n'offre pas ces comptes : sa
	 * réception se rattache à l'import du relevé du prestataire. Les comptes archivés et clos sont
	 * relus (revue P2, P2-3) : seul un compte supprimé, ou une lecture des comptes en échec, donne le
	 * renvoi ordinaire. Story 39-11d : le texte mène à la page Wise, où ce relevé s'importe.
	 */
	function payoutGoesToProvider(g: ProviderFinding): boolean {
		const id = g.details?.payoutBankAccountId;
		const dest = typeof id === 'number' ? allBankAccounts.find((b) => b.id === id) : undefined;
		return dest?.provider === 'wise';
	}
</script>

{#snippet importReport(report: StripeImportResponse)}
	<StripeImportReport {report} />
{/snippet}

{#snippet importActions({ busy }: { busy: boolean })}
	{#if canDownload && downloadApi?.configured}
		<div>
			<Button
				variant="outline"
				onclick={() => (downloadOpen = true)}
				disabled={busy}
				data-testid="stripe-download-button"
			>
				{i18nMsg('stripe-download-button', 'Télécharger depuis Stripe')}
			</Button>
			{#if downloadApi.mode === 'sandbox'}
				<span class="ml-2 text-xs text-text-muted" data-testid="stripe-download-sandbox">
					{i18nMsg('stripe-download-sandbox', 'bac à sable')}
				</span>
			{/if}
		</div>
	{/if}
{/snippet}

{#snippet importNotes()}
	{#if role === 'Admin' && downloadApi !== null && !downloadApi.configured}
		<p class="mt-2 text-xs text-text-muted" data-testid="stripe-download-not-configured">
			{i18nMsg(
				'stripe-download-not-configured',
				'Téléchargement depuis Stripe non configuré pour ce compte : voir le manuel d’administration.',
			)}
		</p>
	{/if}
{/snippet}

{#snippet proposalAction(p: StripeProposal)}
	{stripeProposalActionLabel(p, accounts)}
{/snippet}

{#snippet gapExtra(g: ProviderFinding)}
	{@const entryId = payoutEntryId(g)}
	{#if entryId !== null}
		<span class="block text-xs" data-testid="stripe-gap-receipt-links">
			<a class="text-primary underline" href="/journal-entries/{entryId}">
				{i18nMsg('stripe-gap-payout-entry', 'Écriture du versement')}
			</a>
			·
			{#if payoutGoesToProvider(g)}
				<a class="text-primary underline" href="/reconciliation/wise" data-testid="stripe-gap-receipt-import">
					{i18nMsg(
						'stripe-gap-receipt-import',
						'Importez le relevé du compte de destination (Wise).',
					)}
				</a>
			{:else}
				<a class="text-primary underline" href="/reconciliation">
					{i18nMsg('stripe-gap-receipt-link', 'Rattacher la réception à la réconciliation')}
				</a>
			{/if}
		</span>
	{/if}
{/snippet}

{#snippet toReviewHead()}
	<th class="py-2 pr-4 font-semibold">{i18nMsg('stripe-col-category', 'Catégorie Stripe')}</th>
	<th class="py-2 pr-4 font-semibold">{i18nMsg('stripe-col-id', 'Identifiant')}</th>
{/snippet}

{#snippet toReviewCells(t: ProviderFinding)}
	<td class="py-2 pr-4">{t.transaction?.providerType ?? ''}</td>
	<td class="py-2 pr-4 font-mono text-xs">{t.transaction?.transactionId ?? ''}</td>
{/snippet}

<ProviderReviewPage
	bind:this={reviewPage}
	bind:selectedAccount
	bind:bankAccounts
	bind:selectedYearId
	bind:fiscalYears
	testId="stripe"
	texts={{
		title: i18nMsg('stripe-page-title', 'Stripe'),
		subtitle: i18nMsg(
			'stripe-page-subtitle',
			'Importez le rapport « Rapprochement des versements → Détaillé » d’un compte Stripe, puis acceptez les propositions de rapprochement. Rien n’est écrit sans votre acceptation.',
		),
		noAccount: i18nMsg(
			'stripe-no-account',
			'Aucun compte Stripe. Créez un compte bancaire par devise, case « Compte Stripe » cochée, dans les comptes bancaires.',
		),
		accountLabel: i18nMsg('stripe-label-account', 'Compte Stripe'),
		importButton: i18nMsg('stripe-import-button', 'Importer un rapport Stripe'),
		idColumn: i18nMsg('stripe-col-id', 'Identifiant'),
		hint: i18nMsg(
			'stripe-hint-sales-first',
			'Les ventes viennent des boutiques : importez-les avant le rapport Stripe, pour que chaque charge trouve sa vente.',
		),
	}}
	api={{ importFile: importStripeReport, getReview: getStripeReview, accept: acceptStripeProposals }}
	accountLabels={stripeAccountLabels}
	importErrorLines={stripeImportErrorLines}
	failedLabel={(f) => stripeFailedLabel(f.errorCode, f.details)}
	proposalKey={stripeProposalKey}
	proposalTransactionId={stripeProposalTransactionId}
	proposalKind={stripeProposalKindLabel}
	acceptBodies={stripeSelectedBodies}
	gapLabel={stripeGapLabel}
	gapDetails={stripeGapDetails}
	toReviewLabel={stripeToReviewLabel}
	onLoad={loadStripe}
	{importReport}
	{importActions}
	{importNotes}
	{proposalAction}
	{gapExtra}
	{toReviewHead}
	{toReviewCells}
/>

{#if selectedAccount !== null}
	<ProviderDownloadDialog
		open={downloadOpen}
		onOpenChange={(v) => (downloadOpen = v)}
		provider="stripe"
		account={selectedAccount}
		title={i18nMsg('stripe-download-title', 'Télécharger depuis Stripe')}
		stepLabel={i18nMsg('stripe-download-step', 'versement')}
		notice={i18nMsg(
			'stripe-download-notice',
			'Les versements payés dont l’arrivée tombe dans la période (dates de Zurich) sont lus, puis leurs transactions, un versement après l’autre ; un versement récent peut tarder à paraître. Les versements manuels ne se téléchargent pas.',
		)}
		sandbox={downloadApi?.mode === 'sandbox'}
		defaultPeriod={downloadPeriod}
		lineLabel={(code) => stripeLineErrorLabel({ code, field: null, value: null })}
		onDone={() => void reviewPage?.reload()}
	>
		{#snippet report(result: StripeDownloadResult)}
			<StripeImportReport report={result} testid="stripe-download-report" />
		{/snippet}
	</ProviderDownloadDialog>
{/if}
