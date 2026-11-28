<script lang="ts">
	// Story 46-2 (AC-2, AC-7) — la barre de présentation du bilan et du compte de
	// résultat (et de la balance, Story 46-6b) : hiérarchie montrée ou masquée, niveau de détail, « Tout ouvrir »,
	// « Tout fermer », place des sous-totaux. Elle ne calcule rien : elle rend la
	// vue suivante à la page (`onChange`), qui la mémorise et redemande le rapport.
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import {
		closeAll,
		detailLevelOptions,
		openAll,
		withDetailLevel,
		type GroupingView,
	} from './reports-grouping';
	import type { DetailLevel, SubtotalPosition } from './reports.types';

	interface Props {
		view: GroupingView;
		/** `grouping.maxDepth` de la dernière réponse groupée ; `null` avant la première. */
		maxDepth: number | null;
		onChange: (next: GroupingView) => void;
		/**
		 * Story 46-6b — l'info-bulle du niveau de détail propre au rapport (la
		 * balance n'a pas les sections légales du bilan) ; absente : celle du bilan
		 * et du compte de résultat.
		 */
		levelHint?: string;
	}
	let { view, maxDepth, onChange, levelHint }: Props = $props();

	let levels = $derived(detailLevelOptions(maxDepth ?? 0, view.detailLevel));
	// Désactivé hiérarchie masquée, avant toute réponse groupée, et pour un plan
	// sans groupe ou un rapport vide (`maxDepth` ≤ 1 : seul « Tous les comptes »).
	let levelDisabled = $derived(!view.shown || maxDepth === null || maxDepth <= 1);
	let levelValue = $derived(String(view.detailLevel));
	let hint = $derived(
		levelHint ??
			i18nMsg(
				'reports-groups-level-hint',
				'Les sections légales du bilan restent (Actifs, Passifs, Capitaux propres). Un affichage replié, au niveau 1 en particulier, est un affichage de travail, non des comptes annuels.',
			),
	);

	function onLevel(event: Event): void {
		const raw = (event.currentTarget as HTMLSelectElement).value;
		const level: DetailLevel = raw === 'all' ? 'all' : Number(raw);
		onChange(withDetailLevel(view, level));
	}

	function onPosition(event: Event): void {
		const position = (event.currentTarget as HTMLSelectElement).value as SubtotalPosition;
		onChange({ ...view, subtotalPosition: position });
	}
</script>

<div
	class="flex flex-wrap items-center gap-x-4 gap-y-2 rounded border border-border bg-surface px-3 py-2 text-sm"
	role="group"
	aria-label={i18nMsg('reports-groups-controls-label', 'Présentation par groupes de comptes')}
	data-testid="grouping-controls"
>
	<label class="flex items-center gap-2">
		<input
			type="checkbox"
			checked={view.shown}
			onchange={(e) => onChange({ ...view, shown: (e.currentTarget as HTMLInputElement).checked })}
			data-testid="grouping-shown"
		/>
		{i18nMsg('reports-groups-hierarchy', 'Hiérarchie des groupes')}
	</label>

	<!-- Revue P1 L2 — le libellé du sélecteur reste court (« Niveau de détail ») ;
	     l'info-bulle n'en est que la description (`aria-describedby`), hors du
	     `<label>`. -->
	<div class="flex items-center gap-2">
		<label for="reports-groups-level">{i18nMsg('reports-groups-level', 'Niveau de détail')}</label>
		<select
			id="reports-groups-level"
			class="rounded border border-border px-2 py-1 disabled:opacity-50"
			value={levelValue}
			disabled={levelDisabled}
			onchange={onLevel}
			aria-describedby="reports-groups-level-hint"
			data-testid="grouping-level"
		>
			{#each levels as level (level)}
				<option value={String(level)}
					>{i18nMsg('reports-groups-level-n', 'Niveau { $level }', { level })}</option
				>
			{/each}
			<option value="all">{i18nMsg('reports-groups-level-all', 'Tous les comptes')}</option>
		</select>
		<!-- Revue P2 L4 — l'info-bulle visuelle ne paraît qu'au survol du ⓘ. -->
		<span class="cursor-help text-gray-500" aria-hidden="true" title={hint}>ⓘ</span>
		<span id="reports-groups-level-hint" class="sr-only">{hint}</span>
	</div>

	<button
		type="button"
		class="rounded border border-border px-2 py-1 disabled:opacity-50"
		disabled={!view.shown}
		onclick={() => onChange(openAll(view))}
		data-testid="grouping-open-all"
	>
		{i18nMsg('reports-groups-open-all', 'Tout ouvrir')}
	</button>
	<button
		type="button"
		class="rounded border border-border px-2 py-1 disabled:opacity-50"
		disabled={!view.shown}
		onclick={() => onChange(closeAll(view))}
		data-testid="grouping-close-all"
	>
		{i18nMsg('reports-groups-close-all', 'Tout fermer')}
	</button>

	<label class="flex items-center gap-2">
		{i18nMsg('reports-groups-subtotals', 'Sous-totaux')}
		<select
			class="rounded border border-border px-2 py-1 disabled:opacity-50"
			value={view.subtotalPosition}
			disabled={!view.shown}
			onchange={onPosition}
			data-testid="grouping-position"
		>
			<option value="header">{i18nMsg('reports-groups-subtotals-header', 'sur le titre')}</option>
			<option value="footer">{i18nMsg('reports-groups-subtotals-footer', 'en pied')}</option>
		</select>
	</label>
</div>
