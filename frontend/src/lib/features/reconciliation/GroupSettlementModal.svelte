<!--
  Story 43-3 (#823) — « Régler plusieurs factures… » : une transaction du relevé règle plusieurs
  factures, à la manière de bexio (C-43-14).

  En tête, la ligne du relevé ; un champ de recherche (`GET …/group-candidates`, Story 43-2)
  dont chaque résultat s'AJOUTE d'un clic ; la liste des factures IMPUTÉES, chacune retirable,
  avec son reste dû ; en pied, le montant du relevé, le total imputé et l'« Écart », chacun
  avec un texte (jamais la seule couleur). Aucune présélection : la modale s'ouvre avec zéro
  facture imputée, et le nom de la contrepartie préremplit seulement la recherche. Une facture
  imputée est passée à `exclude` : elle ne revient pas dans les résultats.

  Les grandeurs (`group-settlement.ts`) se calculent en entiers de 1/10 000 sur les restes dus
  BRUTS : l'écart `e` (au centime) et l'arrondi `r` (sous le centime, une information). Dès que
  `e ≠ 0`, un sélecteur de compte apparaît sous l'écart, avec le sens en libellé (« Moins-payé »,
  « Trop-perçu » pour les clients ; « Débit supérieur / inférieur aux factures » pour les
  fournisseurs) et un préremplissage par le compte désigné utilisable (C-43-26). Le choix de
  l'utilisateur survit au changement de signe (C-e433-3).

  Les refus (D5) : une facture qui n'est plus réglable est retirée et nommée ; un refus de la
  transaction ferme la modale après un toast ; un refus du compte de l'écart s'affiche sous le
  sélecteur, une phrase par cause ; les autres laissent la sélection intacte.

  Gardes de concurrence (D4, `modal-guards.ts`) : une session neuve à chaque ouverture,
  fermeture, changement de transaction et soumission ; une recherche à la fois (anti-rebond,
  réponse dépassée ignorée).
