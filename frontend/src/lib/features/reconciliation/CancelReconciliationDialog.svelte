<!--
  Story 25-3-b (#418) — Dialogue « Annuler le rapprochement ».

  ⛔ UN SEUL composant, pour deux pages : le détail d'un import bancaire et la
  fiche facture client (à côté d'un règlement refusé au motif « rapproché »).
  Trois genres de rapprochement (Story 30-1b) : règlement client, règlement
  fournisseur (`supplier_settlement`), écriture propre.
  Premier dialogue partagé entre deux pages : la 25-3-a-1 confirme DANS la page.

  Contrat : `bankTransactionId`, `open`, `onClose()`, `onSuccess(result)`.
  Le composant fait LUI-MÊME la lecture (`GET …/transactions/{id}`) à
  l'ouverture et l'appel d'annulation ; chaque page ne fait que relire dans
  `onSuccess`, et masque le bouton selon le rôle.

  Il montre SOIT le motif qui refuse l'annulation, SOIT la confirmation — qui
  dit ce qui va se passer selon le genre du rapprochement, ou, pour un côté d'un
  virement rattaché des deux côtés (Story 34-11b, `unlinkOnly`), que seul le
  lien est retiré.
-->
<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import * as Dialog from '$lib/components/ui/dialog';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { supplierInvoiceShortLabel } from '$lib/features/supplier-invoices/supplier-invoice-label';
	import { isApiError } from '$lib/shared/utils/api-client';
	import { errorMessageOf } from '$lib/shared/utils/error-message';
	import { cancelReconciliation, getReconciliationTransaction } from './reconciliation.api';
	import type {
		CancelReconciliationResponse,
		ReconciliationTransactionResponse,
	} from './reconciliation.types';
	import {
		groupedCancelBlockedMessage,
		isReconciliationCancelCode,
		reconciliationCancelErrorMessage,
		reconciliationCancelMessage,
	} from './reconciliation-cancel';
	import type { GroupedPieceRef } from './reconciliation.types';
	import { displayDue, parseUnits } from './group-settlement';
	import { formatSwissDate } from '$lib/features/reports/reports.api';
	import { gapSenseLabel } from './gap-label';

	/** Story 43-3 — le numéro d'une facture d'un groupe, `#id` à défaut. */
	function pieceLabel(p: GroupedPieceRef): string {
		return p.number !== null && p.number.trim() !== '' ? p.number : `#${p.id}`;
	}

	type Props = {
		bankTransactionId: number;
		open: boolean;
		onClose: () => void;
		onSuccess: (result: CancelReconciliationResponse) => void;
	};

	let { bankTransactionId, open, onClose, onSuccess }: Props = $props();

	let view = $state<ReconciliationTransactionResponse | null>(null);
	let loading = $state(false);
	let submitting = $state(false);
	let errorMsg = $state('');

	// Relecture à chaque ouverture : l'état a pu changer depuis la dernière.
	$effect(() => {
		if (open) {
			void load(bankTransactionId);
		}
	});

	async function load(id: number) {
		view = null;
		errorMsg = '';
		loading = true;
		try {
			view = await getReconciliationTransaction(id);
		} catch (err) {
			errorMsg = errorMessageOf(err);
		} finally {
			loading = false;
		}
	}

	/** Le motif affiché à la place de la confirmation, ou `null`. */
	const motif = $derived.by(() => {
		const code = view?.cancelBlockedBy;
		if (!view || view.cancellable || !code) return null;
		if (!isReconciliationCancelCode(code)) return code;
		// Story 43-3 (#823, AC5) — un refus par facture d'un règlement groupé nomme la facture.
		const blockedInvoice = view.kind === 'grouped_settlement' ? view.cancelBlockedInvoice : null;
		if (blockedInvoice) {
			const grouped = groupedCancelBlockedMessage(code, pieceLabel(blockedInvoice));
			if (grouped !== null) return grouped;
		}
		return reconciliationCancelMessage(code, view.cancelBlockedLabel);
	});

	/** Story 43-3 (#823, AC5) — les factures d'un règlement groupé, que l'annulation rouvrira. */
	const groupedInvoices = $derived(
		view?.kind === 'grouped_settlement' ? (view.groupedInvoices ?? []) : [],
	);

	/**
	 * Story 43-3 — l'écart du groupe et son compte, tels que le serveur les rend (`gap`,
	 * `differenceAccount`, recalculés par la 43-2), l'écart dit avec son **sens**, comme dans la
	 * modale (revue P1, A L-2, E-L11) ; annoncé même si son compte n'a pas pu être relu ; rien si
	 * l'écart est nul ou illisible.
	 */
	const groupGapText = $derived.by(() => {
		if (view?.kind !== 'grouped_settlement') return null;
		const gap = parseUnits(view.gap ?? null);
		if (gap === null || gap === 0n) return null;
		const family = (view.groupedInvoices ?? []).some((g) => g.kind === 'supplier_invoice')
			? 'supplier'
			: 'client';
		const label = gapSenseLabel(family, gap);
		const account = view.differenceAccount ?? null;
		if (account === null) {
			return i18nMsg(
				'reconciliation-cancel-confirm-group-gap-no-account',
				'{ $gap } sera contre-passé avec l’écriture.',
				{ gap: label },
			);
		}
		return i18nMsg(
			'reconciliation-cancel-confirm-group-gap',
			'{ $gap }, passé au compte { $account }, sera contre-passé avec l’écriture.',
			{ gap: label, account: `${account.number} ${account.name}` },
		);
	});

	const confirmText = $derived.by(() => {
		// Story 34-11b (#972, AC-4b) — un côté d'un virement rattaché des deux côtés : seul le lien
		// de cette transaction est retiré, l'écriture reste celle de l'autre côté. Avant le genre :
		// aucune contre-passation, aucun règlement retiré.
		if (view?.unlinkOnly) {
			return i18nMsg(
				'reconciliation-cancel-confirm-unlink-only',
				'Seul le lien de cette transaction à l’écriture du virement sera retiré : l’écriture reste rapprochée de l’autre côté du virement, sur votre autre compte, et aucune écriture inverse n’est passée. La transaction redeviendra à rapprocher.',
			);
		}
		// Story 53-6a (#1674, D4) — la contre-passation est datée de l'écriture du rapprochement,
		// dans son exercice : la confirmation les nomme, au lieu de « aujourd'hui ».
		// Une vue rendue par un serveur antérieur ne porte ni l'un ni l'autre : un tiret plutôt
		// qu'un blanc, qui laisserait une phrase tronquée (revue P1, L4).
		const date = view?.reversalDate ? formatSwissDate(view.reversalDate) : '—';
		const fiscalYear = view?.reversalFiscalYearName ?? '—';
		if (view?.kind === 'invoice_settlement') {
			return i18nMsg(
				'reconciliation-cancel-confirm-invoice',
				"Une écriture inverse datée du { $date } (date de l'écriture du rapprochement) sera passée dans l'exercice « { $fiscalYear } », le règlement de la facture { $invoice } sera retiré et la facture redeviendra à régler ; la transaction redeviendra à rapprocher.",
				{ date, fiscalYear, invoice: view.invoiceNumber ?? String(view.invoiceId ?? '') },
			);
		}
		// Story 30-1b (#686, AC-b11) — un règlement fournisseur : son paiement est retiré
		// et la facture redevient validée, à payer (Story v017-4b : « validée », décision Q4).
		if (view?.kind === 'supplier_settlement') {
			return i18nMsg(
				'reconciliation-cancel-confirm-supplier-invoice',
				"Une écriture inverse datée du { $date } (date de l'écriture du rapprochement) sera passée dans l'exercice « { $fiscalYear } », le paiement de la facture fournisseur { $invoice } sera retiré et la facture redeviendra validée, à payer ; la transaction redeviendra à rapprocher.",
				{
					date,
					fiscalYear,
					invoice: supplierInvoiceShortLabel({
						id: view.supplierInvoiceId ?? 0,
						internalNumber: view.internalNumber,
						supplierInvoiceNumber: view.supplierInvoiceNumber,
					}),
				},
			);
		}
		// Story 43-3 (#823, AC5) — un règlement groupé : ses N factures redeviennent ouvertes.
		if (view?.kind === 'grouped_settlement') {
			const invoices = (view.groupedInvoices ?? []).map(pieceLabel).join(', ');
			// D6 — un groupe fournisseur : le paiement est retiré, les factures redeviennent validées.
			if ((view.groupedInvoices ?? []).some((g) => g.kind === 'supplier_invoice')) {
				return i18nMsg(
					'reconciliation-cancel-confirm-group-supplier',
					"Une écriture inverse datée du { $date } (date de l'écriture du rapprochement) sera passée dans l'exercice « { $fiscalYear } », le paiement des factures fournisseur { $invoices } sera retiré et elles redeviendront validées, à payer ; la transaction redeviendra à rapprocher.",
					{ date, fiscalYear, invoices },
				);
			}
			return i18nMsg(
				'reconciliation-cancel-confirm-group',
				"Une écriture inverse datée du { $date } (date de l'écriture du rapprochement) sera passée dans l'exercice « { $fiscalYear } », le règlement des factures { $invoices } sera retiré et elles redeviendront à régler ; la transaction redeviendra à rapprocher.",
				{ date, fiscalYear, invoices },
			);
		}
		return i18nMsg(
			'reconciliation-cancel-confirm-entry',
			"Une écriture inverse datée du { $date } (date de l'écriture du rapprochement) sera passée dans l'exercice « { $fiscalYear } », et la transaction redeviendra à rapprocher.",
			{ date, fiscalYear },
		);
	});

	/**
	 * Story 53-6a (#1674, D4), corrigée par la 63-1 — la contre-passation d'une **écriture propre**
	 * (`kind === 'entry'` : éclatement, règle, rapprochement manuel, écriture saisie au journal)
	 * ne change la TVA de sa période que si l'écriture porte le compte d'impôt préalable : le
	 * rapport lit ce compte au grand livre (`vat_report.rs`, `recoverable_balance`). Un règlement
	 * client, un paiement fournisseur ou un règlement groupé ne portent aucune ligne de TVA : leurs
	 * seuls effets sont les soldes de la période (contrepartie, débiteurs ou créanciers) — aucune
	 * phrase de TVA n'est montrée. Jamais pour un retrait seul (aucune écriture inverse).
	 */
	const vatText = $derived(
		view && !view.unlinkOnly && (view.kind === 'entry' || view.kind == null)
			? i18nMsg(
					'reconciliation-cancel-confirm-vat',
					"Si cette écriture porte le compte d'impôt préalable, la TVA de sa période change : si cette période a déjà été déclarée, faites d'abord verrouiller la période — l'annulation sera alors refusée, ce qui protège la déclaration.",
				)
			: null,
	);

	async function confirm() {
		if (submitting) return;
		submitting = true;
		errorMsg = '';
		try {
			// Story 34-11b (revue P1 E1-1) : le geste porte la décision que la confirmation montre.
			const result = await cancelReconciliation(bankTransactionId, view?.unlinkOnly ?? false);
			onSuccess(result);
		} catch (err) {
			const message = isApiError(err)
				? reconciliationCancelErrorMessage(err)
				: errorMessageOf(err);
			// La décision a changé entre-temps : la vue est relue, la confirmation juste remontrée,
			// et le refus reste affiché au-dessus — rien n'a été annulé.
			if (isApiError(err) && err.code === 'RECONCILIATION_CANCEL_CHANGED') {
				await load(bankTransactionId);
			}
			errorMsg = message;
		} finally {
			submitting = false;
		}
	}
</script>

<Dialog.Root {open} onOpenChange={(v) => !v && onClose()}>
	<Dialog.Content data-testid="reconciliation-cancel-dialog">
		<Dialog.Header>
			<Dialog.Title>
				{i18nMsg('reconciliation-cancel-button', 'Annuler le rapprochement')}
			</Dialog.Title>
		</Dialog.Header>

		{#if loading}
			<p class="text-sm text-text-muted">
				{i18nMsg('reconciliation-cancel-loading', 'Lecture du rapprochement…')}
			</p>
		{:else if motif}
			<p class="text-sm" data-testid="reconciliation-cancel-motif">{motif}</p>
		{:else if view}
			<p class="text-sm" data-testid="reconciliation-cancel-confirm-text">{confirmText}</p>
			{#if groupedInvoices.length > 0}
				<!-- Story 43-3 (#823, AC5) — les N factures que l'annulation rouvrira, montant brut. -->
				<ul class="list-disc pl-6 text-sm" data-testid="reconciliation-cancel-group-invoices">
					{#each groupedInvoices as g (`${g.kind}-${g.id}`)}
						<li data-testid="reconciliation-cancel-group-invoice" data-invoice-id={g.id}>
							{pieceLabel(g)} — {displayDue(g.amount)}
						</li>
					{/each}
				</ul>
			{/if}
			{#if groupGapText}
				<p class="text-sm" data-testid="reconciliation-cancel-group-gap">{groupGapText}</p>
			{/if}
			{#if vatText}
				<p class="text-sm text-text-muted" data-testid="reconciliation-cancel-vat">{vatText}</p>
			{/if}
		{/if}

		{#if errorMsg}
			<div
				class="rounded-md border border-destructive bg-destructive/10 px-3 py-2 text-sm text-destructive"
				role="alert"
				data-testid="reconciliation-cancel-error"
			>
				{errorMsg}
			</div>
		{/if}

		<Dialog.Footer>
			<Button variant="outline" onclick={onClose} disabled={submitting}>
				{i18nMsg('reconciliation-cancel-dismiss', 'Fermer')}
			</Button>
			{#if view?.cancellable}
				<Button
					variant="destructive"
					onclick={confirm}
					disabled={submitting}
					data-testid="reconciliation-cancel-submit"
				>
					{i18nMsg('reconciliation-cancel-confirm-submit', "Confirmer l'annulation")}
				</Button>
			{/if}
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>
