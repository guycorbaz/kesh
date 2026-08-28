<!--
  Story 5.4 — Badge statut paiement.
  Couleurs design tokens : vert = payée, gris = impayée, orange = en retard.
  Accessibilité : aria-label explicite (pas uniquement couleur), contraste AA.
-->
<script lang="ts">
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';

	/**
	 * ⚠️ `partial` (Story 24-2, #371) est **dérivé**, comme les trois autres :
	 * aucun état nouveau en base. `chk_invoices_status` n'autorise toujours que
	 * `draft / validated / cancelled` — « payée » se lit déjà sur `paidAt`, et
	 * « partielle » se lit sur « réglé > 0 alors que le solde n'est pas nul ».
	 */
	type PaymentStatus = 'paid' | 'unpaid' | 'overdue' | 'partial';

	let { status }: { status: PaymentStatus } = $props();

	const LABEL_KEY: Record<PaymentStatus, string> = {
		paid: 'payment-status-paid',
		unpaid: 'payment-status-unpaid',
		overdue: 'payment-status-overdue',
		partial: 'payment-status-partial',
	};
	const LABEL_FALLBACK: Record<PaymentStatus, string> = {
		paid: 'Payée',
		unpaid: 'Impayée',
		overdue: 'En retard',
		partial: 'Partiellement payée',
	};

	let label = $derived(i18nMsg(LABEL_KEY[status], LABEL_FALLBACK[status]));
</script>

<span
	class="inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium"
	class:paid={status === 'paid'}
	class:unpaid={status === 'unpaid'}
	class:overdue={status === 'overdue'}
	class:partial={status === 'partial'}
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
	/*
	 * Story 24-2 — « partiellement payée » : un état intermédiaire, ni le vert
	 * de l'acquitté ni l'orange du retard.
	 *
	 * ⚠️ Avant-plan = texte primaire, tint neutre à 20 % — le patron AA-safe
	 * imposé par #256 sur `.unpaid`, et pour la même raison : une couleur
	 * d'accent sur son propre tint passe sous 4.5:1 en texte xs. Ne PAS
	 * reprendre le patron de `.paid`/`.overdue` ici.
	 */
	.partial {
		background-color: color-mix(in srgb, var(--color-primary, #4f46e5) 18%, transparent);
		color: var(--color-text, #1e293b);
	}
</style>
