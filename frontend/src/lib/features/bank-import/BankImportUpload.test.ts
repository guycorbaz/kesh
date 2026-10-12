// Story 27-3 (#626) — tests Vitest de l'écran d'import : l'aperçu se recalcule
// au changement de profil, la confirmation part de l'aperçu affiché (instantané
// + empreinte), le refus 409 BANK_IMPORT_PREVIEW_STALE relance l'aperçu, et les
// libellés de l'écran passent par le catalogue (AC17).
//
// Pattern : mocks AVANT l'import du composant (hoisting Vitest), patron
// `BankProfileForm.test.ts`. Les promesses « retenues » (`deferred`) figent une
// requête en vol pour observer l'état de l'écran pendant ce temps.

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, fireEvent, cleanup } from '@testing-library/svelte';
import { tick } from 'svelte';

// Mode du mock i18n : repli (texte fr) par défaut, ou la CLÉ (V13).
const i18nMode = vi.hoisted(() => ({ keys: false }));

vi.mock('./bank-import.api', () => ({
	previewBankImport: vi.fn(),
	createBankImport: vi.fn(),
}));

vi.mock('./bank-profile.api', () => ({
	listBankProfiles: vi.fn(),
}));

vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (key: string, fallback: string) => (i18nMode.keys ? key : fallback),
}));

import * as api from './bank-import.api';
import * as profileApi from './bank-profile.api';
import BankImportUpload from './BankImportUpload.svelte';
import BankImportList from './BankImportList.svelte';
import type {
	BankImportPreviewResponse,
	BankImportResponse,
	PreviewWarnings,
} from './bank-import.types';
import type { BankProfile } from './bank-profile.types';
import type { ApiError } from '$lib/shared/types/api';

const mockApi = vi.mocked(api);
const mockProfileApi = vi.mocked(profileApi);

const A = 11;
const B = 12;
const C = 13;

function profile(id: number, bankName: string): BankProfile {
	return {
		id,
		bankName,
		filenamePattern: null,
		columnMapping: { date: 0, amount: 1 },
		dateFormat: '%Y-%m-%d',
		decimalSeparator: '.',
		fieldSeparator: ';',
		encoding: null,
		headerRowCount: 1,
		debitSign: 'negative',
		createdAt: '2026-01-01T00:00:00',
		updatedAt: '2026-01-01T00:00:00',
	};
}

const ACCOUNTS = [
	{ id: 1, bankName: 'UBS', description: null, iban: 'CH4431999123000889012' },
	{ id: 2, bankName: 'PostFinance', description: null, iban: 'CH9300762011623852957' },
];

const FILE = new File(['date;montant\n2026-01-01;1.00\n'], 'releve.csv', { type: 'text/csv' });

const fp = (c: string) => c.repeat(64);

type PreviewOpts = {
	tag?: string;
	profileId?: number | null;
	autoMatched?: boolean;
	fingerprint?: string;
	warnings?: Partial<PreviewWarnings>;
	ignored?: number;
};

/** Réponse d'aperçu synthétique ; `tag` marque la transaction (`tx-<tag>`). */
function previewResp(o: PreviewOpts = {}): BankImportPreviewResponse {
	const profileId = o.profileId === undefined ? A : o.profileId;
	return {
		fileHash: 'h'.repeat(64),
		filename: 'releve.csv',
		sourceFormat: profileId === null ? 'camt.053.001.04' : 'CSV',
		selectedStatement: {
			statementId: null,
			accountIban: 'CH4431999123000889012',
			currency: 'CHF',
			periodFrom: '2026-01-01',
			periodTo: '2026-01-31',
			openingBalance: '10.00',
			closingBalance: '11.00',
		},
		ignoredStatements: Array.from({ length: o.ignored ?? 0 }, () => ({
			statementId: null,
			accountIban: 'CH0000000000000000000',
		})),
		warnings: { duplicateLines: [], ...o.warnings },
		transactions: [
			{
				bookingDate: '2026-01-01',
				valueDate: null,
				amount: '1.00',
				currency: 'CHF',
				reference: null,
				details: `tx-${o.tag ?? 'A'}`,
				counterpartyIban: null,
				counterpartyName: null,
			},
		],
		csvProfileMatch:
			profileId === null
				? undefined
				: { profileId, profileName: `P${profileId}`, autoMatched: o.autoMatched ?? false },
		previewFingerprint: o.fingerprint ?? fp('a'),
	};
}

function apiError(code: string, status: number, details?: Record<string, unknown>): ApiError {
	return { code, status, message: `message ${code}`, details };
}

const DUPLICATE_FILE = {
	duplicateFile: {
		existingImportId: 5,
		existingFilename: 'ancien.csv',
		existingImportedAt: '2026-01-02T00:00:00Z',
	},
};