-->
<script lang="ts">
	import { untrack } from 'svelte';
	import { toast } from 'svelte-sonner';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import * as Dialog from '$lib/components/ui/dialog';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { isApiError } from '$lib/shared/utils/api-client';
	import { errorMessageOf } from '$lib/shared/utils/error-message';
	import { formatSwissDate } from '$lib/features/reports/reports.api';
	import AccountAutocomplete from '$lib/features/journal-entries/AccountAutocomplete.svelte';
	import type { AccountResponse } from '$lib/features/accounts/accounts.types';
	import { getInvoiceSettings } from '$lib/features/invoices/invoices.api';
	import { acceptProposals, getGroupCandidates } from './reconciliation.api';
	import type {
		AcceptProposalInput,
		AcceptedProposal,
		FailedProposal,
		GroupCandidate,
		GroupCandidatesResponse,
		ReconciliationProposal,
	} from './reconciliation.types';
	import { counterpartyAccounts } from './reconciliation-counterparties';
	import {
		detailString,
		failedProposalLabel,
		failureReason,
		isDifferenceAccountRefusal,
	} from './failed-proposal-label';
	import {
		NO_CHOICE,
		candidateNumber,
		decompose,
		displayDue,
		effectiveDifferenceAccount,
		formatSignedUnits,
		formatTotal,
		formatUnits,
		gapSense,
		parseUnits,
		prefillAccount,
		selectAccount,
		submitBlock,
		type AccountChoice,
		type DesignatedAccounts,
		type GroupFamily,
	} from './group-settlement';
	import { createLatestRequest, createSessionGuard } from './modal-guards';

	type Props = {
		open: boolean;
		onOpenChange: (v: boolean) => void;
		bankAccountId: number;
		proposal: ReconciliationProposal | null;
		accounts: AccountResponse[];
		bankLedgerAccountId: number | null;
		/** Le règlement est enregistré : le parent relit sa liste. */
		onSuccess: () => void;
		/** La transaction n'est plus réglable ici : la modale s'est fermée, le parent relit sa liste. */
		onStale: () => void;
	};

	let {
		open,
		onOpenChange,
		bankAccountId,
		proposal,
		accounts,
		bankLedgerAccountId,
		onSuccess,
		onStale,
	}: Props = $props();

	/** Le délai anti-rebond de la recherche (D4). */
	const SEARCH_DEBOUNCE_MS = 250;
	/** Le plafond du nombre de factures d'un groupe (43-1a, `too_many`). */
	const MAX_INVOICES = 50;

	/**
	 * D5 — la facture n'est plus réglable : retirée des imputées, nommée, la recherche relue (elle
	 * filtre la date comme le `POST` : une facture retirée ne revient pas).
	 */
	const RETIRER_SUR = new Set([
		'RECONCILIATION_INVOICE_NOT_ELIGIBLE',
		'INVOICE_NOT_FOUND',
		'SUPPLIER_INVOICE_NOT_FOUND',
		'RECONCILIATION_SUPPLIER_PAYMENT_HOLD',
	]);

	/** D5 — le refus porte sur la transaction : toast, fermeture, relecture de la liste. */
	const FERMER_SUR = new Set([
		'BANK_TRANSACTION_NOT_FOUND',
		'RECONCILIATION_ALREADY_RECONCILED',
		'RECONCILIATION_ALREADY_REJECTED',
		'RECONCILIATION_TRANSACTION_NOT_CREDIT',
		'RECONCILIATION_TRANSACTION_NOT_DEBIT',
		'RECONCILIATION_CURRENCY_MISMATCH',
	]);

	/** D5 — les erreurs de la recherche qui ferment (transaction supprimée, ou plus en attente). */
	const FERMER_SUR_RECHERCHE: Record<string, string> = {
		NOT_FOUND: 'BANK_TRANSACTION_NOT_FOUND',
		RECONCILIATION_TRANSACTION_NOT_PENDING: 'RECONCILIATION_TRANSACTION_NOT_PENDING',
	};

	const session = createSessionGuard();
	const latest = createLatestRequest<GroupCandidatesResponse>(SEARCH_DEBOUNCE_MS);

	let query = $state('');
	let results = $state<GroupCandidate[]>([]);
	let hasMore = $state(false);
	let direction = $state<'client' | 'supplier' | null>(null);
	let maxDifference = $state<string | null>(null);
	/** Le lot de paiement qui retient le montant d'un débit (revue P4 de la 43-2, P4-1). */
	let heldByBatch = $state<number | null>(null);
	let searched = $state(false);
	let searching = $state(false);
	let searchError = $state('');
	let imputed = $state<GroupCandidate[]>([]);
	let designated = $state<DesignatedAccounts | null>(null);
	let choice = $state<AccountChoice>(NO_CHOICE);
	let accountError = $state('');
	let refusals = $state<string[]>([]);
	let submitting = $state(false);
	let errorMsg = $state('');

	/** La famille, par le sens de la transaction (C-43-15) ; la réponse de la recherche prime. */
	const family = $derived.by((): GroupFamily => {
		if (direction !== null) return direction;
		const u = proposal ? parseUnits(proposal.transaction.amount) : null;
		return u !== null && u < 0n ? 'supplier' : 'client';
	});

	const offered = $derived(counterpartyAccounts(accounts, bankLedgerAccountId));
	const decomposition = $derived(
		proposal ? decompose(proposal.transaction.amount, imputed.map((c) => c.amountDueRaw)) : null,
	);
	const sense = $derived(decomposition ? gapSense(family, decomposition.gap) : null);
	const prefill = $derived(prefillAccount(sense, designated, accounts, bankLedgerAccountId));
	const differenceAccountId = $derived(
		effectiveDifferenceAccount(choice, prefill, accounts, bankLedgerAccountId),
	);
	const block = $derived(
		submitBlock({
			count: imputed.length,
			decomposition,
			maxDifference,
			roundingDesignated: designated === null ? null : designated.roundingAccountId !== null,
			differenceAccountId,
		}),
	);
	/** Le total imputé, au brut s'il porte une fraction de centime (D3). */
	const totalText = $derived(decomposition ? formatTotal(decomposition.total) : '');
	const gapAbs = $derived(
		decomposition ? formatUnits(decomposition.gap < 0n ? -decomposition.gap : decomposition.gap) : '',
	);

	/** Le libellé de l'écart : son sens et son montant, au centime (D2 bis, C-43-26). */
	const gapText = $derived.by(() => {
		if (!decomposition) return '';
		if (block === 'gap-too-large') {
			return i18nMsg('reconciliation-group-gap-too-large', 'Écart trop grand (au plus { $max })', {
				max: maxDifference ?? '',
			});
		}
		switch (sense) {
			case 'client-under':
				return i18nMsg('reconciliation-group-gap-client-under', 'Moins-payé de { $amount }', {
					amount: gapAbs,
				});
			case 'client-over':
				return i18nMsg('reconciliation-group-gap-client-over', 'Trop-perçu de { $amount }', {
					amount: gapAbs,
				});
			case 'supplier-over':
				return i18nMsg(
					'reconciliation-group-gap-supplier-over',
					'Débit supérieur aux factures de { $amount }',
					{ amount: gapAbs },
				);
			case 'supplier-under':
				return i18nMsg(
					'reconciliation-group-gap-supplier-under',
					'Débit inférieur aux factures de { $amount }',
					{ amount: gapAbs },
				);
			default:
				return i18nMsg('reconciliation-group-gap-none', 'Aucun écart');
		}
	});

	/** Le sélecteur n'apparaît que pour un écart non nul dans le plafond. */
	const showAccount = $derived(
		decomposition !== null && decomposition.gap !== 0n && block !== 'gap-too-large',
	);

	/** Le texte qui dit pourquoi *Enregistrer* reste inactif, s'il y a lieu. */
	const blockText = $derived.by(() => {
		switch (block) {
			case 'too-few':
				return i18nMsg('reconciliation-group-min-two', 'Ajoutez au moins deux factures.');
			case 'account-required':
				// Le même texte que le refus du serveur (`difference_account_required`).
				return failedProposalLabel('RECONCILIATION_GROUP_INVALID', {
					reason: 'difference_account_required',
				});
			default:
				return '';
		}
	});

	// À chaque ouverture, fermeture ou changement de transaction : session neuve ; à l'ouverture,
	// remise à zéro, lecture des comptes désignés et première recherche (par le nom de la
	// contrepartie, sans rien imputer).
	// ⚠️ Seuls `open` et `proposal` sont suivis : le reste s'exécute hors suivi (`untrack`), sans
	// quoi la recherche, qui lit `query` et `imputed`, ferait de chaque frappe une réouverture.
	$effect(() => {
		const ouverte = open ? proposal : null;
		untrack(() => {
			const mySession = session.next();
			latest.cancel();
			submitting = false;
			if (!ouverte) return;
			query = ouverte.transaction.counterpartyName ?? '';
			results = [];
			hasMore = false;
			direction = null;
			maxDifference = null;
			heldByBatch = null;
			searched = false;
			searching = false;
			searchError = '';
			imputed = [];
			designated = null;
			choice = NO_CHOICE;
			accountError = '';
			refusals = [];
			errorMsg = '';
			void loadDesignated(mySession);
			search(true);
		});
	});

	/** Les comptes désignés (préremplissage, compte d'arrondi) ; illisibles : `null` (C-e433-2). */
	async function loadDesignated(mySession: number): Promise<void> {
		try {
			const s = await getInvoiceSettings();
			if (!session.isCurrent(mySession)) return;
			designated = {
				roundingAccountId: s.defaultRoundingAccountId,
				bankFeesAccountId: s.defaultBankFeesAccountId,
			};
		} catch {
			if (session.isCurrent(mySession)) designated = null;
		}
	}

	/** Lance la recherche (anti-rebond, sauf `immediate`), les imputées exclues. */
	function search(immediate = false): void {
		if (!proposal) return;
		const txId = proposal.bankTransactionId;
		const q = query;
		const exclude = imputed.map((c) => c.id);
		const mySession = session.current();
		latest.run(
			() => getGroupCandidates(txId, { q, exclude }),
			{
				onStart: () => {
					if (session.isCurrent(mySession)) searching = true;
				},
				onResult: (r) => {
					if (!session.isCurrent(mySession)) return;
					searching = false;
					searched = true;
					searchError = '';
					results = r.candidates;
					hasMore = r.hasMore;
					direction = r.direction;
					maxDifference = r.maxDifference;
					heldByBatch = r.heldByPaymentBatch?.paymentBatchId ?? null;
				},
				onError: (e) => {
					if (!session.isCurrent(mySession)) return;
					searching = false;
					const fermer = isApiError(e) ? FERMER_SUR_RECHERCHE[e.code] : undefined;
					if (fermer !== undefined) {
						closeStale(failedProposalLabel(fermer));
						return;
					}
					searchError = errorMessageOf(e);
				},
			},
			immediate,
		);
	}

	/** D5 — dit le refus en toast, ferme la modale, et fait relire la liste du parent. */
	function closeStale(message: string): void {
		toast.error(message);
		session.next();
		latest.cancel();
		onOpenChange(false);
		onStale();
	}

	function addInvoice(c: GroupCandidate): void {
		if (imputed.length >= MAX_INVOICES || imputed.some((x) => x.id === c.id)) return;
		imputed = [...imputed, c];
		results = results.filter((x) => x.id !== c.id);
		refusals = [];
		search(true);
	}

	function removeInvoice(id: number): void {
		imputed = imputed.filter((x) => x.id !== id);
		refusals = [];
		search(true);
	}

	function onAccountSelect(id: number | null): void {
		choice = selectAccount(id);
		accountError = '';
	}

	/** Le numéro d'une facture en cause dans un refus : `details.invoiceNumber`, sinon la liste imputée. */
	function refusedInvoiceId(details: unknown): number | null {
		if (typeof details !== 'object' || details === null) return null;
		const d = details as { invoiceId?: unknown; supplierInvoiceId?: unknown };
		const id = family === 'client' ? d.invoiceId : d.supplierInvoiceId;
		return typeof id === 'number' ? id : null;
	}

	function refusedInvoiceName(details: unknown): string | null {
		const number = detailString(details, 'invoiceNumber');
		if (number !== null) return number;
		const id = refusedInvoiceId(details);
		if (id === null) return null;
		const c = imputed.find((x) => x.id === id);
		return c ? candidateNumber(c) : `#${id}`;
	}

	/** « Facture {numéro} : {libellé} », ou le libellé seul si aucune facture n'est en cause. */
	function refusalText(f: FailedProposal, sentAccountId: number | null): string {
		const label = failedProposalLabel(f.errorCode, f.details, {
			group: true,
			differenceAccountId: sentAccountId,
		});
		const name = refusedInvoiceName(f.details);
		return name === null
			? label
			: i18nMsg('reconciliation-group-refusal-invoice', 'Facture { $number } : { $label }', {
					number: name,
					label,
				});
	}

	/**
	 * Relit les restes dus des factures imputées (C-e433-4) : chacune cherchée par son numéro ;
	 * une facture que la recherche ne rend plus n'est plus réglable, elle est retirée et nommée.
	 */
	async function rereadImputed(txId: number, mySession: number): Promise<void> {
		const current = imputed;
		const reread = await Promise.all(
			current.map(async (c) => {
				try {
					const r = await getGroupCandidates(txId, { q: candidateNumber(c), limit: 100 });
					return r.candidates.find((x) => x.kind === c.kind && x.id === c.id) ?? null;
				} catch {
					return c;
				}
			}),
		);
		if (!session.isCurrent(mySession)) return;
		const kept: GroupCandidate[] = [];
		const removed: string[] = [];
		current.forEach((c, i) => {
			const fresh = reread[i];
			if (fresh === null) removed.push(candidateNumber(c));
			else kept.push(fresh);
		});
		imputed = kept;
		refusals = [
			...refusals,
			...removed.map((number) =>
				i18nMsg('reconciliation-group-refusal-invoice', 'Facture { $number } : { $label }', {
					number,
					label: failedProposalLabel('RECONCILIATION_INVOICE_NOT_ELIGIBLE'),
				}),
			),
		];
	}

	/** D5 — un refus de la proposition, selon son genre. */
	async function handleRefusal(
		f: FailedProposal,
		txId: number,
		sentAccountId: number | null,
		mySession: number,
	): Promise<void> {
		if (FERMER_SUR.has(f.errorCode)) {
			closeStale(failedProposalLabel(f.errorCode, f.details));
			return;
		}
		const text = refusalText(f, sentAccountId);
		if (RETIRER_SUR.has(f.errorCode)) {
			const id = refusedInvoiceId(f.details);
			if (id !== null) imputed = imputed.filter((x) => x.id !== id);
			refusals = [text];
			search(true);
			return;
		}
		const stale =
			f.errorCode === 'RECONCILIATION_GROUP_DIFFERENCE_TOO_LARGE' ||
			(f.errorCode === 'RECONCILIATION_GROUP_INVALID' &&
				failureReason(f.details) === 'difference_account_required');
		if (f.errorCode === 'RECONCILIATION_GROUP_DIFFERENCE_TOO_LARGE') {
			const max = detailString(f.details, 'maxDifference');
			if (max !== null) maxDifference = max;
		}
		if (stale) await rereadImputed(txId, mySession);
		if (!session.isCurrent(mySession)) return;
		if (isDifferenceAccountRefusal(f.errorCode, f.details, sentAccountId)) {
			accountError = failedProposalLabel(f.errorCode, f.details, {
				group: true,
				differenceAccountId: sentAccountId,
			});
			return;
		}
		refusals = [...refusals, text];
	}

	/** Le compte nommé « numéro nom », ou l'identifiant s'il n'est pas dans la liste. */
	function accountLabel(id: number | null | undefined): string {
		if (id === null || id === undefined) return '—';
		const a = accounts.find((x) => x.id === id);
		return a ? `${a.number} ${a.name}` : `#${id}`;
	}

	/**
	 * D3, contrôle de cohérence : `gap` contre `e`, `roundingDifference` contre `r`. S'ils
	 * diffèrent (montants changés entre la recherche et l'envoi), un toast dit ce qui a été
	 * réellement passé, et sur quels comptes.
	 */
	function checkCoherence(a: AcceptedProposal): void {
		if (!decomposition) return;
		const gap = parseUnits(a.gap);
		const rounding = parseUnits(a.roundingDifference);
		if (gap === null || rounding === null) return;
		if (gap === decomposition.gap && rounding === decomposition.rounding) return;
		toast.warning(
			i18nMsg(
				'reconciliation-group-amounts-changed',
				'Les montants ont changé avant l’enregistrement. Écart réellement passé : { $gap } (compte { $account }) ; arrondi passé au compte d’arrondi : { $rounding }.',
				{
					gap: formatUnits(gap),
					account: accountLabel(a.differenceAccountId),
					rounding: formatSignedUnits(rounding),
				},
			),
		);
	}

	async function submit(): Promise<void> {
		if (!proposal || block !== null || submitting || !decomposition) return;
		const txId = proposal.bankTransactionId;
		const ids = imputed.map((c) => c.id);
		const sentAccountId = decomposition.gap !== 0n ? differenceAccountId : null;
		const account = sentAccountId === null ? {} : { differenceAccountId: sentAccountId };
		const body: AcceptProposalInput =
			family === 'client'
				? { type: 'invoice_group', bankTransactionId: txId, invoiceIds: ids, ...account }
				: {
						type: 'supplier_invoice_group',
						bankTransactionId: txId,
						supplierInvoiceIds: ids,
						...account,
					};
		latest.cancel();
		const mySession = session.next();
		submitting = true;
		errorMsg = '';
		accountError = '';
		refusals = [];
		try {
			const r = await acceptProposals(bankAccountId, [body]);
			const accepted = r.accepted.find((a) => a.bankTransactionId === txId);
			if (accepted) {
				if (!session.isCurrent(mySession)) {
					// Fermée pendant l'envoi : le règlement a eu lieu ; le parent relit sa liste si
					// aucune modale n'est ouverte, une modale rouverte n'est pas touchée.
					if (!open) onSuccess();
					return;
				}
				checkCoherence(accepted);
				toast.success(
					i18nMsg(
						'reconciliation-group-success',
						'Règlement enregistré : { $count } factures réglées par cette transaction.',
						{ count: ids.length },
					),
				);
				onSuccess();
				onOpenChange(false);
				return;
			}
			if (!session.isCurrent(mySession)) return;
			const f = r.failed.find((x) => x.bankTransactionId === txId) ?? r.failed[0];
			if (f) await handleRefusal(f, txId, sentAccountId, mySession);
		} catch (e) {
			if (!session.isCurrent(mySession)) return;
			errorMsg = errorMessageOf(e);
		} finally {
			if (session.isCurrent(mySession)) submitting = false;
		}
	}
