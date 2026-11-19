// Story 48-10 (#1175 ; AC2, AC5, AC7, AC12) — les frais d'un niveau de rappel se lisent comme le
// montant d'une écriture. Avant la story, la saisie partait brute (`feeAmount: fFee`) : « 1.250 »
// recevait du serveur « Les frais de rappel ne peuvent avoir plus de 2 décimales. », qui poussait
// à écrire `1.25` pour mille deux cent cinquante ; « 1'250 » était refusé (`Number` le lit `NaN`) ;
// « 1e3 » et « 10.01000 », que `Number` lit, partaient bruts.
//
// Premier test de cette page. Patron des `*-page.test.ts` : mocks hissés AVANT l'import du
// composant ; le mock d'`i18nMsg` interpole `{ $max }` (M10a).

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, fireEvent, cleanup, waitFor } from '@testing-library/svelte';
import type { DunningLevelResponse } from '$lib/features/dunning';

vi.mock('$app/environment', () => ({ browser: true }));
vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_k: string, fallback: string, args?: Record<string, string | number>) =>
		args ? fallback.replace(/\{\s*\$(\w+)\s*\}/g, (_, n) => String(args[n] ?? '')) : fallback,
}));
vi.mock('svelte-sonner', () => ({ toast: { success: vi.fn(), error: vi.fn() } }));

const listMock = vi.fn();
const createMock = vi.fn();
const updateMock = vi.fn();
vi.mock('$lib/features/dunning', () => ({
	listDunningLevels: () => listMock(),
	createDunningLevel: (...a: unknown[]) => createMock(...a),
	updateDunningLevel: (...a: unknown[]) => updateMock(...a),
	deleteDunningLevel: vi.fn(),
	getDunningSettings: vi.fn(async () => ({ gracePeriodDays: 5, seededAt: null, version: 1 })),
	updateDunningSettings: vi.fn(),
}));

import Page from './+page.svelte';

/** Un niveau tel que le serveur le rend (`DECIMAL(7,2)` : deux décimales). */
const NIVEAU: DunningLevelResponse = {
	id: 2,
	levelNumber: 1,
	delayDays: 10,
	feeAmount: '10.00',
	version: 3,
} as DunningLevelResponse;

const AMBIGU = 'Montant ambigu';
const ILLISIBLE_2 = '2 décimales au plus';

beforeEach(() => {
	listMock.mockResolvedValue([NIVEAU]);
	createMock.mockResolvedValue({ ...NIVEAU, id: 9 });
	updateMock.mockResolvedValue(NIVEAU);
});
afterEach(() => {
	cleanup();
	vi.clearAllMocks();
});

const frais = () => document.querySelector<HTMLInputElement>('[data-testid="dunning-form-fee"]')!;
const messageSousLeChamp = () => document.querySelector('[data-testid="dunning-form-fee-error"]');
const blocCommun = () => document.querySelector('[data-testid="dunning-form-error"]');

async function ouvrir(mode: 'create' | 'edit'): Promise<void> {
	const { findByTestId, findAllByText } = render(Page);
	if (mode === 'create') {
		await fireEvent.click(await findByTestId('dunning-level-new'));
	} else {
		await fireEvent.click((await findAllByText('Modifier'))[0]);
	}
	// Sonde positive : le champ est TROUVÉ avant toute assertion d'absence.
	if (!frais()) throw new Error('champ des frais introuvable : formulaire non monté');
}

async function taper(valeur: string): Promise<void> {
	await fireEvent.input(frais(), { target: { value: valeur } });
}

async function enregistrer(): Promise<void> {
	await fireEvent.click(document.querySelector('[data-testid="dunning-form-submit"]')!);
	await new Promise((r) => setTimeout(r, 0));
}

