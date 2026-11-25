// Story 30-2b (#685, AC10, AC14, AC16) — la page `/supplier-invoices/new`.
//
// ⚠️ Chaque test nomme la mutation qu'il attrape. Patron des `*-page.test.ts` : mocks
// hoistés AVANT l'import du composant.

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, waitFor, cleanup } from '@testing-library/svelte';
import type { ImportedSupplierInvoice } from '$lib/features/imported-supplier-invoices/imported-supplier-invoices.types';

const url = { value: new URL('http://localhost/supplier-invoices/new') };
vi.mock('$app/navigation', () => ({ goto: vi.fn() }));
vi.mock('$app/state', () => ({
	page: {
		get url() {
			return url.value;
		},
	},
}));
vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string, args?: Record<string, string | number>) =>
		args
			? fallback.replace(/\{\s*\$(\w+)\s*\}/g, (_, n) => String(args[n] ?? ''))
			: fallback,
}));
vi.mock('$lib/shared/utils/notify', () => ({
	notifySuccess: vi.fn(),
	notifyError: vi.fn(),
	notifyWarning: vi.fn(),
}));
const role = { value: 'Comptable' };
vi.mock('$lib/app/stores/auth.svelte', () => ({
	authState: {
		get currentUser() {
			return { role: role.value, username: 'c', userId: '1' };
		},
	},
}));

const listImportedMock = vi.fn();
const candidatesMock = vi.fn();
const fetchDocMock = vi.fn();
const completeMock = vi.fn();
vi.mock('$lib/features/imported-supplier-invoices/imported-supplier-invoices.api', () => ({
	// Story 48-12b (D2) : le mock transmet `CallOptions` (`{ retry: false }` de la relecture).
	listImported: (...args: unknown[]) => listImportedMock(...args),
	getSupplierCandidates: (id: number) => candidatesMock(id),
	fetchImportedSourceDocument: (id: number) => fetchDocMock(id),
	completeImport: (id: number, req: unknown) => completeMock(id, req),
}));
const listContactsMock = vi.fn();
vi.mock('$lib/features/contacts/contacts.api', () => ({
	listContacts: (q: unknown) => listContactsMock(q),
	createContact: vi.fn(),
}));
vi.mock('$lib/features/accounts/accounts.api', () => ({
	fetchAccounts: () =>
		Promise.resolve([
			{ id: 40, number: '4000', name: 'Achats', accountType: 'Expense', active: true, postable: true },
		]),
}));
vi.mock('$lib/features/vat-rates/vat-rates.api', () => ({ listVatRates: () => Promise.resolve([]) }));
vi.mock('$lib/features/projects/projects.api', () => ({ listProjects: () => Promise.resolve([]) }));
vi.mock('$lib/features/supplier-invoices/supplier-invoices.api', () => ({
	createSupplierInvoice: vi.fn(),
	scanQrSupplierInvoice: vi.fn(),
}));

import Page from './+page.svelte';
import { fireEvent } from '@testing-library/svelte';
import { goto } from '$app/navigation';
import { notifySuccess } from '$lib/shared/utils/notify';

function doc(over: Partial<ImportedSupplierInvoice> = {}): ImportedSupplierInvoice {
	return {
		id: 7,
		companyId: 1,
		status: 'to_complete',
		supplierInvoiceId: null,
		fileHash: 'h',
		storagePath: 'h.png',
		originalFilename: 'facture-dubarde.png',
		mimeType: 'image/png',
		byteSize: 10,
		creditorIban: 'CH9300762011623852957',
		isQrIban: false,
		creditorAddressType: 'K',
		creditorName: 'Dubarde SA',
		creditorLine1: 'Rue du Lac 1',
		creditorLine2: '1000 Lausanne',
		creditorPostalCode: null,
		creditorTown: null,
		creditorCountry: 'CH',
		referenceType: 'NON',
		referenceValue: null,
		amount: '100.00',
		currency: 'CHF',
		unstructuredMessage: null,
		billingInformation: null,
		version: 1,
		createdAt: '2026-10-15T08:30:00',
		updatedAt: '2026-10-15T08:30:00',
		...over,
	};
}

