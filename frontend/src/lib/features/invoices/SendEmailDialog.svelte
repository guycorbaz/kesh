<!--
  Story 20-3b2 — Dialog « Envoyer par e-mail » (#224).
  - Destinataire READ-ONLY (verrouillé `contacts.email`, décision #13 epic-20) ;
    `preview.to === null` = contact sans e-mail → envoi désactivé + message.
  - Objet/corps éditables, pré-remplis depuis la preview serveur (template
    rendu dans la langue du contact) ; ce que l'utilisateur voit est ce qui part.
  - Émet `onConfirm(subject, body)` ; le parent gère l'appel API et la matrice
    d'erreurs (convention MarkPaidDialog : le composant n'appelle jamais l'API).
-->
<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import * as Dialog from '$lib/components/ui/dialog';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import type { EmailPreviewResponse } from './invoices.types';

	type Props = {
		open: boolean;
		onOpenChange: (v: boolean) => void;
		/** Preview serveur (destinataire + objet/corps rendus). */
		preview: EmailPreviewResponse | null;
		submitting?: boolean;
		errorMsg?: string;
		onConfirm: (subject: string, body: string) => void;
	};

	let { open, onOpenChange, preview, submitting = false, errorMsg = '', onConfirm }: Props = $props();

	// IDs DOM stables et HTTP-LAN-safe ($props.id() — pas de crypto.randomUUID,
	// indisponible hors contexte sécurisé, cf. #145).
	const uid = $props.id();

	let subject = $state('');
	let body = $state('');

	// Re-hydrate depuis la preview à chaque ouverture (pas de brouillon
	// conservé entre deux envois — le template peut avoir changé).
	$effect(() => {
		if (open && preview) {
			subject = preview.subject;
			body = preview.body;
		}
	});

	let recipientMissing = $derived(!preview?.to);

	let clientError = $derived.by(() => {
		if (recipientMissing) return '';
		if (!subject.trim() || !body.trim()) {
			return i18nMsg(
				'invoice-send-email-error-empty',
				"L'objet et le corps de l'e-mail ne peuvent pas être vides.",
			);
		}
		return '';
	});

	function handleConfirm() {
		// `submitting` inclus (review P1 ECH) : double-clic = double envoi réel.
		if (submitting || recipientMissing || clientError) return;
		onConfirm(subject.trim(), body.trim());
	}
</script>

<Dialog.Root {open} {onOpenChange}>
	<Dialog.Content>
		<Dialog.Header>
			<Dialog.Title>{i18nMsg('invoice-send-email-title', 'Envoyer la facture par e-mail')}</Dialog.Title>
		</Dialog.Header>
		<div class="mt-2 space-y-3">
			<div>
				<div class="mb-1 text-xs text-text-muted">
					{i18nMsg('invoice-send-email-to-label', 'Destinataire')}
				</div>
				{#if preview?.to}
					<!-- READ-ONLY : jamais un input (anti-exfiltration, décision #13). -->
					<div class="text-sm" data-testid="send-email-to">{preview.to}</div>
				{:else}
					<div
						class="rounded-md border border-destructive bg-destructive/10 px-3 py-2 text-sm text-destructive"
						data-testid="send-email-to-missing"
					>
						{i18nMsg(
							'invoice-send-email-to-missing',
							"Le contact n'a pas d'adresse e-mail — renseignez-la sur la fiche contact.",
						)}
					</div>
				{/if}
			</div>
			<div>
				<label class="mb-1 block text-xs text-text-muted" for="{uid}-subject">
					{i18nMsg('invoice-send-email-subject-label', 'Objet')}
				</label>
				<Input id="{uid}-subject" bind:value={subject} disabled={recipientMissing} />
			</div>
			<div>
				<label class="mb-1 block text-xs text-text-muted" for="{uid}-body">
					{i18nMsg('invoice-send-email-body-label', 'Message')}
				</label>
				<textarea
					id="{uid}-body"
					rows="10"
					bind:value={body}
					disabled={recipientMissing}
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
				disabled={submitting || recipientMissing || !!clientError}
				data-testid="send-email-confirm"
			>
				{i18nMsg('invoice-send-email-confirm', "Envoyer l'e-mail")}
			</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>
