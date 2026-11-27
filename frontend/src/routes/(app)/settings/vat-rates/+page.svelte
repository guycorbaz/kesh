<!--
  Story 11-1 — Gestion des taux TVA `/settings/vat-rates` (Administrateur).
  Taux groupés par catégorie (courant + historique). Création, « changer le
  taux » (clôture l'ancien PUIS crée le nouveau — ordre imposé par l'invariant
  de non-chevauchement), désactivation. Calque l'UI CRUD bank-accounts/api-keys.
  Pas d'API secure-context-only (déploiement HTTP LAN).
-->
<script lang="ts">
	import { onMount } from 'svelte';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	// v0172-correctifs (#880, § 24 bis) : dates de validité en forme suisse.
	import { formatSwissDate } from '$lib/features/reports/reports.api';
	import { isApiError } from '$lib/shared/utils/api-client';
	import { isUnknownOutcome } from '$lib/shared/utils/unknown-outcome';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { toast } from 'svelte-sonner';
	import {
		listVatRates,
		createVatRate,
		updateVatRate,
		deactivateVatRate,
		type VatRateResponse,
	} from '$lib/features/vat-rates/vat-rates.api';
	import {
		decimalOf,
		validateRateInput,
		validateDateInput,
		type RateInput,
		type RateError,
	} from '$lib/features/vat-rates/vat-rates.validate';

	const KNOWN_CATEGORIES = ['normal', 'reduced', 'special', 'exempt', 'custom'];

	let rates = $state<VatRateResponse[]>([]);
	let loading = $state(true);
	let loadError = $state<string | null>(null);

	type Mode =
		| { kind: 'none' }
		| { kind: 'create' }
		| { kind: 'change'; rate: VatRateResponse }
		| { kind: 'deactivate'; rate: VatRateResponse };
	let mode = $state<Mode>({ kind: 'none' });
	let submitting = $state(false);
	let formError = $state<string | null>(null);

	// Champs de formulaire (création / changement).
	let fCategory = $state('normal');
	let fLabel = $state('');
	// Story 64-1 (#1178) : un `<input type="number">` lié rend un NOMBRE (`null` si vidé).
	let fRate = $state<RateInput>('');
	let fValidFrom = $state('');
	let fValidTo = $state('');

	/** Libellé d'affichage d'une catégorie via sa clé i18n (fallback clé brute). */
	function categoryLabel(category: string): string {
		return i18nMsg(`vat-category-${category}`, category);
	}

	/** Libellé d'un taux : si `label` est une clé connue, la traduire, sinon afficher tel quel. */
	function rateLabel(r: VatRateResponse): string {
		return i18nMsg(r.label, r.label);
	}

	/** Groupe les taux par catégorie, triés (actif d'abord, puis validFrom DESC). */
	const grouped = $derived.by(() => {
		const map = new Map<string, VatRateResponse[]>();
		for (const r of rates) {
			const arr = map.get(r.category) ?? [];
			arr.push(r);
			map.set(r.category, arr);
		}
		for (const arr of map.values()) {
			arr.sort((a, b) => {
				if (a.active !== b.active) return a.active ? -1 : 1;
				return b.validFrom.localeCompare(a.validFrom);
			});
		}
		return [...map.entries()].sort((a, b) => a[0].localeCompare(b[0]));
	});

	async function load() {
		loading = true;
		loadError = null;
		try {
			rates = await listVatRates(true);
		} catch (e) {
			loadError = isApiError(e)
				? e.message
				: i18nMsg('vat-rates-load-error', 'Impossible de charger les taux TVA.');
		} finally {
			loading = false;
		}
	}

	onMount(load);

	function resetForm() {
		fCategory = 'normal';
		fLabel = '';
		fRate = '';
		fValidFrom = '';
		fValidTo = '';
		formError = null;
	}

	function openCreate() {
		resetForm();
		mode = { kind: 'create' };
	}

	function openChange(rate: VatRateResponse) {
		resetForm();
		fCategory = rate.category;
		fRate = '';
		fValidFrom = '';
		mode = { kind: 'change', rate };
	}

	function cancel() {
		mode = { kind: 'none' };
		formError = null;
	}

	/** Message du refus d'un taux avant l'envoi (Story 64-1, D2). */
	function rateErrorMessage(code: RateError): string {
		switch (code) {
			case 'required':
				return i18nMsg('vat-rates-err-rate-required', 'Saisissez un taux.');
			case 'invalid':
				return i18nMsg('vat-rates-err-rate-invalid', 'Le taux doit être un nombre.');
			case 'range':
				return i18nMsg('vat-rates-err-rate-range', 'Le taux doit être compris entre 0 et 100.');
			case 'decimals':
				return i18nMsg(
					'vat-rates-err-rate-decimals',
					'Le taux ne peut avoir plus de deux décimales.',
				);
		}
	}

	/** Cas évidents refusés AVANT toute écriture ; `null` si le formulaire peut partir. */
	function inputError(dateMsg: string): string | null {
		// Revue 64-1 (R1-9) : un texte que le champ numérique ne sait pas lire (« 8,1 » selon le
		// navigateur) est lié à `null`, ce que `validateRateInput` prendrait pour un champ vide.
		const field = document.querySelector<HTMLInputElement>(
			'[data-testid="vat-form-rate"], [data-testid="vat-change-rate"]',
		);
		const rateCode: RateError | null = field?.validity?.badInput
			? 'invalid'
			: validateRateInput(fRate);
		if (rateCode) return rateErrorMessage(rateCode);
		if (validateDateInput(fValidFrom)) return dateMsg;
		return null;
	}

	async function submitCreate() {
		formError = inputError(
			i18nMsg('vat-rates-err-from-required', 'Saisissez la date de début.'),
		);
		if (formError) return;
		submitting = true;
		try {
			await createVatRate({
				category: fCategory.trim(),
				label: fLabel.trim() || undefined,
				rate: decimalOf(fRate),
				validFrom: fValidFrom,
				validTo: fValidTo ? fValidTo : null,
			});
			toast.success(i18nMsg('vat-rates-created', 'Taux TVA créé.'));
			mode = { kind: 'none' };
			await load();
		} catch (e) {
			formError = isApiError(e)
				? e.message
				: i18nMsg('vat-rates-create-error', 'La création a échoué.');
		} finally {
			submitting = false;
		}
	}

	// « Changer le taux » : clôture l'ancien (PUT valid_to = date de bascule)
	// PUIS crée le nouveau (POST même catégorie, valid_from = bascule). Ordre
	// imposé par l'invariant de non-chevauchement (sinon rejet).
	async function submitChange(old: VatRateResponse) {
		formError = inputError(
			i18nMsg('vat-rates-err-switch-required', 'Saisissez la date de bascule.'),
		);
		if (formError) return;
		submitting = true;
		// Posé juste après la clôture : dit quel appel a échoué (rien n'est clos si c'est le PUT).
		let closed = false;
		try {
			await updateVatRate(old.id, {
				label: old.label,
				validTo: fValidFrom, // la bascule clôt l'ancien à cette date
				active: true,
				version: old.version,
			});
			closed = true;
			await createVatRate({
				category: old.category,
				rate: decimalOf(fRate),
				validFrom: fValidFrom,
				validTo: null,
			});
			toast.success(i18nMsg('vat-rates-changed', 'Taux mis à jour.'));
			mode = { kind: 'none' };
			await load();
		} catch (e) {
			if (closed) {
				// L'ancien taux est clos, le nouveau n'existe pas : le dire, avec le motif, et
				// comment finir (seule réparation dans l'écran : refaire à la même date).
				// Revue 64-1 (R1-7) : sur une coupure ou un délai, le `POST` a pu aboutir.
				formError = isUnknownOutcome(e)
					? i18nMsg(
							'vat-rates-change-error-closed-unknown',
							"L'ancien taux a été clôturé à la date de bascule ; le nouveau taux n'a peut-être pas été créé. Vérifiez la liste : s'il manque, refaites « Changer le taux » à la même date.",
						)
					: isApiError(e)
					? i18nMsg(
							'vat-rates-change-error-closed-reason',
							"L'ancien taux a été clôturé à la date de bascule, mais le nouveau taux n'a pas été créé : { $reason } Une fois la cause levée, refaites « Changer le taux » à la même date.",
							{ reason: e.message },
						)
					: i18nMsg(
							'vat-rates-change-error-closed',
							"L'ancien taux a été clôturé à la date de bascule, mais le nouveau taux n'a pas été créé. Une fois la cause levée, refaites « Changer le taux » à la même date.",
						);
			} else {
				formError = isApiError(e)
					? e.message
					: i18nMsg('vat-rates-change-error', 'Le changement de taux a échoué.');
			}
			// Recharger : l'ancien a pu être clôturé même si la création a échoué.
			await load();
		} finally {
			submitting = false;
		}
	}

	async function confirmDeactivate(rate: VatRateResponse) {
		submitting = true;
		try {
			await deactivateVatRate(rate.id, rate.version);
			toast.success(i18nMsg('vat-rates-deactivated', 'Taux désactivé.'));
			mode = { kind: 'none' };
			await load();
		} catch (e) {
			toast.error(
				isApiError(e)
					? e.message
					: i18nMsg('vat-rates-deactivate-error', 'La désactivation a échoué.'),
			);
		} finally {
			submitting = false;
		}
	}