beforeEach(() => {
	role.value = 'Comptable';
	url.value = new URL('http://localhost/supplier-invoices/new');
	listImportedMock.mockReset();
	candidatesMock.mockReset();
	fetchDocMock.mockReset();
	listContactsMock.mockReset();
	listContactsMock.mockResolvedValue({ items: [], total: 0, limit: 200, offset: 0 });
	candidatesMock.mockResolvedValue({ candidates: [] });
	fetchDocMock.mockResolvedValue(new Blob(['x'], { type: 'image/png' }));
});
afterEach(() => cleanup());

describe('page /supplier-invoices/new', () => {
	it('sans document : le formulaire seul, avec le scan, sans lire les documents à traiter', async () => {
		const { findByTestId, queryByTestId } = render(Page);
		await findByTestId('supplier-invoice-form');
		expect(queryByTestId('supplier-invoice-scan-btn')).not.toBeNull();
		expect(queryByTestId('supplier-invoice-new-preview')).toBeNull();
		expect(listImportedMock).not.toHaveBeenCalled();
	});

	it('à côté du document : aperçu et formulaire, sans scan', async () => {
		url.value = new URL('http://localhost/supplier-invoices/new?document=7');
		listImportedMock.mockResolvedValue([doc()]);
		const { findByTestId, getByTestId, queryByTestId } = render(Page);
		await findByTestId('supplier-invoice-form');
		expect(getByTestId('supplier-invoice-new-preview').textContent).toContain('facture-dubarde.png');
		expect(queryByTestId('supplier-invoice-scan-btn')).toBeNull();
		await waitFor(() => expect(fetchDocMock).toHaveBeenCalledWith(7));
	});

	it.each([
		['inconnu', 'document=99'],
		['illisible', 'document=abc'],
	])('document %s ou plus à traiter : le même message, un lien, aucun formulaire', async (_l, q) => {
		url.value = new URL(`http://localhost/supplier-invoices/new?${q}`);
		listImportedMock.mockResolvedValue([doc()]);
		const { findByTestId, queryByTestId } = render(Page);
		const gone = await findByTestId('supplier-invoice-new-doc-gone');
		expect(gone.textContent).toContain('Ce document n’est plus à traiter.');
		expect(gone.querySelector('a')?.getAttribute('href')).toBe('/supplier-invoices/import');
		expect(queryByTestId('supplier-invoice-form')).toBeNull();
	});

	it('état de chargement, puis état d’erreur de la lecture', async () => {
		url.value = new URL('http://localhost/supplier-invoices/new?document=7');
		let rejectList: (e: unknown) => void = () => {};
		listImportedMock.mockReturnValue(
			new Promise((_, reject) => {
				rejectList = reject;
			}),
		);
		const { findByTestId, getByTestId } = render(Page);
		expect(getByTestId('supplier-invoice-new-loading')).not.toBeNull();
		rejectList({ code: 'INTERNAL_ERROR', status: 500, message: 'panne' });
		const err = await findByTestId('supplier-invoice-new-error');
		expect(err.textContent).toContain('panne');
	});

	it('propose par IBAN puis par nom, la recherche filtrée sur les fournisseurs (M-b5)', async () => {
		url.value = new URL('http://localhost/supplier-invoices/new?document=7');
		listImportedMock.mockResolvedValue([doc()]);
		const { findAllByTestId } = render(Page);
		await findAllByTestId('supplier-invoice-form');
		await waitFor(() => expect(candidatesMock).toHaveBeenCalledWith(7));
		const searchCall = listContactsMock.mock.calls.find(
			(c) => (c[0] as { search?: string }).search !== undefined,
		);
		expect(searchCall?.[0]).toEqual({ search: 'dubarde', isSupplier: true, limit: 20 });
	});

	it('un document sans QR ne déclenche aucune proposition', async () => {
		url.value = new URL('http://localhost/supplier-invoices/new?document=7');
		listImportedMock.mockResolvedValue([
			doc({
				creditorIban: null,
				isQrIban: null,
				creditorAddressType: null,
				creditorName: null,
				creditorCountry: null,
				referenceType: null,
				amount: null,
				currency: null,
			}),
		]);
		const { findByTestId } = render(Page);
		await findByTestId('supplier-invoice-form');
		expect(candidatesMock).not.toHaveBeenCalled();
	});

	// Story v017-4b (#849, AC-24 (e)) — la complétion crée un BROUILLON : la notification
	// le dit (clé `imported-supplier-invoices-completed`), plus « Facture créée ».
	it('après la complétion d’un document, « Brouillon enregistré. », puis la fiche', async () => {
		url.value = new URL('http://localhost/supplier-invoices/new?document=7');
		listImportedMock.mockResolvedValue([doc()]);
		listContactsMock.mockResolvedValue({
			items: [{ id: 1, name: 'Dubarde SA', isSupplier: true }],
			total: 1,
			limit: 200,
			offset: 0,
		});
		completeMock.mockResolvedValue({ id: 33, status: 'draft' });
		const { findByTestId, getByTestId, container } = render(Page);
		await findByTestId('supplier-invoice-form');
		const q = (id: string) => container.querySelector(`[data-testid="${id}"]`) as HTMLInputElement;
		const sup = q('supplier-invoice-supplier') as unknown as HTMLSelectElement;
		sup.value = '1';
		await fireEvent.change(sup);
		await fireEvent.input(q('supplier-invoice-date'), { target: { value: '2026-10-01' } });
		await fireEvent.input(q('supplier-invoice-line-desc'), { target: { value: 'Prestation' } });
		await fireEvent.input(q('supplier-invoice-line-unit-price'), { target: { value: '100.00' } });
		const acc = q('supplier-invoice-line-account') as unknown as HTMLSelectElement;
		acc.value = '40';
		await fireEvent.change(acc);
		await fireEvent.click(getByTestId('supplier-invoice-submit'));
		await waitFor(() => expect(completeMock).toHaveBeenCalledTimes(1));
		await waitFor(() => expect(notifySuccess).toHaveBeenCalledWith('Brouillon enregistré.'));
		expect(goto).toHaveBeenCalledWith('/supplier-invoices/33');
	});

	it('pour un rôle Consultation, la page ne propose rien', async () => {
		role.value = 'Consultation';
		const { findByTestId, queryByTestId } = render(Page);
		await findByTestId('supplier-invoice-new-forbidden');
		expect(queryByTestId('supplier-invoice-form')).toBeNull();
		expect(listContactsMock).not.toHaveBeenCalled();
	});
});

