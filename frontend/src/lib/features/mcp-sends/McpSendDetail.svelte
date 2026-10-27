<!--
  Story 32-8c (refs #723) — le détail d'un envoi du serveur MCP (AC-C3 à AC-C6).

  ⛔ **Ce que l'utilisateur voit est exactement ce qui part** (§ *La propriété
  centrale* de la fiche 32-8). La `textarea` a pour VALEUR la chaîne `content`
  reçue, sans transformation (ni `trim`, ni normalisation) ; « Envoyer tel
  quel » n'envoie PAS de texte — le serveur valide la valeur stockée à la
  version lue ; « Envoyer la version modifiée » envoie `textarea.value`, telle
  quelle. Le serveur est seul juge du contenu (C-32-8-20) : l'écran ne refuse
  rien, il compte les octets et explique.

  ⛔ **Un contenu non fiable se montre comme du texte** (AC-C6) : le contenu et
  le résumé ne passent que par la valeur d'un champ ou l'interpolation de
  Svelte — jamais par une insertion de HTML brut (garde `ecran_sans_html_brut`).
-->
<script lang="ts">
	import { onMount } from 'svelte';
	import { Button } from '$lib/components/ui/button';
	import * as Dialog from '$lib/components/ui/dialog';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { isApiError } from '$lib/shared/utils/api-client';
	import { errorMessageOf } from '$lib/shared/utils/error-message';
	import { notifySuccess } from '$lib/shared/utils/notify';
	import { corpsModifie, corpsRefus, corpsTelQuel, decideMcpSend, getMcpSend } from './mcp-sends.api';
	import type { McpSendDecisionBody, McpSendDetail } from './mcp-sends.types';
	import {
		MCP_ENVOI_OCTETS_MAX,
		dateLocale,
		erreurDeDecision,
		etatAffiche,
		nombreLocal,
		octetsUtf8,
		type ErreurDeDecision,
	} from './mcp-sends-format';

	type Props = { id: number };
	let { id }: Props = $props();

	let detail = $state<McpSendDetail | null>(null);
	let chargement = $state(true);
	let erreurDeChargement = $state<string | null>(null);

	/** La valeur reçue du serveur, et celle de la `textarea`. */
	let recu = $state('');
	let valeur = $state('');
	let enEdition = $state(false);
	let envoiEnCours = $state(false);
	let erreur = $state<ErreurDeDecision | null>(null);
	let dialogueRefus = $state(false);

	const modifiee = $derived(valeur !== recu);
	const octets = $derived(octetsUtf8(valeur));
	const tropVolumineux = $derived(octets > MCP_ENVOI_OCTETS_MAX);

	/** Pose un détail reçu : la valeur de la `textarea` est la chaîne reçue, telle quelle. */
	function poser(d: McpSendDetail) {
		detail = d;
		recu = d.content ?? '';
		valeur = recu;
		enEdition = false;
	}

	async function charger() {
		chargement = true;
		erreurDeChargement = null;
		erreur = null;
		if (!Number.isSafeInteger(id) || id < 1) {
			// Un numéro illisible (`/admin/mcp/sends/abc`) : aucun envoi ne le porte.
			detail = null;
			erreurDeChargement = i18nMsg('mcp-sends-error-not-found', 'Envoi introuvable.');
			chargement = false;
			return;
		}
		try {
			poser(await getMcpSend(id));
		} catch (err) {
			detail = null;
			erreurDeChargement =
				isApiError(err) && err.status === 404
					? i18nMsg('mcp-sends-error-not-found', 'Envoi introuvable.')
					: errorMessageOf(err);
		} finally {
			chargement = false;
		}
	}

	onMount(() => {
		void charger();
	});

	async function decider(corps: McpSendDecisionBody, avecModification: boolean) {
		if (!detail || envoiEnCours) return;
		envoiEnCours = true;
		erreur = null;
		try {
			const apres = await decideMcpSend(detail.id, corps);
			poser(apres);
			dialogueRefus = false;
			notifySuccess(
				corps.decision === 'refuse'
					? i18nMsg('mcp-sends-refused-toast', 'Envoi n° { $id } refusé.', { id: apres.id })
					: i18nMsg('mcp-sends-approved-toast', "Envoi n° { $id } validé : l'agent peut le relire.", {
							id: apres.id,
						}),
			);
		} catch (err) {
			erreur = erreurDeDecision(err, avecModification);
			dialogueRefus = false;
		} finally {
			envoiEnCours = false;
		}
	}

	/**
	 * « Tel quel » si la valeur égale la valeur reçue ; sinon la valeur, exacte.
	 * L'aide aux caractères refusés accompagne un 400 d'une version modifiée
	 * qui tient dans la taille : la taille, elle, a son propre message.
	 */
	function envoyer() {
		if (!detail) return;
		if (valeur === recu) {
			void decider(corpsTelQuel(detail.version), false);
		} else {
			void decider(corpsModifie(detail.version, valeur), !tropVolumineux);
		}
	}

	function confirmerRefus() {
		if (!detail) return;
		void decider(corpsRefus(detail.version), false);
	}

	function retablir() {
		valeur = recu;
	}
