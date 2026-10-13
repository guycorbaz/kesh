<!--
  Story 8-2 T6 — Liste des profils CSV bancaires.
-->
<script lang="ts">
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { errorMessageOf } from '$lib/shared/utils/error-message';
	import { notifyError } from '$lib/shared/utils/notify';
	import { listBankProfiles, deleteBankProfile } from './bank-profile.api';
	import type { BankProfile } from './bank-profile.types';

	let profiles = $state<BankProfile[]>([]);
	let isLoading = $state(true);
	let errorMessage = $state<string | null>(null);

	async function load(): Promise<void> {
		isLoading = true;
		try {
			const res = await listBankProfiles();
			profiles = res.items;
		} catch (err) {
			// Story 28-2 (#520) : le message du refus se suffit — un préfixe
			// en dur n'était ni traduit, ni exact pour une panne réseau.
			errorMessage = errorMessageOf(err);
		} finally {
			isLoading = false;
		}
	}

	async function handleDelete(id: number): Promise<void> {
		if (!confirm(i18nMsg('bank-import-profile-labels-confirm-delete', 'Supprimer ce profil ?'))) {
			return;
		}
		// Story 28-2 (C-28-2-20) : sans `try/catch`, un refus devenait un rejet
		// non géré et rien ne s'affichait. Une notification, et non
		// `errorMessage`, qui remplacerait toute la liste pour un refus ponctuel.
		try {
			await deleteBankProfile(id);
			await load();
		} catch (err) {
			notifyError(errorMessageOf(err));
		}
	}

	$effect(() => {
		void load();
	});
</script>

<section data-testid="bank-import-profile-list">
	{#if isLoading}
		<p class="text-text-muted">Chargement…</p>
	{:else if errorMessage}
		<p class="text-error">{errorMessage}</p>
	{:else if profiles.length === 0}
		<p class="text-text-muted">
			{i18nMsg('bank-import-profile-labels-no-profiles', 'Aucun profil bancaire configuré.')}
		</p>
	{:else}
		<table class="w-full table-auto text-sm">
			<thead>
				<tr>
					<th class="text-left">{i18nMsg('bank-import-profile-labels-bank-name', 'Banque')}</th>
					<th class="text-left">{i18nMsg('bank-import-profile-labels-filename-pattern', 'Pattern')}</th>
					<th class="text-left">{i18nMsg('bank-import-profile-labels-encoding', 'Encodage')}</th>
					<th class="text-left">
						<!-- L'en-tête de la colonne d'actions était VIDE : `axe` le relève en
						     `empty-table-header`, un lecteur d'écran annonçant une colonne sans
						     nom. Le libellé est masqué visuellement mais lu — même patron que
						     `ReconciliationProposals.svelte:238`. -->
						<span class="sr-only">
							{i18nMsg('bank-import-profile-labels-actions', 'Actions')}
						</span>
					</th>
				</tr>
			</thead>
			<tbody>
				{#each profiles as profile (profile.id)}
					<tr data-testid="bank-import-profile-list-row">
						<td>{profile.bankName}</td>
						<td><code>{profile.filenamePattern ?? '—'}</code></td>
						<td>{profile.encoding ?? 'auto'}</td>
						<td>
							<a href="/bank-import/profiles/{profile.id}" data-testid="bank-import-profile-edit-link">
								{i18nMsg('bank-import-profile-labels-edit', 'Éditer')}
							</a>
							<button
								type="button"
								onclick={() => handleDelete(profile.id)}
								data-testid="bank-import-profile-delete-button"
								class="ml-2 text-error"
							>
								{i18nMsg('bank-import-profile-labels-delete', 'Supprimer')}
							</button>
						</td>
					</tr>
				{/each}
			</tbody>
		</table>
	{/if}
</section>
