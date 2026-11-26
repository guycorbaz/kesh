<!--
  Story 39-11d (AC-2) — la page Wise, sur la page commune des prestataires
  (`ProviderReviewPage`, 40-7d) : choix du compte Wise, import d'un geste de
  ses relevés CSV — un par solde, donc par devise — ou du zip que Wise fait
  télécharger, tel quel (39-11e), et le compte rendu **par fichier** (archive,
  lignes insérées, doublons, soldes, avertissement de continuité, entrées du
  zip non lues).

  La consigne d'export est à l'écran : un relevé exporté sans l'option « frais
  indiqués séparément » est refusé (`WISE_CSV_FEES_NOT_SEPARATED`), avec la
  même consigne.

  Story 39-11d2 (AC-2 à AC-4) — la revue et l'acceptation Wise (39-11c,
  `GET /api/v1/wise/review`, `POST /api/v1/wise/accept`) : exercice,
  propositions à cocher — frais, conversion au cours de Wise, rattachement à
  l'autre côté déjà passé, règle, paiement par carte groupé avec ses lignes en
  sous-lignes —, leur genre et « ce qui sera fait » en clair, une phrase par
  motif « à examiner » et par écart, acceptation par paquets. Libellés des
  comptes et des règles lus au chargement (best-effort). Les liens propres à
  Wise sous un « à examiner » : les écritures d'une part déjà rapprochée, la
  page Stripe pour un versement Stripe sans son écriture (si la société a un
  compte Stripe).
-->
<script lang="ts">
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { fetchAccounts } from '$lib/features/accounts/accounts.api';
	import type { AccountResponse } from '$lib/features/accounts/accounts.types';
	import type { BankAccountSummary } from '$lib/features/bank-accounts/bank-accounts.api';
	import { wiseAccountLabels } from '$lib/features/bank-accounts/provider';
	import ProviderReviewPage from '$lib/features/provider/ProviderReviewPage.svelte';
	import { ruleLabelsById } from '$lib/features/provider/provider-rules';
	import type { ProviderFinding, ProviderReviewTransaction } from '$lib/features/provider/provider-review.types';
	import { acceptWiseProposals, getWiseReview, importWiseFiles } from '$lib/features/wise/wise.api';
	import {
		wiseFailedLabel,
		wiseFileReportLine,
		wiseGapLabel,
		wiseImportErrorLines,
		wiseLineLabel,
		wisePartlyReconciledEntryIds,
		wiseProposalActionLabel,
		wiseProposalKindLabel,
		wiseToReviewLabel,
		wiseWarningLabel,
	} from '$lib/features/wise/wise-labels';
	import {
		wiseFindingLines,
		wiseProposalKey,
		wiseProposalLines,
		wiseProposalTransactionId,
		wiseSelectedBodies,
	} from '$lib/features/wise/wise-review';
	import type { WiseImportResponse, WiseProposal } from '$lib/features/wise/wise.types';

	// Comptes du plan : libellés des comptes de frais et de contrepartie (best-effort).
	// fx-filtre-comptes : exempté — aucun sélecteur de compte : la liste ne sert qu'à nommer les
	// comptes que la revue a choisis (en CHF ou dans la devise du solde).
	let accounts = $state<AccountResponse[]>([]);
	// Libellés des règles, par identifiant : la revue ne sert que `ruleId`.
	let ruleLabels = $state<Map<number, string>>(new Map());
	// Les comptes bancaires lus par la page commune : la société a-t-elle un compte Stripe ?
	let bankAccounts = $state<BankAccountSummary[]>([]);
	let hasStripe = $derived(bankAccounts.some((b) => b.provider === 'stripe'));

	/** Lectures propres à Wise : comptes du plan et libellés des règles (best-effort). */
	async function loadWise() {
		const [acc, rules] = await Promise.allSettled([fetchAccounts(false), ruleLabelsById()]);
		accounts = acc.status === 'fulfilled' ? acc.value : [];
		ruleLabels = rules.status === 'fulfilled' ? rules.value : new Map();
	}
</script>

