<!--
  Story 48-12b (#975, #985) — l'avis d'un refus de doublon (E4, E5, F4), d'une issue
  inconnue (E6) ou d'une pièce déjà enregistrée (E9).

  Un texte, puis, s'il y en a un, un lien qui s'ouvre **dans un nouvel onglet**
  (`target="_blank" rel="noopener"` : la saisie, figée ou non, reste à l'écran pour la
  comparaison), puis les boutons que l'appelant passe (`children`). Le texte d'un refus
  de doublon est celui du serveur (clés `error-supplier-invoice-*`), lu par l'appelant.
-->
<script lang="ts">
	import type { Snippet } from 'svelte';

	interface Props {
		/** Le `data-testid` de l'avis. */
		testid: string;
		/** Le texte de l'avis. */
		text: string;
		/** Le lien, ouvert dans un nouvel onglet, ou rien. */
		link?: { href: string; label: string; prefix?: string; testid: string } | null;
		/** Les boutons de l'avis. */
		children?: Snippet;
	}

	let { testid, text, link = null, children }: Props = $props();
</script>

<div class="space-y-2 rounded border border-warning bg-warning-soft p-3 text-sm" role="alert" data-testid={testid}>
	<p data-testid="{testid}-text">{text}</p>
	{#if link}
		<p>
			{#if link.prefix}<span>{link.prefix}</span>{/if}
			<a
				class="text-primary underline"
				href={link.href}
				target="_blank"
				rel="noopener"
				data-testid={link.testid}>{link.label}</a
			>
		</p>
	{/if}
	{#if children}
		<div class="flex flex-wrap gap-2">{@render children()}</div>
	{/if}
</div>