const SIGN_REFUSAL = apiError('BANK_CSV_AMOUNT_SIGN_MISMATCH', 422, {
	lines: [{ line: 3, code: 'AMOUNT_SIGN_MISMATCH', value: '-1.00', messageI18nKey: 'k' }],
	totalErrors: 1,
	truncated: false,
});

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

function q(c: HTMLElement, testid: string): HTMLElement | null {
	return c.querySelector(`[data-testid="${testid}"]`);
}

function get<T extends HTMLElement = HTMLElement>(c: HTMLElement, testid: string): T {
	const el = q(c, testid);
	expect(el, testid).not.toBeNull();
	return el as T;
}

async function mount(onSuccess = vi.fn()) {
	const r = render(BankImportUpload, { props: { bankAccounts: ACCOUNTS, onSuccess } });
	await flush(); // onMount : liste des profils
	return { c: r.container, onSuccess };
}

async function chooseAccount(c: HTMLElement, id: number): Promise<void> {
	const s = get<HTMLSelectElement>(c, 'bank-account-select');
	s.value = String(id);
	await fireEvent.change(s);
	await flush();
}

async function chooseProfile(c: HTMLElement, id: number | null): Promise<void> {
	const s = get<HTMLSelectElement>(c, 'bank-profile-select');
	s.value = id === null ? '' : String(id);
	await fireEvent.change(s);
	await flush();
}

/** Dépôt par « cliquer pour parcourir » : `files` posé, puis `change`. */
async function chooseFile(c: HTMLElement, file: File = FILE): Promise<void> {
	const input = get<HTMLInputElement>(c, 'bank-import-file-input');
	Object.defineProperty(input, 'files', { value: [file], configurable: true });
	await fireEvent.change(input);
	await flush();
}

async function confirm(c: HTMLElement): Promise<void> {
	await fireEvent.click(get(c, 'bank-import-confirm'));
	await flush();
}

/** Aperçu affiché pour le profil A (choisi avant le dépôt). */
async function previewWithA(c: HTMLElement, o: PreviewOpts = {}): Promise<void> {
	await chooseAccount(c, 1);
	mockApi.previewBankImport.mockResolvedValueOnce(previewResp({ tag: 'A', profileId: A, ...o }));
	const calls = mockApi.previewBankImport.mock.calls.length;
	await chooseProfile(c, A);
	expect(mockApi.previewBankImport.mock.calls.length, 'aucun fichier : pas d’aperçu').toBe(calls);
	await chooseFile(c);
	// Revue P1, E-4 : le profil choisi AVANT le dépôt part avec le premier aperçu.
	expect(profileFlag(calls), 'profil choisi avant le dépôt').toBe(A);
	expect(q(c, 'bank-import-preview'), 'montage : aperçu A affiché').not.toBeNull();
}

function profileFlag(callIndex: number): number | undefined {
	return mockApi.previewBankImport.mock.calls[callIndex][2]?.bankProfileId;
}

beforeEach(() => {
	mockApi.previewBankImport.mockReset();
	mockApi.createBankImport.mockReset();
	mockProfileApi.listBankProfiles.mockReset();
	mockProfileApi.listBankProfiles.mockResolvedValue({
		items: [profile(A, 'Profil A'), profile(B, 'Profil B'), profile(C, 'Profil C')],
		total: 3,
		page: 1,
		perPage: 50,
	} as never);
});

afterEach(() => {
	cleanup();
	i18nMode.keys = false;
});

