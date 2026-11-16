<!--
  Story 34-4 (#828, AC-6) — confirmation avant un geste sur une transaction signalée « déjà
  comptabilisée » (`already-booked.ts`) : un geste qui passerait une écriture et, depuis la
  Story 35-4 (#1023), *Rejeter*.

  Ouvert par *Accepter (N)* ou *Rejeter* quand la sélection contient au moins une ligne
  signalée, et par *Écriture manuelle* ou *Éclater* sur une ligne signalée. Il nomme chaque
  transaction signalée (date suisse, montant, contrepartie) et dit ce que fait le geste :
  - `gesture = 'write'` : une écriture serait passée pour un mouvement peut-être déjà aux
    livres ; *Continuer quand même* ;
  - `gesture = 'reject'` (Story 35-4, D2) : le rejet retire les `selectedCount` lignes cochées
    de la liste sans écriture — un mouvement réel qui n'est pas aux livres y manquerait —, il
    vaut mieux rattacher une ligne déjà aux livres, et un rejet s'annule depuis le détail de
    son import ; *Rejeter quand même*. Il ne nomme pas l'écriture ressemblante (#1052).
  Le geste reprend tel quel, ou *Annuler* ne fait rien. Aucun refus : le serveur accepte le
  geste confirmé comme avant. *Rattacher à une écriture existante*, que l'avertissement
  recommande, ne passe pas par lui (choix C-e342-8).

  Contrat : `open`, `transactions` (les lignes signalées concernées), `gesture`,
  `selectedCount` (les lignes cochées, toutes, pour *Rejeter*), `onContinue()`, `onCancel()` ;
  fermer le dialogue (Échap, clic hors, X) vaut *Annuler*.
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
		/** Le geste confirmé : une écriture (*Accepter*, *Écriture manuelle*, *Éclater*) ou un rejet. */
		gesture: 'write' | 'reject';
		/** Pour *Rejeter* : le nombre de lignes cochées, signalées ou non (le lot entier part). */
		selectedCount: number;
		onContinue: () => void;
		onCancel: () => void;
	};
	let { open, transactions, gesture, selectedCount, onContinue, onCancel }: Props = $props();
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

		<p class="text-sm" data-testid="already-booked-confirm-intro">
			{#if gesture === 'reject'}
				{#if selectedCount === 1}
					{i18nMsg(
						'reconciliation-labels-already-booked-confirm-intro-reject-one',
						'Chaque transaction ci-dessous semble déjà comptabilisée. Rejeter retire la transaction cochée de la liste sans passer d’écriture : si elle est un mouvement réel qui n’est pas encore aux livres, il y manquera ; si elle y est déjà, rattachez-la plutôt (« Rattacher à une écriture existante »). Un rejet s’annule depuis le détail de son import (« Annuler le rejet »).',
					)}
				{:else}
					{i18nMsg(
						'reconciliation-labels-already-booked-confirm-intro-reject-other',
						'Chaque transaction ci-dessous semble déjà comptabilisée. Rejeter retire les { $count } transactions cochées de la liste sans passer d’écriture : si l’une est un mouvement réel qui n’est pas encore aux livres, il y manquera ; si elle y est déjà, rattachez-la plutôt (« Rattacher à une écriture existante »). Un rejet s’annule depuis le détail de son import (« Annuler le rejet »).',
						{ count: selectedCount },
					)}
				{/if}
			{:else}
				{i18nMsg(
					'reconciliation-labels-already-booked-confirm-intro',
					'Chaque transaction ci-dessous semble déjà comptabilisée (voir l’avertissement de sa ligne). Continuer passerait une écriture pour un mouvement qui est peut-être déjà aux livres : il serait compté deux fois.',
				)}
			{/if}
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
				{#if gesture === 'reject'}
					{i18nMsg('reconciliation-labels-already-booked-confirm-reject', 'Rejeter quand même')}
				{:else}
					{i18nMsg('reconciliation-labels-already-booked-confirm-continue', 'Continuer quand même')}
				{/if}
			</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>
