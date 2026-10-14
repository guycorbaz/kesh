<!--
  Story 29-8 (#670, AC4) — rattacher une transaction bancaire au règlement déjà saisi depuis
  la fiche de la facture.

  Kesh PROPOSE une liste (`GET …/link-candidates`), l'utilisateur CHOISIT et CONFIRME : aucune
  ligne n'est présélectionnée, « Rattacher » reste grisé tant qu'aucun choix n'est fait (règle
  « Un appariement automatique propose, il ne crée jamais », C-29-8-7). Un candidat à écart non
  nul affiche un avertissement AVANT la confirmation : l'écart reste au compte de la banque.

  Sur `409 RECONCILIATION_SETTLEMENT_NOT_LINKABLE` (la liste a changé entre l'ouverture et le
  clic), la modale recharge la liste et remet le choix à vide — le message dit « relire la
  liste », la modale le fait (C-29-8-24). De même sur les deux autres refus d'une course :
  `404 NOT_FOUND` (règlement annulé depuis la fiche) et `404 RECONCILIATION_TRANSACTION_NOT_PENDING`
  (transaction rapprochée ailleurs) — revue de code P1, B3 (C-29-8-38).

  Garde de session (revue de code P1, B1/E3, C-29-8-37) : chaque ouverture, fermeture ou
  changement de transaction ouvre une session neuve ; un chargement ou une soumission lancés
  dans une session précédente n'agissent jamais sur la modale courante.
-->
<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import * as Dialog from '$lib/components/ui/dialog';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { isApiError } from '$lib/shared/utils/api-client';
	import { errorMessageOf } from '$lib/shared/utils/error-message';
	import type { LinkCandidate, ReconciliationProposal } from './reconciliation.types';
	import { getLinkCandidates, linkTransaction } from './reconciliation.api';

	type Props = {
		open: boolean;
		onOpenChange: (v: boolean) => void;
		proposal: ReconciliationProposal | null;
		onSuccess: () => void;
	};

	let { open, onOpenChange, proposal, onSuccess }: Props = $props();

	let candidates = $state<LinkCandidate[]>([]);
	let selectedId = $state<number | null>(null);
	let loading = $state(false);
	let loadError = $state('');
	let submitting = $state(false);
	let errorMsg = $state('');

	// Garde de génération : une réponse d'un chargement périmé (réouverture, autre
	// transaction, rechargement plus récent) ne remplace pas la liste courante.
	let loadGen = 0;
	// Garde de session : incrémentée à chaque ouverture, fermeture ou changement de
	// transaction. Ni `$state` ni lue par l'effet : elle ne déclenche rien.
	let session = 0;

	/** Les refus d'une course : la liste a changé, la modale la relit. */
	const RELIRE_SUR = new Set([
		'RECONCILIATION_SETTLEMENT_NOT_LINKABLE',
		'NOT_FOUND',
		'RECONCILIATION_TRANSACTION_NOT_PENDING',
	]);

	async function loadCandidates(txId: number): Promise<void> {
		const myGen = ++loadGen;
		loading = true;
		loadError = '';
		selectedId = null;
		try {
			const r = await getLinkCandidates(txId);
			if (myGen !== loadGen) return;
			candidates = r.candidates;
		} catch (e) {
			if (myGen !== loadGen) return;
			candidates = [];
			loadError = errorMessageOf(e);
		} finally {
			if (myGen === loadGen) loading = false;
		}
	}

	// À chaque ouverture, fermeture ou changement de transaction : session neuve ;
	// à l'ouverture, remise à zéro et chargement.
	$effect(() => {
		const ouverte = open ? proposal : null;
		session++;
		submitting = false;
		if (ouverte) {
			errorMsg = '';
			candidates = [];
			void loadCandidates(ouverte.bankTransactionId);
		} else {
			// Un chargement encore en vol ne s'appliquera plus.
			loadGen++;
		}
	});

	const selected = $derived(candidates.find((c) => c.settlementId === selectedId) ?? null);
	const hasGap = $derived(selected !== null && Number(selected.gap) !== 0);

	async function handleConfirm() {
		if (!proposal || selectedId === null) return;
		const txId = proposal.bankTransactionId;
		const mySession = session;
		submitting = true;
		errorMsg = '';
		try {
			await linkTransaction(txId, selectedId);
			if (mySession !== session) {
				// La modale a été fermée pendant l'envoi. Le rattachement a eu lieu :
				// le parent relit sa liste si aucune modale n'est ouverte ; une modale
				// rouverte entre-temps n'est pas touchée (C-29-8-37).
				if (!open) onSuccess();
				return;
			}
			onSuccess();
			onOpenChange(false);
		} catch (e) {
			if (mySession !== session) return;
			errorMsg = errorMessageOf(e);
			if (isApiError(e) && RELIRE_SUR.has(e.code)) {
				await loadCandidates(txId);
			}
		} finally {
			if (mySession === session) submitting = false;
		}
	}
</script>

