<!--
  Story 21-6b (#231) — Page « Rappels ».
  Liste des factures à rappeler (groupées par débiteur) + envoi e-mail unitaire
  (aperçu éditable, choix de niveau) + envoi lot { accepted, failed } + rappel
  manuel. Anti-double-submit = protection critique (le backend n'en a aucune).
-->
<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import { Mail, FileText } from '@lucide/svelte';
	import { onMount } from 'svelte';

	import { isApiError } from '$lib/shared/utils/api-client';
	import { authState } from '$lib/app/stores/auth.svelte';
	import { notifyError, notifySuccess } from '$lib/shared/utils/notify';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import {
		listReminders,
		getReminderPreview,
		sendReminder,
		sendReminderBatch,
		recordManualReminder,
	} from '$lib/features/reminders/reminders.api';
	import { listDunningLevels } from '$lib/features/dunning/dunning.api';
	import type {
		ContactGroup,
		ReminderCandidate,
		ReminderPreviewResponse,
		SendReminderBatchResponse,
	} from '$lib/features/reminders/reminders.types';
	import ReminderNoEmailBadge from '$lib/features/reminders/ReminderNoEmailBadge.svelte';
	import ReminderTerminalBadge from '$lib/features/reminders/ReminderTerminalBadge.svelte';
	import ReminderSendDialog from '$lib/features/reminders/ReminderSendDialog.svelte';
	import ManualReminderDialog from '$lib/features/reminders/ManualReminderDialog.svelte';
	import ReminderBatchReport from '$lib/features/reminders/ReminderBatchReport.svelte';

	/** Cap dur backend (BATCH_TOO_LARGE au-delà, après dédup). */
	const BATCH_CAP = 20;

	const canManage = $derived(
		authState.currentUser?.role === 'Admin' || authState.currentUser?.role === 'Comptable',
	);

	let groups = $state<ContactGroup[]>([]);
	let loading = $state(false);
	let loadSeq = 0;

	// Sélection lot : Set d'invoiceId (réassigné pour la réactivité runes).
	let selected = $state<Set<number>>(new Set());
	// hasEmail par invoiceId, pour interdire la sélection d'un contact sans e-mail.
	let emailByInvoice = $state<Map<number, boolean>>(new Map());
	// Niveau max configuré (dunning_levels). Le rappel MANUEL autorise le saut
	// vers n'importe quel niveau existant (D18) — contrairement à l'unitaire,
	// borné au prochain. Récupéré au chargement (la liste des candidats ne le
	// porte pas). `0` avant chargement → le manuel retombe sur son défaut sûr.
	let maxConfiguredLevel = $state(0);

	// --- Flags anti-double-submit (couche C : le PARENT possède l'état) ---
	let batchSending = $state(false);
	let sendingUnit = $state(false);
	let savingManual = $state(false);

	let batchReport = $state<SendReminderBatchResponse | null>(null);

	// --- Modale envoi unitaire ---
	let sendOpen = $state(false);
	let sendError = $state('');
	let sendPreview = $state<ReminderPreviewResponse | null>(null);
	let previewLoading = $state(false);
	let sendTarget = $state<ReminderCandidate | null>(null);

	// --- Modale rappel manuel ---
	let manualOpen = $state(false);
	let manualError = $state('');
	let manualTarget = $state<ReminderCandidate | null>(null);

	async function load() {
		if (!canManage) return;
		const seq = ++loadSeq;
		loading = true;
		try {
			// La LISTE des candidats est le fetch primaire (fonction cœur de la
			// page). Le nombre de niveaux configurés n'est utile qu'au saut du
			// rappel manuel (feature secondaire) : on le charge séparément, en
			// dégradation gracieuse — un échec de la config ne doit PAS vider la
			// page ni empêcher d'envoyer des rappels (le manuel retombe alors sur
			// le prochain niveau).
			//
			// ⚠️ Anti-race : TOUS les awaits d'abord, puis UN SEUL garde `seq`,
			// puis TOUTES les écritures d'état. Un garde par-await laisserait le
			// chemin `catch` de la config (ou le code partagé) écraser l'état d'un
			// `load()` plus récent avec des données périmées (deux load()
			// concurrents : envoi lot puis rappel manuel).
			const res = await listReminders();
			let maxLevel = 0;
			try {
				const levels = await listDunningLevels();
				maxLevel = levels.reduce((mx, l) => Math.max(mx, l.levelNumber), 0);
			} catch {
				maxLevel = 0; // le dialog manuel retombe sur nextLevel
			}
			if (seq !== loadSeq) return; // unique garde, après TOUS les awaits
			groups = res.groups;
			maxConfiguredLevel = maxLevel;
			const m = new Map<number, boolean>();
			for (const g of res.groups) for (const inv of g.invoices) m.set(inv.invoiceId, g.hasEmail);
			emailByInvoice = m;
			// Purge la sélection des factures qui ne sont plus dans la liste OU
			// qui ne sont plus sélectionnables (contact ayant perdu son e-mail,
			// facture devenue terminale) — évite un lot voué à un échec par item.
			selected = new Set(
				[...selected].filter((id) => {
					const inv = res.groups.flatMap((g) => g.invoices).find((i) => i.invoiceId === id);
					return inv !== undefined && (m.get(id) ?? false) && !inv.terminal;
				}),
			);
		} catch (err) {
			if (seq !== loadSeq) return;
			if (isApiError(err)) notifyError(err.message);
			groups = [];
			emailByInvoice = new Map();
		} finally {
			if (seq === loadSeq) loading = false;
		}
	}

	onMount(() => {
		load();
	});

	function invoiceLabel(inv: ReminderCandidate): string {
		return inv.invoiceNumber ?? `#${inv.invoiceId}`;
	}

	/** Une facture est cochable si son contact a un e-mail et qu'elle n'est pas terminale. */
	function selectable(inv: ReminderCandidate): boolean {
		return (emailByInvoice.get(inv.invoiceId) ?? false) && !inv.terminal;
	}

	function toggle(id: number) {
		const next = new Set(selected);
		if (next.has(id)) next.delete(id);
		else next.add(id);
		selected = next;
	}

	const overCap = $derived(selected.size > BATCH_CAP);

	// --- Envoi lot (anti-double-submit couches A/C) ---
	async function submitBatch() {
		if (batchSending || selected.size === 0 || overCap) return; // (A) + (C) garde
		batchSending = true; // (C) avant l'appel
		batchReport = null;
		try {
			const res = await sendReminderBatch([...selected]);
			batchReport = res;
			selected = new Set();
			await load();
		} catch (err) {
			if (isApiError(err)) notifyError(err.message);
		} finally {
			batchSending = false; // (C) finally
		}
	}

	// --- Envoi unitaire ---
	async function openSend(inv: ReminderCandidate) {
		if (inv.nextLevel === null) return; // terminale : pas d'envoi e-mail
		sendTarget = inv;
		sendError = '';
		sendPreview = null;
		previewLoading = true;
		try {
			sendPreview = await getReminderPreview(inv.invoiceId, inv.nextLevel);
			sendOpen = true;
		} catch (err) {
			if (isApiError(err)) notifyError(err.message);
		} finally {
			previewLoading = false;
		}
	}

	async function refetchPreview(level: number) {
		if (!sendTarget) return;
		previewLoading = true;
		try {
			sendPreview = await getReminderPreview(sendTarget.invoiceId, level);
		} catch (err) {
			if (isApiError(err)) notifyError(err.message);
		} finally {
			previewLoading = false;
		}
	}

	async function confirmSend(levelNumber: number, subject: string, body: string) {
		if (!sendTarget || sendingUnit) return; // (C) ré-entrance
		sendingUnit = true; // (C)
		sendError = '';
		try {
			await sendReminder(sendTarget.invoiceId, { levelNumber, subject, body });
			notifySuccess(i18nMsg('reminders-send-success', 'Rappel envoyé'));
			sendOpen = false;
			await load();
		} catch (err) {
			if (isApiError(err)) {
				// Codes post-SMTP « e-mail parti » (unitaire) : l'e-mail est
				// irréversiblement parti chez le débiteur, mais l'enregistrement a
				// échoué → NE JAMAIS reproposer l'envoi (un ré-essai renverrait un
				// vrai second e-mail — leçon 21-5b). On ferme la modale ET on émet
				// un TOAST : sinon le message inline disparaîtrait avec la modale et
				// l'utilisateur ignorerait qu'un e-mail est parti sans trace.
				if (
					err.code === 'REMINDER_SENT_BUT_INVOICE_GONE' ||
					err.code === 'REMINDER_SENT_BUT_NOT_RECORDED'
				) {
					notifyError(err.message);
					sendOpen = false;
					await load();
				} else {
					// Erreur avant/pendant l'envoi (l'e-mail n'est PAS parti) : rester
					// dans la modale, l'utilisateur peut corriger et réessayer.
					sendError = err.message;
				}
			}
		} finally {
			sendingUnit = false; // (C)
		}
	}

	// --- Rappel manuel ---
	function openManual(inv: ReminderCandidate) {
		manualTarget = inv;
		manualError = '';
		manualOpen = true;
	}

	async function confirmManual(levelNumber: number, sentAt: string, note: string | null) {
		if (!manualTarget || savingManual) return; // (C)
		savingManual = true; // (C)
		manualError = '';
		try {
			await recordManualReminder(manualTarget.invoiceId, { levelNumber, sentAt, note });
			notifySuccess(i18nMsg('reminders-manual-success', 'Rappel manuel enregistré'));
			manualOpen = false;
			await load();
		} catch (err) {
			if (isApiError(err)) manualError = err.message;
		} finally {
			savingManual = false; // (C)
		}
	}
</script>

<svelte:head>
	<title>{i18nMsg('reminders-page-title', 'Rappels')} — Kesh</title>
</svelte:head>

<div class="p-4">
	<div class="mb-4 flex items-center justify-between">
		<h1 class="text-xl font-semibold">{i18nMsg('reminders-page-title', 'Rappels')}</h1>
		<!-- Story 21-6c (D-c3) : lien croisé retour vers l'échéancier. -->
		<Button variant="outline" href="/invoices/due-dates" data-testid="reminders-link-due-dates">
			{i18nMsg('reminders-link-due-dates', "Voir l'échéancier")}
		</Button>
	</div>

	{#if !canManage}
		<p class="text-sm text-text-muted" data-testid="reminders-forbidden">
			{i18nMsg('reminders-forbidden', 'Accès réservé aux comptables et administrateurs.')}
		</p>
	{:else}
		<!-- Barre d'action lot -->
		<div class="mb-4 flex flex-wrap items-center gap-3">
			<span class="text-sm text-text-muted" data-testid="reminder-selected-count">
				{i18nMsg('reminders-selected-count', '{ $n } sélectionnée(s)', { n: selected.size })}
			</span>
			{#if overCap}
				<span class="text-sm text-destructive" data-testid="reminder-cap-warning">
					{i18nMsg('reminders-batch-cap', 'Maximum { $cap } factures par lot.', { cap: BATCH_CAP })}
				</span>
			{/if}
			<Button
				onclick={submitBatch}
				disabled={batchSending || selected.size === 0 || overCap}
				data-testid="reminder-batch-send"
			>
				{batchSending
					? i18nMsg('reminders-sending', 'Envoi…')
					: i18nMsg('reminders-batch-send', 'Envoyer les rappels sélectionnés')}
			</Button>
		</div>

		{#if batchReport}
			<ReminderBatchReport report={batchReport} />
		{/if}

		{#if loading}
			<p class="text-sm text-text-muted">{i18nMsg('common-loading', 'Chargement…')}</p>
		{:else if groups.length === 0}
			<p class="text-sm text-text-muted" data-testid="reminders-empty">
				{i18nMsg('reminders-empty', 'Aucune facture à rappeler.')}
			</p>
		{:else}
			<div class="space-y-4" data-testid="reminders-list">
				{#each groups as g (g.contactId)}
					<div class="rounded border border-border">
						<div class="flex items-center gap-2 border-b border-border bg-surface-alt px-3 py-2">
							<span class="font-medium">{g.contactName}</span>
							{#if !g.hasEmail}
								<ReminderNoEmailBadge />
							{/if}
						</div>
						<table class="w-full text-sm">
							<tbody>
								{#each g.invoices as inv (inv.invoiceId)}
									<tr class="border-b border-border last:border-0" data-testid="reminder-row">
										<td class="py-2 pl-3 pr-2">
											<input
												type="checkbox"
												checked={selected.has(inv.invoiceId)}
												onchange={() => toggle(inv.invoiceId)}
												disabled={!selectable(inv) || batchSending}
												aria-label={i18nMsg('reminders-select-invoice', 'Sélectionner { $inv }', {
													inv: invoiceLabel(inv),
												})}
												data-testid="reminder-batch-checkbox"
											/>
										</td>
										<td class="py-2 pr-2 font-mono">{invoiceLabel(inv)}</td>
										<td class="py-2 pr-2">{inv.dueDate}</td>
										<td class="py-2 pr-2">
											{#if inv.terminal}
												<ReminderTerminalBadge />
											{:else}
												{i18nMsg('reminders-level-next', 'Prochain : rappel { $level }', {
													level: inv.nextLevel ?? 0,
												})}
											{/if}
										</td>
										<td class="py-2 pr-2 text-text-muted">
											{#if inv.lastReminderAt}
												{i18nMsg('reminders-last-sent', 'dernier le { $date }', {
													date: inv.lastReminderAt.slice(0, 10),
												})}
											{/if}
										</td>
										<td class="flex justify-end gap-1 py-2 pr-3">
											{#if !inv.terminal && g.hasEmail}
												<Button
													variant="ghost"
													size="sm"
													onclick={() => openSend(inv)}
													data-testid="reminder-send-open"
												>
													<Mail class="h-4 w-4" aria-hidden="true" />
													<span class="sr-only">{i18nMsg('reminders-send-open', 'Envoyer un rappel')}</span>
												</Button>
											{/if}
											<Button
												variant="ghost"
												size="sm"
												onclick={() => openManual(inv)}
												data-testid="reminder-manual-open"
											>
												<FileText class="h-4 w-4" aria-hidden="true" />
												<span class="sr-only">{i18nMsg('reminders-manual-open', 'Rappel manuel')}</span>
											</Button>
										</td>
									</tr>
								{/each}
							</tbody>
						</table>
					</div>
				{/each}
			</div>
		{/if}
	{/if}
</div>

<!-- Modale envoi unitaire — anti-double-submit couche D : non fermable en vol. -->
{#if sendTarget}
	<ReminderSendDialog
		open={sendOpen}
		onOpenChange={(o) => {
			if (!o && sendingUnit) return; // (D)
			sendOpen = o;
			if (!o) sendError = '';
		}}
		preview={sendPreview}
		maxLevel={sendTarget.nextLevel ?? 1}
		invoiceLabel={invoiceLabel(sendTarget)}
		{previewLoading}
		submitting={sendingUnit}
		errorMsg={sendError}
		onLevelChange={refetchPreview}
		onConfirm={confirmSend}
	/>
{/if}

<!-- Modale rappel manuel — couche D. -->
{#if manualTarget}
	<ManualReminderDialog
		open={manualOpen}
		onOpenChange={(o) => {
			if (!o && savingManual) return; // (D)
			manualOpen = o;
			if (!o) manualError = '';
		}}
		invoiceLabel={invoiceLabel(manualTarget)}
		maxLevel={Math.max(1, maxConfiguredLevel, manualTarget.nextLevel ?? 1)}
		defaultLevel={manualTarget.nextLevel ?? Math.max(1, manualTarget.currentLevel)}
		submitting={savingManual}
		errorMsg={manualError}
		onConfirm={confirmManual}
	/>
{/if}
