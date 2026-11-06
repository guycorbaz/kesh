<svelte:options namespace="svg" />

<script lang="ts">
	// Story 33-3b (AC14, C-e332-17) — les motifs d'un graphique, dans son `<defs>`.
	// Le trait porte la couleur : c'est sur elle que se contrôle le contraste.
	import { patternId, type ChartStyle } from './homepage-charts';

	interface Props {
		prefix: string;
		styles: readonly ChartStyle[];
		/** Côté de la maille, en unités du `viewBox`. */
		size?: number;
	}
	let { prefix, styles, size = 8 }: Props = $props();
	let stroke = $derived(size * 0.38);
</script>

<defs>
	{#each styles.filter((s) => s.pattern !== 'solid') as style (style.pattern)}
		<!-- Une hachure est une ligne droite dans une maille tournée : elle se raccorde d'une
		     maille à l'autre, ce qu'une ligne tournée dans une maille droite ne fait pas. -->
		<pattern
			id={patternId(prefix, style)}
			width={size}
			height={size}
			patternUnits="userSpaceOnUse"
			patternTransform={style.pattern === 'diagonal' ? 'rotate(45)' : style.pattern === 'reverse-diagonal' ? 'rotate(-45)' : undefined}
		>
			<rect width={size} height={size} fill="#ffffff" />
			{#if style.pattern === 'dots'}
				<circle cx={size / 2} cy={size / 2} r={size * 0.3} fill={style.color} />
			{:else if style.pattern === 'horizontal' || style.pattern === 'diagonal' || style.pattern === 'reverse-diagonal'}
				<line x1="0" y1={size / 2} x2={size} y2={size / 2} stroke={style.color} stroke-width={stroke} />
			{:else if style.pattern === 'cross'}
				<line x1="0" y1={size / 2} x2={size} y2={size / 2} stroke={style.color} stroke-width={stroke * 0.7} />
				<line x1={size / 2} y1="0" x2={size / 2} y2={size} stroke={style.color} stroke-width={stroke * 0.7} />
			{/if}
		</pattern>
	{/each}
</defs>
