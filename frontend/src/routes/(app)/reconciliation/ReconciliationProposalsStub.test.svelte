<!-- Story 38-11d — remplaçant de `ReconciliationProposals` pour le test de la page. -->
<script lang="ts">
	let { bankAccountId }: { bankAccountId: number } = $props();
</script>

<div data-testid="proposals-stub">{bankAccountId}</div>
