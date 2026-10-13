<!--
  Story 27-1 (#632) — « Supprimer l'import », dans le détail d'un import bancaire.

  Contrat : `detail`, `canManage`, `onDeleted(filename)`.
  - `canManage` faux (rôle Consultation) ⇒ le composant ne rend RIEN.
  - Aucune transaction rapprochée ⇒ bouton actif ; au moins une ⇒ bouton
    désactivé et motif avec le nombre (définition : `isReconciled`, qui
    renvoie au serveur). Le serveur seul décide : un `409` reste possible si
    une transaction a été rapprochée entre-temps.
  - Clic ⇒ dialogue qui nomme le fichier, la période et le nombre de
    transactions supprimées ; la confirmation appelle `deleteBankImport`.
-->
<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import * as Dialog from '$lib/components/ui/dialog';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { isApiError } from '$lib/shared/utils/api-client';
	import { errorMessageOf } from '$lib/shared/utils/error-message';
	import { deleteBankImport } from './bank-import.api';
	import { reconciledCount } from './bank-import-deletion';
	import type { BankImportDetailResponse } from './bank-import.types';

	type Props = {
		detail: BankImportDetailResponse;
		canManage: boolean;
		onDeleted: (filename: string) => void;
	};

	let { detail, canManage, onDeleted }: Props = $props();

	let open = $state(false);
	let submitting = $state(false);
	let errorMsg = $state('');

	const blocked = $derived(reconciledCount(detail.transactions));

	function openDialog() {
		errorMsg = '';
		open = true;
	}

	async function confirm() {
		if (submitting) return;
		submitting = true;
		errorMsg = '';
		try {
			await deleteBankImport(detail.id);
			open = false;
			onDeleted(detail.filename);
		} catch (err) {
			if (isApiError(err) && err.code === 'BANK_IMPORT_HAS_RECONCILED_TRANSACTIONS') {
				errorMsg = i18nMsg(
					'bank-import-delete-error-reconciled',
					'{ $count } transaction(s) de cet import ont été rapprochées entre-temps : rechargez la page.',
					{ count: Number(err.details?.reconciledCount ?? 0) },
				);
			} else if (isApiError(err) && err.status === 404) {
				errorMsg = i18nMsg(
					'bank-import-delete-error-not-found',
					"Cet import n'existe plus : il a peut-être déjà été supprimé.",
				);
			} else {
				errorMsg = errorMessageOf(err);
			}
		} finally {
			submitting = false;
		}
	}
</script>

{#if canManage}
	<div class="mt-4">
		<Button
			variant="destructive"
			onclick={openDialog}
			disabled={blocked > 0}
			data-testid="bank-import-delete-button"
		>
			{i18nMsg('bank-import-delete-button', "Supprimer l'import")}
		</Button>
		{#if blocked > 0}
			<p class="mt-2 text-sm text-text-muted" data-testid="bank-import-delete-blocked">
				{i18nMsg(
					'bank-import-delete-blocked',
					"{ $count } transaction(s) de cet import sont rapprochées. Pour supprimer l'import, annulez d'abord leur rapprochement.",
					{ count: blocked },
				)}
			</p>
		{/if}
	</div>

	<!-- Revue de code P2 (B2-1) : pendant l'envoi, ni Échap, ni clic extérieur,
	     ni croix — l'erreur éventuelle ne serait plus montrée nulle part. Le
	     wrapper `Dialog` du projet tient sa propre copie d'`open` : la garde vit
	     donc dans `Dialog.Content` (patron de `WriteOffDialog`), et `open` est
	     LIÉ (`bind:open`) pour qu'il n'existe qu'un état — un `{open}` à sens
	     unique laissait le parent à `true` après une fermeture, et le bouton ne
	     rouvrait plus le dialogue. -->
	<Dialog.Root bind:open>
		<Dialog.Content
			escapeKeydownBehavior={submitting ? 'ignore' : 'close'}
			interactOutsideBehavior={submitting ? 'ignore' : 'close'}
			showCloseButton={!submitting}
			data-testid="bank-import-delete-dialog"
		>
			<Dialog.Header>
				<Dialog.Title>
					{i18nMsg('bank-import-delete-dialog-title', 'Supprimer cet import ?')}
				</Dialog.Title>
			</Dialog.Header>

			<p class="text-sm" data-testid="bank-import-delete-dialog-body">
				{i18nMsg(
					'bank-import-delete-dialog-body',
					'Le fichier { $filename } (période du { $from } au { $to }) et ses { $count } transaction(s) seront supprimés. Vous pourrez ensuite réimporter le même fichier. Cette suppression ne peut pas être annulée.',
					{
						filename: detail.filename,
						from: detail.periodFrom,
						to: detail.periodTo,
						count: detail.transactions.length,
					},
				)}
			</p>

			{#if errorMsg}
				<div
					class="rounded-md border border-destructive bg-destructive/10 px-3 py-2 text-sm text-destructive"
					role="alert"
					data-testid="bank-import-delete-error"
				>
					{errorMsg}
				</div>
			{/if}

			<Dialog.Footer>
				<Button variant="outline" onclick={() => (open = false)} disabled={submitting}>
					{i18nMsg('bank-import-labels-cancel', 'Annuler')}
				</Button>
				<Button
					variant="destructive"
					onclick={confirm}
					disabled={submitting}
					data-testid="bank-import-delete-submit"
				>
					{i18nMsg('bank-import-delete-dialog-confirm', 'Supprimer définitivement')}
				</Button>
			</Dialog.Footer>
		</Dialog.Content>
	</Dialog.Root>
{/if}
