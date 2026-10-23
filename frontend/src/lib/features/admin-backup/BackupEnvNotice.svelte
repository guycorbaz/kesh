<!--
  Story 31-1b (AC17, #503) — avis permanent : le fichier .keshbackup ne contient pas
  la configuration du serveur (.env). Rendu par `AdminBackupPanel` (export) et par
  `AdminRestorePanel` (import, depuis l'application ou depuis l'assistant).

  Un seul composant, une seule clé (`admin-backup-not-included-env`) : la clé vit
  dans le dossier qui la possède (règle `lint-i18n-ownership`), et le panneau de
  restauration importe le composant au lieu de la clé (C-31-1-69).
-->
<script lang="ts">
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
</script>

<div
	role="note"
	class="rounded-md border border-border bg-surface-alt px-3 py-2 text-sm text-text"
	data-testid="backup-env-notice"
>
	{i18nMsg(
		'admin-backup-not-included-env',
		"Le fichier ne contient pas la configuration du serveur (fichier .env : messagerie, secret de session, dossiers…). Conservez-la à part : sur une nouvelle machine, recréez-la avant d'importer.",
	)}
</div>