// ---------------------------------------------------------------------------
// Story 48-12b (#985, #975 ; table A, E5 à E9 ; D2, D2bis) — la complétion relue, et le
// refus de doublon à la complétion.
// ---------------------------------------------------------------------------

const NOT_PENDING = {
	code: 'IMPORT_NOT_PENDING_COMPLETION',
	status: 409,
	message: 'Ce document n’est plus à traiter.',
};
const LOST = { code: 'NETWORK_ERROR', status: 0, message: 'Réseau indisponible' };
const OLD_MESSAGE = 'Ce document a déjà été traité ou retiré de la liste.';
const FOR_NOW = 'Aucune facture n’est enregistrée pour l’instant : vous pouvez réessayer.';
const COMPLETED_TEXT =
	'Cette pièce a déjà été enregistrée comme facture. Ouvrez-la pour vérifier qu’elle correspond à votre saisie.';

/** La page de complétion du document 7, le formulaire rempli ; rend le rendu. */
async function completionPrete() {
	completeMock.mockReset();
	url.value = new URL('http://localhost/supplier-invoices/new?document=7');
	listImportedMock.mockResolvedValueOnce([doc()]);
	listContactsMock.mockResolvedValue({
		items: [{ id: 1, name: 'Dubarde SA', isSupplier: true }],
		total: 1,
		limit: 200,
		offset: 0,
	});
	const r = render(Page);
	await r.findByTestId('supplier-invoice-form');
	const q = (id: string) => r.container.querySelector(`[data-testid="${id}"]`) as HTMLInputElement;
	const sup = q('supplier-invoice-supplier') as unknown as HTMLSelectElement;
	sup.value = '1';
	await fireEvent.change(sup);
	await fireEvent.input(q('supplier-invoice-date'), { target: { value: '2026-10-01' } });
	await fireEvent.input(q('supplier-invoice-line-desc'), { target: { value: 'Prestation' } });
	await fireEvent.input(q('supplier-invoice-line-unit-price'), { target: { value: '100.00' } });
	const acc = q('supplier-invoice-line-account') as unknown as HTMLSelectElement;
	acc.value = '40';
	await fireEvent.change(acc);
	return r;
}

