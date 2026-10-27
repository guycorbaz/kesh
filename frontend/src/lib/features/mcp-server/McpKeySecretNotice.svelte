<!--
  Story 32-7 (refs #723) — l'encart du secret d'une clé MCP créée (AC-11).

  Le secret paraît dans UN seul élément `<code>` — jamais dans un `<input>` ni un
  attribut. Ce composant ne le garde pas : il le reçoit du parent, qui le tient dans
  une instance de `SecretUnique` ; *Fermer* rend la main au parent, qui l'oublie.
-->
<script lang="ts">
	import { toast } from 'svelte-sonner';
	import { Button } from '$lib/components/ui/button';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { copyToClipboard } from '$lib/shared/utils/clipboard';

	interface Props {
		nom: string;
		valeur: string;
		onFermer: () => void;
	}

	let { nom, valeur, onFermer }: Props = $props();

	async function copier() {
		if (await copyToClipboard(valeur)) {
			toast.success(i18nMsg('mcp-server-secret-copied', 'Clé copiée dans le presse-papiers.'));
		} else {
			toast.error(
				i18nMsg('mcp-server-secret-copy-failed', 'La copie a échoué : sélectionnez la clé et copiez-la à la main.'),
			);
		}
	}
</script>

<div class="space-y-2 rounded-md border border-border bg-muted/40 p-4 text-sm" role="alert" data-testid="mcp-server-secret-notice">
	<p class="font-semibold">{i18nMsg('mcp-server-secret-created', 'Clé « { $name } » créée.', { name: nom })}</p>
	<p>{i18nMsg('mcp-server-secret-copy-now', 'Copiez-la maintenant : elle ne sera plus jamais affichée.')}</p>
	<p>
		{i18nMsg(
			'mcp-server-secret-client',
			'Donnez-la au client MCP en portée locale ou utilisateur — jamais dans un .mcp.json versionné (manuel d’administration, § Configurer un client).',
		)}
	</p>
	<code class="block break-all rounded bg-surface px-2 py-1 font-mono" data-testid="mcp-server-secret-value">{valeur}</code>
	<div class="flex gap-2">
		<Button variant="outline" onclick={copier} data-testid="mcp-server-secret-copy">
			{i18nMsg('mcp-server-secret-copy', 'Copier')}
		</Button>
		<Button variant="outline" onclick={onFermer} data-testid="mcp-server-secret-close">
			{i18nMsg('mcp-server-secret-close', 'Fermer')}
		</Button>
	</div>
</div>
