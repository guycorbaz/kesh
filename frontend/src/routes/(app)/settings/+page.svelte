<script lang="ts">
	import { onMount } from 'svelte';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Separator } from '$lib/components/ui/separator';
	import { i18nMsg } from '$lib/features/onboarding/onboarding.svelte';
	import {
		fetchCompanyCurrent,
		updateCompanyContactDetails,
		updateCompanyEmail
	} from '$lib/features/settings/settings.api';
	import type { CompanyCurrentResponse } from '$lib/features/settings/settings.types';
	import { authState } from '$lib/app/stores/auth.svelte';
	import { isApiError } from '$lib/shared/utils/api-client';
	import { isPlausibleEmail } from '$lib/shared/utils/email';
	import { notifySuccess } from '$lib/shared/utils/notify';

	function msg(key: string, fallback: string): string {
		return i18nMsg(key, fallback);
	}

	let data = $state<CompanyCurrentResponse | null>(null);
	let loading = $state(true);

	// Story 20-3b2 — édition inline de l'e-mail société (Reply-To), Admin-only.
	let isAdmin = $derived(authState.currentUser?.role === 'Admin');
	let emailEditing = $state(false);
	let emailValue = $state('');
	let emailSubmitting = $state(false);
	let emailError = $state('');

	function startEmailEdit() {
		emailValue = data?.company.email ?? '';
		emailError = '';
		emailEditing = true;
	}

	async function saveEmail() {
		if (!data) return;
		const trimmed = emailValue.trim();
		if (trimmed && !isPlausibleEmail(trimmed)) {
			emailError = msg('settings-company-email-invalid', 'Adresse e-mail invalide.');
			return;
		}
		emailSubmitting = true;
		emailError = '';
		try {
			// Champ vidé = effacement (Reply-To omis à l'envoi).
			const updated = await updateCompanyEmail({
				email: trimmed || null,
				version: data.company.version,
			});
			data = { ...data, company: updated };
			emailEditing = false;
			notifySuccess(msg('settings-company-email-saved', 'E-mail de la société enregistré'));
		} catch (err) {
			if (isApiError(err)) {
				if (err.code === 'OPTIMISTIC_LOCK_CONFLICT') {
					// Conflit de version : recharger et laisser l'admin recommencer.
					// Si le reload échoue aussi (review P3 BH), ne pas prétendre
					// « rechargées » — la version locale reste périmée, re-cliquer
					// redonnerait le même 409 : inviter à recharger la page.
					try {
						data = await fetchCompanyCurrent();
						emailError = msg(
							'settings-company-email-conflict',
							'Conflit de version — les données ont été rechargées, réessayez.',
						);
					} catch {
						emailError = msg(
							'settings-company-email-conflict-reload-failed',
							'Conflit de version et rechargement impossible — rechargez la page.',
						);
					}
				} else {
					emailError = err.message;
				}
			} else {
				// `common-error` = clé générique existante (review P1 : éviter le
				// doublon avec `error-unexpected`, gardée en FTL pour la page
				// contacts qui la consommait sans définition).
				emailError = msg('common-error', 'Erreur inattendue');
			}
		} finally {
			emailSubmitting = false;
		}
	}

	// Story 16-3a (#151) — téléphone et site web, rendus sur le PDF de facture.
	// Édition inline sur le patron exact de l'e-mail ci-dessus : c'est le SEUL
	// chemin de saisie pour une instance déjà installée, l'onboarding ne se
	// rejouant pas.
	let contactEditing = $state(false);
	let phoneValue = $state('');
	let websiteValue = $state('');
	let contactSubmitting = $state(false);
	let contactError = $state('');

	function startContactEdit() {
		phoneValue = data?.company.phone ?? '';
		websiteValue = data?.company.website ?? '';
		contactError = '';
		contactEditing = true;
	}

	async function saveContactDetails() {
		if (!data) return;
		contactSubmitting = true;
		contactError = '';
		try {
			// Champ vidé = effacement : la ligne disparaît du PDF (D2).
			const updated = await updateCompanyContactDetails({
				phone: phoneValue.trim() || null,
				website: websiteValue.trim() || null,
				version: data.company.version,
			});
			data = { ...data, company: updated };
			contactEditing = false;
			notifySuccess(msg('settings-company-contact-saved', 'Coordonnées enregistrées'));
		} catch (err) {
			if (isApiError(err)) {
				if (err.code === 'OPTIMISTIC_LOCK_CONFLICT') {
					// ⚠️ Clés DÉDIÉES au bloc coordonnées, et non celles du bloc
					// e-mail : les deux libellés sont génériques aujourd'hui, mais
					// les partager ferait s'afficher ici toute reformulation
					// destinée au contexte e-mail, sans qu'aucun test ne le voie.
					// (Passe 6 de revue.)
					try {
						data = await fetchCompanyCurrent();
						contactError = msg(
							'settings-company-contact-conflict',
							'Conflit de version — les données ont été rechargées, réessayez.',
						);
					} catch {
						contactError = msg(
							'settings-company-contact-conflict-reload-failed',
							'Conflit de version et rechargement impossible — rechargez la page.',
						);
					}
				} else {
					contactError = err.message;
				}
			} else {
				contactError = msg('common-error', 'Erreur inattendue');
			}
		} finally {
			contactSubmitting = false;
		}
	}

	onMount(async () => {
		try {
			data = await fetchCompanyCurrent();
		} catch {
			// Company may not exist (404) — that's OK
		} finally {
			loading = false;
		}
	});

