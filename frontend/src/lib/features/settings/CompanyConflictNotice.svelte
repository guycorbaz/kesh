<script lang="ts">
	/**
	 * Story 34-6 (#953) — l'avis d'un conflit de version d'une édition de la société.
	 *
	 * Deux emplois, un seul composant (C-e343-7) :
	 * - **avec `field`** : la ligne posée sous un champ — « Enregistré entre-temps :
	 *   *valeur* » si le champ est en conflit, « Repris de l'enregistrement entre-temps :
	 *   *valeur* » s'il a été fusionné ; rien sinon ;
	 * - **sans `field`** : l'avis du bloc — enregistrer de nouveau remplacera les valeurs
	 *   nommées (s'il y en a en conflit), et les champs non modifiés ont été repris (s'il y en
	 *   a de fusionnés).
	 *
	 * Les `data-testid` se dérivent de `testid` : `<testid>-changed-<champ>`,
	 * `<testid>-merged-<champ>` et `<testid>-notice`, le champ en kebab-case comme les autres
	 * `data-testid` de la page (`postalCode` → `postal-code` ; revue P1, B-3).
	 *
	 * ⚠️ Les variables passent par le troisième argument d'`i18nMsg`, jamais par un
	 * `.replace()` : le backend pré-résout Fluent sans arguments, et sa marque d'isolation
	 * bidi ne se consomme qu'avec le paramètre.
	 */
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import type { CompanyConflictNoticeState } from './company-conflict';

	type Props = { notice: CompanyConflictNoticeState | null; testid: string; field?: string };
	let { notice, testid, field }: Props = $props();

	let slug = $derived(field?.replace(/[A-Z]/g, (c) => `-${c.toLowerCase()}`));

	function shown(value: string): string {
		return value === '' ? i18nMsg('settings-company-conflict-empty', '(vide)') : value;
	}
</script>

{#if notice}
	{#if field !== undefined}
		{#if notice.changed.includes(field)}
			<p class="mt-1 text-xs text-warning" data-testid="{testid}-changed-{slug}">
				{i18nMsg('settings-company-conflict-changed', 'Enregistré entre-temps : { $value }', {
					value: shown(notice.stored[field] ?? ''),
				})}
			</p>
		{:else if notice.merged.includes(field)}
			<p class="mt-1 text-xs text-text-muted" data-testid="{testid}-merged-{slug}">
				{i18nMsg('settings-company-conflict-merged', 'Repris de l’enregistrement entre-temps : { $value }', {
					value: shown(notice.stored[field] ?? ''),
				})}
			</p>
		{/if}
	{:else}
		<div class="space-y-1 text-xs text-warning" role="status" data-testid="{testid}-notice">
			{#if notice.changed.length > 0}
				<p>
					{i18nMsg(
						'settings-company-conflict-will-replace',
						'Conflit de version : la société a été modifiée entre-temps. Enregistrer de nouveau remplacera les valeurs enregistrées indiquées sous les champs.',
					)}
				</p>
			{/if}
			{#if notice.merged.length > 0}
				<p>
					{i18nMsg(
						'settings-company-conflict-merged-notice',
						'Conflit de version : les champs que vous n’aviez pas modifiés ont repris la valeur enregistrée entre-temps. Vérifiez, puis enregistrez de nouveau.',
					)}
				</p>
			{/if}
		</div>
	{/if}
{/if}
