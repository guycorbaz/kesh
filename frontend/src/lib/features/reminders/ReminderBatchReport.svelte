<!--
  Story 21-6b (#231) — Rapport d'un envoi de rappels par lot { accepted, failed }.

  Patron supplier-invoices/import : compte des réussis + liste des échoués avec
  identifiant business + raison traduite. Un lot est un succès HTTP 200 même
  partiel. Aucun bouton « Réessayer » sur les échecs (l'e-mail a pu partir).
-->
<script lang="ts">
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { reminderErrorLabel } from './reminder-error-label';
	import type { SendReminderBatchResponse } from './reminders.types';

	let { report }: { report: SendReminderBatchResponse } = $props();
</script>

<div class="mb-4 rounded border border-border p-4" data-testid="reminder-batch-report">
	<p class="text-sm font-medium">
		{i18nMsg('reminders-batch-accepted', '{ $n } rappel(s) envoyé(s).', {
			n: report.accepted.length,
		})}
	</p>
	{#if report.failed.length > 0}
		<div class="mt-3">
			<p class="text-sm font-medium text-destructive">
				{i18nMsg('reminders-batch-failed', '{ $n } échec(s) :', { n: report.failed.length })}
			</p>
			<ul class="mt-1 space-y-1 text-sm">
				{#each report.failed as f (f.invoiceId)}
					<li class="text-destructive" data-testid="reminder-batch-failed-row">
						<span class="font-mono">#{f.invoiceId}</span> — {reminderErrorLabel(f.errorCode)}
					</li>
				{/each}
			</ul>
		</div>
	{/if}
</div>
