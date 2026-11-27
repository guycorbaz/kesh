/**
 * v0172-correctifs (#880, § 24 bis de la fiche, AC-C2) — l'écran *Taux de TVA* dit les
 * dates de validité en forme suisse ; une validité sans fin reste « — ».
 *
 * Premier test de cette page. Le nom n'est pas `+page.test.ts` : SvelteKit réserve le
 * préfixe `+`.
 */
import { describe, it, expect, vi, afterEach } from 'vitest';
import { render, cleanup, fireEvent, waitFor } from '@testing-library/svelte';

vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_k: string, fallback: string, args?: Record<string, string | number>) =>
		args ? fallback.replace(/\{\s*\$(\w+)\s*\}/g, (_, k) => String(args[k] ?? '')) : fallback,
}));
vi.mock('svelte-sonner', () => ({ toast: { success: vi.fn(), error: vi.fn() } }));
const listMock = vi.fn();
const createMock = vi.fn();
const updateMock = vi.fn();
vi.mock('$lib/features/vat-rates/vat-rates.api', () => ({
	listVatRates: (h: boolean) => listMock(h),
	createVatRate: (p: unknown) => createMock(p),
	updateVatRate: (id: number, p: unknown) => updateMock(id, p),
	deactivateVatRate: vi.fn(),
}));

import Page from './+page.svelte';

afterEach(() => {
	cleanup();
	vi.clearAllMocks();
});

describe('taux de TVA — dates de validité en forme suisse (v0172, #880)', () => {
	it('du 01.01.2018 au 31.12.2023, puis dès le 01.01.2024 ; validité sans fin « — » ; jamais l’ISO (MC2)', async () => {
		listMock.mockResolvedValue([
			{
				id: 1,
				category: 'normal',
				label: 'Normal',
				rate: '7.70',
				validFrom: '2018-01-01',
				validTo: '2023-12-31',
				active: false,
				version: 1,
			},
			{
				id: 2,
				category: 'normal',
				label: 'Normal',
				rate: '8.10',
				validFrom: '2024-01-01',
				validTo: null,
				active: true,
				version: 1,
			},
		]);
		const { findAllByTestId } = render(Page);
		const rows = await findAllByTestId('vat-rate-row');
		const cells = rows.map((r) =>
			Array.from(r.querySelectorAll('td'))
				.slice(1, 3)
				.map((td) => td.textContent?.trim()),
		);
		// L'écran trie les taux d'une catégorie ; l'ordre n'est pas l'objet du test.
		expect(cells.sort()).toEqual([
			['01.01.2018', '31.12.2023'],
			['01.01.2024', '—'],
		]);
		expect(rows.map((r) => r.textContent).join(' ')).not.toMatch(/20\d{2}-\d{2}-\d{2}/);
	});
});


// ---------------------------------------------------------------------------
// Story 64-1 (#1178) — le taux saisi part réellement, en chaîne décimale.
// ---------------------------------------------------------------------------

const ACTIVE = {
	id: 2,
	category: 'normal',
	label: 'Normal',
	rate: '8.10',
	validFrom: '2024-01-01',
	validTo: null,
	active: true,
	version: 3,
};

async function openCreateForm() {
	listMock.mockResolvedValue([]);
	const r = render(Page);
	await waitFor(() => expect(r.getByTestId('vat-rates-new')).toBeTruthy());
	await fireEvent.click(r.getByTestId('vat-rates-new'));
	return r;
}

async function fill(r: { getByTestId: (id: string) => HTMLElement }, id: string, value: string) {
	await fireEvent.input(r.getByTestId(id), { target: { value } });
}

async function openChangeForm() {
	listMock.mockResolvedValue([ACTIVE]);
	const r = render(Page);
	await fireEvent.click(await r.findByText('Changer le taux'));
	return r;
}

