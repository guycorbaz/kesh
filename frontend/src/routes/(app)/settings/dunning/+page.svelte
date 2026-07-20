<script lang="ts">
	// Story 21-4 — Réglages des rappels débiteurs (dunning).
	// Calqué sur settings/vat-rates (CRUD collection) + settings/invoicing (singleton
	// grâce, verrou optimiste). RBAC : guard Admin dans +page.ts (défense backend en sus).
	import { onMount } from 'svelte';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { toast } from 'svelte-sonner';
	import { isApiError } from '$lib/shared/utils/api-client';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import {
		listDunningLevels,
		createDunningLevel,
		updateDunningLevel,
		deleteDunningLevel,
		getDunningSettings,
		updateDunningSettings,
		type DunningLevelResponse,
	} from '$lib/features/dunning';

	// IDs DOM stables et HTTP-LAN-safe ($props.id() — pas de crypto.randomUUID,
	// indisponible hors secure-context sur déploiement NAS HTTP, cf. #145).
	const uid = $props.id();

	let levels = $state<DunningLevelResponse[]>([]);
	let gracePeriodDays = $state(5);
	let graceVersion = $state(0);
	let loading = $state(true);
	let loadError = $state('');

	// Machine à états du formulaire de niveau.
	type Mode = { kind: 'none' } | { kind: 'create' } | { kind: 'edit'; level: DunningLevelResponse };
	let mode = $state<Mode>({ kind: 'none' });
	let fDelay = $state('10');
	let fFee = $state('0.00');
	let formError = $state('');
	let submitting = $state(false);
	let confirmDelete = $state<DunningLevelResponse | null>(null);

	let graceInput = $state('5');
	let graceSaving = $state(false);
	let graceError = $state('');

	// Exemple des délais cumulés (D7) — calculé sur les niveaux PERSISTÉS (jamais le
	// formulaire en cours) : niveau N part à `échéance + grâce + Σ(délais 1..N)`.
	let example = $derived.by(() => {
		const grace = Number.isFinite(gracePeriodDays) ? gracePeriodDays : 0;
		let cumulative = grace;
		return levels
			.slice()
			.sort((a, b) => a.levelNumber - b.levelNumber)
			.map((l) => {
				cumulative += l.delayDays;
				return { level: l.levelNumber, days: cumulative };
			});
	});

	async function load() {
		loading = true;
		loadError = '';
		try {
			// Séquentiel (PAS Promise.all) : le GET des réglages déclenche le seed lazy
			// des 3 niveaux par défaut côté backend ; on liste APRÈS pour les voir.
			const settings = await getDunningSettings();
			const lv = await listDunningLevels();
			levels = lv;
			gracePeriodDays = settings.gracePeriodDays;
			graceVersion = settings.version;
			graceInput = String(settings.gracePeriodDays);
		} catch (e) {
			loadError = isApiError(e)
				? e.message
				: i18nMsg('dunning-load-error', 'Impossible de charger les réglages de rappel.');
		} finally {
			loading = false;
		}
	}
	onMount(load);

	function openCreate() {
		fDelay = '10';
		fFee = '0.00';
		formError = '';
		mode = { kind: 'create' };
	}
	function openEdit(level: DunningLevelResponse) {
		fDelay = String(level.delayDays);
		fFee = level.feeAmount;
		formError = '';
		mode = { kind: 'edit', level };
	}
	function closeForm() {
		mode = { kind: 'none' };
		formError = '';
	}

	async function submitForm() {
		formError = '';
		const delay = Number(fDelay);
		if (!Number.isInteger(delay) || delay < 0) {
			formError = i18nMsg('dunning-form-error-delay', 'Le délai doit être un entier positif.');
			return;
		}
		if (fFee.trim() === '' || Number.isNaN(Number(fFee))) {
			formError = i18nMsg('dunning-form-error-fee', 'Les frais doivent être un montant valide.');
			return;
		}
		submitting = true;
		try {
			if (mode.kind === 'create') {
				await createDunningLevel({ delayDays: delay, feeAmount: fFee });
				toast.success(i18nMsg('dunning-created', 'Niveau de rappel ajouté.'));
			} else if (mode.kind === 'edit') {
				await updateDunningLevel(mode.level.id, {
					delayDays: delay,
					feeAmount: fFee,
					version: mode.level.version,
				});
				toast.success(i18nMsg('dunning-updated', 'Niveau de rappel mis à jour.'));
			}
			mode = { kind: 'none' };
			await load();
		} catch (e) {
			if (isApiError(e) && e.code === 'OPTIMISTIC_LOCK_CONFLICT') {
				toast.error(i18nMsg('dunning-conflict', 'Le niveau a été modifié entre-temps, rechargé.'));
				await load();
				mode = { kind: 'none' };
			} else {
				formError = isApiError(e) ? e.message : i18nMsg('dunning-form-error', 'Échec de l’enregistrement.');
			}
		} finally {
			submitting = false;
		}
	}

	async function doDelete() {
		if (!confirmDelete) return;
		const level = confirmDelete;
		submitting = true;
		try {
			await deleteDunningLevel(level.id, level.version);
			toast.success(i18nMsg('dunning-deleted', 'Niveau de rappel supprimé.'));
			confirmDelete = null;
			await load();
		} catch (e) {
			toast.error(isApiError(e) ? e.message : i18nMsg('dunning-form-error', 'Échec de la suppression.'));
			await load();
			confirmDelete = null;
		} finally {
			submitting = false;
		}
	}

	async function saveGrace() {
		graceError = '';
		const grace = Number(graceInput);
		if (!Number.isInteger(grace) || grace < 0) {
			graceError = i18nMsg('dunning-form-error-delay', 'La période de grâce doit être un entier positif.');
			return;
		}
		graceSaving = true;
		try {
			const updated = await updateDunningSettings({ gracePeriodDays: grace, version: graceVersion });
			gracePeriodDays = updated.gracePeriodDays;
			graceVersion = updated.version;
			graceInput = String(updated.gracePeriodDays);
			toast.success(i18nMsg('dunning-grace-saved', 'Période de grâce enregistrée.'));
		} catch (e) {
			if (isApiError(e) && e.code === 'OPTIMISTIC_LOCK_CONFLICT') {
				toast.error(i18nMsg('dunning-conflict', 'Les réglages ont changé entre-temps, rechargés.'));
				await load();
			} else {
				graceError = isApiError(e) ? e.message : i18nMsg('dunning-form-error', 'Échec de l’enregistrement.');
			}
		} finally {
			graceSaving = false;
		}
	}
