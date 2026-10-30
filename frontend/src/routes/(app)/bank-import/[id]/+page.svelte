<!--
  Story 8-1b — Page détail d'un import bancaire (review code Pass 1 H2).
  Route /bank-import/[id]. Consomme GET /api/v1/bank-imports/{id}.
  Affiche l'entête (filename, période, soldes, source_format) + table
  des transactions persistées.

  Story 28-5b (#526) — c'est aussi le seul écran qui montre une transaction
  REJETÉE (elle a quitté la liste de la réconciliation) : colonne
  *Rapprochement*, mention « Rejetée le … » et, pour l'Admin et le Comptable,
  *Annuler le rejet* (`UnrejectTransactionAction`).
-->
<script lang="ts">
	import { page } from '$app/state';
	import { onMount } from 'svelte';
	import { getBankImportDetail } from '$lib/features/bank-import/bank-import.api';
	import type { BankImportDetailResponse } from '$lib/features/bank-import/bank-import.types';
	import { isApiError } from '$lib/shared/utils/api-client';
	import { authState } from '$lib/app/stores/auth.svelte';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import CancelReconciliationDialog from '$lib/features/reconciliation/CancelReconciliationDialog.svelte';
	import DeleteBankImportAction from '$lib/features/bank-import/DeleteBankImportAction.svelte';
	import UnrejectTransactionAction from '$lib/features/reconciliation/UnrejectTransactionAction.svelte';
	import { notifySuccess } from '$lib/shared/utils/notify';
	import { goto } from '$app/navigation';
	import {
		importDisplayName,
		isManualImport,
		sourceFormatLabel,
	} from '$lib/features/bank-import/bank-import-display';

	// Story 25-3-b (#418) — annuler un rapprochement depuis le détail d'import,
	// le seul écran qui montre une transaction rapprochée. Le bouton est masqué
	// pour un rôle sans droit d'écriture ; le serveur refuse de toute façon.
	let canManage = $derived(
		authState.currentUser?.role === 'Admin' || authState.currentUser?.role === 'Comptable',
	);
	let cancelTxId = $state<number | null>(null);

	// Story 27-1 (#632) — après une suppression, retour à la liste, qui se
	// recharge à son montage.
	// Story v016-2b (#816, AC-23) — un relevé saisi a sa propre notification, qui ne
	// nomme pas de fichier.
	function onImportDeleted(filename: string) {
		if (detail && isManualImport(detail)) {
			notifySuccess(
				i18nMsg(
					'bank-import-manual-delete-success',
					'Relevé saisi à la main du { $from } au { $to } supprimé.',
					{ from: detail.periodFrom, to: detail.periodTo },
				),
			);
		} else {
			notifySuccess(i18nMsg('bank-import-delete-success', 'Import { $filename } supprimé.', { filename }));
		}
		void goto('/bank-import');
	}

	let detail = $state<BankImportDetailResponse | null>(null);
	let errorMessage = $state<string | null>(null);
	let loading = $state(true);

	onMount(() => load());

	// Story 29-9 (#655, AC5) — numéro de la dernière lecture lancée. Deux relectures peuvent être
	// en vol (deux « Annuler le rejet » coup sur coup) : seule la plus récente écrit `detail` et
	// `errorMessage`, sans quoi la réponse la plus LENTE l'emportait — une erreur ancienne
	// masquait une page à jour. Patron : `loadGen` de `ReconciliationProposals.svelte`.
	let loadSeq = 0;

	async function load() {
		const mySeq = ++loadSeq;
		// Story 28-5b, revue P2 (P2-1) — chaque relecture repart d'un état sans erreur : la
		// bannière masque les lignes, et une relecture réussie après une relecture en échec
		// (après « Annuler le rejet » ou « Annuler le rapprochement ») doit les rendre.
		errorMessage = null;
		const idStr = page.params.id;
		const id = idStr ? Number(idStr) : NaN;
		if (!Number.isFinite(id) || id <= 0) {
			errorMessage = i18nMsg('bank-import-detail-error-invalid-id', 'Identifiant invalide.');
			loading = false;
			return;
		}
		try {
			const d = await getBankImportDetail(id);
			if (mySeq !== loadSeq) return; // relecture dépassée par une plus récente
			detail = d;
		} catch (err) {
			if (mySeq !== loadSeq) return;
			if (isApiError(err)) {
				errorMessage =
					err.status === 404
						? i18nMsg('bank-import-detail-error-not-found', 'Relevé introuvable.')
						: err.message;
			} else {
				errorMessage = i18nMsg('error-unexpected', 'Erreur inattendue.');
			}
		} finally {
			// Non gardé (C-29-9-14) : `load()` ne remet jamais `loading` à `true`, et une
			// relecture ne part que d'une page déjà rendue — un `false` périmé n'a aucun effet.
			// ⚠️ Le jour où `load()` remettra `loading` à `true`, garder aussi cette ligne.
			loading = false;
		}
	}
</script>

<svelte:head>
	<title>{i18nMsg('bank-import-detail-title', 'Détail du relevé bancaire')} - Kesh</title>
</svelte:head>

<a href="/bank-import" class="text-primary underline" data-testid="bank-import-detail-back">
	{i18nMsg('bank-import-detail-back', '← Retour aux imports')}
</a>

<h1 class="mt-2 text-2xl font-semibold text-text" data-testid="bank-import-detail-title">
	{i18nMsg('bank-import-detail-title', 'Détail du relevé bancaire')}
</h1>

{#if loading}
	<p class="mt-4 text-text-muted">{i18nMsg('common-loading', 'Chargement…')}</p>
{:else if errorMessage}
	<div
		class="mt-4 rounded border border-error bg-error-soft p-3"
		data-testid="bank-import-detail-error"
		role="alert"
	>
		<p class="text-error">{errorMessage}</p>
	</div>
{:else if detail}
	<section class="mt-6" data-testid="bank-import-detail">
		<h2 class="text-lg font-semibold">
			{i18nMsg('bank-import-detail-heading', 'Relevé #{ $id }', { id: detail.id })}
		</h2>
		<dl class="mt-2 grid grid-cols-2 gap-2 text-sm">
			{#if isManualImport(detail)}
				<!-- Story v016-2b (#816, AC-23) : un relevé saisi n'a pas de fichier. -->
				<dt>{i18nMsg('bank-import-detail-statement', 'Relevé')}</dt>
				<dd data-testid="detail-filename">{importDisplayName(detail)}</dd>
				<dt>{i18nMsg('bank-import-detail-entered-at', 'Saisi le')}</dt>
				<dd>{detail.importedAt}</dd>
			{:else}
				<dt>{i18nMsg('bank-import-detail-file', 'Fichier')}</dt>
				<dd data-testid="detail-filename">{detail.filename}</dd>
				<dt>{i18nMsg('bank-import-detail-imported-at', 'Importé le')}</dt>
				<dd>{detail.importedAt}</dd>
			{/if}
			<dt>{i18nMsg('bank-import-detail-source-format', 'Format source')}</dt>
			<dd data-testid="detail-source-format">{sourceFormatLabel(detail.sourceFormat)}</dd>
			<dt>{i18nMsg('bank-import-labels-period', 'Période')}</dt>
			<dd>{detail.periodFrom} → {detail.periodTo}</dd>
			<dt>{i18nMsg('bank-import-labels-opening-balance', 'Solde ouverture')}</dt>
			<dd>{detail.openingBalance ?? '—'}</dd>
			<dt>{i18nMsg('bank-import-labels-closing-balance', 'Solde clôture')}</dt>
			<dd>{detail.closingBalance ?? '—'}</dd>
			<dt>{i18nMsg('bank-import-labels-transaction-count', 'Nombre de transactions')}</dt>
			<dd data-testid="detail-tx-count">{detail.transactionCount}</dd>
		</dl>

		<DeleteBankImportAction {detail} {canManage} onDeleted={onImportDeleted} />

		<h2 class="mt-6 text-lg font-semibold">
			{i18nMsg('bank-import-labels-list-col-transactions', 'Transactions')}
		</h2>
		<table class="mt-2 w-full table-auto text-sm" data-testid="detail-tx-table">
			<thead>
				<tr>
					<th class="text-left">{i18nMsg('bank-import-labels-tx-col-date', 'Date')}</th>
					<th class="text-right">{i18nMsg('bank-import-labels-tx-col-amount', 'Montant')}</th>
					<th class="text-left">{i18nMsg('bank-import-detail-col-reference', 'Référence')}</th>
					<th class="text-left">{i18nMsg('bank-import-labels-tx-col-details', 'Détails')}</th>
					<th class="text-left">{i18nMsg('bank-import-detail-col-status', 'Statut')}</th>
					<th class="text-left">
						{i18nMsg('reconciliation-cancel-column', 'Rapprochement')}
					</th>
				</tr>
			</thead>
			<tbody>
				{#each detail.transactions as tx (tx.id)}
					<tr data-testid="detail-tx-row" data-tx-id={tx.id}>
						<td>{tx.bookingDate}</td>
						<td class="text-right">{tx.amount} {tx.currency}</td>
						<td>{tx.reference ?? '—'}</td>
						<td>{tx.details}</td>
						<td>{tx.status}</td>
						<td>
							{#if tx.matchedEntryId !== null}
								<a
									href="/journal-entries/{tx.matchedEntryId}"
									class="text-primary underline"
									data-testid="detail-tx-entry-link"
								>
									{i18nMsg('reconciliation-cancel-entry-link', "Voir l'écriture")}
								</a>
							{/if}
							{#if tx.status === 'reconciled' && canManage}
								<button
									type="button"
									class="ml-2 text-destructive underline"
									data-testid="detail-tx-cancel-reconciliation"
									onclick={() => (cancelTxId = tx.id)}
								>
									{i18nMsg('reconciliation-cancel-button', 'Annuler le rapprochement')}
								</button>
							{/if}
							<UnrejectTransactionAction {tx} {canManage} onUnrejected={() => void load()} />
						</td>
					</tr>
				{/each}
			</tbody>
		</table>
	</section>
{/if}

{#if cancelTxId !== null}
	<CancelReconciliationDialog
		bankTransactionId={cancelTxId}
		open={cancelTxId !== null}
		onClose={() => (cancelTxId = null)}
		onSuccess={() => {
			cancelTxId = null;
			void load();
		}}
	/>
{/if}
