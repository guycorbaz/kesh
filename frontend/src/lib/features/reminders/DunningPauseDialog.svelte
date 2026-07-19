<!--
  Story 21-6c (#231, D-c1) — Dialog « Suspendre les rappels » (note optionnelle).

  Présentationnel (patron ManualReminderDialog) : n'appelle JAMAIS l'API, émet
  `onConfirm(note)`. Le parent gère l'appel + le flag `submitting`
  (anti-double-submit) et rend la modale non-fermable en vol (patron
  SendEmailDialog).

  La reprise n'ouvre PAS de dialog (D-c1 — action directe côté parent).

  Namespace i18n : clés `reminders-*` uniquement (composant sous
  `features/reminders/`, contrainte lint #30).
-->
<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import * as Dialog from '$lib/components/ui/dialog';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';

	/** Borne serveur `PAUSE_NOTE_MAX` (invoices.dunning_paused_note VARCHAR(500)). */
	const PAUSE_NOTE_MAX = 500;

	type Props = {
		open: boolean;
		onOpenChange: (v: boolean) => void;
		/** Numéro de facture pour le titre. */
		invoiceLabel: string;
		submitting?: boolean;
		/** Erreur remontée par le parent (affichée inline, le dialog reste ouvert). */
		errorMsg?: string;
		onConfirm: (note: string | null) => void;
	};

	let {
		open,
		onOpenChange,
		invoiceLabel,
		submitting = false,
		errorMsg = '',
		onConfirm,
	}: Props = $props();

	// IDs DOM stables et HTTP-LAN-safe ($props.id() — pas de crypto.randomUUID).
	const uid = $props.id();

	let note = $state('');

	// Reset à chaque ouverture — pas de brouillon conservé entre deux suspensions.
	$effect(() => {
		if (open) note = '';
	});

	function handleConfirm() {
		if (submitting) return;
		onConfirm(note.trim() || null);
	}
</script>

<Dialog.Root {open} {onOpenChange}>
	<Dialog.Content>
		<Dialog.Header>
			<Dialog.Title>
				{i18nMsg('reminders-pause-title', 'Suspendre les rappels')} — {invoiceLabel}
			</Dialog.Title>
		</Dialog.Header>
		<p class="text-sm text-text-muted">
			{i18nMsg(
				'reminders-pause-body',
				'Les rappels automatiques de cette facture sont suspendus jusqu’à leur reprise. Vous pouvez noter le motif (litige, arrangement).',
			)}
		</p>

		<div class="mt-3">
			<label class="mb-1 block text-xs text-text-muted" for="{uid}-note">
				{i18nMsg('reminders-pause-note-label', 'Motif (facultatif)')}
			</label>
			<textarea
				id="{uid}-note"
				data-testid="dunning-pause-note"
				bind:value={note}
				rows="2"
				maxlength={PAUSE_NOTE_MAX}
				class="w-full rounded-md border border-border bg-background px-2 py-1 text-sm"
			></textarea>
		</div>

		{#if errorMsg}
			<div class="mt-2 rounded-md border border-destructive bg-destructive/10 px-3 py-2 text-sm text-destructive">
				{errorMsg}
			</div>
		{/if}

		<Dialog.Footer>
			<Button variant="outline" onclick={() => onOpenChange(false)} disabled={submitting}>
				{i18nMsg('common-cancel', 'Annuler')}
			</Button>
			<Button onclick={handleConfirm} disabled={submitting} data-testid="dunning-pause-confirm">
				{submitting
					? i18nMsg('reminders-pause-submitting', 'Suspension…')
					: i18nMsg('reminders-pause-confirm', 'Suspendre')}
			</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>
