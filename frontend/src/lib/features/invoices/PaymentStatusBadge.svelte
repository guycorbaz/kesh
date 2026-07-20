<!--
  Story 5.4 — Badge statut paiement.
  Couleurs design tokens : vert = payée, gris = impayée, orange = en retard.
  Accessibilité : aria-label explicite (pas uniquement couleur), contraste AA.
-->
<script lang="ts">
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';

	type PaymentStatus = 'paid' | 'unpaid' | 'overdue';

	let { status }: { status: PaymentStatus } = $props();

	const LABEL_KEY: Record<PaymentStatus, string> = {
		paid: 'payment-status-paid',
		unpaid: 'payment-status-unpaid',
		overdue: 'payment-status-overdue',
	};
	const LABEL_FALLBACK: Record<PaymentStatus, string> = {
		paid: 'Payée',
		unpaid: 'Impayée',
		overdue: 'En retard',
	};

	let label = $derived(i18nMsg(LABEL_KEY[status], LABEL_FALLBACK[status]));
</script>

<span
	class="inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium"
	class:paid={status === 'paid'}
	class:unpaid={status === 'unpaid'}
	class:overdue={status === 'overdue'}
	aria-label={label}
>
	{label}
</span>

<style>
	.paid {
		background-color: color-mix(in srgb, var(--color-success, #10b981) 15%, transparent);
		color: var(--color-success, #10b981);
	}
	/*
	 * #256 : `--color-text-muted` en avant-plan sur son propre tint 15 % passait
	 * sous le ratio AA (4.5:1) pour du texte xs. On adopte le patron AA-safe déjà
	 * utilisé par `DunningPausedBadge` : avant-plan = texte primaire (theme-aware),
	 * tint neutre à 20 %. Reste visuellement « neutre/inactif » vs le vert/orange.
	 */
	.unpaid {
		background-color: color-mix(in srgb, var(--color-text-muted, #64748b) 20%, transparent);
		color: var(--color-text, #1e293b);
	}
	.overdue {
		background-color: color-mix(in srgb, var(--color-warning, #f59e0b) 20%, transparent);
		color: var(--color-warning, #f59e0b);
	}
</style>
