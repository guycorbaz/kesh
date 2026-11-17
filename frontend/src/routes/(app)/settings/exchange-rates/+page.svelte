<!--
  Story 37-2 (AC-6) — *Paramètres → Cours de change*. Tout rôle authentifié lit le
  tableau ; l'Administrateur et le Comptable le modifient (le serveur refuse les
  écritures à Consultation). `?date=AAAA-MM-JJ` (lien du bandeau des corrections)
  choisit l'exercice multi-devises qui couvre cette date.
-->
<script lang="ts">
	import { page } from '$app/state';
	import ExchangeRatesScreen from '$lib/features/exchange-rates/ExchangeRatesScreen.svelte';

	const date = page.url.searchParams.get('date');
	const initialDate = date && /^\d{4}-\d{2}-\d{2}$/.test(date) ? date : null;
</script>

<ExchangeRatesScreen {initialDate} />
