<!--
  Story 8-4 (FR44) — top-1 only par tx via checkbox tx-level. Candidates
  2..N ignorées côté UI v0.1 — voir KF-026 #76 pour le pattern multi-
  candidates v0.2.

  Liste de propositions de réconciliation avec sélection par tx
  (checkbox) + actions batch Accept/Reject. Pour chaque transaction
  pending, on affiche `candidates[0]` — la première dans l'ordre que le
  serveur a fixé (`order_candidates`, `routes/reconciliation.rs`, Story
  29-3 : une facture qui porte un signal d'identité passe devant la règle
  d'affectation ; l'écran ne refait pas cet ordre) — et un
  checkbox au niveau tx. Si la transaction n'a aucune candidate,
  on affiche « Aucune correspondance », avec une checkbox comme les
  autres lignes (Story 29-6, #665, AC10) : elle se coche pour être
  rejetée, ou avant « Affecter manuellement » / « Éclater » ;
  « Accepter » ne la traite pas — il est grisé tant qu'une telle ligne
  est cochée, et la rangée des boutons le dit. Erreurs partielles
  affichées en bas via `failed`.

  Story 29-8 (#670, AC4) — une ligne de crédit en CHF porte un troisième
  bouton, « Rattacher à un règlement » (`LinkSettlementModal`) : la
  transaction se rattache au règlement déjà saisi depuis la fiche de la
  facture, sans écriture.

  Story 15-5c (AC3, #492, choix C38) — chaque refus de `failed[]` est affiché
  par son LIBELLÉ traduit (`failedProposalLabel`) et désigne la transaction par
  sa date, son montant et sa contrepartie, relevés AVANT le rechargement qui
  suit le lot (la transaction acceptée ou rejetée n'est plus dans la liste
  ensuite). Le compteur et les *Échecs partiels* restent affichés même quand
  le lot a vidé la liste.

  Story 30-1b (#686, AC-b11) — un débit reçoit des factures FOURNISSEUR
  (`candidateType = 'supplier_invoice'`) : badge « Facture fournisseur », nom
  du fournisseur, numéro (à défaut `#id`) et montant ; « Accepter » envoie
  `{ type: 'supplier_invoice', bankTransactionId, supplierInvoiceId }`. Une
  ligne qui porte un `notice` l'affiche À LA PLACE de toute proposition ;
  « Accepter » reste grisé pour elle, comme pour toute ligne sans candidate.
  Le texte de l'avis ne conseille jamais de confirmer le lot (qui compterait la
  dépense deux fois) et ne cite aucun numéro d'issue.
-->
<script lang="ts">
	import { errorMessageOf } from '$lib/shared/utils/error-message';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import {
		acceptProposals,
		getProposals,
		PROPOSALS_DEFAULT_LIMIT,
		PROPOSALS_MAX_LIMIT,
		rejectProposals,
	} from './reconciliation.api';
	import type {
		FailedProposal,
		ReconciliationProposal,
	} from './reconciliation.types';
	import ScoreBadge from './ScoreBadge.svelte';
	import { failedProposalLabel } from './failed-proposal-label';
	import ManualMatchModal from './ManualMatchModal.svelte';
	import TransactionSplitModal from './TransactionSplitModal.svelte';
	import LinkSettlementModal from './LinkSettlementModal.svelte';
	import { fetchAccounts } from '$lib/features/accounts/accounts.api';
	import type { AccountResponse } from '$lib/features/accounts/accounts.types';
	import { listBankAccounts } from '$lib/features/bank-accounts/bank-accounts.api';

	type Props = {
		bankAccountId: number;
	};
	let { bankAccountId }: Props = $props();

	let proposals = $state<ReconciliationProposal[]>([]);
	// Story 8-5a-base FR45 — manual match modal state.
	let manualOpen = $state(false);
	let manualProposal = $state<ReconciliationProposal | null>(null);
	// Story 8-5a-bis FR48 — split modal state.
	let splitOpen = $state(false);
	let splitProposal = $state<ReconciliationProposal | null>(null);
	// Story 29-8 (#670) — modale « Rattacher à un règlement ».
	let linkOpen = $state(false);
	let linkProposal = $state<ReconciliationProposal | null>(null);
	let accounts = $state<AccountResponse[]>([]);
	// γ refactor BS1 — checkbox tx-level top-1 only : selected = Set<txId>
	// (l'invoiceId top-1 est ré-extrait au moment du submit).
	let selected = $state<Set<number>>(new Set());
	let loading = $state(false);
	let busy = $state(false);
	let errorMsg = $state<string | null>(null);
	let failed = $state<FailedProposal[]>([]);
	// Story 15-5c (AC3) — transactions du lot, relevées avant le `load()` qui
	// suit : un refus doit pouvoir nommer une transaction sortie de la liste.
	let failedTx = $state<Map<number, ReconciliationProposal['transaction']>>(new Map());
	let lastSuccessCount = $state(0);

	/** Relève, avant l'envoi d'un lot, les transactions qui le composent. */
	function snapshotSelected(): void {
		const next = new Map<number, ReconciliationProposal['transaction']>();
		for (const p of proposals) {
			if (selected.has(p.bankTransactionId)) next.set(p.bankTransactionId, p.transaction);
		}
		failedTx = next;
	}

	/**
	 * Efface le bilan du dernier lot — compteur, *Échecs partiels* et transactions relevées.
	 *
	 * Revue de code P1 de la 15-5c (B1, E3) : le bilan est rendu hors des branches de la liste
	 * (C38) ; sans remise à zéro, celui d'un lot ancien survivait à une affectation manuelle ou
	 * à un éclatement. Appelée à l'ouverture de tout nouveau bilan — lot, affectation, éclatement.
	 */
	function clearBatchReport(): void {
		failed = [];
		lastSuccessCount = 0;
		failedTx = new Map();
	}

	/** « date · montant devise · contrepartie », ou `TX #<id>` si la transaction n'a pas été relevée. */
	function describeTx(id: number): string {
		const t = failedTx.get(id);
		if (!t) return `TX #${id}`;
		return [t.bookingDate, `${t.amount} ${t.currency}`, t.counterpartyName]
			.filter((v) => v !== null && v !== undefined && v !== '')
			.join(' · ');
	}

	// H7 Pass 1 code review — generation tag pour drop les responses
	// `getProposals` stale (race sur fast account switch).
	let loadGen = 0;

	// Story 29-9 (#649, AC2) — combien de transactions la liste demande, et s'il en reste
	// au-delà (`hasMore` du serveur). `limit` part du défaut (100) et ne monte qu'au plafond
	// (500), par le bouton « Afficher aussi les plus anciennes » ; il tient ensuite pour
	// tous les rechargements (lot, affectation, éclatement, rattachement). Le `{#key
	// selectedId}` de `routes/(app)/reconciliation/+page.svelte` remonte ce composant à
	// chaque changement de compte, ce qui remet `limit` à 100 : ⚠️ retirer ce `{#key}`
	// exigerait de remettre `limit` à zéro au changement de compte.
	let limit = $state(PROPOSALS_DEFAULT_LIMIT);
	let hasMore = $state(false);

	async function load() {
		const myGen = ++loadGen;
		loading = true;
		errorMsg = null;
		try {
			// `limit` est lu ici, avant le premier `await` : l'`$effect` qui appelle `load()` le
			// suit, si bien que changer `limit` recharge la liste.
			const r = await getProposals(bankAccountId, limit);
			if (myGen !== loadGen) return; // stale response, drop
			proposals = r.proposals;
			hasMore = r.hasMore;
			selected = new Set();
		} catch (e) {
			if (myGen !== loadGen) return;
			errorMsg = errorMessageOf(e);
		} finally {
			if (myGen === loadGen) loading = false;
		}
	}

	$effect(() => {
		void load();
	});

	/**
	 * Story 29-9 (#649, AC2) — porte la liste au plafond. Ne fait QUE poser `limit` : l'`$effect`
	 * ci-dessus, qui suit `limit`, recharge. ⚠️ Appeler `load()` en plus enverrait deux requêtes.
	 * Le rechargement vide la sélection, comme tout rechargement.
	 */
	function showOlder(): void {
		limit = PROPOSALS_MAX_LIMIT;
	}

	// Story 8-5a-base — load accounts une fois (utilisé par le ManualMatchModal,
	// pré-filtré dans la modal).
	$effect(() => {
		void (async () => {
			try {
				accounts = await fetchAccounts();
			} catch {
				// Silencieux : la modal affichera un select vide si erreur,
				// le user ne peut pas réconcilier sans accounts mais le
				// flow accept/reject reste fonctionnel.
				accounts = [];
			}
		})();
	});

	// Story 15-6d (AC4, #524) — compte comptable du compte bancaire monté, que la modale
	// d'affectation manuelle écarte des contreparties (une écriture `D banque / C banque` est
	// nulle). Résolu UNE fois, au montage : la page remonte ce composant à chaque changement de
	// compte bancaire (`{#key selectedId}`, `routes/(app)/reconciliation/+page.svelte`), si bien
	// qu'une réponse ne peut pas arriver pour un autre `bankAccountId` que celui du montage —
	// d'où l'absence de garde de génération (choix C-15-6-28). ⚠️ Retirer ce `{#key}` exigerait
	// la garde. Repli `null` — aucun filtrage, la garde serveur de `POST /reconciliation/manual`
	// tranche — pendant le chargement, en cas d'échec (ignoré sans message, comme le chargement
	// des comptes ci-dessus), et si le compte bancaire est absent de la liste (archivé, ou
	// clos depuis la Story 28-4b : `listBankAccounts()` n'en rend aucun des deux) ou non lié.
	let bankLedgerAccountId = $state<number | null>(null);
	$effect(() => {
		const mountedBankAccountId = bankAccountId;
		void (async () => {
			try {
				const list = await listBankAccounts();
				bankLedgerAccountId =
					list.find((ba) => ba.id === mountedBankAccountId)?.journalAccountId ?? null;
			} catch {
				bankLedgerAccountId = null;
			}
		})();
	});

	function openManualMatch(p: ReconciliationProposal) {
		manualProposal = p;
		manualOpen = true;
	}

	async function onManualSuccess() {
		clearBatchReport();
		manualOpen = false;
		manualProposal = null;
		await load();
	}

	function openSplit(p: ReconciliationProposal) {
		splitProposal = p;
		splitOpen = true;
	}

	async function onSplitSuccess() {
		clearBatchReport();
		splitOpen = false;
		splitProposal = null;
		await load();
	}

	/**
	 * Story 29-8 (#670, AC4) — miroir de la condition 1 du serveur
	 * (`transaction_can_be_linked`) : seul un crédit en CHF se rattache à un règlement.
	 */
	function canLink(p: ReconciliationProposal): boolean {
		return Number(p.transaction.amount) > 0 && p.transaction.currency === 'CHF';
	}

	function openLink(p: ReconciliationProposal) {
		linkProposal = p;
		linkOpen = true;
	}

	/**
	 * Un rattachement réussi ouvre un nouveau bilan : celui du lot précédent est effacé, puis
	 * le compteur dit l'opération réussie (choix C-29-8-32), et la liste est relue.
	 */
	async function onLinkSuccess() {
		clearBatchReport();
		lastSuccessCount = 1;
		linkOpen = false;
		linkProposal = null;
		await load();
	}

	// Story 29-6 (#665, AC10) — une ligne sans proposition se coche (pour
	// « Rejeter ») ; « Accepter » ne la traite pas : grisé, gardé, et dit.
	const selectionHasUnmatched = $derived(
		proposals.some((p) => selected.has(p.bankTransactionId) && p.candidates.length === 0),
	);

	function toggle(txId: number) {
		const next = new Set(selected);
		if (next.has(txId)) next.delete(txId);
		else next.add(txId);
		selected = next;
	}

	async function onAccept() {
		if (selected.size === 0 || selectionHasUnmatched) return;
		busy = true;
		errorMsg = null;
		clearBatchReport();
		snapshotSelected();
		try {
			// γ refactor — pour chaque txId sélectionné, retrouver la
			// candidate top-1 (candidates[0]) dans la proposal courante.
			// Une tx sans candidate peut être sélectionnée (pour « Rejeter »,
			// ou avant « Affecter manuellement » / « Éclater » ; Story 29-6) :
			// « Accepter » est alors grisé et la garde de tête l'arrête ; le
			// `throw` ci-dessous reste le dernier rempart.
			const items = Array.from(selected).map((txId) => {
				const p = proposals.find((x) => x.bankTransactionId === txId);
				if (!p || p.candidates.length === 0) {
					throw new Error(`No candidate for txId=${txId}`);
				}
				const c = p.candidates[0];
				if (c.candidateType === 'rule' && c.ruleId !== null && c.counterpartyAccountId !== null) {
					return {
						type: 'rule' as const,
						bankTransactionId: txId,
						ruleId: c.ruleId,
						counterpartyAccountId: c.counterpartyAccountId,
					};
				}
				if (c.candidateType === 'invoice' && c.invoiceId !== null) {
					return {
						type: 'invoice' as const,
						bankTransactionId: txId,
						invoiceId: c.invoiceId,
					};
				}
				// Story 30-1b (#686) — une facture fournisseur, pour un débit.
				if (c.candidateType === 'supplier_invoice' && c.supplierInvoiceId !== null) {
					return {
						type: 'supplier_invoice' as const,
						bankTransactionId: txId,
						supplierInvoiceId: c.supplierInvoiceId,
					};
				}
				throw new Error(`Invalid candidate for txId=${txId}`);
			});
			const r = await acceptProposals(bankAccountId, items);
			lastSuccessCount = r.accepted.length;
			failed = r.failed;
			await load();
		} catch (e) {
			errorMsg = errorMessageOf(e);
		} finally {
			busy = false;
		}
	}

	async function onReject() {
		if (selected.size === 0) return;
		busy = true;
		errorMsg = null;
		clearBatchReport();
		snapshotSelected();
		try {
			const ids = Array.from(selected);
			const r = await rejectProposals(bankAccountId, ids);
			lastSuccessCount = r.rejected.length;
			failed = r.failed;
			await load();
		} catch (e) {
			errorMsg = errorMessageOf(e);
		} finally {
			busy = false;
		}
	}
</script>

<section data-testid="reconciliation-proposals">
	<!-- Story 15-5c (AC3, C38) — le compteur du dernier lot, hors des branches
	     de la liste : il reste visible quand le lot l'a vidée. -->
	{#if lastSuccessCount > 0}
		<p class="mb-2 text-green-700" data-testid="reconciliation-success">
			{lastSuccessCount}
			{i18nMsg('reconciliation-labels-success-suffix', 'opération(s) réussie(s).')}
		</p>
	{/if}
	{#if loading}
		<p class="text-text-muted" data-testid="reconciliation-loading">
			{i18nMsg('reconciliation-labels-loading', 'Chargement des propositions…')}
		</p>
	{:else if errorMsg}
		<p class="text-red-700" data-testid="reconciliation-error">{errorMsg}</p>
	{:else if proposals.length === 0}
		<p class="text-text-muted" data-testid="reconciliation-empty">
			{i18nMsg(
				'reconciliation-labels-empty',
				'Aucune transaction en attente de réconciliation.',
			)}
		</p>
	{:else}
		<div class="mb-4 flex flex-wrap items-center gap-2">
			<button
				type="button"
				class="rounded bg-primary px-4 py-2 text-white disabled:opacity-50"
				disabled={busy || selected.size === 0 || selectionHasUnmatched}
				onclick={onAccept}
				data-testid="reconciliation-accept-btn"
			>
				{i18nMsg('reconciliation-actions-accept', 'Accepter')} ({selected.size})
			</button>
			<button
				type="button"
				class="rounded border border-text px-4 py-2 disabled:opacity-50"
				disabled={busy || selected.size === 0}
				onclick={onReject}
				data-testid="reconciliation-reject-btn"
			>
				{i18nMsg('reconciliation-actions-reject', 'Rejeter')}
			</button>
			<!-- Story 29-6 (#665, AC10, C-29-6-18) — dans la rangée des boutons, et non entre
			     elle et la table : un paragraphe apparu au cochage au-dessus de la table la
			     descendrait sous le pointeur. Ne nomme aucun autre geste (C-29-6-16). -->
			<!-- Story 29-9 (#678, AC11, C-29-9-21) — l'indication vit dans une région d'annonce
			     rendue DÈS le rendu de la liste, vide tant qu'aucune ligne sans proposition n'est
			     cochée : un lecteur d'écran n'annonce que ce qui CHANGE dans une région déjà
			     présente, et une région insérée avec son contenu se tairait. La région englobe le
			     `{#if}`, et non l'inverse ; elle ne doit entrer ni dans un `{#key}` ni dans le
			     `{#if}`, sans quoi son nœud serait recréé au changement. Dernier enfant de la
			     rangée (C-29-6-18 tenu). -->
			<div role="status" aria-live="polite" aria-atomic="true" data-testid="reconciliation-selection-status">
				{#if selectionHasUnmatched}
					<p class="text-sm text-text-muted" data-testid="reconciliation-unmatched-not-acceptable-hint">
						{i18nMsg(
							'reconciliation-labels-unmatched-not-acceptable-hint',
							'Une transaction sans correspondance ne peut pas être acceptée : décochez-la pour accepter les autres.',
						)}
					</p>
				{/if}
			</div>
		</div>

		<!-- Story 29-3 (#527, AC5, C-29-3-14) — la priorité dite à l'écran, seulement si une
		     ligne au moins a une candidate : sans quoi « une seule proposition est affichée »
		     ne dirait rien de vrai. -->
		{#if proposals.some((p) => p.candidates.length > 0)}
			<p class="mb-2 text-sm text-text-muted" data-testid="reconciliation-priority-hint">
				{i18nMsg(
					'reconciliation-labels-priority-hint',
					'Une seule proposition est affichée par transaction : pour un crédit, la facture client la mieux notée, pour un débit, la facture fournisseur la mieux notée, s’il y en a une, sinon la règle d’affectation qui correspond. Une facture proposée passe toujours devant une règle.',
				)}
			</p>
		{/if}

		<table class="w-full table-auto text-sm" data-testid="reconciliation-table">
			<thead>
				<tr>
					<th class="text-left"></th>
					<th class="text-left">{i18nMsg('reconciliation-cols-tx-date', 'Date')}</th>
					<th class="text-right">{i18nMsg('reconciliation-cols-tx-amount', 'Montant')}</th>
					<th class="text-left">{i18nMsg('reconciliation-cols-tx-counterparty', 'Contrepartie')}</th>
					<th class="text-left">{i18nMsg('reconciliation-cols-candidate', 'Candidate')}</th>
					<th class="text-left">{i18nMsg('reconciliation-cols-score', 'Score')}</th>
					<th class="text-left" aria-label={i18nMsg('reconciliation-cols-actions', 'Actions')}>
						<span class="sr-only">{i18nMsg('reconciliation-cols-actions', 'Actions')}</span>
					</th>
				</tr>
			</thead>
			<tbody>
				{#each proposals as p (p.bankTransactionId)}
					<tr data-testid="reconciliation-row" data-tx-id={p.bankTransactionId}>
						<td>
							<!-- Story 29-6 (#665, AC10) — une case sur toute ligne, avec ou sans proposition. -->
							<!-- Story 29-9 (#678, AC11, C-29-9-20, C-29-9-24) — la case nomme sa transaction
							     par ce que la ligne affiche, tel quel. Deux appels à clé LITTÉRALE : une clé
							     calculée échapperait aux gardes i18n (i18n-entrees-a-variables.test.ts). Une
							     contrepartie vide ou nulle n'est pas nommée (la cellule n'affiche rien). -->
							<input
								type="checkbox"
								checked={selected.has(p.bankTransactionId)}
								onchange={() => toggle(p.bankTransactionId)}
								aria-label={p.transaction.counterpartyName
									? i18nMsg(
											'reconciliation-labels-select-transaction',
											'Sélectionner la transaction du { $date }, { $amount } { $currency }, { $counterparty }',
											{
												date: p.transaction.bookingDate,
												amount: p.transaction.amount,
												currency: p.transaction.currency,
												counterparty: p.transaction.counterpartyName,
											},
										)
									: i18nMsg(
											'reconciliation-labels-select-transaction-no-counterparty',
											'Sélectionner la transaction du { $date }, { $amount } { $currency }',
											{
												date: p.transaction.bookingDate,
												amount: p.transaction.amount,
												currency: p.transaction.currency,
											},
										)}
								data-testid="tx-checkbox"
								data-tx-id={p.bankTransactionId}
							/>
						</td>
						<td>{p.transaction.bookingDate}</td>
						<td class="text-right">{p.transaction.amount} {p.transaction.currency}</td>
						<td>{p.transaction.counterpartyName ?? ''}</td>
						{#if p.notice}
							<!-- Story 30-1b (#686, AC-b5, AC-b11) — la ligne ne propose rien, et dit pourquoi. -->
							<td
								colspan="2"
								class="text-text-muted"
								data-testid="proposal-notice"
								data-notice-code={p.notice.code}
							>
								{i18nMsg(
									'reconciliation-labels-notice-supplier-invoice-in-batch',
									'La facture fournisseur { $invoice } de ce montant figure dans le lot de paiement n° { $batch } en cours sur ce compte. Si cette transaction est le paiement de ce lot, annulez le lot : la facture vous sera alors proposée ici. Sinon, affectez la transaction à la main.',
									{
										invoice: p.notice.supplierInvoiceNumber ?? `#${p.notice.supplierInvoiceId}`,
										batch: p.notice.paymentBatchId,
									},
								)}
							</td>
						{:else if p.candidates.length === 0}
							<td colspan="2" class="text-text-muted">
								{i18nMsg('reconciliation-labels-no-candidate', 'Aucune correspondance')}
							</td>
						{:else}
							<td>
								{#if p.candidates[0].candidateType === 'rule'}
									<span
										class="mr-1 inline-block rounded bg-blue-100 px-2 py-0.5 text-xs font-semibold text-blue-800"
										data-testid="candidate-rule-badge"
										data-rule-id={p.candidates[0].ruleId}
									>
										{i18nMsg('reconciliation-rules-applied-badge', 'Règle')}
									</span>
									<span
										data-testid="candidate-top-1"
										data-candidate-type="rule"
										data-rule-id={p.candidates[0].ruleId}
									>
										{p.candidates[0].ruleLabel ?? `#${p.candidates[0].ruleId}`}
										→ {p.candidates[0].counterpartyAccountName ??
											`#${p.candidates[0].counterpartyAccountId}`}
									</span>
								{:else if p.candidates[0].candidateType === 'supplier_invoice'}
									<!-- Story 30-1b (#686, AC-b11) — une facture fournisseur, pour un débit. -->
									<span
										class="mr-1 inline-block rounded bg-amber-100 px-2 py-0.5 text-xs font-semibold text-amber-900"
										data-testid="candidate-supplier-invoice-badge"
									>
										{i18nMsg('reconciliation-labels-supplier-invoice-badge', 'Facture fournisseur')}
									</span>
									<span
										data-testid="candidate-top-1"
										data-candidate-type="supplier_invoice"
										data-supplier-invoice-id={p.candidates[0].supplierInvoiceId}
									>
										{#if p.candidates[0].contactName}{p.candidates[0].contactName} — {/if}{p.candidates[0]
											.invoiceNumber ?? `#${p.candidates[0].supplierInvoiceId}`}
										({p.candidates[0].invoiceAmount})
									</span>
								{:else}
									<span data-testid="candidate-top-1" data-candidate-type="invoice" data-invoice-id={p.candidates[0].invoiceId}>
										{p.candidates[0].invoiceNumber ?? `#${p.candidates[0].invoiceId}`}
										({p.candidates[0].invoiceAmount}{#if p.candidates[0].invoiceTotalTtc}
											<span data-testid="candidate-top-1-amount-due-of"
												>, {i18nMsg('reconciliation-labels-amount-due-of', 'reste dû sur { $total }', {
													total: p.candidates[0].invoiceTotalTtc,
												})}</span
											>{/if})
									</span>
								{/if}
							</td>
							<td>
								<ScoreBadge score={p.candidates[0].score.total} />
							</td>
						{/if}
						<td>
							<!-- Story 8-5a-base FR45 — bouton « Affecter manuellement »
							     disponible sur toutes les rows pending (avec ou sans
							     candidate auto-matchée). -->
							<button
								type="button"
								class="text-xs text-primary underline disabled:opacity-50"
								disabled={busy}
								onclick={() => openManualMatch(p)}
								data-testid="manual-match-button"
								data-tx-id={p.bankTransactionId}
							>
								{i18nMsg('reconciliation-manual-button-label', 'Affecter manuellement')}
							</button>
							<!-- Story 8-5a-bis FR48 — bouton « Éclater » disponible sur
							     toutes les rows pending (cf. AC #93). -->
							<button
								type="button"
								class="ml-2 text-xs text-primary underline disabled:opacity-50"
								disabled={busy}
								onclick={() => openSplit(p)}
								data-testid="split-button"
								data-tx-id={p.bankTransactionId}
							>
								{i18nMsg('reconciliation-split-button-label', 'Éclater')}
							</button>
							<!-- Story 29-8 (#670, AC4) — rattacher la transaction au règlement déjà
							     saisi depuis la fiche de la facture : lignes de crédit en CHF seulement. -->
							{#if canLink(p)}
								<button
									type="button"
									class="ml-2 text-xs text-primary underline disabled:opacity-50"
									disabled={busy}
									onclick={() => openLink(p)}
									data-testid="link-settlement-button"
									data-tx-id={p.bankTransactionId}
								>
									{i18nMsg('reconciliation-link-button-label', 'Rattacher à un règlement')}
								</button>
							{/if}
						</td>
					</tr>
				{/each}
			</tbody>
		</table>

		<!-- Story 29-9 (#649, AC1, AC3) — s'il reste des transactions plus anciennes, l'écran le
		     dit, immédiatement sous la table et avant les modales. Sous le plafond, un bouton
		     porte la liste à 500 (grisé pendant un lot, dont le rechargement final relit la
		     liste) ; au plafond, un avis sans bouton, qui ne conseille pas le rejet (une
		     transaction rejetée reste en attente et empêche la clôture du compte bancaire). -->
		{#if hasMore && limit < PROPOSALS_MAX_LIMIT}
			<div class="mt-3 flex flex-wrap items-center gap-2">
				<p class="text-sm text-text-muted" data-testid="reconciliation-more">
					{i18nMsg(
						'reconciliation-labels-more-hidden',
						'Seules les { $count } transactions en attente les plus récentes sont affichées : il en reste de plus anciennes.',
						{ count: proposals.length },
					)}
				</p>
				<button
					type="button"
					class="rounded border border-text px-4 py-2 disabled:opacity-50"
					disabled={busy}
					onclick={showOlder}
					data-testid="reconciliation-show-older-btn"
				>
					{i18nMsg(
						'reconciliation-actions-show-older',
						"Afficher aussi les plus anciennes (jusqu'à { $max } en tout)",
						{ max: PROPOSALS_MAX_LIMIT },
					)}
				</button>
			</div>
		{:else if hasMore}
			<p class="mt-3 text-sm text-text-muted" data-testid="reconciliation-more-capped">
				{i18nMsg(
					'reconciliation-labels-more-capped',
					'Les { $max } transactions en attente les plus récentes sont affichées, le maximum que cette page montre à la fois. Il en reste de plus anciennes : elles apparaîtront à mesure que celles-ci seront acceptées, affectées manuellement, éclatées ou rattachées à un règlement.',
					{ max: PROPOSALS_MAX_LIMIT },
				)}
			</p>
		{/if}

		<ManualMatchModal
			open={manualOpen}
			onOpenChange={(v) => {
				manualOpen = v;
				if (!v) manualProposal = null;
			}}
			{bankAccountId}
			proposal={manualProposal}
			{accounts}
			{bankLedgerAccountId}
			onSuccess={onManualSuccess}
		/>

		<TransactionSplitModal
			open={splitOpen}
			onOpenChange={(v) => {
				splitOpen = v;
				if (!v) splitProposal = null;
			}}
			{bankAccountId}
			proposal={splitProposal}
			{accounts}
			onSuccess={onSplitSuccess}
		/>

		<LinkSettlementModal
			open={linkOpen}
			onOpenChange={(v) => {
				linkOpen = v;
				if (!v) linkProposal = null;
			}}
			proposal={linkProposal}
			onSuccess={onLinkSuccess}
		/>
	{/if}

	<!-- Story 15-5c (AC3, C38) — hors de la branche « liste non vide » : un lot
	     qui vide la liste (tous les refus de « Rejeter » portent sur une
	     transaction sortie de la liste) doit encore montrer son bilan. -->
	{#if failed.length > 0}
		<div class="mt-4" data-testid="reconciliation-failed">
			<h3 class="font-semibold text-red-700">
				{i18nMsg('reconciliation-labels-failed', 'Échecs partiels')}
			</h3>
			<ul class="list-disc pl-6 text-sm">
				{#each failed as f (f.bankTransactionId)}
					<li
						data-testid="reconciliation-failed-item"
						data-tx-id={f.bankTransactionId}
						data-error-code={f.errorCode}
						title={f.errorCode}
					>
						{describeTx(f.bankTransactionId)} — {failedProposalLabel(f.errorCode, f.details)}
					</li>
				{/each}
			</ul>
		</div>
	{/if}

	<!-- Story 28-5b (#526, AC6, C-28-5-10, C-28-5-22) — où retrouver une transaction rejetée.
	     En fin de section et hors de toute branche : visible liste vide, chargée ou en erreur
	     (un lot qui rejette la dernière ligne vide la liste). Texte vrai pour tout rôle. -->
	<p class="mt-4 text-sm text-text-muted" data-testid="reconciliation-rejected-hint">
		{i18nMsg(
			'reconciliation-labels-rejected-hint',
			'Une transaction rejetée quitte cette liste. Elle reste dans le détail de son import (Importer des relevés), où son rejet peut être annulé si votre rôle le permet.',
		)}
	</p>
</section>
