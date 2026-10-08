<!--
  Doublure de test (revue de code P1 de la 15-5c, B1/E3) — remplace `ManualMatchModal` et
  `TransactionSplitModal` dans `ReconciliationProposals.test.ts` : un bouton qui appelle
  `onSuccess`, pour exercer `onManualSuccess` / `onSplitSuccess` sans piloter l'autocomplétion
  des comptes. Le suffixe `.test.` la tient hors des relevés i18n et hors du build.
-->
<script lang="ts">
	type Props = { open: boolean; onSuccess: () => void };
	let { open, onSuccess }: Props = $props();
</script>

{#if open}
	<button type="button" data-testid="stub-modal-success" onclick={() => onSuccess()}>ok</button>
{/if}
