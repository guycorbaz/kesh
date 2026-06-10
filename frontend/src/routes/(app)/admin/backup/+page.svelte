<!--
  Story 17-3b — Route page /admin/backup (Admin only via sidebar isAdmin +
  backend require_admin_role). Thin wrapper : toute la logique + l'UI vivent
  dans le composant feature `AdminBackupPanel` (testable, ownership i18n
  `backup-*`). Première route sous `(app)/admin/` côté front ; 17-3d ajoutera
  `/admin/restore`.
-->
<script lang="ts">
	import AdminBackupPanel from '$lib/features/admin-backup/AdminBackupPanel.svelte';
</script>

<AdminBackupPanel />
