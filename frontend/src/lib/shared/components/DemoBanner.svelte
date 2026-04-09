<script lang="ts">
	import { goto } from '$app/navigation';
	import { Button } from '$lib/components/ui/button';
	import * as Dialog from '$lib/components/ui/dialog';
	import { onboardingState, i18nMsg } from '$lib/features/onboarding/onboarding.svelte';
	import { toast } from 'svelte-sonner';

	let showConfirm = $state(false);
	let resetting = $state(false);

	function msg(key: string, fallback: string): string {
		return i18nMsg(key, fallback);
	}

	async function handleReset() {
		resetting = true;
		try {
			await onboardingState.resetDemo();
			showConfirm = false;
			goto('/onboarding');
		} catch {
			toast.error(msg('error-internal', 'Erreur lors de la réinitialisation'));
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
	<Button variant="outline" size="sm" onclick={() => (showConfirm = true)}>
		{msg('demo-banner-reset', 'Réinitialiser pour la production')}
	</Button>
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
