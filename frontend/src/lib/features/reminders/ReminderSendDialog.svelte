<!--
  Story 21-6b (#231) — Dialog « Envoyer un rappel par e-mail » (unitaire).

  Présentationnel (patron SendEmailDialog) : n'appelle jamais l'API, émet
  `onConfirm(levelNumber, subject, body)` et `onLevelChange(level)` (pour que le
  parent re-fetch l'aperçu). Le parent possède le flag `submitting`
  (anti-double-submit).

  - Destinataire READ-ONLY (verrouillé contacts.email) ; `preview.to === null`
    → envoi désactivé (mais on n'ouvre normalement pas cette modale sur un
    contact sans e-mail — le bouton est déjà masqué côté liste).
  - Choix du niveau ≤ prochain (D18 : ré-émettre un e-mail perdu ; jamais de saut
    vers le haut, que le backend refuserait en 409). Changer de niveau re-rend
    l'aperçu serveur.
-->
<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import * as Dialog from '$lib/components/ui/dialog';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import type { ReminderPreviewResponse } from './reminders.types';

	type Props = {
		open: boolean;
		onOpenChange: (v: boolean) => void;
		/** Aperçu serveur du niveau courant (destinataire + objet/corps rendus). */
		preview: ReminderPreviewResponse | null;
		/** Niveau prochain = borne haute sélectionnable (jamais de saut, D18). */
		maxLevel: number;
		/** Numéro d'échéance/facture pour le titre. */
		invoiceLabel: string;
		/** Aperçu en cours de (re)chargement → champs désactivés. */
		previewLoading?: boolean;
		submitting?: boolean;
		errorMsg?: string;
		onLevelChange: (level: number) => void;
		onConfirm: (levelNumber: number, subject: string, body: string) => void;
	};

	let {
		open,
		onOpenChange,
		preview,
		maxLevel,
		invoiceLabel,
		previewLoading = false,
		submitting = false,
		errorMsg = '',
		onLevelChange,
		onConfirm,
	}: Props = $props();

	// IDs DOM stables et HTTP-LAN-safe ($props.id() — pas de crypto.randomUUID).
	const uid = $props.id();

	let subject = $state('');
	let body = $state('');

	// Re-hydrate depuis l'aperçu à chaque (ré)ouverture ET à chaque changement de
	// niveau (l'aperçu change) — pas de brouillon conservé.
	$effect(() => {
		if (open && preview) {
			subject = preview.subject;
			body = preview.body;
		}
	});

	const levelOptions = $derived(Array.from({ length: Math.max(1, maxLevel) }, (_, i) => i + 1));

	let recipientMissing = $derived(!preview?.to);

	let clientError = $derived.by(() => {
		if (recipientMissing || previewLoading) return '';
		if (!subject.trim() || !body.trim()) {
			return i18nMsg('reminders-send-empty', "L'objet et le corps ne peuvent pas être vides.");
		}
		return '';
	});

	function handleLevelChange(e: Event) {
		const lvl = Number((e.target as HTMLSelectElement).value);
		onLevelChange(lvl);
	}

	function handleConfirm() {
		if (submitting || previewLoading || recipientMissing || clientError) return;
		onConfirm(preview?.level ?? maxLevel, subject.trim(), body.trim());
	}
</script>

<Dialog.Root {open} {onOpenChange}>
	<Dialog.Content>
		<Dialog.Header>
			<Dialog.Title>
				{i18nMsg('reminders-send-title', 'Envoyer un rappel')} — {invoiceLabel}
			</Dialog.Title>
		</Dialog.Header>
		<div class="mt-2 space-y-3">
			<div>
				<label class="mb-1 block text-xs text-text-muted" for="{uid}-level">
					{i18nMsg('reminders-send-level-label', 'Niveau de rappel')}
				</label>
				<select
					id="{uid}-level"
					data-testid="reminder-send-level"
					value={preview?.level ?? maxLevel}
					onchange={handleLevelChange}
					disabled={submitting}
					class="h-9 rounded-md border border-border bg-background px-2 text-sm"
				>
					{#each levelOptions as n (n)}
						<option value={n}>{i18nMsg('reminders-level-name', 'Rappel { $level }', { level: n })}</option>
					{/each}
				</select>
			</div>
			<div>
				<div class="mb-1 text-xs text-text-muted">
					{i18nMsg('reminders-send-to-label', 'Destinataire')}
				</div>
				{#if preview?.to}
					<div class="text-sm" data-testid="reminder-send-to">{preview.to}</div>
				{:else}
					<div
						class="rounded-md border border-destructive bg-destructive/10 px-3 py-2 text-sm text-destructive"
						data-testid="reminder-send-to-missing"
					>
						{i18nMsg('reminders-send-no-recipient', "Le contact n'a pas d'adresse e-mail.")}
					</div>
				{/if}
			</div>
			<div>
				<label class="mb-1 block text-xs text-text-muted" for="{uid}-subject">
					{i18nMsg('reminders-send-subject-label', 'Objet')}
				</label>
				<Input
					id="{uid}-subject"
					bind:value={subject}
					disabled={recipientMissing || previewLoading || submitting}
				/>
			</div>
			<div>
				<label class="mb-1 block text-xs text-text-muted" for="{uid}-body">
					{i18nMsg('reminders-send-body-label', 'Message')}
				</label>
				<textarea
					id="{uid}-body"
					rows="10"
					bind:value={body}
					disabled={recipientMissing || previewLoading || submitting}
					class="border-input bg-background w-full rounded-md border px-3 py-2 text-sm disabled:opacity-50"
				></textarea>
			</div>
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
			<Button
				onclick={handleConfirm}
				disabled={submitting || previewLoading || recipientMissing || !!clientError}
				data-testid="reminder-send-confirm"
			>
				{submitting
					? i18nMsg('reminders-sending', 'Envoi…')
					: i18nMsg('reminders-send-confirm', 'Envoyer le rappel')}
			</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>