describe('niveaux de rappel — frais ambigus (Story 48-10, #1175)', () => {
	// AC2 — rouge avant le correctif : `feeAmount: '1.250'` partait.
	for (const mode of ['create', 'edit'] as const) {
		it(`${mode} : « 1.250 » → message ambigu sous le champ, rien ne part, formulaire ouvert, saisie conservée`, async () => {
			await ouvrir(mode);
			await taper('1.250');
			expect(messageSousLeChamp()?.textContent).toContain(AMBIGU);
			await enregistrer();
			expect(createMock).not.toHaveBeenCalled();
			expect(updateMock).not.toHaveBeenCalled();
			expect(frais()).not.toBeNull();
			expect(frais().value).toBe('1.250');
			// Hors du bloc commun (D2) : celui-ci ne désigne pas le champ.
			expect(blocCommun()).toBeNull();
		});
	}

	// AC2 — M4a : rouge avant le correctif (« 1'250 » refusé, `Number` le lit `NaN`).
	it("création : « 1'250 » part `1250` (M4a)", async () => {
		await ouvrir('create');
		await taper("1'250");
		await enregistrer();
		await waitFor(() => expect(createMock).toHaveBeenCalledTimes(1));
		expect(createMock.mock.calls[0][0]).toEqual({ delayDays: 10, feeAmount: '1250' });
	});

	// AC2 — M4b.
	it("modification : « 1'250 » part `1250` (M4b)", async () => {
		await ouvrir('edit');
		await taper("1'250");
		await enregistrer();
		await waitFor(() => expect(updateMock).toHaveBeenCalledTimes(1));
		expect(updateMock.mock.calls[0][0]).toBe(NIVEAU.id);
		expect(updateMock.mock.calls[0][1]).toEqual({ delayDays: 10, feeAmount: '1250', version: 3 });
	});

	// AC2 — M5 : rogné avant la lecture et l'envoi.
	it('«  12.50  » (bordé de blancs) part `12.50`, sans message (M5)', async () => {
		await ouvrir('create');
		await taper('  12.50  ');
		expect(messageSousLeChamp()).toBeNull();
		await enregistrer();
		await waitFor(() => expect(createMock).toHaveBeenCalledTimes(1));
		expect(createMock.mock.calls[0][0]).toMatchObject({ feeAmount: '12.50' });
	});

	// AC2 — M10a : le message illisible dit « 2 décimales au plus ».
	it("« 12'34 » → message illisible, « 2 décimales au plus », rien ne part (M10a)", async () => {
		await ouvrir('create');
		await taper("12'34");
		expect(messageSousLeChamp()?.textContent).toContain('Montant illisible');
		expect(messageSousLeChamp()?.textContent).toContain(ILLISIBLE_2);
		await enregistrer();
		expect(createMock).not.toHaveBeenCalled();
	});

	// AC2, AC5 — ce que `Number` lisait et que la grammaire du journal ne lit pas.
	for (const saisie of ['-5', '1e3', '10.01000', '.5', '5.']) {
		it(`« ${saisie} » → refus sans appel`, async () => {
			await ouvrir('create');
			await taper(saisie);
			expect(messageSousLeChamp()?.textContent).toContain('Montant illisible');
			await enregistrer();
			expect(createMock).not.toHaveBeenCalled();
		});
	}

	// Le vide garde son message du bloc commun (inchangé).
	it('frais vides : le message du bloc commun, rien ne part', async () => {
		await ouvrir('create');
		await taper('');
		expect(messageSousLeChamp()).toBeNull();
		await enregistrer();
		expect(blocCommun()?.textContent).toContain('Les frais doivent être un montant valide.');
		expect(createMock).not.toHaveBeenCalled();
	});

	// AC5 — rien de juste n'est refusé ; la charge part normalisée.
	for (const [saisie, charge] of [
		['1.25', '1.25'],
		['1250', '1250'],
		["1'250.50", '1250.50'],
		['1’250', '1250'],
		['1,25', '1.25'],
		['0.50', '0.50'],
		[' 1250 ', '1250'],
	]) {
		it(`« ${saisie} » accepté sans message, part « ${charge} »`, async () => {
			await ouvrir('create');
			await taper(saisie);
			expect(messageSousLeChamp()).toBeNull();
			await enregistrer();
			await waitFor(() => expect(createMock).toHaveBeenCalledTimes(1));
			expect(createMock.mock.calls[0][0]).toMatchObject({ feeAmount: charge });
		});
	}

	// AC2, AC7 — FILET (M12 le tue) : un niveau rouvert sans toucher aux frais s'enregistre sans
	// message, à la valeur d'origine.
	it('un niveau rouvert à 10.00 s’enregistre sans message, à la même valeur (M12)', async () => {
		await ouvrir('edit');
		expect(frais().value).toBe('10.00');
		expect(messageSousLeChamp()).toBeNull();
		await enregistrer();
		await waitFor(() => expect(updateMock).toHaveBeenCalledTimes(1));
		expect(updateMock.mock.calls[0][1]).toMatchObject({ feeAmount: '10.00' });
	});

	// AC12 — accessibilité du refus (M16).
	it('« 1.250 » : aria-invalid et aria-describedby nomment le message ; corrigé, ils le quittent', async () => {
		await ouvrir('create');
		await taper('1.250');
		const msg = messageSousLeChamp()!;
		expect(msg.id).not.toBe('');
		expect(msg.getAttribute('role')).toBe('alert');
		expect(frais().getAttribute('aria-invalid')).toBe('true');
		expect((frais().getAttribute('aria-describedby') ?? '').split(' ')).toContain(msg.id);
		const id = msg.id;
		await taper("1'250");
		expect(['false', null]).toContain(frais().getAttribute('aria-invalid'));
		expect((frais().getAttribute('aria-describedby') ?? '').split(' ')).not.toContain(id);
	});

	// Revue P1 (B3) — le refus est rendu SOUS LE CHAMP des frais (dans son bloc), non sous la rangée.
	it('le refus des frais est dans le bloc du champ des frais (M22)', async () => {
		await ouvrir('create');
		await taper('1.250');
		expect(frais().parentElement!.contains(messageSousLeChamp())).toBe(true);
	});

	// Revue P1 (B2) — le message du bloc commun ne reste pas empilé, périmé, sous le refus du champ.
	it('frais vides puis « 1.250 » : le message du bloc commun s’efface, le refus du champ reste (M23)', async () => {
		await ouvrir('create');
		await taper('');
		await enregistrer();
		expect(blocCommun()?.textContent).toContain('Les frais doivent être un montant valide.');
		await taper('1.250');
		expect(blocCommun()).toBeNull();
		expect(messageSousLeChamp()?.textContent).toContain(AMBIGU);
	});

	it('un refus du délai n’est pas effacé par une saisie des frais', async () => {
		await ouvrir('create');
		await fireEvent.input(document.querySelector('[data-testid="dunning-form-delay"]')!, {
			target: { value: '-1' },
		});
		await enregistrer();
		expect(blocCommun()?.textContent).toContain('Le délai doit être un entier positif.');
		await taper('12.50');
		expect(blocCommun()?.textContent).toContain('Le délai doit être un entier positif.');
	});

	// Revue P1 (B4) — la lecture coïncide avec le message « 2 décimales au plus ».
	for (const saisie of ['10.1250', '0.125', '1.2345']) {
		it(`« ${saisie} » → illisible, « 2 décimales au plus », rien ne part (M21)`, async () => {
			await ouvrir('create');
			await taper(saisie);
			expect(messageSousLeChamp()?.textContent).toContain(ILLISIBLE_2);
			await enregistrer();
			expect(createMock).not.toHaveBeenCalled();
		});
	}

	// Revue P1 (B4) — les zéros de fin ne comptent pas, et la charge est ramenée au centime : le
	// serveur des frais compte les décimales écrites (`scale_within(fee, 2)`, sans `normalize`).
	it('« 10.0100 » accepté sans message, part `10.01`', async () => {
		await ouvrir('create');
		await taper('10.0100');
		expect(messageSousLeChamp()).toBeNull();
		await enregistrer();
		await waitFor(() => expect(createMock).toHaveBeenCalledTimes(1));
		expect(createMock.mock.calls[0][0]).toMatchObject({ feeAmount: '10.01' });
	});
});
