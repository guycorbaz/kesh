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
		isReconciliationCancelCode,
		reconciliationCancelErrorMessage,
		reconciliationCancelMessage,
	} from './reconciliation-cancel';

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
		return isReconciliationCancelCode(code)
			? reconciliationCancelMessage(code, view.cancelBlockedLabel)
			: code;
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
		if (view?.kind === 'invoice_settlement') {
			return i18nMsg(
				'reconciliation-cancel-confirm-invoice',
				"Une écriture inverse datée d'aujourd'hui sera passée, le règlement de la facture { $invoice } sera retiré et la facture redeviendra à régler ; la transaction redeviendra à rapprocher.",
				{ invoice: view.invoiceNumber ?? String(view.invoiceId ?? '') },
			);
		}
		// Story 30-1b (#686, AC-b11) — un règlement fournisseur : son paiement est retiré
		// et la facture redevient validée, à payer (Story v017-4b : « validée », décision Q4).
		if (view?.kind === 'supplier_settlement') {
			return i18nMsg(
				'reconciliation-cancel-confirm-supplier-invoice',
				"Une écriture inverse datée d'aujourd'hui sera passée, le paiement de la facture fournisseur { $invoice } sera retiré et la facture redeviendra validée, à payer ; la transaction redeviendra à rapprocher.",
				{
					invoice: supplierInvoiceShortLabel({
						id: view.supplierInvoiceId ?? 0,
						internalNumber: view.internalNumber,
						supplierInvoiceNumber: view.supplierInvoiceNumber,
					}),
				},
			);
		}
		return i18nMsg(
			'reconciliation-cancel-confirm-entry',
			"Une écriture inverse datée d'aujourd'hui sera passée, et la transaction redeviendra à rapprocher.",
		);
	});

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
