<script lang="ts">
	/**
	 * Tooltip bilingue pour les termes comptables (FR73, UX-DR39).
	 *
	 * Affiche sur survol ou focus 2 lignes :
	 * - Ligne 1 (strong) : langage naturel pour débutant
	 * - Ligne 2 (muted) : terminologie comptable classique
	 *
	 * Les clés i18n suivent le pattern `tooltip-{term}-natural` et
	 * `tooltip-{term}-technical`. Le composant est volontairement placé
	 * dans `shared/components/` (pas `features/journal-entries/`) pour
	 * être réutilisable par les stories futures (contacts, factures).
	 *
	 * **Exemple d'usage** :
	 * ```svelte
	 * <AccountingTooltip term="debit">
	 *   <span class="cursor-help">Débit</span>
	 * </AccountingTooltip>
	 * ```
	 *
	 * **Note structure HTML** : à utiliser INTERNE au conteneur (`<th>`,
	 * `<label>`, `<span>`), jamais wrapper externe un `<th>` entier (qui
	 * casserait la structure `<tr>`).
	 */

	import * as Tooltip from '$lib/components/ui/tooltip';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import type { Snippet } from 'svelte';

	interface Props {
		/** Terme comptable. Suffixé par `-natural` et `-technical` pour les clés i18n. */
		term: string;
		/** Contenu cliquable/hoverable qui déclenche le tooltip. */
		children: Snippet;
	}

	let { term, children }: Props = $props();

	// $derived pour que les clés suivent toute modification réactive de `term`.
	const naturalKey = $derived(`tooltip-${term}-natural`);
	const technicalKey = $derived(`tooltip-${term}-technical`);
</script>

<Tooltip.Root>
	<Tooltip.Trigger>
		{@render children()}
	</Tooltip.Trigger>
	<Tooltip.Content class="max-w-xs">
		<p class="font-semibold">{i18nMsg(naturalKey, term)}</p>
		<p class="text-xs text-muted-foreground mt-1">
			{i18nMsg(technicalKey, term)}
		</p>
	</Tooltip.Content>
</Tooltip.Root>
