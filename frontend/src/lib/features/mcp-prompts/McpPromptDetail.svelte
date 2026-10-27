<!--
  Story 32-10b (refs #723) — le détail d'un prompt du serveur MCP : l'en-tête,
  le texte servi par Kesh, les versions à valider, l'historique et les gestes
  (AC-4 à AC-10, AC-12 et son amendement, AC-13, AC-14).

  ⛔ **On montre le texte servi, rendu visible par le serveur** (AC-5) —
  `McpPromptVersion` l'interpole sans aucune transformation ; l'écran n'a
  aucune logique Unicode et ne recompose aucun texte.

  ⛔ **On approuve ce qu'on a vu** (AC-7) : *Approuver* n'existe que sur la
  dernière version, d'agent, à valider et que le serveur dit `servable` ; il
  ouvre un dialogue qui restate ce qui sera servi ; le `POST` porte le `numero`
  et la `version` DE L'OBJET AFFICHÉ — aucun `GET` entre l'affichage et
  l'écriture ; la relecture vient APRÈS.

  ⛔ **Un seul envoi à la fois** (C-32-10b-18) : un état `envoiEnCours` pour
  tous les gestes ; dès la confirmation, tous sont désactivés, le bouton cliqué
  porte `aria-busy`, les dialogues ne se ferment pas ; chaque gestionnaire
  garde son retour anticipé (Svelte appelle le gestionnaire délégué même sur un
  bouton désactivé).

  ⛔ **Rien ne s'écrase** : chaque geste ajoute ou décide une version ; après
  toute relecture réussie, l'éditeur est remis sur la version en vigueur —
  SAUF après un 409 rendu à l'enregistrement d'une nouvelle version : la saisie
  de l'Administrateur survit au rechargement (amendement de l'AC-12, KF-055
  #796, C-32-10b-20), et rien n'est renvoyé d'office.

  ⛔ **Un texte non fiable se montre comme du texte** (AC-13) : interpolation
  ou valeur de champ seulement, jamais d'insertion de HTML brut.
-->
<script lang="ts">
	import { onMount, tick } from 'svelte';
	import { Button } from '$lib/components/ui/button';
	import * as Dialog from '$lib/components/ui/dialog';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { notifySuccess } from '$lib/shared/utils/notify';
	import {
		addMcpPromptVersion,
		approveMcpPromptVersion,
		corpsRetrait,
		corpsTexte,
		getMcpPrompt,
		refuseMcpPromptVersion,
		restoreMcpPromptStandard,
	} from './mcp-prompts.api';
	import type { McpPromptDetail, McpPromptVersion as Version, Origine } from './mcp-prompts.types';
	import {
		dateLocale,
		derniereVersion,
		erreurDeGeste,
		etatAffiche,
		natureAffichee,
		origineAffichee,
		peutApprouver,
		peutRefuser,
		peutRevenirAuStandard,
		versionEnVigueur,
		versionsAValider,
		type ErreurDeGeste,
	} from './mcp-prompts-format';
	import McpPromptVersion from './McpPromptVersion.svelte';
	import McpPromptEditor from './McpPromptEditor.svelte';

	type Props = { id: number };
	let { id }: Props = $props();

	type Geste = 'approuver' | 'refuser' | 'enregistrer' | 'retirer' | 'standard';

	let detail = $state<McpPromptDetail | null>(null);
	let chargement = $state(true);
	let erreurDeChargement = $state<string | null>(null);

	let envoiEnCours = $state(false);
	let gesteEnCours = $state<Geste | null>(null);
	let erreur = $state<ErreurDeGeste | null>(null);
	/** Vrai si l'erreur vient d'un geste de l'éditeur : elle s'affiche près de lui (revue P1, E3). */
	let erreurPresDeLEditeur = $state(false);
	/**
	 * La page est occupée : un envoi ou une lecture est en cours. Tous les gestes sont alors
	 * désactivés (C-32-10b-18 ; et pendant un rechargement, revue de code P1, B3 = E8).
	 */
	const occupe = $derived(envoiEnCours || chargement);
	/** Vrai si le dernier 409 est venu de l'enregistrement d'une saisie : elle survit au rechargement. */
	let saisieAConserver = $state(false);
	/** Vrai après un rechargement qui a conservé la saisie : l'écran le dit. */
	let saisieConservee = $state(false);

	/** L'éditeur : description et texte ENREGISTRÉS d'une version, et l'origine de cette version. */
	let editeurDescription = $state('');
	let editeurTexte = $state('');
	let editeurOrigine = $state<Origine | null>(null);

	let aApprouver = $state<Version | null>(null);
	let aRefuser = $state<Version | null>(null);
	let dialogueRetrait = $state(false);
	let dialogueStandard = $state(false);

	const enVigueur = $derived(detail ? versionEnVigueur(detail) : null);
	const derniere = $derived(detail ? derniereVersion(detail) : null);
	const numeroSuivant = $derived((derniere?.numero ?? 0) + 1);
	const aValider = $derived(detail ? versionsAValider(detail) : []);
	const historique = $derived(detail ? [...detail.versions].sort((a, b) => b.numero - a.numero) : []);
	/** Le bloc *Texte servi par Kesh* : la version en vigueur, si elle est un texte que le serveur sert. */
	const serviParKesh = $derived(
		enVigueur !== null && enVigueur.nature === 'text' && enVigueur.servable ? enVigueur : null,
	);

	/** Pré-remplit l'éditeur avec le texte ENREGISTRÉ d'une version (jamais le texte servi). */
	function preRemplir(v: Version | null) {
		if (v !== null && v.nature === 'text' && v.description !== null && v.texte !== null) {
			editeurDescription = v.description.exact;
			editeurTexte = v.texte.exact;
			editeurOrigine = v.origine;
		} else {
			editeurDescription = '';
			editeurTexte = '';
			editeurOrigine = null;
		}
	}

	/** Pose un détail lu : l'éditeur est remis sur la version en vigueur, sauf saisie à conserver. */
	function poser(d: McpPromptDetail, garderSaisie: boolean) {
		detail = d;
		saisieConservee = garderSaisie;
		if (!garderSaisie) preRemplir(versionEnVigueur(d));
	}

	/**
	 * Lit le détail. Au premier chargement, un échec remplace la page par son message (et
	 * *Réessayer*). Un **rechargement** en échec, lui, garde le détail affiché et la saisie
	 * — conservée ou non — et dit l'erreur avec *Recharger* encore offert : l'amendement de
	 * l'AC-12 tient aussi quand la relecture tombe en panne (revue de code P1, E2 = B2 = A2).
	 * `saisieAConserver` n'est remis à faux qu'après une lecture réussie.
	 */
	async function charger(garderSaisie = false) {
		chargement = true;
		erreurDeChargement = null;
		erreur = null;
		if (!Number.isSafeInteger(id) || id < 1) {
			detail = null;
			erreurDeChargement = i18nMsg('mcp-prompts-error-not-found', 'Prompt introuvable.');
			chargement = false;
			return;
		}
		try {
			const d = await getMcpPrompt(id);
			saisieAConserver = false;
			poser(d, garderSaisie);
		} catch (err) {
			const e = erreurDeGeste(err);
			if (detail === null) {
				erreurDeChargement = e.message;
			} else {
				void afficherErreur({ message: e.message, recharger: true }, garderSaisie);
			}
		} finally {
			chargement = false;
		}
	}

	onMount(() => {
		void charger();
	});

	/** *Recharger*, après un 409 : la saisie survit si le 409 est venu de son enregistrement. */
	function recharger() {
		if (occupe) return;
		void charger(saisieAConserver);
	}

	/** L'élément de l'erreur affichée, pour l'amener sous les yeux (revue de code P1, E3). */
	let elementErreur = $state<HTMLElement | null>(null);

	/**
	 * Affiche une erreur, près de l'éditeur si elle vient d'un geste de l'éditeur — sinon en
	 * tête de page —, puis la fait défiler dans la vue et lui donne le focus : un refus du
	 * serveur ne passe jamais inaperçu (revue de code P1, E3).
	 */
	async function afficherErreur(e: ErreurDeGeste, presDeLEditeur: boolean) {
		erreur = e;
		erreurPresDeLEditeur = presDeLEditeur;
		await tick();
		elementErreur?.scrollIntoView?.({ block: 'center' });
		elementErreur?.focus();
	}

	function fermerDialogues() {
		aApprouver = null;
		aRefuser = null;
		dialogueRetrait = false;
		dialogueStandard = false;
	}

	/**
	 * Un geste : un seul à la fois, et jamais pendant un rechargement ; l'écriture, PUIS la
	 * relecture — **deux temps distincts** (revue de code P1, B1 = E1 = A1) :
	 * - l'écriture refusée : son erreur ; un 409 rendu à l'enregistrement d'une saisie la
	 *   garde pour le rechargement ;
	 * - l'écriture réussie : le succès est notifié aussitôt ; si la relecture échoue
	 *   ensuite, l'écran dit que le geste a eu lieu et offre *Recharger* — il n'annonce
	 *   jamais l'échec d'un geste qui a eu lieu.
	 * Aucun envoi n'est jamais rejoué.
	 */
	async function agir(geste: Geste, ecrire: () => Promise<unknown>, succes: string) {
		if (occupe) return;
		envoiEnCours = true;
		gesteEnCours = geste;
		erreur = null;
		saisieConservee = false;
		const presDeLEditeur = geste === 'enregistrer';
		try {
			try {
				await ecrire();
			} catch (err) {
				const e = erreurDeGeste(err);
				saisieAConserver = presDeLEditeur && e.recharger;
				fermerDialogues();
				void afficherErreur(e, presDeLEditeur);
				return;
			}
			fermerDialogues();
			notifySuccess(succes);
			saisieAConserver = false;
			try {
				poser(await getMcpPrompt(id), false);
			} catch {
				void afficherErreur(
					{
						message: i18nMsg(
							'mcp-prompts-error-reread',
							'Le geste a été enregistré, mais la relecture du prompt a échoué : rechargez-le.',
						),
						recharger: true,
					},
					presDeLEditeur,
				);
			}
		} finally {
			envoiEnCours = false;
			gesteEnCours = null;
		}
	}

	function confirmerApprobation() {
		if (occupe || !detail || !aApprouver) return;
		// Le numéro et la `version` DE L'OBJET AFFICHÉ — jamais une relecture.
		const { numero } = aApprouver;
		const version = detail.version;
		void agir(
			'approuver',
			() => approveMcpPromptVersion(detail!.id, numero, version),
			i18nMsg('mcp-prompts-approved-toast', 'Version n° { $numero } approuvée.', { numero }),
		);
	}

	function confirmerRefus() {
		if (occupe || !detail || !aRefuser) return;
		const { numero } = aRefuser;
		const version = detail.version;
		void agir(
			'refuser',
			() => refuseMcpPromptVersion(detail!.id, numero, version),
			i18nMsg('mcp-prompts-refused-toast', 'Version n° { $numero } refusée.', { numero }),
		);
	}

	function enregistrer(description: string, texte: string) {
		if (occupe || !detail) return;
		const version = detail.version;
		void agir(
			'enregistrer',
			() => addMcpPromptVersion(detail!.id, corpsTexte(version, description, texte)),
			i18nMsg('mcp-prompts-saved-toast', 'Nouvelle version enregistrée.'),
		);
	}

	function confirmerRetrait() {
		if (occupe || !detail) return;
		const version = detail.version;
		void agir(
			'retirer',
			() => addMcpPromptVersion(detail!.id, corpsRetrait(version)),
			i18nMsg('mcp-prompts-withdrawn-toast', 'Prompt retiré.'),
		);
	}

	function confirmerStandard() {
		if (occupe || !detail) return;
		const version = detail.version;
		void agir(
			'standard',
			() => restoreMcpPromptStandard(detail!.id, version),
			i18nMsg('mcp-prompts-standard-toast', 'Prompt revenu au texte standard.'),
		);
	}

	/** Le libellé de la clé auteur d'une proposition. */
	function cleAuteur(v: Version): string {
		return i18nMsg('mcp-prompts-key-label', 'clé n° { $id } ({ $nom })', {
			id: v.cleAuteurId ?? '—',
			nom: v.cleNom ?? '—',
		});
	}

	/** Les réglages d'un dialogue : il ne se ferme pas pendant l'envoi. */
	const fermeture = $derived(envoiEnCours ? 'ignore' : 'close');
</script>

<div class="space-y-4">
	<a class="text-sm text-primary underline" href="/admin/mcp/prompts" data-testid="mcp-prompt-back">
		{i18nMsg('mcp-prompts-back', 'Retour à la liste')}
	</a>

	{#if chargement && !detail}
		<p class="text-sm text-text-muted">{i18nMsg('common-loading', 'Chargement…')}</p>
	{:else if erreurDeChargement}
		<div
			class="rounded-md border border-destructive bg-destructive/10 px-3 py-2 text-sm text-destructive"
			role="alert"
		>
			<p data-testid="mcp-prompt-load-error">{erreurDeChargement}</p>
			<Button variant="outline" onclick={() => charger()} disabled={chargement} data-testid="mcp-prompt-load-retry">
				{i18nMsg('mcp-prompts-retry', 'Réessayer')}
			</Button>
		</div>
	{:else if detail}
		<div class="flex flex-wrap items-center gap-2">
			<h1 class="font-mono text-2xl font-semibold" data-testid="mcp-prompt-title">{detail.nom}</h1>
			{#if detail.standard}
				<span class="pastille" data-testid="mcp-prompt-standard-badge">
					{i18nMsg('mcp-prompts-badge-standard', 'Standard')}
				</span>
			{/if}
		</div>

		<dl class="grid grid-cols-[max-content_1fr] gap-x-4 gap-y-1 text-sm" data-testid="mcp-prompt-header">
			<dt class="text-text-muted">{i18nMsg('mcp-prompts-col-in-force', 'Version en vigueur')}</dt>
			<dd data-testid="mcp-prompt-header-in-force">
				{#if enVigueur}
					{i18nMsg('mcp-prompts-version-number', 'n° { $numero }', { numero: enVigueur.numero })}
					— {origineAffichee(enVigueur.origine)}
					{#if enVigueur.nature === 'withdrawal'}
						— {i18nMsg('mcp-prompts-badge-withdrawn', 'Retiré')}
					{/if}
				{:else}
					{i18nMsg('mcp-prompts-not-served', 'Non servi')}
				{/if}
			</dd>
			<dt class="text-text-muted">{i18nMsg('mcp-prompts-modified', 'Dernière modification')}</dt>
			<dd data-testid="mcp-prompt-modified">{dateLocale(detail.modifieLe)}</dd>
		</dl>

		<!-- L'erreur d'un geste : ici, en tête de page — sauf celle d'un geste de l'éditeur,
		     rendue près de lui (revue de code P1, E3). Un seul site de rendu, deux places. -->
		{#snippet alerte(e: ErreurDeGeste)}
			<div
				class="space-y-2 rounded-md border border-destructive bg-destructive/10 px-3 py-2 text-sm text-destructive"
				role="alert"
				tabindex="-1"
				bind:this={elementErreur}
				data-testid="mcp-prompt-error"
			>
				<p>{e.message}</p>
				{#if e.recharger}
					<Button variant="outline" onclick={recharger} disabled={occupe} data-testid="mcp-prompt-reload">
						{i18nMsg('mcp-prompts-reload', 'Recharger')}
					</Button>
				{/if}
			</div>
		{/snippet}

		{#if erreur && !erreurPresDeLEditeur}
			{@render alerte(erreur)}
		{/if}

		<section class="space-y-2" data-testid="mcp-prompt-served-block">
			<h2 class="text-lg font-semibold">{i18nMsg('mcp-prompts-served-title', 'Texte servi par Kesh')}</h2>
			{#if serviParKesh}
				<div
					class="rounded-md border border-warning bg-warning/10 px-3 py-2 text-sm"
					role="note"
					data-testid="mcp-prompt-notice"
				>
					{i18nMsg(
						'mcp-prompts-notice',
						'Le texte est montré tel que Kesh le servirait ; certains caractères invisibles y sont écrits sous la forme { $code }.',
						{ code: '\\u{…}' },
					)}
				</div>
				<McpPromptVersion version={serviParKesh} zone="servi-" />
			{:else}
				<p class="text-sm" data-testid="mcp-prompt-not-served">
					{i18nMsg('mcp-prompts-not-served-body', 'Ce prompt n’est servi à aucun agent.')}
					<span data-testid="mcp-prompt-not-served-reason">
						{#if enVigueur === null}
							{i18nMsg('mcp-prompts-not-served-none', 'Aucune version n’est approuvée.')}
						{:else if enVigueur.nature === 'withdrawal'}
							{i18nMsg('mcp-prompts-not-served-withdrawn', 'Il est retiré.')}
						{:else}
							{i18nMsg(
								'mcp-prompts-not-served-invalid',
								'Sa version en vigueur n’est pas valide : Kesh refuse de la servir (elle a pu être restaurée d’une sauvegarde ou écrite hors de Kesh).',
							)}
						{/if}
					</span>
				</p>
				{#if enVigueur !== null && enVigueur.nature === 'text'}
					<h3 class="text-sm font-semibold">
						{i18nMsg('mcp-prompts-not-served-text-title', 'Texte de la version en vigueur, non servi')}
					</h3>
					<McpPromptVersion version={enVigueur} zone="non-servi-" />
				{/if}
			{/if}
		</section>

		{#if aValider.length > 0}
			<section class="space-y-3" data-testid="mcp-prompt-pending">
				<h2 class="text-lg font-semibold">{i18nMsg('mcp-prompts-pending-title', 'Versions à valider')}</h2>
				{#each aValider as v (v.numero)}
					<article class="space-y-2 rounded-md border border-warning p-3" data-testid="mcp-prompt-pending-{v.numero}">
						<p class="text-sm" data-testid="mcp-prompt-pending-key-{v.numero}">
							{i18nMsg('mcp-prompts-version-number', 'n° { $numero }', { numero: v.numero })}
							— {cleAuteur(v)}
							{#if v.cleRevoquee}
								<span class="text-xs text-text-muted">({i18nMsg('mcp-prompts-key-revoked', 'révoquée')})</span>
							{/if}
							— {i18nMsg('mcp-prompts-proposed-on', 'proposée le { $date }', { date: dateLocale(v.creeLe) })}
						</p>
						<McpPromptVersion version={v} zone="a-valider-" />
						{#if derniere !== null && v.numero !== derniere.numero}
							<p class="text-sm text-text-muted" data-testid="mcp-prompt-not-latest-{v.numero}">
								{i18nMsg(
									'mcp-prompts-not-latest',
									'Une version plus récente existe : celle-ci ne peut plus être approuvée, seulement refusée.',
								)}
							</p>
						{/if}
						<div class="flex flex-wrap gap-2">
							{#if peutApprouver(detail, v)}
								<Button
									onclick={() => (aApprouver = v)}
									disabled={occupe}
									data-testid="mcp-prompt-approve-{v.numero}"
								>
									{i18nMsg('mcp-prompts-approve', 'Approuver')}
								</Button>
							{/if}
							{#if peutRefuser(v)}
								<Button
									variant="destructive"
									onclick={() => (aRefuser = v)}
									disabled={occupe}
									data-testid="mcp-prompt-refuse-{v.numero}"
								>
									{i18nMsg('mcp-prompts-refuse', 'Refuser')}
								</Button>
							{/if}
						</div>
					</article>
				{/each}
			</section>
		{/if}

		<section class="space-y-2">
			<h2 class="text-lg font-semibold">{i18nMsg('mcp-prompts-history-title', 'Historique')}</h2>
			<p class="text-sm text-text-muted">
				{i18nMsg(
					'mcp-prompts-history-help',
					'Chaque modification crée une version ; aucune n’est effacée ni réécrite.',
				)}
			</p>
			<div class="overflow-x-auto">
				<table class="w-full text-sm" data-testid="mcp-prompt-history">
					<thead>
						<tr class="border-b border-border text-left">
							<th class="py-2 pr-4">{i18nMsg('mcp-prompts-col-number', 'N°')}</th>
							<th class="py-2 pr-4">{i18nMsg('mcp-prompts-col-origin', 'Origine')}</th>
							<th class="py-2 pr-4">{i18nMsg('mcp-prompts-col-state', 'État')}</th>
							<th class="py-2 pr-4">{i18nMsg('mcp-prompts-col-nature', 'Nature')}</th>
							<th class="py-2 pr-4">{i18nMsg('mcp-prompts-col-created', 'Créée le')}</th>
							<th class="py-2 pr-4">{i18nMsg('mcp-prompts-col-decision', 'Décision')}</th>
							<th class="py-2"></th>
						</tr>
					</thead>
					<tbody>
						{#each historique as v (v.numero)}
							<tr class="border-b border-border align-top" data-testid="mcp-prompt-history-row-{v.numero}">
								<td class="py-2 pr-4">
									<span class="font-mono">{v.numero}</span>
									{#if detail.numeroEnVigueur === v.numero}
										<span class="pastille pastille-succes ml-1" data-testid="mcp-prompt-in-force-badge-{v.numero}">
											{i18nMsg('mcp-prompts-badge-in-force', 'En vigueur')}
										</span>
									{/if}
								</td>
								<td class="py-2 pr-4" data-testid="mcp-prompt-history-origin-{v.numero}">
									{#if v.origine === 'agent'}
										<span class="pastille pastille-avertissement">{origineAffichee(v.origine)}</span>
										— {i18nMsg('mcp-prompts-key-number', 'clé n° { $id }', { id: v.cleAuteurId ?? '—' })}
									{:else if v.origine === 'admin'}
										{origineAffichee(v.origine)} — {v.auteurNom ?? '—'}
									{:else}
										{origineAffichee(v.origine)}
									{/if}
								</td>
								<td class="py-2 pr-4" data-testid="mcp-prompt-history-state-{v.numero}">{etatAffiche(v.etat)}</td>
								<td class="py-2 pr-4" data-testid="mcp-prompt-history-nature-{v.numero}">{natureAffichee(v.nature)}</td>
								<td class="py-2 pr-4 whitespace-nowrap">{dateLocale(v.creeLe)}</td>
								<td class="py-2 pr-4">
									{#if v.origine === 'agent' && v.decideLe !== null}
										<span data-testid="mcp-prompt-history-decision-{v.numero}">
											{i18nMsg('mcp-prompts-decided', 'par { $user }, le { $date }', {
												user: v.decideurNom ?? '—',
												date: dateLocale(v.decideLe),
											})}
										</span>
									{/if}
								</td>
								<td class="py-2">
									{#if v.nature === 'text' && v.description !== null && v.texte !== null}
										<Button
											variant="outline"
											size="sm"
											onclick={() => preRemplir(v)}
											disabled={occupe}
											data-testid="mcp-prompt-history-reuse-{v.numero}"
										>
											{i18nMsg('mcp-prompts-reuse', 'Reprendre cette version')}
										</Button>
									{/if}
								</td>
							</tr>
							{#if v.nature === 'text'}
								<tr class="border-b border-border">
									<td colspan="7" class="py-1">
										<details data-testid="mcp-prompt-history-details-{v.numero}">
											<summary class="cursor-pointer text-sm text-primary">
												{i18nMsg('mcp-prompts-history-show', 'Voir le texte de la version n° { $numero }', {
													numero: v.numero,
												})}
											</summary>
											<div class="pt-2">
												<McpPromptVersion version={v} zone="historique-" />
											</div>
										</details>
									</td>
								</tr>
							{/if}
						{/each}
					</tbody>
				</table>
			</div>
		</section>

		<div class="space-y-2" data-testid="mcp-prompt-editor-zone">
			{#if erreur && erreurPresDeLEditeur}
				{@render alerte(erreur)}
			{/if}

			{#if saisieConservee}
				<div
					class="rounded-md border border-warning bg-warning/10 px-3 py-2 text-sm"
					role="note"
					data-testid="mcp-prompt-editor-kept"
				>
					{i18nMsg(
						'mcp-prompts-editor-kept',
						'Votre saisie est conservée dans l’éditeur. Relisez l’historique rechargé avant de l’enregistrer de nouveau.',
					)}
				</div>
			{/if}

			<McpPromptEditor
				bind:description={editeurDescription}
				bind:texte={editeurTexte}
				origine={editeurOrigine}
				{numeroSuivant}
				envoiEnCours={occupe}
				occupe={gesteEnCours === 'enregistrer'}
				onEnregistrer={enregistrer}
			/>
		</div>

		<div class="flex flex-wrap gap-2" data-testid="mcp-prompt-actions">
			{#if enVigueur === null || enVigueur.nature !== 'withdrawal'}
				<Button
					variant="destructive"
					onclick={() => (dialogueRetrait = true)}
					disabled={occupe}
					data-testid="mcp-prompt-withdraw"
				>
					{i18nMsg('mcp-prompts-withdraw', 'Retirer ce prompt')}
				</Button>
			{/if}
			{#if peutRevenirAuStandard(detail)}
				<Button
					variant="outline"
					onclick={() => (dialogueStandard = true)}
					disabled={occupe}
					data-testid="mcp-prompt-restore-standard"
				>
					{i18nMsg('mcp-prompts-restore-standard', 'Revenir au texte standard')}
				</Button>
			{/if}
		</div>

		<Dialog.Root open={aApprouver !== null} onOpenChange={(o) => { if (!o && !envoiEnCours) aApprouver = null; }}>
			<Dialog.Content
				escapeKeydownBehavior={fermeture}
				interactOutsideBehavior={fermeture}
				showCloseButton={!envoiEnCours}
				data-testid="mcp-prompt-approve-dialog"
			>
				{#if aApprouver}
					<Dialog.Header>
						<Dialog.Title>{i18nMsg('mcp-prompts-approve-dialog-title', 'Approuver cette version ?')}</Dialog.Title>
					</Dialog.Header>
					<p class="text-sm"><span class="font-mono">{detail.nom}</span></p>
					<p class="text-sm">
						{i18nMsg('mcp-prompts-approve-dialog-version', 'version n° { $numero }, proposée par la clé n° { $id } ({ $nom })', {
							numero: aApprouver.numero,
							id: aApprouver.cleAuteurId ?? '—',
							nom: aApprouver.cleNom ?? '—',
						})}
					</p>
					<p class="text-sm">
						{i18nMsg(
							'mcp-prompts-approve-dialog-body',
							'Une fois approuvé, ce texte sera servi à tout agent dont la clé porte books:read, précédé de la marque.',
						)}
					</p>
					<Dialog.Footer>
						<Button
							variant="outline"
							onclick={() => (aApprouver = null)}
							disabled={occupe}
							data-testid="mcp-prompt-approve-cancel"
						>
							{i18nMsg('mcp-prompts-cancel', 'Annuler')}
						</Button>
						<Button
							onclick={confirmerApprobation}
							disabled={occupe}
							aria-busy={gesteEnCours === 'approuver' ? 'true' : undefined}
							data-testid="mcp-prompt-approve-confirm"
						>
							{i18nMsg('mcp-prompts-approve-confirm', 'Approuver la version n° { $numero }', {
								numero: aApprouver.numero,
							})}
						</Button>
					</Dialog.Footer>
				{/if}
			</Dialog.Content>
		</Dialog.Root>

		<Dialog.Root open={aRefuser !== null} onOpenChange={(o) => { if (!o && !envoiEnCours) aRefuser = null; }}>
			<Dialog.Content
				escapeKeydownBehavior={fermeture}
				interactOutsideBehavior={fermeture}
				showCloseButton={!envoiEnCours}
				data-testid="mcp-prompt-refuse-dialog"
			>
				{#if aRefuser}
					<Dialog.Header>
						<Dialog.Title>
							{i18nMsg('mcp-prompts-refuse-dialog-title', 'Refuser la version n° { $numero } ?', {
								numero: aRefuser.numero,
							})}
						</Dialog.Title>
					</Dialog.Header>
					<p class="text-sm">
						{i18nMsg('mcp-prompts-refuse-dialog-body', 'Un refus est définitif ; la version reste à l’historique.')}
					</p>
					<Dialog.Footer>
						<Button variant="outline" onclick={() => (aRefuser = null)} disabled={occupe}>
							{i18nMsg('mcp-prompts-cancel', 'Annuler')}
						</Button>
						<Button
							variant="destructive"
							onclick={confirmerRefus}
							disabled={occupe}
							aria-busy={gesteEnCours === 'refuser' ? 'true' : undefined}
							data-testid="mcp-prompt-refuse-confirm"
						>
							{i18nMsg('mcp-prompts-refuse-confirm', 'Refuser la version')}
						</Button>
					</Dialog.Footer>
				{/if}
			</Dialog.Content>
		</Dialog.Root>

		<Dialog.Root open={dialogueRetrait} onOpenChange={(o) => { if (!o && !envoiEnCours) dialogueRetrait = false; }}>
			<Dialog.Content
				escapeKeydownBehavior={fermeture}
				interactOutsideBehavior={fermeture}
				showCloseButton={!envoiEnCours}
				data-testid="mcp-prompt-withdraw-dialog"
			>
				<Dialog.Header>
					<Dialog.Title>{i18nMsg('mcp-prompts-withdraw-dialog-title', 'Retirer ce prompt ?')}</Dialog.Title>
				</Dialog.Header>
				<p class="text-sm">
					{i18nMsg(
						'mcp-prompts-withdraw-dialog-body',
						'Le prompt cessera d’être servi. Une version n° { $numero } de retrait est ajoutée ; l’historique reste. Une nouvelle version le servira de nouveau.',
						{ numero: numeroSuivant },
					)}
				</p>
				<Dialog.Footer>
					<Button variant="outline" onclick={() => (dialogueRetrait = false)} disabled={occupe}>
						{i18nMsg('mcp-prompts-cancel', 'Annuler')}
					</Button>
					<Button
						variant="destructive"
						onclick={confirmerRetrait}
						disabled={occupe}
						aria-busy={gesteEnCours === 'retirer' ? 'true' : undefined}
						data-testid="mcp-prompt-withdraw-confirm"
					>
						{i18nMsg('mcp-prompts-withdraw', 'Retirer ce prompt')}
					</Button>
				</Dialog.Footer>
			</Dialog.Content>
		</Dialog.Root>

		<Dialog.Root open={dialogueStandard} onOpenChange={(o) => { if (!o && !envoiEnCours) dialogueStandard = false; }}>
			<Dialog.Content
				escapeKeydownBehavior={fermeture}
				interactOutsideBehavior={fermeture}
				showCloseButton={!envoiEnCours}
				class="max-h-[90vh] overflow-y-auto sm:max-w-3xl"
				data-testid="mcp-prompt-standard-dialog"
			>
				<Dialog.Header>
					<Dialog.Title>{i18nMsg('mcp-prompts-standard-dialog-title', 'Revenir au texte standard ?')}</Dialog.Title>
				</Dialog.Header>
				{#if detail.standardDuCode}
					<p class="text-xs font-medium text-text-muted">{i18nMsg('mcp-prompts-field-description', 'Description')}</p>
					<pre class="whitespace-pre-wrap break-words rounded-md border border-border bg-surface-alt px-3 py-2 font-mono text-sm" dir="ltr" data-testid="mcp-prompt-standard-description">{detail.standardDuCode.description.visible}</pre>
					<p class="text-xs font-medium text-text-muted">{i18nMsg('mcp-prompts-field-text', 'Texte')}</p>
					<pre class="whitespace-pre-wrap break-words rounded-md border border-border bg-surface-alt px-3 py-2 font-mono text-sm" dir="ltr" data-testid="mcp-prompt-standard-text">{detail.standardDuCode.texte.visible}</pre>
				{/if}
				<p class="text-sm">
					{i18nMsg(
						'mcp-prompts-standard-dialog-body',
						'Une nouvelle version n° { $numero } reprend ce texte, le standard de cette version de Kesh. Vos versions et celles de l’agent restent à l’historique, et vous pourrez reprendre l’une d’elles. Le prompt redevient non personnalisé : il suivra désormais le texte des prochaines versions de Kesh.',
						{ numero: numeroSuivant },
					)}
				</p>
				<Dialog.Footer>
					<Button variant="outline" onclick={() => (dialogueStandard = false)} disabled={occupe}>
						{i18nMsg('mcp-prompts-cancel', 'Annuler')}
					</Button>
					<Button
						onclick={confirmerStandard}
						disabled={occupe}
						aria-busy={gesteEnCours === 'standard' ? 'true' : undefined}
						data-testid="mcp-prompt-standard-confirm"
					>
						{i18nMsg('mcp-prompts-restore-standard', 'Revenir au texte standard')}
					</Button>
				</Dialog.Footer>
			</Dialog.Content>
		</Dialog.Root>
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
	.pastille-succes {
		background-color: color-mix(in srgb, var(--color-success) 20%, transparent);
	}
</style>
