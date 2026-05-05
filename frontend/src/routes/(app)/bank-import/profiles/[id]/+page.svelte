<script lang="ts">
	import { page } from '$app/state';
	import BankProfileForm from '$lib/features/bank-import/BankProfileForm.svelte';
	import { getBankProfile } from '$lib/features/bank-import/bank-profile.api';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import type { BankProfile } from '$lib/features/bank-import/bank-profile.types';

	let profile = $state<BankProfile | null>(null);
	let isLoading = $state(true);
	let errorMessage = $state<string | null>(null);

	$effect(() => {
		const id = Number(page.params.id);
		if (!Number.isFinite(id)) {
			errorMessage = 'ID invalide';
			isLoading = false;
			return;
		}
		void getBankProfile(id)
			.then((p) => {
				profile = p;
				isLoading = false;
			})
			.catch((err) => {
				errorMessage = `Profil introuvable : ${String(err)}`;
				isLoading = false;
			});
	});

	function onUpdated(updated: BankProfile): void {
		profile = updated;
	}
</script>

<h1 class="text-2xl font-semibold">
	{i18nMsg('bank-profile-labels-page-title-edit', 'Éditer le profil bancaire')}
</h1>

{#if isLoading}
	<p class="mt-4 text-text-muted">Chargement…</p>
{:else if errorMessage}
	<p class="mt-4 text-error">{errorMessage}</p>
{:else if profile}
	<div class="mt-6">
		<BankProfileForm existing={profile} onSuccess={onUpdated} />
	</div>
{/if}
