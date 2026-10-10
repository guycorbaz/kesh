<script lang="ts">
	// Story 15-1c-i (AC11, C-15-1c-22) — le lien d'un code de lettrage vers son
	// groupe (`/open-items?group=<code>`). ⛔ **Aucune clé i18n** : le code est son
	// propre texte — sans quoi `lint-i18n-ownership` refuserait une clé
	// `open-items-*` dans `features/reports` ou `journal-entries`, où la 15-1c-ii
	// emploie ce composant.
	import { groupHref } from './open-items';

	interface Props {
		code: string;
		/** Compte et date de la vue courante, conservés dans le lien s'ils sont connus. */
		accountId?: number | null;
		asOf?: string | null;
	}
	let { code, accountId = null, asOf = null }: Props = $props();
</script>

<a
	href={groupHref(code, accountId, asOf)}
	class="font-mono text-primary underline"
	data-testid="lettering-code-link-{code}">{code}</a
>
