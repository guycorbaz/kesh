<!--
  Story 21-6b (#231) — Badge « dernier niveau atteint » (facture terminale).

  Une facture au dernier niveau de rappel reste visible (jamais de disparition
  silencieuse, D9) : poursuite à envisager. Teinte d'alerte (`--color-warning`),
  aria-label explicite.
-->
<script lang="ts">
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';

	let label = $derived(i18nMsg('reminders-badge-terminal', 'Dernier niveau atteint'));
</script>

<span
	class="terminal inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium"
	aria-label={label}
	data-testid="reminder-terminal-badge"
>
	{label}
</span>

<style>
	.terminal {
		/* Fond ambre pâle + texte foncé du thème → contraste AA garanti (le token
		   `--color-warning` seul en texte serait sous AA, cf. leçon 21-6a). */
		background-color: color-mix(in srgb, var(--color-warning, #d97706) 22%, transparent);
		color: var(--color-text, #1e293b);
	}
</style>
