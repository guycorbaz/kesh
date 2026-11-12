<!--
  Story 34-4 (#828, AC-6) — confirmation avant un geste qui passerait une écriture pour une
  transaction signalée « déjà comptabilisée » (`already-booked.ts`).

  Ouvert par *Accepter (N)* quand la sélection contient au moins une ligne signalée, et par
  *Écriture manuelle* ou *Éclater* sur une ligne signalée. Il nomme chaque transaction signalée
  (date suisse, montant, contrepartie) et propose *Continuer quand même* — le geste reprend tel
  quel — ou *Annuler*, qui ne fait rien. Aucun refus : le serveur accepte le geste confirmé
  comme avant (AC-4). *Rejeter* et *Rattacher à une écriture existante*, que l'avertissement
  recommande, ne passent pas par lui (choix C-e342-8).

  Contrat : `open`, `transactions` (les lignes signalées concernées), `onContinue()`,
  `onCancel()` ; fermer le dialogue (Échap, clic hors) vaut *Annuler*.
-->
<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import * as Dialog from '$lib/components/ui/dialog';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { describeTransaction } from './already-booked';
	import type { ReconciliationProposal } from './reconciliation.types';

	type Props = {
		open: boolean;
		transactions: ReconciliationProposal[];
		onContinue: () => void;
		onCancel: () => void;
	};
	let { open, transactions, onContinue, onCancel }: Props = $props();
</script>

<Dialog.Root {open} onOpenChange={(v) => !v && onCancel()}>
	<Dialog.Content data-testid="already-booked-confirm">
		<Dialog.Header>
			<Dialog.Title>
				{i18nMsg(
					'reconciliation-labels-already-booked-confirm-title',
					'Confirmer malgré l’avertissement',
				)}
			</Dialog.Title>
		</Dialog.Header>

		<p class="text-sm">
			{i18nMsg(
				'reconciliation-labels-already-booked-confirm-intro',
				'Chaque transaction ci-dessous semble déjà comptabilisée (voir l’avertissement de sa ligne). Continuer passerait une écriture pour un mouvement qui est peut-être déjà aux livres : il serait compté deux fois.',
			)}
		</p>
		<ul class="list-disc pl-6 text-sm">
			{#each transactions as p (p.bankTransactionId)}
				<li data-testid="already-booked-confirm-item" data-tx-id={p.bankTransactionId}>
					{describeTransaction(p.transaction)}
				</li>
			{/each}
		</ul>

		<Dialog.Footer>
			<Button variant="outline" onclick={onCancel} data-testid="already-booked-confirm-cancel">
				{i18nMsg('reconciliation-labels-already-booked-confirm-cancel', 'Annuler')}
			</Button>
			<Button onclick={onContinue} data-testid="already-booked-confirm-continue">
				{i18nMsg('reconciliation-labels-already-booked-confirm-continue', 'Continuer quand même')}
			</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>
