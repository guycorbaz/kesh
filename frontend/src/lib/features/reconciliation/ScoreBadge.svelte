<!--
  Story 8-4 — Badge de score de matching (vert ≥0.90, jaune 0.70-0.90, rouge <0.70).
  Indicateur visuel du niveau de confiance d'une proposition de réconciliation.
  H3 Pass 1 code review : seuils 0.90/0.70 alignés sur §matching-algo
  (cf. spec L25 IEEE 754 note — combinaison 0.50 amount + 0.40 reference
  produit 0.90 exactement en f64 pour le cas nominal QR-Bill).
-->
<script lang="ts">
	type Props = {
		score: number;
	};
	let { score }: Props = $props();

	const tier = $derived(
		score >= 0.9 ? 'high' : score >= 0.7 ? 'medium' : 'low',
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
