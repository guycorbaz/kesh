<!--
  Story 36-16a (#1167) — « Changer le compte analytique », le dialogue PARTAGÉ
  des trois gestes de réimputation : une facture client (36-16a), une facture
  fournisseur (36-16b), les lignes d'une écriture (36-16c).

  Contrat (fiche 36-16a, D12) :
  - `rows` : une rangée par objet à réimputer — une pour une facture (`id` = la
    facture), une par ligne pour une écriture (`id` = la ligne) ;
  - `projects` : les comptes analytiques, archivés compris (la page les charge) ;
  - `submit(choices, acknowledge)` : l'appel de l'API, fourni par la page, qui
    REPREND l'objet rendu (version comprise) ; il rejette avec l'erreur de l'API ;
  - `onDone()` : le dialogue est fini — enregistré ou abandonné. La page le monte
    à l'ouverture et le démonte ici.

  Les refus de période close (`details { entryCount, fiscalYears, lockedThrough }`) :
  - `409 PROJECT_REASSIGNMENT_CLOSED_PERIOD` — avertissement et *Confirmer*, qui
    renvoie avec l'accusé ;
  - `403 PROJECT_REASSIGNMENT_CLOSED_PERIOD_ADMIN_ONLY` — le message de l'API,
    sans *Confirmer* : le dialogue ne cache pas la période close au Comptable.
-->
<script lang="ts">
	import { untrack } from 'svelte';
	import { Button } from '$lib/components/ui/button';
	import * as Dialog from '$lib/components/ui/dialog';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { isApiError } from '$lib/shared/utils/api-error';
	import { formatSwissDate } from '$lib/features/reports/reports.api';
	import type { ProjectResponse } from './projects.types';

	export type ReassignRow = { id: number; label: string; currentProjectId: number | null };
	export type ReassignChoice = { id: number; projectId: number | null };

	type Props = {
		rows: ReassignRow[];
		projects: ProjectResponse[];
		submit: (choices: ReassignChoice[], acknowledge: boolean) => Promise<void>;
		onDone: () => void;
	};

	let { rows, projects, submit, onDone }: Props = $props();

	type Warning = { entryCount: number; fiscalYears: string[]; lockedThrough: string | null };

	// Le choix de chaque rangée, initialisé à son compte actuel (copie à
	// l'ouverture : le dialogue est monté à chaque ouverture).
	let choices = $state<Record<number, number | null>>(
		Object.fromEntries(untrack(() => rows).map((r) => [r.id, r.currentProjectId])),
	);
	let submitting = $state(false);
	let errorMsg = $state('');
	let warning = $state<Warning | null>(null);

	let changed = $derived(rows.some((r) => choices[r.id] !== r.currentProjectId));

	/** Les comptes proposés à une rangée : les non archivés, plus son actuel s'il est archivé. */
	function optionsFor(row: ReassignRow): ProjectResponse[] {
		return projects.filter((p) => !p.archived || p.id === row.currentProjectId);
	}

	function label(p: ProjectResponse): string {
		const name = `${p.code} — ${p.name}`;
		return p.archived
			? i18nMsg('projects-reassign-archived', `${name} (archivé)`, { name })
			: name;
	}

	function readWarning(details: Record<string, unknown> | undefined): Warning {
		return {
			entryCount: Number(details?.entryCount ?? 0),
			fiscalYears: Array.isArray(details?.fiscalYears) ? (details.fiscalYears as string[]) : [],
			lockedThrough: typeof details?.lockedThrough === 'string' ? details.lockedThrough : null,
		};
	}

	async function send(acknowledge: boolean) {
		// La garde vit ici aussi : un bouton désactivé n'arrête pas tous les clics
		// (patron de `WriteOffDialog.handleConfirm`).
		if (submitting || !changed) return;
		submitting = true;
		errorMsg = '';
		try {
			await submit(
				rows.map((r) => ({ id: r.id, projectId: choices[r.id] ?? null })),
				acknowledge,
			);
			onDone();
		} catch (err) {
			if (isApiError(err) && err.code === 'PROJECT_REASSIGNMENT_CLOSED_PERIOD') {
				warning = readWarning(err.details);
			} else {
				warning = null;
				errorMsg = isApiError(err)
					? err.message
					: i18nMsg('error-unexpected', 'Erreur inattendue.');
			}
		} finally {
			submitting = false;
		}
	}

	function selectValue(e: Event): number | null {
		const v = (e.currentTarget as HTMLSelectElement).value;
		return v === '' ? null : Number(v);
	}
</script>

<Dialog.Root open={true} onOpenChange={(o) => !o && !submitting && onDone()}>
	<Dialog.Content
		escapeKeydownBehavior={submitting ? 'ignore' : 'close'}
		interactOutsideBehavior={submitting ? 'ignore' : 'close'}
		showCloseButton={!submitting}
	>
		<Dialog.Header>
			<Dialog.Title>
				{i18nMsg('projects-reassign-title', 'Changer le compte analytique')}
			</Dialog.Title>
		</Dialog.Header>

		<p class="text-sm">
			{i18nMsg(
				'projects-reassign-body',
				'La pièce et toutes ses écritures passent au compte analytique choisi. Les montants, les comptes, les dates, les numéros, le lettrage et le rapprochement ne changent pas.',
			)}
		</p>

		<div class="mt-2 space-y-3">
			{#each rows as row (row.id)}
				<div>
					<label class="mb-1 block text-sm font-medium" for="project-reassign-{row.id}">
						{row.label}
					</label>
					<select
						id="project-reassign-{row.id}"
						value={choices[row.id] === null ? '' : String(choices[row.id])}
						onchange={(e) => {
							choices[row.id] = selectValue(e);
							warning = null;
							errorMsg = '';
						}}
						disabled={submitting}
						data-testid="project-reassign-select-{row.id}"
						class="w-full h-9 rounded-md border border-input bg-background px-2 text-sm"
					>
						<option value="">{i18nMsg('projects-reassign-none', '— Aucun')}</option>
						{#each optionsFor(row) as p (p.id)}
							<option value={String(p.id)}>{label(p)}</option>
						{/each}
					</select>
				</div>
			{/each}
		</div>

		{#if warning}
			<div
				class="rounded-md border border-warning bg-warning/10 px-3 py-2 text-sm space-y-1"
				role="alert"
				data-testid="project-reassign-warning"
			>
				<p class="font-medium">
					{i18nMsg('projects-reassign-warning-title', 'Exercice clos ou période verrouillée')}
				</p>
				<!-- Deux clés plates : le dictionnaire du frontend résout sans arguments,
				     un sélecteur Fluent y tomberait toujours sur sa branche `other`. -->
				<p data-testid="project-reassign-warning-entries">
					{warning.entryCount === 1
						? i18nMsg(
								'projects-reassign-warning-entries-one',
								"Une écriture d'une période close changera de compte analytique.",
							)
						: i18nMsg(
								'projects-reassign-warning-entries-other',
								`${warning.entryCount} écritures d'une période close changeront de compte analytique.`,
								{ entryCount: warning.entryCount },
							)}
				</p>
				{#if warning.fiscalYears.length > 0}
					<p data-testid="project-reassign-warning-fiscal-years">
						{i18nMsg(
							'projects-reassign-warning-fiscal-years',
							`Exercice clos : ${warning.fiscalYears.join(', ')}.`,
							{ names: warning.fiscalYears.join(', ') },
						)}
					</p>
				{/if}
				{#if warning.lockedThrough}
					<p data-testid="project-reassign-warning-locked">
						{i18nMsg(
							'projects-reassign-warning-locked',
							`Écritures verrouillées jusqu'au ${formatSwissDate(warning.lockedThrough)}.`,
							{ date: formatSwissDate(warning.lockedThrough) },
						)}
					</p>
				{/if}
				<p>
					{i18nMsg(
						'projects-reassign-warning-reports',
						'Les rapports analytiques de cette période changeront.',
					)}
				</p>
			</div>
		{/if}

		{#if errorMsg}
			<div
				class="rounded-md border border-destructive bg-destructive/10 px-3 py-2 text-sm text-destructive"
				role="alert"
				data-testid="project-reassign-error"
			>
				{errorMsg}
			</div>
		{/if}

		<Dialog.Footer>
			<Button variant="outline" onclick={onDone} disabled={submitting}>
				{i18nMsg('common-cancel', 'Annuler')}
			</Button>
			{#if warning}
				<Button
					onclick={() => send(true)}
					disabled={submitting}
					data-testid="project-reassign-confirm"
				>
					{i18nMsg('projects-reassign-confirm', 'Confirmer')}
				</Button>
			{:else}
				<Button
					onclick={() => send(false)}
					disabled={submitting || !changed}
					data-testid="project-reassign-save"
				>
					{i18nMsg('projects-reassign-save', 'Enregistrer')}
				</Button>
			{/if}
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>
