<!--
  Story 43-3 (#823) — enveloppe de `ModalSuccessStub` qui la marque `data-modal="group"` :
  `ReconciliationProposals.test.ts` remplace ainsi `GroupSettlementModal`, testée à part
  (`GroupSettlementModal.test.ts`).

  Story 48-14 (#1262, T5) : le bouton « arrière-plan » (`stub-group-background`) appelle
  `onSettledInBackground` — un règlement parti d'une fenêtre du groupe fermée pendant l'envoi ;
  il est rendu **hors** de `{#if open}`, puisque ce règlement aboutit pendant qu'une autre
  modale, ou aucune, est ouverte. Sous `{#if open}`, deux boutons rejouent l'ordre réel des
  appels de la modale : `stub-group-success-then-close` (`onSuccess(); onOpenChange(false);`,
  le succès) et `stub-group-close-then-stale` (`onOpenChange(false); onStale();`, `closeStale`).
  `globalThis.__stubGroupBackground` garde `onSettledInBackground` après le démontage.
-->
<script lang="ts">
	import ModalSuccessStub from './ModalSuccessStub.test.svelte';

	type Props = {
		open: boolean;
		onSuccess: () => void;
		onOpenChange?: (v: boolean) => void;
		onStale?: () => void;
		onSettledInBackground?: () => void;
		bankLedgerAccountId?: number | null;
	};
	let { open, onSuccess, onOpenChange, onStale, onSettledInBackground, bankLedgerAccountId }: Props =
		$props();

	// Reprise de la revue P1 (48-14) : la fin d'un envoi survit au démontage de la modale (toute
	// relecture la démonte, `{#if loading}`). La doublure garde donc son `onSettledInBackground`
	// hors du DOM, pour qu'un test le rappelle pendant une relecture en vol.
	$effect(() => {
		(globalThis as { __stubGroupBackground?: () => void }).__stubGroupBackground =
			onSettledInBackground;
	});
</script>

<ModalSuccessStub {open} {onSuccess} {onOpenChange} {bankLedgerAccountId} modal="group" />
<button type="button" data-testid="stub-group-background" onclick={() => onSettledInBackground?.()}
	>arrière-plan</button
>
{#if open}
	<button
		type="button"
		data-testid="stub-group-success-then-close"
		onclick={() => {
			onSuccess();
			onOpenChange?.(false);
		}}>succès puis fermer</button
	>
	<button
		type="button"
		data-testid="stub-group-close-then-stale"
		onclick={() => {
			onOpenChange?.(false);
			onStale?.();
		}}>fermer puis périmée</button
	>
{/if}
