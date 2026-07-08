<script lang="ts">
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

	// Toutes les entrées effectives (v1 : 1 type × 4 langues). Indexées par langue.
	let templates = $state<Record<EmailTemplateLanguage, EmailTemplateResponse | null>>({
		FR: null,
		DE: null,
		IT: null,
		EN: null,
	});
	let templateType = $state('invoice_send');
	let allowedVariables = $state<string[]>([]);

	let loading = $state(true);
	let loadError = $state('');
	let submitting = $state(false);

	// Onglet langue actif + champs éditables de l'onglet courant.
	let activeLang = $state<EmailTemplateLanguage>('FR');
	let subject = $state('');
	let body = $state('');
	let unknownVars = $state<string[]>([]);

	// Modale « restaurer le défaut ».
	let restoreOpen = $state(false);
	let restoreSubmitting = $state(false);
	let restoreError = $state('');

	let current = $derived(templates[activeLang]);
	let isDefault = $derived(current?.isDefault ?? true);
	let canSave = $derived(subject.trim().length > 0 && body.trim().length > 0);

	/** Charge les champs éditables depuis l'entrée de la langue donnée. */
	function hydrateFields(lang: EmailTemplateLanguage): void {
		const t = templates[lang];
		subject = t?.subject ?? '';
		body = t?.body ?? '';
		unknownVars = [];
		restoreError = '';
	}

	function selectLang(lang: EmailTemplateLanguage): void {
		activeLang = lang;
		hydrateFields(lang);
	}

	onMount(async () => {
		try {
			const all = await listEmailTemplates();
			for (const t of all) {
				templates[t.language] = t;
			}
			// v1 : un seul type ; on prend le type + les variables de la 1re entrée.
			if (all.length > 0) {
				templateType = all[0].templateType;
				allowedVariables = all[0].allowedVariables;
			}
			hydrateFields(activeLang);
		} catch (err) {
			loadError = isApiError(err) ? err.message : msg('error-unexpected', 'Erreur de chargement');
		} finally {
			loading = false;
		}
	});

	/** Recharge une seule langue depuis le backend et ré-hydrate si c'est l'active. */
	async function reloadLang(lang: EmailTemplateLanguage): Promise<void> {
		const fresh = await getEmailTemplate(templateType, lang);
		templates[lang] = fresh;
		allowedVariables = fresh.allowedVariables;
		if (lang === activeLang) hydrateFields(lang);
	}

	async function save(): Promise<void> {
		if (!canSave) return;
		submitting = true;
		unknownVars = [];
		try {
			const updated = await updateEmailTemplate(templateType, activeLang, {
				subject: subject.trim(),
				body: body.trim(),
				expectedVersion: current?.version ?? null,
			});
			templates[activeLang] = updated;
			allowedVariables = updated.allowedVariables;
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
					notifyError(
						msg('email-templates-conflict', 'Conflit de version — le modèle a été rechargé'),
					);
					try {
						await reloadLang(activeLang);
					} catch {
						// on garde le toast d'erreur
					}
				} else {
					notifyError(err.message);
				}
			} else {
				notifyError(msg('error-unexpected', "Erreur lors de l'enregistrement"));
			}
		} finally {
			submitting = false;
		}
	}

	async function submitRestore(): Promise<void> {
		restoreSubmitting = true;
		restoreError = '';
		try {
			await restoreEmailTemplateDefault(templateType, activeLang);
			await reloadLang(activeLang);
			restoreOpen = false;
			notifySuccess(msg('email-templates-restored', 'Modèle par défaut restauré'));
		} catch (err) {
			restoreError = isApiError(err)
				? err.message
				: msg('error-unexpected', 'Erreur lors de la restauration');
		} finally {
			restoreSubmitting = false;
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
	<p class="text-sm text-text-muted">{msg('loading', 'Chargement…')}</p>
{:else if loadError}
	<p class="text-sm text-destructive">{loadError}</p>
{:else}
	<section class="space-y-4 rounded-lg border border-border bg-white p-6 shadow-sm">
		<div class="flex items-center justify-between">
			<h2 class="text-lg font-semibold">
				{msg('email-templates-type-invoice_send', 'Envoi de facture')}
			</h2>
			{#if isDefault}
				<span
					class="rounded-full bg-gray-100 px-3 py-1 text-xs font-medium text-gray-600"
					data-testid="email-template-badge">{msg('email-templates-badge-default', 'Défaut')}</span
				>
			{:else}
				<span
					class="rounded-full bg-primary-light px-3 py-1 text-xs font-medium text-primary"
					data-testid="email-template-badge"
					>{msg('email-templates-badge-custom', 'Personnalisé')}</span
				>
			{/if}
		</div>

		<!-- Onglets de langue (pattern fait-main, cf. invoices/due-dates). -->
		<div role="tablist" class="flex gap-1" aria-label={msg('email-templates-lang-tablist', 'Langue')}>
			{#each EMAIL_TEMPLATE_LANGUAGES as lang (lang)}
				<button
					role="tab"
					type="button"
					aria-selected={activeLang === lang}
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

		<div class="grid gap-6 md:grid-cols-[2fr_1fr]">
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
					<Input id="{uid}-subject" bind:value={subject} data-testid="email-template-subject" />
				</div>
				<div>
					<label class="mb-1 block text-sm font-medium" for="{uid}-body">
						{msg('email-templates-body-label', 'Corps du message')}
					</label>
					<textarea
						id="{uid}-body"
						bind:value={body}
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
						{submitting ? msg('saving', 'Enregistrement…') : msg('save', 'Enregistrer')}
					</Button>
				</div>
			</form>

			<!-- Panneau d'aide : variables autorisées. -->
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
					'Votre texte personnalisé pour cette langue sera supprimé et remplacé par le modèle par défaut. Cette action est irréversible.',
				)}
			</Dialog.Description>
		</Dialog.Header>
		{#if restoreError}
			<p class="text-sm text-red-600" role="alert">{restoreError}</p>
		{/if}
		<Dialog.Footer>
			<Dialog.Close>
				<Button variant="outline" type="button">{msg('cancel', 'Annuler')}</Button>
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
