<!--
  Story v016-2b (#816) — saisir un relevé bancaire à la main.

  Contrat : `bankAccounts` (ceux de `GET /companies/current`, ni archivés ni
  clos), `onSaved(imported)` après une création réussie.

  Machine d'état :
  - saisie : totaux, solde calculé et écart recalculés à chaque frappe
    (`manual-statement.ts`) ; « Enregistrer » inactif avec **une** raison, la
    première de la séquence de l'AC-21 ;
  - « Vérifier » (`verify`) : prend un INSTANTANÉ du corps (clé sans les cases,
    table des rangs) et appelle l'aperçu ; le formulaire est figé pendant l'envoi.
    Chaque demande porte un numéro (`seq`) : seule la dernière pose son résultat.
    La réponse est liée à l'instantané envoyé — si le relevé a changé entre-temps,
    l'aperçu est aussitôt périmé (patron `loadSeq` de `bank-import/[id]`) ;
  - aperçu réussi (`previewFresh`) tant que la clé du relevé courant égale celle
    de l'instantané : toute modification d'un champ du relevé le périme (cases
    décochées) ; cocher une case ne le périme pas ;
  - « Enregistrer » (`save`, garde `submitting`) : envoie le corps courant, cases
    comprises. Un refus qui demande une confirmation (relevé identique, doublons,
    écart de solde de départ apparus depuis l'aperçu) retire l'aperçu : il faut
    vérifier de nouveau pour voir la case.
  Les refus du serveur désignent la cellule (ligne de l'écran, par la table des
  rangs de l'instantané) ou le champ d'entête ; un `400 VALIDATION_ERROR` sans
  `field` s'affiche par un message générique.
-->
<script lang="ts">
	import Big from 'big.js';
	import { Button } from '$lib/components/ui/button';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { formatSwissAmount } from '$lib/features/journal-entries/balance';
	import {
		bankAccountOptionLabel,
		type BankAccountOptionSource,
	} from '$lib/features/bank-accounts/format';
	import { previewManualStatement, createManualStatement } from './bank-import.api';
	import type { BankImportResponse, ManualStatementPreviewResponse } from './bank-import.types';
	import {
		buildManualStatementBody,
		canVerify,
		computeManualTotals,
		describeManualError,
		emptyManualLine,
		isBlankLine,
		lineFaults,
		manualSnapshotKey,
		saveBlockReason,
		screenRowOf,
		type ManualBlockReason,
		type ManualErrorView,
		type ManualLineCell,
		type ManualStatementInput,
	} from './manual-statement';
	import { manualStatementReasonLabel } from './manual-statement-reason-label';

	type Props = {
		bankAccounts: Array<BankAccountOptionSource & { id: number }>;
		onSaved?: (imported: BankImportResponse) => void;
	};
	let { bankAccounts, onSaved }: Props = $props();

	/** Un aperçu réussi, lié à l'instantané du corps qu'il a vu. */
	type PreviewSnapshot = {
		key: string;
		rowMap: number[];
		response: ManualStatementPreviewResponse;
	};

	let form = $state<ManualStatementInput>({
		bankAccountId: null,
		periodFrom: '',
		periodTo: '',
		statementNumber: '',
		openingBalance: '',
		closingBalance: '',
		lines: [emptyManualLine()],
	});
	let confirmDuplicateFile = $state(false);
	let confirmDuplicateLines = $state(false);
	let confirmOpeningBalanceGap = $state(false);
	let preview = $state<PreviewSnapshot | null>(null);
	let previewStale = $state(false);
	let previewing = $state(false);
	let submitting = $state(false);
	let error = $state<{ view: ManualErrorView; key: string } | null>(null);
	/** Numéro de la dernière vérification lancée (variable ordinaire, patron `loadSeq`). */
	let seq = 0;

	const built = $derived(
		buildManualStatementBody(form, { confirmDuplicateFile, confirmDuplicateLines, confirmOpeningBalanceGap }),
	);
	const currentKey = $derived(manualSnapshotKey(built.body));
	const previewFresh = $derived(preview !== null && preview.key === currentKey);
	const warnings = $derived(previewFresh && preview ? preview.response.warnings : null);
	const confirmationsMissing = $derived(
		warnings !== null &&
			((warnings.duplicateFile !== null && !confirmDuplicateFile) ||
				(warnings.duplicateLines.length > 0 && !confirmDuplicateLines) ||
				(warnings.openingBalanceGap !== null && !confirmOpeningBalanceGap)),
	);
	const blockReason = $derived(saveBlockReason(form, { previewSucceeded: previewFresh, confirmationsMissing }));
	const totals = $derived(computeManualTotals(form));
	const faults = $derived(form.lines.map((l) => (isBlankLine(l) ? [] : lineFaults(l, form))));
	const frozen = $derived(previewing || submitting);
	/** Lignes de l'écran que le serveur désigne (0-based). */
	const serverRows = $derived.by(() => {
		const v = error?.view;
		if (v?.kind === 'cell') return [v.row];
		if (v?.kind === 'rows') return v.rows;
		return [] as number[];
	});

	// Un aperçu dont le relevé a changé est périmé : il tombe, et ses cases avec lui.
	$effect(() => {
		if (preview !== null && !previewFresh) {
			preview = null;
			previewStale = true;
			resetConfirmations();
		}
	});

	// Un refus affiché vaut pour le relevé qui l'a reçu.
	$effect(() => {
		if (error !== null && error.key !== currentKey) error = null;
	});

	function resetConfirmations(): void {
		confirmDuplicateFile = false;
		confirmDuplicateLines = false;
		confirmOpeningBalanceGap = false;
	}

	function addLine(): void {
		form.lines.push(emptyManualLine());
	}

	function removeLine(index: number): void {
		form.lines.splice(index, 1);
	}

	async function verify(): Promise<void> {
		if (frozen || !canVerify(form)) return;
		const mySeq = ++seq;
		const sent = built;
		const key = currentKey;
		previewing = true;
		error = null;
		preview = null;
		previewStale = false;
		resetConfirmations();
		try {
			const response = await previewManualStatement(sent.body);
			if (mySeq !== seq) return;
			preview = { key, rowMap: sent.rowMap, response };
		} catch (err) {
			if (mySeq !== seq) return;
			error = { view: describeManualError(err, sent.rowMap), key };
		} finally {
			if (mySeq === seq) previewing = false;
		}
	}

	/** Les refus qui demandent une case : l'aperçu tombe, il faut vérifier de nouveau. */
	const NEEDS_NEW_PREVIEW = new Set([
		'BANK_IMPORT_DUPLICATE_FILE',
		'BANK_STATEMENT_DUPLICATE_LINES',
		'BANK_STATEMENT_OPENING_BALANCE_GAP',
	]);

	async function save(): Promise<void> {
		if (submitting || previewing || blockReason !== null) return;
		submitting = true;
		error = null;
		const sent = built;
		const key = currentKey;
		try {
			const imported = await createManualStatement(sent.body);
			onSaved?.(imported);
		} catch (err) {
			error = { view: describeManualError(err, sent.rowMap), key };
			const code = (err as { code?: unknown })?.code;
			if (typeof code === 'string' && NEEDS_NEW_PREVIEW.has(code)) {
				preview = null;
				resetConfirmations();
			}
		} finally {
			submitting = false;
		}
	}

	/** Le libellé d'un champ du corps : celui du formulaire ou de l'en-tête de colonne. */
	function fieldLabel(field: string): string {
		switch (field) {
			case 'periodFrom':
				return i18nMsg('bank-import-manual-period-from', 'Du');
			case 'periodTo':
				return i18nMsg('bank-import-manual-period-to', 'Au');
			case 'statementNumber':
				return i18nMsg('bank-import-manual-statement-number', 'Numéro du relevé');
			case 'openingBalance':
				return i18nMsg('bank-import-manual-opening-balance', 'Solde de départ');
			case 'closingBalance':
				return i18nMsg('bank-import-manual-closing-balance', "Solde d'arrivée");
			case 'lines':
				return i18nMsg('bank-import-manual-field-lines', 'Lignes');
			case 'bookingDate':
				return i18nMsg('bank-import-manual-col-booking-date', 'Date');
			case 'valueDate':
				return i18nMsg('bank-import-manual-col-value-date', 'Valeur');
			case 'details':
				return i18nMsg('bank-import-manual-col-details', 'Libellé');
			case 'counterpartyName':
				return i18nMsg('bank-import-manual-col-counterparty', 'Contrepartie');
			case 'reference':
				return i18nMsg('bank-import-manual-col-reference', 'Référence');
			case 'amount':
			case 'direction':
				return i18nMsg('bank-import-manual-field-amount', 'Débit / Crédit');
			default:
				return field;
		}
	}

	/** La raison d'un « Enregistrer » inactif, dite à côté du bouton. */
	function blockReasonText(r: ManualBlockReason): string {
		switch (r.kind) {
			case 'account':
				return i18nMsg('bank-import-manual-blocked-account', 'Choisissez le compte bancaire.');
			case 'period':
				return i18nMsg('bank-import-manual-blocked-period', 'Saisissez la période du relevé (du, au).');
			case 'period-inverted':
				return i18nMsg('bank-import-manual-blocked-period-inverted', 'La fin de la période précède son début.');
			case 'statement-number':
				return i18nMsg(
					'bank-import-manual-blocked-statement-number',
					'Le numéro du relevé dépasse 255 caractères.',
				);
			case 'balances':
				return i18nMsg(
					'bank-import-manual-blocked-balances',
					"Saisissez les soldes de départ et d'arrivée : deux décimales au plus, treize chiffres avant la virgule.",
				);
			case 'line-count':
				return i18nMsg('bank-import-manual-blocked-line-count', 'Un relevé saisi compte 500 lignes au plus.');
			case 'line':
				return i18nMsg(
					'bank-import-manual-blocked-line',
					'La ligne { $line } est à corriger : une date dans la période, un libellé, un seul montant positif à deux décimales au plus.',
					{ line: r.line },
				);
			case 'gap':
				return i18nMsg(
					'bank-import-manual-blocked-gap',
					"Le solde calculé ne donne pas le solde d'arrivée : l'écart doit être nul.",
				);
			case 'preview':
				return i18nMsg('bank-import-manual-blocked-preview', "Vérifiez le relevé avant de l'enregistrer.");
			case 'confirmations':
				return i18nMsg(
					'bank-import-manual-blocked-confirmations',
					'Cochez les confirmations que demande la vérification.',
				);
		}
	}

	/** Le texte d'un refus du serveur. */
	function errorText(v: ManualErrorView): string {
		switch (v.kind) {
			case 'cell':
				return i18nMsg('bank-import-manual-error-cell', 'Ligne { $line }, { $field } : { $reason }', {
					line: v.row + 1,
					field: fieldLabel(v.field),
					reason: manualStatementReasonLabel(v.reason),
				});
			case 'field':
				return i18nMsg('bank-import-manual-error-field', '{ $field } : { $reason }', {
					field: fieldLabel(v.field),
					reason: manualStatementReasonLabel(v.reason),
				});
			case 'rows':
				return `${v.message} ${i18nMsg('bank-import-manual-error-rows', 'Lignes concernées : { $lines }.', {
					lines: v.rows.map((r) => r + 1).join(', '),
				})}`;
			case 'rejected':
				return i18nMsg('bank-import-manual-error-rejected', 'Relevé refusé : trop volumineux ou illisible.');
			case 'duplicate-file':
				return i18nMsg(
					'bank-import-manual-error-duplicate-file',
					'Un relevé identique est déjà enregistré : vérifiez de nouveau le relevé, puis confirmez pour l’enregistrer quand même.',
				);
			case 'message':
				return v.message;
			case 'unexpected':
				return i18nMsg('error-unexpected', 'Erreur inattendue.');
		}
	}

	const cellClass = (bad: boolean) =>
		`w-full rounded border bg-surface p-1 text-text ${bad ? 'border-destructive' : 'border-border'}`;
	const isFaulty = (i: number, cell: ManualLineCell) => faults[i]?.includes(cell) ?? false;
	/** Un montant du serveur (chaîne décimale) au format suisse. */
	const swiss = (s: string) => formatSwissAmount(new Big(s));
	/** Les rangs envoyés (1-based) d'avertissements, en lignes de l'écran (1-based). */
	const screenLines = (lines: number[], rowMap: readonly number[]) =>
		lines.map((l) => (screenRowOf(rowMap, l) ?? l - 1) + 1).join(', ');
</script>

<section data-testid="bank-import-manual-form">
	<fieldset disabled={frozen} class="grid gap-4 md:grid-cols-2">
		<div class="md:col-span-2">
			<label class="block text-sm font-medium" for="manual-account">
				{i18nMsg('bank-import-labels-bank-account-selector', 'Compte bancaire cible')}
			</label>
			<select
				id="manual-account"
				data-testid="bank-import-manual-account"
				bind:value={form.bankAccountId}
				class="mt-1 block w-full rounded border-border bg-surface p-2 text-text"
			>
				<option value={null}>— {i18nMsg('bank-import-labels-bank-account-placeholder', 'Sélectionner un compte')} —</option>
				{#each bankAccounts as account (account.id)}
					<option value={account.id}>{bankAccountOptionLabel(account, 'parens')}</option>
				{/each}
			</select>
		</div>
		<div>
			<label class="block text-sm font-medium" for="manual-period-from">{fieldLabel('periodFrom')}</label>
			<input
				id="manual-period-from"
				type="date"
				data-testid="bank-import-manual-period-from"
				bind:value={form.periodFrom}
				class={cellClass(error?.view.kind === 'field' && error.view.field === 'periodFrom')}
			/>
		</div>
		<div>
			<label class="block text-sm font-medium" for="manual-period-to">{fieldLabel('periodTo')}</label>
			<input
				id="manual-period-to"
				type="date"
				data-testid="bank-import-manual-period-to"
				bind:value={form.periodTo}
				class={cellClass(error?.view.kind === 'field' && error.view.field === 'periodTo')}
			/>
		</div>
		<div class="md:col-span-2">
			<label class="block text-sm font-medium" for="manual-statement-number">
				{fieldLabel('statementNumber')}
				<span class="text-text-muted">({i18nMsg('bank-import-manual-optional', 'facultatif')})</span>
			</label>
			<input
				id="manual-statement-number"
				type="text"
				data-testid="bank-import-manual-statement-number"
				bind:value={form.statementNumber}
				class={cellClass(error?.view.kind === 'field' && error.view.field === 'statementNumber')}
			/>
		</div>
		<div>
			<label class="block text-sm font-medium" for="manual-opening">{fieldLabel('openingBalance')}</label>
			<input
				id="manual-opening"
				type="text"
				inputmode="decimal"
				data-testid="bank-import-manual-opening-balance"
				bind:value={form.openingBalance}
				class="{cellClass(error?.view.kind === 'field' && error.view.field === 'openingBalance')} text-right tabular-nums"
			/>
		</div>
		<div>
			<label class="block text-sm font-medium" for="manual-closing">{fieldLabel('closingBalance')}</label>
			<input
				id="manual-closing"
				type="text"
				inputmode="decimal"
				data-testid="bank-import-manual-closing-balance"
				bind:value={form.closingBalance}
				class="{cellClass(error?.view.kind === 'field' && error.view.field === 'closingBalance')} text-right tabular-nums"
			/>
		</div>

		<div class="md:col-span-2 overflow-x-auto">
			<table class="w-full table-auto text-sm" data-testid="bank-import-manual-lines">
				<thead>
					<tr>
						<th class="text-left">{fieldLabel('bookingDate')}</th>
						<th class="text-left">{fieldLabel('valueDate')}</th>
						<th class="text-left">{fieldLabel('details')}</th>
						<th class="text-left">{fieldLabel('counterpartyName')}</th>
						<th class="text-left">{fieldLabel('reference')}</th>
						<th class="text-right">{i18nMsg('bank-import-manual-col-debit', 'Débit')}</th>
						<th class="text-right">{i18nMsg('bank-import-manual-col-credit', 'Crédit')}</th>
						<th></th>
					</tr>
				</thead>
				<tbody>
					{#each form.lines as line, i (line)}
						<tr
							data-testid="bank-import-manual-line"
							data-error={serverRows.includes(i) ? 'true' : 'false'}
							class={serverRows.includes(i) ? 'bg-error-soft' : ''}
						>
							<td>
								<input
									type="date"
									data-testid="bank-import-manual-line-booking-date"
									bind:value={line.bookingDate}
									aria-invalid={isFaulty(i, 'bookingDate')}
									class={cellClass(isFaulty(i, 'bookingDate'))}
								/>
							</td>
							<td>
								<input
									type="date"
									data-testid="bank-import-manual-line-value-date"
									bind:value={line.valueDate}
									class={cellClass(false)}
								/>
							</td>
							<td>
								<input
									type="text"
									data-testid="bank-import-manual-line-details"
									bind:value={line.details}
									aria-invalid={isFaulty(i, 'details')}
									class={cellClass(isFaulty(i, 'details'))}
								/>
							</td>
							<td>
								<input
									type="text"
									data-testid="bank-import-manual-line-counterparty"
									bind:value={line.counterpartyName}
									aria-invalid={isFaulty(i, 'counterpartyName')}
									class={cellClass(isFaulty(i, 'counterpartyName'))}
								/>
							</td>
							<td>
								<input
									type="text"
									data-testid="bank-import-manual-line-reference"
									bind:value={line.reference}
									aria-invalid={isFaulty(i, 'reference')}
									class={cellClass(isFaulty(i, 'reference'))}
								/>
							</td>
							<td>
								<input
									type="text"
									inputmode="decimal"
									data-testid="bank-import-manual-line-debit"
									bind:value={line.debit}
									aria-invalid={isFaulty(i, 'debit')}
									class="{cellClass(isFaulty(i, 'debit'))} text-right tabular-nums"
								/>
							</td>
							<td>
								<input
									type="text"
									inputmode="decimal"
									data-testid="bank-import-manual-line-credit"
									bind:value={line.credit}
									aria-invalid={isFaulty(i, 'credit')}
									class="{cellClass(isFaulty(i, 'credit'))} text-right tabular-nums"
								/>
							</td>
							<td>
								<button
									type="button"
									class="text-destructive underline"
									data-testid="bank-import-manual-remove-line"
									aria-label={i18nMsg('bank-import-manual-remove-line', 'Retirer la ligne { $line }', {
										line: i + 1,
									})}
									onclick={() => removeLine(i)}
								>
									×
								</button>
							</td>
						</tr>
					{/each}
				</tbody>
			</table>
			<Button variant="outline" class="mt-2" onclick={addLine} data-testid="bank-import-manual-add-line">
				{i18nMsg('bank-import-manual-add-line', 'Ajouter une ligne')}
			</Button>
		</div>
	</fieldset>

	<dl class="mt-6 grid max-w-md grid-cols-2 gap-1 text-sm" data-testid="bank-import-manual-totals">
		<dt>{i18nMsg('bank-import-manual-total-debits', 'Total des débits')}</dt>
		<dd class="text-right tabular-nums" data-testid="bank-import-manual-total-debits">
			{formatSwissAmount(totals.totalDebits)}
		</dd>
		<dt>{i18nMsg('bank-import-manual-total-credits', 'Total des crédits')}</dt>
		<dd class="text-right tabular-nums" data-testid="bank-import-manual-total-credits">
			{formatSwissAmount(totals.totalCredits)}
		</dd>
		{#if totals.computedClosing !== null}
			<dt>{i18nMsg('bank-import-manual-computed-closing', 'Solde calculé')}</dt>
			<dd class="text-right tabular-nums" data-testid="bank-import-manual-computed-closing">
				{formatSwissAmount(totals.computedClosing)}
			</dd>
		{/if}
		{#if totals.enteredClosing !== null}
			<dt>{i18nMsg('bank-import-manual-entered-closing', "Solde d'arrivée saisi")}</dt>
			<dd class="text-right tabular-nums" data-testid="bank-import-manual-entered-closing">
				{formatSwissAmount(totals.enteredClosing)}
			</dd>
		{/if}
		{#if totals.gap !== null}
			<dt class="font-semibold">{i18nMsg('bank-import-manual-gap', 'Écart')}</dt>
			<dd
				class="text-right font-semibold tabular-nums {totals.gap.eq(0) ? 'text-success' : 'text-error'}"
				data-testid="bank-import-manual-gap"
			>
				{formatSwissAmount(totals.gap)}
			</dd>
		{/if}
	</dl>

	{#if error}
		<div
			class="mt-4 rounded border border-error bg-error-soft p-3 text-sm text-error"
			role="alert"
			data-testid="bank-import-manual-error"
		>
			{errorText(error.view)}
		</div>
	{/if}

	{#if previewStale && preview === null && !previewing}
		<p class="mt-4 text-sm text-text-muted" data-testid="bank-import-manual-preview-stale">
			{i18nMsg(
				'bank-import-manual-preview-stale',
				'Le relevé a changé depuis la vérification : vérifiez-le de nouveau.',
			)}
		</p>
	{/if}

	{#if previewFresh && warnings}
		<div class="mt-4 space-y-3">
			<p class="text-sm text-success" data-testid="bank-import-manual-preview-ok">
				{i18nMsg('bank-import-manual-preview-ok', 'Vérification réussie : le relevé est équilibré.')}
			</p>
			{#if warnings.duplicateFile}
				<div class="rounded border border-warning bg-warning-soft p-3 text-sm" data-testid="bank-import-manual-warning-duplicate-file">
					<p>
						{i18nMsg(
							'bank-import-manual-warning-duplicate-file',
							'Un relevé identique (même compte, période, soldes et lignes) est déjà enregistré : relevé #{ $id }.',
							{ id: warnings.duplicateFile.existingImportId },
						)}
					</p>
					<label class="mt-2 flex items-center gap-2">
						<input type="checkbox" data-testid="bank-import-manual-confirm-duplicate-file" bind:checked={confirmDuplicateFile} />
						{i18nMsg('bank-import-manual-confirm-duplicate-file', 'Enregistrer quand même ce relevé')}
					</label>
				</div>
			{/if}
			{#if warnings.duplicateLines.length > 0}
				<div class="rounded border border-warning bg-warning-soft p-3 text-sm" data-testid="bank-import-manual-warning-duplicate-lines">
					<p>
						{i18nMsg(
							'bank-import-manual-warning-duplicate-lines',
							'Ces lignes existent déjà sur ce compte : { $lines }. Elles ne sont jamais ignorées : corrigez-les, ou confirmez pour les enregistrer.',
							{
								lines: screenLines(
									warnings.duplicateLines.map((d) => d.line),
									preview?.rowMap ?? [],
								),
							},
						)}
					</p>
					<label class="mt-2 flex items-center gap-2">
						<input type="checkbox" data-testid="bank-import-manual-confirm-duplicate-lines" bind:checked={confirmDuplicateLines} />
						{i18nMsg('bank-import-manual-confirm-duplicate-lines', 'Enregistrer aussi ces lignes')}
					</label>
				</div>
			{/if}
			{#if warnings.openingBalanceGap}
				<div class="rounded border border-warning bg-warning-soft p-3 text-sm" data-testid="bank-import-manual-warning-opening-gap">
					<p>
						{i18nMsg(
							'bank-import-manual-warning-opening-gap',
							"Le solde de départ attendu, d'après le relevé précédent qui finit le { $date }, est { $expected } ; vous avez saisi { $entered }.",
							{
								date: warnings.openingBalanceGap.previousPeriodTo,
								expected: swiss(warnings.openingBalanceGap.expected),
								entered: swiss(warnings.openingBalanceGap.entered),
							},
						)}
					</p>
					<label class="mt-2 flex items-center gap-2">
						<input type="checkbox" data-testid="bank-import-manual-confirm-opening-gap" bind:checked={confirmOpeningBalanceGap} />
						{i18nMsg('bank-import-manual-confirm-opening-gap', 'Enregistrer avec ce solde de départ')}
					</label>
				</div>
			{/if}
			{#if warnings.overlappingStatements.length > 0}
				<p class="rounded border border-border bg-surface p-3 text-sm" data-testid="bank-import-manual-info-overlap">
					{i18nMsg(
						'bank-import-manual-info-overlap',
						'Information : ce relevé chevauche des relevés de ce compte ({ $periods }).',
						{
							periods: warnings.overlappingStatements
								.map((o) => `${o.periodFrom} → ${o.periodTo}`)
								.join(', '),
						},
					)}
				</p>
			{/if}
		</div>
	{/if}

	<div class="mt-6 flex flex-wrap items-center gap-3">
		<Button
			variant="outline"
			onclick={verify}
			disabled={frozen || !canVerify(form)}
			data-testid="bank-import-manual-verify"
		>
			{i18nMsg('bank-import-manual-verify', 'Vérifier')}
		</Button>
		<Button onclick={save} disabled={frozen || blockReason !== null} data-testid="bank-import-manual-save">
			{i18nMsg('bank-import-manual-save', 'Enregistrer')}
		</Button>
		{#if blockReason !== null}
			<p class="text-sm text-text-muted" data-testid="bank-import-manual-block-reason" data-reason={blockReason.kind}>
				{blockReasonText(blockReason)}
			</p>
		{/if}
	</div>
</section>
