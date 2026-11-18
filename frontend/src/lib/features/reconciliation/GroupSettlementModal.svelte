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
	import { onDestroy, untrack } from 'svelte';
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
		MAX_GROUP_INVOICES,
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
		submitBlocks,
		type AccountChoice,
		type DesignatedAccounts,
		type GroupFamily,
	} from './group-settlement';
	import { gapSenseLabel } from './gap-label';
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
	const MAX_INVOICES = MAX_GROUP_INVOICES;

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
	// Revue P1 (B-L11) : un délai anti-rebond en attente ne part pas après le démontage.
	onDestroy(() => latest.cancel());
	/**
	 * Revue P1 (E-L2) : l'ouverture qui a lancé la lecture des réglages. Distincte de `session`,
	 * qu'une soumission renouvelle : une soumission partie avant leur arrivée ne les perd plus.
	 */
	let openToken = 0;

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
	/**
	 * Revue P2 (L-1) : le serveur a demandé un compte de l'écart (`difference_account_required`) ;
	 * le sélecteur s'affiche même si l'écart calculé ici est nul. Tombe quand le groupe change.
	 */
	let serverWantsAccount = $state(false);
	/**
	 * Revue P2 (L-1) : les factures imputées dont la relecture a échoué ; leur phrase reste tant
	 * qu'elles sont imputées sans avoir été relues (elle ne s'efface pas avec les refus).
	 */
	let notReread = $state<number[]>([]);
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
	/** Toutes les causes qui retiennent *Enregistrer* (revue P1 : aucune n'en masque une autre). */
	const blocks = $derived(
		submitBlocks({
			count: imputed.length,
			decomposition,
			maxDifference,
			roundingDesignated: designated === null ? null : designated.roundingAccountId !== null,
			differenceAccountId,
			serverWantsAccount,
		}),
	);
	/** Au moins deux factures : avant, l'écran n'annonce aucun écart (revue P1, A M-1, E-M1). */
	const enoughInvoices = $derived(imputed.length >= 2);
	const tooLarge = $derived(blocks.includes('gap-too-large'));
	/** Le total imputé, au brut s'il porte une fraction de centime (D3). */
	const totalText = $derived(decomposition ? formatTotal(decomposition.total) : '');

	/** Le libellé de l'écart : son sens et son montant, au centime (D2 bis, C-43-26). */
	const gapText = $derived.by(() => {
		if (!decomposition || !enoughInvoices) return '—';
		if (tooLarge) {
			// Revue P2 (L-5) : le sens, inséré après le deux-points, prend une minuscule.
			return i18nMsg(
				'reconciliation-group-gap-too-large',
				'Écart trop grand : { $gap } (au plus { $max })',
				{ gap: gapSenseLabel(family, decomposition.gap, true), max: maxDifference ?? '' },
			);
		}
		return gapSenseLabel(family, decomposition.gap);
	});

	/**
	 * Le sélecteur n'apparaît que pour un écart non nul — ou un compte que le serveur a demandé
	 * (revue P2, L-1) —, dans le plafond, d'au moins deux factures, le plafond lu.
	 */
	const showAccount = $derived(
		decomposition !== null &&
			enoughInvoices &&
			(decomposition.gap !== 0n || serverWantsAccount) &&
			!tooLarge &&
			!blocks.includes('unreadable'),
	);

	/**
	 * Les phrases qui disent pourquoi *Enregistrer* reste inactif — une par cause (revue P1,
	 * B-L4, E-L6). L'arrondi sans compte a son propre avis, plus haut.
	 */
	const blockTexts = $derived(
		blocks.flatMap((b): string[] => {
			switch (b) {
				case 'too-few':
					return [
						i18nMsg('reconciliation-group-min-two', 'Ajoutez au moins deux factures.'),
					];
				case 'unreadable':
					return [
						i18nMsg(
							'reconciliation-group-block-unreadable',
							'Kesh n’a pas pu lire un montant ou le plafond de l’écart : relancez la recherche.',
						),
					];
				case 'gap-too-large':
					return enoughInvoices
						? [
								i18nMsg(
									'reconciliation-group-block-gap-too-large',
									'L’écart dépasse le plafond admis : ajoutez ou retirez une facture, ou réglez la transaction autrement.',
								),
							]
						: [];
				case 'account-required':
					// Le même texte que le refus du serveur (`difference_account_required`).
					return [
						failedProposalLabel('RECONCILIATION_GROUP_INVALID', {
							reason: 'difference_account_required',
						}),
					];
				default:
					return [];
			}
		}),
	);

	// À chaque ouverture, fermeture ou changement de transaction : session neuve ; à l'ouverture,
	// remise à zéro, lecture des comptes désignés et première recherche (par le nom de la
	// contrepartie, sans rien imputer).
	// ⚠️ Seuls `open` et `proposal` sont suivis : le reste s'exécute hors suivi (`untrack`), sans
	// quoi la recherche, qui lit `query` et `imputed`, ferait de chaque frappe une réouverture.
	$effect(() => {
		const ouverte = open ? proposal : null;
		untrack(() => {
			session.next();
			const myOpen = ++openToken;
			latest.cancel();
			submitting = false;
			searching = false;
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
			serverWantsAccount = false;
			notReread = [];
			errorMsg = '';
			void loadDesignated(myOpen);
			search(true);
		});
	});

	/**
	 * Les comptes désignés (préremplissage, compte d'arrondi) ; illisibles : `null` (C-e433-2).
	 * Gardés par l'ouverture, non par la session : une soumission ne les perd pas (E-L2).
	 */
	async function loadDesignated(myOpen: number): Promise<void> {
		try {
			const s = await getInvoiceSettings();
			if (myOpen !== openToken) return;
			designated = {
				roundingAccountId: s.defaultRoundingAccountId,
				bankFeesAccountId: s.defaultBankFeesAccountId,
			};
		} catch {
			if (myOpen === openToken) designated = null;
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
					// Revue P1 (E-L5) : l'invite « d'autres factures » ne survit pas à une erreur.
					hasMore = false;
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
		searching = false;
		onOpenChange(false);
		onStale();
	}

	function addInvoice(c: GroupCandidate): void {
		if (imputed.length >= MAX_INVOICES || imputed.some((x) => x.id === c.id)) return;
		imputed = [...imputed, c];
		results = results.filter((x) => x.id !== c.id);
		// Ajoutée depuis la recherche, son reste dû est frais (revue P2, L-1).
		notReread = notReread.filter((x) => x !== c.id);
		clearMessages();
		search(true);
	}

	function removeInvoice(id: number): void {
		imputed = imputed.filter((x) => x.id !== id);
		notReread = notReread.filter((x) => x !== id);
		clearMessages();
		search(true);
	}

	/**
	 * Revue P1 (B-L1, E-L4) : un refus ou une erreur ne survit pas à un changement du groupe — le
	 * seul geste qui change l'écart et son signe. La phrase « non relue » reste, elle, tant que
	 * la facture est imputée sans relecture (revue P2, L-1).
	 */
	function clearMessages(): void {
		refusals = [];
		accountError = '';
		errorMsg = '';
		serverWantsAccount = false;
	}

	/** Les phrases des factures imputées dont le reste dû n'a pas pu être relu (revue P2, L-1). */
	const notRereadTexts = $derived(
		imputed
			.filter((c) => notReread.includes(c.id))
			.map((c) =>
				i18nMsg(
					'reconciliation-group-not-reread',
					'Facture { $number } : son reste dû n’a pas pu être relu, l’écart affiché peut être faux. Retirez-la puis ajoutez-la de nouveau pour le relire.',
					{ number: candidateNumber(c) },
				),
			),
	);

	function onAccountSelect(id: number | null): void {
		choice = selectAccount(id);
		accountError = '';
	}

	/** L'identifiant de la facture en cause dans un refus (`invoiceId` ou `supplierInvoiceId`). */
	function refusedInvoiceId(details: unknown): number | null {
		if (typeof details !== 'object' || details === null) return null;
		const d = details as { invoiceId?: unknown; supplierInvoiceId?: unknown };
		const id = family === 'client' ? d.invoiceId : d.supplierInvoiceId;
		return typeof id === 'number' ? id : null;
	}

	/** Le numéro d'une facture en cause dans un refus : `details.invoiceNumber`, sinon la liste imputée. */
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
	 * Relit les restes dus des factures imputées (C-e433-4, C-e433-12) : chacune cherchée par son
	 * numéro. Une facture que la recherche ne rend pas — numéro absent, recherche trop large,
	 * erreur — n'est **jamais** retirée ni dite « plus réglable » : la recherche textuelle ne le
	 * prouve pas (revue P1, B-M2, E-L1). Elle reste, avec son montant d'avant, et une phrase qui
	 * demeure tant qu'elle n'est pas relue dit que l'écart peut être faux et comment la relire
	 * (revue P2, L-1).
	 *
	 * Seuls deux refus déclenchent la relecture : `RECONCILIATION_GROUP_DIFFERENCE_TOO_LARGE` et
	 * `difference_account_required`, les seuls refus du groupe qui portent sur l'écart, donc sur
	 * les montants (registre, C-e433-12).
	 */
	async function rereadImputed(txId: number, mySession: number): Promise<void> {
		const current = imputed;
		const reread = await Promise.all(
			current.map(async (c): Promise<GroupCandidate | null> => {
				const number = candidateNumber(c);
				if (number.startsWith('#')) return null;
				try {
					const r = await getGroupCandidates(txId, { q: number, limit: 100 });
					return r.candidates.find((x) => x.kind === c.kind && x.id === c.id) ?? null;
				} catch {
					return null;
				}
			}),
		);
		if (!session.isCurrent(mySession)) return;
		const failed: number[] = [];
		imputed = current.map((c, i) => {
			const fresh = reread[i];
			if (fresh === null) {
				failed.push(c.id);
				return c;
			}
			return fresh;
		});
		notReread = failed;
		// Revue P3 ciblée (L-A) : relus, les montants de l'écran sont ceux du serveur ; sa demande
		// d'un compte reposait sur les montants remplacés et ne s'impose plus.
		if (failed.length === 0) serverWantsAccount = false;
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
			if (id !== null) {
				imputed = imputed.filter((x) => x.id !== id);
				notReread = notReread.filter((x) => x !== id);
			}
			// Revue P3 ciblée (L-A) : les factures ont changé, la demande d'un compte tombe.
			serverWantsAccount = false;
			refusals = [text];
			search(true);
			return;
		}
		const accountRequired =
			f.errorCode === 'RECONCILIATION_GROUP_INVALID' &&
			failureReason(f.details) === 'difference_account_required';
		const stale = f.errorCode === 'RECONCILIATION_GROUP_DIFFERENCE_TOO_LARGE' || accountRequired;
		if (f.errorCode === 'RECONCILIATION_GROUP_DIFFERENCE_TOO_LARGE') {
			const max = detailString(f.details, 'maxDifference');
			if (max !== null) maxDifference = max;
		}
		if (stale) await rereadImputed(txId, mySession);
		if (!session.isCurrent(mySession)) return;
		if (accountRequired) {
			// Revue P2 (L-1) : le serveur veut un compte ; le sélecteur s'affiche, même si l'écart
			// calculé ici reste nul (relecture impossible). Revue P2 (L-2) : la phrase « Choisissez
			// le compte de l'écart » est déjà dite sous le pied (`account-required`) ; elle ne
			// se répète pas sous le sélecteur. Revue P3 ciblée (L-A) : la demande ne s'impose que
			// si une facture n'a pas pu être relue ; relus, les montants disent eux-mêmes l'écart.
			serverWantsAccount = notReread.length > 0;
			if (showAccount) {
				if (blocks.includes('account-required')) return;
				// Revue P3 ciblée (L-B) : un compte est déjà dans le champ (choisi et gardé, fiche
				// D2 bis, ou prérempli) alors que l'envoi n'en portait pas — l'écart y était nul ;
				// « Choisissez le compte » serait faux, la phrase dit de vérifier ce compte.
				accountError =
					differenceAccountId !== null && differenceAccountId !== sentAccountId
						? i18nMsg(
								'reconciliation-group-account-check-retry',
								'Kesh demande un compte pour l’écart : vérifiez le compte choisi, puis enregistrez de nouveau.',
							)
						: failedProposalLabel(f.errorCode, f.details, {
								group: true,
								differenceAccountId: sentAccountId,
							});
				return;
			}
			if (notReread.length === 0 && blocks.length === 0) {
				// Revue P3 ciblée (L-A) : relus, les montants ne demandent aucun compte ; l'écran est
				// à jour et *Enregistrer* actif — jamais un refus muet.
				refusals = [
					...refusals,
					i18nMsg(
						'reconciliation-group-reread-retry',
						'Les restes dus ont été relus : vérifiez l’écart, puis enregistrez de nouveau.',
					),
				];
				return;
			}
		}
		// Sous le sélecteur seulement s'il est affiché après la relecture ; sinon avec les autres
		// refus : jamais un refus muet (revue P1, B-M3, E-M2).
		if (isDifferenceAccountRefusal(f.errorCode, f.details, sentAccountId) && showAccount) {
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
					gap: gapSenseLabel(family, gap),
					account: accountLabel(a.differenceAccountId),
					rounding: formatSignedUnits(rounding),
				},
			),
		);
	}

	async function submit(): Promise<void> {
		if (!proposal || blocks.length > 0 || submitting || !decomposition) return;
		const txId = proposal.bankTransactionId;
		const ids = imputed.map((c) => c.id);
		const sentAccountId =
			decomposition.gap !== 0n || serverWantsAccount ? differenceAccountId : null;
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
		// Revue P2 (L-3) : une recherche en cours (relancée par un ajout ou un retrait) est avortée
		// par l'envoi ; si la modale reste ouverte, elle est relancée, sans quoi la liste des
		// résultats resterait périmée (une facture retirée n'y reviendrait pas).
		const searchAborted = latest.busy();
		let settled = false;
		latest.cancel();
		// Revue P1 (B-M1) : la recherche avortée ne rendra rien ; son état « Recherche… » tombe ici.
		searching = false;
		const mySession = session.next();
		submitting = true;
		errorMsg = '';
		accountError = '';
		refusals = [];
		try {
			const r = await acceptProposals(bankAccountId, [body]);
			const accepted = r.accepted.find((a) => a.bankTransactionId === txId);
			if (accepted) {
				settled = true;
				const successText = i18nMsg(
					'reconciliation-group-success',
					'Règlement enregistré : { $count } factures réglées par cette transaction.',
					{ count: ids.length },
				);
				if (!session.isCurrent(mySession)) {
					// Fermée pendant l'envoi : le règlement a eu lieu, et le toast le dit (revue P1,
					// E-L7) ; le parent relit sa liste si aucune modale n'est ouverte, une modale
					// rouverte n'est pas touchée.
					toast.success(successText);
					if (!open) onSuccess();
					return;
				}
				checkCoherence(accepted);
				toast.success(successText);
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
			if (session.isCurrent(mySession)) {
				submitting = false;
				if (searchAborted && !settled && open && !latest.busy()) search(true);
			}
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
				<!-- Revue P1 (A L-1, E-L8) : le montant au centime, comme au pied de la fenêtre. -->
				{formatSwissDate(proposal.transaction.valueDate ?? proposal.transaction.bookingDate)} — {displayDue(
					proposal.transaction.amount,
				)}
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

			<section class="mt-2" data-testid="group-results-section" aria-live="polite">
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
				{#if hasMore && !searching && !searchError}
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
						class="mt-1 w-full table-auto text-sm [&_td]:px-2 [&_td]:py-1 [&_th]:px-2"
						data-testid="group-imputed-table"
					>
						<!-- Revue P1 (B-L5) : des colonnes nommées pour un lecteur d'écran. -->
						<thead class="sr-only">
							<tr>
								<th>{i18nMsg('reconciliation-group-col-invoice', 'Facture')}</th>
								<th>
									{family === 'client'
										? i18nMsg('reconciliation-group-col-client', 'Client')
										: i18nMsg('reconciliation-group-col-supplier', 'Fournisseur')}
								</th>
								<th>{i18nMsg('reconciliation-group-col-due', 'Reste dû')}</th>
								<th>{i18nMsg('reconciliation-group-remove', 'Retirer')}</th>
							</tr>
						</thead>
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
				<dl class="grid grid-cols-[1fr_auto] gap-x-4 gap-y-1" aria-live="polite">
					<dt>{i18nMsg('reconciliation-group-total-transaction', 'Montant du relevé')}</dt>
					<dd class="text-right" data-testid="group-transaction-amount">
						{decomposition ? formatUnits(decomposition.transaction) : proposal.transaction.amount}
					</dd>
					<dt>{i18nMsg('reconciliation-group-total-imputed', 'Total imputé')}</dt>
					<dd class="text-right" data-testid="group-total-imputed">{totalText}</dd>
					<dt>{i18nMsg('reconciliation-group-gap-label', 'Écart')}</dt>
					<dd
						class="text-right font-semibold {tooLarge && enoughInvoices ? 'text-destructive' : ''}"
						data-testid="group-gap"
						data-gap-sense={enoughInvoices ? (sense ?? 'none') : 'none'}
					>
						{gapText}
					</dd>
				</dl>

				{#if decomposition && enoughInvoices && decomposition.rounding !== 0n}
					<!-- D2 bis — l'arrondi est une information, avec ou sans écart. -->
					<p class="mt-2 text-text-muted" data-testid="group-rounding">
						{i18nMsg(
							'reconciliation-group-rounding-info',
							'Arrondi de { $amount } passé au compte d’arrondi',
							{ amount: formatSignedUnits(decomposition.rounding) },
						)}
					</p>
					{#if blocks.includes('rounding-not-configured')}
						<p
							class="mt-1 rounded-md border border-warning bg-warning/10 px-3 py-2"
							role="status"
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
					aria-live="assertive"
					data-testid="group-refusals"
				>
					{#each refusals as r, i (i)}
						<li data-testid="group-refusal">{r}</li>
					{/each}
				</ul>
			{/if}

			{#if notRereadTexts.length > 0}
				<!-- Revue P2 (L-1) : reste affiché tant que la facture n'est pas relue. -->
				<ul
					class="mt-2 list-disc rounded-md border border-warning bg-warning/10 py-2 pr-3 pl-8 text-sm"
					role="status"
					data-testid="group-not-reread"
				>
					{#each notRereadTexts as t, i (i)}
						<li data-testid="group-not-reread-item">{t}</li>
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

			{#if blockTexts.length > 0}
				<!-- Une phrase par cause qui retient Enregistrer (revue P1, B-L4, E-L6). -->
				<div class="mt-2 text-sm text-text-muted" id="group-block-hint" data-testid="group-block-hint">
					{#each blockTexts as t, i (i)}
						<p data-testid="group-block-reason">{t}</p>
					{/each}
				</div>
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
			<Button
				onclick={submit}
				disabled={submitting || blocks.length > 0}
				aria-describedby={blockTexts.length > 0 ? 'group-block-hint' : undefined}
				data-testid="group-submit"
			>
				{i18nMsg('reconciliation-group-submit', 'Enregistrer')}
			</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>
