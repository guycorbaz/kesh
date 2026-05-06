<script lang="ts">
	import BankProfileList from '$lib/features/bank-import/BankProfileList.svelte';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
</script>

<h1 data-testid="bank-import-profile-page-title" class="text-2xl font-semibold">
	{i18nMsg('bank-import-profile-labels-page-title', 'Profils bancaires CSV')}
</h1>

<div class="mt-4">
	<a
		href="/bank-import/profiles/new"
		data-testid="bank-import-profile-new-link"
		class="rounded bg-primary px-4 py-2 text-white"
	>
		{i18nMsg('bank-import-profile-labels-new-profile', 'Nouveau profil')}
	</a>
</div>

<div class="mt-6">
	<BankProfileList />
</div>
