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
		<pattern id={patternId(prefix, style)} width={size} height={size} patternUnits="userSpaceOnUse">
			<rect width={size} height={size} fill="#ffffff" />
			{#if style.pattern === 'diagonal' || style.pattern === 'reverse-diagonal'}
				<g transform="rotate({style.pattern === 'diagonal' ? 45 : -45} {size / 2} {size / 2})">
					<line x1={-size} y1={size / 2} x2={size * 2} y2={size / 2} stroke={style.color} stroke-width={stroke} />
					<line x1={-size} y1={size / 2 - size} x2={size * 2} y2={size / 2 - size} stroke={style.color} stroke-width={stroke} />
					<line x1={-size} y1={size / 2 + size} x2={size * 2} y2={size / 2 + size} stroke={style.color} stroke-width={stroke} />
				</g>
			{:else if style.pattern === 'dots'}
				<circle cx={size / 2} cy={size / 2} r={size * 0.3} fill={style.color} />
			{:else if style.pattern === 'horizontal'}
				<line x1="0" y1={size / 2} x2={size} y2={size / 2} stroke={style.color} stroke-width={stroke} />
			{:else if style.pattern === 'cross'}
				<line x1="0" y1={size / 2} x2={size} y2={size / 2} stroke={style.color} stroke-width={stroke * 0.7} />
				<line x1={size / 2} y1="0" x2={size / 2} y2={size} stroke={style.color} stroke-width={stroke * 0.7} />
			{/if}
		</pattern>
	{/each}
</defs>
