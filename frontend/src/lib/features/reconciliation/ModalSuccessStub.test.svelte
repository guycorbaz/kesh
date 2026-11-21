<!--
  Doublure de test (revue de code P1 de la 15-5c, B1/E3) — remplace `ManualMatchModal` et
  `TransactionSplitModal` dans `ReconciliationProposals.test.ts` : un bouton qui appelle
  `onSuccess`, pour exercer `onManualSuccess` / `onSplitSuccess` sans piloter l'autocomplétion
  des comptes. Le suffixe `.test.` la tient hors des relevés i18n et hors du build.

  Story 15-6d (test 11, choix C-15-6d-1) : la doublure expose aussi la prop
  `bankLedgerAccountId` qu'elle reçoit (`data-bank-ledger-account-id`, `"undefined"` si le
  parent ne la passe pas — cas du rattachement), pour asserter le câblage parent → modale ;
  depuis la Story 34-3 (AC-3), l'écriture manuelle et l'éclatement la reçoivent tous deux. Le
  filtre lui-même est testé dans `ManualMatchModal.test.ts`, `TransactionSplitModal.test.ts`
  et `reconciliation-counterparties.test.ts`.

  Story 34-4, revue P1 (E1-3, A1-2) : `modal` nomme la modale que la doublure remplace
  (`data-modal`), posée par les trois enveloppes `ModalSuccessStub{Manual,Split,Link}.test.svelte` —
  un test peut ainsi asserter que *Continuer quand même* rouvre **le** geste demandé.

  Story 48-14 (#1262, T5) : `onOpenChange`, que le parent passe aux quatre modales, est transmis ;
  le bouton « fermer » (`stub-modal-close`) l'appelle avec `false`, comme *Annuler* ou Échap
  d'une vraie modale — sans relecture de la modale elle-même.
-->
<script lang="ts">
	type Props = {
		open: boolean;
		onSuccess: () => void;
		onOpenChange?: (v: boolean) => void;
		bankLedgerAccountId?: number | null;
		modal?: string;
	};
	let { open, onSuccess, onOpenChange, bankLedgerAccountId, modal }: Props = $props();
</script>

{#if open}
	<button
		type="button"
		data-testid="stub-modal-success"
		data-modal={modal}
		data-bank-ledger-account-id={String(bankLedgerAccountId)}
		onclick={() => onSuccess()}>ok</button
	>
	<button
		type="button"
		data-testid="stub-modal-close"
		data-modal={modal}
		onclick={() => onOpenChange?.(false)}>fermer</button
	>
{/if}