</script>

<Dialog.Root {open} {onOpenChange}>
	<Dialog.Content
		class="max-h-[90vh] overflow-y-auto sm:max-w-4xl"
		data-testid="group-settlement-modal"
	>
		<Dialog.Header>
			<Dialog.Title>
				{i18nMsg('reconciliation-group-modal-title', 'Régler plusieurs factures')}
			</Dialog.Title>
		</Dialog.Header>

		{#if proposal}
			<p class="text-sm text-text-muted" data-testid="group-tx-summary">
				{formatSwissDate(proposal.transaction.valueDate ?? proposal.transaction.bookingDate)} — {proposal
					.transaction.amount}
				{proposal.transaction.currency}{#if proposal.transaction.counterpartyName}
					{' '}— {proposal.transaction.counterpartyName}{/if}
			</p>

			<div class="mt-3">
				<label class="mb-1 block text-xs text-text-muted" for="group-search-input">
					{i18nMsg('reconciliation-group-search-label', 'Rechercher une facture')}
				</label>
				<Input
					id="group-search-input"
					data-testid="group-search-input"
					type="search"
					autocomplete="off"
					placeholder={i18nMsg(
						'reconciliation-group-search-placeholder',
						'Nom du tiers, numéro ou référence',
					)}
					value={query}
					oninput={(e) => {
						query = e.currentTarget.value;
						search();
					}}
					disabled={submitting}
				/>
			</div>

			<section class="mt-2" data-testid="group-results-section">
				{#if searching}
					<p class="text-sm text-text-muted" data-testid="group-loading">
						{i18nMsg('reconciliation-group-loading', 'Recherche…')}
					</p>
				{:else if searchError}
					<div
						class="rounded-md border border-destructive bg-destructive/10 px-3 py-2 text-sm text-destructive"
						data-testid="group-load-error"
					>
						{searchError}
					</div>
				{:else if searched && direction === null}
					<p class="text-sm text-text-muted" data-testid="group-no-direction">
						{i18nMsg(
							'reconciliation-group-no-direction',
							'Aucune facture à proposer pour cette transaction.',
						)}
					</p>
				{:else if searched && heldByBatch !== null}
					<!-- Revue P4 de la 43-2 (P4-1) : un lot de paiement en cours retient ce montant ; la
					     liste est vide parce que tout règlement y serait refusé, et l'écran dit pourquoi. -->
					<p
						class="rounded-md border border-warning bg-warning/10 px-3 py-2 text-sm"
						data-testid="group-held-by-batch"
						data-payment-batch-id={heldByBatch}
					>
						{i18nMsg(
							'reconciliation-group-held-by-batch',
							'Le montant de ce débit est retenu par le lot de paiement n° { $batch }, en cours sur ce compte : ce débit en est peut-être le paiement. Pour régler ses factures ici, annulez d’abord le lot.',
							{ batch: heldByBatch },
						)}
					</p>
				{:else if searched && results.length === 0}
					<p class="text-sm text-text-muted" data-testid="group-results-empty">
						{i18nMsg(
							'reconciliation-group-results-empty',
							'Aucune facture ne correspond à cette recherche.',
						)}
					</p>
				{:else if results.length > 0}
					<div class="max-h-56 overflow-x-auto overflow-y-auto">
						<table
							class="w-full table-auto text-sm [&_td]:px-2 [&_td]:py-1 [&_th]:whitespace-nowrap [&_th]:px-2"
							data-testid="group-results-table"
						>
							<thead>
								<tr>
									<th class="text-left">{i18nMsg('reconciliation-group-col-invoice', 'Facture')}</th>
									<th class="text-left">
										{family === 'client'
											? i18nMsg('reconciliation-group-col-client', 'Client')
											: i18nMsg('reconciliation-group-col-supplier', 'Fournisseur')}
									</th>
									<th class="text-left">{i18nMsg('reconciliation-group-col-date', 'Date')}</th>
									<th class="text-right">{i18nMsg('reconciliation-group-col-due', 'Reste dû')}</th>
									<th><span class="sr-only">{i18nMsg('reconciliation-group-add', 'Ajouter')}</span></th>
								</tr>
							</thead>
							<tbody>
								{#each results as c (`${c.kind}-${c.id}`)}
									<tr data-testid="group-result-row" data-invoice-id={c.id}>
										<td class="whitespace-nowrap">{candidateNumber(c)}</td>
										<td>{c.contactName}</td>
										<td class="whitespace-nowrap">{formatSwissDate(c.date)}</td>
										<td class="text-right whitespace-nowrap">{displayDue(c.amountDueRaw)}</td>
										<td class="text-right">
											<Button
												size="sm"
												variant="outline"
												onclick={() => addInvoice(c)}
												disabled={submitting || imputed.length >= MAX_INVOICES}
												aria-label={i18nMsg(
													'reconciliation-group-add-aria',
													'Ajouter la facture { $number }',
													{ number: candidateNumber(c) },
												)}
												data-testid="group-add"
												data-invoice-id={c.id}
											>
												{i18nMsg('reconciliation-group-add', 'Ajouter')}
											</Button>
										</td>
									</tr>
								{/each}
							</tbody>
						</table>
					</div>
				{/if}
				{#if hasMore && !searching}
					<p class="mt-1 text-sm text-text-muted" data-testid="group-has-more">
						{i18nMsg(
							'reconciliation-group-has-more',
							'D’autres factures correspondent : précisez la recherche.',
						)}
					</p>
				{/if}
				{#if imputed.length >= MAX_INVOICES}
					<p class="mt-1 text-sm text-text-muted" data-testid="group-max-reached">
						{i18nMsg(
							'reconciliation-group-max-reached',
							'Un règlement compte au plus { $max } factures.',
							{ max: MAX_INVOICES },
						)}
					</p>
				{/if}
			</section>

			<section class="mt-4" data-testid="group-imputed-section">
				<h3 class="text-sm font-semibold">
					{i18nMsg('reconciliation-group-imputed-title', 'Factures imputées')}
				</h3>
				{#if imputed.length === 0}
					<p class="mt-1 text-sm text-text-muted" data-testid="group-imputed-empty">
						{i18nMsg(
							'reconciliation-group-imputed-empty',
							'Aucune facture imputée : ajoutez-en au moins deux depuis la recherche.',
						)}
					</p>
				{:else}
					<table
						class="mt-1 w-full table-auto text-sm [&_td]:px-2 [&_td]:py-1"
						data-testid="group-imputed-table"
					>
						<tbody>
							{#each imputed as c (`${c.kind}-${c.id}`)}
								<tr data-testid="group-imputed-row" data-invoice-id={c.id}>
									<td class="whitespace-nowrap">{candidateNumber(c)}</td>
									<td>{c.contactName}</td>
									<td class="text-right whitespace-nowrap" data-testid="group-imputed-due">
										{displayDue(c.amountDueRaw)}
									</td>
									<td class="text-right">
										<Button
											size="sm"
											variant="ghost"
											onclick={() => removeInvoice(c.id)}
											disabled={submitting}
											aria-label={i18nMsg(
												'reconciliation-group-remove-aria',
												'Retirer la facture { $number }',
												{ number: candidateNumber(c) },
											)}
											data-testid="group-remove"
											data-invoice-id={c.id}
										>
											{i18nMsg('reconciliation-group-remove', 'Retirer')}
										</Button>
									</td>
								</tr>
							{/each}
						</tbody>
					</table>
				{/if}
			</section>

			<section class="mt-4 border-t pt-3 text-sm" data-testid="group-footer">
				<dl class="grid grid-cols-[1fr_auto] gap-x-4 gap-y-1">
					<dt>{i18nMsg('reconciliation-group-total-transaction', 'Montant du relevé')}</dt>
					<dd class="text-right" data-testid="group-transaction-amount">
						{decomposition ? formatUnits(decomposition.transaction) : proposal.transaction.amount}
					</dd>
					<dt>{i18nMsg('reconciliation-group-total-imputed', 'Total imputé')}</dt>
					<dd class="text-right" data-testid="group-total-imputed">{totalText}</dd>
					<dt>{i18nMsg('reconciliation-group-gap-label', 'Écart')}</dt>
					<dd
						class="text-right font-semibold {block === 'gap-too-large' ? 'text-destructive' : ''}"
						data-testid="group-gap"
						data-gap-sense={sense ?? 'none'}
					>
						{gapText}
					</dd>
				</dl>

				{#if decomposition && decomposition.rounding !== 0n}
					<!-- D2 bis — l'arrondi est une information, avec ou sans écart. -->
					<p class="mt-2 text-text-muted" data-testid="group-rounding">
						{i18nMsg(
							'reconciliation-group-rounding-info',
							'Arrondi de { $amount } passé au compte d’arrondi',
							{ amount: formatSignedUnits(decomposition.rounding) },
						)}
					</p>
					{#if block === 'rounding-not-configured'}
						<p
							class="mt-1 rounded-md border border-warning bg-warning/10 px-3 py-2"
							data-testid="group-rounding-not-configured"
						>
							{i18nMsg(
								'reconciliation-group-rounding-not-configured',
								'Aucun compte de différences d’arrondi n’est désigné : demandez à l’administrateur de le régler.',
							)}
						</p>
					{/if}
				{/if}

				{#if showAccount}
					<div class="mt-3" data-testid="group-account-section">
						<label class="mb-1 block text-xs text-text-muted" for="group-difference-account">
							{i18nMsg('reconciliation-group-account-label', 'Compte de l’écart')}
						</label>
						<AccountAutocomplete
							id="group-difference-account"
							accounts={offered}
							value={differenceAccountId}
							onSelect={onAccountSelect}
							allowClear={true}
							excludeForeignCurrency={true}
							disabled={submitting}
						/>
						{#if accountError}
							<p
								class="mt-1 text-sm text-destructive"
								role="alert"
								data-testid="group-account-error"
							>
								{accountError}
							</p>
						{/if}
					</div>
				{/if}
			</section>

			{#if refusals.length > 0}
				<ul
					class="mt-2 list-disc rounded-md border border-destructive bg-destructive/10 py-2 pr-3 pl-8 text-sm text-destructive"
					role="alert"
					data-testid="group-refusals"
				>
					{#each refusals as r, i (i)}
						<li data-testid="group-refusal">{r}</li>
					{/each}
				</ul>
			{/if}

			{#if errorMsg}
				<div
					class="mt-2 rounded-md border border-destructive bg-destructive/10 px-3 py-2 text-sm text-destructive"
					role="alert"
					data-testid="group-error"
				>
					{errorMsg}
				</div>
			{/if}

			{#if blockText}
				<p class="mt-2 text-sm text-text-muted" data-testid="group-block-hint">{blockText}</p>
			{/if}
		{/if}

		<Dialog.Footer>
			<Button
				variant="outline"
				onclick={() => onOpenChange(false)}
				disabled={submitting}
				data-testid="group-cancel"
			>
				{i18nMsg('reconciliation-group-cancel', 'Annuler')}
			</Button>
			<Button onclick={submit} disabled={submitting || block !== null} data-testid="group-submit">
				{i18nMsg('reconciliation-group-submit', 'Enregistrer')}
			</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>
