<!--
  Story 43-3 (#823) — enveloppe de `ModalSuccessStub` qui la marque `data-modal="group"` :
  `ReconciliationProposals.test.ts` remplace ainsi `GroupSettlementModal`, testée à part
  (`GroupSettlementModal.test.ts`).
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

<ModalSuccessStub {open} {onSuccess} {bankLedgerAccountId} modal="group" />
