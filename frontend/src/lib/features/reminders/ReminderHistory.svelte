<!--
  Story 21-6c (#231) — Historique des rappels d'une facture (fiche facture).

  Présentationnel : reçoit `reminders: ReminderResponse[]` déjà triés serveur
  (`ORDER BY sent_at DESC`, plus récent d'abord) — NE PAS re-trier.

  Un rappel annulé (`cancelledAt !== null`, annulation douce Admin) est distingué
  visuellement (texte barré + mention « annulé le … »). Pas de champ booléen :
  tester la non-nullité de `cancelledAt`.

  Namespace i18n : clés `reminders-*` uniquement (composant sous
  `features/reminders/`, contrainte lint #30).
-->
<script lang="ts">
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { formatInvoiceTotal } from '$lib/features/invoices/invoice-helpers';
	import type { ReminderResponse } from './reminders.types';

	let { reminders }: { reminders: ReminderResponse[] } = $props();

	/** Libellé du canal (`"email"` | `"manual"`) via clés i18n. */
	function channelLabel(channel: string): string {
		return channel === 'manual'
			? i18nMsg('reminders-history-channel-manual', 'Manuel')
			: i18nMsg('reminders-history-channel-email', 'E-mail');
	}
</script>

<section class="space-y-2" data-testid="reminder-history">
	<h2 class="text-lg font-semibold">
		{i18nMsg('reminders-history-title', 'Historique des rappels')}
	</h2>

	{#if reminders.length === 0}
		<p class="text-sm text-text-muted" data-testid="reminder-history-empty">
			{i18nMsg('reminders-history-empty', 'Aucun rappel envoyé.')}
		</p>
	{:else}
		<table class="w-full border-collapse text-sm">
			<thead>
				<tr class="border-b border-border text-left">
					<th class="py-2 pr-2">{i18nMsg('reminders-history-col-date', 'Date')}</th>
					<th class="py-2 pr-2">{i18nMsg('reminders-history-col-level', 'Niveau')}</th>
					<th class="py-2 pr-2">{i18nMsg('reminders-history-col-channel', 'Canal')}</th>
					<th class="py-2 pr-2">{i18nMsg('reminders-history-col-recipient', 'Destinataire')}</th>
					<th class="py-2 pr-2 text-right">{i18nMsg('reminders-history-col-fee', 'Frais')}</th>
				</tr>
			</thead>
			<tbody>
				{#each reminders as r (r.id)}
					{@const cancelled = r.cancelledAt !== null}
					<tr
						class="border-b border-border {cancelled ? 'text-text-muted line-through' : ''}"
						data-testid="reminder-history-row"
					>
						<td class="py-2 pr-2">{r.sentAt.slice(0, 10)}</td>
						<td class="py-2 pr-2">
							{i18nMsg('reminders-level-name', 'Rappel { $level }', { level: r.levelNumber })}
						</td>
						<td class="py-2 pr-2">{channelLabel(r.channel)}</td>
						<td class="py-2 pr-2">{r.sentTo ?? '—'}</td>
						<td class="py-2 pr-2 text-right font-mono">
							{formatInvoiceTotal(r.feeAmount)}
						</td>
					</tr>
					{#if cancelled}
						<tr class="border-b border-border">
							<td colspan="5" class="pb-2 text-xs text-text-muted">
								{i18nMsg('reminders-history-cancelled-at', 'Annulé le { $date }', {
									date: r.cancelledAt!.slice(0, 10),
								})}
							</td>
						</tr>
					{/if}
				{/each}
			</tbody>
		</table>
	{/if}
</section>
