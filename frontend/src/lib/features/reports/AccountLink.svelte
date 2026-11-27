<script lang="ts">
	// Story 55-2 (D6, AC-1, AC-16) — le numéro d'un compte dans un rapport : un lien vers son grand livre sur
	// la période donnée, le même partout (titre, classe, repli en texte). C'est le SEUL site des vues des
	// Rapports qui appelle `ledgerHref` : la garde DRY de l'AC-1 le vérifie
	// (`grep -rn "ledgerHref(" frontend/src/lib/features/reports/*.svelte`).
	//
	// Réutilisable hors des Rapports (C-55-2-4) : il ne lit rien de la page, l'appelant passe la période.
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { LINK_CLASS } from '$lib/shared/navigation/link-class';
	import { ledgerHref } from './reports.api';

	interface Props {
		/** Le compte ; `null` (numéro qui ne se résout pas en compte) : le numéro en texte. */
		accountId: number | null;
		/** Le numéro affiché. */
		number: string;
		/** Début de la période du lien (`AAAA-MM-JJ`). */
		from: string;
		/** Fin de la période du lien (`AAAA-MM-JJ`). */
		to: string;
		/** Faux : le numéro en texte (p. ex. le titre de l'unique section d'un grand livre). */
		linked?: boolean;
		/** `data-testid` du lien ou du texte, choisi par la vue. */
		testId?: string;
	}
	let { accountId, number, from, to, linked = true, testId }: Props = $props();
</script>

{#if linked && accountId !== null}
	<a
		class={LINK_CLASS}
		href={ledgerHref(accountId, from, to)}
		title={i18nMsg('reports-ledger-open-from-balance', 'Voir le détail dans le grand livre')}
		data-testid={testId}>{number}</a
	>
{:else}
	<span data-testid={testId}>{number}</span>
{/if}