describe('BankImportUpload — changer de profil recalcule l’aperçu (Story 27-3)', () => {
	it('V1 — A puis B : aperçu de B, confirmation avec B et son empreinte, aucune annotation « auto-détecté »', async () => {
		const { c } = await mount();
		await previewWithA(c, { fingerprint: fp('a') });
		mockApi.previewBankImport.mockResolvedValueOnce(
			previewResp({ tag: 'B', profileId: B, fingerprint: fp('b') }),
		);
		await chooseProfile(c, B);

		expect(mockApi.previewBankImport).toHaveBeenCalledTimes(2);
		expect(mockApi.previewBankImport.mock.calls[1][0]).toBe(FILE);
		expect(profileFlag(1)).toBe(B);
		const shown = get(c, 'bank-import-preview').textContent ?? '';
		expect(shown).toContain('tx-B');
		expect(shown).not.toContain('tx-A');
		const select = get<HTMLSelectElement>(c, 'bank-profile-select');
		expect(select.textContent).not.toContain('auto-détecté');

		mockApi.createBankImport.mockResolvedValueOnce({ id: 1 } as BankImportResponse);
		await confirm(c);
		expect(mockApi.createBankImport).toHaveBeenCalledTimes(1);
		const [file, account, flags] = mockApi.createBankImport.mock.calls[0];
		expect(file).toBe(FILE);
		expect(account).toBe(1);
		expect(flags?.bankProfileId).toBe(B);
		expect(flags?.previewFingerprint).toBe(fp('b'));
		// Revue de code P2, L1 : ni le changement de profil ni le succès ne relisent la liste.
		expect(mockProfileApi.listBankProfiles, 'aucune relecture hors 409').toHaveBeenCalledTimes(1);
	});

	it('V2 — pendant le recalcul : ni aperçu, ni Confirmer, ni zone de dépôt ; indicateur de recalcul ; sélecteurs actifs', async () => {
		const { c } = await mount();
		await previewWithA(c);
		const pending = deferred<BankImportPreviewResponse>();
		mockApi.previewBankImport.mockReturnValueOnce(pending.promise);
		await chooseProfile(c, B);

		expect(q(c, 'bank-import-preview')).toBeNull();
		expect(q(c, 'bank-import-confirm')).toBeNull();
		expect(q(c, 'bank-import-drop-zone')).toBeNull();
		const loading = get(c, 'bank-import-loading');
		expect(loading.getAttribute('role')).toBe('status');
		expect(loading.textContent?.trim()).toBe('Recalcul de la prévisualisation…');
		expect(get<HTMLSelectElement>(c, 'bank-account-select').disabled).toBe(false);
		expect(get<HTMLSelectElement>(c, 'bank-profile-select').disabled).toBe(false);

		pending.resolve(previewResp({ tag: 'B', profileId: B }));
		await flush();
		expect(q(c, 'bank-import-loading')).toBeNull();
		expect(get(c, 'bank-import-preview').textContent).toContain('tx-B');
	});

	it('V3 — A, B, C rapides, réponses A, C, B : seule la dernière demandée s’affiche', async () => {
		const { c } = await mount();
		await previewWithA(c);
		const dA = deferred<BankImportPreviewResponse>();
		const dB = deferred<BankImportPreviewResponse>();
		const dC = deferred<BankImportPreviewResponse>();
		mockApi.previewBankImport
			.mockReturnValueOnce(dA.promise)
			.mockReturnValueOnce(dB.promise)
			.mockReturnValueOnce(dC.promise);
		await chooseProfile(c, null); // « A » de la course : auto-détection
		await chooseProfile(c, B);
		await chooseProfile(c, C);
		expect(mockApi.previewBankImport).toHaveBeenCalledTimes(4);

		dA.resolve(previewResp({ tag: 'A' }));
		await flush();
		expect(q(c, 'bank-import-loading'), 'réponse périmée A : indicateur allumé').not.toBeNull();
		expect(q(c, 'bank-import-preview')).toBeNull();

		dC.resolve(previewResp({ tag: 'C', profileId: C }));
		await flush();
		expect(get(c, 'bank-import-preview').textContent).toContain('tx-C');
		expect(q(c, 'bank-import-loading')).toBeNull();

		dB.resolve(previewResp({ tag: 'B', profileId: B }));
		await flush();
		const shown = get(c, 'bank-import-preview').textContent ?? '';
		expect(shown).toContain('tx-C');
		expect(shown).not.toContain('tx-B');
	});

	it('V4 — Auto-détection + autoMatched : un seul aperçu, profil présélectionné et annoté', async () => {
		const { c } = await mount();
		await chooseAccount(c, 1);
		mockApi.previewBankImport.mockResolvedValueOnce(
			previewResp({ tag: 'A', profileId: B, autoMatched: true }),
		);
		await chooseFile(c);
		await flush();

		expect(mockApi.previewBankImport).toHaveBeenCalledTimes(1);
		expect(profileFlag(0)).toBeUndefined();
		const select = get<HTMLSelectElement>(c, 'bank-profile-select');
		expect(select.value).toBe(String(B));
		const optionB = select.querySelector(`option[value="${B}"]`);
		expect(optionB?.textContent).toContain('(auto-détecté)');
		const optionA = select.querySelector(`option[value="${A}"]`);
		expect(optionA?.textContent).not.toContain('auto-détecté');

		// Revue P1, E-2 (AC5) : la confirmation envoie le profil RÉSOLU par
		// l'aperçu (B), pas le profil demandé (aucun, Auto-détection).
		mockApi.createBankImport.mockResolvedValueOnce({ id: 1 } as BankImportResponse);
		await confirm(c);
		const flags = mockApi.createBankImport.mock.calls[0][2];
		expect(flags?.bankProfileId).toBe(B);
		expect(flags?.previewFingerprint).toBe(fp('a'));
	});

	it('V5 — aperçu en échec, puis autre profil : relancé avec le même fichier, erreur effacée', async () => {
		const { c } = await mount();
		await chooseAccount(c, 1);
		await chooseProfile(c, B);
		mockApi.previewBankImport.mockRejectedValueOnce(SIGN_REFUSAL);
		await chooseFile(c);
		expect(get(c, 'bank-import-error').getAttribute('data-error-code')).toBe(
			'BANK_CSV_AMOUNT_SIGN_MISMATCH',
		);
		expect(q(c, 'bank-import-preview')).toBeNull();

		const pending = deferred<BankImportPreviewResponse>();
		mockApi.previewBankImport.mockReturnValueOnce(pending.promise);
		await chooseProfile(c, A);
		expect(mockApi.previewBankImport).toHaveBeenCalledTimes(2);
		expect(mockApi.previewBankImport.mock.calls[1][0]).toBe(FILE);
		expect(profileFlag(1)).toBe(A);
		expect(q(c, 'bank-import-error'), 'erreur effacée dès le geste').toBeNull();
		pending.resolve(previewResp({ tag: 'A' }));
		await flush();
		expect(get(c, 'bank-import-preview').textContent).toContain('tx-A');
	});

	it('V7 — case cochée, puis autre profil : la case du nouvel aperçu est décochée', async () => {
		const { c } = await mount();
		await previewWithA(c, { warnings: DUPLICATE_FILE });
		const box = get<HTMLInputElement>(c, 'confirm-duplicate-file');
		await fireEvent.click(box);
		await flush();
		expect(get<HTMLInputElement>(c, 'confirm-duplicate-file').checked).toBe(true);
		expect(get<HTMLButtonElement>(c, 'bank-import-confirm').disabled).toBe(false);

		mockApi.previewBankImport.mockResolvedValueOnce(
			previewResp({ tag: 'B', profileId: B, warnings: DUPLICATE_FILE }),
		);
		await chooseProfile(c, B);
		expect(get<HTMLInputElement>(c, 'confirm-duplicate-file').checked).toBe(false);
		expect(get<HTMLButtonElement>(c, 'bank-import-confirm').disabled).toBe(true);
	});
});

