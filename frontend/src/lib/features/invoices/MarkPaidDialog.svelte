<!--
  Story 5.4 — Dialog « Marquer payée ».
  - Date défaut = aujourd'hui (YYYY-MM-DD).
  - Validation client : paidAt <= today && paidAt >= invoice.date.
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
		const today = todayIso();
		if (paidAt > today) {
			return i18nMsg(
				'invoice-error-paid-at-future',
				'La date de paiement ne peut être postérieure à aujourd\'hui',
			);
		}
		if (invoiceDate && paidAt < invoiceDate) {
			return i18nMsg(
				'invoice-error-paid-at-before-invoice-date',
				'La date de paiement ne peut être antérieure à la date de facture',
			);
		}
		return '';
	});

	function handleConfirm() {
		if (clientError) return;
		// Convertir YYYY-MM-DD en ISO 8601 datetime (midi UTC — évite les soucis TZ
		// où "00:00 locale" pourrait retomber à la veille en UTC).
		onConfirm(`${paidAt}T12:00:00`);
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
				max={todayIso()}
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