</script>

<div class="flex flex-col gap-6">
	<div>
		<h1 class="text-2xl font-bold">{i18nMsg('dunning-title', 'Rappels débiteurs')}</h1>
		<p class="mt-1 text-sm text-text-muted">
			{i18nMsg('dunning-subtitle', 'Configurez les niveaux de rappel, les délais et les frais de relance.')}
		</p>
	</div>

	{#if loading}
		<p class="text-sm text-text-muted">{i18nMsg('common-loading', 'Chargement…')}</p>
	{:else if loadError}
		<p class="text-sm text-error" data-testid="dunning-load-error" role="alert">{loadError}</p>
	{:else}
		<!-- Période de grâce -->
		<section class="rounded-lg border border-border bg-white p-6 shadow-sm">
			<h2 class="text-lg font-semibold">{i18nMsg('dunning-grace-heading', 'Période de grâce')}</h2>
			<p class="mt-1 text-sm text-text-muted">
				{i18nMsg('dunning-grace-help', 'Jours après l’échéance avant que le 1er rappel ne devienne éligible.')}
			</p>
			<div class="mt-3 flex items-end gap-3">
				<div>
					<label class="text-sm font-medium" for="{uid}-grace">{i18nMsg('dunning-grace-label', 'Grâce (jours)')}</label>
					<Input id="{uid}-grace" type="number" min="0" step="1" bind:value={graceInput} data-testid="dunning-grace-input" class="w-32" />
				</div>
				<Button onclick={saveGrace} disabled={graceSaving} data-testid="dunning-grace-save">
					{i18nMsg('dunning-grace-save', 'Enregistrer')}
				</Button>
			</div>
			{#if graceError}<p class="mt-2 text-sm text-error" role="alert">{graceError}</p>{/if}
		</section>

		<!-- Niveaux de rappel -->
		<section class="rounded-lg border border-border bg-white p-6 shadow-sm">
			<div class="flex items-center justify-between">
				<h2 class="text-lg font-semibold">{i18nMsg('dunning-levels-heading', 'Niveaux de rappel')}</h2>
				<Button variant="outline" size="sm" onclick={openCreate} data-testid="dunning-level-new">
					{i18nMsg('dunning-level-new', 'Ajouter un niveau')}
				</Button>
			</div>

			{#if levels.length === 0}
				<p class="mt-3 text-sm text-text-muted">{i18nMsg('dunning-empty', 'Aucun niveau configuré — les rappels sont désactivés.')}</p>
			{:else}
				<table class="mt-3 w-full text-sm">
					<thead>
						<tr class="border-b border-border text-left text-text-muted">
							<th class="py-2">{i18nMsg('dunning-col-level', 'Niveau')}</th>
							<th class="py-2">{i18nMsg('dunning-col-delay', 'Délai (jours)')}</th>
							<th class="py-2">{i18nMsg('dunning-col-fee', 'Frais (CHF)')}</th>
							<th class="py-2"><span class="sr-only">{i18nMsg('dunning-col-actions', 'Actions')}</span></th>
						</tr>
					</thead>
					<tbody>
						{#each levels as level (level.id)}
							<tr class="border-b border-border/50" data-testid="dunning-level-row">
								<td class="py-2">{level.levelNumber}</td>
								<td class="py-2">{level.delayDays}</td>
								<td class="py-2">{level.feeAmount}</td>
								<td class="py-2 text-right">
									<Button variant="ghost" size="sm" onclick={() => openEdit(level)}>{i18nMsg('dunning-edit', 'Modifier')}</Button>
									<Button variant="ghost" size="sm" onclick={() => (confirmDelete = level)}>{i18nMsg('dunning-delete', 'Supprimer')}</Button>
								</td>
							</tr>
						{/each}
					</tbody>
				</table>
			{/if}

			<!-- Exemple des délais cumulés (D7) -->
			{#if example.length > 0}
				<div class="mt-4 rounded border border-info/30 bg-info/5 p-3 text-sm" data-testid="dunning-example">
					<p class="font-medium">{i18nMsg('dunning-example-heading', 'Échéancier prévisionnel')}</p>
					<ul class="mt-1 list-disc pl-5">
						{#each example as ex (ex.level)}
							<li>{i18nMsg('dunning-example-line', '{$level}. rappel proposé {$days} j après l’échéance', { level: ex.level, days: ex.days })}</li>
						{/each}
					</ul>
				</div>
			{/if}

			<!-- Avertissement CGV (D13) -->
			<p class="mt-4 rounded border border-warning/30 bg-warning/5 p-3 text-xs text-text-muted" data-testid="dunning-cgv-hint">
				{i18nMsg('dunning-cgv-hint', 'Les frais de rappel ne sont exigibles qu’avec une base contractuelle (CGV). Ils ne sont pas inclus dans le QR de la facture jointe.')}
			</p>
		</section>

		<!-- Formulaire création / édition -->
		{#if mode.kind !== 'none'}
			<section class="rounded-lg border border-border bg-white p-6 shadow-sm">
				<h2 class="text-lg font-semibold">
					{mode.kind === 'create' ? i18nMsg('dunning-level-new', 'Ajouter un niveau') : i18nMsg('dunning-edit', 'Modifier le niveau')}
				</h2>
				<div class="mt-3 flex flex-col gap-3 sm:flex-row sm:items-end">
					<div>
						<label class="text-sm font-medium" for="{uid}-delay">{i18nMsg('dunning-delay-label', 'Délai (jours)')}</label>
						<p class="text-xs text-text-muted">{i18nMsg('dunning-delay-help', 'Jours depuis l’étape précédente (échéance + grâce pour le 1er).')}</p>
						<Input id="{uid}-delay" type="number" min="0" step="1" bind:value={fDelay} data-testid="dunning-form-delay" class="w-32" />
					</div>
					<div>
						<label class="text-sm font-medium" for="{uid}-fee">{i18nMsg('dunning-fee-label', 'Frais (CHF)')}</label>
						<!-- type="text" (pas number) : le montant décimal doit rester une STRING
						     ($lib/components/ui/input coerce type=number en nombre, cassant le Decimal
						     backend qui attend une string via serde-str). Bornes validées serveur. -->
						<Input id="{uid}-fee" type="text" inputmode="decimal" bind:value={fFee} data-testid="dunning-form-fee" class="w-32" />
					</div>
					<div class="flex gap-2">
						<Button onclick={submitForm} disabled={submitting} data-testid="dunning-form-submit">{i18nMsg('dunning-form-submit', 'Enregistrer')}</Button>
						<Button variant="outline" onclick={closeForm}>{i18nMsg('dunning-form-cancel', 'Annuler')}</Button>
					</div>
				</div>
				{#if formError}<p class="mt-2 text-sm text-error" data-testid="dunning-form-error" role="alert">{formError}</p>{/if}
			</section>
		{/if}

		<!-- Confirmation de suppression -->
		{#if confirmDelete}
			<section class="rounded-lg border border-error/40 bg-error/5 p-6 shadow-sm" data-testid="dunning-level-delete-confirm">
				<p class="text-sm">{i18nMsg('dunning-delete-confirm-body', 'Supprimer ce niveau de rappel ? Les niveaux suivants seront renumérotés.')}</p>
				<div class="mt-3 flex gap-2">
					<Button variant="destructive" onclick={doDelete} disabled={submitting}>{i18nMsg('dunning-delete-confirm-action', 'Supprimer')}</Button>
					<Button variant="outline" onclick={() => (confirmDelete = null)}>{i18nMsg('dunning-form-cancel', 'Annuler')}</Button>
				</div>
			</section>
		{/if}
	{/if}
</div>
