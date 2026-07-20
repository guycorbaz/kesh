<!--
  Story 21-6b (#231) — Badge « sans e-mail » (niveau contact).

  Un contact sans e-mail ne peut pas recevoir de rappel par e-mail (le rappel
  papier reste possible). Teinte neutre, aria-label explicite.

  ⚠️ Contraste : texte en `--color-text` (pas `--color-text-muted` comme le fond).
  Le gabarit `PaymentStatusBadge` colore texte ET fond avec la même variable →
  sous AA (mesuré 3.69:1 en 21-6a). Ne pas « harmoniser » sans re-mesurer.
-->
<script lang="ts">
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';

	let label = $derived(i18nMsg('reminders-badge-no-email', 'sans e-mail'));
</script>

<span
	class="no-email inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium"
	aria-label={label}
	data-testid="reminder-no-email-badge"
>
	{label}
</span>

<style>
	.no-email {
		background-color: color-mix(in srgb, var(--color-text-muted, #6b7280) 20%, transparent);
		color: var(--color-text, #1e293b);
	}
</style>
