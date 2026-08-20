<script lang="ts">
	import { onMount } from 'svelte';
	import Big from 'big.js';
	import { isApiError } from '$lib/shared/utils/api-client';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { notifySuccess, notifyError, notifyWarning } from '$lib/shared/utils/notify';
	import {
		triggerInboxImport,
		listImported,
		completeImport,
		discardImport,
		downloadImportedSourceDocument,
	} from '$lib/features/imported-supplier-invoices/imported-supplier-invoices.api';
	import type {
		ImportedSupplierInvoice,
		InboxImportReport,
		CompleteImportLineRequest,
	} from '$lib/features/imported-supplier-invoices/imported-supplier-invoices.types';
	import { importErrorLabel } from '$lib/features/imported-supplier-invoices/error-label';
	import { listContacts } from '$lib/features/contacts/contacts.api';
	import type { ContactResponse } from '$lib/features/contacts/contacts.types';
	import { fetchAccounts } from '$lib/features/accounts/accounts.api';
	import type { AccountResponse } from '$lib/features/accounts/accounts.types';
	import { listVatRates } from '$lib/features/vat-rates/vat-rates.api';
	import type { VatRateResponse } from '$lib/features/vat-rates/vat-rates.types';
	import { lineVatAmount } from '$lib/features/journal-entries/vat-purchase';
	import { formatSwissAmount, isValidAmount } from '$lib/features/journal-entries/balance';

	let toComplete = $state<ImportedSupplierInvoice[]>([]);
	let loading = $state(true);
	let errorMsg = $state('');

	let report = $state<InboxImportReport | null>(null);
	let importing = $state(false);

	// Référentiels du formulaire de complétion.
	let suppliers = $state<ContactResponse[]>([]);
	let expenseAccounts = $state<AccountResponse[]>([]);
	let vatRates = $state<VatRateResponse[]>([]);

	// État du formulaire de complétion (par ligne `to_complete`).
	let completingId = $state<number | null>(null);
	let saving = $state(false);
	let formError = $state('');
	const today = new Date().toISOString().slice(0, 10);
	let fContactId = $state<number | null>(null);
	let fNumber = $state('');
	let fInvoiceDate = $state(today);
	let fDueDate = $state('');
	let fLines = $state<CompleteImportLineRequest[]>([
		{ description: '', quantity: '1', unitPrice: '', vatRate: '0', expenseAccountId: 0 },
	]);

	async function reloadList() {
		toComplete = await listImported('to_complete');
	}

	/** Rechargement « best-effort » : ne propage jamais (évite une rejection non
	 * gérée quand appelé depuis un `catch`, BH7/EC6). Notifie sur échec. */
	async function safeReloadList() {
		try {
			await reloadList();
		} catch {
			notifyWarning(
				i18nMsg(
					'imported-supplier-invoices-reload-failed',
					'La liste n’a pas pu être rechargée — actualisez la page.',
				),
			);
		}
	}

	onMount(async () => {
		try {
			const [, contactsRes, accountsRes, ratesRes] = await Promise.all([
				reloadList(),
				listContacts({ isSupplier: true, limit: 200 }),
				fetchAccounts(),
				listVatRates(),
			]);
			suppliers = contactsRes.items;
			expenseAccounts = accountsRes.filter(
				(a) => a.accountType === 'Expense' && a.active && a.postable,
			); // 14-3b : compte de charge posté à la complétion d'import
			vatRates = ratesRes;
		} catch (err) {
			if (isApiError(err)) errorMsg = err.message;
		} finally {
			loading = false;
		}
	});

	async function runImport() {
		if (importing) return;
		importing = true;
		try {
			report = await triggerInboxImport();
		} catch (err) {
			report = null;
			if (isApiError(err) && err.code === 'INBOX_IMPORT_ALREADY_RUNNING') {
				notifyError(
					i18nMsg(
						'imported-supplier-invoices-import-running',
						'Un import est déjà en cours. Réessayez dans quelques instants.',
					),
				);
			} else {
				notifyError(
					(isApiError(err) && err.message) ||
						i18nMsg('imported-supplier-invoices-import-failed', 'Erreur inattendue lors de l’import.'),
				);
			}
			importing = false;
			return;
		}
		// Rechargement de la liste séparé du déclenchement (BH1/EC2) : un échec de
		// `reloadList` ne doit PAS effacer le rapport d'import qui vient de réussir.
		try {
			await reloadList();
		} catch {
			notifyWarning(
				i18nMsg(
					'imported-supplier-invoices-completed-reload-failed',
					'Import effectué, mais la liste n’a pas pu être rechargée — actualisez la page.',
				),
			);
		} finally {
			importing = false;
		}
	}

	function openComplete(row: ImportedSupplierInvoice) {
		completingId = row.id;
		formError = '';
		fContactId = null;
		fNumber = '';
		fInvoiceDate = today;
		fDueDate = '';
		fLines = [{ description: '', quantity: '1', unitPrice: '', vatRate: '0', expenseAccountId: 0 }];
	}

	function cancelComplete() {
		completingId = null;
		formError = '';
	}

	function addLine() {
		fLines = [
			...fLines,
			{ description: '', quantity: '1', unitPrice: '', vatRate: '0', expenseAccountId: 0 },
		];
	}

	function removeLine(idx: number) {
		fLines = fLines.filter((_, i) => i !== idx);
	}

	/** Σ TTC pleine précision en parité backend (`Σ line_total + Σ lineVatAmount`). */
	function sumTtc(lines: CompleteImportLineRequest[]): Big {
		let total = new Big(0);
		for (const l of lines) {
			let lineTotal: Big;
			try {
				lineTotal = new Big(l.quantity || '0').times(l.unitPrice || '0');
			} catch {
				continue; // saisie incomplète → ignorée pour le live total
			}
			total = total.plus(lineTotal);
			try {
				total = total.plus(lineVatAmount(lineTotal.toString(), l.vatRate || '0'));
			} catch {
				// taux non numérique → TVA ignorée pour le live total
			}
		}
		return total;
	}

	/** Cible montant du QR (string) de la facture en cours de complétion. */
	function targetAmount(id: number | null): string | null {
		const row = toComplete.find((r) => r.id === id);
		return row?.amount ?? null;
	}

	/** `true` si le montant QR porte des sous-centimes (>2 décimales non nulles). */
	function hasSubCentime(amount: string | null): boolean {
		if (!amount) return false;
		try {
			const a = new Big(amount);
			return !a.round(2, Big.roundHalfUp).eq(a);
		} catch {
			return false;
		}
	}

	/** Indicateur visuel : `Σ TTC` diffère-t-il du montant cible (pleine précision) ? */
	function amountMismatch(lines: CompleteImportLineRequest[], target: string | null): boolean {
		if (!target) return false;
		try {
			return !sumTtc(lines).eq(new Big(target));
		} catch {
			return false;
		}
	}

	/**
	 * Invalidité **structurelle** → bouton Valider désactivé (M3). Couvre :
	 * fournisseur manquant, date vide, ≥1 ligne, et par ligne : description non
	 * vide, compte de charge sélectionné (≠ 0), quantité et PU des montants
	 * valides. Le **montant total** (réconciliation QR) n'en fait JAMAIS partie
	 * (DC-d3 : autorité backend `AMOUNT_MISMATCH`).
	 */
	function structurallyInvalid(): boolean {
		if (!fContactId) return true;
		if (!fInvoiceDate.trim()) return true;
		if (fLines.length === 0) return true;
		return fLines.some(
			(l) =>
				!l.description.trim() ||
				l.expenseAccountId === 0 ||
				!isValidAmount(l.quantity) ||
				!isValidAmount(l.unitPrice),
		);
	}

	async function submitComplete(id: number) {
		formError = '';
		if (structurallyInvalid() || fContactId === null) {
			formError = i18nMsg(
				'imported-supplier-invoices-err-form',
				'Vérifiez le fournisseur, la date et chaque ligne (description, montants, compte de charge).',
			);
			return;
		}
		const contactId = fContactId;
		saving = true;
		try {
			const created = await completeImport(id, {
				contactId,
				invoiceDate: fInvoiceDate,
				supplierInvoiceNumber: fNumber.trim() || null,
				dueDate: fDueDate || null,
				lines: fLines.map((l) => ({
					description: l.description.trim(),
					quantity: l.quantity || '1',
					unitPrice: l.unitPrice || '0',
					vatRate: l.vatRate || '0',
					expenseAccountId: Number(l.expenseAccountId),
				})),
			});
			toComplete = toComplete.filter((r) => r.id !== id);
			// Ne fermer le formulaire que s'il s'agit toujours de la ligne soumise
			// (EC3 : éviter de fermer/polluer un autre formulaire ouvert entre-temps).
			if (completingId === id) completingId = null;
			notifySuccess(
				i18nMsg('imported-supplier-invoices-completed', 'Facture créée.'),
				i18nMsg('imported-supplier-invoices-completed-hint', 'Facture #{$id} enregistrée.', {
					id: created.id,
				}),
			);
		} catch (err) {
			formError = completeErrorLabel(err);
		} finally {
			saving = false;
		}
	}

	function completeErrorLabel(err: unknown): string {
		if (!isApiError(err)) return i18nMsg('imported-supplier-invoices-err-generic', 'Erreur inattendue.');
		switch (err.code) {
			case 'CURRENCY_NOT_SUPPORTED':
				return i18nMsg(
					'imported-supplier-invoices-err-currency',
					'Devise non supportée (CHF uniquement en v0.4).',
				);
			case 'IBAN_REFERENCE_MISMATCH':
				return i18nMsg(
					'imported-supplier-invoices-err-iban-ref',
					'Incohérence entre l’IBAN et la référence QRR.',
				);
			case 'AMOUNT_MISMATCH': {
				const d = err.details as { actual?: string; expected?: string } | undefined;
				return i18nMsg(
					'imported-supplier-invoices-err-amount',
					'Le total des lignes ({$actual}) ne correspond pas au montant du QR ({$expected}).',
					{ actual: d?.actual ?? '?', expected: d?.expected ?? '?' },
				);
			}
			case 'IMPORT_NOT_PENDING_COMPLETION':
				return i18nMsg(
					'imported-supplier-invoices-err-not-pending',
					'Cette facture a déjà été complétée ou écartée.',
				);
			case 'IMPORTED_INVOICE_NOT_FOUND':
				return i18nMsg('imported-supplier-invoices-err-not-found', 'Facture importée introuvable.');
			case 'FISCAL_YEAR_INVALID':
				return i18nMsg(
					'imported-supplier-invoices-err-fiscal-year',
					'Aucun exercice ouvert ne couvre cette date.',
				);
			default:
				return err.message;
		}
	}

	async function discard(row: ImportedSupplierInvoice) {
		if (
			!confirm(
				i18nMsg(
					'imported-supplier-invoices-discard-confirm',
					'Écarter cette facture importée ? Le fichier justificatif reste conservé.',
				),
			)
		) {
			return;
		}
		try {
			await discardImport(row.id);
			toComplete = toComplete.filter((r) => r.id !== row.id);
			if (completingId === row.id) completingId = null;
			notifySuccess(i18nMsg('imported-supplier-invoices-discarded', 'Facture écartée.'));
		} catch (err) {
			if (isApiError(err) && err.code === 'IMPORT_NOT_PENDING_COMPLETION') {
				notifyWarning(
					i18nMsg(
						'imported-supplier-invoices-discard-conflict',
						'Cette facture a déjà été complétée ou écartée par une autre session.',
					),
				);
				await safeReloadList();
			} else if (isApiError(err) && err.code === 'IMPORTED_INVOICE_NOT_FOUND') {
				notifyWarning(i18nMsg('imported-supplier-invoices-err-not-found', 'Facture importée introuvable.'));
				await safeReloadList();
			} else {
				notifyError(
					(isApiError(err) && err.message) ||
						i18nMsg('imported-supplier-invoices-discard-failed', 'Impossible d’écarter la facture.'),
				);
			}
		}
	}

	async function downloadJustif(row: ImportedSupplierInvoice) {
		try {
			await downloadImportedSourceDocument(row.id);
		} catch (err) {
			if (isApiError(err) && err.code === 'SOURCE_DOCUMENT_GONE') {
				notifyWarning(
					i18nMsg('imported-supplier-invoices-doc-gone', 'Le justificatif n’a pas été restauré.'),
				);
			} else if (isApiError(err) && err.code === 'IMPORTED_INVOICE_NOT_FOUND') {
				notifyWarning(i18nMsg('imported-supplier-invoices-err-not-found', 'Facture importée introuvable.'));
			} else {
				notifyError(
					(isApiError(err) && err.message) ||
						i18nMsg('imported-supplier-invoices-doc-failed', 'Téléchargement impossible.'),
				);
			}
		}
	}

	function fmt(amount: string | null): string {
		if (!amount) return '—';
		try {
			return formatSwissAmount(new Big(amount));
		} catch {
			return amount;
		}
	}
