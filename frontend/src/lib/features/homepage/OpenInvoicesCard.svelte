<script lang="ts">
	// Story 25-6-a (#388) — la tuile « Factures ouvertes », chiffrée et pour TOUS
	// les rôles. Elle n'affichait qu'un nombre de factures à rappeler, et
	// seulement pour Admin/Comptable : un rôle Consultation voyait « Aucune
	// facture ouverte » en permanence. Les montants sont des RESTES DUS (le
	// résumé de l'échéancier, 25-4-b1).
	import Big from 'big.js';
	import { Button } from '$lib/components/ui/button';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { formatSwissAmount } from '$lib/features/journal-entries/balance';
	import type { DueDatesSummary } from '$lib/features/invoices/invoices.types';

	interface Props {
		state: 'loading' | 'ready' | 'error';
		summary: DueDatesSummary | null;
		canManage: boolean;
		isGuided: boolean;
		/** Nombre de factures à rappeler (Admin/Comptable seulement). */
		reminderCount: number;
	}
	let { state, summary, canManage, isGuided, reminderCount }: Props = $props();

	let amount = (v: string) => formatSwissAmount(new Big(v));
</script>

<div class="rounded-lg border border-border bg-white p-6 shadow-sm" data-testid="homepage-card-open-invoices">
	<h2 class="text-lg font-semibold text-text">
		{i18nMsg('homepage-invoices-title', 'Factures ouvertes')}
	</h2>
	{#if state === 'error'}
		<p class="mt-2 text-sm text-text-muted" data-testid="homepage-invoices-unavailable">
			{i18nMsg('homepage-invoices-unavailable', 'Factures indisponibles pour le moment.')}
		</p>
	{:else if state === 'ready' && summary && summary.unpaidCount > 0}
		<p
			class="mt-2 text-sm font-medium"
			data-testid="homepage-invoices-open-count"
			data-amount={summary.unpaidTotal}
		>
			{i18nMsg('homepage-invoices-open', '{ $n } facture(s) ouverte(s) — { $amount }', {
				n: summary.unpaidCount,
				amount: amount(summary.unpaidTotal),
			})}
		</p>
		{#if summary.overdueCount > 0}
			<p class="mt-1 text-sm text-red-700" data-testid="homepage-invoices-overdue">
				{i18nMsg('homepage-invoices-overdue', 'dont { $n } échue(s) — { $amount }', {
					n: summary.overdueCount,
					amount: amount(summary.overdueTotal),
				})}
			</p>
		{/if}
	{:else if state === 'ready'}
		<p class="mt-2 text-sm text-text-muted">
			{#if isGuided && canManage}
				{i18nMsg(
					'homepage-invoices-empty-guided',
					'Aucune facture ouverte. Créez une facture pour facturer vos clients.',
				)}
			{:else}
				{i18nMsg('homepage-invoices-empty', 'Aucune facture ouverte.')}
			{/if}
		</p>
	{/if}
	{#if canManage && reminderCount > 0}
		<!-- Story 21-6c (D-c2) : N factures à rappeler → lien vers la page Rappels. -->
		<p class="mt-2 text-sm">
			<a class="font-medium text-primary underline" href="/invoices/reminders">
				<span data-testid="homepage-reminders-count">
					{i18nMsg('homepage-reminders-count', '{ $n } facture(s) à rappeler', {
						n: reminderCount,
					})}
				</span>
			</a>
		</p>
	{/if}
	{#if canManage}
		<Button variant="outline" class="mt-4" href="/invoices">
			{i18nMsg('homepage-invoices-action', 'Créer une facture')}
		</Button>
	{/if}
</div>
