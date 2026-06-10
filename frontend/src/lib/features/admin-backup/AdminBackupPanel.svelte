<!--
  Story 17-3b — Panneau d'export complet d'installation (.keshbackup).
  Logique + UI extraites en composant feature (testable + ownership i18n des
  clés `backup-*`). La route `(app)/admin/backup/+page.svelte` ne fait que le
  rendre. Consomme `downloadFullExport` → `GET /api/v1/admin/full-export`
  (Story 17-3a, Admin strict + anti-PAT). Gating RBAC doublé : sidebar `isAdmin`
  (UX) + `require_admin_role` backend (sécurité).

  ⚠️ Distinct de l'« Export global » per-company (Story 9-2b, /export) : ici
  c'est l'installation COMPLÈTE (toutes sociétés + utilisateurs + système).
-->
<script lang="ts">
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { isApiError } from '$lib/shared/utils/api-client';
	import { Button } from '$lib/components/ui/button';
	import { toast } from 'svelte-sonner';
	import { downloadFullExport } from './admin-backup.api';

	let exporting = $state(false);
	let errorMsg = $state<string | null>(null);

	async function startExport(): Promise<void> {
		// Guard de ré-entrance (AC9) : un seul export à la fois.
		if (exporting) return;
		exporting = true;
		errorMsg = null;
		try {
			await downloadFullExport();
			toast.success(
				i18nMsg('admin-backup-toast-success', "Sauvegarde de l'installation téléchargée."),
			);
		} catch (err) {
			// `getBlob` jette une `ApiError` sur non-2xx (ex. 403 si la page est
			// atteinte par un non-Admin via URL directe, 500). Message lisible.
			const msg = isApiError(err)
				? err.message
				: i18nMsg(
						'admin-backup-error-generic',
						"Échec de l'export de l'installation. Réessayez dans quelques instants.",
					);
			errorMsg = msg;
			toast.error(msg);
		} finally {
			exporting = false;
		}
	}
</script>

<svelte:head>
	<title
		>{i18nMsg('admin-backup-page-title', "Sauvegarde complète de l'installation")} – Kesh</title
	>
</svelte:head>

<section class="flex flex-col gap-4" data-testid="admin-backup-panel">
	<header class="flex flex-col gap-1">
		<h1 class="text-2xl font-semibold text-text" data-testid="admin-backup-title">
			{i18nMsg('admin-backup-page-title', "Sauvegarde complète de l'installation")}
		</h1>
		<p class="text-sm text-text-muted" data-testid="admin-backup-description">
			{i18nMsg(
				'admin-backup-page-description',
				"Télécharge l'intégralité de l'installation (toutes les sociétés, les utilisateurs et les données système) dans un fichier .keshbackup unique, pour migrer ou sauvegarder. À distinguer de l'export global d'une seule société.",
			)}
		</p>
	</header>

	{#if errorMsg}
		<div
			role="alert"
			class="rounded-md border border-danger/40 bg-danger/10 px-3 py-2 text-sm text-danger"
			data-testid="admin-backup-error"
		>
			{errorMsg}
		</div>
	{/if}

	<div>
		<Button
			onclick={startExport}
			disabled={exporting}
			aria-busy={exporting}
			data-testid="admin-backup-export-button"
		>
			{exporting
				? i18nMsg('admin-backup-action-exporting', 'Export en cours…')
				: i18nMsg('admin-backup-action-export', "Exporter toute l'installation")}
		</Button>
	</div>

	<p class="text-xs text-text-muted">
		{i18nMsg(
			'admin-backup-page-hint-secret',
			'Le fichier .keshbackup contient des données sensibles (identifiants, jetons). Conservez-le en lieu sûr.',
		)}
	</p>
</section>
