<!--
  Story 32-7 (refs #723) — la section des clés MCP de l'écran Serveur MCP (AC-8 à AC-13).

  La liste (ordre du serveur, révoquées comprises), la création inline, la révocation et
  la politique de validation d'une clé.

  ⛔ Le secret d'une clé créée vit dans UNE instance de `SecretUnique`, créée ici, dans le
  bloc `<script>` d'instance (garde G-b) : *Fermer* et toute nouvelle soumission
  l'oublient ; quitter la page détruit le composant et l'instance. Il ne passe jamais par
  la liste (rechargée par `GET`), un stockage, l'adresse, un journal ni un toast.
-->
<script lang="ts">
	import { onMount } from 'svelte';
	import { Button } from '$lib/components/ui/button';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { isApiError } from '$lib/shared/utils/api-error';
	import { errorMessageOf } from '$lib/shared/utils/error-message';
	import { createMcpKey, listMcpKeys, revokeMcpKey, setMcpKeySendValidation } from './mcp-server.api';
	import { PORTEES_MCP, type McpKey, type McpScope, type McpSendValidation } from './mcp-server.types';
	import { dateAffichee, dateHeureAffichee, estPasse, finDeJourneeLocale, lendemainLocal } from './mcp-time';
	import { SecretUnique } from './secret-unique.svelte';
	import McpKeySecretNotice from './McpKeySecretNotice.svelte';

	/** Longueur maximale du nom, en caractères (`NAME_MAX_CHARS` de la 32-5c). */
	const NOM_MAX = 255;

	const secret = new SecretUnique();

	let cles = $state<McpKey[]>([]);
	let chargement = $state(true);
	let erreurChargement = $state<string | null>(null);

	let message = $state<string | null>(null);
	let erreurAction = $state<string | null>(null);

	let formulaireOuvert = $state(false);
	let nom = $state('');
	let portees = $state<McpScope[]>([]);
	let politique = $state<McpSendValidation>('required');
	let expiration = $state('');
	let expirationMin = $state('');
	let envoi = $state(false);
	let erreurFormulaire = $state<string | null>(null);

	let aRevoquer = $state<number | null>(null);
	let aDispenser = $state<number | null>(null);
	let actionEnCours = $state(false);

	async function charger() {
		chargement = true;
		erreurChargement = null;
		try {
			cles = await listMcpKeys();
		} catch (err) {
			erreurChargement = errorMessageOf(err);
		} finally {
			chargement = false;
		}
	}

	onMount(() => {
		void charger();
	});

	function testid(p: McpScope): string {
		return p.replace(':', '-');
	}

	/** La phrase d'une portée : ce qu'elle ouvre (AC-10), relue au code des 32-9 et 32-10a. */
	function phrasePortee(p: McpScope): string {
		switch (p) {
			case 'books:read':
				return i18nMsg(
					'mcp-server-scope-books',
					'Le plan comptable, les exercices, les écritures, le grand livre, la balance, le bilan, le compte de résultat et les postes ouverts.',
				);
			case 'invoices:read':
				return i18nMsg(
					'mcp-server-scope-invoices',
					'Les factures clientes et fournisseur, l’échéancier et la balance âgée ; les noms des tiers sont lisibles, mais l’IBAN et le QR-IBAN du créancier et l’adresse courriel d’envoi d’une facture sont masqués sans contacts:read.',
				);
			case 'contacts:read':
				return i18nMsg(
					'mcp-server-scope-contacts',
					'Le carnet d’adresses — des données personnelles. Elle démasque aussi, dans les factures, l’IBAN et le QR-IBAN du créancier et l’adresse courriel d’envoi ; les noms des tiers se lisent sans elle. L’IBAN n’est pas une donnée du carnet d’adresses.',
				);
			case 'documents:read':
				return i18nMsg(
					'mcp-server-scope-documents',
					'Le nom du fichier, le type, la taille, l’état et la facture fournisseur liée de chaque pièce importée, sans son contenu.',
				);
			case 'prompts:write':
				return i18nMsg(
					'mcp-server-scope-prompts',
					'Proposer une version d’un prompt, qui attend l’approbation d’un administrateur ; elle ne permet ni d’approuver ni de relire une proposition ; une clé à validation requise peut proposer.',
				);
		}
	}

	function libellePolitique(v: McpSendValidation): string {
		return v === 'exempt'
			? i18nMsg('mcp-server-policy-exempt', 'Dispensée')
			: i18nMsg('mcp-server-policy-required', 'Requise');
	}

	function ouvrir() {
		nom = '';
		portees = [];
		politique = 'required';
		expiration = '';
		expirationMin = lendemainLocal();
		erreurFormulaire = null;
		formulaireOuvert = true;
	}

	function basculerPortee(p: McpScope, coche: boolean) {
		portees = coche ? [...portees.filter((x) => x !== p), p] : portees.filter((x) => x !== p);
	}

	async function creer(event?: Event) {
		event?.preventDefault();
		if (envoi) return;
		// Une nouvelle soumission oublie l'ancien secret AVANT tout envoi (AC-11).
		secret.oublier();
		erreurFormulaire = null;
		const nomRogne = nom.trim();
		if (nomRogne === '') {
			erreurFormulaire = i18nMsg('mcp-server-form-name-required', 'Le nom est requis.');
			return;
		}
		if ([...nomRogne].length > NOM_MAX) {
			erreurFormulaire = i18nMsg('mcp-server-form-name-too-long', 'Le nom compte 255 caractères au plus.');
			return;
		}
		if (portees.length === 0) {
			erreurFormulaire = i18nMsg('mcp-server-form-scope-required', 'Choisissez au moins une portée.');
			return;
		}
		envoi = true;
		try {
			const expiresAt = expiration.trim() === '' ? null : finDeJourneeLocale(expiration);
			const creee = await createMcpKey({
				name: nomRogne,
				scopes: PORTEES_MCP.filter((p) => portees.includes(p)),
				sendValidation: politique,
				...(expiresAt !== null ? { expiresAt } : {}),
			});
			secret.montrer(creee.cle.name, creee.secret);
			formulaireOuvert = false;
			nom = '';
			portees = [];
			politique = 'required';
			expiration = '';
			void charger();
		} catch (err) {
			erreurFormulaire = errorMessageOf(err);
		} finally {
			envoi = false;
		}
	}

	function fermerSecret() {
		secret.oublier();
	}

	/** Un 404 ou un 409 : la clé a changé ou n'existe plus ; la liste se recharge. */
	async function echecDAction(err: unknown) {
		if (isApiError(err) && (err.status === 404 || err.status === 409)) {
			erreurAction = i18nMsg('mcp-server-key-stale', 'La clé a changé ou n’existe plus ; liste rechargée.');
			await charger();
		} else {
			erreurAction = errorMessageOf(err);
		}
	}

	async function revoquer(k: McpKey) {
		if (actionEnCours) return;
		actionEnCours = true;
		message = null;
		erreurAction = null;
		try {
			await revokeMcpKey(k.id, k.version);
			aRevoquer = null;
			message = i18nMsg('mcp-server-key-revoked', 'Clé révoquée.');
			await charger();
		} catch (err) {
			aRevoquer = null;
			await echecDAction(err);
		} finally {
			actionEnCours = false;
		}
	}

	async function changerPolitique(k: McpKey, vers: McpSendValidation) {
		if (actionEnCours) return;
		actionEnCours = true;
		message = null;
		erreurAction = null;
		try {
			const relue = await setMcpKeySendValidation(k.id, vers, k.version);
			aDispenser = null;
			cles = cles.map((c) => (c.id === relue.id ? relue : c));
			message = i18nMsg('mcp-server-policy-changed', 'Politique de validation modifiée.');
		} catch (err) {
			aDispenser = null;
			await echecDAction(err);
		} finally {
			actionEnCours = false;
		}
	}

	function demanderPolitique(k: McpKey) {
		if (k.sendValidation === 'required') {
			aDispenser = k.id;
		} else {
			void changerPolitique(k, 'required');
		}
	}

	const montre = $derived(secret.lire());
</script>

<section class="space-y-4 rounded-md border border-border p-4" data-testid="mcp-server-keys">
	<div class="flex flex-wrap items-center justify-between gap-2">
		<h2 class="text-lg font-semibold">{i18nMsg('mcp-server-keys-title', 'Clés MCP')}</h2>
		<Button variant="outline" onclick={ouvrir} disabled={formulaireOuvert} data-testid="mcp-server-keys-new">
			{i18nMsg('mcp-server-keys-new', 'Nouvelle clé')}
		</Button>
	</div>

	{#if montre !== null}
		<McpKeySecretNotice nom={montre.nom} valeur={montre.valeur} onFermer={fermerSecret} />
	{/if}

	{#if formulaireOuvert}
		<form class="space-y-3 rounded-md border border-border p-4" onsubmit={creer} data-testid="mcp-server-form">
			<h3 class="font-semibold">{i18nMsg('mcp-server-form-title', 'Nouvelle clé MCP')}</h3>
			<label class="block space-y-1 text-sm">
				<span class="font-medium">{i18nMsg('mcp-server-form-name', 'Nom')}</span>
				<input
					class="w-full rounded-md border border-border bg-surface px-3 py-2"
					bind:value={nom}
					data-testid="mcp-server-form-name"
				/>
			</label>

			<fieldset class="space-y-2 text-sm">
				<legend class="font-medium">{i18nMsg('mcp-server-form-scopes', 'Portées')}</legend>
				<p class="text-text-muted">
					{i18nMsg('mcp-server-form-scopes-intro', 'Aucune portée n’écrit dans les livres ni n’administre Kesh.')}
				</p>
				{#each PORTEES_MCP as p (p)}
					<label class="flex items-start gap-2">
						<input
							type="checkbox"
							checked={portees.includes(p)}
							onchange={(e) => basculerPortee(p, (e.currentTarget as HTMLInputElement).checked)}
							data-testid="mcp-server-form-scope-{testid(p)}"
						/>
						<span>
							<code class="font-mono">{p}</code>
							<span class="block text-text-muted" data-testid="mcp-server-form-scope-help-{testid(p)}">{phrasePortee(p)}</span>
						</span>
					</label>
				{/each}
			</fieldset>

			<fieldset class="space-y-2 text-sm">
				<legend class="font-medium">{i18nMsg('mcp-server-form-policy', 'Validation avant envoi')}</legend>
				<label class="flex items-start gap-2">
					<input type="radio" name="mcp-politique" value="required" bind:group={politique} data-testid="mcp-server-form-policy-required" />
					<span>
						{i18nMsg('mcp-server-policy-required', 'Requise')}
						<span class="block text-text-muted" data-testid="mcp-server-form-policy-help-required">
							{i18nMsg(
								'mcp-server-policy-required-help',
								'Chaque réponse d’outil qui porte des données attend votre validation dans Envois du serveur MCP avant de partir ; un envoi non décidé en 24 heures échoit, et une clé compte au plus 20 envois en attente. Une réponse sans données — introuvable, trop volumineuse, erreur — part sans validation.',
							)}
						</span>
					</span>
				</label>
				<label class="flex items-start gap-2">
					<input type="radio" name="mcp-politique" value="exempt" bind:group={politique} data-testid="mcp-server-form-policy-exempt" />
					<span>
						{i18nMsg('mcp-server-policy-exempt', 'Dispensée')}
						<span class="block text-text-muted" data-testid="mcp-server-form-policy-help-exempt">
							{i18nMsg('mcp-server-policy-exempt-help', 'Les données partent sans validation ; chaque appel reste journalisé.')}
						</span>
					</span>
				</label>
			</fieldset>

			<label class="block space-y-1 text-sm">
				<span class="font-medium">{i18nMsg('mcp-server-form-expires', 'Expiration (facultative)')}</span>
				<input
					type="date"
					class="rounded-md border border-border bg-surface px-3 py-2"
					min={expirationMin}
					bind:value={expiration}
					data-testid="mcp-server-form-expires"
				/>
				<span class="block text-text-muted">
					{i18nMsg('mcp-server-form-expires-help', 'La clé cesse d’être valide à la fin de ce jour, à l’heure de ce poste.')}
				</span>
			</label>

			{#if erreurFormulaire !== null}
				<div
					class="rounded-md border border-destructive bg-destructive/10 px-3 py-2 text-sm text-destructive"
					role="alert"
					data-testid="mcp-server-form-error"
				>
					{erreurFormulaire}
				</div>
			{/if}
			<div class="flex gap-2">
				<Button type="submit" disabled={envoi} aria-busy={envoi ? 'true' : undefined} data-testid="mcp-server-form-submit">
					{i18nMsg('mcp-server-form-create', 'Créer la clé')}
				</Button>
				<Button variant="outline" onclick={() => (formulaireOuvert = false)} disabled={envoi} data-testid="mcp-server-form-cancel">
					{i18nMsg('mcp-server-cancel', 'Annuler')}
				</Button>
			</div>
		</form>
	{/if}

	{#if message !== null}
		<p class="text-sm" role="status" data-testid="mcp-server-keys-message">{message}</p>
	{/if}
	{#if erreurAction !== null}
		<div
			class="rounded-md border border-destructive bg-destructive/10 px-3 py-2 text-sm text-destructive"
			role="alert"
			data-testid="mcp-server-keys-error"
		>
			{erreurAction}
		</div>
	{/if}

	{#if chargement}
		<p class="text-sm text-text-muted" role="status" data-testid="mcp-server-keys-loading">
			{i18nMsg('mcp-server-keys-loading', 'Chargement des clés…')}
		</p>
	{:else if erreurChargement !== null}
		<div
			class="rounded-md border border-destructive bg-destructive/10 px-3 py-2 text-sm text-destructive"
			role="alert"
			data-testid="mcp-server-keys-load-error"
		>
			{erreurChargement}
		</div>
	{:else if cles.length === 0}
		<p class="text-sm text-text-muted" data-testid="mcp-server-keys-empty">{i18nMsg('mcp-server-keys-empty', 'Aucune clé MCP.')}</p>
	{:else}
		<div class="overflow-x-auto">
			<table class="w-full text-sm" data-testid="mcp-server-keys-table">
				<thead>
					<tr class="border-b border-border text-left">
						<th class="px-2 py-1">{i18nMsg('mcp-server-keys-col-name', 'Nom')}</th>
						<th class="px-2 py-1">{i18nMsg('mcp-server-keys-col-scopes', 'Portées')}</th>
						<th class="px-2 py-1">{i18nMsg('mcp-server-keys-col-policy', 'Validation')}</th>
						<th class="px-2 py-1">{i18nMsg('mcp-server-keys-col-created', 'Créée le')}</th>
						<th class="px-2 py-1">{i18nMsg('mcp-server-keys-col-expires', 'Expire le')}</th>
						<th class="px-2 py-1">{i18nMsg('mcp-server-keys-col-last-used', 'Dernière utilisation')}</th>
						<th class="px-2 py-1">{i18nMsg('mcp-server-keys-col-status', 'Statut')}</th>
						<th class="px-2 py-1">{i18nMsg('mcp-server-keys-col-actions', 'Actions')}</th>
					</tr>
				</thead>
				<tbody>
					{#each cles as k (k.id)}
						{@const revoquee = k.revokedAt !== null}
						{@const expiree = !revoquee && estPasse(k.expiresAt)}
						<tr class="border-b border-border align-top {revoquee || expiree ? 'opacity-60' : ''}" data-testid="mcp-server-key-row-{k.id}">
							<td class="px-2 py-1">{k.name}</td>
							<td class="px-2 py-1" data-testid="mcp-server-key-scopes-{k.id}">
								{#each k.scopes as s (s)}
									<code class="mr-1 font-mono">{s}</code>
								{/each}
								{#each k.unknownScopes as u (u)}
									<span class="block text-text-muted"
										>{u} — {i18nMsg('mcp-server-keys-scope-unknown', 'inconnue de cette version — jamais accordée')}</span
									>
								{/each}
								{#if k.scopes.length === 0}
									<span class="block font-medium">{i18nMsg('mcp-server-keys-no-access', 'n’ouvre aucun accès')}</span>
								{/if}
							</td>
							<td class="px-2 py-1" data-testid="mcp-server-key-policy-{k.id}">{libellePolitique(k.sendValidation)}</td>
							<td class="px-2 py-1">{dateHeureAffichee(k.createdAt) ?? ''}</td>
							<td class="px-2 py-1" data-testid="mcp-server-key-expires-{k.id}">
								{dateAffichee(k.expiresAt) ?? i18nMsg('mcp-server-keys-never', 'jamais')}
							</td>
							<td class="px-2 py-1" data-testid="mcp-server-key-last-used-{k.id}">
								{dateHeureAffichee(k.lastUsedAt) ?? i18nMsg('mcp-server-keys-never', 'jamais')}
							</td>
							<td class="px-2 py-1" data-testid="mcp-server-key-status-{k.id}">
								{#if revoquee}
									{i18nMsg('mcp-server-keys-status-revoked', 'Révoquée le { $date }', { date: dateAffichee(k.revokedAt) ?? '' })}
								{:else if expiree}
									{i18nMsg('mcp-server-keys-status-expired', 'Expirée le { $date }', { date: dateAffichee(k.expiresAt) ?? '' })}
								{:else}
									{i18nMsg('mcp-server-keys-status-active', 'Active')}
								{/if}
							</td>
							<td class="space-y-1 px-2 py-1">
								{#if !revoquee}
									<div class="flex flex-wrap gap-1">
										<Button
											variant="outline"
											size="sm"
											onclick={() => demanderPolitique(k)}
											disabled={actionEnCours}
											data-testid="mcp-server-key-policy-toggle-{k.id}"
										>
											{k.sendValidation === 'required'
												? i18nMsg('mcp-server-keys-to-exempt', 'Passer en dispensée')
												: i18nMsg('mcp-server-keys-to-required', 'Passer en requise')}
										</Button>
										<Button
											variant="outline"
											size="sm"
											onclick={() => (aRevoquer = k.id)}
											disabled={actionEnCours}
											data-testid="mcp-server-key-revoke-{k.id}"
										>
											{i18nMsg('mcp-server-keys-revoke', 'Révoquer')}
										</Button>
									</div>
									{#if aDispenser === k.id}
										<div class="space-y-1 rounded-md border border-border p-2">
											<p data-testid="mcp-server-key-exempt-confirm-text-{k.id}">
												{i18nMsg(
													'mcp-server-exempt-confirm-text',
													'Les données partiront à l’agent sans vous être montrées ; chaque appel reste journalisé.',
												)}
											</p>
											<div class="flex gap-1">
												<Button size="sm" onclick={() => changerPolitique(k, 'exempt')} disabled={actionEnCours} data-testid="mcp-server-key-exempt-confirm-{k.id}">
													{i18nMsg('mcp-server-exempt-confirm', 'Passer en dispensée')}
												</Button>
												<Button variant="outline" size="sm" onclick={() => (aDispenser = null)} data-testid="mcp-server-key-exempt-cancel-{k.id}">
													{i18nMsg('mcp-server-cancel', 'Annuler')}
												</Button>
											</div>
										</div>
									{/if}
									{#if aRevoquer === k.id}
										<div class="space-y-1 rounded-md border border-border p-2">
											<p data-testid="mcp-server-key-revoke-confirm-text-{k.id}">
												{i18nMsg(
													'mcp-server-revoke-confirm-text',
													'L’agent qui emploie cette clé perd l’accès immédiatement. La révocation est définitive ; seule la restauration d’une sauvegarde antérieure rendrait la clé de nouveau active.',
												)}
											</p>
											<div class="flex gap-1">
												<Button variant="destructive" size="sm" onclick={() => revoquer(k)} disabled={actionEnCours} data-testid="mcp-server-key-revoke-confirm-{k.id}">
													{i18nMsg('mcp-server-revoke-confirm', 'Révoquer définitivement')}
												</Button>
												<Button variant="outline" size="sm" onclick={() => (aRevoquer = null)} data-testid="mcp-server-key-revoke-cancel-{k.id}">
													{i18nMsg('mcp-server-cancel', 'Annuler')}
												</Button>
											</div>
										</div>
									{/if}
								{/if}
							</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>
	{/if}

	<div class="space-y-1 text-sm text-text-muted">
		<p data-testid="mcp-server-keys-last-used-note">
			{i18nMsg(
				'mcp-server-keys-last-used-note',
				'La dernière utilisation est posée chaque fois que la clé est reconnue, même si l’appel est ensuite refusé ou n’est pas servi.',
			)}
		</p>
		<div data-testid="mcp-server-keys-policy-note">
			<p>
				<span class="font-medium">{i18nMsg('mcp-server-policy-required', 'Requise')}</span> —
				{i18nMsg(
					'mcp-server-policy-required-help',
					'Chaque réponse d’outil qui porte des données attend votre validation dans Envois du serveur MCP avant de partir ; un envoi non décidé en 24 heures échoit, et une clé compte au plus 20 envois en attente. Une réponse sans données — introuvable, trop volumineuse, erreur — part sans validation.',
				)}
			</p>
			<p>
				<span class="font-medium">{i18nMsg('mcp-server-policy-exempt', 'Dispensée')}</span> —
				{i18nMsg('mcp-server-policy-exempt-help', 'Les données partent sans validation ; chaque appel reste journalisé.')}
			</p>
		</div>
		<p>
			<a class="text-primary underline" href="/admin/mcp/sends" data-testid="mcp-server-sends-link">
				{i18nMsg('mcp-server-sends-link', 'Voir les envois du serveur MCP')}
			</a>
		</p>
	</div>
</section>
