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
			const res = await listReminders();
			if (seq !== loadSeq) return;
			groups = res.groups;
			const m = new Map<number, boolean>();
			for (const g of res.groups) for (const inv of g.invoices) m.set(inv.invoiceId, g.hasEmail);
			emailByInvoice = m;
			// Purge la sélection des factures qui ne sont plus dans la liste.
			const stillHere = new Set([...m.keys()]);
			selected = new Set([...selected].filter((id) => stillHere.has(id)));
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
			if (isApiError(err)) sendError = err.message;
			// Codes « e-mail parti » : ne jamais reproposer l'envoi ; recharger.
			if (isApiError(err) && (err.code === 'REMINDER_SENT_BUT_INVOICE_GONE' || err.code === 'REMINDER_SENT_BUT_NOT_RECORDED')) {
				sendOpen = false;
				await load();
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
	<h1 class="mb-4 text-xl font-semibold">{i18nMsg('reminders-page-title', 'Rappels')}</h1>

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
		maxLevel={Math.max(1, manualTarget.nextLevel ?? manualTarget.currentLevel + 1)}
		defaultLevel={manualTarget.nextLevel ?? Math.max(1, manualTarget.currentLevel)}
		submitting={savingManual}
		errorMsg={manualError}
		onConfirm={confirmManual}
	/>
{/if}
