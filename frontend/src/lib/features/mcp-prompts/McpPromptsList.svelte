<!--
  Story 32-10b (refs #723) — la liste des prompts du serveur MCP (AC-3) et la
  création d'un prompt (AC-11).

  Un tableau, dans l'ordre du serveur (par nom) : nom, *Standard*, la version
  en vigueur (« n° N », origine, *Retiré*) ou « Non servi », le nombre à
  valider, *Personnalisé* ou « Diffère du standard de cette version de Kesh »
  (le drapeau `standardDiffere` n'est pas affiché tel quel, C-32-10b-19).
  **Aucun texte ni description** : la liste ne lit que la synthèse.

  La création : nom, description, texte, partis EXACTS (fabrique de l'API) ;
  la règle du nom est dite, non contrôlée — le 400 du serveur est affiché et
  la saisie conservée. Un Administrateur seulement : pour tout autre rôle, l'API
  répond 403 et la page montre ce refus.
-->
<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { Button } from '$lib/components/ui/button';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { createMcpPrompt, listMcpPrompts } from './mcp-prompts.api';
	import type { McpPromptSummary } from './mcp-prompts.types';
	import { erreurDeGeste, etiquetteStandard, origineAffichee } from './mcp-prompts-format';

	let prompts = $state<McpPromptSummary[]>([]);
	let chargement = $state(true);
	let erreur = $state<string | null>(null);

	let formulaireOuvert = $state(false);
	let nom = $state('');
	let description = $state('');
	let texte = $state('');
	let envoiEnCours = $state(false);
	let erreurDeCreation = $state<string | null>(null);
	/** Le prompt créé quand la navigation vers son détail a échoué (revue de code P1, B7). */
	let creeSansNavigation = $state<number | null>(null);

	async function charger() {
		chargement = true;
		erreur = null;
		try {
			prompts = await listMcpPrompts();
		} catch (err) {
			erreur = erreurDeGeste(err).message;
		} finally {
			chargement = false;
		}
	}

	onMount(() => {
		void charger();
	});

	/**
	 * La création : les valeurs EXACTES de la saisie ; un seul envoi à la fois. La création
	 * et la navigation sont deux temps : une navigation en échec ne se dit pas en échec de
	 * création (le prompt existe) — l'écran offre un lien vers lui (revue de code P1, B7).
	 */
	async function creer(event?: Event) {
		event?.preventDefault();
		if (envoiEnCours) return;
		envoiEnCours = true;
		erreurDeCreation = null;
		creeSansNavigation = null;
		let ecrite;
		try {
			ecrite = await createMcpPrompt({ nom, description, texte });
		} catch (err) {
			erreurDeCreation = erreurDeGeste(err).message;
			envoiEnCours = false;
			return;
		}
		try {
			await goto(`/admin/mcp/prompts/${ecrite.promptId}`);
		} catch {
			creeSansNavigation = ecrite.promptId;
			formulaireOuvert = false;
			void charger();
		} finally {
			envoiEnCours = false;
		}
	}
</script>

<div class="space-y-4">
	<div class="flex flex-wrap items-center justify-between gap-2">
		<h1 class="text-2xl font-semibold" data-testid="mcp-prompts-title">
			{i18nMsg('mcp-prompts-title', 'Prompts du serveur MCP')}
		</h1>
		<div class="flex gap-2">
			<Button
				variant="outline"
				onclick={() => (formulaireOuvert = true)}
				disabled={formulaireOuvert}
				data-testid="mcp-prompts-new"
			>
				{i18nMsg('mcp-prompts-new', 'Nouveau prompt')}
			</Button>
			<Button variant="outline" onclick={charger} disabled={chargement} data-testid="mcp-prompts-refresh">
				{i18nMsg('mcp-prompts-refresh', 'Actualiser')}
			</Button>
		</div>
	</div>

	<p class="text-sm text-text-muted">
		{i18nMsg(
			'mcp-prompts-intro',
			'Un prompt est une consigne que vous choisissez de donner à un agent depuis son client MCP. Il n’ouvre aucun droit : l’agent reste borné par les portées de sa clé. Une proposition d’un agent n’est servie qu’après votre approbation.',
		)}
	</p>

	{#if creeSansNavigation !== null}
		<p class="text-sm" role="status">
			<a class="text-primary underline" href="/admin/mcp/prompts/{creeSansNavigation}" data-testid="mcp-prompts-created">
				{i18nMsg('mcp-prompts-created-open', 'Prompt créé : ouvrir son détail')}
			</a>
		</p>
	{/if}

	{#if formulaireOuvert}
		<form class="space-y-3 rounded-md border border-border p-4" onsubmit={creer} data-testid="mcp-prompts-create-form">
			<h2 class="text-lg font-semibold">{i18nMsg('mcp-prompts-new', 'Nouveau prompt')}</h2>
			<label class="block space-y-1 text-sm">
				<span>{i18nMsg('mcp-prompts-field-name', 'Nom')}</span>
				<input
					class="w-full rounded-md border border-border bg-surface px-3 py-2 font-mono"
					dir="ltr"
					spellcheck="false"
					bind:value={nom}
					data-testid="mcp-prompts-create-name"
				/>
				<span class="block text-text-muted">
					{i18nMsg(
						'mcp-prompts-name-help',
						'Lettres minuscules sans accent, chiffres, - et _, 64 au plus ; un nom ne commence pas par « kesh », réservé aux prompts standard.',
					)}
				</span>
			</label>
			<label class="block space-y-1 text-sm">
				<span>{i18nMsg('mcp-prompts-field-description', 'Description')}</span>
				<input
					class="w-full rounded-md border border-border bg-surface px-3 py-2"
					dir="ltr"
					spellcheck="false"
					bind:value={description}
					data-testid="mcp-prompts-create-description"
				/>
			</label>
			<label class="block space-y-1 text-sm">
				<span>{i18nMsg('mcp-prompts-field-text', 'Texte')}</span>
				<textarea
					class="w-full rounded-md border border-border bg-surface px-3 py-2 font-mono text-sm"
					rows="10"
					dir="ltr"
					spellcheck="false"
					bind:value={texte}
					data-testid="mcp-prompts-create-text"
				></textarea>
				<span class="block text-text-muted">
					{i18nMsg(
						'mcp-prompts-bounds-help',
						'La description tient sur une ligne, 500 caractères au plus ; le texte compte 16 000 caractères au plus. Kesh refuse tout caractère invisible.',
					)}
				</span>
			</label>
			{#if erreurDeCreation}
				<div
					class="rounded-md border border-destructive bg-destructive/10 px-3 py-2 text-sm text-destructive"
					role="alert"
					data-testid="mcp-prompts-create-error"
				>
					{erreurDeCreation}
				</div>
			{/if}
			<div class="flex gap-2">
				<Button
					type="submit"
					disabled={envoiEnCours}
					aria-busy={envoiEnCours ? 'true' : undefined}
					data-testid="mcp-prompts-create-submit"
				>
					{i18nMsg('mcp-prompts-create', 'Créer')}
				</Button>
				<Button
					variant="outline"
					onclick={() => (formulaireOuvert = false)}
					disabled={envoiEnCours}
					data-testid="mcp-prompts-create-cancel"
				>
					{i18nMsg('mcp-prompts-cancel', 'Annuler')}
				</Button>
			</div>
		</form>
	{/if}

	{#if erreur}
		<div
			class="rounded-md border border-destructive bg-destructive/10 px-3 py-2 text-sm text-destructive"
			role="alert"
			data-testid="mcp-prompts-error"
		>
			{erreur}
		</div>
	{:else if chargement && prompts.length === 0}
		<p class="text-sm text-text-muted">{i18nMsg('common-loading', 'Chargement…')}</p>
	{:else if prompts.length === 0}
		<p class="text-sm text-text-muted" data-testid="mcp-prompts-empty">
			{i18nMsg('mcp-prompts-empty', 'Aucun prompt')}
		</p>
	{:else}
		<div class="overflow-x-auto">
			<table class="w-full text-sm" data-testid="mcp-prompts-table">
				<thead>
					<tr class="border-b border-border text-left">
						<th class="py-2 pr-4">{i18nMsg('mcp-prompts-col-name', 'Nom')}</th>
						<th class="py-2 pr-4">{i18nMsg('mcp-prompts-col-in-force', 'Version en vigueur')}</th>
						<th class="py-2 pr-4">{i18nMsg('mcp-prompts-col-pending', 'À valider')}</th>
						<th class="py-2"></th>
					</tr>
				</thead>
				<tbody>
					{#each prompts as p (p.id)}
						{@const etiquette = etiquetteStandard(p)}
						<tr class="border-b border-border" data-testid="mcp-prompts-row-{p.id}" data-nom={p.nom}>
							<td class="py-2 pr-4">
								<span class="font-mono" data-testid="mcp-prompts-name-{p.id}">{p.nom}</span>
								{#if p.standard}
									<span class="pastille ml-1" data-testid="mcp-prompts-standard-{p.id}">
										{i18nMsg('mcp-prompts-badge-standard', 'Standard')}
									</span>
								{/if}
								{#if etiquette === 'personnalise'}
									<span class="pastille ml-1" data-testid="mcp-prompts-customized-{p.id}">
										{i18nMsg('mcp-prompts-badge-customized', 'Personnalisé')}
									</span>
								{:else if etiquette === 'differe'}
									<span class="pastille pastille-avertissement ml-1" data-testid="mcp-prompts-differs-{p.id}">
										{i18nMsg('mcp-prompts-badge-differs', 'Diffère du standard de cette version de Kesh')}
									</span>
								{/if}
							</td>
							<td class="py-2 pr-4">
								{#if p.enVigueur}
									<span data-testid="mcp-prompts-in-force-{p.id}">
										{i18nMsg('mcp-prompts-version-number', 'n° { $numero }', { numero: p.enVigueur.numero })}
									</span>
									<span
										class="pastille ml-1"
										class:pastille-avertissement={p.enVigueur.origine === 'agent'}
										data-testid="mcp-prompts-origin-{p.id}"
									>
										{origineAffichee(p.enVigueur.origine)}
									</span>
									{#if p.enVigueur.nature === 'withdrawal'}
										<span class="pastille ml-1" data-testid="mcp-prompts-withdrawn-{p.id}">
											{i18nMsg('mcp-prompts-badge-withdrawn', 'Retiré')}
										</span>
									{/if}
								{:else}
									<span class="text-text-muted" data-testid="mcp-prompts-not-served-{p.id}">
										{i18nMsg('mcp-prompts-not-served', 'Non servi')}
									</span>
								{/if}
							</td>
							<td class="py-2 pr-4">
								{#if p.aValider > 0}
									<span class="pastille pastille-avertissement" data-testid="mcp-prompts-pending-{p.id}">
										{i18nMsg('mcp-prompts-pending-count', '{ $n } à valider', { n: p.aValider })}
									</span>
								{/if}
							</td>
							<td class="py-2">
								<a class="text-primary underline" href="/admin/mcp/prompts/{p.id}" data-testid="mcp-prompts-open-{p.id}">
									{i18nMsg('mcp-prompts-open', 'Voir')}
								</a>
							</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>
	{/if}
</div>

<style>
	.pastille {
		display: inline-flex;
		align-items: center;
		border-radius: 9999px;
		padding: 0.125rem 0.5rem;
		font-size: 0.75rem;
		font-weight: 500;
		background-color: color-mix(in srgb, var(--color-text-muted) 18%, transparent);
		color: var(--color-text);
	}
	.pastille-avertissement {
		background-color: color-mix(in srgb, var(--color-warning) 22%, transparent);
	}
</style>
