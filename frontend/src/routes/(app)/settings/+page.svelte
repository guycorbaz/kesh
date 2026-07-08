<script lang="ts">
	import { onMount } from 'svelte';
	import { Button } from '$lib/components/ui/button';
	import { Separator } from '$lib/components/ui/separator';
	import { i18nMsg } from '$lib/features/onboarding/onboarding.svelte';
	import { fetchCompanyCurrent } from '$lib/features/settings/settings.api';
	import type { CompanyCurrentResponse } from '$lib/features/settings/settings.types';

	function msg(key: string, fallback: string): string {
		return i18nMsg(key, fallback);
	}

	let data = $state<CompanyCurrentResponse | null>(null);
	let loading = $state(true);

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
	</div>
{:else}
	<p class="text-text-muted">{msg('settings-no-company', 'Aucune organisation configurée. Complétez l\'onboarding.')}</p>
{/if}
