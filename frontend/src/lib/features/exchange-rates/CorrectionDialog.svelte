<!--
  Story 37-2 (AC-6, U3, A23) — dialogue du ↑ d'une correction de l'AFC.

  ⚠️ **Spécifié, non rendu aujourd'hui** : l'écran ne l'ouvre que si la condition
  unique `canManageSeveralCompanies` est vraie (constante `false` tant qu'un
  utilisateur n'appartient qu'à une société, `users.company_id`). Elle gouverne le
  dialogue entier, non la seule option « toutes les sociétés » : le jour où elle
  devient vraie, le dialogue apparaît sans réécrire l'écran. L'option « toutes les
  sociétés sous votre gestion » reste désactivée tant que la route du ↑ ne porte
  pas la portée `managedCompanies` (AC-5, spécifiée, non livrée).
-->
<script lang="ts">
	import * as Dialog from '$lib/components/ui/dialog';
	import { Button } from '$lib/components/ui/button';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';

	interface Props {
		open: boolean;
		companyName: string;
		/** Le cours retenu par la société (affiché). */
		retained: string;
		/** La valeur corrigée par l'AFC (affichée, envoyée au ↑). */
		corrected: string;
		/** Date de la correction, JJ.MM.AAAA. */
		correctedOn: string;
		onConfirm: () => void;
		onCancel: () => void;
	}

	let { open = $bindable(), companyName, retained, corrected, correctedOn, onConfirm, onCancel }: Props =
		$props();
</script>

<Dialog.Root
	bind:open
	onOpenChange={(o) => {
		if (!o) onCancel();
	}}
>
	<Dialog.Content data-testid="exchange-rates-correction-dialog">
		<Dialog.Header>
			<Dialog.Title>{i18nMsg('exchange-rates-correction-title', 'Reprendre la valeur corrigée')}</Dialog.Title>
			<Dialog.Description>
				{i18nMsg(
					'exchange-rates-correction-body',
					'Cours retenu : { $retained } ; valeur corrigée par l’AFC : { $corrected } (le { $date }). La reprise ne modifie aucune écriture passée : une ligne au cours AFC précédent reste admise tant qu’on ne la modifie pas ; modifiée, elle passe en cours saisi ou prend le nouveau cours.',
					{ retained, corrected, date: correctedOn },
				)}
			</Dialog.Description>
		</Dialog.Header>
		<fieldset class="mt-2 flex flex-col gap-1 text-sm">
			<label class="flex items-center gap-2">
				<input type="radio" name="exchange-rates-correction-scope" checked />
				{i18nMsg('exchange-rates-correction-scope-company', 'Corriger pour { $company }', {
					company: companyName,
				})}
			</label>
			<label class="flex items-center gap-2 text-text-muted">
				<input type="radio" name="exchange-rates-correction-scope" disabled />
				{i18nMsg(
					'exchange-rates-correction-scope-managed',
					'Corriger pour toutes les sociétés sous votre gestion (exercices ouverts seulement)',
				)}
			</label>
		</fieldset>
		<Dialog.Footer class="mt-4">
			<Button variant="outline" type="button" onclick={onCancel} data-testid="exchange-rates-correction-cancel">
				{i18nMsg('common-cancel', 'Annuler')}
			</Button>
			<Button type="button" onclick={onConfirm} data-testid="exchange-rates-correction-confirm">
				{i18nMsg('exchange-rates-correction-confirm', 'Corriger')}
			</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>
