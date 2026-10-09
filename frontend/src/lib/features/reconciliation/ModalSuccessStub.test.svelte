<!--
  Doublure de test (revue de code P1 de la 15-5c, B1/E3) — remplace `ManualMatchModal` et
  `TransactionSplitModal` dans `ReconciliationProposals.test.ts` : un bouton qui appelle
  `onSuccess`, pour exercer `onManualSuccess` / `onSplitSuccess` sans piloter l'autocomplétion
  des comptes. Le suffixe `.test.` la tient hors des relevés i18n et hors du build.

  Story 15-6d (test 11, choix C-15-6d-1) : la doublure expose aussi la prop
  `bankLedgerAccountId` qu'elle reçoit (`data-bank-ledger-account-id`, `"undefined"` si le
  parent ne la passe pas — cas de l'éclatement), pour asserter le câblage parent → modale ;
  le filtre lui-même est testé dans `ManualMatchModal.test.ts`.
-->
<script lang="ts">
	type Props = {
		open: boolean;
		onSuccess: () => void;
		bankLedgerAccountId?: number | null;
	};
	let { open, onSuccess, bankLedgerAccountId }: Props = $props();
</script>

{#if open}
	<button
		type="button"
		data-testid="stub-modal-success"
		data-bank-ledger-account-id={String(bankLedgerAccountId)}
		onclick={() => onSuccess()}>ok</button
	>
{/if}