</script>

<svelte:head>
	<title>{i18nMsg('imported-supplier-invoices-title', 'Importer des factures')} — Kesh</title>
</svelte:head>

<div class="mb-6 flex items-center justify-between">
	<h1 class="text-2xl font-semibold">
		{i18nMsg('imported-supplier-invoices-title', 'Importer des factures')}
	</h1>
	<button
		class="rounded bg-primary px-4 py-2 text-sm text-primary-foreground disabled:opacity-60"
		data-testid="inbox-import-trigger"
		onclick={runImport}
		disabled={importing}
	>
		{importing
			? i18nMsg('imported-supplier-invoices-importing', 'Import en cours…')
			: i18nMsg('imported-supplier-invoices-import', 'Importer le dossier')}
	</button>
</div>

{#if report}
	<div class="mb-6 rounded border border-border p-4" data-testid="inbox-import-report">
		<p class="text-sm font-medium">
			{i18nMsg('imported-supplier-invoices-report-accepted', '{$n} facture(s) importée(s).', {
				n: report.accepted.length,
			})}
		</p>
		{#if report.failed.length > 0}
			<div class="mt-3">
				<p class="text-sm font-medium text-destructive">
					{i18nMsg('imported-supplier-invoices-report-failed', '{$n} échec(s) :', {
						n: report.failed.length,
					})}
				</p>
				<ul class="mt-1 space-y-1 text-sm">
					{#each report.failed as f, fi (fi)}
						<li data-testid="inbox-import-failed-row">
							<span class="font-mono">{f.fileName}</span> — {importErrorLabel(f.errorCode)}
						</li>
					{/each}
				</ul>
			</div>
		{/if}
		{#if report.warnings.length > 0}
			<ul class="mt-3 space-y-1 text-sm text-text-muted">
				{#each report.warnings as w, wi (wi)}
					<li>⚠️ {w}</li>
				{/each}
			</ul>
		{/if}
	</div>
{/if}

<h2 class="mb-3 text-lg font-medium">
	{i18nMsg('imported-supplier-invoices-to-complete', 'Factures à compléter')}
</h2>

{#if loading}
	<p class="text-sm text-text-muted">{i18nMsg('common-loading', 'Chargement…')}</p>
{:else if errorMsg}
	<p class="text-sm text-destructive">{errorMsg}</p>
{:else if toComplete.length === 0}
	<p class="text-sm text-text-muted" data-testid="imported-empty">
		{i18nMsg('imported-supplier-invoices-empty', 'Aucune facture à compléter.')}
	</p>
{:else}
	<ul class="space-y-4" data-testid="imported-list">
		{#each toComplete as row (row.id)}
			<li class="rounded border border-border p-4" data-testid="imported-row">
				<div class="flex flex-wrap items-start justify-between gap-3">
					<div class="text-sm">
						<div class="font-medium">{row.creditorName}</div>
						<div class="text-text-muted">{row.creditorIban}</div>
						<div class="text-text-muted">
							{i18nMsg('imported-supplier-invoices-amount', 'Montant')}: {fmt(row.amount)}
							{row.currency}
							{#if row.referenceValue}
								· {i18nMsg('imported-supplier-invoices-reference', 'Réf.')}: <span class="font-mono"
									>{row.referenceValue}</span
								>
							{/if}
						</div>
					</div>
					<div class="flex gap-2">
						<button
							class="rounded bg-primary px-3 py-1 text-sm text-primary-foreground disabled:opacity-60"
							data-testid="imported-complete-open"
							disabled={saving}
							onclick={() => openComplete(row)}>{i18nMsg('imported-supplier-invoices-complete', 'Compléter')}</button
						>
						<button
							class="rounded border px-3 py-1 text-sm disabled:opacity-60"
							data-testid="imported-discard"
							disabled={saving}
							onclick={() => discard(row)}>{i18nMsg('imported-supplier-invoices-discard', 'Écarter')}</button
						>
						<button
							class="rounded border px-3 py-1 text-sm text-primary"
							data-testid="imported-download"
							onclick={() => downloadJustif(row)}
							>{i18nMsg('imported-supplier-invoices-view-doc', 'Voir le justificatif')}</button
						>
					</div>
				</div>

				{#if completingId === row.id}
					{@const target = targetAmount(row.id)}
					<form
						class="mt-4 space-y-4 border-t border-border pt-4"
						data-testid="imported-complete-form"
						onsubmit={(e) => {
							e.preventDefault();
							submitComplete(row.id);
						}}
					>
						{#if hasSubCentime(target)}
							<p class="rounded border border-warning bg-warning-soft p-2 text-sm">
								{i18nMsg(
									'imported-supplier-invoices-subcentime',
									'Le montant du QR ({$amount}) contient des sous-centimes — impossible à atteindre par des lignes centime-exactes. Recommandation : écarter cette facture.',
									{ amount: target ?? '' },
								)}
							</p>
						{/if}

						<div class="grid grid-cols-2 gap-4">
							<label class="block text-sm">
								{i18nMsg('imported-supplier-invoices-field-supplier', 'Fournisseur')}
								<select class="mt-1 w-full rounded border px-2 py-1" bind:value={fContactId} data-testid="imported-supplier-select">
									<option value={null}>—</option>
									{#each suppliers as s (s.id)}
										<option value={s.id}>{s.name}</option>
									{/each}
								</select>
							</label>
							<label class="block text-sm">
								{i18nMsg('imported-supplier-invoices-field-number', 'N° facture fournisseur')}
								<input class="mt-1 w-full rounded border px-2 py-1" bind:value={fNumber} />
							</label>
							<label class="block text-sm">
								{i18nMsg('imported-supplier-invoices-field-date', 'Date de facture')}
								<input
									type="date"
									class="mt-1 w-full rounded border px-2 py-1"
									bind:value={fInvoiceDate}
								/>
							</label>
							<label class="block text-sm">
								{i18nMsg('imported-supplier-invoices-field-due', 'Échéance')}
								<input type="date" class="mt-1 w-full rounded border px-2 py-1" bind:value={fDueDate} />
							</label>
						</div>

						<div>
							<div class="mb-2 text-sm font-medium">
								{i18nMsg('imported-supplier-invoices-lines', 'Lignes')}
							</div>
							{#each fLines as line, idx (idx)}
								<div class="mb-2 grid grid-cols-12 gap-2">
									<input
										class="col-span-4 rounded border px-2 py-1 text-sm"
										placeholder={i18nMsg('imported-supplier-invoices-line-desc', 'Description')}
										bind:value={line.description}
									/>
									<input
										class="col-span-1 rounded border px-2 py-1 text-sm"
										inputmode="decimal"
										placeholder={i18nMsg('imported-supplier-invoices-line-qty', 'Qté')}
										bind:value={line.quantity}
									/>
									<input
										class="col-span-2 rounded border px-2 py-1 text-sm"
										inputmode="decimal"
										placeholder={i18nMsg('imported-supplier-invoices-line-ht', 'PU HT')}
										bind:value={line.unitPrice}
									/>
									<select class="col-span-2 rounded border px-2 py-1 text-sm" bind:value={line.vatRate}>
										<option value="0">0%</option>
										{#each vatRates as r (r.id)}
											<option value={r.rate}>{r.rate}%</option>
										{/each}
									</select>
									<select
										class="col-span-2 rounded border px-2 py-1 text-sm"
										bind:value={line.expenseAccountId}
									>
										<option value={0}>{i18nMsg('imported-supplier-invoices-line-account', 'Compte')}</option>
										{#each expenseAccounts as a (a.id)}
											<option value={a.id}>{a.number} {a.name}</option>
										{/each}
									</select>
									<button
										type="button"
										class="col-span-1 text-destructive"
										onclick={() => removeLine(idx)}
										disabled={fLines.length === 1}>✕</button
									>
								</div>
							{/each}
							<button type="button" class="text-sm text-primary" onclick={addLine}>
								+ {i18nMsg('imported-supplier-invoices-add-line', 'Ajouter une ligne')}
							</button>
						</div>

						<div class="text-sm" data-testid="imported-sum-ttc">
							{i18nMsg('imported-supplier-invoices-sum-ttc', 'Total TTC des lignes')}:
							<span class:text-destructive={amountMismatch(fLines, target)}>
								{formatSwissAmount(sumTtc(fLines))}
							</span>
							{#if target}
								/ {i18nMsg('imported-supplier-invoices-target', 'cible QR')}: {fmt(target)}
								{#if amountMismatch(fLines, target)}
									<span class="text-destructive"
										>— {i18nMsg('imported-supplier-invoices-mismatch', 'écart à corriger')}</span
									>
								{/if}
							{/if}
						</div>

						{#if formError}
							<p class="text-sm text-destructive" data-testid="imported-form-error">{formError}</p>
						{/if}

						<div class="flex gap-2">
							<button
								type="submit"
								class="rounded bg-primary px-4 py-2 text-sm text-primary-foreground disabled:opacity-60"
								data-testid="imported-complete-submit"
								disabled={saving || structurallyInvalid()}
							>
								{saving ? '…' : i18nMsg('imported-supplier-invoices-save', 'Valider la facture')}
							</button>
							<button type="button" class="rounded border px-4 py-2 text-sm" onclick={cancelComplete}>
								{i18nMsg('common-cancel', 'Annuler')}
							</button>
						</div>
					</form>
				{/if}
			</li>
		{/each}
	</ul>
{/if}