describe('taux de TVA — création (64-1, AC1)', () => {
	it('createVatRate reçoit le taux en chaîne, le toast et le rechargement suivent', async () => {
		createMock.mockResolvedValue({});
		const r = await openCreateForm();
		await fill(r, 'vat-form-rate', '8.1');
		await fill(r, 'vat-form-from', '2027-01-01');
		await fireEvent.click(r.getByTestId('vat-form-submit'));
		await waitFor(() => expect(createMock).toHaveBeenCalledTimes(1));
		expect(createMock.mock.calls[0][0]).toMatchObject({ rate: '8.1', validFrom: '2027-01-01' });
		expect(typeof (createMock.mock.calls[0][0] as { rate: unknown }).rate).toBe('string');
		expect(r.queryByTestId('vat-form-error')).toBeNull();
		const { toast } = await import('svelte-sonner');
		await waitFor(() => expect(toast.success).toHaveBeenCalledWith('Taux TVA créé.'));
		expect(listMock).toHaveBeenCalledTimes(2);
	});

	it.each([
		['taux vidé', '', '2027-01-01', 'Saisissez un taux.'],
		['taux hors bornes', '150', '2027-01-01', 'Le taux doit être compris entre 0 et 100.'],
		['taux négatif', '-1', '2027-01-01', 'Le taux doit être compris entre 0 et 100.'],
		['trois décimales', '8.125', '2027-01-01', 'Le taux ne peut avoir plus de deux décimales.'],
		['date vide', '8.1', '', 'Saisissez la date de début.'],
	])('%s : rien n’est envoyé, le message dit pourquoi', async (_n, rate, from, msg) => {
		const r = await openCreateForm();
		if (rate !== '') await fill(r, 'vat-form-rate', rate);
		else {
			await fill(r, 'vat-form-rate', '8.1');
			await fill(r, 'vat-form-rate', '');
		}
		if (from) await fill(r, 'vat-form-from', from);
		await fireEvent.click(r.getByTestId('vat-form-submit'));
		await waitFor(() => expect(r.getByTestId('vat-form-error').textContent).toBe(msg));
		expect(createMock).not.toHaveBeenCalled();
	});

	it('champ jamais touché (valeur initiale) : refusé par le formulaire, pas « La création a échoué. »', async () => {
		const r = await openCreateForm();
		await fill(r, 'vat-form-from', '2027-01-01');
		await fireEvent.click(r.getByTestId('vat-form-submit'));
		await waitFor(() => expect(r.getByTestId('vat-form-error').textContent).toBe('Saisissez un taux.'));
		expect(createMock).not.toHaveBeenCalled();
	});
});

