<!--
  Story 17-3d — Panneau d'import complet d'installation (.keshbackup).
  Logique + UI en composant feature (testable + ownership i18n `admin-restore-*`).
  La route `(app)/admin/restore/+page.svelte` ne fait que le rendre.

  ⚠️ OPÉRATION DESTRUCTRICE : remplace TOUTES les données de l'installation.
  Confirmation forte via modal `Dialog` (DC-D3) — l'upload n'est déclenché QUE
  par « Confirmer » du modal, jamais par le bouton « Importer » directement.
  Succès → avis de ce que la restauration a rendu, puis déconnexion + redirection
  /login au clic « Se reconnecter » (DC-D4, sessionInvalidated ; Story 31-1b, AC19,
  C-31-1-39). Gating RBAC doublé : guard `+page.ts` (DC-D1) + `require_admin_role`
  backend.

  Story 31-1b (#503) : rendu aussi par `/onboarding/restore` (restaurer une
  installation neuve depuis l'assistant, AC15) ; avis permanent sur le `.env`
  (AC17) ; issue inconnue d'un import long (AC18b).

  Story 36-5 (#931, KF-056) : l'import invalide la session au serveur ; sans
  précaution, le premier battement d'activité refusé menait à la connexion avant
  que le compte rendu soit lu. Le panneau est le SEUL à poser le maintien de
  session (`authState.holdForRestore`) : à la confirmation pour
  `importHoldMs(B)` (B = la durée d'inactivité réglée, 60 min si inconnue) ; au
  succès, réarmé pour B, suivi de session arrêté, puis fin de session au bout de B
  si l'écran reste ouvert ; après un échec certain ou une issue inconnue, échéance
  ramenée à au plus B (jamais allongée) et levé au premier geste qui suit le message ; au 401 de l'import, levé aussitôt et la
  session terminée ; au démontage, levé (et la session terminée si le compte
  rendu était affiché, sauf « Se reconnecter » en cours).
-->
<script lang="ts">
	import { onDestroy } from 'svelte';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { expireSession, isApiError } from '$lib/shared/utils/api-client';
	import { isUnknownOutcome } from '$lib/shared/utils/unknown-outcome';
	import { authState } from '$lib/app/stores/auth.svelte';
	import { sessionIdle } from '$lib/app/stores/session-idle.svelte';
	import { Button } from '$lib/components/ui/button';
	import * as Dialog from '$lib/components/ui/dialog';
	import { toast } from 'svelte-sonner';
	import BackupEnvNotice from '$lib/features/admin-backup/BackupEnvNotice.svelte';
	import { importHoldMs, uploadFullImport, type FullImportResponse } from './admin-restore.api';

	let selectedFile = $state<File | null>(null);
	let confirmOpen = $state(false);
	let importing = $state(false);
	let errorMsg = $state<string | null>(null);
	/**
	 * Story 31-1b (AC19) — réponse d'un import réussi. Non nulle : l'installation est
	 * remplacée, la session invalidée ; le panneau n'attend plus que « Se reconnecter ».
	 */
	let imported = $state<FullImportResponse | null>(null);
	let reconnecting = $state(false);
	/** Story 31-1b (AC18b) — le dernier import a fini sans réponse : « Se reconnecter » est proposé. */
	let unknownOutcome = $state(false);

	/** Story 36-5 — le panneau a quitté l'écran : une réponse tardive n'arme plus rien. */
	let destroyed = false;
	/** Story 36-5 — minuterie de fin de lecture du compte rendu (B), armée au succès seul. */
	let readingTimer: ReturnType<typeof setTimeout> | null = null;
	/** Story 36-5 — retire l'écouteur de la levée au premier geste, s'il est posé. */
	let disarmGesture: (() => void) | null = null;

	/**
	 * Story 36-5 — B : la durée d'inactivité réglée (ms), 60 min si l'écran ne la
	 * connaît pas encore (revue P1, E1-4 : relue au succès et à l'échec, non figée à la
	 * confirmation).
	 */
	function readingMs(): number {
		return (sessionIdle.minutes ?? 60) * 60_000;
	}

	/** Story 36-5 — fin de session comme le chemin 401 ordinaire, maintien levé d'abord. */
	function endSession(): void {
		authState.releaseRestoreHold();
		expireSession();
	}

	/**
	 * Story 36-5 — après un échec certain ou une issue inconnue : le maintien est levé
	 * au premier geste (`pointerdown` ou `keydown` sur la fenêtre), qui vaut lecture du
	 * message. Écouteur posé une fois, en capture, retiré après usage, au démontage et
	 * au départ d'un nouvel import (`confirmImport` ; revue P1, E1-1).
	 */
	function armReleaseOnNextGesture(): void {
		if (destroyed || disarmGesture !== null) return;
		const onGesture = () => {
			disarmGesture?.();
			authState.releaseRestoreHold();
		};
		const options = { capture: true };
		window.addEventListener('pointerdown', onGesture, options);
		window.addEventListener('keydown', onGesture, options);
		disarmGesture = () => {
			window.removeEventListener('pointerdown', onGesture, options);
			window.removeEventListener('keydown', onGesture, options);
			disarmGesture = null;
		};
	}

	onDestroy(() => {
		destroyed = true;
		disarmGesture?.();
		// Phase 2 (compte rendu affiché, minuterie armée) : la session est connue
		// morte et le suivi arrêté — la quitter mène à la connexion, sauf si
		// « Se reconnecter » y mène déjà.
		const reading = readingTimer !== null;
		if (readingTimer !== null) {
			clearTimeout(readingTimer);
			readingTimer = null;
		}
		authState.releaseRestoreHold();
		if (reading && !reconnecting) expireSession();
	});

	/**
	 * Story 31-1b (AC18b) — échecs après lesquels l'issue de l'import est inconnue :
	 * la réponse n'est pas arrivée (délai du client dépassé, connexion coupée) ou un
	 * proxy inverse a abandonné l'attente (502, 504 — Kesh n'émet ni l'un ni
	 * l'autre). Le serveur, lui, va au bout de l'import (AC8b) : l'installation est
	 * peut-être déjà remplacée, et « réessayez » inviterait à la remplacer encore.
	 * Le 503 n'en est pas : Kesh l'émet quand la base est injoignable, presque
	 * toujours avant tout import. ⚠️ Angle mort assumé (revue P1, B-3 ; corrigé en
	 * revue P2, M-1) : un échec au `COMMIT` de la restauration ne rend PAS un 503 —
	 * `routes/admin.rs` le convertit en `AdminFullImportFailed` (500,
	 * `ADMIN_FULL_IMPORT_FAILED`), dont le message dit l'état préservé et invite à
	 * réessayer. C'est vrai pour la base dans le cas ordinaire (le `COMMIT` échoué
	 * est annulé), mais si la coupure a eu lieu après que le serveur a validé, l'issue
	 * réelle est inconnue et l'écran le présente comme un échec ordinaire ; cas rare,
	 * et un nouvel import rejouerait les mêmes données. Les pièces déjà écrites
	 * restent dans le dossier des documents (angle mort E-3 de la 31-1a, au manuel).
	 *
	 * Après une issue inconnue, l'import n'est plus proposé (revue P1, E-LOW-3) : on
	 * vérifie d'abord en se reconnectant ; recharger la page le rouvre.
	 *
	 * Story 32-1b (#711 E-7) : le prédicat est extrait dans
	 * `$lib/shared/utils/unknown-outcome` et partagé avec « Sauvegarder maintenant » ;
	 * ce qui précède reste propre à l'import. Faux pour une erreur qui n'est pas une
	 * `ApiError`, comme avant l'extraction.
	 */
	const issueInconnue = isUnknownOutcome;

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
		if (!selectedFile || importing || imported || unknownOutcome) return;
		errorMsg = null;
		confirmOpen = true;
	}

	/** Mappe une erreur d'import en message lisible (AC20). */
	function importErrorMessage(err: unknown): string {
		if (issueInconnue(err)) {
			return i18nMsg(
				'admin-restore-error-unknown-outcome',
				"La réponse du serveur n'est pas arrivée : l'import a pu aboutir. Reconnectez-vous pour vérifier l'état de l'installation.",
			);
		}
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
		if (!selectedFile || importing || imported) return;
		importing = true;
		// Story 36-5 (revue P1, E1-1) — l'écouteur laissé par un échec précédent est
		// retiré : un nouvel import activé par un `click` seul (technologie d'assistance)
		// le laisserait lever le maintien au premier geste de la phase 1 ou 2.
		disarmGesture?.();
		confirmOpen = false;
		errorMsg = null;
		unknownOutcome = false;
		// Story 36-5 — le maintien AVANT l'envoi, avant tout `await`.
		const b = readingMs();
		authState.holdForRestore(importHoldMs(b));
		/** Vrai quand le maintien ne doit pas être levé au premier geste (succès, 401). */
		let keepHold = false;

		try {
			// Story 31-1b (AC19, C-31-1-39) : au succès, AUCUNE redirection ici — le
			// panneau dit ce que la restauration a rendu, et attend « Se reconnecter ».
			const result = await uploadFullImport(selectedFile);
			// Story 36-5 (V1-4) — l'écran est quitté : régime ordinaire, rien à armer.
			if (destroyed) return;
			imported = result;
			keepHold = true;
			// Story 36-5 — la session n'existe plus au serveur : lecture bornée par B,
			// plus aucun battement, puis fin de session si l'écran reste ouvert. La
			// minuterie est armée AVANT le maintien (revue P1, B1-3) : l'échéance du
			// maintien ne la précède pas ; si le navigateur la retarde (onglet en arrière-
			// plan), un 401 venu entre les deux mène à la même connexion par le chemin 401.
			// Rien ici ne jette (une lecture, un `setTimeout`, une affectation, `stop()` qui
			// retire des écouteurs) : aucun message d'échec sous le compte rendu (E1-3).
			const reading = readingMs();
			readingTimer = setTimeout(() => {
				readingTimer = null;
				endSession();
			}, reading);
			authState.holdForRestore(reading);
			sessionIdle.stop();
		} catch (err) {
			if (destroyed) return;
			// Story 36-5 (V2-5) — l'import lui-même refusé en 401 : la session était
			// morte avant tout import ; fin de session, comme hors maintien.
			if (isApiError(err) && err.code === 'UNAUTHENTICATED') {
				keepHold = true;
				endSession();
				return;
			}
			unknownOutcome = issueInconnue(err);
			const msg = importErrorMessage(err);
			errorMsg = msg;
			toast.error(msg);
		} finally {
			importing = false;
			// Une instance démontée ne touche plus au maintien (revue P2, E2-1) : il est
			// un état de module, et pourrait appartenir à un autre panneau monté depuis.
			if (!keepHold && !destroyed) {
				// Story 36-5 — échec certain ou issue inconnue (aussi si le `catch` a jeté) :
				// l'échéance est ramenée à au plus B à partir de maintenant, jamais allongée
				// (revue P1, E1-2), et le maintien est levé au premier geste.
				authState.shortenRestoreHold(readingMs());
				armReleaseOnNextGesture();
			}
		}
	}

	/**
	 * Story 31-1b (AC19) — clic « Se reconnecter », après tout import réussi.
	 *
	 * DC-D4 : la session est invalidée (refresh_tokens remplacés). logout
	 * best-effort PUIS redirection **garantie** vers /login — même si `logout()`
	 * jetait, on redirige quand même (review P1 : ne pas laisser une exception
	 * logout masquer la redirection après un import réussi, ce qui afficherait
	 * une fausse erreur sur une installation déjà remplacée).
	 */
	async function relogin() {
		if (reconnecting) return;
		reconnecting = true;
		try {
			await authState.logout();
		} catch {
			// best-effort — le cookie est déjà invalide côté serveur.
		}
		window.location.replace('/login');
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

	<BackupEnvNotice />

	<div class="flex flex-col gap-2">
		<label class="text-sm font-medium text-text" for="admin-restore-file">
			{i18nMsg('admin-restore-file-label', 'Fichier .keshbackup à importer')}
		</label>
		<input
			id="admin-restore-file"
			type="file"
			accept=".keshbackup"
			onchange={onFileChange}
			disabled={importing || imported !== null || unknownOutcome}
			aria-describedby={errorMsg ? 'admin-restore-error' : undefined}
			data-testid="admin-restore-file-input"
			class="text-sm"
		/>
	</div>

	{#if errorMsg}
		<div
			id="admin-restore-error"
			role="alert"
			class="rounded-md border border-destructive bg-destructive/10 px-3 py-2 text-sm text-destructive"
			data-testid="admin-restore-error"
		>
			{errorMsg}
		</div>
	{/if}

	{#if imported}
		<!-- Story 31-1b (AC19) : ce que la restauration a rendu, lu avant la reconnexion. -->
		<div
			role="status"
			class="rounded-md border border-success/40 bg-success/10 px-3 py-2 text-sm text-text"
			data-testid="admin-restore-success"
		>
			<p>
				{i18nMsg(
					'admin-restore-success',
					"Import réussi : l'installation est celle de la sauvegarde. Reconnectez-vous avec les identifiants de la sauvegarde.",
				)}
			</p>
			{#if typeof imported.filesRestored === 'number'}
				<p class="mt-1" data-testid="admin-restore-files-restored">
					{i18nMsg('admin-restore-files-restored', 'Pièces justificatives restaurées : { $count }.', {
						count: imported.filesRestored,
					})}
				</p>
			{/if}
		</div>
		{#if imported.filesMissing}
			<div
				role="status"
				class="rounded-md border border-warning bg-warning/10 px-3 py-2 text-sm text-text"
				data-testid="admin-restore-files-missing"
			>
				{i18nMsg(
					'admin-restore-files-missing',
					"Pièces référencées introuvables après la restauration : { $count }. Restaurez le dossier des documents depuis une autre sauvegarde.",
					{ count: imported.filesMissing },
				)}
			</div>
		{/if}
	{/if}

	{#if imported || unknownOutcome}
		<div>
			<Button
				onclick={relogin}
				disabled={reconnecting}
				aria-busy={reconnecting}
				data-testid="admin-restore-relogin"
			>
				{i18nMsg('admin-restore-action-relogin', 'Se reconnecter')}
			</Button>
		</div>
	{/if}

	<div>
		<Button
			variant="destructive"
			onclick={requestImport}
			disabled={!selectedFile || importing || imported !== null || unknownOutcome}
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
					"Cette action va remplacer TOUTES les données de l'installation actuelle. Une sauvegarde de l'état actuel sera créée côté serveur avant l'import. À la fin, votre session sera fermée : vous vous reconnecterez avec les identifiants de l'instance importée.",
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
