<script lang="ts" module>
	/**
	 * Le récapitulatif de la page « Configuration terminée », tel que la page le construit
	 * depuis `GET /api/v1/companies/current` et `GET /api/v1/fiscal-years`. Il ne porte
	 * **aucun IBAN** : la page ne recopie des comptes bancaires que le nom de la banque et
	 * la description.
	 */
	export interface OnboardingSummary {
		companyName: string;
		/** `Independant`, `Association` ou `Pme`. */
		orgType: string;
		/** Code de la langue comptable (`FR`, `DE`, `IT`, `EN`). */
		accountingLanguage: string;
		fiscalYears: ReadonlyArray<{ name: string; startDate: string; endDate: string }>;
		bankAccounts: ReadonlyArray<{ bankName: string; description: string | null }>;
	}
</script>

<script lang="ts">
	/**
	 * Story 36-17 (#1187, D1, D6) — la page « Configuration terminée ».
	 *
	 * Ce qui a été créé (société, plan comptable, exercice, compte bancaire) et ce qui reste
	 * à faire (comptes de TVA pour un assujetti ; compte bancaire s'il a été passé), puis
	 * *Commencer* : un lien vers `/` à **chargement complet** (`data-sveltekit-reload`),
	 * jamais une navigation d'office. Si le récapitulatif ne se lit pas, une ligne le dit ;
	 * le titre, le « à faire » fixe et *Commencer* restent.
	 *
	 * Présentationnel : la lecture du récapitulatif et son `catch` vivent dans `+page.svelte`.
	 */
	import { Button } from '$lib/components/ui/button';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { formatSwissDate } from '$lib/features/reports/reports.api';
	import { onboardingLanguageLabel } from './onboarding-screens';

	let {
		summary,
		summaryError,
	}: {
		/** `null` tant que le récapitulatif se lit, ou s'il n'a pas pu être lu. */
		summary: OnboardingSummary | null;
		/** Le motif de l'échec de la lecture, `null` sinon. */
		summaryError: string | null;
	} = $props();

	function orgTypeLabel(orgType: string): string {
		switch (orgType) {
			case 'Independant':
				return i18nMsg('onboarding-org-independant', 'Indépendant');
			case 'Association':
				return i18nMsg('onboarding-org-association', 'Association');
			case 'Pme':
				return i18nMsg('onboarding-org-pme', 'PME');
			default:
				return orgType;
		}
	}

	const sansCompteBancaire = $derived(summary !== null && summary.bankAccounts.length === 0);

	/**
	 * La phrase de la lecture en échec ; le motif ne la suit que s'il en dit plus (la page
	 * passe par `errorMessageOf(err, summaryErrorText)`, qui rend ce texte même sans motif).
	 */
	const summaryErrorText = $derived(
		i18nMsg('onboarding-done-summary-error', "Le récapitulatif n'a pas pu être lu.")
	);
</script>

<div class="flex flex-col gap-6" data-testid="onboarding-screen-done">
	<h2 class="text-center text-xl font-semibold">
		{i18nMsg('onboarding-done-title', 'Configuration terminée')}
	</h2>

	<section>
		<h3 class="mb-2 text-sm font-semibold uppercase text-text-muted">
			{i18nMsg('onboarding-done-created', 'Ce qui a été créé')}
		</h3>
		{#if summary}
			<dl class="grid grid-cols-[auto_1fr] gap-x-4 gap-y-1 text-sm" data-testid="onboarding-done-summary">
				<dt class="text-text-muted">{i18nMsg('onboarding-done-company', 'Société')}</dt>
				<dd>{summary.companyName}</dd>
				<dt class="text-text-muted">{i18nMsg('onboarding-done-chart', 'Plan comptable')}</dt>
				<dd>
					{orgTypeLabel(summary.orgType)} — {i18nMsg('onboarding-choose-accounting-lang', 'Langue comptable')}
					: {onboardingLanguageLabel(summary.accountingLanguage)}
				</dd>
				{#each summary.fiscalYears as fy (fy.name + fy.startDate)}
					<dt class="text-text-muted">{i18nMsg('onboarding-done-fiscal-year', 'Exercice')}</dt>
					<dd>{fy.name} ({formatSwissDate(fy.startDate)} – {formatSwissDate(fy.endDate)})</dd>
				{/each}
				{#each summary.bankAccounts as ba, i (i)}
					<dt class="text-text-muted">{i18nMsg('onboarding-done-bank-account', 'Compte bancaire')}</dt>
					<dd>{ba.bankName}{#if ba.description} — {ba.description}{/if}</dd>
				{/each}
			</dl>
		{:else if summaryError}
			<p class="text-sm" role="alert" data-testid="onboarding-done-summary-error">
				{summaryErrorText}
				{#if summaryError !== summaryErrorText}({summaryError}){/if}
			</p>
		{:else}
			<p class="text-sm text-text-muted" role="status" data-testid="onboarding-done-summary-loading">
				{i18nMsg('common-loading', 'Chargement…')}
			</p>
		{/if}
	</section>

	<section>
		<h3 class="mb-2 text-sm font-semibold uppercase text-text-muted">
			{i18nMsg('onboarding-done-todo', 'Ce qui reste à faire')}
		</h3>
		<ul class="list-disc space-y-1 pl-5 text-sm">
			<li data-testid="onboarding-done-todo-vat">
				{i18nMsg(
					'onboarding-done-todo-vat',
					'Si vous êtes assujetti à la TVA, désignez les comptes de TVA dans Administration → Facturation avant votre première facture avec TVA.'
				)}
			</li>
			{#if sansCompteBancaire}
				<li data-testid="onboarding-done-todo-bank">
					{i18nMsg(
						'onboarding-done-todo-bank',
						'Ajoutez votre compte bancaire dans Administration → Comptes bancaires.'
					)}
				</li>
			{/if}
		</ul>
	</section>

	<Button href="/" data-sveltekit-reload data-testid="onboarding-done-start">
		{i18nMsg('onboarding-start', 'Commencer')}
	</Button>
</div>
