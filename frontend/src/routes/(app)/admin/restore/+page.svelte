<!--
  Story 17-3d — Route page /admin/restore (Admin only via +page.ts guard +
  sidebar isAdmin). Thin wrapper : logique + UI dans le composant feature
  `AdminRestorePanel`. 2ᵉ route sous `(app)/admin/` (après /admin/backup, 17-3b).
-->
<script lang="ts">
	import AdminRestorePanel from '$lib/features/admin-restore/AdminRestorePanel.svelte';
</script>

<AdminRestorePanel />
