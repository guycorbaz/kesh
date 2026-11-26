<!-- Story 53-1a — remplaçant de test de `VatPurchaseAssistant` : un bouton qui rend un résultat fixe. -->
<script lang="ts">
	let { onApply }: { onApply: (r: { lines: unknown[]; description: string }) => void } = $props();
</script>

<button
	type="button"
	data-testid="stub-vat-apply"
	onclick={() =>
		onApply({
			lines: [
				{ accountId: 1, debit: '50.00', credit: '', projectId: null },
				{ accountId: 2, debit: '', credit: '50.00', projectId: null }
			],
			description: 'Assistant'
		})}>apply</button
>
