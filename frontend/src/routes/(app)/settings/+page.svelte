<script lang="ts">
	import { onMount } from 'svelte';
	import { Button } from '$lib/components/ui/button';
	import { Separator } from '$lib/components/ui/separator';
	import { i18nMsg } from '$lib/features/onboarding/onboarding.svelte';
	import { fetchCompanyCurrent } from '$lib/features/settings/settings.api';
	import type { CompanyCurrentResponse } from '$lib/features/settings/settings.types';
	import { toast } from 'svelte-sonner';

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

	function notYet() {
		toast.info(msg('settings-edit-coming-soon', 'Édition bientôt disponible'));
	}
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
				<Button variant="outline" size="sm" onclick={notYet}>{msg('settings-edit', 'Modifier')}</Button>
			</div>
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

		<!-- Section Utilisateurs -->
		<section class="rounded-lg border border-border bg-white p-6 shadow-sm">
			<div class="flex items-center justify-between">
				<h2 class="text-lg font-semibold">{msg('settings-users-title', 'Utilisateurs')}</h2>
				<Button variant="outline" size="sm" href="/users">{msg('settings-manage', 'Gérer')}</Button>
			</div>
		</section>
	</div>
{:else}
	<p class="text-text-muted">{msg('settings-no-company', 'Aucune organisation configurée. Complétez l\'onboarding.')}</p>
{/if}