describe('BankImportUpload — séquence, compte et confirmation (Story 27-3)', () => {
	it('V8 — changement de compte pendant un aperçu : indicateur éteint, zone de dépôt, réponse tardive ignorée', async () => {
		const { c } = await mount();
		await chooseAccount(c, 1);
		const pending = deferred<BankImportPreviewResponse>();
		mockApi.previewBankImport.mockReturnValueOnce(pending.promise);
		await chooseFile(c);
		expect(q(c, 'bank-import-loading')).not.toBeNull();

		await chooseAccount(c, 2);
		expect(q(c, 'bank-import-loading')).toBeNull();
		expect(q(c, 'bank-import-drop-zone')).not.toBeNull();

		pending.resolve(previewResp({ tag: 'A' }));
		await flush();
		expect(q(c, 'bank-import-preview')).toBeNull();
		expect(q(c, 'bank-import-error')).toBeNull();
		expect(q(c, 'bank-import-drop-zone')).not.toBeNull();
	});

	it('V8 (variante) — la réponse tardive est un rejet : ni erreur, ni indicateur', async () => {
		const { c } = await mount();
		await chooseAccount(c, 1);
		const pending = deferred<BankImportPreviewResponse>();
		mockApi.previewBankImport.mockReturnValueOnce(pending.promise);
		await chooseFile(c);
		await chooseAccount(c, 2);

		pending.reject(apiError('BANK_CSV_NO_PROFILE_MATCH', 404));
		await flush();
		expect(q(c, 'bank-import-error')).toBeNull();
		expect(q(c, 'bank-import-loading')).toBeNull();
		expect(q(c, 'bank-import-drop-zone')).not.toBeNull();
	});

	it('V9 — confirmation en vol après un recalcul : sélecteurs et Annuler désactivés, « Chargement… »', async () => {
		const { c } = await mount();
		await previewWithA(c);
		mockApi.previewBankImport.mockResolvedValueOnce(previewResp({ tag: 'B', profileId: B }));
		await chooseProfile(c, B); // previewReason = 'profile'
		const pending = deferred<BankImportResponse>();
		mockApi.createBankImport.mockReturnValueOnce(pending.promise);
		await confirm(c);

		expect(get<HTMLSelectElement>(c, 'bank-account-select').disabled).toBe(true);
		expect(get<HTMLSelectElement>(c, 'bank-profile-select').disabled).toBe(true);
		expect(get<HTMLButtonElement>(c, 'bank-import-cancel').disabled).toBe(true);
		// Revue P1, E-3 : garde anti-double-clic.
		expect(get<HTMLButtonElement>(c, 'bank-import-confirm').disabled).toBe(true);
		expect(get(c, 'bank-import-loading').textContent?.trim()).toBe('Chargement…');

		pending.resolve({ id: 1 } as BankImportResponse);
		await flush();
	});

	it('V14 — confirmation rejetée autrement qu’en 409 : l’écran n’est pas figé', async () => {
		// Revue P1, E-1 : seule la clause `finally` éteint `isConfirming` sur
		// cette voie.
		const { c, onSuccess } = await mount();
		await previewWithA(c);
		mockApi.createBankImport.mockRejectedValueOnce(apiError('BANK_IMPORT_DUPLICATE_FILE', 422));
		await confirm(c);

		expect(get(c, 'bank-import-error').getAttribute('data-error-code')).toBe(
			'BANK_IMPORT_DUPLICATE_FILE',
		);
		expect(q(c, 'bank-import-loading')).toBeNull();
		expect(get<HTMLSelectElement>(c, 'bank-account-select').disabled).toBe(false);
		expect(get<HTMLSelectElement>(c, 'bank-profile-select').disabled).toBe(false);
		expect(get<HTMLButtonElement>(c, 'bank-import-cancel').disabled).toBe(false);
		expect(get<HTMLButtonElement>(c, 'bank-import-confirm').disabled).toBe(false);
		expect(get(c, 'bank-import-preview').textContent).toContain('tx-A');
		expect(mockApi.previewBankImport, 'aucun recalcul hors 409').toHaveBeenCalledTimes(1);
		// Revue de code P2, L1 : la liste des profils n'est relue qu'après un 409.
		expect(mockProfileApi.listBankProfiles, 'aucune relecture hors 409').toHaveBeenCalledTimes(1);
		expect(onSuccess).not.toHaveBeenCalled();
	});

	it('V10 — CAMT : le sélecteur vaut A, la confirmation part sans bankProfileId (instantané)', async () => {
		const { c } = await mount();
		await chooseAccount(c, 1);
		await chooseProfile(c, A);
		mockApi.previewBankImport.mockResolvedValueOnce(previewResp({ tag: 'X', profileId: null }));
		await chooseFile(c);
		expect(get<HTMLSelectElement>(c, 'bank-profile-select').value).toBe(String(A));
		mockApi.createBankImport.mockResolvedValueOnce({ id: 1 } as BankImportResponse);
		await confirm(c);
		const flags = mockApi.createBankImport.mock.calls[0][2];
		expect(flags?.bankProfileId).toBeUndefined();
		expect(flags?.previewFingerprint).toBe(fp('a'));
	});

	it('V12 — choisir un fichier vide la valeur de l’input et lance l’aperçu', async () => {
		const { c } = await mount();
		await chooseAccount(c, 1);
		mockApi.previewBankImport.mockResolvedValueOnce(previewResp());
		const input = get<HTMLInputElement>(c, 'bank-import-file-input');
		const setValue = vi.fn();
		Object.defineProperty(input, 'value', { set: setValue, get: () => '', configurable: true });
		Object.defineProperty(input, 'files', { value: [FILE], configurable: true });
		await fireEvent.change(input);
		await flush();
		expect(setValue).toHaveBeenCalledWith('');
		expect(mockApi.previewBankImport.mock.calls[0][0]).toBe(FILE);
	});
});