{#snippet importReportMain(report: WiseImportResponse)}
	<ul class="mt-2 space-y-2" data-testid="wise-import-report-files">
		{#each report.files as f, i (i)}
			<li data-testid="wise-import-report-file-{f.currency}">
				<p class="font-medium" data-testid="wise-import-report-file-name">
					{f.archive
						? i18nMsg('wise-import-report-file-in-archive', '{ $filename } (archive { $archive })', {
								filename: f.filename,
								archive: f.archive,
							})
						: f.filename}
				</p>
				<p data-testid="wise-import-report-file-line">{wiseFileReportLine(f)}</p>
				{#if f.importId === null}
					<p class="text-text-muted" data-testid="wise-import-report-file-all-known">
						{i18nMsg('wise-import-report-all-known', 'Toutes les lignes étaient déjà importées : aucun import créé.')}
					</p>
				{/if}
				{#each f.warnings as w, j (j)}
					<p class="text-amber-800" role="status" data-testid="wise-import-report-warning-{w.code}">
						{wiseWarningLabel(w, f.currency)}
					</p>
				{/each}
			</li>
		{/each}
	</ul>
	{#if report.ignoredEntries.length > 0}
		<p class="mt-2">{i18nMsg('wise-import-report-ignored', 'Entrées du zip non lues (ni relevé CSV à la racine) :')}</p>
		<ul class="mt-1 list-disc pl-5" data-testid="wise-import-report-ignored">
			{#each report.ignoredEntries as e, i (i)}
				<li>{e.entry}</li>
			{/each}
		</ul>
	{/if}
{/snippet}

{#snippet proposalAction(p: WiseProposal)}
	{wiseProposalActionLabel(p, accounts, ruleLabels)}
{/snippet}

{#snippet toReviewExtra(f: ProviderFinding)}
	{@const entryIds = wisePartlyReconciledEntryIds(f)}
	{#if entryIds.length > 0}
		<span class="block text-xs" data-testid="wise-to-review-entries">
			{#each entryIds as id, i (id)}
				{#if i > 0}·{/if}
				<a class="text-primary underline" href="/journal-entries/{id}" data-testid="wise-to-review-entry-{id}">
					{i18nMsg('wise-review-see-entry', 'Voir l’écriture')}
				</a>
			{/each}
		</span>
	{/if}
	{#if f.code === 'stripe_payout_unmatched' && hasStripe}
		<a class="block text-xs text-primary underline" href="/reconciliation/stripe" data-testid="wise-to-review-stripe-link">
			{i18nMsg('wise-review-stripe-page', 'Ouvrir la page Stripe')}
		</a>
	{/if}
{/snippet}

{#snippet toReviewHead()}
	<th class="py-2 pr-4 font-semibold">{i18nMsg('wise-col-name', 'Nom')}</th>
	<th class="py-2 pr-4 font-semibold">{i18nMsg('wise-col-id', 'Identifiant Wise')}</th>
{/snippet}

<!-- Le nom et l'identifiant Wise d'une ligne « à examiner » et de ses sous-lignes : de quoi la retrouver. -->
{#snippet transactionCells(t: ProviderReviewTransaction | undefined)}
	<td class="py-2 pr-4">{t?.name ?? ''}</td>
	<td class="py-2 pr-4 font-mono text-xs whitespace-nowrap">{t?.transactionId ?? ''}</td>
{/snippet}

{#snippet toReviewCells(f: ProviderFinding)}
	{@render transactionCells(f.transaction)}
{/snippet}

{#snippet toReviewLineCells(t: ProviderReviewTransaction)}
	{@render transactionCells(t)}
{/snippet}

<ProviderReviewPage
	testId="wise"
	fileAccept=".csv,text/csv,.zip,application/zip"
	multiple
	texts={{
		title: i18nMsg('wise-page-title', 'Wise'),
		subtitle: i18nMsg(
			'wise-page-subtitle',
			'Importez les relevés d’un compte Wise : le zip que Wise vous fait télécharger, tel quel, ou ses fichiers CSV, un par devise. Chaque relevé va au compte bancaire de sa devise. Puis acceptez les propositions de rapprochement : rien n’est écrit dans les livres sans votre acceptation.',
		),
		noAccount: i18nMsg(
			'wise-no-account',
			'Aucun compte Wise. Créez un compte bancaire par devise (un par solde Wise), case « Compte Wise » cochée, dans les comptes bancaires.',
		),
		accountLabel: i18nMsg('wise-label-account', 'Compte Wise'),
		importButton: i18nMsg('wise-import-button', 'Importer des relevés Wise (zip ou CSV)'),
		importHelp: i18nMsg(
			'wise-import-help',
			'Dans Wise, ouvrez l’export des relevés : choisissez la période et le format CSV, et cochez « Afficher les transactions dont les frais sont indiqués séparément » — sans cette option, le relevé est refusé. Donnez ensuite ici le zip téléchargé, tel quel.',
		),
		idColumn: i18nMsg('wise-col-id', 'Identifiant Wise'),
	}}
	api={{ importFiles: importWiseFiles, getReview: getWiseReview, accept: acceptWiseProposals }}
	accountLabels={wiseAccountLabels}
	importErrorLines={wiseImportErrorLines}
	failedLabel={(f) => wiseFailedLabel(f.errorCode, f.details)}
	proposalKey={wiseProposalKey}
	proposalTransactionId={wiseProposalTransactionId}
	proposalKind={wiseProposalKindLabel}
	acceptBodies={wiseSelectedBodies}
	gapLabel={wiseGapLabel}
	toReviewLabel={wiseToReviewLabel}
	proposalLines={wiseProposalLines}
	findingLines={wiseFindingLines}
	lineLabel={wiseLineLabel}
	onLoad={loadWise}
	bind:bankAccounts
	{importReportMain}
	{proposalAction}
	{toReviewExtra}
	{toReviewHead}
	{toReviewCells}
	{toReviewLineCells}
/>
