<!--
  Story 37-2 (AC-6) — une case du tableau annuel des cours de change : le cours
  retenu par la société (une seule valeur, modifiable), puis la mention de sa
  source et de la valeur publiée, et le ↑ quand il s'offre.

  - Validation du champ (sortie ou Entrée) : règle `commitDecision` — champ vidé →
    valeur restaurée sans appel ; valeur égale au cours affiché → rien ; valeur
    égale à la valeur publiée → ↑ (`onReset`), jamais un `PUT` ; sinon `onPut`.
  - Une case **liée** (Ouverture ou Clôture en genre `month` qui est la même case
    qu'une colonne de mois) montre la valeur sans champ ni ↑ : elle s'édite dans la
    colonne du mois.
  - Lecture seule (Consultation, exercice clos) : ni champ ni ↑.
-->
<script lang="ts">
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { formatSwissDate, formatSwissDateTime } from '$lib/features/reports/reports.api';
	import { commitDecision, formatRate, mentionOf } from './exchange-rates-grid';
	import { arrowLabel, mentionText } from './exchange-rates-labels';
	import type { Cell } from './exchange-rates.types';

	interface Props {
		cell: Cell;
		currency: string;
		/** Identifiant des `data-testid` : `<devise>-<emplacement>-<date>`. */
		testKey: string;
		editable: boolean;
		/** Libellé de la colonne de mois à laquelle la case est liée, ou `null`. */
		linkedTo?: string | null;
		/** Raison, au survol, d'une case non récupérable. */
		notFetchable?: string | null;
		/** Échec de la dernière récupération de la colonne. */
		fetchError?: string | null;
		onPut: (rate: string) => Promise<void>;
		onReset: () => Promise<void>;
		/** Le clic sur ↑ (défaut : `onReset`) — l'écran peut y ouvrir le dialogue U3. */
		onArrow?: () => Promise<void>;
		/** Message d'un refus à l'écran (saisie invalide). */
		onInvalid: (message: string) => void;
	}

	let {
		cell,
		currency,
		testKey,
		editable,
		linkedTo = null,
		notFetchable = null,
		fetchError = null,
		onPut,
		onReset,
		onArrow,
		onInvalid,
	}: Props = $props();

	let display = $derived(formatRate(cell.effectiveRate));
	let draft = $state('');
	let busy = $state(false);
	let invalid = $state(false);

	$effect(() => {
		draft = display;
		invalid = false;
	});

	let mention = $derived(mentionOf(cell));
	let arrowVisible = $derived(editable && linkedTo === null && mention.arrow !== null);

	/** « publié le 28.06.2024 » quand la date de publication diffère du jour du cours. */
	let publishedOn = $derived(
		cell.kind === 'day' && cell.published?.sourceDate && cell.published.sourceDate !== cell.rateDate
			? i18nMsg('exchange-rates-published-on', 'publié le { $date }', {
					date: formatSwissDate(cell.published.sourceDate),
				})
			: null,
	);

	let title = $derived(
		[
			cell.row
				? cell.row.updatedByKesh
					? i18nMsg('exchange-rates-copied-by-kesh', 'Copié automatiquement par Kesh')
					: i18nMsg('exchange-rates-set-by-user', 'Retenu par un utilisateur le { $date }', {
							date: formatSwissDateTime(cell.row.updatedAt),
						})
				: null,
			notFetchable,
		]
			.filter((t): t is string => !!t)
			.join(' — '),
	);

	async function commit() {
		if (busy) return;
		const d = commitDecision(cell, draft);
		switch (d.action) {
			case 'restore':
			case 'none':
				draft = display;
				invalid = false;
				return;
			case 'invalid':
				invalid = true;
				onInvalid(
					i18nMsg(
						'exchange-rates-rate-invalid',
						'Cours { $currency } invalide : strictement positif, par unité, au plus 10 décimales et au plus 1 000 000.',
						{ currency },
					),
				);
				return;
			case 'reset':
				busy = true;
				try {
					await onReset();
				} finally {
					busy = false;
				}
				return;
			case 'put':
				busy = true;
				try {
					await onPut(d.rate);
				} finally {
					busy = false;
				}
				return;
		}
	}

	async function arrow() {
		if (busy) return;
		busy = true;
		try {
			await (onArrow ?? onReset)();
		} finally {
			busy = false;
		}
	}
</script>

<div class="flex min-w-28 flex-col gap-0.5" title={title || undefined} data-testid="exchange-rates-cell-{testKey}">
	{#if linkedTo !== null}
		<span class="font-mono text-sm" data-testid="exchange-rates-value-{testKey}">{display}</span>
		<span class="text-xs text-text-muted" data-testid="exchange-rates-linked-{testKey}">
			{i18nMsg('exchange-rates-linked', 'Même case que { $month }', { month: linkedTo })}
		</span>
	{:else}
		{#if editable}
			<input
				class="w-28 rounded border px-1.5 py-0.5 text-right font-mono text-sm {invalid
					? 'border-error'
					: 'border-border'}"
				inputmode="decimal"
				aria-label={i18nMsg('exchange-rates-input-label', 'Cours { $currency } pour une unité', {
					currency,
				})}
				aria-invalid={invalid}
				disabled={busy}
				bind:value={draft}
				onblur={() => void commit()}
				onkeydown={(e) => {
					if (e.key === 'Enter') {
						e.preventDefault();
						void commit();
					}
				}}
				data-testid="exchange-rates-input-{testKey}"
			/>
		{:else}
			<span class="font-mono text-sm" data-testid="exchange-rates-value-{testKey}">{display}</span>
		{/if}
		<span class="flex items-center gap-1 text-xs">
			<span
				class={mention.kind === 'afc-corrected' ? 'font-medium text-error' : 'text-text-muted'}
				data-testid="exchange-rates-mention-{testKey}"
				data-kind={mention.kind}>{mentionText(mention)}</span
			>
			{#if arrowVisible && mention.arrow}
				<button
					type="button"
					class="rounded px-1 font-semibold text-primary hover:bg-surface-alt disabled:opacity-50"
					aria-label={arrowLabel(mention.arrow)}
					title={arrowLabel(mention.arrow)}
					disabled={busy}
					onclick={() => void arrow()}
					data-testid="exchange-rates-arrow-{testKey}"
					data-arrow={mention.arrow}>↑</button
				>
			{/if}
		</span>
	{/if}
	{#if publishedOn}
		<span class="text-xs text-text-muted" data-testid="exchange-rates-published-on-{testKey}">{publishedOn}</span>
	{/if}
	{#if fetchError}
		<span class="text-xs text-error" data-testid="exchange-rates-fetch-error-{testKey}">{fetchError}</span>
	{/if}
</div>
