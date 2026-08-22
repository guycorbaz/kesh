<script lang="ts">
	// Story 20-2 (socle) + Story 21-4 (multi-type / multi-niveau).
	// Le backend renvoie tous les templates effectifs (type × langue × niveau) ;
	// on les indexe par CLÉ COMPOSITE STRING `${type}:${level}:${lang}` (pas par
	// langue seule, sinon collision). INVARIANT (bug 20-2) : changer de type, de
	// niveau OU de langue ne re-fetch JAMAIS et ne perd JAMAIS un brouillon —
	// `syncDraftFromTemplate` est réservé à load/save/reload-409/restore.
	import { onMount } from 'svelte';
	import * as Dialog from '$lib/components/ui/dialog';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { notifyError, notifySuccess } from '$lib/shared/utils/notify';
	import { isApiError } from '$lib/shared/utils/api-client';
	import { authState } from '$lib/app/stores/auth.svelte';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import {
		listEmailTemplates,
		getEmailTemplate,
		updateEmailTemplate,
		restoreEmailTemplateDefault,
		EMAIL_TEMPLATE_LANGUAGES,
		type EmailTemplateLanguage,
		type EmailTemplateResponse,
	} from '$lib/features/email-templates';

	function msg(key: string, fallback: string): string {
		return i18nMsg(key, fallback);
	}

	// IDs DOM stables et HTTP-LAN-safe ($props.id() — pas de crypto.randomUUID,
	// indisponible hors contexte sécurisé sur déploiement HTTP NAS, cf. #145).
	const uid = $props.id();

	let isAdmin = $derived(authState.currentUser?.role === 'Admin');

	type Draft = { subject: string; body: string };
	const TEMPLATE_TYPES = ['invoice_send', 'invoice_reminder'] as const;
	type TemplateType = (typeof TEMPLATE_TYPES)[number];

	/** Clé composite d'indexation : type × niveau × langue. */
	function ckey(type: string, level: number, lang: EmailTemplateLanguage): string {
		return `${type}:${level}:${lang}`;
	}

	let templatesMap = $state<Record<string, EmailTemplateResponse>>({});
	let drafts = $state<Record<string, Draft>>({});
	let activeType = $state<TemplateType>('invoice_send');
	let activeLevel = $state(0);
	let activeLang = $state<EmailTemplateLanguage>('FR');

	let loading = $state(true);
	let loadError = $state('');
	let submitting = $state(false);
	let unknownVars = $state<string[]>([]);

	let restoreOpen = $state(false);
	let restoreSubmitting = $state(false);
	let restoreError = $state('');

	// Plus grand niveau de rappel présent (sélecteur de niveau 0..maxLevel).
	let maxLevel = $derived.by(() => {
		let m = 0;
		for (const t of Object.values(templatesMap)) {
			if (t.templateType === 'invoice_reminder' && t.levelNumber > m) m = t.levelNumber;
		}
		return m;
	});
	let reminderLevels = $derived(Array.from({ length: maxLevel + 1 }, (_, i) => i));

	let activeKey = $derived(ckey(activeType, activeLevel, activeLang));
	let current = $derived(templatesMap[activeKey]);
	let isDefault = $derived(current?.isDefault ?? true);
	let allowedVariables = $derived(current?.allowedVariables ?? []);
	let canSave = $derived(
		(drafts[activeKey]?.subject.trim().length ?? 0) > 0 &&
			(drafts[activeKey]?.body.trim().length ?? 0) > 0,
	);

	function levelLabel(level: number): string {
		return level === 0
			? msg('email-templates-level-generic', 'Générique')
			: i18nMsg('email-templates-level-n', 'Rappel {$n}', { n: level });
	}
	function typeLabel(type: string): string {
		return type === 'invoice_reminder'
			? msg('email-templates-type-invoice_reminder', 'Rappel de facture')
			: msg('email-templates-type-invoice_send', 'Envoi de facture');
	}

	/** Recopie la valeur serveur d'une combinaison dans son brouillon (load/save/reload/restore). */
	function syncDraftFromTemplate(key: string): void {
		const t = templatesMap[key];
		drafts[key] = { subject: t?.subject ?? '', body: t?.body ?? '' };
	}

	function selectType(type: TemplateType): void {
		activeType = type;
		// H1 : invoice_send n'existe qu'au niveau 0 → reset avant tout rendu.
		if (type === 'invoice_send') activeLevel = 0;
		unknownVars = [];
		restoreError = '';
	}
	function selectLevel(level: number): void {
		activeLevel = level;
		unknownVars = [];
		restoreError = '';
	}
	function selectLang(lang: EmailTemplateLanguage): void {
		activeLang = lang;
		// Brouillons préservés par combinaison → aucune ré-hydratation ici.
		unknownVars = [];
		restoreError = '';
	}

	onMount(async () => {
		try {
			const all = await listEmailTemplates();
			for (const t of all) {
				const key = ckey(t.templateType, t.levelNumber, t.language);
				templatesMap[key] = t;
				drafts[key] = { subject: t.subject, body: t.body };
			}
		} catch (err) {
			loadError = isApiError(err) ? err.message : msg('email-templates-load-error', 'Erreur de chargement');
		} finally {
			loading = false;
		}
	});

	/** Recharge la combinaison active depuis le backend et resynchronise son brouillon. */
	async function reloadCurrent(): Promise<void> {
		const type = activeType,
			level = activeLevel,
			lang = activeLang;
		const fresh = await getEmailTemplate(type, lang, level);
		const key = ckey(type, level, lang);
		templatesMap[key] = fresh;
		syncDraftFromTemplate(key);
	}

	async function save(): Promise<void> {
		if (submitting || !canSave) return;
		submitting = true;
		unknownVars = [];
		const type = activeType,
			level = activeLevel,
			lang = activeLang;
		const key = ckey(type, level, lang);
		try {
			const updated = await updateEmailTemplate(
				type,
				lang,
				{
					subject: drafts[key].subject.trim(),
					body: drafts[key].body.trim(),
					expectedVersion: templatesMap[key]?.version ?? null,
				},
				level,
			);
			templatesMap[key] = updated;
			syncDraftFromTemplate(key);
			notifySuccess(msg('email-templates-saved', 'Modèle enregistré'));
		} catch (err) {
			if (isApiError(err)) {
				if (err.status === 422 && err.code === 'EMAIL_TEMPLATE_UNKNOWN_VARIABLES') {
					const list = err.details?.unknownVariables;
					unknownVars = Array.isArray(list) ? (list as string[]) : [];
					notifyError(
						msg('email-templates-unknown-variables', 'Le modèle contient des variables inconnues'),
					);
				} else if (err.status === 409) {
					notifyError(msg('email-templates-conflict', 'Conflit de version — le modèle a été rechargé'));
					try {
						await reloadCurrent();
					} catch {
						// on garde le toast d'erreur
					}
				} else {
					notifyError(err.message);
				}
			} else {
				notifyError(msg('email-templates-save-error', "Erreur lors de l'enregistrement"));
			}
		} finally {
			submitting = false;
		}
	}

	async function submitRestore(): Promise<void> {
		if (restoreSubmitting) return;
		restoreSubmitting = true;
		restoreError = '';
		const type = activeType,
			level = activeLevel,
			lang = activeLang;
		try {
			await restoreEmailTemplateDefault(type, lang, level);
		} catch (err) {
			restoreError = isApiError(err)
				? err.message
				: msg('email-templates-restore-error', 'Erreur lors de la restauration');
			restoreSubmitting = false;
			return;
		}
		restoreSubmitting = false;
		restoreOpen = false;
		notifySuccess(msg('email-templates-restored', 'Modèle par défaut restauré'));
		try {
			await reloadCurrent();
		} catch {
			// best-effort : le serveur est déjà revenu au défaut.
		}
	}
