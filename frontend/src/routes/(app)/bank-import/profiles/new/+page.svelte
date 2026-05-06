<script lang="ts">
	import BankProfileForm from '$lib/features/bank-import/BankProfileForm.svelte';
	import { goto } from '$app/navigation';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import type { BankProfile } from '$lib/features/bank-import/bank-profile.types';

	function onCreated(profile: BankProfile): void {
		void goto(`/bank-import/profiles/${profile.id}`);
	}
</script>

<h1 class="text-2xl font-semibold">
	{i18nMsg('bank-import-profile-labels-page-title-new', 'Nouveau profil bancaire')}
</h1>

<div class="mt-6">
	<BankProfileForm onSuccess={onCreated} />
</div>