describe('taux de TVA — changement de taux (64-1, AC2, AC3)', () => {
	it('updateVatRate clôture, puis createVatRate reçoit le taux en chaîne ; toast et rechargement', async () => {
		updateMock.mockResolvedValue({});
		createMock.mockResolvedValue({});
		const r = await openChangeForm();
		await fill(r, 'vat-change-rate', '8.1');
		await fill(r, 'vat-change-from', '2027-01-01');
		await fireEvent.click(r.getByTestId('vat-change-submit'));
		await waitFor(() => expect(createMock).toHaveBeenCalledTimes(1));
		expect(updateMock).toHaveBeenCalledWith(2, expect.objectContaining({ validTo: '2027-01-01' }));
		expect(createMock.mock.calls[0][0]).toMatchObject({ category: 'normal', rate: '8.1', validFrom: '2027-01-01' });
		const { toast } = await import('svelte-sonner');
		await waitFor(() => expect(toast.success).toHaveBeenCalledWith('Taux mis à jour.'));
	});

	it.each([
		['taux vidé (null)', '__vide__', '2027-01-01', 'Saisissez un taux.'],
		['taux jamais touché', '__neuf__', '2027-01-01', 'Saisissez un taux.'],
		['150', '150', '2027-01-01', 'Le taux doit être compris entre 0 et 100.'],
		['-1', '-1', '2027-01-01', 'Le taux doit être compris entre 0 et 100.'],
		['8.125', '8.125', '2027-01-01', 'Le taux ne peut avoir plus de deux décimales.'],
		['date de bascule vide', '8.1', '', 'Saisissez la date de bascule.'],
	])('%s : ni updateVatRate ni createVatRate', async (_n, rate, from, msg) => {
		const r = await openChangeForm();
		if (rate === '__vide__') {
			await fill(r, 'vat-change-rate', '8.1');
			await fill(r, 'vat-change-rate', '');
		} else if (rate !== '__neuf__') await fill(r, 'vat-change-rate', rate);
		if (from) await fill(r, 'vat-change-from', from);
		await fireEvent.click(r.getByTestId('vat-change-submit'));
		await waitFor(() => expect(r.getByTestId('vat-change-error').textContent).toBe(msg));
		expect(updateMock).not.toHaveBeenCalled();
		expect(createMock).not.toHaveBeenCalled();
	});

	it('createVatRate refusé APRÈS la clôture (ApiError) : liste rechargée, « clôturé », motif du serveur, comment finir', async () => {
		updateMock.mockResolvedValue({});
		createMock.mockRejectedValue({ code: 'VALIDATION_ERROR', status: 400, message: 'Chevauchement avec un autre taux.' });
		const r = await openChangeForm();
		await fill(r, 'vat-change-rate', '8.1');
		await fill(r, 'vat-change-from', '2027-01-01');
		await fireEvent.click(r.getByTestId('vat-change-submit'));
		await waitFor(() => expect(r.getByTestId('vat-change-error').textContent).toMatch(/clôturé/));
		const t = r.getByTestId('vat-change-error').textContent ?? '';
		expect(t).toContain('Chevauchement avec un autre taux.');
		expect(t).toMatch(/même date/);
		expect(listMock).toHaveBeenCalledTimes(2);
	});

	it('createVatRate refusé après la clôture (erreur non typée) : « clôturé » sans motif', async () => {
		updateMock.mockResolvedValue({});
		createMock.mockRejectedValue(new Error('réseau'));
		const r = await openChangeForm();
		await fill(r, 'vat-change-rate', '8.1');
		await fill(r, 'vat-change-from', '2027-01-01');
		await fireEvent.click(r.getByTestId('vat-change-submit'));
		await waitFor(() => expect(r.getByTestId('vat-change-error').textContent).toMatch(/clôturé/));
		expect(r.getByTestId('vat-change-error').textContent).toMatch(/même date/);
		expect(listMock).toHaveBeenCalledTimes(2);
	});

	it('updateVatRate lui-même refusé : createVatRate non appelé, le message ne dit pas qu’un taux a été clôturé', async () => {
		updateMock.mockRejectedValue({ code: 'VALIDATION_ERROR', status: 400, message: 'Date de bascule invalide.' });
		const r = await openChangeForm();
		await fill(r, 'vat-change-rate', '8.1');
		await fill(r, 'vat-change-from', '2027-01-01');
		await fireEvent.click(r.getByTestId('vat-change-submit'));
		await waitFor(() => expect(r.getByTestId('vat-change-error').textContent).toBe('Date de bascule invalide.'));
		expect(createMock).not.toHaveBeenCalled();
		expect(r.getByTestId('vat-change-error').textContent).not.toMatch(/clôtur/);
	});

	it('updateVatRate refusé par une erreur non typée : message générique, sans « clôturé »', async () => {
		updateMock.mockRejectedValue(new Error('réseau'));
		const r = await openChangeForm();
		await fill(r, 'vat-change-rate', '8.1');
		await fill(r, 'vat-change-from', '2027-01-01');
		await fireEvent.click(r.getByTestId('vat-change-submit'));
		await waitFor(() => expect(r.getByTestId('vat-change-error').textContent).toBe('Le changement de taux a échoué.'));
		expect(createMock).not.toHaveBeenCalled();
	});
});