<Dialog.Root {open} {onOpenChange}>
	<!-- Six colonnes : la largeur par défaut (`sm:max-w-sm`) les écrasait (recette visuelle). -->
	<Dialog.Content class="max-h-[85vh] overflow-y-auto sm:max-w-3xl" data-testid="link-settlement-modal">
		<Dialog.Header>
			<Dialog.Title>
				{i18nMsg('reconciliation-link-modal-title', 'Rattacher à un règlement déjà enregistré')}
			</Dialog.Title>
		</Dialog.Header>

		{#if proposal}
			<div class="text-sm text-text-muted">
				<!-- La date de paiement (valeur, à défaut comptabilisation) : celle d'où se
				     comptent les « jours d'écart » (revue de code P1, A7, C-29-8-39). -->
				<span data-testid="link-tx-summary">
					{proposal.transaction.valueDate ?? proposal.transaction.bookingDate} — {proposal.transaction.amount}
					{proposal.transaction.currency}{#if proposal.transaction.counterpartyName}
						{' '}— {proposal.transaction.counterpartyName}{/if}
				</span>
			</div>

			<p class="mt-2 text-sm" data-testid="link-no-entry">
				{i18nMsg(
					'reconciliation-link-no-entry',
					'Le rattachement ne passe aucune écriture : il relie la transaction au règlement déjà enregistré.',
				)}
			</p>

			{#if loading}
				<p class="mt-2 text-sm text-text-muted" data-testid="link-loading">
					{i18nMsg('reconciliation-link-loading', 'Chargement des règlements…')}
				</p>
			{:else if loadError}
				<div
					class="mt-2 rounded-md border border-destructive bg-destructive/10 px-3 py-2 text-sm text-destructive"
					data-testid="link-load-error"
				>
					{loadError}
				</div>
			{:else if candidates.length === 0}
				<p class="mt-2 text-sm text-text-muted" data-testid="link-empty">
					{i18nMsg(
						'reconciliation-link-empty',
						"Aucun règlement par virement sur ce compte bancaire, au montant de cette transaction, n'attend de transaction. Enregistrez d'abord le règlement depuis la fiche de la facture, sur ce compte bancaire.",
					)}
				</p>
			{:else}
				<div class="mt-2 max-h-80 overflow-x-auto overflow-y-auto">
					<table
						class="w-full table-auto text-sm [&_td]:px-2 [&_td]:py-1 [&_th]:whitespace-nowrap [&_th]:px-2"
						data-testid="link-candidates-table"
					>
						<thead>
							<tr>
								<th class="text-left"><span class="sr-only">{i18nMsg('reconciliation-link-choose', 'Choisir ce règlement')}</span></th>
								<th class="text-left">{i18nMsg('reconciliation-link-col-invoice', 'Facture')}</th>
								<th class="text-left">{i18nMsg('reconciliation-link-col-client', 'Client')}</th>
								<th class="text-left">{i18nMsg('reconciliation-link-col-settled-on', 'Date du règlement')}</th>
								<th class="text-right">{i18nMsg('reconciliation-link-col-amount', 'Montant')}</th>
								<th class="text-right">{i18nMsg('reconciliation-link-col-gap', 'Écart')}</th>
								<th class="text-right">{i18nMsg('reconciliation-link-col-days', "Jours d'écart")}</th>
							</tr>
						</thead>
						<tbody>
							{#each candidates as c (c.settlementId)}
								<tr data-testid="link-candidate-row" data-settlement-id={c.settlementId}>
									<td>
										<input
											type="radio"
											name="link-settlement"
											checked={selectedId === c.settlementId}
											onchange={() => (selectedId = c.settlementId)}
											disabled={submitting}
											aria-label={i18nMsg('reconciliation-link-choose', 'Choisir ce règlement')}
											data-testid="link-candidate-radio"
											data-settlement-id={c.settlementId}
										/>
									</td>
									<td class="whitespace-nowrap">{c.invoiceNumber ?? `#${c.invoiceId}`}</td>
									<td>{c.contactName}</td>
									<td class="whitespace-nowrap">{c.settledOn}</td>
									<td class="text-right">{c.amount}</td>
									<td class="text-right">{c.gap}</td>
									<td class="text-right">{c.daysApart}</td>
								</tr>
							{/each}
						</tbody>
					</table>
				</div>
			{/if}

			{#if hasGap && selected}
				<div
					class="mt-2 rounded-md border border-warning bg-warning/10 px-3 py-2 text-sm"
					data-testid="link-gap-warning"
				>
					{i18nMsg(
						'reconciliation-link-gap-warning',
						"La transaction dépasse ce règlement de { $gap } CHF. L'écart reste au compte de la banque : justifiez-le (remboursement au client, ou écriture manuelle).",
						{ gap: selected.gap },
					)}
				</div>
			{/if}

			{#if errorMsg}
				<div
					class="mt-2 rounded-md border border-destructive bg-destructive/10 px-3 py-2 text-sm text-destructive"
					data-testid="link-error"
				>
					{errorMsg}
				</div>
			{/if}
		{/if}

		<Dialog.Footer>
			<Button
				variant="outline"
				onclick={() => onOpenChange(false)}
				disabled={submitting}
				data-testid="link-cancel"
			>
				{i18nMsg('reconciliation-link-cancel', 'Annuler')}
			</Button>
			<Button
				onclick={handleConfirm}
				disabled={submitting || loading || !!loadError || selectedId === null}
				data-testid="link-submit"
			>
				{i18nMsg('reconciliation-link-submit', 'Rattacher')}
			</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>
