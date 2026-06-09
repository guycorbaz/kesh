<!--
  Story 17-3d — Panneau d'import complet d'installation (.keshbackup).
  Logique + UI en composant feature (testable + ownership i18n `admin-restore-*`).
  La route `(app)/admin/restore/+page.svelte` ne fait que le rendre.

  ⚠️ OPÉRATION DESTRUCTRICE : remplace TOUTES les données de l'installation.
  Confirmation forte via modal `Dialog` (DC-D3) — l'upload n'est déclenché QUE
  par « Confirmer » du modal, jamais par le bouton « Importer » directement.
  Succès → déconnexion + redirection /login (DC-D4, sessionInvalidated). Gating
  RBAC doublé : guard `+page.ts` (DC-D1) + `require_admin_role` backend.
-->
<script lang="ts">
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { isApiError } from '$lib/shared/utils/api-client';
	import { authState } from '$lib/app/stores/auth.svelte';
	import { Button } from '$lib/components/ui/button';
	import * as Dialog from '$lib/components/ui/dialog';
	import { toast } from 'svelte-sonner';
	import { uploadFullImport } from './admin-restore.api';

	let selectedFile = $state<File | null>(null);
	let confirmOpen = $state(false);
	let importing = $state(false);
	let errorMsg = $state<string | null>(null);

	function onFileChange(event: Event) {
		const input = event.target as HTMLInputElement;
		const file = input.files?.[0] ?? null;
		errorMsg = null;
		// Rejet précoce d'une extension inattendue (review P1) : `accept` filtre le
		// picker mais pas le drag-drop ; évite d'uploader un mauvais fichier (le
		// backend rejette aussi en 400, mais autant échouer tôt côté client).
		if (file && !file.name.toLowerCase().endsWith('.keshbackup')) {
			selectedFile = null;
			// Vide aussi l'input natif (review P2) pour éviter la divergence
			// état/DOM et permettre de re-sélectionner le même fichier corrigé.
			input.value = '';
			errorMsg = i18nMsg(
				'admin-restore-error-invalid',
				"Fichier de sauvegarde invalide ou corrompu. Vérifiez qu'il s'agit bien d'un fichier .keshbackup produit par Kesh.",
			);
			return;
		}
		selectedFile = file;
	}

	/** Ouvre le modal de confirmation (n'envoie RIEN — DC-D3). */
	function requestImport() {
		if (!selectedFile || importing) return;
		errorMsg = null;
		confirmOpen = true;
	}

	/** Mappe une erreur d'import en message lisible (AC20). */
	function importErrorMessage(err: unknown): string {
		if (isApiError(err)) {
			switch (err.code) {
				case 'IMPORT_VERSION_INCOMPATIBLE': {
					const d = err.details as
						| { sourceMinRequired?: string; binaryVersion?: string }
						| undefined;
					return i18nMsg(
						'admin-restore-error-version',
						'Ce backup requiert une version de Kesh plus récente ({ $src }) que celle installée ({ $bin }). Mettez à jour Kesh avant de réimporter.',
						{ src: d?.sourceMinRequired ?? '?', bin: d?.binaryVersion ?? '?' },
					);
				}
				case 'IMPORT_SCHEMA_MISMATCH': {
					const d = err.details as { table?: string } | undefined;
					return i18nMsg(
						'admin-restore-error-schema',
						'Schéma du backup incompatible avec cette version de Kesh (table { $table }).',
						{ table: d?.table ?? '?' },
					);
				}
				case 'INVALID_BACKUP_STRUCTURE':
					return i18nMsg(
						'admin-restore-error-invalid',
						'Fichier de sauvegarde invalide ou corrompu. Vérifiez qu’il s’agit bien d’un fichier .keshbackup produit par Kesh.',
					);
				default:
					// 413 (taille), 500 (échec restore, état préservé), etc. → message
					// backend déjà lisible (ex. « Erreur serveur (413) »).
					return err.message;
			}
		}
		return i18nMsg(
			'admin-restore-error-generic',
			"Échec de l'import. L'état précédent de l'installation a été préservé.",
		);
	}

	/** Déclenché par « Confirmer » du modal uniquement (DC-D3). */
	async function confirmImport() {
		// `importing = true` en TOUT PREMIER (avant de fermer le modal et avant
		// tout `await`) — garde de ré-entrance définitive contre un double-clic
		// « Confirmer » sur cette opération destructrice (review P1).
		if (!selectedFile || importing) return;
		importing = true;
		confirmOpen = false;
		errorMsg = null;

		let imported = false;
		try {
			await uploadFullImport(selectedFile);
			imported = true;
		} catch (err) {
			const msg = importErrorMessage(err);
			errorMsg = msg;
			toast.error(msg);
		} finally {
			importing = false;
		}

		// DC-D4 : au succès, la session est invalidée (refresh_tokens remplacés).
		// logout best-effort PUIS redirection **garantie** vers /login — même si
		// `logout()` jetait, on redirige quand même (review P1 : ne pas laisser
		// une exception logout masquer la redirection après un import réussi, ce
		// qui afficherait une fausse erreur sur une installation déjà remplacée).
		if (imported) {
			toast.success(
				i18nMsg('admin-restore-toast-success', 'Import réussi — vous allez être déconnecté.'),
			);
			try {
				await authState.logout();
			} catch {
				// best-effort — le cookie est déjà invalide côté serveur.
			}
			window.location.replace('/login');
		}
	}
