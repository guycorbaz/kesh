<!--
  Story 21-6b (#231) — Dialog « Enregistrer un rappel manuel » (papier/recommandé).

  Présentationnel (patron MarkPaidDialog) : n'appelle jamais l'API, émet
  `onConfirm(levelNumber, sentAt, note)`. Le parent gère l'appel + le flag
  `submitting` (anti-double-submit).

  Le manuel autorise le saut de niveau (D18) : le `<select>` va de 1 à maxLevel.
-->
<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import * as Dialog from '$lib/components/ui/dialog';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';

	type Props = {
		open: boolean;
		onOpenChange: (v: boolean) => void;
		/** Numéro d'échéance/facture pour le titre. */
		invoiceLabel: string;
		/** Niveau maximum sélectionnable (saut autorisé — au moins 1). */
		maxLevel: number;
		/** Niveau proposé par défaut. */
		defaultLevel: number;
		submitting?: boolean;
		errorMsg?: string;
		onConfirm: (levelNumber: number, sentAt: string, note: string | null) => void;
	};

	let {
		open,
		onOpenChange,
		invoiceLabel,
		maxLevel,
		defaultLevel,
		submitting = false,
		errorMsg = '',
		onConfirm,
	}: Props = $props();

	function todayIso(): string {
		return new Date().toISOString().slice(0, 10);
	}

	let level = $state(defaultLevel);
	let sentAt = $state(todayIso());
	let note = $state('');

	// Reset à chaque ouverture — pas de brouillon conservé entre deux rappels.
	$effect(() => {
		if (open) {
			level = defaultLevel;
			sentAt = todayIso();
			note = '';
		}
	});

	const levelOptions = $derived(
		Array.from({ length: Math.max(1, maxLevel) }, (_, i) => i + 1),
	);

	let clientError = $derived.by(() => {
		if (!sentAt) return i18nMsg('reminders-manual-date-required', "Date d'envoi obligatoire");
		// Le backend rejette une date d'envoi future (422 REMINDER_DATE_IN_FUTURE).
		if (sentAt > todayIso()) {
			return i18nMsg('reminders-manual-date-future', "La date d'envoi ne peut être dans le futur");
		}
		return '';
	});

	function handleConfirm() {
		if (submitting || clientError) return;
		// #249 : `sent_at` est un NaiveDateTime côté backend — une date nue
		// ("YYYY-MM-DD") est rejetée à la désérialisation. Suffixer T12:00:00
		// (heure fixée à midi pour éviter tout décalage de date à l'affichage).
		onConfirm(level, `${sentAt}T12:00:00`, note.trim() || null);
	}
</script>

<Dialog.Root {open} {onOpenChange}>
	<Dialog.Content>
		<Dialog.Header>
			<Dialog.Title>
				{i18nMsg('reminders-manual-title', 'Enregistrer un rappel manuel')} — {invoiceLabel}
			</Dialog.Title>
		</Dialog.Header>
		<p class="text-sm text-text-muted">
			{i18nMsg(
				'reminders-manual-body',
				'Enregistrez un rappel déjà envoyé hors Kesh (courrier, recommandé). Aucun e-mail ne sera envoyé.',
			)}
		</p>

		<div class="mt-3">
			<label class="mb-1 block text-xs text-text-muted" for="manual-reminder-level">
				{i18nMsg('reminders-manual-level-label', 'Niveau de rappel')}
			</label>
			<select
				id="manual-reminder-level"
				data-testid="manual-reminder-level"
				bind:value={level}
				class="h-9 rounded-md border border-border bg-background px-2 text-sm"
			>
				{#each levelOptions as n (n)}
					<option value={n}>{i18nMsg('reminders-level-name', 'Rappel { $level }', { level: n })}</option>
				{/each}
			</select>
		</div>

		<div class="mt-3">
			<label class="mb-1 block text-xs text-text-muted" for="manual-reminder-date">
				{i18nMsg('reminders-manual-date-label', "Date d'envoi")}
			</label>
			<Input id="manual-reminder-date" type="date" bind:value={sentAt} max={todayIso()} />
		</div>

		<div class="mt-3">
			<label class="mb-1 block text-xs text-text-muted" for="manual-reminder-note">
				{i18nMsg('reminders-manual-note-label', 'Note (facultatif)')}
			</label>
			<textarea
				id="manual-reminder-note"
				bind:value={note}
				rows="2"
				class="w-full rounded-md border border-border bg-background px-2 py-1 text-sm"
			></textarea>
		</div>

		{#if clientError}
			<div class="mt-2 rounded-md border border-destructive bg-destructive/10 px-3 py-2 text-sm text-destructive">
				{clientError}
			</div>
		{:else if errorMsg}
			<div class="mt-2 rounded-md border border-destructive bg-destructive/10 px-3 py-2 text-sm text-destructive">
				{errorMsg}
			</div>
		{/if}

		<Dialog.Footer>
			<Button variant="outline" onclick={() => onOpenChange(false)} disabled={submitting}>
				{i18nMsg('common-cancel', 'Annuler')}
			</Button>
			<Button
				onclick={handleConfirm}
				disabled={submitting || !!clientError}
				data-testid="manual-reminder-confirm"
			>
				{submitting
					? i18nMsg('reminders-saving', 'Enregistrement…')
					: i18nMsg('reminders-manual-confirm', 'Enregistrer')}
			</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>
