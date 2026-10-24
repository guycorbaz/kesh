<!--
  Story 31-1c (#503, AC33, AC34) — section « Sauvegarde automatique » du panneau
  d'administration : réglage (activée, fréquence, nombre conservé), état
  (dernière réussite, dernière complète, dernière tentative, prochaine prévue),
  « Sauvegarder maintenant », et la liste des sauvegardes du serveur avec leur
  téléchargement — la voie normale pour récupérer une sauvegarde pré-import,
  SSH restant le secours.

  Un échec (`failed`, `insufficient_space`, `disk_full`) s'affiche en bandeau
  d'erreur (`role="alert"`) ; `partial` en avertissement (`role="status"`) — un
  seul rôle par statut. Les dates et le retard sont ceux du serveur ; l'écran
  ne compare aucune date à l'horloge du navigateur.
-->
<script lang="ts">
	import { onMount } from 'svelte';
	import { toast } from 'svelte-sonner';
	import { Button } from '$lib/components/ui/button';
	import { isApiError } from '$lib/shared/utils/api-client';
	import { i18nLocale, i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import {
		BACKUP_FAILURE_STATUSES,
		BACKUP_INTERVAL_HOURS,
		RETENTION_MAX,
		RETENTION_MIN,
		downloadServerBackup,
		exceedsImportLimit,
		fetchBackupSchedule,
		fetchServerBackups,
		runBackupNow,
		saveBackupSchedule,
		type BackupSchedule,
		type BackupStatus,
		type ServerBackup,
		type ServerBackups,
	} from './backup-schedule.api';
	import { formatBackupDate, formatBackupSize } from './backup-format';
	import { setBackupScheduleState } from './backup-schedule.svelte';

	let schedule = $state<BackupSchedule | null>(null);
	let loadError = $state<string | null>(null);

	let enabled = $state(true);
	let intervalHours = $state(24);
	let retentionCount = $state(7);

	let saving = $state(false);
	let saveError = $state<string | null>(null);
	let conflict = $state(false);

	let running = $state(false);
	let runError = $state<string | null>(null);

	let backups = $state<ServerBackups | null>(null);
	let listError = $state<string | null>(null);
	let downloading = $state<string | null>(null);

	const locale = $derived(i18nLocale());

	function errorText(err: unknown, fallback: string): string {
		return isApiError(err) && err.message ? err.message : fallback;
	}

	function apply(value: BackupSchedule): void {
		schedule = value;
		enabled = value.enabled;
		intervalHours = value.intervalHours;
		retentionCount = value.retentionCount;
		setBackupScheduleState(value);
	}

	async function loadSchedule(): Promise<void> {
		loadError = null;
		conflict = false;
		saveError = null;
		try {
			apply(await fetchBackupSchedule());
		} catch (err) {
			loadError = errorText(
				err,
				i18nMsg(
					'admin-backup-auto-load-error',
					"Le réglage de la sauvegarde automatique n'a pas pu être lu.",
				),
			);
		}
	}

	async function loadBackups(): Promise<void> {
		listError = null;
		try {
			backups = await fetchServerBackups();
		} catch (err) {
			listError = errorText(
				err,
				i18nMsg(
					'admin-backup-server-error',
					"La liste des sauvegardes du serveur n'a pas pu être lue.",
				),
			);
		}
	}

	onMount(() => {
		void loadSchedule();
		void loadBackups();
	});

	async function save(): Promise<void> {
		if (!schedule || saving) return;
		saving = true;
		saveError = null;
		conflict = false;
		try {
			apply(
				await saveBackupSchedule({
					enabled,
					intervalHours: Number(intervalHours),
					retentionCount: Number(retentionCount),
					version: schedule.version,
				}),
			);
			toast.success(
				i18nMsg('admin-backup-auto-saved', 'Réglage de la sauvegarde automatique enregistré.'),
			);
		} catch (err) {
			if (isApiError(err) && err.status === 409 && err.code === 'OPTIMISTIC_LOCK_CONFLICT') {
				conflict = true;
			} else {
				saveError = errorText(
					err,
					i18nMsg('admin-backup-auto-save-error', "Le réglage n'a pas pu être enregistré."),
				);
			}
		} finally {
			saving = false;
		}
	}

	async function runNow(): Promise<void> {
		if (running) return;
		running = true;
		runError = null;
		try {
			apply(await runBackupNow());
			await loadBackups();
		} catch (err) {
			runError = errorText(
				err,
				i18nMsg('admin-backup-auto-run-error', "La sauvegarde n'a pas pu être lancée."),
			);
		} finally {
			running = false;
		}
	}

	async function download(backup: ServerBackup): Promise<void> {
		if (downloading) return;
		downloading = backup.name;
		try {
			await downloadServerBackup(backup);
		} catch (err) {
			toast.error(
				errorText(
					err,
					i18nMsg('admin-backup-server-download-error', 'Le téléchargement a échoué.'),
				),
			);
		} finally {
			downloading = null;
		}
	}

	function failureMessage(status: BackupStatus): string {
		switch (status) {
			case 'insufficient_space':
				return i18nMsg(
					'admin-backup-auto-status-insufficient-space',
					"La dernière sauvegarde automatique a été refusée : l'espace disque est insuffisant. Libérez de la place sur le disque du dossier de sauvegarde, par exemple en supprimant d'anciennes sauvegardes de ./backup/auto ; le nombre de sauvegardes conservées ne s'applique qu'après une sauvegarde réussie.",
				);
			case 'disk_full':
				return i18nMsg(
					'admin-backup-auto-status-disk-full',
					"La dernière sauvegarde automatique a échoué : le disque ou le quota du dossier de sauvegarde s'est rempli pendant l'écriture. Libérez de la place, puis lancez « Sauvegarder maintenant ».",
				);
			default:
				return i18nMsg(
					'admin-backup-auto-status-failed',
					"La dernière sauvegarde automatique a échoué (écriture ou relecture impossible) — voir le journal du serveur.",
				);
		}
	}

	function frequencyLabel(hours: number): string {
		switch (hours) {
			case 6:
				return i18nMsg('admin-backup-auto-frequency-6', 'Toutes les 6 heures');
			case 12:
				return i18nMsg('admin-backup-auto-frequency-12', 'Toutes les 12 heures');
			case 168:
				return i18nMsg('admin-backup-auto-frequency-168', 'Chaque semaine');
			default:
				return i18nMsg('admin-backup-auto-frequency-24', 'Chaque jour');
		}
	}

	let failure = $derived(
		schedule?.lastStatus && BACKUP_FAILURE_STATUSES.includes(schedule.lastStatus)
			? schedule.lastStatus
			: null,
	);
	let showComplete = $derived(
		!!schedule?.lastCompleteFile && schedule.lastCompleteFile !== schedule.lastSuccessFile,
	);
	let completeMissing = $derived(
		!!schedule?.lastCompleteFile &&
			backups !== null &&
			!backups.items.some((b) => b.kind === 'auto' && b.name === schedule?.lastCompleteFile),
	);
</script>

<section class="flex flex-col gap-3 rounded-md border border-border p-4" data-testid="auto-backup-section">
	<header class="flex flex-col gap-1">
		<h2 class="text-lg font-semibold text-text">
			{i18nMsg('admin-backup-auto-title', 'Sauvegarde automatique')}
		</h2>
		<p class="text-sm text-text-muted">
			{i18nMsg(
				'admin-backup-auto-description',
				"Kesh écrit lui-même, à intervalle régulier, une sauvegarde complète (base et pièces justificatives) dans le dossier de sauvegarde de l'installation.",
			)}
		</p>
	</header>

	{#if loadError}
		<div
			role="alert"
			class="rounded-md border border-destructive bg-destructive/10 px-3 py-2 text-sm text-destructive"
			data-testid="auto-backup-load-error"
		>
			{loadError}
		</div>
	{/if}

	{#if schedule}
		<!-- Réglage -->
		<div class="flex flex-wrap items-end gap-4">
			<label class="flex items-center gap-2 text-sm">
				<input type="checkbox" bind:checked={enabled} data-testid="auto-backup-enabled" />
				{i18nMsg('admin-backup-auto-enabled', 'Activée')}
			</label>
			<label class="flex flex-col gap-1 text-sm">
				{i18nMsg('admin-backup-auto-frequency', 'Fréquence')}
				<select
					bind:value={intervalHours}
					class="h-9 rounded-md border border-border bg-background px-2 text-sm"
					data-testid="auto-backup-frequency"
				>
					{#each BACKUP_INTERVAL_HOURS as hours (hours)}
						<option value={hours}>{frequencyLabel(hours)}</option>
					{/each}
				</select>
			</label>
			<label class="flex flex-col gap-1 text-sm">
				{i18nMsg('admin-backup-auto-retention', 'Nombre de sauvegardes conservées')}
				<input
					type="number"
					min={RETENTION_MIN}
					max={RETENTION_MAX}
					bind:value={retentionCount}
					class="h-9 w-24 rounded-md border border-border bg-background px-2 text-sm"
					data-testid="auto-backup-retention"
				/>
			</label>
			<Button
				onclick={save}
				disabled={saving}
				aria-busy={saving}
				data-testid="auto-backup-save"
			>
				{i18nMsg('admin-backup-auto-save', 'Enregistrer')}
			</Button>
		</div>

		{#if conflict}
			<div
				role="alert"
				class="flex flex-wrap items-center gap-3 rounded-md border border-destructive bg-destructive/10 px-3 py-2 text-sm text-destructive"
				data-testid="auto-backup-conflict"
			>
				<span>
					{i18nMsg(
						'admin-backup-auto-conflict',
						"Le réglage a été modifié entre-temps. Rechargez-le avant de l'enregistrer.",
					)}
				</span>
				<Button variant="outline" onclick={loadSchedule} data-testid="auto-backup-reload">
					{i18nMsg('admin-backup-auto-reload', 'Recharger')}
				</Button>
			</div>
		{/if}
		{#if saveError}
			<div
				role="alert"
				class="rounded-md border border-destructive bg-destructive/10 px-3 py-2 text-sm text-destructive"
				data-testid="auto-backup-save-error"
			>
				{saveError}
			</div>
		{/if}

		<!-- État -->
		<dl class="grid grid-cols-1 gap-x-4 gap-y-1 text-sm sm:grid-cols-[auto_1fr]" data-testid="auto-backup-state">
			<dt class="text-text-muted">{i18nMsg('admin-backup-auto-last-success', 'Dernière réussite')}</dt>
			<dd data-testid="auto-backup-last-success">
				{#if schedule.lastSuccessAt}
					{formatBackupDate(schedule.lastSuccessAt, locale)}
					{#if schedule.lastSuccessBytes !== null}
						({formatBackupSize(schedule.lastSuccessBytes, locale)})
					{/if}
				{:else}
					{i18nMsg('admin-backup-auto-none', 'Aucune')}
				{/if}
			</dd>
			{#if showComplete}
				<dt class="text-text-muted">
					{i18nMsg('admin-backup-auto-last-complete', 'Dernière sauvegarde complète')}
				</dt>
				<dd data-testid="auto-backup-last-complete">
					<span class="break-all">{schedule.lastCompleteFile}</span>
					{#if completeMissing}
						<span class="text-destructive" data-testid="auto-backup-last-complete-missing">
							{i18nMsg('admin-backup-auto-last-complete-missing', '(introuvable sur le serveur)')}
						</span>
					{/if}
				</dd>
			{/if}
			<dt class="text-text-muted">{i18nMsg('admin-backup-auto-last-attempt', 'Dernière tentative')}</dt>
			<dd data-testid="auto-backup-last-attempt">
				{#if schedule.lastAttemptAt}
					{formatBackupDate(schedule.lastAttemptAt, locale)}
					{#if schedule.lastStatus === 'ok'}
						— {i18nMsg('admin-backup-auto-status-ok', 'réussie')}
					{/if}
				{:else}
					{i18nMsg('admin-backup-auto-never-attempted', 'Jamais tentée')}
				{/if}
			</dd>
			<dt class="text-text-muted">{i18nMsg('admin-backup-auto-next', 'Prochaine prévue')}</dt>
			<dd data-testid="auto-backup-next">
				{#if schedule.enabled && schedule.nextDueAt}
					{formatBackupDate(schedule.nextDueAt, locale)}
				{:else}
					{i18nMsg('admin-backup-auto-disabled', 'Sauvegarde automatique désactivée')}
				{/if}
			</dd>
		</dl>

		{#if failure}
			<div
				role="alert"
				class="rounded-md border border-destructive bg-destructive/10 px-3 py-2 text-sm text-destructive"
				data-testid="auto-backup-failure"
				data-status={failure}
			>
				{failureMessage(failure)}
			</div>
		{:else if schedule.lastStatus === 'partial'}
			<div
				role="status"
				class="rounded-md border border-warning bg-warning/10 px-3 py-2 text-sm text-text"
				data-testid="auto-backup-partial"
			>
				{i18nMsg(
					'admin-backup-auto-status-partial',
					"Sauvegarde incomplète — pièces non incluses : { $count } (voir le journal du serveur). La dernière sauvegarde complète est conservée.",
					{ count: schedule.lastFilesNotIncluded ?? 0 },
				)}
			</div>
		{/if}

		<div class="flex flex-wrap items-center gap-3">
			<Button
				variant="outline"
				onclick={runNow}
				disabled={running}
				aria-busy={running}
				data-testid="auto-backup-run"
			>
				{running
					? i18nMsg('admin-backup-auto-running', 'Sauvegarde en cours…')
					: i18nMsg('admin-backup-auto-run', 'Sauvegarder maintenant')}
			</Button>
		</div>
		{#if runError}
			<div
				role="alert"
				class="rounded-md border border-destructive bg-destructive/10 px-3 py-2 text-sm text-destructive"
				data-testid="auto-backup-run-error"
			>
				{runError}
			</div>
		{/if}
	{/if}

	<p class="text-xs text-text-muted" data-testid="auto-backup-reminder">
		{i18nMsg(
			'admin-backup-auto-reminder',
			"Ces sauvegardes sont écrites dans le dossier de sauvegarde de l'installation (./backup/auto avec les fichiers docker-compose fournis), sur le même disque que Kesh : copiez-les ailleurs, par exemple avec Hyper Backup. Elles contiennent des données sensibles. Chacune pèse à peu près la base plus le dossier des documents : le dossier auto en garde le nombre conservé plus une, et le dossier de sauvegarde y ajoute une sauvegarde par import, jamais purgée. Le réglage de la sauvegarde automatique ne voyage pas dans le fichier .keshbackup : il appartient à l'installation, comme le fichier .env.",
		)}
	</p>
</section>

<section class="flex flex-col gap-3 rounded-md border border-border p-4" data-testid="server-backups-section">
	<h2 class="text-lg font-semibold text-text">
		{i18nMsg('admin-backup-server-title', 'Sauvegardes sur le serveur')}
	</h2>
	{#if listError}
		<div
			role="alert"
			class="rounded-md border border-destructive bg-destructive/10 px-3 py-2 text-sm text-destructive"
			data-testid="server-backups-error"
		>
			{listError}
		</div>
	{:else if backups && backups.items.length === 0}
		<p class="text-sm text-text-muted" data-testid="server-backups-empty">
			{i18nMsg('admin-backup-server-empty', 'Aucune sauvegarde sur le serveur.')}
		</p>
	{:else if backups}
		<ul class="flex flex-col divide-y divide-border text-sm" data-testid="server-backups-list">
			{#each backups.items as backup (backup.kind + '/' + backup.name)}
				<li class="flex flex-wrap items-center gap-x-4 gap-y-1 py-2" data-testid="server-backup-row" data-kind={backup.kind}>
					<span class="min-w-0 flex-1 break-all font-mono text-xs">{backup.name}</span>
					<span class="text-text-muted">
						{backup.kind === 'auto'
							? i18nMsg('admin-backup-server-kind-auto', 'Automatique')
							: i18nMsg('admin-backup-server-kind-pre-import', 'Avant import')}
					</span>
					<span>{formatBackupSize(backup.sizeBytes, locale)}</span>
					<span class="text-text-muted">{formatBackupDate(backup.modifiedAt, locale)}</span>
					<Button
						variant="outline"
						size="sm"
						onclick={() => download(backup)}
						disabled={downloading !== null}
						data-testid="server-backup-download"
					>
						{i18nMsg('admin-backup-server-download', 'Télécharger')}
					</Button>
					{#if exceedsImportLimit(backup.sizeBytes, backups.importMaxMib)}
						<span class="w-full text-xs text-destructive" data-testid="server-backup-too-large">
							{i18nMsg(
								'admin-backup-server-too-large',
								"trop grande pour être restaurée par l'écran (limite { $mib } Mio)",
								{ mib: backups.importMaxMib },
							)}
						</span>
					{/if}
				</li>
			{/each}
		</ul>
	{/if}
</section>