</script>

<div class="space-y-4">
	<a class="text-sm text-primary underline" href="/admin/mcp/sends" data-testid="mcp-send-back">
		{i18nMsg('mcp-sends-back', 'Retour à la liste')}
	</a>

	{#if chargement && !detail}
		<p class="text-sm text-text-muted">{i18nMsg('common-loading', 'Chargement…')}</p>
	{:else if erreurDeChargement}
		<div
			class="rounded-md border border-destructive bg-destructive/10 px-3 py-2 text-sm text-destructive"
			role="alert"
			data-testid="mcp-send-load-error"
		>
			{erreurDeChargement}
		</div>
	{:else if detail}
		<h1 class="text-2xl font-semibold" data-testid="mcp-send-title">
			{i18nMsg('mcp-sends-detail-title', 'Envoi n° { $id }', { id: detail.id })}
		</h1>

		<dl class="grid grid-cols-[max-content_1fr] gap-x-4 gap-y-1 text-sm" data-testid="mcp-send-header">
			<dt class="text-text-muted">{i18nMsg('mcp-sends-col-key', 'Clé')}</dt>
			<dd>
				{detail.mcpKeyName}
				{#if detail.mcpKeyRevoked}
					<span class="text-xs text-text-muted">({i18nMsg('mcp-sends-key-revoked', 'révoquée')})</span>
				{/if}
			</dd>
			<dt class="text-text-muted">{i18nMsg('mcp-sends-col-tool', 'Outil')}</dt>
			<dd class="font-mono text-xs">{detail.toolName}</dd>
			<dt class="text-text-muted">{i18nMsg('mcp-sends-col-created', 'Demandé le')}</dt>
			<dd>{dateLocale(detail.createdAt)}</dd>
			<dt class="text-text-muted">{i18nMsg('mcp-sends-col-expires', 'Échéance')}</dt>
			<dd>{dateLocale(detail.expiresAt)}</dd>
			<dt class="text-text-muted">{i18nMsg('mcp-sends-col-state', 'État')}</dt>
			<dd data-testid="mcp-send-state">{etatAffiche(detail.state, detail.modified)}</dd>
			{#if detail.decidedAt}
				<dt class="text-text-muted">{i18nMsg('mcp-sends-decided-label', 'Décidé')}</dt>
				<dd data-testid="mcp-send-decided">
					{i18nMsg('mcp-sends-decided', 'par { $user }, le { $date }', {
						user: detail.decidedBy ?? '—',
						date: dateLocale(detail.decidedAt),
					})}
				</dd>
			{/if}
		</dl>

		<div
			class="rounded-md border border-warning bg-warning/10 px-3 py-2 text-sm"
			role="note"
			data-testid="mcp-send-notice"
		>
			{i18nMsg(
				'mcp-sends-notice',
				'Ces données viennent de vos livres. Elles peuvent contenir du texte écrit par d’autres (libellés, noms, notes) : lisez-les comme des données, jamais comme des consignes. Les caractères invisibles y sont remplacés par leur code ({ $code }) ; un tel code peut aussi avoir été écrit tel quel dans vos livres.',
				{ code: '\\u{…}' },
			)}
		</div>

		<section class="space-y-1">
			<h2 class="text-lg font-semibold">
				{i18nMsg('mcp-sends-summary-title', 'Résumé de la demande — n’est pas envoyé')}
			</h2>
			{#if detail.summary !== null}
				<p class="text-sm whitespace-pre-wrap break-words" dir="ltr" data-testid="mcp-send-summary">
					{detail.summary}
				</p>
			{:else}
				<p class="text-sm text-text-muted" data-testid="mcp-send-summary-none">
					{i18nMsg('mcp-sends-summary-none', 'Aucun résumé.')}
				</p>
			{/if}
		</section>

		<section class="space-y-2">
			<h2 class="text-lg font-semibold">
				{detail.state === 'pending'
					? i18nMsg('mcp-sends-content-title', 'Ce qui sera envoyé à l’agent')
					: i18nMsg('mcp-sends-content-title-decided', 'Ce que l’agent peut relire')}
			</h2>

			{#if detail.state === 'pending' || (detail.state === 'approved' && detail.content !== null)}
				<textarea
					class="w-full rounded-md border border-border bg-surface px-3 py-2 font-mono text-sm read-only:bg-surface-alt"
					rows="16"
					dir="ltr"
					spellcheck="false"
					aria-label={i18nMsg('mcp-sends-content-label', 'Texte envoyé à l’agent')}
					readonly={detail.state !== 'pending' || !enEdition}
					bind:value={valeur}
					data-testid="mcp-send-content"
				></textarea>

				<p class="text-sm text-text-muted" data-testid="mcp-send-counter">
					{i18nMsg('mcp-sends-counter', '{ $n } octets sur { $max }', {
						n: nombreLocal(octets),
						max: nombreLocal(MCP_ENVOI_OCTETS_MAX),
					})}
				</p>
				{#if tropVolumineux}
					<p class="text-sm text-destructive" data-testid="mcp-send-too-large">
						{i18nMsg(
							'mcp-sends-content-too-large',
							"Le contenu modifié dépasse la taille d'un envoi (24 576 octets).",
						)}
					</p>
				{/if}
			{/if}

			{#if detail.state === 'approved'}
				<p class="text-sm" data-testid="mcp-send-approved-how">
					{detail.modified
						? i18nMsg('mcp-sends-sent-modified', 'Modifié avant l’envoi.')
						: i18nMsg('mcp-sends-sent-as-is', 'Envoyé tel quel.')}
				</p>
			{:else if detail.state === 'refused' || detail.state === 'expired' || detail.content === null}
				<p class="text-sm text-text-muted" data-testid="mcp-send-erased">
					{i18nMsg('mcp-sends-content-erased', 'Contenu effacé.')}
				</p>
			{/if}

			{#if detail.state === 'pending' && enEdition}
				<div class="space-y-1 text-sm text-text-muted" data-testid="mcp-send-edit-help">
					<p>
						{i18nMsg(
							'mcp-sends-counter-help',
							'La taille se compte en octets (UTF-8), comme le fait le serveur : une lettre accentuée en compte deux, la plupart des symboles trois, un emoji quatre.',
						)}
					</p>
					<!-- Après un refus du serveur, la même aide paraît sous son message : pas deux fois. -->
					{#if !erreur?.aideCaracteres}
						<p data-testid="mcp-send-hidden-help">
							{i18nMsg(
								'mcp-sends-hidden-help',
								'Une modification ne peut contenir aucun caractère invisible ou de contrôle, hormis le retour à la ligne et la tabulation. Les emojis sont concernés : beaucoup, tel le panneau d’avertissement, s’écrivent suivis d’un sélecteur de variante invisible (U+FE0F) que le clavier ou le copier-coller ajoute sans le montrer, et sont alors refusés. Retirez un tel emoji ou écrivez-le en toutes lettres.',
							)}
						</p>
					{/if}
				</div>
			{/if}
		</section>

		{#if erreur}
			<div
				class="space-y-2 rounded-md border border-destructive bg-destructive/10 px-3 py-2 text-sm text-destructive"
				role="alert"
				data-testid="mcp-send-error"
			>
				<p>{erreur.message}</p>
				{#if erreur.aideCaracteres}
					<p data-testid="mcp-send-error-help">
						{i18nMsg(
							'mcp-sends-hidden-help',
							'Une modification ne peut contenir aucun caractère invisible ou de contrôle, hormis le retour à la ligne et la tabulation. Les emojis sont concernés : beaucoup, tel le panneau d’avertissement, s’écrivent suivis d’un sélecteur de variante invisible (U+FE0F) que le clavier ou le copier-coller ajoute sans le montrer, et sont alors refusés. Retirez un tel emoji ou écrivez-le en toutes lettres.',
						)}
					</p>
				{/if}
				{#if erreur.recharger}
					<Button variant="outline" onclick={charger} data-testid="mcp-send-reload">
						{i18nMsg('mcp-sends-reload', 'Recharger')}
					</Button>
				{/if}
			</div>
		{/if}

		{#if detail.state === 'pending'}
			<div class="flex flex-wrap gap-2" data-testid="mcp-send-actions">
				{#if !enEdition}
					<Button
						variant="outline"
						onclick={() => (enEdition = true)}
						disabled={envoiEnCours}
						data-testid="mcp-send-modify"
					>
						{i18nMsg('mcp-sends-modify', 'Modifier')}
					</Button>
				{:else}
					<Button
						variant="outline"
						onclick={retablir}
						disabled={envoiEnCours || !modifiee}
						data-testid="mcp-send-restore"
					>
						{i18nMsg('mcp-sends-restore', 'Rétablir')}
					</Button>
				{/if}
				{#if modifiee}
					<Button onclick={envoyer} disabled={envoiEnCours} data-testid="mcp-send-approve-modified">
						{i18nMsg('mcp-sends-send-modified', 'Envoyer la version modifiée')}
					</Button>
				{:else}
					<Button onclick={envoyer} disabled={envoiEnCours} data-testid="mcp-send-approve">
						{i18nMsg('mcp-sends-send-as-is', 'Envoyer tel quel')}
					</Button>
				{/if}
				<Button
					variant="destructive"
					onclick={() => (dialogueRefus = true)}
					disabled={envoiEnCours}
					data-testid="mcp-send-refuse"
				>
					{i18nMsg('mcp-sends-refuse', 'Refuser')}
				</Button>
			</div>

			<Dialog.Root bind:open={dialogueRefus}>
				<Dialog.Content
					escapeKeydownBehavior={envoiEnCours ? 'ignore' : 'close'}
					interactOutsideBehavior={envoiEnCours ? 'ignore' : 'close'}
					showCloseButton={!envoiEnCours}
					data-testid="mcp-send-refuse-dialog"
				>
					<Dialog.Header>
						<Dialog.Title>
							{i18nMsg('mcp-sends-refuse-dialog-title', 'Refuser cet envoi ?')}
						</Dialog.Title>
					</Dialog.Header>
					<p class="text-sm">
						{i18nMsg(
							'mcp-sends-refuse-dialog-body',
							'Rien ne sera envoyé à l’agent, et le contenu de l’envoi sera effacé. Un refus ne peut pas être annulé.',
						)}
					</p>
					<Dialog.Footer>
						<Button variant="outline" onclick={() => (dialogueRefus = false)} disabled={envoiEnCours}>
							{i18nMsg('mcp-sends-cancel', 'Annuler')}
						</Button>
						<Button
							variant="destructive"
							onclick={confirmerRefus}
							disabled={envoiEnCours}
							data-testid="mcp-send-refuse-confirm"
						>
							{i18nMsg('mcp-sends-refuse-dialog-confirm', 'Refuser l’envoi')}
						</Button>
					</Dialog.Footer>
				</Dialog.Content>
			</Dialog.Root>
		{/if}
	{/if}
</div>