describe('BankImportUpload — refus 409 BANK_IMPORT_PREVIEW_STALE (Story 27-3, AC9)', () => {
	const STALE = apiError('BANK_IMPORT_PREVIEW_STALE', 409);

	/** Aperçu A, Confirmer → 409 ; `arrangeRecompute` prépare la réponse du recalcul. */
	async function staleConfirm(
		c: HTMLElement,
		arrangeRecompute: () => void,
		o: PreviewOpts = {},
	): Promise<void> {
		await previewWithA(c, o);
		mockApi.createBankImport.mockRejectedValueOnce(STALE);
		arrangeRecompute();
		await confirm(c);
	}

	it('V6 — recalcul relancé, avis visible, une seule création, sélecteurs actifs pendant le recalcul', async () => {
		const { c, onSuccess } = await mount();
		const pending = deferred<BankImportPreviewResponse>();
		await staleConfirm(c, () => mockApi.previewBankImport.mockReturnValueOnce(pending.promise));

		expect(mockApi.previewBankImport).toHaveBeenCalledTimes(2);
		expect(mockApi.previewBankImport.mock.calls[1][0]).toBe(FILE);
		expect(mockApi.previewBankImport.mock.calls[1][1]).toBe(1);
		expect(profileFlag(1)).toBe(A);
		expect(q(c, 'bank-import-preview-recomputed')).not.toBeNull();
		expect(get(c, 'bank-import-preview-recomputed').getAttribute('role')).toBe('status');
		expect(get(c, 'bank-import-loading').textContent?.trim()).toBe(
			'Recalcul de la prévisualisation…',
		);
		expect(get<HTMLSelectElement>(c, 'bank-account-select').disabled).toBe(false);
		expect(get<HTMLSelectElement>(c, 'bank-profile-select').disabled).toBe(false);
		expect(mockApi.createBankImport).toHaveBeenCalledTimes(1);
		expect(onSuccess).not.toHaveBeenCalled();
		expect(q(c, 'bank-import-error')).toBeNull();

		pending.resolve(previewResp({ tag: 'A2' }));
		await flush();
		expect(get(c, 'bank-import-preview').textContent).toContain('tx-A2');
		expect(q(c, 'bank-import-preview-recomputed')).not.toBeNull();
		expect(mockApi.createBankImport).toHaveBeenCalledTimes(1);
	});

	it('V6 (variante) — le recalcul échoue : l’avis ET l’erreur sont visibles', async () => {
		const { c } = await mount();
		await staleConfirm(c, () => mockApi.previewBankImport.mockRejectedValueOnce(SIGN_REFUSAL));
		await flush();
		expect(q(c, 'bank-import-preview-recomputed')).not.toBeNull();
		expect(get(c, 'bank-import-error').getAttribute('data-error-code')).toBe(
			'BANK_CSV_AMOUNT_SIGN_MISMATCH',
		);
	});

	it('V6 (variante « cases ») — la voie 409 remet les cases à zéro', async () => {
		const { c } = await mount();
		await previewWithA(c, { warnings: DUPLICATE_FILE });
		await fireEvent.click(get(c, 'confirm-duplicate-file'));
		await flush();
		expect(get<HTMLInputElement>(c, 'confirm-duplicate-file').checked).toBe(true);
		mockApi.createBankImport.mockRejectedValueOnce(STALE);
		mockApi.previewBankImport.mockResolvedValueOnce(
			previewResp({ tag: 'A2', warnings: DUPLICATE_FILE }),
		);
		await confirm(c);
		expect(mockApi.createBankImport.mock.calls[0][2]?.confirmDuplicateFile).toBe(true);
		expect(get(c, 'bank-import-preview').textContent).toContain('tx-A2');
		expect(get<HTMLInputElement>(c, 'confirm-duplicate-file').checked).toBe(false);
		expect(get<HTMLButtonElement>(c, 'bank-import-confirm').disabled).toBe(true);
	});

	it('V6-ter — après un 409, le sélecteur relit la liste des profils', async () => {
		// Revue P1, B-1 (C-27-3-51) : un profil renommé ailleurs n'y garde pas
		// son ancien nom.
		const { c } = await mount();
		expect(mockProfileApi.listBankProfiles).toHaveBeenCalledTimes(1);
		mockProfileApi.listBankProfiles.mockResolvedValueOnce({
			items: [profile(A, 'Profil A renommé'), profile(B, 'Profil B')],
			total: 2,
			page: 1,
			perPage: 50,
		} as never);
		await staleConfirm(c, () =>
			mockApi.previewBankImport.mockResolvedValueOnce(previewResp({ tag: 'A2' })),
		);
		expect(mockProfileApi.listBankProfiles).toHaveBeenCalledTimes(2);
		const select = get<HTMLSelectElement>(c, 'bank-profile-select');
		expect(select.querySelector(`option[value="${A}"]`)?.textContent).toContain('Profil A renommé');
		expect(select.querySelector(`option[value="${C}"]`)).toBeNull();
	});

	it('V6-ter (variante) — la relecture échoue : la liste en place est gardée', async () => {
		const { c } = await mount();
		mockProfileApi.listBankProfiles.mockRejectedValueOnce(apiError('INTERNAL_ERROR', 500));
		await staleConfirm(c, () =>
			mockApi.previewBankImport.mockResolvedValueOnce(previewResp({ tag: 'A2' })),
		);
		expect(mockProfileApi.listBankProfiles).toHaveBeenCalledTimes(2);
		const select = get<HTMLSelectElement>(c, 'bank-profile-select');
		expect(select.querySelector(`option[value="${C}"]`)?.textContent).toContain('Profil C');
		expect(get(c, 'bank-import-preview').textContent).toContain('tx-A2');
	});

	/** Liste relue au 409 ; `items` remplace la liste du montage. */
	function relistWith(items: BankProfile[]): void {
		mockProfileApi.listBankProfiles.mockResolvedValueOnce({
			items,
			total: items.length,
			page: 1,
			perPage: 50,
		} as never);
	}

	const NO_MATCH = apiError('BANK_CSV_NO_PROFILE_MATCH', 404);

	it('V6-quater — le profil choisi a été supprimé : retour à l’auto-détection, erreur affichée, redépôt sans profil', async () => {
		// Revue de code P2, M1 (C-27-3-57).
		const { c } = await mount();
		relistWith([profile(B, 'Profil B')]);
		await staleConfirm(c, () => mockApi.previewBankImport.mockRejectedValueOnce(NO_MATCH));
		expect(profileFlag(1), 'le recalcul en vol garde son profil').toBe(A);
		const select = get<HTMLSelectElement>(c, 'bank-profile-select');
		expect(select.querySelector(`option[value="${A}"]`)).toBeNull();
		expect(select.value).toBe('');
		expect(q(c, 'bank-import-preview-recomputed')).not.toBeNull();
		expect(get(c, 'bank-import-error').getAttribute('data-error-code')).toBe(
			'BANK_CSV_NO_PROFILE_MATCH',
		);
		mockApi.previewBankImport.mockResolvedValueOnce(previewResp({ tag: 'X', profileId: B }));
		await chooseFile(c);
		expect(mockApi.previewBankImport).toHaveBeenCalledTimes(3);
		expect(profileFlag(2), 'redépôt : auto-détection').toBeUndefined();
	});

	it('V6-quater (liste vide) — plus aucun profil : sélecteur absent, redépôt sans profil', async () => {
		const { c } = await mount();
		relistWith([]);
		await staleConfirm(c, () => mockApi.previewBankImport.mockRejectedValueOnce(NO_MATCH));
		expect(q(c, 'bank-profile-select')).toBeNull();
		expect(get(c, 'bank-import-error').getAttribute('data-error-code')).toBe(
			'BANK_CSV_NO_PROFILE_MATCH',
		);
		mockApi.previewBankImport.mockResolvedValueOnce(previewResp({ tag: 'X', profileId: null }));
		await chooseFile(c);
		expect(mockApi.previewBankImport).toHaveBeenCalledTimes(3);
		expect(profileFlag(2), 'redépôt : auto-détection').toBeUndefined();
	});

	it('V6-quater (séquence) — deux relectures croisées : seule la dernière demandée pose la liste', async () => {
		// Revue de code P2, L4.
		const { c } = await mount();
		const first = deferred<unknown>();
		const second = deferred<unknown>();
		mockProfileApi.listBankProfiles.mockReturnValueOnce(first.promise as never);
		await staleConfirm(c, () =>
			mockApi.previewBankImport.mockResolvedValueOnce(previewResp({ tag: 'A2' })),
		);
		mockProfileApi.listBankProfiles.mockReturnValueOnce(second.promise as never);
		mockApi.createBankImport.mockRejectedValueOnce(STALE);
		mockApi.previewBankImport.mockResolvedValueOnce(previewResp({ tag: 'A3' }));
		await confirm(c);
		expect(mockProfileApi.listBankProfiles).toHaveBeenCalledTimes(3);
		second.resolve({ items: [profile(A, 'Profil A récent')], total: 1, page: 1, perPage: 50 });
		await flush();
		first.resolve({
			items: [profile(A, 'Profil A ancien'), profile(C, 'Profil C')],
			total: 2,
			page: 1,
			perPage: 50,
		});
		await flush();
		const select = get<HTMLSelectElement>(c, 'bank-profile-select');
		expect(select.querySelector(`option[value="${A}"]`)?.textContent).toContain('Profil A récent');
		expect(select.querySelector(`option[value="${C}"]`)).toBeNull();
	});

	it('V6-bis (a) — changer de profil efface l’avis dès le geste', async () => {
		const { c } = await mount();
		await staleConfirm(c, () =>
			mockApi.previewBankImport.mockResolvedValueOnce(previewResp({ tag: 'A2' })),
		);
		expect(q(c, 'bank-import-preview-recomputed')).not.toBeNull();
		mockApi.previewBankImport.mockReturnValueOnce(deferred<BankImportPreviewResponse>().promise);
		await chooseProfile(c, B);
		expect(q(c, 'bank-import-preview-recomputed')).toBeNull();
	});

	it('V6-bis (b) — cliquer Confirmer efface l’avis dès le clic ; l’indicateur dit « Chargement… »', async () => {
		const { c } = await mount();
		await staleConfirm(c, () =>
			mockApi.previewBankImport.mockResolvedValueOnce(previewResp({ tag: 'A2' })),
		);
		expect(q(c, 'bank-import-preview-recomputed')).not.toBeNull();
		mockApi.createBankImport.mockReturnValueOnce(deferred<BankImportResponse>().promise);
		await confirm(c);
		expect(q(c, 'bank-import-preview-recomputed')).toBeNull();
		expect(get(c, 'bank-import-loading').textContent?.trim()).toBe('Chargement…');
	});

	it('V6-bis (c) — changer de compte efface l’avis', async () => {
		const { c } = await mount();
		await staleConfirm(c, () =>
			mockApi.previewBankImport.mockResolvedValueOnce(previewResp({ tag: 'A2' })),
		);
		expect(q(c, 'bank-import-preview-recomputed')).not.toBeNull();
		await chooseAccount(c, 2);
		expect(q(c, 'bank-import-preview-recomputed')).toBeNull();
	});
});

