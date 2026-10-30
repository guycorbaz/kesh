// Story v016-2b (#816) — tests Vitest du formulaire de saisie d'un relevé :
// aperçu périmé par une modification (AC-22, M15), cases transmises dans le
// corps (AC-22, M19), réponse tardive ignorée (AC-22, M29), raison « écart »
// après un aperçu réussi (AC-21, M14), ligne désignée après une ligne vide
// (AC-20, M25), refus générique d'un corps rejeté (AC-22).
//
// Patron : `BankImportUpload.test.ts` — mocks AVANT l'import du composant ;
// promesses retenues (`deferred`) pour figer une requête en vol.

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, fireEvent, cleanup } from '@testing-library/svelte';
import { tick } from 'svelte';

vi.mock('./bank-import.api', () => ({
	previewManualStatement: vi.fn(),
	createManualStatement: vi.fn(),
}));

vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_k: string, fallback: string, args?: Record<string, string | number>) =>
		args ? fallback.replace(/\{\s*\$(\w+)\s*\}/g, (_, n) => String(args[n] ?? '')) : fallback,
}));

import * as api from './bank-import.api';
import ManualStatementForm from './ManualStatementForm.svelte';
import type { ManualStatementPreviewResponse } from './bank-import.types';

const mockApi = vi.mocked(api);

const ACCOUNTS = [{ id: 3, bankName: 'Banque Exemple', description: null, iban: 'CH9300762011623852957' }];

function deferred<T>() {
	let resolve!: (v: T) => void;
	let reject!: (e: unknown) => void;
	const promise = new Promise<T>((res, rej) => {
		resolve = res;
		reject = rej;
	});
	return { promise, resolve, reject };
}

async function flush(): Promise<void> {
	for (let i = 0; i < 3; i++) {
		await new Promise((r) => setTimeout(r, 0));
		await tick();
	}
}

function previewOk(o: Partial<ManualStatementPreviewResponse['warnings']> = {}): ManualStatementPreviewResponse {
	return {
		totalCredits: '50.00',
		totalDebits: '130.00',
		computedClosingBalance: '920.00',
		transactionCount: 2,
		warnings: {
			duplicateFile: null,
			duplicateLines: [],
			openingBalanceGap: null,
			overlappingStatements: [],
			...o,
		},
	};
}

const q = (c: HTMLElement, id: string) => c.querySelector<HTMLElement>(`[data-testid="${id}"]`);
const qa = (c: HTMLElement, id: string) => Array.from(c.querySelectorAll<HTMLElement>(`[data-testid="${id}"]`));

async function type(el: Element | null, value: string): Promise<void> {
	if (!el) throw new Error('champ introuvable');
	await fireEvent.input(el, { target: { value } });
}

/** Remplit l'entête et les lignes (une ligne par tableau [date, libellé, débit, crédit]). */
async function fill(
	c: HTMLElement,
	o: { opening?: string; closing?: string; lines?: [string, string, string, string][] } = {},
): Promise<void> {
	const select = q(c, 'bank-import-manual-account') as HTMLSelectElement;
	select.value = '3';
	await fireEvent.change(select);
	await type(q(c, 'bank-import-manual-period-from'), '2026-09-01');
	await type(q(c, 'bank-import-manual-period-to'), '2026-09-30');
	await type(q(c, 'bank-import-manual-opening-balance'), o.opening ?? '1000');
	await type(q(c, 'bank-import-manual-closing-balance'), o.closing ?? '920');
	const lines = o.lines ?? [
		['2026-09-03', 'Assurance', '130', ''],
		['2026-09-15', 'Virement reçu', '', '50'],
	];
	for (let i = 0; i < lines.length; i++) {
		if (i > 0) await fireEvent.click(q(c, 'bank-import-manual-add-line')!);
		const [date, details, debit, credit] = lines[i];
		await type(qa(c, 'bank-import-manual-line-booking-date')[i], date);
		await type(qa(c, 'bank-import-manual-line-details')[i], details);
		await type(qa(c, 'bank-import-manual-line-debit')[i], debit);
		await type(qa(c, 'bank-import-manual-line-credit')[i], credit);
	}
	await flush();
}

