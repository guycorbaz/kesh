<!--
  Story 5.4 — Dialog « Marquer payée ».
  - Date défaut = aujourd'hui (YYYY-MM-DD).
  - Validation client : paidAt >= invoice.date - 1j (la borne haute future est
    autorisée — AC#8 amendé 2026-04-15 : paid_at = date d'exécution bancaire).
  - Émet `onConfirm({ paidAt })` ; le parent gère l'appel API (409 recharge, 400 toast).
-->
<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import * as Dialog from '$lib/components/ui/dialog';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';

	type Props = {
		open: boolean;
		onOpenChange: (v: boolean) => void;
		/** Date facture (YYYY-MM-DD) — borne basse pour paidAt. */
		invoiceDate: string;
		submitting?: boolean;
		errorMsg?: string;
		onConfirm: (paidAt: string) => void;
	};

	let {
		open,
		onOpenChange,
		invoiceDate,
		submitting = false,
		errorMsg = '',
		onConfirm,
	}: Props = $props();

	function todayIso(): string {
		return new Date().toISOString().slice(0, 10);
	}

	let paidAt = $state(todayIso());

	// Reset à l'ouverture pour ne pas conserver la valeur précédente.
	$effect(() => {
		if (open) paidAt = todayIso();
	});

	let clientError = $derived.by(() => {
		if (!paidAt) return i18nMsg('invoice-error-paid-at-required', 'Date de paiement obligatoire');
		// AC#8 amendé : pas de borne haute — paid_at peut être dans le futur
		// (date d'exécution bancaire, ordre programmé, décalage week-end/férié).
		if (invoiceDate && paidAt < invoiceDate.slice(0, 10)) {
			return i18nMsg(
				'invoice-error-paid-at-before-invoice-date',
				'La date de paiement ne peut être antérieure à la date de facture',
			);
		}
		return '';
	});

	function handleConfirm() {
		if (clientError) return;
		// Convertir YYYY-MM-DD en ISO 8601 datetime à midi UTC explicite (`Z`)
		// — évite l'ambiguïté naive datetime que les parsers JS/Rust
		// interprètent comme heure locale.
		onConfirm(`${paidAt}T12:00:00Z`);
	}
</script>

<Dialog.Root {open} {onOpenChange}>
	<Dialog.Content>
		<Dialog.Header>
			<Dialog.Title>{i18nMsg('invoice-mark-paid-dialog-title', 'Marquer la facture comme payée')}</Dialog.Title>
		</Dialog.Header>
		<p class="text-sm">
			{i18nMsg(
				'invoice-mark-paid-dialog-body',
				'Indiquez la date à laquelle vous avez reçu le paiement.',
			)}
		</p>
		<div class="mt-2">
			<label class="mb-1 block text-xs text-text-muted" for="mark-paid-date">
				{i18nMsg('invoice-mark-paid-date-label', 'Date de paiement')}
			</label>
			<Input
				id="mark-paid-date"
				type="date"
				bind:value={paidAt}
				min={invoiceDate}
			/>
		</div>
		{#if clientError}
			<div class="rounded-md border border-destructive bg-destructive/10 px-3 py-2 text-sm text-destructive">
				{clientError}
			</div>
		{:else if errorMsg}
			<div class="rounded-md border border-destructive bg-destructive/10 px-3 py-2 text-sm text-destructive">
				{errorMsg}
			</div>
		{/if}
		<Dialog.Footer>
			<Button variant="outline" onclick={() => onOpenChange(false)} disabled={submitting}>
				{i18nMsg('common-cancel', 'Annuler')}
			</Button>
			<Button onclick={handleConfirm} disabled={submitting || !!clientError}>
				{i18nMsg('invoice-mark-paid-confirm', 'Confirmer le paiement')}
			</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>
