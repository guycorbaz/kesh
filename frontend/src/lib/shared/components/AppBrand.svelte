<script lang="ts">
	/**
	 * Le logo « Kesh » et, dessous, le nom de la comptabilité ouverte — #1582 (D3, D4).
	 *
	 * Le nom est dans le lien du logo, sur deux éléments `div` (blocs : l'espace entre les deux
	 * mots du nom accessible, « Kesh Dupont Sàrl », et l'effet de `max-w-56` / `truncate`).
	 * Un nom long est coupé par la CSS : le texte complet reste dans le DOM et dans `title`.
	 * `name` nul ou vide : le logo seul.
	 */
	let { name }: { name: string | null } = $props();
</script>

<a href="/" class="flex flex-col items-start font-semibold text-primary">
	<div class="text-xl">Kesh</div>
	{#if name}
		<div
			class="max-w-56 truncate text-xs font-normal text-text"
			title={name}
			data-testid="company-name"
		>
			{name}
		</div>
	{/if}
</a>
