<!--
  Story 29-8 (#670, AC4) — rattacher une transaction bancaire au règlement déjà saisi depuis
  la fiche de la facture.

  Story v017-3a (#671, AC-13) — généralisée, nom de fichier gardé (choix C-v017-3-41) :
  « Rattacher à une écriture existante ». Le dialogue montre deux sections : « Règlements de
  factures clients » (crédits seulement ; la liste, les colonnes et l'écart de la 29-8
  inchangés) et « Écritures passées sur le compte de la banque » (`GET …/entry-candidates`),
  toute transaction en CHF de montant non nul. UN SEUL choix parmi les deux sections, rien de
  présélectionné ; « Rattacher » appelle `…/link` pour un règlement, `…/link-entry` pour une
  écriture.

  Kesh PROPOSE, l'utilisateur CHOISIT et CONFIRME : aucune ligne n'est présélectionnée,
  « Rattacher » reste grisé tant qu'aucun choix n'est fait (règle « Un appariement
  automatique propose, il ne crée jamais », C-29-8-7). Un règlement à écart non nul affiche
  un avertissement AVANT la confirmation : l'écart reste au compte de la banque.

  Une écriture choisie dit ce que deviendrait l'annulation du rattachement — une VÉRITÉ DATÉE,
  jamais une promesse (fiche v017-3a, D12) : sans motif lu, « Aujourd'hui, Kesh ne voit aucun
  obstacle… » ; avec un motif (`cancelBlockedBy`, lu par la fonction de blocage de
  l'annulation), l'introduction « Ce rattachement ne pourra pas être annulé tant que dure
  l'obstacle suivant. » puis le texte du motif (`reconciliationCancelMessage`, réutilisée), et
  une case à cocher sans laquelle « Rattacher » reste inactif.

  Sur `409 RECONCILIATION_SETTLEMENT_NOT_LINKABLE` ou `409 RECONCILIATION_ENTRY_NOT_LINKABLE`
  (la liste a changé entre l'ouverture et le clic), la modale relit LES DEUX listes et remet
  le choix à vide (C-29-8-24). De même sur les deux autres refus d'une course :
  `404 NOT_FOUND` (règlement annulé, écriture supprimée) et
  `404 RECONCILIATION_TRANSACTION_NOT_PENDING` (transaction rapprochée ailleurs) — revue de
  code P1 de la 29-8, B3 (C-29-8-38).

  Garde de session (revue de code P1 de la 29-8, B1/E3, C-29-8-37) : chaque ouverture,
  fermeture ou changement de transaction ouvre une session neuve ; un chargement ou une
  soumission lancés dans une session précédente n'agissent jamais sur la modale courante.
-->
<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import * as Dialog from '$lib/components/ui/dialog';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { isApiError } from '$lib/shared/utils/api-client';
	import { errorMessageOf } from '$lib/shared/utils/error-message';
	import type {
		EntryCandidate,
		LinkCandidate,
		ReconciliationProposal,
	} from './reconciliation.types';
	import {
		getEntryCandidates,
		getLinkCandidates,
		linkEntry,
		linkTransaction,
	} from './reconciliation.api';
	import { reconciliationCancelMessage } from './reconciliation-cancel';

	type Props = {
		open: boolean;
		onOpenChange: (v: boolean) => void;
		proposal: ReconciliationProposal | null;
		onSuccess: () => void;
	};

	/** Le choix de l'utilisateur : un règlement (29-8) ou une écriture (v017-3a), jamais deux. */
	type Choice = { kind: 'settlement'; id: number } | { kind: 'entry'; id: number } | null;

	let { open, onOpenChange, proposal, onSuccess }: Props = $props();

	let candidates = $state<LinkCandidate[]>([]);
	let entries = $state<EntryCandidate[]>([]);
	let choice = $state<Choice>(null);
	/** La case de l'écriture dont l'annulation est empêchée aujourd'hui. */
	let ack = $state(false);
	let loading = $state(false);
	let loadError = $state('');
	let submitting = $state(false);
	let errorMsg = $state('');

	// Garde de génération : une réponse d'un chargement périmé (réouverture, autre
	// transaction, rechargement plus récent) ne remplace pas les listes courantes.
	let loadGen = 0;
	// Garde de session : incrémentée à chaque ouverture, fermeture ou changement de
	// transaction. Ni `$state` ni lue par l'effet : elle ne déclenche rien.
	let session = 0;

	/** Les refus d'une course : une liste a changé, la modale relit les deux. */
	const RELIRE_SUR = new Set([
		'RECONCILIATION_SETTLEMENT_NOT_LINKABLE',
		'RECONCILIATION_ENTRY_NOT_LINKABLE',
		'NOT_FOUND',
		'RECONCILIATION_TRANSACTION_NOT_PENDING',
	]);

	/** Un crédit : seul il a une section « règlements de factures clients » (29-8). */
	function isCredit(p: ReconciliationProposal): boolean {
		return Number(p.transaction.amount) > 0;
	}

	function choose(c: Choice) {
		choice = c;
		ack = false;
	}

	async function loadCandidates(txId: number, credit: boolean): Promise<void> {
		const myGen = ++loadGen;
		loading = true;
		loadError = '';
		choose(null);
		try {
			const [s, e] = await Promise.all([
				credit ? getLinkCandidates(txId) : Promise.resolve(null),
				getEntryCandidates(txId),
			]);
			if (myGen !== loadGen) return;
			candidates = s?.candidates ?? [];
			entries = e.candidates;
		} catch (err) {
			if (myGen !== loadGen) return;
			candidates = [];
			entries = [];
			loadError = errorMessageOf(err);
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
			entries = [];
			void loadCandidates(ouverte.bankTransactionId, isCredit(ouverte));
		} else {
			// Un chargement encore en vol ne s'appliquera plus.
			loadGen++;
		}
	});

	const credit = $derived(proposal !== null && isCredit(proposal));
	const selected = $derived(
		choice?.kind === 'settlement'
			? (candidates.find((c) => c.settlementId === choice?.id) ?? null)
			: null,
	);
	const selectedEntry = $derived(
		choice?.kind === 'entry' ? (entries.find((e) => e.journalEntryId === choice?.id) ?? null) : null,
	);
	const hasGap = $derived(selected !== null && Number(selected.gap) !== 0);
	const blocked = $derived(selectedEntry !== null && selectedEntry.cancelBlockedBy !== null);
	const bothEmpty = $derived(candidates.length === 0 && entries.length === 0);

	async function handleConfirm() {
		if (!proposal || choice === null) return;
		if (blocked && !ack) return;
		const txId = proposal.bankTransactionId;
		const credit = isCredit(proposal);
		const mine = choice;
		const mySession = session;
		submitting = true;
		errorMsg = '';
		try {
			if (mine.kind === 'settlement') {
				await linkTransaction(txId, mine.id);
			} else {
				await linkEntry(txId, mine.id);
			}
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
				await loadCandidates(txId, credit);
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
				{i18nMsg('reconciliation-link-modal-title', 'Rattacher à une écriture existante')}
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
					'Le rattachement ne passe aucune écriture : il relie la transaction à une écriture déjà passée.',
				)}
			</p>

			{#if loading}
				<p class="mt-2 text-sm text-text-muted" data-testid="link-loading">
					{i18nMsg('reconciliation-link-loading', 'Chargement…')}
				</p>
			{:else if loadError}
				<div
					class="mt-2 rounded-md border border-destructive bg-destructive/10 px-3 py-2 text-sm text-destructive"
					data-testid="link-load-error"
				>
					{loadError}
				</div>
			{:else if credit && bothEmpty}
				<p class="mt-2 text-sm text-text-muted" data-testid="link-empty">
					{i18nMsg(
						'reconciliation-link-empty',
						"Aucun règlement de facture client ni aucune écriture du compte de la banque ne correspond à cette transaction. Enregistrez d'abord le règlement depuis la fiche de la facture, ou saisissez l'écriture, puis revenez ici.",
					)}
				</p>
			{:else if !credit && entries.length === 0}
				<p class="mt-2 text-sm text-text-muted" data-testid="link-entries-empty-debit">
					{i18nMsg(
						'reconciliation-link-entries-empty-debit',
						"Aucune écriture du compte de la banque ne correspond à ce débit. S'il est déjà comptabilisé par un autre chemin, rejetez-le plutôt que de l'affecter.",
					)}
				</p>
			{:else}
				{#if credit}
					<section class="mt-3" data-testid="link-settlements-section">
						<h3 class="text-sm font-semibold">
							{i18nMsg('reconciliation-link-section-settlements', 'Règlements de factures clients')}
						</h3>
						{#if candidates.length === 0}
							<p class="mt-1 text-sm text-text-muted" data-testid="link-settlements-empty">
								{i18nMsg(
									'reconciliation-link-settlements-empty',
									'Aucun règlement de facture client ne correspond.',
								)}
							</p>
						{:else}
							<div class="mt-1 max-h-60 overflow-x-auto overflow-y-auto">
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
														name="link-choice"
														checked={choice?.kind === 'settlement' && choice.id === c.settlementId}
														onchange={() => choose({ kind: 'settlement', id: c.settlementId })}
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
					</section>
				{/if}

				<section class="mt-3" data-testid="link-entries-section">
					<h3 class="text-sm font-semibold">
						{i18nMsg(
							'reconciliation-link-section-entries',
							'Écritures passées sur le compte de la banque',
						)}
					</h3>
					{#if entries.length === 0}
						<p class="mt-1 text-sm text-text-muted" data-testid="link-entries-empty">
							{i18nMsg(
								'reconciliation-link-entries-empty',
								'Aucune écriture du compte de la banque ne correspond.',
							)}
						</p>
					{:else}
						<div class="mt-1 max-h-60 overflow-x-auto overflow-y-auto">
							<table
								class="w-full table-auto text-sm [&_td]:px-2 [&_td]:py-1 [&_th]:whitespace-nowrap [&_th]:px-2"
								data-testid="entry-candidates-table"
							>
								<thead>
									<tr>
										<th class="text-left"><span class="sr-only">{i18nMsg('reconciliation-link-choose-entry', 'Choisir cette écriture')}</span></th>
										<th class="text-left">{i18nMsg('reconciliation-link-col-entry-number', 'N°')}</th>
										<th class="text-left">{i18nMsg('reconciliation-link-col-entry-date', "Date de l'écriture")}</th>
										<th class="text-left">{i18nMsg('reconciliation-link-col-journal', 'Journal')}</th>
										<th class="text-left">{i18nMsg('reconciliation-link-col-description', 'Libellé')}</th>
										<th class="text-right">{i18nMsg('reconciliation-link-col-amount', 'Montant')}</th>
										<th class="text-right">{i18nMsg('reconciliation-link-col-days', "Jours d'écart")}</th>
									</tr>
								</thead>
								<tbody>
									{#each entries as e (e.journalEntryId)}
										<tr data-testid="entry-candidate-row" data-entry-id={e.journalEntryId}>
											<td>
												<input
													type="radio"
													name="link-choice"
													checked={choice?.kind === 'entry' && choice.id === e.journalEntryId}
													onchange={() => choose({ kind: 'entry', id: e.journalEntryId })}
													disabled={submitting}
													aria-label={i18nMsg('reconciliation-link-choose-entry', 'Choisir cette écriture')}
													data-testid="entry-candidate-radio"
													data-entry-id={e.journalEntryId}
												/>
											</td>
											<td class="whitespace-nowrap">{e.entryNumber}</td>
											<td class="whitespace-nowrap">{e.entryDate}</td>
											<td class="whitespace-nowrap">{e.journal}</td>
											<td>{e.description}</td>
											<td class="text-right">{e.amount}</td>
											<td class="text-right">{e.daysApart}</td>
										</tr>
									{/each}
								</tbody>
							</table>
						</div>
					{/if}
				</section>
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

			{#if selectedEntry && selectedEntry.cancelBlockedBy === null}
				<p class="mt-2 text-sm" data-testid="entry-cancel-today">
					{i18nMsg(
						'reconciliation-link-entry-cancel-today',
						"Aujourd'hui, Kesh ne voit aucun obstacle : annuler ce rapprochement contre-passerait cette écriture (ou annulerait le règlement qu'elle porte). Une clôture d'exercice, une période verrouillée ou un exercice postérieur clos pourraient l'empêcher, dès aujourd'hui sans que Kesh le voie, ou plus tard.",
					)}
				</p>
			{:else if selectedEntry && selectedEntry.cancelBlockedBy !== null}
				<div
					class="mt-2 rounded-md border border-warning bg-warning/10 px-3 py-2 text-sm"
					data-testid="entry-candidate-cancel-blocked"
				>
					<p data-testid="entry-cancel-blocked-intro">
						{i18nMsg(
							'reconciliation-link-entry-cancel-blocked-intro',
							"Ce rattachement ne pourra pas être annulé tant que dure l'obstacle suivant.",
						)}
					</p>
					<p class="mt-1" data-testid="entry-cancel-blocked-reason">
						{reconciliationCancelMessage(
							selectedEntry.cancelBlockedBy,
							selectedEntry.cancelBlockedLabel,
						)}
					</p>
					<label class="mt-2 flex items-start gap-2">
						<input
							type="checkbox"
							bind:checked={ack}
							disabled={submitting}
							data-testid="entry-cancel-blocked-ack"
						/>
						<span>
							{i18nMsg(
								'reconciliation-link-entry-cancel-blocked-ack',
								"J'ai compris : tant que cet obstacle dure, ce rattachement ne pourra pas être annulé.",
							)}
						</span>
					</label>
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
				disabled={submitting || loading || !!loadError || choice === null || (blocked && !ack)}
				data-testid="link-submit"
			>
				{i18nMsg('reconciliation-link-submit', 'Rattacher')}
			</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>
