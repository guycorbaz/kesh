<!--
  Story 21-6a (#231, D10) — Badge « rappels suspendus ».

  Teinte NEUTRE volontaire (pas `--color-warning` comme « en retard ») : une
  suspension est une décision délibérée de l'utilisateur, pas une alerte.

  Accessibilité : aria-label explicite — l'état ne repose jamais sur la seule
  couleur (patron PaymentStatusBadge).

  ⚠️ Contraste : le texte est `--color-text` (#1e293b), PAS `--color-text-muted`
  comme le fond. Le patron `PaymentStatusBadge` colore le texte avec la même
  variable que la teinte de fond — sur la variante neutre cela donne
  #64748b sur #e0e3e8 = **3.69:1, sous le minimum AA de 4.5:1** (mesuré par
  axe-core sur ce badge en liste peuplée). Ici : 11.4:1. Ne pas « harmoniser »
  avec le patron sans re-vérifier le contraste.
-->
<script lang="ts">
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';

	let { note = null }: { note?: string | null } = $props();

	let label = $derived(i18nMsg('invoice-paused-badge', 'Suspendu'));
	// La note de suspension n'a aucune autre surface d'affichage en v1 (le
	// toggle et la fiche viennent en 21-6c) — l'infobulle est le seul endroit
	// où l'utilisateur peut lire POURQUOI il a suspendu cette facture.
	let tooltip = $derived(note ? `${label} — ${note}` : label);
</script>

<span
	class="paused inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium"
	aria-label={tooltip}
	title={tooltip}
	data-testid="invoice-paused-badge"
>
	{label}
</span>

<style>
	.paused {
		background-color: color-mix(in srgb, var(--color-text-muted, #6b7280) 20%, transparent);
		color: var(--color-text, #1e293b);
	}
</style>