</script>

<div class="mx-auto max-w-3xl space-y-6 p-4">
	<header class="flex items-center justify-between">
		<div>
			<h1 class="text-2xl font-semibold">{i18nMsg('vat-rates-title', 'Taux de TVA')}</h1>
			<p class="text-muted-foreground text-sm">
				{i18nMsg(
					'vat-rates-subtitle',
					'Configurez les taux de TVA et leurs dates de validité. Les anciens taux restent appliqués aux opérations antérieures.',
				)}
			</p>
		</div>
		<Button onclick={openCreate} data-testid="vat-rates-new">
			{i18nMsg('vat-rates-new', 'Nouveau taux')}
		</Button>
	</header>

	{#if loading}
		<p class="text-muted-foreground">{i18nMsg('common-loading', 'Chargement…')}</p>
	{:else if loadError}
		<p class="text-destructive" data-testid="vat-rates-load-error">{loadError}</p>
	{:else}
		{#each grouped as [category, list] (category)}
			<section class="rounded-lg border p-4">
				<h2 class="mb-3 text-lg font-semibold">{categoryLabel(category)}</h2>
				<table class="w-full text-sm">
					<thead>
						<tr class="text-muted-foreground text-left">
							<th class="py-1">{i18nMsg('vat-rates-col-rate', 'Taux')}</th>
							<th class="py-1">{i18nMsg('vat-rates-col-from', 'Valide dès')}</th>
							<th class="py-1">{i18nMsg('vat-rates-col-to', "Jusqu'au")}</th>
							<th class="py-1">{i18nMsg('vat-rates-col-status', 'Statut')}</th>
							<th class="py-1 text-right">{i18nMsg('vat-rates-col-actions', 'Actions')}</th>
						</tr>
					</thead>
					<tbody>
						{#each list as r (r.id)}
							<tr class="border-t" data-testid="vat-rate-row">
								<td class="py-2 font-medium">{r.rate} %</td>
								<td class="py-2">{formatSwissDate(r.validFrom)}</td>
								<td class="py-2">{r.validTo ? formatSwissDate(r.validTo) : '—'}</td>
								<td class="py-2">
									{#if r.active}
										<span class="text-green-700">{i18nMsg('vat-rates-active', 'Actif')}</span>
									{:else}
										<span class="text-muted-foreground"
											>{i18nMsg('vat-rates-inactive', 'Inactif')}</span
										>
									{/if}
								</td>
								<td class="py-2 text-right">
									{#if r.active}
										<Button variant="outline" size="sm" onclick={() => openChange(r)}>
											{i18nMsg('vat-rates-change', 'Changer le taux')}
										</Button>
										<Button
											variant="ghost"
											size="sm"
											onclick={() => (mode = { kind: 'deactivate', rate: r })}
										>
											{i18nMsg('vat-rates-deactivate', 'Désactiver')}
										</Button>
									{/if}
								</td>
							</tr>
						{/each}
					</tbody>
				</table>
			</section>
		{/each}
		{#if grouped.length === 0}
			<p class="text-muted-foreground">{i18nMsg('vat-rates-empty', 'Aucun taux configuré.')}</p>
		{/if}
	{/if}

	<!-- Formulaire création -->
	{#if mode.kind === 'create'}
		<section class="rounded-lg border p-4">
			<h2 class="mb-3 text-lg font-semibold">{i18nMsg('vat-rates-new', 'Nouveau taux')}</h2>
			<div class="grid gap-3">
				<label class="grid gap-1">
					<span class="text-sm">{i18nMsg('vat-rates-field-category', 'Catégorie')}</span>
					<input
						class="border-input bg-background h-9 rounded-md border px-3"
						list="vat-known-categories"
						bind:value={fCategory}
						data-testid="vat-form-category"
					/>
					<datalist id="vat-known-categories">
						{#each KNOWN_CATEGORIES as c (c)}
							<option value={c}>{categoryLabel(c)}</option>
						{/each}
					</datalist>
				</label>
				<label class="grid gap-1">
					<span class="text-sm">{i18nMsg('vat-rates-field-rate', 'Taux (%)')}</span>
					<Input type="number" step="0.01" min="0" max="100" bind:value={fRate} data-testid="vat-form-rate" />
				</label>
				<label class="grid gap-1">
					<span class="text-sm">{i18nMsg('vat-rates-field-from', 'Valide dès')}</span>
					<Input type="date" bind:value={fValidFrom} data-testid="vat-form-from" />
				</label>
				<label class="grid gap-1">
					<span class="text-sm">{i18nMsg('vat-rates-field-to', "Jusqu'au (optionnel)")}</span>
					<Input type="date" bind:value={fValidTo} data-testid="vat-form-to" />
				</label>
				<label class="grid gap-1">
					<span class="text-sm">{i18nMsg('vat-rates-field-label', 'Libellé (optionnel)')}</span>
					<Input bind:value={fLabel} data-testid="vat-form-label" />
				</label>
			</div>
			{#if formError}
				<p class="text-destructive mt-2 text-sm" data-testid="vat-form-error">{formError}</p>
			{/if}
			<div class="mt-3 flex gap-2">
				<Button onclick={submitCreate} disabled={submitting} data-testid="vat-form-submit">
					{i18nMsg('common-save', 'Enregistrer')}
				</Button>
				<Button variant="ghost" onclick={cancel} disabled={submitting}>
					{i18nMsg('common-cancel', 'Annuler')}
				</Button>
			</div>
		</section>
	{/if}

	<!-- Formulaire changement de taux -->
	{#if mode.kind === 'change'}
		<section class="rounded-lg border p-4">
			<h2 class="mb-1 text-lg font-semibold">
				{i18nMsg('vat-rates-change', 'Changer le taux')} — {categoryLabel(mode.rate.category)}
			</h2>
			<p class="text-muted-foreground mb-3 text-sm">
				{i18nMsg(
					'vat-rates-change-hint',
					"L'ancien taux sera clôturé à la date de bascule, et le nouveau taux prendra effet à cette date.",
				)}
			</p>
			<div class="grid gap-3">
				<label class="grid gap-1">
					<span class="text-sm">{i18nMsg('vat-rates-field-new-rate', 'Nouveau taux (%)')}</span>
					<Input type="number" step="0.01" min="0" max="100" bind:value={fRate} data-testid="vat-change-rate" />
				</label>
				<label class="grid gap-1">
					<span class="text-sm">{i18nMsg('vat-rates-field-switch-date', 'Date de bascule')}</span>
					<Input type="date" bind:value={fValidFrom} data-testid="vat-change-from" />
				</label>
			</div>
			{#if formError}
				<p class="text-destructive mt-2 text-sm" data-testid="vat-change-error">{formError}</p>
			{/if}
			<div class="mt-3 flex gap-2">
				<Button
					onclick={() => mode.kind === 'change' && submitChange(mode.rate)}
					disabled={submitting}
					data-testid="vat-change-submit"
				>
					{i18nMsg('common-save', 'Enregistrer')}
				</Button>
				<Button variant="ghost" onclick={cancel} disabled={submitting}>
					{i18nMsg('common-cancel', 'Annuler')}
				</Button>
			</div>
		</section>
	{/if}

	<!-- Confirmation désactivation -->
	{#if mode.kind === 'deactivate'}
		<section class="border-destructive rounded-lg border p-4">
			<p class="mb-3">
				{i18nMsg(
					'vat-rates-deactivate-confirm',
					'Désactiver ce taux ? Il ne sera plus proposé à la saisie mais restera dans l’historique.',
				)}
				<strong>{rateLabel(mode.rate)} — {mode.rate.rate} %</strong>
			</p>
			<div class="flex gap-2">
				<Button
					variant="destructive"
					onclick={() => mode.kind === 'deactivate' && confirmDeactivate(mode.rate)}
					disabled={submitting}
					data-testid="vat-deactivate-confirm"
				>
					{i18nMsg('vat-rates-deactivate', 'Désactiver')}
				</Button>
				<Button variant="ghost" onclick={cancel} disabled={submitting}>
					{i18nMsg('common-cancel', 'Annuler')}
				</Button>
			</div>
		</section>
	{/if}
</div>