</script>

<svelte:head>
	<title
		>{i18nMsg('admin-restore-page-title', "Restauration / import d'installation")} – Kesh</title
	>
</svelte:head>

<section class="flex flex-col gap-4" data-testid="admin-restore-panel">
	<header class="flex flex-col gap-1">
		<h1 class="text-2xl font-semibold text-text" data-testid="admin-restore-title">
			{i18nMsg('admin-restore-page-title', "Restauration / import d'installation")}
		</h1>
		<p class="text-sm text-text-muted">
			{i18nMsg(
				'admin-restore-page-description',
				"Téléverse un fichier .keshbackup pour remplacer l'intégralité de l'installation actuelle (migration ou restauration). Opération destructrice : une sauvegarde de l'état actuel est créée côté serveur avant l'import.",
			)}
		</p>
	</header>

	<div class="flex flex-col gap-2">
		<label class="text-sm font-medium text-text" for="admin-restore-file">
			{i18nMsg('admin-restore-file-label', 'Fichier .keshbackup à importer')}
		</label>
		<input
			id="admin-restore-file"
			type="file"
			accept=".keshbackup"
			onchange={onFileChange}
			disabled={importing}
			aria-describedby={errorMsg ? 'admin-restore-error' : undefined}
			data-testid="admin-restore-file-input"
			class="text-sm"
		/>
	</div>

	{#if errorMsg}
		<div
			id="admin-restore-error"
			role="alert"
			class="rounded-md border border-danger/40 bg-danger/10 px-3 py-2 text-sm text-danger"
			data-testid="admin-restore-error"
		>
			{errorMsg}
		</div>
	{/if}

	<div>
		<Button
			variant="destructive"
			onclick={requestImport}
			disabled={!selectedFile || importing}
			aria-busy={importing}
			data-testid="admin-restore-import-button"
		>
			{importing
				? i18nMsg('admin-restore-action-importing', 'Import en cours…')
				: i18nMsg('admin-restore-action-import', "Importer et remplacer l'installation")}
		</Button>
	</div>
</section>

<!-- Confirmation forte (AC19) : l'upload n'est déclenché QUE par « Confirmer ». -->
<Dialog.Root bind:open={confirmOpen}>
	<Dialog.Content showCloseButton={false} data-testid="admin-restore-confirm-dialog">
		<Dialog.Header>
			<Dialog.Title>
				{i18nMsg('admin-restore-confirm-title', "Remplacer toute l'installation ?")}
			</Dialog.Title>
			<Dialog.Description>
				{i18nMsg(
					'admin-restore-confirm-body',
					"Cette action va remplacer TOUTES les données de l'installation actuelle. Une sauvegarde de l'état actuel sera créée côté serveur avant l'import. Vous serez déconnecté et devrez vous reconnecter avec les identifiants de l'instance importée.",
				)}
			</Dialog.Description>
		</Dialog.Header>
		<Dialog.Footer>
			<Button
				variant="ghost"
				onclick={() => (confirmOpen = false)}
				data-testid="admin-restore-cancel"
			>
				{i18nMsg('admin-restore-confirm-cancel', 'Annuler')}
			</Button>
			<Button
				variant="destructive"
				onclick={confirmImport}
				data-testid="admin-restore-confirm"
			>
				{i18nMsg('admin-restore-confirm-ok', "Confirmer le remplacement")}
			</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>
