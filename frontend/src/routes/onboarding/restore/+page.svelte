<!--
  Story 31-1b (AC15, #503) — restaurer une sauvegarde depuis l'assistant, sur une
  installation neuve (après `/setup`, avant toute configuration). Rend le panneau
  d'import de l'application, `AdminRestorePanel` (C-31-1-10) : même confirmation,
  mêmes avis, même « Se reconnecter ». `/admin/restore` reste injoignable tant que
  l'assistant n'est pas terminé (garde de `(app)/+layout.ts`), d'où cette page.
  Garde de rôle : `+page.ts`.
-->
<script lang="ts">
	import { onMount } from 'svelte';
	import AdminRestorePanel from '$lib/features/admin-restore/AdminRestorePanel.svelte';
	import { i18nMsg, loadI18nMessages } from '$lib/shared/utils/i18n.svelte';

	// La page peut être ouverte avant le choix de la langue (étape 0) : les
	// messages se chargent comme sur l'assistant ; à défaut, les replis français.
	onMount(() => {
		void loadI18nMessages();
	});
</script>

<div class="flex flex-col gap-6">
	<a
		href="/onboarding"
		class="text-sm text-primary underline hover:no-underline"
		data-testid="onboarding-restore-back"
	>
		{i18nMsg('onboarding-restore-back', "← Retour à l'assistant")}
	</a>
	<AdminRestorePanel />
</div>
