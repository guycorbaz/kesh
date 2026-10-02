<!--
  Story 25-4-d2b (#490 ; API : 25-4-d2a, #384) — Dialogue « Solder le reste ».

  ⛔ **Pas de montant saisi** : le serveur solde le reste EXACT. Le dialogue
  l'affiche, et c'est tout.

  ⚠️ **Le reste et la version se lisent dans `amountDue` / la facture du parent,
  au moment de la confirmation** — jamais dans une copie faite à l'ouverture
  (le patron `SettleInvoiceDialog` ne se resynchronise qu'à l'ouverture) : après
  un 409 de version, le parent relit la facture, le dialogue reste ouvert et la
  tentative suivante part avec la version relue. Seule la SAISIE (nature, date)
  est réinitialisée à l'ouverture.

  Émet `onConfirm({ nature, settledOn })` ; le parent ajoute la `version`, appelle
  l'API et gère les erreurs.
-->
<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import * as Dialog from '$lib/components/ui/dialog';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { formatExactAmount, hasSubCentime } from './invoice-helpers';
	import {
		WRITE_OFF_NATURES,
		isNatureOffered,
		missingAccount,
		writeOffNatureHelp,
		writeOffNatureLabel,
	} from './write-off';
	import type { InvoiceSettingsResponse, WriteOffNature } from './invoices.types';

	export type WriteOffPayload = { nature: WriteOffNature; settledOn: string };

	type Props = {
		open: boolean;
		onOpenChange: (v: boolean) => void;
		/** Date facture (YYYY-MM-DD) — borne basse de `settledOn`. */
		invoiceDate: string;
		/**
		 * Le reste dû EXACT (quatre décimales possibles), lu à chaque rendu ;
		 * `null` s'il n'est pas calculé (réponse d'un envoi d'e-mail) — le
		 * dialogue reste monté, mais la confirmation est désactivée.
		 */
		amountDue: string | null;
		/** Les réglages de facturation ; `null` si inconnus (aucun pré-contrôle). */
		settings: InvoiceSettingsResponse | null;
		submitting?: boolean;
		errorMsg?: string;
		onConfirm: (payload: WriteOffPayload) => void;
	};

	let {
		open,
		onOpenChange,
		invoiceDate,
		amountDue,
		settings,
		submitting = false,
		errorMsg = '',
		onConfirm,
	}: Props = $props();

	function todayIso(): string {
		return new Date().toISOString().slice(0, 10);
	}

	let nature = $state<WriteOffNature | null>(null);
	let settledOn = $state(todayIso());

	// La saisie seule est réinitialisée à l'ouverture.
	$effect(() => {
		if (open) {
			nature = null;
			settledOn = todayIso();
		}
	});

	let offered = $derived(
		amountDue === null ? [] : WRITE_OFF_NATURES.filter((n) => isNatureOffered(n, amountDue)),
	);

	// Après une relecture, une nature qui n'est plus proposée (reste d'arrondi
	// devenu ≥ 0.05) est retirée de la saisie.
	$effect(() => {
		if (nature !== null && !offered.includes(nature)) nature = null;
	});

	let missing = $derived(
		nature === null || amountDue === null ? null : missingAccount(settings, nature, amountDue),
	);

	let clientError = $derived.by(() => {
		if (amountDue === null) {
			return i18nMsg('invoices-write-off-error-unknown-due', 'Reste dû en cours de calcul…');
		}
		if (nature === null) {
			return i18nMsg('invoices-write-off-error-nature', "Choisissez la nature de l'écart.");
		}
		if (!settledOn) {
			return i18nMsg('invoices-write-off-error-date-required', 'Date du solde obligatoire');
		}
		if (invoiceDate && settledOn < invoiceDate.slice(0, 10)) {
			return i18nMsg(
				'invoices-write-off-error-date-before-invoice',
				'La date du solde ne peut être antérieure à la date de facture',
			);
		}
		return '';
	});

	function handleConfirm() {
		if (clientError || missing !== null || nature === null) return;
		onConfirm({ nature, settledOn });
	}
</script>

<Dialog.Root {open} {onOpenChange}>
	<!-- Pendant l'envoi, ni Échap, ni clic extérieur, ni croix : le refus
	     éventuel ne serait plus montré nulle part (revue P1, B-H1). La garde
	     vit ici, pas dans le parent : le composant `Dialog` du projet relie
	     `open` en interne et se fermerait quoi que le parent décide. -->
	<Dialog.Content
		escapeKeydownBehavior={submitting ? 'ignore' : 'close'}
		interactOutsideBehavior={submitting ? 'ignore' : 'close'}
		showCloseButton={!submitting}
	>
		<Dialog.Header>
			<Dialog.Title>
				{i18nMsg('invoices-write-off-dialog-title', 'Solder le reste')}
			</Dialog.Title>
		</Dialog.Header>

		<p class="text-sm">
			{i18nMsg(
				'invoices-write-off-dialog-body',
				"Le reste dû est éteint et passé au compte de la nature choisie. Un solde s'annule comme un règlement.",
			)}
		</p>

		<div class="mt-2 flex items-baseline justify-between text-sm">
			<span class="text-text-muted">
				{i18nMsg('invoices-write-off-amount-label', 'Montant soldé')}
			</span>
			<span class="font-mono font-semibold" data-testid="write-off-amount">
				{amountDue === null ? '—' : formatExactAmount(amountDue)}
			</span>
		</div>
		{#if amountDue !== null && hasSubCentime(amountDue)}
			<p class="text-xs text-text-muted" data-testid="write-off-sub-centime">
				{i18nMsg(
					'invoices-write-off-sub-centime',
					"La fraction de centime est passée au compte de différences d'arrondi.",
				)}
			</p>
		{/if}

		<fieldset class="mt-3 space-y-2">
			<legend class="mb-1 text-xs text-text-muted">
				{i18nMsg('invoices-write-off-nature-label', "Nature de l'écart")}
			</legend>
			{#each offered as n (n)}
				<label class="flex items-start gap-2 text-sm">
					<input
						type="radio"
						name="write-off-nature"
						value={n}
						bind:group={nature}
						data-testid="write-off-nature-{n}"
					/>
					<span>
						<span class="font-medium">{writeOffNatureLabel(n)}</span>
						<span class="block text-xs text-text-muted">{writeOffNatureHelp(n)}</span>
					</span>
				</label>
			{/each}
		</fieldset>

		{#if missing === 'nature'}
			<div
				class="rounded-md border border-warning bg-warning/10 px-3 py-2 text-sm"
				data-testid="write-off-missing-account"
			>
				{i18nMsg(
					'invoices-write-off-missing-nature-account',
					"Aucun compte n'est désigné pour cette nature : demandez à un administrateur de le choisir dans Paramètres → Facturation.",
				)}
			</div>
		{:else if missing === 'rounding'}
			<div
				class="rounded-md border border-warning bg-warning/10 px-3 py-2 text-sm"
				data-testid="write-off-missing-account"
			>
				{i18nMsg(
					'invoices-write-off-missing-rounding-account',
					"Ce reste porte une fraction de centime, qui va au compte de différences d'arrondi — aucun n'est désigné : demandez à un administrateur de le choisir dans Paramètres → Facturation.",
				)}
			</div>
		{/if}

		<div class="mt-2">
			<label class="mb-1 block text-xs text-text-muted" for="write-off-date">
				{i18nMsg('invoices-write-off-date-label', 'Date du solde')}
			</label>
			<Input
				id="write-off-date"
				type="date"
				bind:value={settledOn}
				min={invoiceDate}
				data-testid="write-off-date"
			/>
		</div>

		{#if errorMsg}
			<div
				class="rounded-md border border-destructive bg-destructive/10 px-3 py-2 text-sm text-destructive"
				role="alert"
				data-testid="write-off-error"
			>
				{errorMsg}
			</div>
		{:else if clientError && nature !== null}
			<div
				class="rounded-md border border-destructive bg-destructive/10 px-3 py-2 text-sm text-destructive"
			>
				{clientError}
			</div>
		{/if}

		<Dialog.Footer>
			<Button variant="outline" onclick={() => onOpenChange(false)} disabled={submitting}>
				{i18nMsg('common-cancel', 'Annuler')}
			</Button>
			<Button
				onclick={handleConfirm}
				disabled={submitting || !!clientError || missing !== null}
				data-testid="write-off-confirm"
			>
				{i18nMsg('invoices-write-off-confirm', 'Solder le reste')}
			</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>