describe('Écran d’import — libellés traduits (Story 27-3, AC17)', () => {
	const KEYS = [
		'bank-import-labels-bank-account-placeholder',
		'bank-import-errors-bank-account-required',
		'bank-import-labels-col-line',
		'bank-import-labels-col-code',
		'bank-import-labels-col-value',
		'bank-import-labels-iban',
		'bank-import-labels-period',
		'bank-import-labels-currency',
		'bank-import-labels-opening-balance',
		'bank-import-labels-closing-balance',
		'bank-import-labels-transaction-count',
		'bank-import-labels-duplicate-col-index',
		'bank-import-labels-duplicate-col-existing-id',
		'bank-import-labels-duplicate-col-key',
		'bank-import-labels-ignored-statements-summary',
		'bank-import-labels-tx-col-date',
		'bank-import-labels-tx-col-amount',
		'bank-import-labels-tx-col-details',
		'bank-import-labels-list-col-imported-at',
		'bank-import-labels-list-col-filename',
		'bank-import-labels-list-col-transactions',
		'common-loading',
		'error-unexpected',
	];
	const FR_TEXTS = [
		'Sélectionner un compte',
		"Sélectionnez d'abord",
		'Ligne',
		'Valeur',
		'Période',
		'Devise',
		'Solde ouverture',
		'Solde clôture',
		'Nombre de transactions',
		'ID existant',
		'Clé',
		'statement(s)',
		'Montant',
		'Détails',
		'Date import',
		'Fichier',
		'Erreur inattendue',
		'Chargement',
	];
	// Mots ambigus (un IBAN, une date peuvent les contenir) : cherchés en nœud
	// de texte entier.
	const FR_WHOLE = /> *(Code|IBAN|Index|Date|Transactions) *</;

	it('V13 — chaque clé de l’AC17 apparaît, aucun texte fr en dur ne subsiste', async () => {
		i18nMode.keys = true;
		const pages: string[] = [];
		const snap = (c: HTMLElement) => pages.push(c.innerHTML);

		// (a) dépôt sans compte (par glisser-déposer : l'input est désactivé).
		let { c } = await mount();
		await fireEvent.drop(get(c, 'bank-import-drop-zone'), { dataTransfer: { files: [FILE] } });
		await flush();
		expect(get(c, 'bank-import-error').textContent).toContain(
			'bank-import-errors-bank-account-required',
		);
		snap(c);

		// (b) aperçu en vol → common-loading.
		await chooseAccount(c, 1);
		const pending = deferred<BankImportPreviewResponse>();
		mockApi.previewBankImport.mockReturnValueOnce(pending.promise);
		await chooseFile(c);
		expect(get(c, 'bank-import-loading').textContent?.trim()).toBe('common-loading');
		snap(c);
		// (c) aperçu rejeté par une erreur qui n'est pas une ApiError.
		pending.reject(new Error('panne'));
		await flush();
		expect(get(c, 'bank-import-error').textContent).toContain('error-unexpected');
		snap(c);
		cleanup();

		// (c bis) confirmation rejetée de même.
		({ c } = await mount());
		await previewWithA(c);
		mockApi.createBankImport.mockRejectedValueOnce(new Error('panne'));
		await confirm(c);
		expect(get(c, 'bank-import-error').textContent).toContain('error-unexpected');
		cleanup();

		// (d) aperçu avec doublons de lignes, lignes invalides, relevés ignorés.
		({ c } = await mount());
		await previewWithA(c, {
			ignored: 2,
			warnings: {
				duplicateLines: [{ newIndex: 0, existingTransactionId: 9, key: 'k' }],
				invalidLines: {
					lines: [{ line: 4, code: 'INVALID_DATE', value: 'x', messageI18nKey: 'k' }],
					totalErrors: 1,
					truncated: false,
				},
			},
		});
		snap(c);
		cleanup();

		// (e) refus de signe avec lignes.
		({ c } = await mount());
		await chooseAccount(c, 1);
		mockApi.previewBankImport.mockRejectedValueOnce(SIGN_REFUSAL);
		await chooseFile(c);
		expect(q(c, 'bank-import-error-line')).not.toBeNull();
		snap(c);
		cleanup();

		// (f) la liste des imports.
		const list = render(BankImportList, {
			props: {
				imports: [
					{
						id: 1,
						bankAccountId: 1,
						filename: 'a.csv',
						fileHash: 'h',
						sourceFormat: 'CSV',
						statementId: null,
						periodFrom: '2026-01-01',
						periodTo: '2026-01-31',
						openingBalance: null,
						closingBalance: null,
						transactionCount: 3,
						importedAt: '2026-02-01T00:00:00Z',
					},
				],
			},
		});
		snap(list.container);

		const all = pages.join('\n');
		for (const key of KEYS) expect(all, key).toContain(key);
		for (const text of FR_TEXTS) expect(all, text).not.toContain(text);
		expect(all).not.toMatch(FR_WHOLE);
	});
});