</script>

<svelte:head>
	<title>Paramètres - Kesh</title>
</svelte:head>

<h1 class="mb-6 text-2xl font-semibold text-text">
	{msg('settings-title', 'Paramètres')}
</h1>

{#if loading}
	<p class="text-text-muted">{msg('loading', 'Chargement...')}</p>
{:else if data}
	<div class="flex flex-col gap-6">
		<!-- Section Organisation -->
		<section class="rounded-lg border border-border bg-white p-6 shadow-sm">
			<h2 class="mb-4 text-lg font-semibold">{msg('settings-org-title', 'Organisation')}</h2>
			<dl class="grid grid-cols-2 gap-x-6 gap-y-3 text-sm">
				<div>
					<dt class="font-medium text-text-muted">{msg('settings-field-name', 'Nom')}</dt>
					<dd>{data.company.name}</dd>
				</div>
				<div>
					<dt class="font-medium text-text-muted">{msg('settings-field-org-type', 'Type')}</dt>
					<dd>{data.company.orgType}</dd>
				</div>
				<div class="col-span-2">
					<dt class="font-medium text-text-muted">{msg('settings-field-address', 'Adresse')}</dt>
					<dd class="whitespace-pre-line">{data.company.address}</dd>
				</div>
				{#if data.company.ideNumber}
					<div>
						<dt class="font-medium text-text-muted">{msg('settings-field-ide', 'IDE')}</dt>
						<dd>{data.company.ideNumber}</dd>
					</div>
				{/if}
				<div>
					<dt class="font-medium text-text-muted">{msg('settings-field-instance-language', 'Langue interface')}</dt>
					<dd>{data.company.instanceLanguage}</dd>
				</div>
				<!-- Story 20-3b2 : e-mail de contact (Reply-To des factures envoyées). -->
				<div class="col-span-2">
					<dt class="font-medium text-text-muted">
						{msg('settings-field-company-email', 'E-mail (adresse de réponse)')}
					</dt>
					{#if emailEditing}
						<dd class="mt-1 flex max-w-md items-start gap-2">
							<div class="flex-1">
								<Input
									type="email"
									bind:value={emailValue}
									maxlength={320}
									data-testid="settings-company-email-input"
								/>
								{#if emailError}
									<p class="mt-1 text-xs text-destructive" data-testid="settings-company-email-error">
										{emailError}
									</p>
								{/if}
							</div>
							<Button size="sm" onclick={saveEmail} disabled={emailSubmitting} data-testid="settings-company-email-save">
								{msg('common-save', 'Enregistrer')}
							</Button>
							<Button
								size="sm"
								variant="outline"
								onclick={() => {
									emailEditing = false;
									emailError = '';
								}}
								disabled={emailSubmitting}
							>
								{msg('common-cancel', 'Annuler')}
							</Button>
						</dd>
					{:else}
						<dd class="flex items-center gap-3">
							<span data-testid="settings-company-email">{data.company.email ?? '—'}</span>
							{#if isAdmin}
								<Button size="sm" variant="outline" onclick={startEmailEdit} data-testid="settings-company-email-edit">
									{msg('common-edit', 'Modifier')}
								</Button>
							{/if}
						</dd>
					{/if}
					<p class="mt-1 text-xs text-text-muted">
						{msg(
							'settings-company-email-help',
							"Adresse de réponse (Reply-To) des factures envoyées par e-mail. Vide = pas d'adresse de réponse.",
						)}
					</p>
				</div>

				<!-- Story 16-3a (#151) : téléphone et site web, rendus sur le PDF. -->
				<div class="col-span-2">
					<dt class="font-medium text-text-muted">
						{msg('settings-field-company-phone', 'Téléphone')} ·
						{msg('settings-field-company-website', 'Site web')}
					</dt>
					{#if contactEditing}
						<dd class="mt-1 max-w-md space-y-2">
							<Input
								type="text"
								bind:value={phoneValue}
								maxlength={50}
								placeholder={msg('settings-field-company-phone', 'Téléphone')}
								data-testid="settings-company-phone-input"
							/>
							<Input
								type="text"
								bind:value={websiteValue}
								maxlength={255}
								placeholder={msg('settings-field-company-website', 'Site web')}
								data-testid="settings-company-website-input"
							/>
							{#if contactError}
								<p class="text-xs text-destructive" data-testid="settings-company-contact-error">
									{contactError}
								</p>
							{/if}
							<div class="flex gap-2">
								<Button
									size="sm"
									onclick={saveContactDetails}
									disabled={contactSubmitting}
									data-testid="settings-company-contact-save"
								>
									{msg('common-save', 'Enregistrer')}
								</Button>
								<Button
									size="sm"
									variant="outline"
									onclick={() => {
										contactEditing = false;
										contactError = '';
									}}
									disabled={contactSubmitting}
								>
									{msg('common-cancel', 'Annuler')}
								</Button>
							</div>
						</dd>
					{:else}
						<dd class="flex items-center gap-3">
							<span data-testid="settings-company-phone">{data.company.phone ?? '—'}</span>
							<span data-testid="settings-company-website">{data.company.website ?? '—'}</span>
							{#if isAdmin}
								<Button
									size="sm"
									variant="outline"
									onclick={startContactEdit}
									data-testid="settings-company-contact-edit"
								>
									{msg('common-edit', 'Modifier')}
								</Button>
							{/if}
						</dd>
					{/if}
					<p class="mt-1 text-xs text-text-muted">
						{msg(
							'settings-company-phone-help',
							'Numéro de téléphone affiché sur vos factures. Vide = ligne omise.',
						)}
					</p>
					<p class="mt-1 text-xs text-text-muted">
						{msg(
							'settings-company-website-help',
							'Adresse de votre site, affichée sur vos factures. Vide = ligne omise.',
						)}
					</p>
				</div>
			</dl>
		</section>

		<Separator />

		<!-- Section Comptabilité -->
		<section class="rounded-lg border border-border bg-white p-6 shadow-sm">
			<h2 class="mb-4 text-lg font-semibold">{msg('settings-accounting-title', 'Comptabilité')}</h2>
			<dl class="text-sm">
				<dt class="font-medium text-text-muted">{msg('settings-field-accounting-language', 'Langue comptable')}</dt>
				<dd>{data.company.accountingLanguage}</dd>
			</dl>
		</section>

		<Separator />

		<!-- Section Comptes bancaires -->
		<section class="rounded-lg border border-border bg-white p-6 shadow-sm">
			<div class="flex items-center justify-between">
				<h2 class="text-lg font-semibold">{msg('settings-bank-title', 'Comptes bancaires')}</h2>
				<Button variant="outline" size="sm" href="/bank-accounts" data-testid="settings-bank-manage-link">
					{msg('settings-bank-manage', 'Gérer dans Administration → Comptes bancaires')}
				</Button>
			</div>
			<p class="mt-2 text-xs text-text-muted">
				{msg(
					'settings-bank-manage-hint',
					'Pour ajouter, modifier ou archiver un compte bancaire, utilisez la page dédiée Administration → Comptes bancaires.',
				)}
			</p>
			{#if data.bankAccounts.length > 0}
				{#each data.bankAccounts as account}
					<div class="mt-3 text-sm">
						<p class="font-medium">{account.bankName}</p>
						<p class="text-text-muted">{account.iban}</p>
						{#if account.qrIban}
							<p class="text-text-muted">QR-IBAN: {account.qrIban}</p>
						{/if}
					</div>
				{/each}
			{:else}
				<p class="mt-3 text-sm text-text-muted">{msg('settings-no-bank', 'Aucun compte bancaire configuré.')}</p>
			{/if}
		</section>

		<Separator />

		<!-- Section Clés API (Story 17-2b) -->
		<section class="rounded-lg border border-border bg-white p-6 shadow-sm">
			<div class="flex items-center justify-between">
				<h2 class="text-lg font-semibold">{msg('settings-api-keys-title', 'Clés API')}</h2>
				<Button variant="outline" size="sm" href="/settings/api-keys" data-testid="settings-api-keys-manage-link">
					{msg('settings-api-keys-manage', 'Gérer')}
				</Button>
			</div>
			<p class="mt-2 text-xs text-text-muted">
				{msg(
					'settings-api-keys-hint',
					'Créez des clés d\'accès API pour vos intégrations (IA externe, scripts, logiciels tiers).',
				)}
			</p>
		</section>

		<Separator />

		<!-- Section Utilisateurs -->
		<section class="rounded-lg border border-border bg-white p-6 shadow-sm">
			<div class="flex items-center justify-between">
				<h2 class="text-lg font-semibold">{msg('settings-users-title', 'Utilisateurs')}</h2>
				<Button variant="outline" size="sm" href="/users">{msg('settings-manage', 'Gérer')}</Button>
			</div>
		</section>

		<Separator />

		<!-- Section Exercices comptables (Story 3.7) -->
		<section class="rounded-lg border border-border bg-white p-6 shadow-sm">
			<div class="flex items-center justify-between">
				<h2 class="text-lg font-semibold">{msg('fiscal-year-title', 'Exercices comptables')}</h2>
				<Button variant="outline" size="sm" href="/settings/fiscal-years">{msg('settings-manage', 'Gérer')}</Button>
			</div>
			<p class="mt-2 text-sm text-text-muted">
				{msg('settings-fiscal-years-link', "Créez, renommez ou clôturez les exercices comptables de votre entreprise.")}
			</p>
		</section>

		<Separator />

		<!-- Section Taux de TVA (Story 11-1, Administrateur) -->
		<section class="rounded-lg border border-border bg-white p-6 shadow-sm">
			<div class="flex items-center justify-between">
				<h2 class="text-lg font-semibold">{msg('vat-rates-title', 'Taux de TVA')}</h2>
				<Button variant="outline" size="sm" href="/settings/vat-rates" data-testid="settings-vat-rates-manage-link">{msg('settings-manage', 'Gérer')}</Button>
			</div>
			<p class="mt-2 text-sm text-text-muted">
				{msg('settings-vat-rates-link', "Configurez les taux de TVA et leurs dates de validité (changements de taux gérés dans le temps).")}
			</p>
		</section>

		<Separator />

		<!-- Section Modèles d'e-mail (Story 20-2, Administrateur) -->
		<section class="rounded-lg border border-border bg-white p-6 shadow-sm">
			<div class="flex items-center justify-between">
				<h2 class="text-lg font-semibold">{msg('email-templates-title', "Modèles d'e-mail")}</h2>
				<Button variant="outline" size="sm" href="/settings/email-templates" data-testid="settings-email-templates-manage-link">{msg('settings-manage', 'Gérer')}</Button>
			</div>
			<p class="mt-2 text-sm text-text-muted">
				{msg('settings-email-templates-link', "Personnalisez le contenu des e-mails envoyés à vos clients (objet et corps, par langue).")}
			</p>
		</section>

		<Separator />

		<!-- Section Rappels débiteurs (Story 21-4, Administrateur) -->
		<section class="rounded-lg border border-border bg-white p-6 shadow-sm">
			<div class="flex items-center justify-between">
				<h2 class="text-lg font-semibold">{msg('dunning-title', 'Rappels débiteurs')}</h2>
				<Button variant="outline" size="sm" href="/settings/dunning" data-testid="settings-dunning-manage-link">{msg('settings-manage', 'Gérer')}</Button>
			</div>
			<p class="mt-2 text-sm text-text-muted">
				{msg('settings-dunning-link', 'Configurez les niveaux de rappel (délais et frais), la période de grâce, et personnalisez les textes d’e-mail de rappel par niveau.')}
			</p>
		</section>
	</div>
{:else}
	<p class="text-text-muted">{msg('settings-no-company', 'Aucune organisation configurée. Complétez l\'onboarding.')}</p>
{/if}
