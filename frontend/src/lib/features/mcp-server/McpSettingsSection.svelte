<!--
  Story 32-7 (refs #723) — la section « Réglages du serveur » de l'écran Serveur MCP
  (AC-5, AC-6, AC-7).

  Lire, enregistrer (remplacement complet, `version` lue), et dire la forme retenue :
  au succès, les champs sont REMPLACÉS par la réponse du serveur, qui met chaque entrée
  sous sa forme canonique (#738 B2 = E4). L'écran ne valide pas les adresses ; il
  retire les lignes vides et calcule des avertissements informatifs, jamais bloquants
  (C-32-7-12), sur les champs tels qu'affichés. L'encadré « Ce que le filtre ne garantit
  pas » est permanent (décision de Guy, 2026-10-25).
-->
<script lang="ts">
	import { onMount } from 'svelte';
	import { Button } from '$lib/components/ui/button';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { isApiError } from '$lib/shared/utils/api-error';
	import { errorMessageOf } from '$lib/shared/utils/error-message';
	import { getMcpServerSettings, updateMcpServerSettings } from './mcp-server.api';
	import type { McpServerSettings } from './mcp-server.types';
	import { dateHeureAffichee } from './mcp-time';
	import { aPrefixeNul, entreesDe, familleDe, texteDe } from './mcp-reglages';

	/** Les réglages LUS (ou enregistrés) : la `version` envoyée et l'état affiché en vient. */
	let lus = $state<McpServerSettings | null>(null);
	let chargement = $state(true);
	let erreurLecture = $state<string | null>(null);

	let actif = $state(false);
	let filtre = $state(true);
	let plages = $state('');
	let mandataires = $state('');
	let origines = $state('');

	let envoi = $state(false);
	let succes = $state(false);
	let erreur = $state<string | null>(null);

	const adresse = new URL('/mcp', window.location.origin).href;

	function remplir(r: McpServerSettings) {
		lus = r;
		actif = r.enabled;
		filtre = r.addressFilterEnabled;
		plages = texteDe(r.allowedNetworks);
		mandataires = texteDe(r.trustedProxies);
		origines = texteDe(r.allowedOrigins);
	}

	/**
	 * Lit les réglages. Rend `true` si la lecture a réussi. Tant qu'ils ne sont pas lus
	 * (`lus === null`), l'écran n'affiche ni état, ni cases, ni listes, ni avertissements :
	 * il n'affirme rien d'un état inconnu (revue de code P1, E3).
	 */
	async function charger(): Promise<boolean> {
		chargement = true;
		erreurLecture = null;
		try {
			remplir(await getMcpServerSettings());
			return true;
		} catch (err) {
			lus = null;
			erreurLecture = errorMessageOf(err);
			return false;
		} finally {
			chargement = false;
		}
	}

	onMount(() => {
		void charger();
	});

	async function enregistrer(event?: Event) {
		event?.preventDefault();
		if (envoi || lus === null) return;
		envoi = true;
		succes = false;
		erreur = null;
		try {
			const r = await updateMcpServerSettings({
				enabled: actif,
				addressFilterEnabled: filtre,
				allowedNetworks: entreesDe(plages),
				trustedProxies: entreesDe(mandataires),
				allowedOrigins: entreesDe(origines),
				version: lus.version,
			});
			remplir(r);
			succes = true;
		} catch (err) {
			if (isApiError(err) && err.status === 409) {
				// « rechargés » ne se dit qu'après une relecture réussie ; sinon l'échec de
				// lecture parle seul (revue de code P1, B8).
				if (await charger()) {
					erreur = i18nMsg(
						'mcp-server-settings-conflict',
						'Les réglages ont changé entre-temps ; ils sont rechargés, vos modifications ne sont pas enregistrées.',
					);
				}
			} else {
				erreur = errorMessageOf(err);
			}
		} finally {
			envoi = false;
		}
	}

	const plagesNulles = $derived(entreesDe(plages).filter(aPrefixeNul));
	const mandatairesNuls = $derived(filtre ? entreesDe(mandataires).filter(aPrefixeNul) : []);
	const personne = $derived(filtre && entreesDe(plages).length === 0);
	const modifieLe = $derived(lus !== null && lus.version > 0 ? dateHeureAffichee(lus.updatedAt) : null);
</script>

<section class="space-y-4 rounded-md border border-border p-4" data-testid="mcp-server-settings">
	<h2 class="text-lg font-semibold">{i18nMsg('mcp-server-settings-title', 'Réglages du serveur')}</h2>

	<div class="space-y-2 rounded-md border border-border bg-muted/40 px-3 py-2 text-sm" data-testid="mcp-server-guarantee">
		<p class="font-semibold">{i18nMsg('mcp-server-guarantee-title', 'Ce que le filtre ne garantit pas')}</p>
		<ul class="list-disc space-y-1 pl-5">
			<li>
				{i18nMsg(
					'mcp-server-guarantee-proxy',
					'Le filtre juge l’adresse que voit Kesh. Derrière un mandataire non déclaré parmi les mandataires de confiance — Traefik, Nginx, le proxy inversé du NAS —, tous les clients paraissent venir de lui ; derrière une publication de port Docker par le mandataire en espace utilisateur, tous peuvent paraître venir de la passerelle Docker. Ce sont des adresses privées, admises par défaut : le filtre n’arrête alors personne.',
				)}
			</li>
			<li>
				{i18nMsg(
					'mcp-server-guarantee-firewall',
					'Il ne remplace ni le pare-feu ni le VPN : n’activez pas le serveur MCP sur une installation exposée à Internet, /mcp est servi par le même port que l’application.',
				)}
			</li>
			<li>
				{i18nMsg(
					'mcp-server-guarantee-tailscale',
					'La plage 100.64.0.0/10 (VPN de type Tailscale) n’est pas admise par défaut : ajoutez-la vous-même aux plages admises.',
				)}
			</li>
			<li>{i18nMsg('mcp-server-guarantee-rate', 'Aucune limite de débit n’est posée sur /mcp.')}</li>
		</ul>
	</div>

	{#if chargement}
		<p class="text-sm text-text-muted" role="status" data-testid="mcp-server-settings-loading">
			{i18nMsg('mcp-server-settings-loading', 'Chargement des réglages…')}
		</p>
	{/if}

	{#if erreurLecture !== null}
		<div
			class="rounded-md border border-destructive bg-destructive/10 px-3 py-2 text-sm text-destructive"
			role="alert"
			data-testid="mcp-server-settings-load-error"
		>
			{erreurLecture}
		</div>
		<p class="text-sm" data-testid="mcp-server-settings-load-failed">
			{i18nMsg(
				'mcp-server-settings-load-failed',
				'Les réglages n’ont pas pu être lus : tant qu’ils ne le sont pas, rien ne s’enregistre depuis cet écran. Si l’erreur persiste, voir le manuel d’administration, § Activer le serveur.',
			)}
		</p>
		<Button variant="outline" onclick={() => charger()} disabled={chargement} data-testid="mcp-server-settings-retry">
			{i18nMsg('mcp-server-settings-retry', 'Réessayer')}
		</Button>
	{/if}

	<form
		class="space-y-4"
		onsubmit={enregistrer}
		oninput={() => (succes = false)}
		onchange={() => (succes = false)}
		data-testid="mcp-server-settings-form"
	>
		<div class="space-y-1">
			<p class="text-sm">
				{i18nMsg('mcp-server-address-label', 'Adresse du serveur :')}
				<code class="rounded bg-muted px-1 font-mono" data-testid="mcp-server-address">{adresse}</code>
			</p>
			<p class="text-sm text-text-muted" data-testid="mcp-server-address-note">
				{i18nMsg(
					'mcp-server-address-note',
					'exactement /mcp, sans barre finale. C’est l’adresse que voit ce navigateur : derrière un mandataire, un agent du réseau local peut devoir en employer une autre.',
				)}
			</p>
			<p class="text-sm text-text-muted" data-testid="mcp-server-restore-note">
				{i18nMsg(
					'mcp-server-restore-note',
					'Une restauration rend ces réglages tels qu’ils étaient dans la sauvegarde : un serveur actif dans la sauvegarde l’est après l’import.',
				)}
			</p>
			<p class="text-sm text-text-muted" data-testid="mcp-server-reset-note">
				{i18nMsg(
					'mcp-server-reset-note',
					'La réinitialisation des données de démonstration efface ces réglages (serveur désactivé) et conserve les clés.',
				)}
			</p>
		</div>

		<!-- Tant que les réglages ne sont pas lus, rien de ce qui suit ne paraît : l'écran
		     n'affirme rien d'un état inconnu ; pendant l'envoi, les champs sont figés. -->
		{#if lus !== null}
		<fieldset class="space-y-4" disabled={envoi} data-testid="mcp-server-settings-fields">
		<div class="space-y-1">
			<label class="flex items-center gap-2 text-sm font-medium">
				<input type="checkbox" bind:checked={actif} data-testid="mcp-server-settings-enabled" />
				{i18nMsg('mcp-server-settings-enabled', 'Serveur activé')}
			</label>
			<p class="text-sm text-text-muted" data-testid="mcp-server-state">
				{#if lus.enabled}
					{i18nMsg(
						'mcp-server-state-on',
						'Activé : le serveur répond aux clés MCP de cette société, depuis les adresses admises.',
					)}
				{:else}
					{i18nMsg('mcp-server-state-off', 'Désactivé : /mcp répond comme une adresse qui n’existe pas.')}
				{/if}
			</p>
		</div>

		<label class="flex items-center gap-2 text-sm font-medium">
			<input type="checkbox" bind:checked={filtre} data-testid="mcp-server-settings-filter" />
			{i18nMsg('mcp-server-settings-filter', 'Filtre d’adresse activé')}
		</label>

		<p class="text-sm text-text-muted">
			<span>{i18nMsg('mcp-server-settings-one-per-line', 'Une entrée par ligne.')}</span>
			<span>{i18nMsg('mcp-server-settings-canonical', 'Kesh enregistre chaque entrée sous sa forme canonique.')}</span>
		</p>

		<div class="grid gap-4 md:grid-cols-3">
			<label class="block space-y-1 text-sm">
				<span class="font-medium">{i18nMsg('mcp-server-settings-networks', 'Plages admises')}</span>
				<textarea
					class="w-full rounded-md border border-border bg-surface px-3 py-2 font-mono text-sm"
					rows="8"
					dir="ltr"
					spellcheck="false"
					bind:value={plages}
					data-testid="mcp-server-settings-networks"
				></textarea>
			</label>
			<label class="block space-y-1 text-sm">
				<span class="font-medium">{i18nMsg('mcp-server-settings-proxies', 'Mandataires de confiance')}</span>
				<textarea
					class="w-full rounded-md border border-border bg-surface px-3 py-2 font-mono text-sm"
					rows="8"
					dir="ltr"
					spellcheck="false"
					bind:value={mandataires}
					data-testid="mcp-server-settings-proxies"
				></textarea>
			</label>
			<label class="block space-y-1 text-sm">
				<span class="font-medium">{i18nMsg('mcp-server-settings-origins', 'Origines admises')}</span>
				<textarea
					class="w-full rounded-md border border-border bg-surface px-3 py-2 font-mono text-sm"
					rows="8"
					dir="ltr"
					spellcheck="false"
					bind:value={origines}
					data-testid="mcp-server-settings-origins"
				></textarea>
				<span class="block text-text-muted">
					{i18nMsg(
						'mcp-server-settings-origins-help',
						'Une origine admise n’ouvre le serveur à aucune page web : elle ne sert qu’aux clients qui envoient un en-tête Origin sans être des navigateurs.',
					)}
				</span>
			</label>
		</div>

		{#if !filtre || plagesNulles.length > 0 || mandatairesNuls.length > 0 || personne}
			<ul class="space-y-1 rounded-md border border-border px-3 py-2 text-sm" data-testid="mcp-server-warnings">
				{#if !filtre}
					<li data-testid="mcp-server-warning-filter-off">
						{i18nMsg('mcp-server-warning-filter-off', 'Filtre désactivé : toute adresse est admise.')}
					</li>
				{/if}
				{#each plagesNulles as entree, i (i)}
					<li data-testid="mcp-server-warning-network-any">
						{#if familleDe(entree) === 'IPv6'}
							{i18nMsg(
								'mcp-server-warning-network-any-v6',
								'{ $entry } admet tout client IPv6 ; pour admettre les deux familles, il faut les deux entrées.',
								{ entry: entree },
							)}
						{:else}
							{i18nMsg(
								'mcp-server-warning-network-any-v4',
								'{ $entry } admet tout client IPv4 ; pour admettre les deux familles, il faut les deux entrées.',
								{ entry: entree },
							)}
						{/if}
					</li>
				{/each}
				{#each mandatairesNuls as entree, i (i)}
					<li data-testid="mcp-server-warning-proxy-any">
						{#if familleDe(entree) === 'IPv6'}
							{i18nMsg(
								'mcp-server-warning-proxy-any-v6',
								'{ $entry } : tout pair IPv6 est cru sur son en-tête X-Forwarded-For.',
								{ entry: entree },
							)}
						{:else}
							{i18nMsg(
								'mcp-server-warning-proxy-any-v4',
								'{ $entry } : tout pair IPv4 est cru sur son en-tête X-Forwarded-For.',
								{ entry: entree },
							)}
						{/if}
					</li>
				{/each}
				{#if personne}
					<li data-testid="mcp-server-warning-nobody">
						{i18nMsg('mcp-server-warning-nobody', 'Filtre activé et aucune plage admise : personne n’est admis.')}
					</li>
				{/if}
			</ul>
		{/if}

		<p class="text-sm text-text-muted" data-testid="mcp-server-settings-updated">
			{#if modifieLe !== null}
				{i18nMsg('mcp-server-settings-updated', 'Dernière modification : { $date }', { date: modifieLe })}
			{:else}
				{i18nMsg('mcp-server-settings-never', 'Jamais modifiés — valeurs par défaut.')}
			{/if}
		</p>
		</fieldset>
		{/if}

		{#if erreur !== null}
			<div
				class="rounded-md border border-destructive bg-destructive/10 px-3 py-2 text-sm text-destructive"
				role="alert"
				data-testid="mcp-server-settings-error"
			>
				{erreur}
			</div>
		{/if}
		{#if succes}
			<p class="text-sm" role="status" data-testid="mcp-server-settings-saved">
				{i18nMsg('mcp-server-settings-saved', 'Réglages enregistrés.')}
			</p>
		{/if}

		<Button
			type="submit"
			disabled={chargement || envoi || lus === null}
			aria-busy={envoi ? 'true' : undefined}
			data-testid="mcp-server-settings-save"
		>
			{i18nMsg('mcp-server-settings-save', 'Enregistrer')}
		</Button>
	</form>
</section>
