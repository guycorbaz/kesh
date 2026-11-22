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
-->
<script lang="ts">
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { fetchAccounts } from '$lib/features/accounts/accounts.api';
	import type { AccountResponse } from '$lib/features/accounts/accounts.types';
	import type { BankAccountSummary } from '$lib/features/bank-accounts/bank-accounts.api';
	import { listBankAccounts } from '$lib/features/bank-accounts/bank-accounts.api';
	import { stripeAccountLabels } from '$lib/features/bank-accounts/provider';
	import ProviderReviewPage from '$lib/features/provider/ProviderReviewPage.svelte';
	import type { ProviderFinding } from '$lib/features/provider/provider-review.types';
	import { acceptStripeProposals, getStripeReview, importStripeReport } from '$lib/features/stripe/stripe.api';
	import {
		stripeFailedLabel,
		stripeGapDetails,
		stripeGapLabel,
		stripeImportErrorLines,
		stripeProposalActionLabel,
		stripeProposalKindLabel,
		stripeToReviewLabel,
	} from '$lib/features/stripe/stripe-labels';
	import {
		stripeProposalKey,
		stripeProposalTransactionId,
		stripeSelectedBodies,
	} from '$lib/features/stripe/stripe-review';
	import type { StripeImportResponse, StripeProposal } from '$lib/features/stripe/stripe.types';

	// Comptes du plan : libellé du compte de frais d'une proposition (best-effort).
	// fx-filtre-comptes : exempté — aucun sélecteur de compte : la liste ne sert qu'à nommer le
	// compte de frais que la revue a choisi (en CHF ou dans la devise du solde).
	let accounts = $state<AccountResponse[]>([]);
	// Tous les comptes bancaires, archivés et clos compris (le serveur ne les écarte pas d'une
	// destination de versement) : de quoi savoir où un versement arrive.
	let bankAccounts = $state<BankAccountSummary[]>([]);

	async function loadStripe() {
		try {
			accounts = await fetchAccounts(false);
		} catch {
			accounts = [];
		}
		try {
			bankAccounts = await listBankAccounts(true, true);
		} catch {
			bankAccounts = [];
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
	 * renvoi ordinaire. La page Wise viendra avec l'Epic 39 (lien à poser alors).
	 */
	function payoutGoesToProvider(g: ProviderFinding): boolean {
		const id = g.details?.payoutBankAccountId;
		const dest = typeof id === 'number' ? bankAccounts.find((b) => b.id === id) : undefined;
		return dest?.provider === 'wise';
	}
</script>

{#snippet importReportExtra(report: StripeImportResponse)}
	{#if report.payouts.length > 0}
		<p class="mt-2">{i18nMsg('stripe-import-report-payouts', 'Versements lus :')}</p>
		<ul class="mt-1 list-disc pl-5" data-testid="stripe-import-report-payouts">
			{#each report.payouts as po (po.payoutId)}
				<li data-testid="stripe-import-report-payout-{po.payoutId}">
					{i18nMsg('stripe-import-report-payout', '{ $payout } : { $lines } ligne(s), net { $net } { $currency }', {
						payout: po.payoutId,
						lines: String(po.lines),
						net: po.net,
						currency: po.currency,
					})}
				</li>
			{/each}
		</ul>
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
				<span data-testid="stripe-gap-receipt-import">
					{i18nMsg(
						'stripe-gap-receipt-import',
						'Importez le relevé du compte de destination (Wise).',
					)}
				</span>
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
	{importReportExtra}
	{proposalAction}
	{gapExtra}
	{toReviewHead}
	{toReviewCells}
/>
