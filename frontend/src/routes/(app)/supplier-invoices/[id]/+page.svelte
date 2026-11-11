<script lang="ts">
	/**
	 * La fiche d'une facture fournisseur.
	 *
	 * Story v017-4b (#849, AC-22) — les actions suivent le cycle brouillon → validée →
	 * payée :
	 * - **brouillon** : *Modifier* (page d'édition), *Valider* (l'écriture d'achat est
	 *   passée), *Supprimer le brouillon* (Administrateur seul) ; ni *Payer* ni *Annuler
	 *   la facture* — rien n'est au journal ;
	 * - **validée** (`open`) : *Repasser en brouillon* (l'écriture d'achat est supprimée, la
	 *   facture redevient modifiable), plus *Payer* et *Annuler* comme avant ;
	 * - **payée**, **annulée** : Story v0172-annulees (#868, AC-12) — *Repasser en brouillon*
	 *   s'y affiche aussi. Annulée : une confirmation propre (achat et contre-passation
	 *   supprimés, deux trous, totaux de deux périodes). Payée : appel direct, sans boîte —
	 *   le serveur refuse au rang 1, avant toute écriture, et le refus dit quoi faire d'abord
	 *   (annuler le règlement, ou le rapprochement).
	 * Un lien « Voir l'écriture d'achat » s'affiche dès qu'il y en a une. Les refus de la
	 * validation et de la suppression s'affichent sous leur bouton par le message du
	 * serveur ; ceux du repassage en brouillon par `supplierInvoiceUnvalidateMessage`
	 * (AC-25 ; v0172-annulees, AC-8).
	 */
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { Button } from '$lib/components/ui/button';
	import * as Dialog from '$lib/components/ui/dialog';
	import {
		cancelSupplierInvoice,
		cancelSupplierInvoiceSettlement,
		deleteSupplierInvoice,
		getSupplierInvoice,
		paySupplierInvoice,
		unvalidateSupplierInvoice,
		validateSupplierInvoice,
	} from '$lib/features/supplier-invoices/supplier-invoices.api';
	import { supplierSettlementCancelMessage } from '$lib/features/supplier-invoices/settlement-cancel';
	import { todayLocal } from '$lib/shared/utils/local-date';
	import { supplierInvoiceCancelMessage } from '$lib/features/supplier-invoices/invoice-cancel';
	import {
		supplierInvoiceUnvalidateMessage,
		type SupplierInvoiceUnvalidateDetails,
	} from '$lib/features/supplier-invoices/supplier-invoice-unvalidate';
	import { authState } from '$lib/app/stores/auth.svelte';
	import { supplierInvoiceStatusLabel } from '$lib/features/supplier-invoices/supplier-invoice-helpers';
	// Story v017-3b (#687, C-v017-3-27) — la fiche est la seule page aux décimales
	// exactes : TTC, prix unitaire et total de ligne, total du sélecteur de document.
	import { formatExactAmount } from '$lib/features/invoices/invoice-helpers';
	import SupplierInvoiceTotal from '$lib/features/supplier-invoices/SupplierInvoiceTotal.svelte';
	import type { SupplierInvoiceResponse } from '$lib/features/supplier-invoices/supplier-invoices.types';
	import { listBankAccounts } from '$lib/features/bank-accounts/bank-accounts.api';
	import type { BankAccountSummary } from '$lib/features/bank-accounts/bank-accounts.api';
	import { bankAccountOptionLabel } from '$lib/features/bank-accounts/format';
	import { fetchAccounts } from '$lib/features/accounts/accounts.api';
	import type { AccountResponse } from '$lib/features/accounts/accounts.types';
	import {
		accountIdsWithRole,
		bankAccountsNotLinkedTo,
		withoutAccountIds,
	} from '$lib/features/accounts/account-options';
	import { listProjects } from '$lib/features/projects/projects.api';
	import type { ProjectResponse } from '$lib/features/projects/projects.types';
	import { isApiError } from '$lib/shared/utils/api-client';
	import { isSettlementOutcomeUnknown } from '$lib/shared/utils/unknown-outcome';
	import type { ApiError } from '$lib/shared/types/api';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { notifyInfo, notifyWarning, notifyError, notifySuccess } from '$lib/shared/utils/notify';
	import {
		attachSourceDocument,
		detachSourceDocument,
		downloadSupplierInvoiceSourceDocument,
		fetchSupplierInvoiceSourceDocument,
	} from '$lib/features/imported-supplier-invoices/imported-supplier-invoices.api';
	import DocumentPreview from '$lib/features/imported-supplier-invoices/DocumentPreview.svelte';
	import AttachDocumentPicker from '$lib/features/imported-supplier-invoices/AttachDocumentPicker.svelte';
	import { isStaleDocumentRefusal } from '$lib/features/imported-supplier-invoices/stale-refusal';
	import { formatSwissDate } from '$lib/features/reports/reports.api';
	import { readableAmount } from '$lib/features/supplier-invoices/supplier-invoice-form';
	import { supplierInvoiceShortLabel } from '$lib/features/supplier-invoices/supplier-invoice-label';

	const id = Number(page.params.id);

	let invoice = $state<SupplierInvoiceResponse | null>(null);
	let loading = $state(true);
	let errorMsg = $state('');

	let bankAccounts = $state<BankAccountSummary[]>([]);
	let accounts = $state<AccountResponse[]>([]);
	/** Story 15-6b : tous les comptes bancaires liés ont été écartés (liés au compte créanciers). */
	let noEligibleBankAccount = $state(false);
	let projects = $state<ProjectResponse[]>([]);

	/** Libellé du projet analytique affecté (Story 19-3), ou `null`. */
	const projectLabel = $derived.by(() => {
		const pid = invoice?.projectId;
		if (pid == null) return null;
		const p = projects.find((x) => x.id === pid);
		return p ? `${p.code} — ${p.name}` : `#${pid}`;
	});

	// Formulaire de règlement binaire.
	let settlementType = $state<'bank_transfer' | 'internal_account'>('bank_transfer');
	// #889 : la date du jour du poste, non celle d'UTC (la veille entre minuit et 1 h).
	let paymentDate = $state(todayLocal());
	let payBankAccountId = $state<number | null>(null);
	let payAccountId = $state<number | null>(null);
	let paying = $state(false);
	let payError = $state('');

	// Story 25-3-a-2 (#414) — annuler le règlement.
	let canManage = $derived(
		authState.currentUser?.role === 'Admin' || authState.currentUser?.role === 'Comptable',
	);
	let cancellingSettlement = $state(false);
	// ⛔ Le refus s'affiche ICI, jamais dans `errorMsg` : celui-ci remplace toute
	// la fiche.
	let settlementCancelError = $state('');

	// Story 25-3-c (#454) — annuler la facture, même payée. ⛔ Même règle : le
	// refus s'affiche localement, jamais dans `errorMsg`.
	let cancelling = $state(false);
	let cancelError = $state('');

	// Story v017-4b (#849, AC-22) — valider, repasser en brouillon, supprimer le brouillon.
	// Chaque geste a sa confirmation (dialogue ; sauf repasser une payée en brouillon,
	// v0172-annulees) et son refus, affiché sous son bouton, jamais
	// dans `errorMsg`.
	const isAdmin = $derived(authState.currentUser?.role === 'Admin');
	let validateOpen = $state(false);
	let validating = $state(false);
	let validateError = $state('');
	let unvalidateOpen = $state(false);
	let unvalidating = $state(false);
	let unvalidateError = $state('');
	let deleteOpen = $state(false);
	let deleting = $state(false);
	let deleteError = $state('');

	// Story 30-2c (AC18, AC20, AC24) — le document de la facture : aperçu, sélecteur
	// pour en joindre un, détachement. ⛔ Le refus du détachement s'affiche ici,
	// jamais dans `errorMsg`.
	let showPreview = $state(false);
	let showPicker = $state(false);
	let detaching = $state(false);
	let detachError = $state('');
	/** Lecture STABLE du document pour l'aperçu (une closure par rendu relirait). */
	const loadSourceDocument = () => fetchSupplierInvoiceSourceDocument(id);

	/**
	 * Story 34-8 (#938, AC-8, C-e343-12) — l'issue du dernier geste de la fiche :
	 * - `known` : connue (succès, ou refus affiché sous son bouton) ;
	 * - `verify` : la réponse s'est perdue (`isSettlementOutcomeUnknown`), la facture a
	 *   été relue ; l'encadré dit de la vérifier avant de réessayer ;
	 * - `reread-failed` : la réponse s'est perdue et la relecture a échoué ; l'encadré
	 *   dit de recharger la page. L'écran n'affirme jamais une relecture qui n'a pas eu
	 *   lieu (le défaut que la revue P1 de la 33-6 a corrigé côté client).
	 *
	 * Pas d'état bloquant (contrairement au règlement client) : chacun des sept gestes
	 * est gardé par le statut, la version ou l'état du document (joindre, détacher), si
	 * bien qu'un second envoi est refusé sans rien écrire deux fois — et ce refus relit la
	 * facture (`reloadAfterStaleRefusal`, `isStaleDocumentRefusal`). L'encadré disparaît
	 * au geste suivant (`clearCycleErrors`) ou à une relecture réussie lancée par la page.
	 */
	let cycleOutcome = $state<'known' | 'verify' | 'reread-failed'>('known');

	/** Le texte de l'encadré d'issue inconnue (AC-8, deux avis selon la relecture). */
	function unknownOutcomeText(kind: 'verify' | 'reread-failed'): string {
		return kind === 'verify'
			? i18nMsg(
					'supplier-invoices-unknown-outcome',
					'L’issue n’est pas connue : la facture a été relue, vérifiez-la avant de réessayer.',
				)
			: i18nMsg(
					'supplier-invoices-unknown-outcome-reread-failed',
					'L’issue n’est pas connue, et la facture n’a pas pu être relue : rechargez la page avant de réessayer.',
				);
	}

	/**
	 * Story 34-8 (AC-8) — après un échec dont l'issue n'est pas connue (réponse perdue,
	 * 5xx, erreur hors `ApiError` : le serveur a pu écrire), relit la facture et pose
	 * l'avis. Rend `true` si c'était le cas : l'appelant n'affiche alors pas son propre
	 * message, qui dirait un échec. Un 4xx, refus certain, rend `false`.
	 */
	async function rereadAfterUnknownOutcome(err: unknown): Promise<boolean> {
		if (!isSettlementOutcomeUnknown(err)) return false;
		cycleOutcome = (await reloadAfterRefusal()) ? 'verify' : 'reread-failed';
		return true;
	}

	/**
	 * Story 30-2c (AC25) — `paidAt` est un horodatage naïf (« 2026-06-20T00:00:00 ») :
	 * la fiche n'en montre que la date, au format suisse. Sans date, la phrase sans date
	 * (deux clés : une seule à variable rendrait « Facture réglée le . »).
	 */
	function paidInfo(paidAt: string | null): string {
		if (!paidAt) return i18nMsg('supplier-invoices-paid-info', 'Facture réglée.');
		return i18nMsg('supplier-invoices-paid-info-on', 'Facture réglée le { $date }.', {
			date: formatSwissDate(paidAt.slice(0, 10)),
		});
	}

	async function load() {
		invoice = await getSupplierInvoice(id);
	}

	onMount(async () => {
		try {
			const [, banks, accts, projs] = await Promise.all([
				load(),
				listBankAccounts(),
				fetchAccounts(),
				listProjects(true), // inclut les archivés pour résoudre le nom d'un projet tagué puis archivé
			]);
			// Story 15-6b (#474, AC8) — le compte créanciers n'est jamais une
			// contrepartie : `D 2000 / C 2000` s'équilibre et la facture passerait
			// « payée » sans paiement. Les ids se calculent sur la liste REÇUE,
			// avant le filtre `active && postable` ; la garde serveur reste
			// l'autorité.
			const payableIds = accountIdsWithRole(accts, 'Payable');
			const linked = banks.filter((b) => b.journalAccountId !== null);
			bankAccounts = bankAccountsNotLinkedTo(linked, payableIds);
			// Le message ne vaut que si le filtre a ÉCARTÉ un compte : une société
			// sans compte bancaire lié garde son comportement.
			noEligibleBankAccount = bankAccounts.length === 0 && linked.length > bankAccounts.length;
			accounts = withoutAccountIds(
				accts.filter((a) => a.active && a.postable), // 14-3b : compte posté
				payableIds,
			);
			projects = projs;
		} catch (err) {
			if (isApiError(err)) errorMsg = err.message;
		} finally {
			loading = false;
		}
	});

	async function pay() {
		payError = '';
		// Revue P2 (B2-2 = A2-2 = E2-1) : un geste qui change le statut périme les refus du
		// cycle, rendus hors des blocs de statut.
		clearCycleErrors();
		if (settlementType === 'bank_transfer' && !payBankAccountId) {
			payError = i18nMsg('supplier-invoices-pay-err-bank', 'Sélectionnez un compte bancaire.');
			return;
		}
		if (settlementType === 'internal_account' && !payAccountId) {
			payError = i18nMsg('supplier-invoices-pay-err-account', 'Sélectionnez un compte.');
			return;
		}
		paying = true;
		try {
			invoice = await paySupplierInvoice(id, {
				settlementType,
				bankAccountId:
					settlementType === 'bank_transfer' ? (payBankAccountId ?? undefined) : undefined,
				accountId: settlementType === 'internal_account' ? (payAccountId ?? undefined) : undefined,
				paymentDate,
			});
		} catch (err) {
			// Story 34-8 (AC-8) : une réponse perdue n'est plus muette — relecture et avis.
			// Ce qui reste est une `ApiError` : `isSettlementOutcomeUnknown` rend vrai pour
			// toute autre erreur (revue P1 de la 34-8, B-2 : le repli était inatteignable).
			if (await rereadAfterUnknownOutcome(err)) return;
			if (isApiError(err)) payError = err.message;
			// Revue P2 de la 34-8 (E-1) : un refus qui dit l'écran périmé — la facture payée
			// par un envoi précédent dont la réponse s'était perdue — relit la facture.
			await reloadAfterStaleRefusal(err);
		} finally {
			paying = false;
		}
	}

	/**
	 * Annule le règlement (Story 25-3-a-2). La confirmation dit ce qui va
	 * s'écrire et, si la facture figure dans un lot confirmé, prévient du
	 * double paiement — un fait HISTORIQUE, jamais « payée par ce lot ».
	 */
	async function cancelSettlement() {
		if (!invoice) return;
		settlementCancelError = '';
		let message = i18nMsg(
			'supplier-invoices-settlement-cancel-confirm',
			"Annuler ce règlement ? Une écriture inverse datée d'aujourd'hui sera passée au grand livre, et la facture redeviendra validée, à payer.",
		);
		const batch = invoice.lastConfirmedBatch;
		if (batch) {
			message +=
				'\n\n' +
				i18nMsg(
					'supplier-invoices-settlement-cancel-batch-warning',
					'Cette facture figure dans le lot de paiement n° { $batch }, confirmé le { $date }. Si la banque a exécuté cet ordre, elle a déjà été payée : corrigez alors par un nouveau règlement direct, jamais par un nouveau lot.',
					// Revue P2 (B2-4) : la date au format suisse, comme l'avertissement de la
					// dévalidation.
					// #889 : un instant (CURRENT_TIMESTAMP, UTC) — sa date suisse, horodatage entier.
					{ batch: batch.id, date: formatSwissDate(batch.confirmedAt ?? '') },
				);
		}
		if (!confirm(message)) return;
		clearCycleErrors(); // revue P2 (B2-2 = A2-2 = E2-1)
		cancellingSettlement = true;
		try {
			const res = await cancelSupplierInvoiceSettlement(id);
			invoice = res.invoice;
			notifySuccess(
				i18nMsg(
					'supplier-invoices-settlement-cancelled',
					"Règlement annulé : l'écriture inverse a été passée et la facture est de nouveau validée, à payer.",
				),
			);
		} catch (err) {
			// Story 34-8 (AC-8) ; ce qui reste est une `ApiError` (revue P2, B-2 = A-2).
			if (await rereadAfterUnknownOutcome(err)) return;
			if (isApiError(err)) settlementCancelError = err.message;
			await reloadAfterStaleRefusal(err); // revue P2 (E-1)
		} finally {
			cancellingSettlement = false;
		}
	}

	/**
	 * Annule la facture (Story 25-3-c), validée ou payée. La confirmation dit
	 * ce qui va s'écrire ; payée, elle dit que le règlement reste au grand
	 * livre, détaché — un paiement sans facture.
	 */
	async function cancel() {
		if (!invoice) return;
		cancelError = '';
		let message = i18nMsg(
			'supplier-invoices-cancel-confirm',
			"Annuler cette facture ? Une écriture inverse de l'achat, datée d'aujourd'hui, sera passée au grand livre.",
		);
		if (invoice.status === 'paid') {
			message +=
				'\n\n' +
				i18nMsg(
					'supplier-invoices-cancel-confirm-paid',
					"Elle est payée : son règlement reste au grand livre, détaché de la facture — un paiement sans facture, à rattacher. Si c'est le paiement lui-même qui est erroné, annulez plutôt le règlement d'abord.",
				);
		}
		if (!confirm(message)) return;
		clearCycleErrors(); // revue P2 (B2-2 = A2-2 = E2-1)
		cancelling = true;
		try {
			invoice = await cancelSupplierInvoice(id);
			notifySuccess(
				i18nMsg(
					'supplier-invoices-cancelled',
					"Facture annulée : l'écriture inverse de l'achat a été passée au grand livre.",
				),
			);
		} catch (err) {
			// Story 34-8 (AC-8) ; ce qui reste est une `ApiError` (revue P2, B-2 = A-2).
			if (await rereadAfterUnknownOutcome(err)) return;
			if (isApiError(err)) cancelError = err.message;
			await reloadAfterStaleRefusal(err); // revue P2 (E-1)
		} finally {
			cancelling = false;
		}
	}

	/** Un refus qui dit l'écran périmé : la facture a changé depuis sa lecture. */
	function isStaleState(err: unknown): boolean {
		// Revue P1 (B-3, E-3) : `SUPPLIER_INVOICE_PAID` (dévalider une facture payée
		// entre-temps) et `SUPPLIER_INVOICE_MUST_BE_UNVALIDATED_FIRST` (supprimer un
		// « brouillon » validé entre-temps) disent eux aussi l'écran périmé. Story
		// v0172-annulees (AC-12) : les deux refus neufs relisent la facture comme
		// `SUPPLIER_INVOICE_PAID`. Revue P2 de la 34-8 (E-1) : les refus de statut des deux
		// annulations — règlement d'une facture qui n'est plus payée, facture déjà annulée ou
		// repassée en brouillon — disent eux aussi l'écran périmé.
		return (
			isApiError(err) &&
			(err.code === 'ILLEGAL_STATE_TRANSITION' ||
				err.code === 'OPTIMISTIC_LOCK_CONFLICT' ||
				err.code === 'SUPPLIER_INVOICE_PAID' ||
				err.code === 'SUPPLIER_INVOICE_PAID_RECONCILED' ||
				err.code === 'SUPPLIER_INVOICE_DETACHED_PAYMENT' ||
				err.code === 'SUPPLIER_INVOICE_MUST_BE_UNVALIDATED_FIRST' ||
				err.code === 'SUPPLIER_INVOICE_NOT_PAID' ||
				err.code === 'SUPPLIER_INVOICE_CANCELLED' ||
				err.code === 'SUPPLIER_INVOICE_DRAFT')
		);
	}

	/**
	 * Revue P2 de la 34-8 (E-1, C-e348-7) — relit la facture après un refus qui dit l'écran
	 * périmé. C'est le second clic après une réponse perdue puis une relecture en échec : le
	 * geste est écrit, le serveur refuse le second envoi, et l'écran montrait encore l'état
	 * d'avant. Si cette relecture échoue aussi, l'encadré `reread-failed` revient : l'écran
	 * n'affiche pas un refus sur une facture qu'il n'a pas pu relire sans le dire.
	 */
	async function reloadAfterStaleRefusal(err: unknown) {
		if (!isStaleState(err)) return;
		if (!(await reloadAfterRefusal())) cycleOutcome = 'reread-failed';
	}

	/**
	 * Efface les refus des gestes du cycle (revue P1, B-2) : au début de **tout geste qui
	 * change le statut** — valider, repasser en brouillon, supprimer, et (revue P2, B2-2 = A2-2 = E2-1)
	 * payer, annuler le règlement, annuler la facture. Les refus sont rendus hors des blocs
	 * de statut : rien d'autre ne les masque. Story 34-8 (AC-8) : l'encadré d'issue
	 * inconnue part avec eux.
	 */
	function clearCycleErrors() {
		validateError = '';
		unvalidateError = '';
		deleteError = '';
		cycleOutcome = 'known';
	}

	/**
	 * Revue P1 (E-2 = A-1, limite L4), reprise en revue P2 (B2-1 = A2-1) — le lot confirmé le
	 * plus récent qui contient la facture : un fait HISTORIQUE, jamais « payée par ce lot »
	 * (`payment_batches::last_confirmed_batch_for_invoice`). Le fait se dit à tous les rôles ;
	 * sans date de confirmation, la phrase sans date (B2-3).
	 */
	function confirmedBatchFact(batch: { id: number; confirmedAt: string | null }): string {
		if (!batch.confirmedAt) {
			return i18nMsg(
				'supplier-invoices-unvalidate-batch-warning-undated',
				'Cette facture figure dans le lot de paiement confirmé n° { $batch }.',
				{ batch: batch.id },
			);
		}
		return i18nMsg(
			'supplier-invoices-unvalidate-batch-warning',
			'Cette facture figure dans le lot de paiement n° { $batch }, confirmé le { $date }.',
			{ batch: batch.id, date: formatSwissDate(batch.confirmedAt) },
		);
	}

	/** La consigne, pour les seuls rôles qui peuvent repasser en brouillon (revue P2, B2-5). */
	function confirmedBatchAction(): string {
		return i18nMsg(
			'supplier-invoices-unvalidate-batch-warning-action',
			'Si la banque a exécuté cet ordre, la facture a peut-être déjà été payée : vérifiez le relevé avant de changer son montant.',
		);
	}

	/**
	 * Valide le brouillon (Story v017-4b, AC-22) : l'écriture d'achat est passée. La
	 * réponse est la facture telle que la validation l'a écrite ; un refus s'affiche sous le
	 * bouton, par le message du
	 * serveur (patron de la fiche client) ; un refus d'écran périmé relit la facture.
	 */
	async function confirmValidate() {
		if (!invoice || validating) return;
		validating = true;
		clearCycleErrors();
		try {
			invoice = await validateSupplierInvoice(invoice.id, invoice.version);
			validateOpen = false;
			notifySuccess(
				i18nMsg(
					'supplier-invoices-validated',
					"Facture validée : l'écriture d'achat a été passée au journal.",
				),
			);
		} catch (err) {
			validateOpen = false;
			// Story 34-8 (AC-8) ; ce qui reste est une `ApiError` (revue P2, B-2 = A-2).
			if (await rereadAfterUnknownOutcome(err)) return;
			if (isApiError(err)) validateError = err.message;
			await reloadAfterStaleRefusal(err); // revue P2 (E-1)
		} finally {
			validating = false;
		}
	}

	/**
	 * Le clic sur *Repasser en brouillon* (Story v0172-annulees, AC-12, C-v0172a-21) :
	 * validée ou annulée, la confirmation s'ouvre ; **payée**, l'appel part directement,
	 * sans boîte — aucune suppression ne peut suivre (le serveur refuse le statut `paid` au
	 * rang 1 ; une facture redevenue validée ailleurs a changé de version et rend
	 * `OPTIMISTIC_LOCK_CONFLICT`), et le refus dit quoi faire d'abord.
	 */
	function requestUnvalidate() {
		if (invoice?.status === 'paid') {
			void confirmUnvalidate();
		} else {
			unvalidateOpen = true;
		}
	}

	/**
	 * Repasse la facture en brouillon (Story v017-4b, AC-22 ; v0172-annulees, AC-12) : son
	 * écriture d'achat — et, annulée, sa contre-passation — est supprimée, elle redevient un
	 * brouillon modifiable. Un refus dit son motif et nomme l'objet à défaire
	 * (`supplierInvoiceUnvalidateMessage`, AC-25, AC-8), à partir de `details`.
	 */
	async function confirmUnvalidate() {
		if (!invoice || unvalidating) return;
		unvalidating = true;
		clearCycleErrors();
		const wasCancelled = invoice.status === 'cancelled';
		try {
			invoice = await unvalidateSupplierInvoice(invoice.id, invoice.version);
			unvalidateOpen = false;
			notifySuccess(
				wasCancelled
					? i18nMsg(
							'supplier-invoices-unvalidated-cancelled',
							"Facture repassée en brouillon : son écriture d'achat et sa contre-passation ont été supprimées.",
						)
					: i18nMsg(
							'supplier-invoices-unvalidated',
							"Facture repassée en brouillon : son écriture d'achat a été supprimée.",
						),
			);
		} catch (err) {
			unvalidateOpen = false;
			// Story 34-8 (AC-8) ; ce qui reste est une `ApiError` (revue P2, B-2 = A-2).
			if (await rereadAfterUnknownOutcome(err)) return;
			if (isApiError(err)) {
				// `details` : le lot, la transaction, l'écriture de paiement (AC-8) ; ou,
				// revue P1 (E-4), le nom de l'exercice postérieur clos.
				unvalidateError = supplierInvoiceUnvalidateMessage(
					err.code,
					(err.details as SupplierInvoiceUnvalidateDetails | undefined) ?? null,
				);
			}
			await reloadAfterStaleRefusal(err); // revue P2 (E-1)
		} finally {
			unvalidating = false;
		}
	}

	/**
	 * Supprime le brouillon (Administrateur ; Story v017-4b, AC-22), puis revient à la
	 * liste. Son document éventuel redevient « à traiter » (serveur).
	 */
	async function confirmDelete() {
		if (!invoice || deleting) return;
		deleting = true;
		clearCycleErrors();
		try {
			await deleteSupplierInvoice(invoice.id);
			deleteOpen = false;
			notifySuccess(i18nMsg('supplier-invoices-deleted', 'Brouillon supprimé.'));
			await goto('/supplier-invoices');
		} catch (err) {
			deleteOpen = false;
			deleteError = isApiError(err) ? err.message : i18nMsg('common-error', 'Erreur inattendue');
			if (isStaleState(err)) await reloadAfterRefusal();
		} finally {
			deleting = false;
		}
	}

	/**
	 * Relit la facture après un refus qui dit l'écran périmé (revue P1, B2 = E5) ;
	 * un échec de cette relecture laisse l'écran tel quel — le refus, lui, est
	 * affiché.
	 *
	 * Story 34-8 (AC-8, validation P1, 34-8-F2) : rend `true` si la relecture a
	 * réussi, `false` sinon — l'avis d'issue inconnue en dépend. Une relecture
	 * réussie retire l'encadré d'une issue inconnue précédente : la fiche montre
	 * désormais l'état écrit.
	 */
	async function reloadAfterRefusal(): Promise<boolean> {
		try {
			// Revue P2 de la 34-8 (E-2, C-e348-8) : sans rejeu. Un `GET` coupé se rejouait
			// ~14 s, pendant lesquels les autres gestes restaient offerts sur l'écran d'avant ;
			// un échec est ici déjà traité (`reread-failed`).
			invoice = await getSupplierInvoice(id, { retry: false });
			cycleOutcome = 'known';
			return true;
		} catch {
			// L'écran reste celui d'avant ; le message du refus suffit.
			return false;
		}
	}

	/**
	 * Joint un document à traiter (Story 30-2c, AC19-AC20) ; la réponse est la
	 * facture telle que le geste l'a écrite, `sourceDocument` calculé. Une erreur remonte
	 * au sélecteur, qui
	 * l'affiche sans se fermer. Sur un refus `404`/`409`, la facture est relue
	 * d'abord : si elle a désormais un document (joint ailleurs), le sélecteur se
	 * ferme sur elle et le refus se dit par une notification.
	 *
	 * Story 34-8 (AC-8, C-e348-2) — une issue inconnue relit la facture :
	 * - elle montre un document : le sélecteur se ferme, l'encadré `verify` s'affiche ;
	 * - elle n'en montre pas : l'erreur remonte au sélecteur avec le texte de l'issue
	 *   inconnue, le document reste sélectionnable, et l'encadré ne s'affiche pas ;
	 * - la relecture échoue : le sélecteur se ferme (sa liste n'est plus sûre) et
	 *   l'encadré `reread-failed` dit de recharger la page.
	 */
	async function attach(importedSupplierInvoiceId: number) {
		cycleOutcome = 'known';
		try {
			invoice = await attachSourceDocument(id, importedSupplierInvoiceId);
		} catch (err) {
			if (isSettlementOutcomeUnknown(err)) {
				if (!(await reloadAfterRefusal())) {
					showPicker = false;
					cycleOutcome = 'reread-failed';
					return;
				}
				if (invoice?.sourceDocument) {
					showPicker = false;
					showPreview = false;
					detachError = '';
					cycleOutcome = 'verify';
					return;
				}
				const unknown: ApiError = {
					code: isApiError(err) ? err.code : 'UNKNOWN_OUTCOME',
					status: isApiError(err) ? err.status : 0,
					message: unknownOutcomeText('verify'),
				};
				throw unknown;
			}
			if (isStaleDocumentRefusal(err)) {
				await reloadAfterRefusal();
				if (invoice?.sourceDocument) {
					showPicker = false;
					// La facture relue montre un document : un message de
					// détachement antérieur serait faux sous lui (revue P2, T1).
					detachError = '';
					// Revue P2 de la 34-8 (B-4) : si c'est CE document, le refus vient d'un
					// envoi précédent dont la réponse s'était perdue — le rattachement est
					// fait, et le dire « déjà traité » annoncerait un échec.
					if (invoice.sourceDocument.importedSupplierInvoiceId === importedSupplierInvoiceId) {
						notifySuccess(
							i18nMsg('supplier-invoices-doc-attached', 'Document joint à la facture.'),
						);
					} else {
						notifyWarning(err.message);
					}
					return;
				}
			}
			throw err;
		}
		showPicker = false;
		showPreview = false;
		detachError = '';
		notifySuccess(i18nMsg('supplier-invoices-doc-attached', 'Document joint à la facture.'));
	}

	/**
	 * Détache le document (Story 30-2c, AC24) : il revient dans « Documents à
	 * traiter » ; la facture et ses écritures ne changent pas. Un refus s'affiche
	 * sous la fiche (`detachError`), jamais dans `errorMsg` ; un refus `404`/`409`
	 * (document déjà détaché ailleurs) fait relire la facture, qui propose alors
	 * « Joindre un document » (revue P1, B2 = E5).
	 */
	async function detach() {
		// Garde du double clic : sous jsdom, un second clic est parti malgré
		// `disabled={detaching}` (revue P1) ; la garde ne dépend pas du rendu.
		if (detaching) return;
		detachError = '';
		cycleOutcome = 'known';
		if (
			!confirm(
				i18nMsg(
					'supplier-invoices-doc-detach-confirm',
					'Détacher ce document ? Il reviendra dans les documents à traiter ; la facture et son écriture ne changent pas.',
				),
			)
		)
			return;
		detaching = true;
		try {
			invoice = await detachSourceDocument(id);
			showPreview = false;
			notifySuccess(
				i18nMsg(
					'supplier-invoices-doc-detached',
					'Document détaché : il est de nouveau dans les documents à traiter.',
				),
			);
		} catch (err) {
			// Story 34-8 (AC-8) : une réponse perdue relit la facture, sans « Erreur
			// inattendue » ; l'aperçu se ferme, la facture relue dit si le document est parti.
			if (isSettlementOutcomeUnknown(err)) {
				showPreview = false;
				await rereadAfterUnknownOutcome(err);
				return;
			}
			// Ce qui reste est une `ApiError` (revue P2 de la 34-8, B-2 = A-2).
			if (isApiError(err)) detachError = err.message;
			if (isStaleDocumentRefusal(err)) {
				showPreview = false;
				await reloadAfterRefusal();
			}
		} finally {
			detaching = false;
		}
	}

	/**
	 * Télécharge le document de la facture (Story 12.5d, DC-d1). Depuis la Story
	 * 30-2c, le bouton n'est rendu que si la facture a un document ; un 404 reste
	 * possible (document détaché entre-temps, ailleurs) et se dit sans bloquer.
	 */
	async function viewSourceDocument() {
		try {
			await downloadSupplierInvoiceSourceDocument(id);
		} catch (err) {
			if (isApiError(err) && err.code === 'SOURCE_DOCUMENT_NOT_FOUND') {
				notifyInfo(
					i18nMsg('imported-supplier-invoices-no-source-doc', 'Cette facture n’a pas de document.'),
				);
			} else if (isApiError(err) && err.code === 'SOURCE_DOCUMENT_GONE') {
				notifyWarning(i18nMsg('imported-supplier-invoices-source-doc-gone', 'Le document n’a pas été restauré.'));
			} else {
				notifyError(
					(isApiError(err) && err.message) ||
						i18nMsg('imported-supplier-invoices-source-doc-failed', 'Téléchargement impossible.'),
				);
			}
		}
	}
