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
-->
<script lang="ts">
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { fetchAccounts } from '$lib/features/accounts/accounts.api';
	import { withoutForeignCurrency, type AccountResponse } from '$lib/features/accounts/accounts.types';
	import type { BankAccountSummary } from '$lib/features/bank-accounts/bank-accounts.api';
	import { accountLabelKey, paypalAccountLabels } from '$lib/features/bank-accounts/provider';
	import ProviderReviewPage from '$lib/features/provider/ProviderReviewPage.svelte';
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

{#snippet importReportExtra(report: PaypalImportResponse)}
	<p class="mt-2" data-testid="paypal-import-report-skipped">
		{i18nMsg('paypal-import-report-skipped', 'Écartées : { $notCompleted } ligne(s) non terminée(s), { $memo } ligne(s) Memo, { $pairs } paire(s) neutre(s).', {
			notCompleted: String(report.skipped.notCompleted.length),
			memo: String(report.skipped.memo),
			pairs: String(report.skipped.neutralPairs),
		})}
	</p>
	{#if report.skipped.notCompleted.length > 0}
		<ul class="mt-1 list-disc pl-5 text-xs text-text-muted" data-testid="paypal-import-report-not-completed">
			{#each report.skipped.notCompleted as l (l.transactionId)}
				<li>{l.transactionId} — {l.type} — {l.status}</li>
			{/each}
		</ul>
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
	bind:selected
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
	{importReportExtra}
	{proposalAction}
	{toReviewHead}
	{toReviewCells}
/>
