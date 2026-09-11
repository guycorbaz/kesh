<script lang="ts">
	import { goto } from '$app/navigation';
	import { Button } from '$lib/components/ui/button';
	import * as Dialog from '$lib/components/ui/dialog';
	import { onboardingState, i18nMsg } from '$lib/features/onboarding/onboarding.svelte';
	import { authState } from '$lib/app/stores/auth.svelte';
	import { isApiError } from '$lib/shared/utils/api-client';
	import { toast } from 'svelte-sonner';

	let showConfirm = $state(false);
	let resetting = $state(false);

	// ⛔ Story 25-1a (#377) — la réinitialisation exige le rôle Admin depuis que la
	// route est passée dans le bloc `admin_routes`. Le bouton est MASQUÉ et non
	// désactivé : un contrôle visible mais mort fait chercher la panne du mauvais
	// côté. Patron `isAdmin` déjà en usage pour la navigation admin-only.
	let isAdmin = $derived(authState.currentUser?.role === 'Admin');

	function msg(key: string, fallback: string): string {
		return i18nMsg(key, fallback);
	}

	async function handleReset() {
		resetting = true;
		try {
			await onboardingState.resetDemo();
			showConfirm = false;
			goto('/onboarding');
		} catch (e) {
			// ⚠️ Un 403 n'est pas une panne. Le message générique faisait chercher
			// la cause du mauvais côté — Story 25-1a (#377).
			const forbidden = isApiError(e) && e.status === 403;
			toast.error(
				forbidden
					? msg('demo-reset-forbidden', 'Seul un administrateur peut réinitialiser cette instance')
					: msg('demo-reset-error', 'Erreur lors de la réinitialisation')
			);
		} finally {
			resetting = false;
		}
	}
</script>

<div
	class="flex items-center justify-between bg-yellow-100 px-4 py-2 text-sm text-yellow-900"
	role="status"
>
	<span>{msg('demo-banner-text', 'Instance de démonstration — données fictives')}</span>
	{#if isAdmin}
		<Button variant="outline" size="sm" onclick={() => (showConfirm = true)}>
			{msg('demo-banner-reset', 'Réinitialiser pour la production')}
		</Button>
	{/if}
</div>

<Dialog.Root bind:open={showConfirm}>
	<Dialog.Content>
		<Dialog.Header>
			<Dialog.Title>{msg('demo-reset-confirm-title', "Réinitialiser l'instance")}</Dialog.Title>
			<Dialog.Description>
				{msg('demo-reset-confirm-body', 'Toutes les données de démonstration seront supprimées. Voulez-vous continuer ?')}
			</Dialog.Description>
		</Dialog.Header>
		<Dialog.Footer>
			<Button variant="outline" onclick={() => (showConfirm = false)} disabled={resetting}>
				{msg('demo-reset-confirm-cancel', 'Annuler')}
			</Button>
			<Button variant="destructive" onclick={handleReset} disabled={resetting}>
				{resetting ? '...' : msg('demo-reset-confirm-ok', 'Confirmer')}
			</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>
