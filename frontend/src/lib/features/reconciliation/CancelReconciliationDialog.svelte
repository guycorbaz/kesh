<!--
  Story 25-3-b (#418) — Dialogue « Annuler le rapprochement ».

  ⛔ UN SEUL composant, pour deux pages : le détail d'un import bancaire et la
  fiche facture client (à côté d'un règlement refusé au motif « rapproché »).
  Premier dialogue partagé entre deux pages : la 25-3-a-1 confirme DANS la page.

  Contrat : `bankTransactionId`, `open`, `onClose()`, `onSuccess(result)`.
  Le composant fait LUI-MÊME la lecture (`GET …/transactions/{id}`) à
  l'ouverture et l'appel d'annulation ; chaque page ne fait que relire dans
  `onSuccess`, et masque le bouton selon le rôle.

  Il montre SOIT le motif qui refuse l'annulation, SOIT la confirmation — qui
  dit ce qui va se passer selon le genre du rapprochement.
-->
<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import * as Dialog from '$lib/components/ui/dialog';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { isApiError } from '$lib/shared/utils/api-client';
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
			errorMsg = isApiError(err) ? err.message : String(err);
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
		if (view?.kind === 'invoice_settlement') {
			return i18nMsg(
				'reconciliation-cancel-confirm-invoice',
				"Une écriture inverse datée d'aujourd'hui sera passée, le règlement de la facture { $invoice } sera retiré et la facture redeviendra à régler ; la transaction redeviendra à rapprocher.",
				{ invoice: view.invoiceNumber ?? String(view.invoiceId ?? '') },
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
			const result = await cancelReconciliation(bankTransactionId);
			onSuccess(result);
		} catch (err) {
			errorMsg = isApiError(err) ? reconciliationCancelErrorMessage(err) : String(err);
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