</script>

<svelte:head>
	<title>{i18nMsg('supplier-invoices-detail-title', 'Facture fournisseur')} — Kesh</title>
</svelte:head>

<button class="mb-4 text-sm text-primary" onclick={() => goto('/supplier-invoices')}>
	← {i18nMsg('common-back', 'Retour')}
</button>

{#if loading}
	<p class="text-sm text-text-muted">{i18nMsg('common-loading', 'Chargement…')}</p>
{:else if errorMsg}
	<p class="text-sm text-destructive">{errorMsg}</p>
{:else if invoice}
	<div class="mb-6 flex items-center justify-between">
		<h1 class="text-2xl font-semibold">
			{supplierInvoiceShortLabel(invoice)}
		</h1>
		<span data-testid="supplier-invoice-status">{supplierInvoiceStatusLabel(invoice.status)}</span>
	</div>

	<dl class="mb-6 grid grid-cols-2 gap-2 text-sm">
		<dt class="text-text-muted">{i18nMsg('supplier-invoices-col-internal-number', 'N° interne')}</dt>
		<dd data-testid="supplier-invoice-detail-internal-number">
			{#if invoice.internalNumber}
				<span class="font-mono">{invoice.internalNumber}</span>
				{#if invoice.status === 'draft'}
					<span class="block text-xs text-text-muted" data-testid="supplier-invoice-detail-number-kept">
						{i18nMsg(
							'supplier-invoices-internal-number-kept',
							'Conservé si la facture est revalidée ; perdu si elle est supprimée.',
						)}
					</span>
				{/if}
			{:else}
				{i18nMsg('supplier-invoices-internal-number-pending', 'Attribué à la validation')}
			{/if}
		</dd>
		<dt class="text-text-muted">{i18nMsg('supplier-invoices-col-number', 'N° fournisseur')}</dt>
		<dd class="font-mono" data-testid="supplier-invoice-detail-supplier-number">
			{invoice.supplierInvoiceNumber ?? '—'}
		</dd>
		<dt class="text-text-muted">{i18nMsg('supplier-invoices-col-date', 'Date')}</dt>
		<!-- Story v0171 (#871) : dates en forme suisse. -->
		<dd data-testid="supplier-invoice-detail-date">{formatSwissDate(invoice.invoiceDate)}</dd>
		<dt class="text-text-muted">{i18nMsg('supplier-invoices-col-due', 'Échéance')}</dt>
		<dd data-testid="supplier-invoice-detail-due">
			{invoice.dueDate ? formatSwissDate(invoice.dueDate) : '—'}
		</dd>
		<dt class="text-text-muted">{i18nMsg('supplier-invoices-col-total', 'TTC')}</dt>
		<dd><SupplierInvoiceTotal totalAmount={invoice.totalAmount} status={invoice.status} /></dd>
		{#if projectLabel}
			<dt class="text-text-muted">{i18nMsg('supplier-invoices-detail-project', 'Projet analytique')}</dt>
			<dd data-testid="supplier-invoice-project">{projectLabel}</dd>
		{/if}
		{#if invoice.creditorIban}
			<dt class="text-text-muted">IBAN</dt>
			<dd class="font-mono">{invoice.creditorIban}</dd>
		{/if}
		<!-- Story 30-2c (ajout de l'orchestrateur, C-30-2-47) : le QR-IBAN d'une facture
		     à QR-IBAN s'affiche aussi — la fiche ne montrait que `creditorIban`. -->
		{#if invoice.creditorQrIban}
			<dt class="text-text-muted">QR-IBAN</dt>
			<dd class="font-mono" data-testid="supplier-invoice-qr-iban">{invoice.creditorQrIban}</dd>
		{/if}
		{#if invoice.paymentReference}
			<dt class="text-text-muted">{i18nMsg('supplier-invoices-detail-reference', 'Référence')}</dt>
			<dd class="font-mono">{invoice.paymentReference}</dd>
		{/if}
	</dl>

	<!-- Story v017-4b (#849, AC-22, validation P2, R14) : le lien vers l'écriture d'achat,
	     dès qu'il y en a une — jamais pour un brouillon, qui n'en a pas. -->
	{#if invoice.purchaseJournalEntryId != null}
		<p class="mb-6 text-sm">
			<a
				class="underline"
				href="/journal-entries/{invoice.purchaseJournalEntryId}"
				data-testid="supplier-invoice-purchase-entry"
			>
				{i18nMsg('supplier-invoices-purchase-entry-link', "Voir l'écriture d'achat")}
			</a>
		</p>
	{/if}

	<!-- Story v017-4b (#849, AC-22) : les gestes du brouillon. -->
	{#if invoice.status === 'draft'}
		<div class="mb-6 space-y-2" data-testid="supplier-invoice-draft-actions">
			<p class="text-sm text-text-muted" data-testid="supplier-invoice-draft-info">
				{i18nMsg(
					'supplier-invoices-draft-info',
					'Brouillon : rien n’est encore passé au journal. Corrigez-le si besoin, puis validez-le.',
				)}
			</p>
			{#if canManage}
				<div class="flex flex-wrap gap-2">
					<a
						class="rounded border px-4 py-2 text-sm text-primary"
						href="/supplier-invoices/{invoice.id}/edit"
						data-testid="supplier-invoice-edit"
					>
						{i18nMsg('supplier-invoices-edit', 'Modifier')}
					</a>
					<Button
						data-testid="supplier-invoice-validate"
						disabled={validating}
						onclick={() => (validateOpen = true)}
					>
						{i18nMsg('supplier-invoices-validate', 'Valider')}
					</Button>
					{#if isAdmin}
						<Button
							variant="destructive"
							data-testid="supplier-invoice-delete"
							disabled={deleting}
							onclick={() => (deleteOpen = true)}
						>
							{i18nMsg('supplier-invoices-delete-draft', 'Supprimer le brouillon')}
						</Button>
					{/if}
				</div>
			{/if}
		</div>
	{:else if (invoice.status === 'open' && (canManage || invoice.lastConfirmedBatch)) || ((invoice.status === 'paid' || invoice.status === 'cancelled') && canManage)}
		<!-- Story v017-4b (#849, AC-22) : repasser en brouillon — Administrateur et
		     Comptable. Story v0172-annulees (#868, AC-12, C-v0172a-8) : aussi sur une payée et
		     une annulée. L'écran ne rejoue pas les empêchements : le serveur refuse, la fiche
		     dit le motif et quoi faire d'abord.
		     Revue P1 (E-2 = A-1, limite L4), reprise en P2 (B2-1, B2-5) : une facture qui
		     figure dans un lot CONFIRMÉ repasse en brouillon ; la fiche nomme ce lot (un fait
		     historique, à tous les rôles) et, aux rôles qui écrivent, dit de vérifier le
		     relevé avant de changer le montant — risque de double paiement. Sans geste ni
		     lot, aucun bloc (P2, A2-4 = E2-3). -->
		<div class="mb-6 space-y-2" data-testid="supplier-invoice-unvalidate-block">
			{#if invoice.status === 'open' && invoice.lastConfirmedBatch}
				<p
					class="rounded-md border border-warning bg-warning/10 px-3 py-2 text-sm"
					role="note"
					data-testid="supplier-invoice-confirmed-batch-warning"
				>
					{confirmedBatchFact(invoice.lastConfirmedBatch)}
					{#if canManage}{confirmedBatchAction()}{/if}
				</p>
			{/if}
			{#if canManage}
				<Button
					variant="outline"
					data-testid="supplier-invoice-unvalidate"
					disabled={unvalidating}
					onclick={requestUnvalidate}
				>
					{i18nMsg('supplier-invoices-unvalidate', 'Repasser en brouillon')}
				</Button>
			{/if}
		</div>
	{/if}
	<!-- Revue P1 (E-1 = A-6) : les refus des gestes du cycle se rendent HORS des blocs de
	     statut — un refus d'écran périmé relit la facture, dont le statut a changé, et le
	     motif doit rester lisible quel que soit l'état relu. Chaque geste efface ceux d'avant
	     (B-2). -->
	{#if validateError}
		<p class="mb-6 text-sm text-destructive" role="alert" data-testid="supplier-invoice-validate-error">
			{validateError}
		</p>
	{/if}
	{#if deleteError}
		<p class="mb-6 text-sm text-destructive" role="alert" data-testid="supplier-invoice-delete-error">
			{deleteError}
		</p>
	{/if}
	{#if unvalidateError}
		<p class="mb-6 text-sm text-destructive" role="alert" data-testid="supplier-invoice-unvalidate-error">
			{unvalidateError}
		</p>
	{/if}
	<!-- Story 34-8 (#938, AC-8, C-e348-1) : l'issue inconnue d'un des sept gestes, en
	     encadré persistant — non une notification, qui passerait. Rendu ici, hors des blocs
	     de statut, comme les refus du cycle : la facture relue a souvent changé de statut. -->
	{#if cycleOutcome !== 'known'}
		<p
			class="mb-6 rounded-md border border-warning bg-warning/10 px-3 py-2 text-sm"
			role="status"
			data-testid="supplier-unknown-outcome"
			data-outcome={cycleOutcome}
		>
			{unknownOutcomeText(cycleOutcome)}
		</p>
	{/if}

	<!-- Story 30-2c (AC18) : « Voir le document » SEULEMENT s'il existe (plus de bouton
	     qui rend 404) ; sans document (`null`), « Joindre un document » pour un rôle
	     qui écrit. `undefined` (non calculé) n'affiche rien : ce n'est pas « sans ». -->
	{#if invoice.sourceDocument}
		<div class="mb-6 space-y-2" data-testid="supplier-invoice-document">
			<p class="text-sm">
				<span class="text-text-muted">{i18nMsg('supplier-invoices-doc-label', 'Document')} :</span>
				<span data-testid="supplier-invoice-document-name">{invoice.sourceDocument.originalFilename}</span>
			</p>
			<div class="flex flex-wrap gap-2">
				<button
					class="rounded border px-4 py-2 text-sm text-primary"
					data-testid="supplier-invoice-source-document"
					aria-expanded={showPreview}
					onclick={() => (showPreview = !showPreview)}
				>
					{showPreview
						? i18nMsg('imported-supplier-invoices-preview-close', 'Fermer l’aperçu')
						: i18nMsg('imported-supplier-invoices-view-source', 'Voir le document')}
				</button>
				<button
					class="rounded border px-4 py-2 text-sm text-primary"
					data-testid="supplier-invoice-source-download"
					onclick={viewSourceDocument}
				>
					{i18nMsg('imported-supplier-invoices-view-doc', 'Télécharger')}
				</button>
				{#if canManage}
					<button
						class="rounded border px-4 py-2 text-sm text-destructive disabled:opacity-60"
						data-testid="supplier-invoice-detach"
						disabled={detaching}
						onclick={detach}
					>
						{i18nMsg('supplier-invoices-doc-detach', 'Détacher le document')}
					</button>
				{/if}
			</div>
			{#if showPreview}
				<div data-testid="supplier-invoice-preview-panel">
					<DocumentPreview
						load={loadSourceDocument}
						fileName={invoice.sourceDocument.originalFilename}
					/>
				</div>
			{/if}
		</div>
	{:else if invoice.sourceDocument === null && canManage}
		{#if showPicker}
			<AttachDocumentPicker
				invoiceTotal={invoice.totalAmount}
				onAttach={attach}
				onClose={() => (showPicker = false)}
			/>
		{:else}
			<button
				class="mb-6 inline-block rounded border px-4 py-2 text-sm text-primary"
				data-testid="supplier-invoice-attach"
				onclick={() => (showPicker = true)}
			>
				{i18nMsg('supplier-invoices-doc-attach', 'Joindre un document')}
			</button>
		{/if}
	{/if}
	<!-- Hors du bloc du document : après un refus `404` (document détaché ailleurs), la
	     facture relue n'a plus de document, et le message doit rester lisible. -->
	{#if detachError}
		<p class="mb-6 text-sm text-destructive" role="alert" data-testid="supplier-invoice-detach-error">
			{detachError}
		</p>
	{/if}

	<table class="mb-6 w-full border-collapse text-sm">
		<thead>
			<tr class="border-b text-left text-text-muted">
				<th class="py-2">{i18nMsg('supplier-invoices-line-desc', 'Description')}</th>
				<th class="py-2 text-right">{i18nMsg('supplier-invoices-col-qty', 'Qté')}</th>
				<th class="py-2 text-right">{i18nMsg('supplier-invoices-line-ht', 'HT')}</th>
				<th class="py-2 text-right">{i18nMsg('supplier-invoices-col-vat', 'TVA')}</th>
				<!-- ⚠️ PAS `supplier-invoices-col-total` : cette colonne affiche `line.lineTotal`, qui
					 vaut `quantity × unit_price` HORS TAXE (`supplier_invoice.rs:89`), tandis que la clé
					 `-col-total` sert `invoice.totalAmount`, documenté TTC (`supplier_invoice.rs:29`).
					 Les deux ont longtemps partagé une clé sans dommage, parce qu'elle manquait des
					 quatre catalogues et que chaque site retombait sur SON repli. Entrer une valeur
					 unique au catalogue l'aurait imposée aux trois sites — « TTC » au-dessus de
					 montants HT. (Story 23-3, révélé par le moissonneur.) -->
				<th class="py-2 text-right">{i18nMsg('supplier-invoices-line-total', 'Total HT')}</th>
			</tr>
		</thead>
		<tbody>
			{#each invoice.lines as line (line.position)}
				<tr class="border-b">
					<td class="py-2">{line.description}</td>
					<!-- Story v0171 (#866) : la quantité lisible, `1.0000` → `1`. -->
					<td class="py-2 text-right" data-testid="supplier-invoice-detail-line-quantity"
						>{readableAmount(line.quantity, 0)}</td
					>
					<td class="py-2 text-right" data-testid="supplier-invoice-detail-line-unit-price"
						>{formatExactAmount(line.unitPrice)}</td
					>
					<td class="py-2 text-right">{line.vatRate}%</td>
					<td class="py-2 text-right" data-testid="supplier-invoice-detail-line-total"
						>{formatExactAmount(line.lineTotal)}</td
					>
				</tr>
			{/each}
		</tbody>
	</table>

	{#if invoice.status === 'open'}
		<div class="mb-6 rounded border border-border p-4" data-testid="supplier-invoice-pay">
			<h2 class="mb-3 font-medium">{i18nMsg('supplier-invoices-pay-title', 'Payer la facture')}</h2>
			<div class="mb-3 flex gap-4 text-sm">
				<label class="flex items-center gap-1">
					<input type="radio" bind:group={settlementType} value="bank_transfer" />
					{i18nMsg('supplier-invoices-pay-transfer', 'Virement bancaire')}
				</label>
				<label class="flex items-center gap-1">
					<input type="radio" bind:group={settlementType} value="internal_account" />
					{i18nMsg('supplier-invoices-pay-internal', 'Compte interne (caisse, carte…)')}
				</label>
			</div>

			{#if settlementType === 'bank_transfer'}
				<select
					class="mb-3 w-full rounded border px-2 py-1 text-sm"
					data-testid="pay-bank-account"
					bind:value={payBankAccountId}
				>
					<option value={null}>{i18nMsg('supplier-invoices-pay-bank-ph', 'Compte bancaire source')}</option>
					{#each bankAccounts as b (b.id)}
						<option value={b.id}>{bankAccountOptionLabel(b)}</option>
					{/each}
				</select>
				{#if noEligibleBankAccount}
					<p class="-mt-2 mb-3 text-xs text-destructive" data-testid="pay-no-eligible-bank">
						{i18nMsg(
							'supplier-invoices-pay-no-eligible-bank-account',
							'Aucun compte bancaire utilisable : le seul compte lié est le compte créanciers de cette facture. Reliez un compte bancaire à son propre compte de banque, ou réglez par un compte interne.',
						)}
					</p>
				{/if}
			{:else}
				<select
					class="mb-3 w-full rounded border px-2 py-1 text-sm"
					data-testid="pay-internal-account"
					bind:value={payAccountId}
				>
					<option value={null}>{i18nMsg('supplier-invoices-pay-account-ph', 'Compte de contrepartie')}</option>
					{#each accounts as a (a.id)}
						<option value={a.id}>{a.number} {a.name}</option>
					{/each}
				</select>
			{/if}

			<label class="mb-3 block text-sm">
				{i18nMsg('supplier-invoices-pay-date', 'Date de règlement')}
				<input type="date" class="mt-1 w-full rounded border px-2 py-1" bind:value={paymentDate} />
			</label>

			{#if payError}
				<p class="mb-2 text-sm text-destructive" data-testid="supplier-invoice-pay-error">{payError}</p>
			{/if}

			<div class="flex gap-2">
				<button
					class="rounded bg-primary px-4 py-2 text-sm text-primary-foreground"
					data-testid="supplier-invoice-pay-submit"
					onclick={pay}
					disabled={paying}
				>
					{paying ? '…' : i18nMsg('supplier-invoices-pay-submit', 'Payer')}
				</button>
			</div>
		</div>
	{:else if invoice.status === 'paid'}
		<div class="mb-6 space-y-2" data-testid="supplier-invoice-settlement">
			<p class="text-sm text-text-muted" data-testid="supplier-invoice-paid-info">
				{paidInfo(invoice.paidAt)}
			</p>
			{#if invoice.settlementJournalEntryId}
				<a
					class="text-sm underline"
					href="/journal-entries/{invoice.settlementJournalEntryId}"
					data-testid="supplier-invoice-settlement-entry"
				>
					{i18nMsg('supplier-invoices-settlement-entry-link', "Voir l'écriture de règlement")}
				</a>
			{/if}
			<!-- Story 25-3-a-2 (#414) : le bouton seulement si le serveur a dit
			     « annulable » ; sinon le MOTIF, et seulement sur une facture payée
			     (le rang 1 dirait « non payée » à côté du formulaire « Payer »). -->
			{#if invoice.settlementCancellable === true}
				{#if canManage}
					<button
						class="rounded border px-4 py-2 text-sm text-destructive"
						data-testid="supplier-invoice-settlement-cancel"
						onclick={cancelSettlement}
						disabled={cancellingSettlement}
					>
						{i18nMsg('supplier-invoices-settlement-cancel-button', 'Annuler le règlement')}
					</button>
				{/if}
			{:else if invoice.settlementCancelBlockedBy}
				<p class="text-xs text-text-muted" data-testid="supplier-invoice-settlement-cancel-blocked">
					{supplierSettlementCancelMessage(
						invoice.settlementCancelBlockedBy,
						invoice.settlementCancelBlockedLabel,
					)}
				</p>
			{/if}
			{#if settlementCancelError}
				<p
					class="text-sm text-destructive"
					role="alert"
					data-testid="supplier-invoice-settlement-cancel-error"
				>
					{settlementCancelError}
				</p>
			{/if}
		</div>
	{:else if invoice.status === 'cancelled'}
		<!-- Story 25-3-c : « annulée », et rien d'autre — un règlement éventuel a
		     été DÉTACHÉ (colonnes vidées), la fiche ne sait plus qu'elle a été
		     payée. Elle ne le recherche toujours pas dans l'audit pour l'afficher.
		     ⚠️ Révision nommée de l'arbitrage Q1 (Story 15-8a, C-15-8-20/25) : ce
		     paiement n'est plus « libre » — la garde de modification des
		     écritures lit, elle, la trace d'audit pour le geler (il reste
		     contre-passable). Palliatif, retiré avec #541. -->
		<p class="mb-6 text-sm text-text-muted" data-testid="supplier-invoice-cancelled-info">
			{i18nMsg('supplier-invoices-cancelled-info', 'Facture annulée.')}
		</p>
	{/if}

	<!-- Story 25-3-c (#454) : UN seul bloc pour annuler la facture, validée ou
	     payée. Le bouton seulement si le serveur a dit « annulable » et pour un
	     rôle d'écriture ; sinon le MOTIF — jamais « déjà annulée ».
	     Story v017-4b (#849, AC-22, M21-bis) : jamais pour un brouillon — rien n'est
	     au journal ; il se corrige ou se supprime. -->
	{#if invoice.status !== 'cancelled' && invoice.status !== 'draft'}
		<div class="mb-6 space-y-2" data-testid="supplier-invoice-cancel-block">
			{#if invoice.cancellable === true}
				{#if canManage}
					<button
						class="rounded border px-4 py-2 text-sm text-destructive"
						data-testid="supplier-invoice-cancel"
						onclick={cancel}
						disabled={cancelling}
					>
						{i18nMsg('supplier-invoices-cancel', 'Annuler la facture')}
					</button>
				{/if}
			{:else if invoice.cancelBlockedBy && invoice.cancelBlockedBy !== 'SUPPLIER_INVOICE_CANCELLED'}
				<p class="text-xs text-text-muted" data-testid="supplier-invoice-cancel-blocked">
					{supplierInvoiceCancelMessage(invoice.cancelBlockedBy, invoice.cancelBlockedLabel)}
				</p>
			{/if}
			{#if cancelError}
				<p class="text-sm text-destructive" role="alert" data-testid="supplier-invoice-cancel-error">
					{cancelError}
				</p>
			{/if}
		</div>
	{/if}
{/if}

<!-- Story v017-4b (#849, AC-22) — les confirmations du cycle : chacune dit ce qui va
     s'écrire. Patron de la fiche client (`invoices/[id]/+page.svelte`). -->
<Dialog.Root bind:open={validateOpen}>
	<Dialog.Content data-testid="supplier-invoice-validate-dialog">
		<Dialog.Header>
			<Dialog.Title>{i18nMsg('supplier-invoices-validate-confirm-title', 'Valider la facture ?')}</Dialog.Title>
		</Dialog.Header>
		<p class="text-sm">
			{i18nMsg(
				'supplier-invoices-validate-confirm-body',
				'L’écriture d’achat sera passée au journal, datée du jour de la facture. Validée, la facture peut être payée ; pour la corriger ensuite, il faudra la repasser en brouillon.',
			)}
		</p>
		<Dialog.Footer>
			<Button variant="outline" onclick={() => (validateOpen = false)}>
				{i18nMsg('common-cancel', 'Annuler')}
			</Button>
			<Button data-testid="supplier-invoice-validate-confirm" disabled={validating} onclick={confirmValidate}>
				{i18nMsg('supplier-invoices-validate', 'Valider')}
			</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>

<Dialog.Root bind:open={unvalidateOpen}>
	<Dialog.Content data-testid="supplier-invoice-unvalidate-dialog">
		<Dialog.Header>
			<Dialog.Title>{i18nMsg('supplier-invoices-unvalidate-confirm-title', 'Repasser la facture en brouillon ?')}</Dialog.Title>
		</Dialog.Header>
		<!-- Story v0172-annulees (AC-12) : une annulée a son propre corps — deux écritures,
		     deux trous, les totaux de deux périodes (V2-5). -->
		<p class="text-sm" data-testid="supplier-invoice-unvalidate-dialog-body">
			{#if invoice?.status === 'cancelled'}
				{i18nMsg(
					'supplier-invoices-unvalidate-confirm-body-cancelled',
					'Son écriture d’achat et sa contre-passation seront supprimées du journal, et la numérotation des écritures gardera deux trous ; le journal d’audit garde les deux écritures. Les totaux de la période de l’achat et de celle de l’annulation changeront, TVA comprise : si l’une a déjà été déclarée, faites d’abord verrouiller la période. La facture redeviendra un brouillon modifiable — à valider de nouveau, ou à supprimer.',
				)}
			{:else}
				{i18nMsg(
					'supplier-invoices-unvalidate-confirm-body',
					'Son écriture d’achat sera supprimée du journal, et la numérotation des écritures gardera un trou. La facture redeviendra un brouillon modifiable, à valider de nouveau.',
				)}
			{/if}
		</p>
		<!-- Revue P1 (E-2 = A-1, L4) : la confirmation rappelle le lot confirmé. -->
		{#if invoice?.lastConfirmedBatch}
			<p
				class="rounded-md border border-warning bg-warning/10 px-3 py-2 text-sm"
				role="note"
				data-testid="supplier-invoice-unvalidate-dialog-batch-warning"
			>
				{confirmedBatchFact(invoice.lastConfirmedBatch)}
				{confirmedBatchAction()}
			</p>
		{/if}
		<Dialog.Footer>
			<Button variant="outline" onclick={() => (unvalidateOpen = false)}>
				{i18nMsg('common-cancel', 'Annuler')}
			</Button>
			<Button
				variant="destructive"
				data-testid="supplier-invoice-unvalidate-confirm"
				disabled={unvalidating}
				onclick={confirmUnvalidate}
			>
				{i18nMsg('supplier-invoices-unvalidate', 'Repasser en brouillon')}
			</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>

<Dialog.Root bind:open={deleteOpen}>
	<Dialog.Content data-testid="supplier-invoice-delete-dialog">
		<Dialog.Header>
			<Dialog.Title>{i18nMsg('supplier-invoices-delete-confirm-title', 'Supprimer le brouillon ?')}</Dialog.Title>
		</Dialog.Header>
		<p class="text-sm">
			{i18nMsg(
				'supplier-invoices-delete-confirm-body',
				'Ce brouillon sera supprimé définitivement. S’il a un document, celui-ci reviendra dans les documents à traiter.',
			)}
		</p>
		{#if invoice?.internalNumber}
			<p class="text-sm" data-testid="supplier-invoice-delete-number-warning">
				{i18nMsg(
					'supplier-invoices-delete-confirm-number',
					'Le numéro { $number } sera perdu. Kesh ne le reprendra pas, hors les cas dits au manuel (sauvegarde restaurée, réinitialisation).',
					{ number: invoice.internalNumber },
				)}
			</p>
		{/if}
		<Dialog.Footer>
			<Button variant="outline" onclick={() => (deleteOpen = false)}>
				{i18nMsg('common-cancel', 'Annuler')}
			</Button>
			<Button
				variant="destructive"
				data-testid="supplier-invoice-delete-confirm"
				disabled={deleting}
				onclick={confirmDelete}
			>
				{i18nMsg('supplier-invoices-delete-draft', 'Supprimer le brouillon')}
			</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>
