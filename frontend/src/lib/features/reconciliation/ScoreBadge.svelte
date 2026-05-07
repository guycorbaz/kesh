<!--
  Story 8-4 — Badge de score de matching (vert ≥0.85, jaune 0.60-0.85, rouge <0.60).
  Indicateur visuel du niveau de confiance d'une proposition de réconciliation.
-->
<script lang="ts">
	type Props = {
		score: number;
	};
	let { score }: Props = $props();

	const tier = $derived(
		score >= 0.85 ? 'high' : score >= 0.6 ? 'medium' : 'low',
	);
	const colorClass = $derived(
		tier === 'high'
			? 'bg-green-100 text-green-800'
			: tier === 'medium'
				? 'bg-yellow-100 text-yellow-800'
				: 'bg-red-100 text-red-800',
	);
	const percent = $derived(Math.round(score * 100));
</script>

<span
	class="inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium {colorClass}"
	data-testid="score-badge"
	data-score-tier={tier}
>
	{percent}%
</span>
