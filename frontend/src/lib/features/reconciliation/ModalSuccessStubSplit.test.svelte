<!--
  Story 34-4, revue P1 (E1-3) — enveloppe de `ModalSuccessStub` qui la marque `data-modal="split"` :
  `ReconciliationProposals.test.ts` distingue ainsi les trois modales remplacées.
-->
<script lang="ts">
	import ModalSuccessStub from './ModalSuccessStub.test.svelte';

	type Props = {
		open: boolean;
		onSuccess: () => void;
		bankLedgerAccountId?: number | null;
	};
	let { open, onSuccess, bankLedgerAccountId }: Props = $props();
</script>

<ModalSuccessStub {open} {onSuccess} {bankLedgerAccountId} modal="split" />