describe('complétion relue (#985, E8 → E9, E7, E6)', () => {
	it('IMPORT_NOT_PENDING_COMPLETION : to_complete PUIS completed, sans rejeu → E9 figé, lien nouvel onglet, ancien message absent', async () => {
		const r = await completionPrete();
		completeMock.mockRejectedValue(NOT_PENDING);
		listImportedMock
			.mockResolvedValueOnce([])
			.mockResolvedValueOnce([doc({ status: 'completed', supplierInvoiceId: 55 })]);
		await fireEvent.click(r.getByTestId('supplier-invoice-submit'));
		const notice = await r.findByTestId('supplier-invoice-already-completed');
		expect(r.getByTestId('supplier-invoice-already-completed-text').textContent).toBe(COMPLETED_TEXT);
		const link = r.getByTestId('supplier-invoice-already-completed-link');
		expect(link.getAttribute('href')).toBe('/supplier-invoices/55');
		// Mutation « lien de E9 dans le même onglet ».
		expect(link.getAttribute('target')).toBe('_blank');
		// Mutation « lire completed d'abord » : l'ordre et `{ retry: false }`.
		expect(listImportedMock.mock.calls.slice(1)).toEqual([
			['to_complete', { retry: false }],
			['completed', { retry: false }],
		]);
		// Mutation « relancer l'erreur d'origine après la relecture » : l'ancien message absent.
		expect(r.container.textContent).not.toContain(OLD_MESSAGE);
		expect(notice).toBeTruthy();
		// Mutation « locked ignoré » (avec la garde du formulaire, défense en profondeur).
		expect(r.getByTestId('supplier-invoice-number').matches(':disabled')).toBe(true);
		expect((r.getByTestId('supplier-invoice-submit') as HTMLButtonElement).disabled).toBe(true);
	});

	it('issue inconnue de la complétion, document complété → E9 aussi', async () => {
		const r = await completionPrete();
		completeMock.mockRejectedValue(LOST);
		listImportedMock
			.mockResolvedValueOnce([])
			.mockResolvedValueOnce([doc({ status: 'completed', supplierInvoiceId: 56 })]);
		await fireEvent.click(r.getByTestId('supplier-invoice-submit'));
		await r.findByTestId('supplier-invoice-already-completed');
	});

	it('encore à traiter → E7 « pour l’instant », saisie active', async () => {
		const r = await completionPrete();
		completeMock.mockRejectedValue(LOST);
		listImportedMock.mockResolvedValueOnce([doc()]);
		await fireEvent.click(r.getByTestId('supplier-invoice-submit'));
		await waitFor(() => expect(r.getByTestId('supplier-invoice-form-error').textContent?.trim()).toBe(FOR_NOW));
		expect((r.getByTestId('supplier-invoice-submit') as HTMLButtonElement).disabled).toBe(false);
	});

	it('retiré (ni à traiter ni complété) → E7, message actuel', async () => {
		const r = await completionPrete();
		completeMock.mockRejectedValue(NOT_PENDING);
		listImportedMock.mockResolvedValueOnce([]).mockResolvedValueOnce([]);
		await fireEvent.click(r.getByTestId('supplier-invoice-submit'));
		await waitFor(() =>
			expect(r.getByTestId('supplier-invoice-form-error').textContent?.trim()).toBe(OLD_MESSAGE),
		);
	});

	it('complété sans supplierInvoiceId → E6 (mutation « traité comme complété »)', async () => {
		const r = await completionPrete();
		completeMock.mockRejectedValue(NOT_PENDING);
		listImportedMock.mockResolvedValueOnce([]).mockResolvedValueOnce([doc({ status: 'completed' })]);
		await fireEvent.click(r.getByTestId('supplier-invoice-submit'));
		await r.findByTestId('supplier-invoice-unknown-outcome');
		expect(r.queryByTestId('supplier-invoice-already-completed')).toBeNull();
	});

	it('complété, supplierInvoiceId ABSENT (hors contrat) → E6 avec reprise, jamais E9 figé sans avis (revue P1, E3)', async () => {
		const r = await completionPrete();
		completeMock.mockRejectedValue(NOT_PENDING);
		const sansChamp: Record<string, unknown> = { ...doc({ status: 'completed' }) };
		delete sansChamp.supplierInvoiceId;
		listImportedMock.mockResolvedValueOnce([]).mockResolvedValueOnce([sansChamp]);
		await fireEvent.click(r.getByTestId('supplier-invoice-submit'));
		await r.findByTestId('supplier-invoice-unknown-outcome');
		expect(r.queryByTestId('supplier-invoice-already-completed')).toBeNull();
		// Mutations « `=== null` à la page » (locked posé) : la reprise rend la saisie.
		await fireEvent.click(r.getByTestId('supplier-invoice-resume-entry'));
		expect(r.getByTestId('supplier-invoice-number').matches(':disabled')).toBe(false);
	});

	it('relecture en échec → E6, jamais « déjà traité » (mutation « déjà traité sur relecture en échec »)', async () => {
		const r = await completionPrete();
		completeMock.mockRejectedValue(NOT_PENDING);
		listImportedMock.mockRejectedValueOnce({ code: 'INTERNAL_ERROR', status: 500, message: 'panne' });
		await fireEvent.click(r.getByTestId('supplier-invoice-submit'));
		await r.findByTestId('supplier-invoice-unknown-outcome');
		expect(r.container.textContent).not.toContain(OLD_MESSAGE);
	});

	it('doublon probable à la complétion → E5, AUCUNE relecture (mutation « onRefusal pour tout code »)', async () => {
		const r = await completionPrete();
		completeMock.mockRejectedValueOnce({
			code: 'SUPPLIER_INVOICE_PROBABLE_DUPLICATE',
			status: 409,
			message: 'x',
			details: { supplierInvoiceId: 12, internalNumber: null, status: 'draft', confirmable: true },
		});
		await fireEvent.click(r.getByTestId('supplier-invoice-submit'));
		await r.findByTestId('supplier-invoice-probable-duplicate');
		expect(listImportedMock).toHaveBeenCalledTimes(1);
		// La confirmation renvoie le corps gardé, avec le drapeau.
		completeMock.mockResolvedValueOnce({ id: 33, status: 'draft' });
		await fireEvent.click(r.getByTestId('supplier-invoice-confirm-not-duplicate'));
		await waitFor(() => expect(completeMock).toHaveBeenCalledTimes(2));
		expect(completeMock.mock.calls[1][1]).toEqual({ ...completeMock.mock.calls[0][1], confirmNotDuplicate: true });
	});

	it('E8 figé : relecture en attente → aucun contrôle de saisie actif, « Enregistrer » inactif', async () => {
		const r = await completionPrete();
		completeMock.mockRejectedValue(NOT_PENDING);
		listImportedMock.mockReturnValueOnce(new Promise(() => {}));
		await fireEvent.click(r.getByTestId('supplier-invoice-submit'));
		await waitFor(() => expect(listImportedMock).toHaveBeenCalledTimes(2));
		expect(r.getByTestId('supplier-invoice-number').matches(':disabled')).toBe(true);
		expect((r.getByTestId('supplier-invoice-submit') as HTMLButtonElement).disabled).toBe(true);
	});

	it('E6 par relecture en échec, reprise, renvoi → IMPORT_NOT_PENDING → E8 → E9 (V4-3)', async () => {
		const r = await completionPrete();
		completeMock.mockRejectedValue(LOST);
		listImportedMock.mockRejectedValueOnce({ code: 'INTERNAL_ERROR', status: 500, message: 'panne' });
		await fireEvent.click(r.getByTestId('supplier-invoice-submit'));
		await r.findByTestId('supplier-invoice-unknown-outcome');
		await fireEvent.click(r.getByTestId('supplier-invoice-resume-entry'));
		completeMock.mockRejectedValue(NOT_PENDING);
		listImportedMock
			.mockResolvedValueOnce([])
			.mockResolvedValueOnce([doc({ status: 'completed', supplierInvoiceId: 57 })]);
		await fireEvent.click(r.getByTestId('supplier-invoice-submit'));
		await r.findByTestId('supplier-invoice-already-completed');
	});
});