const saveDisabled = (c: HTMLElement) => (q(c, 'bank-import-manual-save') as HTMLButtonElement).disabled;

beforeEach(() => {
	mockApi.previewManualStatement.mockReset();
	mockApi.createManualStatement.mockReset();
});

afterEach(() => cleanup());

describe('ManualStatementForm', () => {
	it('s’ouvre avec une ligne vide ; écart affiché en continu, « Enregistrer » inactif avant l’aperçu', async () => {
		const { container: c } = render(ManualStatementForm, { props: { bankAccounts: ACCOUNTS } });
		await flush();
		expect(qa(c, 'bank-import-manual-line')).toHaveLength(1);
		expect(q(c, 'bank-import-manual-gap')).toBeNull();
		await fill(c, { closing: '900' });
		expect(q(c, 'bank-import-manual-gap')?.textContent).toContain('-20.00');
		expect(saveDisabled(c)).toBe(true);
		expect((q(c, 'bank-import-manual-verify') as HTMLButtonElement).disabled).toBe(false);
	});

	it('M14 — un aperçu réussi sur un relevé déséquilibré : « Enregistrer » inactif, raison « écart »', async () => {
		mockApi.previewManualStatement.mockResolvedValue(previewOk());
		const { container: c } = render(ManualStatementForm, { props: { bankAccounts: ACCOUNTS } });
		await fill(c, { closing: '900' });
		await fireEvent.click(q(c, 'bank-import-manual-verify')!);
		await flush();
		expect(saveDisabled(c)).toBe(true);
		expect(q(c, 'bank-import-manual-block-reason')?.dataset.reason).toBe('gap');
	});

	it('M15 — une modification après l’aperçu le périme : cases décochées, « Enregistrer » inactif', async () => {
		mockApi.previewManualStatement.mockResolvedValue(
			previewOk({ duplicateFile: { existingImportId: 1, existingFilename: 'f', existingImportedAt: 'x' } }),
		);
		const { container: c } = render(ManualStatementForm, { props: { bankAccounts: ACCOUNTS } });
		await fill(c);
		await fireEvent.click(q(c, 'bank-import-manual-verify')!);
		await flush();
		const box = q(c, 'bank-import-manual-confirm-duplicate-file') as HTMLInputElement;
		await fireEvent.click(box);
		await flush();
		// Cocher une case ne périme pas l'aperçu.
		expect(saveDisabled(c)).toBe(false);
		await type(q(c, 'bank-import-manual-statement-number'), '9/2026');
		await flush();
		expect(saveDisabled(c)).toBe(true);
		expect(q(c, 'bank-import-manual-block-reason')?.dataset.reason).toBe('preview');
		expect(q(c, 'bank-import-manual-preview-stale')).not.toBeNull();
		expect(q(c, 'bank-import-manual-confirm-duplicate-file')).toBeNull();
	});

	it('M19 — les cases cochées partent dans le corps de la création', async () => {
		mockApi.previewManualStatement.mockResolvedValue(
			previewOk({
				duplicateFile: { existingImportId: 1, existingFilename: 'f', existingImportedAt: 'x' },
				duplicateLines: [{ line: 1, existingTransactionId: 7, key: 'k' }],
				openingBalanceGap: {
					expected: '990.00',
					entered: '1000.00',
					previousImportId: 4,
					previousPeriodTo: '2026-08-31',
				},
			}),
		);
		mockApi.createManualStatement.mockResolvedValue({ id: 5 } as never);
		const onSaved = vi.fn();
		const { container: c } = render(ManualStatementForm, { props: { bankAccounts: ACCOUNTS, onSaved } });
		await fill(c);
		await fireEvent.click(q(c, 'bank-import-manual-verify')!);
		await flush();
		expect(saveDisabled(c)).toBe(true);
		expect(q(c, 'bank-import-manual-block-reason')?.dataset.reason).toBe('confirmations');
		await fireEvent.click(q(c, 'bank-import-manual-confirm-duplicate-file')!);
		await fireEvent.click(q(c, 'bank-import-manual-confirm-duplicate-lines')!);
		await fireEvent.click(q(c, 'bank-import-manual-confirm-opening-gap')!);
		await flush();
		expect(saveDisabled(c)).toBe(false);
		await fireEvent.click(q(c, 'bank-import-manual-save')!);
		await flush();
		expect(mockApi.createManualStatement).toHaveBeenCalledTimes(1);
		const body = mockApi.createManualStatement.mock.calls[0][0];
		expect(body.confirmDuplicateFile).toBe(true);
		expect(body.confirmDuplicateLines).toBe(true);
		expect(body.confirmOpeningBalanceGap).toBe(true);
		expect(body.openingBalance).toBe('1000.00');
		expect(onSaved).toHaveBeenCalledWith({ id: 5 });
	});

	it('M29 — une réponse d’aperçu arrivée après une modification est ignorée', async () => {
		const d = deferred<ManualStatementPreviewResponse>();
		mockApi.previewManualStatement.mockReturnValue(d.promise);
		const { container: c } = render(ManualStatementForm, { props: { bankAccounts: ACCOUNTS } });
		await fill(c);
		await fireEvent.click(q(c, 'bank-import-manual-verify')!);
		await flush();
		// Le formulaire est figé pendant l'envoi ; la garde de l'instantané tient même si
		// une modification passe malgré tout.
		await type(q(c, 'bank-import-manual-statement-number'), '9/2026');
		d.resolve(previewOk());
		await flush();
		expect(q(c, 'bank-import-manual-preview-ok')).toBeNull();
		expect(saveDisabled(c)).toBe(true);
		expect(q(c, 'bank-import-manual-block-reason')?.dataset.reason).toBe('preview');
	});

	it('M25 — une faute du serveur sur la ligne qui suit une ligne vide désigne CETTE ligne', async () => {
		mockApi.previewManualStatement.mockRejectedValue({
			status: 400,
			code: 'BANK_STATEMENT_INVALID',
			message: 'Le relevé saisi contient une valeur absente ou invalide.',
			details: { line: 2, field: 'details', reason: 'too_long' },
		});
		const { container: c } = render(ManualStatementForm, { props: { bankAccounts: ACCOUNTS } });
		await fill(c, {
			lines: [
				['2026-09-03', 'Assurance', '130', ''],
				['', '', '', ''],
				['2026-09-15', 'Virement reçu', '', '50'],
			],
		});
		await fireEvent.click(q(c, 'bank-import-manual-verify')!);
		await flush();
		const rows = qa(c, 'bank-import-manual-line');
		expect(rows.map((r) => r.dataset.error)).toEqual(['false', 'false', 'true']);
		expect(q(c, 'bank-import-manual-error')?.textContent).toContain('Ligne 3');
		expect(saveDisabled(c)).toBe(true);
	});

	it('un 400 VALIDATION_ERROR sans field : message générique, aucune ligne désignée, aperçu non réussi', async () => {
		mockApi.previewManualStatement.mockRejectedValue({
			status: 400,
			code: 'VALIDATION_ERROR',
			message: 'Corps de requête illisible',
		});
		const { container: c } = render(ManualStatementForm, { props: { bankAccounts: ACCOUNTS } });
		await fill(c);
		await fireEvent.click(q(c, 'bank-import-manual-verify')!);
		await flush();
		const err = q(c, 'bank-import-manual-error')?.textContent ?? '';
		expect(err).toContain('trop volumineux ou illisible');
		expect(err).not.toContain('Corps de requête illisible');
		expect(qa(c, 'bank-import-manual-line').every((r) => r.dataset.error === 'false')).toBe(true);
		expect(saveDisabled(c)).toBe(true);
		expect(q(c, 'bank-import-manual-block-reason')?.dataset.reason).toBe('preview');
	});
});
