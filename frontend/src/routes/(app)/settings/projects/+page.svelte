<!--
  Story 19-1 (Epic 19) — Gestion des projets analytiques `/settings/projects` (Comptable+).
  Arbre à 2 niveaux (projet → sous-projets). Création, édition, archivage/désarchivage,
  filtre « afficher archivés ». Calque l'UI CRUD vat-rates. Pas d'API secure-context-only.
-->
<script lang="ts">
	import { onMount } from 'svelte';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { isApiError } from '$lib/shared/utils/api-client';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { toast } from 'svelte-sonner';
	import {
		listProjects,
		createProject,
		updateProject,
		archiveProject,
		unarchiveProject,
		type ProjectResponse,
	} from '$lib/features/projects/projects.api';

	let projects = $state<ProjectResponse[]>([]);
	let loading = $state(true);
	let loadError = $state<string | null>(null);
	let includeArchived = $state(false);

	type Mode = { kind: 'none' } | { kind: 'create' } | { kind: 'edit'; project: ProjectResponse };
	let mode = $state<Mode>({ kind: 'none' });
	let submitting = $state(false);
	let formError = $state<string | null>(null);

	// Champs de formulaire.
	let fParentId = $state<number | null>(null);
	let fCode = $state('');
	let fName = $state('');
	let fDescription = $state('');
	let fStartDate = $state('');
	let fEndDate = $state('');

	/** Projets racines (parentId null), pour l'affichage arborescent et le sélecteur de parent. */
	const roots = $derived(projects.filter((p) => p.parentId === null));
	/** Sous-projets d'une racine donnée. */
	function childrenOf(rootId: number): ProjectResponse[] {
		return projects.filter((p) => p.parentId === rootId);
	}

	async function load() {
		loading = true;
		loadError = null;
		try {
			projects = await listProjects(includeArchived);
		} catch (err) {
			loadError = isApiError(err)
				? err.message
				: i18nMsg('projects-load-error', 'Impossible de charger les projets.');
		} finally {
			loading = false;
		}
	}

	onMount(load);

	function resetForm() {
		fParentId = null;
		fCode = '';
		fName = '';
		fDescription = '';
		fStartDate = '';
		fEndDate = '';
		formError = null;
	}

	function openCreate() {
		resetForm();
		mode = { kind: 'create' };
	}

	function openEdit(p: ProjectResponse) {
		fParentId = p.parentId;
		fCode = p.code;
		fName = p.name;
		fDescription = p.description ?? '';
		fStartDate = p.startDate ?? '';
		fEndDate = p.endDate ?? '';
		formError = null;
		mode = { kind: 'edit', project: p };
	}

	function cancel() {
		mode = { kind: 'none' };
		resetForm();
	}

	async function submit() {
		formError = null;
		if (!fCode.trim()) {
			formError = i18nMsg('projects-err-code', 'Le code du projet est requis.');
			return;
		}
		if (!fName.trim()) {
			formError = i18nMsg('projects-err-name', 'Le nom du projet est requis.');
			return;
		}
		submitting = true;
		try {
			const base = {
				parentId: fParentId,
				code: fCode.trim(),
				name: fName.trim(),
				description: fDescription.trim() || null,
				startDate: fStartDate || null,
				endDate: fEndDate || null,
			};
			if (mode.kind === 'edit') {
				await updateProject(mode.project.id, { ...base, version: mode.project.version });
				toast.success(i18nMsg('projects-updated', 'Projet mis à jour.'));
			} else {
				await createProject(base);
				toast.success(i18nMsg('projects-created', 'Projet créé.'));
			}
			mode = { kind: 'none' };
			await load();
		} catch (err) {
			formError = isApiError(err)
				? err.message
				: i18nMsg('projects-save-error', 'Enregistrement impossible.');
		} finally {
			submitting = false;
		}
	}

	async function toggleArchive(p: ProjectResponse) {
		try {
			if (p.archived) {
				await unarchiveProject(p.id, p.version);
				toast.success(i18nMsg('projects-unarchived', 'Projet désarchivé.'));
			} else {
				await archiveProject(p.id, p.version);
				toast.success(i18nMsg('projects-archived', 'Projet archivé.'));
			}
			await load();
		} catch (err) {
			toast.error(
				isApiError(err) ? err.message : i18nMsg('projects-archive-error', 'Opération impossible.'),
			);
			// Recharge pour rafraîchir les versions (ex. conflit optimiste 409 concurrent) :
			// évite qu'un retour d'erreur laisse une version périmée et fasse échouer le retry.
			await load();
		}
	}

	async function onToggleArchivedFilter() {
		includeArchived = !includeArchived;
		await load();
	}
</script>

<svelte:head>
	<title>{i18nMsg('projects-title', 'Projets analytiques')} — Kesh</title>
</svelte:head>

<div class="mb-6 flex items-center justify-between">
	<div>
		<h1 class="text-2xl font-semibold">{i18nMsg('projects-title', 'Projets analytiques')}</h1>
		<p class="text-sm text-text-muted">
			{i18nMsg(
				'projects-subtitle',
				'Regroupez vos dépenses et revenus par projet (rénovation, investissement) pour les analyser isolément.',
			)}
		</p>
	</div>
	<Button data-testid="project-new" onclick={openCreate}>
		{i18nMsg('projects-new', 'Nouveau projet')}
	</Button>
