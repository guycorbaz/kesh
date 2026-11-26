<script lang="ts">
	// Story 55-1a (AC-17) — l'en-tête d'un rapport dit sa période RÉELLE : celle de la réponse affichée
	// (`from`, `to`), non celle des champs. Le texte vient de `enTetePeriode` (fiscal-year-menu.ts) ;
	// la liste des exercices passe par `Intl.ListFormat` de la locale, jamais par concaténation.

	import { i18nLocale, i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { enTetePeriode, type ExerciceBornes } from './fiscal-year-menu';

	interface Props {
		/** Début et fin de la période de la réponse, `AAAA-MM-JJ`. */
		from: string;
		to: string;
		/** Les exercices de la société (vide : l'en-tête dit « hors exercice »). */
		fiscalYears?: readonly ExerciceBornes[];
	}

	let { from, to, fiscalYears = [] }: Props = $props();

	function listFormat(items: string[]): string {
		try {
			return new Intl.ListFormat(i18nLocale(), { type: 'conjunction' }).format(items);
		} catch {
			// Locale inconnue de l'environnement (ou module i18n simulé dans un test) : le français.
			return new Intl.ListFormat('fr-CH', { type: 'conjunction' }).format(items);
		}
	}

	const enTete = $derived(enTetePeriode(from, to, fiscalYears, listFormat));
</script>

<!-- Un appel littéral par clé : la garde i18n-keys relève chaque clé et son texte de repli. -->
<p class="text-sm text-gray-600" data-testid="report-period-heading">
	{#if enTete.key === 'reports-nav-heading-fy'}
		{i18nMsg('reports-nav-heading-fy', 'Exercice { $years } — du { $from } au { $to }', { years: enTete.args.years, from: enTete.args.from, to: enTete.args.to })}
	{:else if enTete.key === 'reports-nav-heading-named'}
		{i18nMsg('reports-nav-heading-named', '{ $name } — du { $from } au { $to }', { name: enTete.args.name, from: enTete.args.from, to: enTete.args.to })}
	{:else if enTete.key === 'reports-nav-heading-part'}
		{i18nMsg('reports-nav-heading-part', 'Du { $from } au { $to } (partie de l’exercice { $fiscalYear })', { from: enTete.args.from, to: enTete.args.to, fiscalYear: enTete.args.fiscalYear })}
	{:else if enTete.key === 'reports-nav-heading-several'}
		{i18nMsg('reports-nav-heading-several', 'Du { $from } au { $to } (exercices { $fiscalYears })', { from: enTete.args.from, to: enTete.args.to, fiscalYears: enTete.args.fiscalYears })}
	{:else}
		{i18nMsg('reports-nav-heading-outside', 'Du { $from } au { $to } (hors exercice)', { from: enTete.args.from, to: enTete.args.to })}
	{/if}
</p>