</script>

<svelte:head>
	<title>{msg('email-templates-title', "Modèles d'e-mail")} — Kesh</title>
</svelte:head>

<h1 class="mb-2 text-2xl font-semibold">{msg('email-templates-title', "Modèles d'e-mail")}</h1>
<p class="mb-6 text-sm text-text-muted">
	{msg(
		'email-templates-subtitle',
		"Personnalisez le contenu des e-mails envoyés à vos clients, par langue. Si vous ne modifiez rien, un modèle par défaut est utilisé automatiquement.",
	)}
</p>

{#if !isAdmin}
	<p class="rounded-md border border-amber-400 bg-amber-50 px-4 py-3 text-sm text-amber-900">
		{msg('email-templates-admin-only', 'Accès réservé aux administrateurs.')}
	</p>
{:else if loading}
	<p class="text-sm text-text-muted">{msg('common-loading', 'Chargement…')}</p>
{:else if loadError}
	<p class="text-sm text-destructive">{loadError}</p>
{:else}
	<section class="space-y-4 rounded-lg border border-border bg-white p-6 shadow-sm">
		<div class="space-y-4" data-testid="email-template-selectors">
		<!-- Sélecteur de type -->
		<div
			class="flex flex-wrap items-center gap-2"
			role="group"
			aria-label={msg('email-templates-type-label', 'Type')}
		>
			<span class="text-sm font-medium">{msg('email-templates-type-label', 'Type')} :</span>
			{#each TEMPLATE_TYPES as type (type)}
				<button
					type="button"
					data-testid="email-template-type-{type}"
					aria-pressed={activeType === type}
					class="rounded-md border px-3 py-1 text-sm"
					class:border-primary={activeType === type}
					class:bg-primary-light={activeType === type}
					onclick={() => selectType(type)}
				>
					{typeLabel(type)}
				</button>
			{/each}
		</div>

		<!-- Sélecteur de niveau (rappels uniquement) -->
		{#if activeType === 'invoice_reminder'}
			<div
				class="flex flex-wrap items-center gap-2"
				role="group"
				aria-label={msg('email-templates-level-label', 'Niveau')}
			>
				<span class="text-sm font-medium">{msg('email-templates-level-label', 'Niveau')} :</span>
				{#each reminderLevels as level (level)}
					<button
						type="button"
						data-testid="email-template-level-{level}"
						aria-pressed={activeLevel === level}
						class="rounded-md border px-3 py-1 text-sm"
						class:border-primary={activeLevel === level}
						class:bg-primary-light={activeLevel === level}
						onclick={() => selectLevel(level)}
					>
						{levelLabel(level)}
					</button>
				{/each}
			</div>
		{/if}
		</div>

		<div class="flex items-center justify-between">
			<h2 class="text-lg font-semibold">
				{typeLabel(activeType)}{activeType === 'invoice_reminder' ? ` — ${levelLabel(activeLevel)}` : ''}
			</h2>
			{#if isDefault}
				<span
					class="rounded-full bg-gray-100 px-3 py-1 text-xs font-medium text-gray-600"
					data-testid="email-template-badge"
					data-variant="default">{msg('email-templates-badge-default', 'Défaut')}</span
				>
			{:else}
				<span
					class="rounded-full bg-primary-light px-3 py-1 text-xs font-medium text-primary"
					data-testid="email-template-badge"
					data-variant="custom"
					>{msg('email-templates-badge-custom', 'Personnalisé')}</span
				>
			{/if}
		</div>

		<!-- Onglets de langue -->
		<div role="tablist" class="flex gap-1" aria-label={msg('email-templates-lang-tablist', 'Langue')}>
			{#each EMAIL_TEMPLATE_LANGUAGES as lang (lang)}
				<button
					role="tab"
					type="button"
					id="{uid}-tab-{lang}"
					aria-selected={activeLang === lang}
					aria-controls="{uid}-tabpanel"
					tabindex={activeLang === lang ? 0 : -1}
					data-testid="email-template-lang-tab-{lang}"
					class="rounded-md border px-3 py-1 text-sm"
					class:border-primary={activeLang === lang}
					class:bg-primary-light={activeLang === lang}
					onclick={() => selectLang(lang)}
				>
					{lang}
				</button>
			{/each}
		</div>

		<div
			class="grid gap-6 md:grid-cols-[2fr_1fr]"
			role="tabpanel"
			id="{uid}-tabpanel"
			aria-labelledby="{uid}-tab-{activeLang}"
		>
			<form
				class="space-y-4"
				onsubmit={(e) => {
					e.preventDefault();
					void save();
				}}
			>
				<div>
					<label class="mb-1 block text-sm font-medium" for="{uid}-subject">
						{msg('email-templates-subject-label', 'Objet')}
					</label>
					<Input
						id="{uid}-subject"
						bind:value={drafts[activeKey].subject}
						data-testid="email-template-subject"
					/>
				</div>
				<div>
					<label class="mb-1 block text-sm font-medium" for="{uid}-body">
						{msg('email-templates-body-label', 'Corps du message')}
					</label>
					<textarea
						id="{uid}-body"
						bind:value={drafts[activeKey].body}
						rows="10"
						data-testid="email-template-body"
						class="w-full rounded-md border border-border bg-white px-3 py-2 text-sm"
					></textarea>
				</div>

				{#if unknownVars.length > 0}
					<p class="text-sm text-destructive" role="alert" data-testid="email-template-unknown-vars">
						{msg('email-templates-unknown-variables-list', 'Variables inconnues :')}
						{unknownVars.map((v) => `{${v}}`).join(', ')}
					</p>
				{/if}

				<div class="flex items-center justify-between">
					<Button
						type="button"
						variant="outline"
						disabled={isDefault || submitting}
						data-testid="email-template-restore-button"
						onclick={() => {
							restoreError = '';
							restoreOpen = true;
						}}
					>
						{msg('email-templates-restore', 'Restaurer le défaut')}
					</Button>
					<Button
						type="submit"
						disabled={submitting || !canSave}
						data-testid="email-template-save-button"
					>
						{submitting ? msg('common-saving', 'Enregistrement…') : msg('common-save', 'Enregistrer')}
					</Button>
				</div>
			</form>

			<!-- Panneau d'aide : variables autorisées (selon la combinaison active). -->
			<aside class="rounded-md border border-border bg-gray-50 p-4">
				<h3 class="mb-2 text-sm font-semibold">
					{msg('email-templates-variables-title', 'Variables disponibles')}
				</h3>
				<p class="mb-3 text-xs text-text-muted">
					{msg(
						'email-templates-variables-hint',
						'Insérez ces variables dans l’objet ou le corps ; elles seront remplacées à l’envoi.',
					)}
				</p>
				<ul class="space-y-1" data-testid="email-template-variables">
					{#each allowedVariables as v (v)}
						<li><code class="rounded bg-white px-1 py-0.5 text-xs">{`{${v}}`}</code></li>
					{/each}
				</ul>
			</aside>
		</div>
	</section>
{/if}

<!-- Modale : confirmer la restauration du défaut (action irréversible). -->
<Dialog.Root bind:open={restoreOpen}>
	<Dialog.Content>
		<Dialog.Header>
			<Dialog.Title>
				{msg('email-templates-restore-confirm-title', 'Restaurer le modèle par défaut ?')}
			</Dialog.Title>
			<Dialog.Description>
				{msg(
					'email-templates-restore-confirm-body',
					'Votre texte personnalisé pour cette combinaison sera supprimé et remplacé par le modèle par défaut. Cette action est irréversible.',
				)}
			</Dialog.Description>
		</Dialog.Header>
		{#if restoreError}
			<p class="text-sm text-red-600" role="alert">{restoreError}</p>
		{/if}
		<Dialog.Footer>
			<Dialog.Close>
				<Button variant="outline" type="button" disabled={restoreSubmitting}
					>{msg('common-cancel', 'Annuler')}</Button
				>
			</Dialog.Close>
			<Button
				type="button"
				class="bg-red-600 hover:bg-red-700"
				disabled={restoreSubmitting}
				data-testid="email-template-restore-confirm"
				onclick={submitRestore}
			>
				{restoreSubmitting
					? msg('email-templates-restoring', 'Restauration…')
					: msg('email-templates-restore-confirm-action', 'Restaurer le défaut')}
			</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>
