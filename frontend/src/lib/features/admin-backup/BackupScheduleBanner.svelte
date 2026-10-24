<!--
  Story 31-1c (#503, AC31 (b)) — bandeau discret sous l'en-tête de
  l'application, pour un Administrateur seulement, quand la sauvegarde
  automatique est activée et (a) que la dernière tentative a échoué, ou (b)
  que le serveur dit qu'aucune réussite ne date de moins de deux intervalles
  (`overdue`). `partial` ne l'allume pas (C-31-1-40). `role="status"`, jamais
  `alert` (une base E2E dont la dernière réussite est ancienne ferait sinon
  rougir les specs qui exigent zéro `alert`).

  Lecture : au montage puis toutes les 30 minutes tant que l'application est
  ouverte, et après chaque action du panneau (état partagé). Un échec de la
  première lecture n'affiche aucun bandeau ; un échec ultérieur garde le
  dernier état lu, et une réponse tardive est ignorée (Story 32-1b, #711 E-8).

  Story 32-1b (#711 L-B5) : en retard, une dernière réussite datée APRÈS
  l'heure du serveur (`lastSuccessInFuture`, calculé par le serveur — le
  bandeau ne compare aucune date) se dit telle : l'horloge du serveur a
  reculé, et « aucune sauvegarde réussie depuis le <date future> » serait faux.
-->
<script lang="ts">
	import { authState } from '$lib/app/stores/auth.svelte';
	import { i18nLocale, i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { bannerKind } from './backup-schedule.api';
	import { formatBackupDate } from './backup-format';
	import { backupScheduleState, refreshBackupScheduleState } from './backup-schedule.svelte';

	const REFRESH_MS = 30 * 60_000;

	let isAdmin = $derived(authState.currentUser?.role === 'Admin');

	$effect(() => {
		if (!isAdmin) return;
		void refreshBackupScheduleState();
		const timer = setInterval(() => void refreshBackupScheduleState(), REFRESH_MS);
		return () => clearInterval(timer);
	});

	let kind = $derived(isAdmin ? bannerKind(backupScheduleState.value) : null);
	let since = $derived(formatBackupDate(backupScheduleState.value?.lastSuccessAt ?? null, i18nLocale()));
</script>

{#if kind}
	<div
		role="status"
		class="flex items-center justify-between gap-3 border-b border-warning bg-warning/10 px-4 py-2 text-sm text-text"
		data-testid="backup-schedule-banner"
		data-kind={kind}
	>
		<span>
			{#if kind === 'failed'}
				{i18nMsg('admin-backup-banner-failed', 'La dernière sauvegarde automatique a échoué.')}
			{:else if backupScheduleState.value?.lastSuccessInFuture}
				{i18nMsg(
					'admin-backup-banner-future',
					"La dernière sauvegarde automatique réussie est datée du { $date }, après l'heure du serveur : son horloge a reculé.",
					{ date: since },
				)}
			{:else}
				{i18nMsg(
					'admin-backup-banner-overdue',
					'Aucune sauvegarde automatique réussie depuis le { $date }.',
					{ date: since },
				)}
			{/if}
		</span>
		<a href="/admin/backup" class="font-medium underline" data-testid="backup-schedule-banner-link">
			{i18nMsg('admin-backup-banner-link', 'Administration → Sauvegarde complète')}
		</a>
	</div>
{/if}