</div>

<label class="mb-4 flex items-center gap-2 text-sm">
	<input type="checkbox" checked={includeArchived} onchange={onToggleArchivedFilter} />
	{i18nMsg('projects-show-archived', 'Afficher les projets archivés')}
</label>

{#if mode.kind !== 'none'}
	<form
		class="mb-6 space-y-3 rounded border border-border p-4"
		data-testid="project-form"
		onsubmit={(e) => {
			e.preventDefault();
			submit();
		}}
	>
		<h2 class="font-medium">
			{mode.kind === 'edit'
				? i18nMsg('projects-form-edit', 'Modifier le projet')
				: i18nMsg('projects-form-create', 'Nouveau projet')}
		</h2>
		<div class="grid grid-cols-2 gap-3">
			<label class="block text-sm">
				{i18nMsg('projects-field-code', 'Code')}
				<Input data-testid="project-code" maxlength={32} bind:value={fCode} />
			</label>
			<label class="block text-sm">
				{i18nMsg('projects-field-name', 'Nom')}
				<Input data-testid="project-name" maxlength={150} bind:value={fName} />
			</label>
			<label class="col-span-2 block text-sm">
				{i18nMsg('projects-field-description', 'Description (optionnel)')}
				<Input bind:value={fDescription} />
			</label>
			<label class="block text-sm">
				{i18nMsg('projects-field-parent', 'Projet parent (optionnel)')}
				<select class="mt-1 w-full rounded border px-2 py-1" bind:value={fParentId}>
					<option value={null}>{i18nMsg('projects-parent-none', '— Aucun (projet racine)')}</option>
					{#each roots as r (r.id)}
						<!-- Une racine archivée ne peut pas être parent (le backend rejette) → exclue. -->
						{#if !r.archived && !(mode.kind === 'edit' && mode.project.id === r.id)}
							<option value={r.id}>{r.code} — {r.name}</option>
						{/if}
					{/each}
				</select>
			</label>
			<div class="grid grid-cols-2 gap-2">
				<label class="block text-sm">
					{i18nMsg('projects-field-start', 'Début')}
					<input type="date" class="mt-1 w-full rounded border px-2 py-1" bind:value={fStartDate} />
				</label>
				<label class="block text-sm">
					{i18nMsg('projects-field-end', 'Fin')}
					<input type="date" class="mt-1 w-full rounded border px-2 py-1" bind:value={fEndDate} />
				</label>
			</div>
		</div>
		{#if formError}
			<p class="text-sm text-destructive" data-testid="project-form-error">{formError}</p>
		{/if}
		<div class="flex gap-2">
			<Button type="submit" data-testid="project-submit" disabled={submitting}>
				{submitting ? '…' : i18nMsg('projects-save', 'Enregistrer')}
			</Button>
			<Button type="button" variant="outline" onclick={cancel}>
				{i18nMsg('common-cancel', 'Annuler')}
			</Button>
		</div>
	</form>
{/if}

{#if loading}
	<p class="text-sm text-text-muted">{i18nMsg('common-loading', 'Chargement…')}</p>
{:else if loadError}
	<p class="text-sm text-destructive">{loadError}</p>
{:else if projects.length === 0}
	<p class="text-sm text-text-muted" data-testid="projects-empty">
		{i18nMsg('projects-empty', 'Aucun projet. Créez votre premier projet pour commencer.')}
	</p>
{:else}
	<ul class="space-y-2" data-testid="projects-list">
		{#each roots as root (root.id)}
			<li class="rounded border border-border p-3" data-testid="project-row">
				<div class="flex items-center justify-between gap-3">
					<div class="text-sm">
						<span class="font-medium">{root.code}</span> — {root.name}
						{#if root.archived}
							<span class="ml-2 text-xs text-text-muted"
								>({i18nMsg('projects-archived-tag', 'archivé')})</span
							>
						{/if}
					</div>
					<div class="flex gap-2">
						<Button variant="outline" size="sm" onclick={() => openEdit(root)}>
							{i18nMsg('projects-edit', 'Modifier')}
						</Button>
						<Button variant="outline" size="sm" onclick={() => toggleArchive(root)}>
							{root.archived
								? i18nMsg('projects-unarchive', 'Désarchiver')
								: i18nMsg('projects-archive', 'Archiver')}
						</Button>
					</div>
				</div>
				{#each childrenOf(root.id) as child (child.id)}
					<div
						class="mt-2 flex items-center justify-between gap-3 border-l-2 border-border pl-3 text-sm"
						data-testid="project-child"
					>
						<div>
							<span class="font-medium">{child.code}</span> — {child.name}
							{#if child.archived}
								<span class="ml-2 text-xs text-text-muted"
									>({i18nMsg('projects-archived-tag', 'archivé')})</span
								>
							{/if}
						</div>
						<div class="flex gap-2">
							<Button variant="outline" size="sm" onclick={() => openEdit(child)}>
								{i18nMsg('projects-edit', 'Modifier')}
							</Button>
							<Button variant="outline" size="sm" onclick={() => toggleArchive(child)}>
								{child.archived
									? i18nMsg('projects-unarchive', 'Désarchiver')
									: i18nMsg('projects-archive', 'Archiver')}
							</Button>
						</div>
					</div>
				{/each}
			</li>
		{/each}
	</ul>
{/if}
