<!--
  Story 32-10b (refs #723) — l'éditeur d'une nouvelle version d'administrateur (AC-9).

  Deux champs, pré-remplis par le détail avec la description et le texte
  ENREGISTRÉS de la version en vigueur (ou de *Reprendre cette version*) —
  jamais avec le texte servi. Ce qui est saisi part EXACT (`onEnregistrer`
  reçoit la valeur des champs, sans `trim` ni normalisation) : le serveur
  normalise et valide, et son 400 est affiché par le détail.

  **Aucun contrôle côté écran** : ni compteur, ni aperçu des caractères cachés
  (l'écran n'a aucune logique Unicode) ; l'aide dit les bornes du serveur.

  **P26** : pré-rempli par une version d'agent, l'éditeur dit qu'enregistrer en
  fait la version de l'Administrateur, servie SANS la marque.

  L'état d'envoi est celui de la page (`envoiEnCours`, un seul pour tous les
  gestes, C-32-10b-18) ; `occupe` dit que ce geste-ci est en cours (`aria-busy`).
-->
<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import type { Origine } from './mcp-prompts.types';

	type Props = {
		description: string;
		texte: string;
		/** L'origine de la version qui a pré-rempli l'éditeur ; nulle s'il est vide. */
		origine: Origine | null;
		/** Le numéro que portera la version enregistrée. */
		numeroSuivant: number;
		envoiEnCours: boolean;
		occupe: boolean;
		onEnregistrer: (description: string, texte: string) => void;
	};
	let {
		description = $bindable(),
		texte = $bindable(),
		origine,
		numeroSuivant,
		envoiEnCours,
		occupe,
		onEnregistrer,
	}: Props = $props();

	function enregistrer() {
		if (envoiEnCours) return;
		onEnregistrer(description, texte);
	}
</script>

<section class="space-y-3 rounded-md border border-border p-4" data-testid="mcp-prompt-editor">
	<h2 class="text-lg font-semibold">{i18nMsg('mcp-prompts-editor-title', 'Écrire une nouvelle version')}</h2>

	{#if origine === 'agent'}
		<div
			class="rounded-md border border-warning bg-warning/10 px-3 py-2 text-sm"
			role="note"
			data-testid="mcp-prompt-editor-agent-origin"
		>
			{i18nMsg(
				'mcp-prompts-editor-agent-origin',
				'Ce texte a été écrit par un agent. L’enregistrer en fait votre version : elle sera servie sans la marque d’un prompt d’agent. Relisez-le comme s’il venait de vous.',
			)}
		</div>
	{/if}

	<label class="block space-y-1 text-sm">
		<span>{i18nMsg('mcp-prompts-field-description', 'Description')}</span>
		<input
			class="w-full rounded-md border border-border bg-surface px-3 py-2"
			dir="ltr"
			spellcheck="false"
			bind:value={description}
			data-testid="mcp-prompt-editor-description"
		/>
	</label>
	<label class="block space-y-1 text-sm">
		<span>{i18nMsg('mcp-prompts-field-text', 'Texte')}</span>
		<textarea
			class="w-full rounded-md border border-border bg-surface px-3 py-2 font-mono text-sm"
			rows="14"
			dir="ltr"
			spellcheck="false"
			bind:value={texte}
			data-testid="mcp-prompt-editor-text"
		></textarea>
	</label>
	<p class="text-sm text-text-muted">
		{i18nMsg(
			'mcp-prompts-bounds-help',
			'La description tient sur une ligne, 500 caractères au plus ; le texte compte 16 000 caractères au plus. Kesh refuse tout caractère invisible.',
		)}
	</p>
	<div class="space-y-1">
		<Button
			onclick={enregistrer}
			disabled={envoiEnCours}
			aria-busy={occupe ? 'true' : undefined}
			data-testid="mcp-prompt-editor-save"
		>
			{i18nMsg('mcp-prompts-editor-save', 'Enregistrer comme nouvelle version')}
		</Button>
		<p class="text-sm text-text-muted" data-testid="mcp-prompt-editor-next">
			{i18nMsg(
				'mcp-prompts-editor-next',
				'Une nouvelle version n° { $numero } est créée ; les versions précédentes restent à l’historique.',
				{ numero: numeroSuivant },
			)}
		</p>
	</div>
</section>
