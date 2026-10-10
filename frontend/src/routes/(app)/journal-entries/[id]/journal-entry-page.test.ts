// Story 15-1c-ii (AC9, tests 1 et 2) — la fiche d'une écriture montre le
// lettrage de ses lignes, et le motif `ENTRY_LETTERED` porte le lien du groupe.
//
// ⚠️ Cette page n'avait AUCUN test Vitest avant cette story : ses gestes
// (modifier, supprimer, contre-passer) sont couverts par l'E2E. Le harnais est
// calqué sur celui des pages voisines : mocks hoistés AVANT l'import du
// composant, `authState` piloté par `login`.
//
// ⚠️ `i18nMsg` rend le repli : les assertions portent sur le texte français.

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/svelte';
import { authState } from '$lib/app/stores/auth.svelte';
import type { JournalEntryDetailResponse } from '$lib/features/journal-entries/journal-entries.types';
import type { AccountResponse } from '$lib/features/accounts/accounts.types';

vi.mock('$app/environment', () => ({ browser: true }));
vi.mock('$app/navigation', () => ({ goto: vi.fn() }));
vi.mock('$app/state', () => ({ page: { params: { id: '7' } } }));
vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string) => fallback
}));
/** Les clés demandées par la page, pour prouver la clé et non le seul repli. */
const keysAsked = vi.hoisted(() => new Set<string>());
vi.mock('$lib/features/onboarding/onboarding.svelte', () => ({
	i18nMsg: (key: string, fallback: string) => {
		keysAsked.add(key);
		return fallback;
	}
}));
vi.mock('svelte-sonner', () => ({ toast: { success: vi.fn(), error: vi.fn() } }));

const getJournalEntryMock = vi.fn<() => Promise<JournalEntryDetailResponse>>();
vi.mock('$lib/features/journal-entries/journal-entries.api', () => ({
	getJournalEntry: () => getJournalEntryMock(),
	deleteJournalEntry: vi.fn(),
	reverseJournalEntry: vi.fn()
}));
const fetchAccountsMock = vi.fn<() => Promise<AccountResponse[]>>();
vi.mock('$lib/features/accounts/accounts.api', () => ({
	fetchAccounts: () => fetchAccountsMock()
}));
vi.mock('$lib/features/projects/projects.api', () => ({
	listProjects: () =>
		Promise.resolve([
			{ id: 3, code: 'P1', name: 'Projet un', parentId: null, active: true }
		])
}));
vi.mock('$lib/features/fiscal-years/fiscal-years.api', () => ({ listFiscalYears: vi.fn() }));
vi.mock('$lib/features/settings/settings.api', () => ({ fetchCompanyCurrent: vi.fn() }));
vi.mock('$lib/features/invoices/invoices.api', () => ({ getInvoiceSettings: vi.fn() }));

import Page from './+page.svelte';

type Line = JournalEntryDetailResponse['lines'][number];

function line(over: Partial<Line> = {}): Line {
	return {
		id: 1,
		accountId: 10,
		lineOrder: 1,
		debit: '100.00',
		credit: '0.00',
		projectId: null,
		letteringKey: null,
		letteringCode: null,
		letteringOrigin: null,
		...over
	};
}

function entry(over: Partial<JournalEntryDetailResponse> = {}): JournalEntryDetailResponse {
	return {
		id: 7,
		companyId: 1,
		fiscalYearId: 1,
		entryNumber: 12,
		entryDate: '2026-03-04',
		journal: 'Banque',
		description: 'Acompte',
		version: 1,
		reversesEntryId: null,
		lines: [
			line({ id: 1, accountId: 10, debit: '100.00', credit: '0.00', letteringKey: 4, letteringCode: 'AB', letteringOrigin: 'manual' }),
			line({ id: 2, accountId: 11, debit: '0.00', credit: '100.00', lineOrder: 2 })
		],
		createdAt: '2026-03-04T00:00:00',
		updatedAt: '2026-03-04T00:00:00',
		reversedByEntryId: null,
		reversable: true,
		reversalBlockedBy: null,
		reversalBlockedLabel: null,
		modifiable: true,
		modificationBlockedBy: null,
		modificationBlockedLabel: null,
		...over
	};
}

/** La somme des `colspan` d'une rangée (1 par cellule sans attribut). */
function span(tr: Element): number {
	return Array.from(tr.children).reduce(
		(acc, td) => acc + Number(td.getAttribute('colspan') ?? '1'),
		0
	);
}

async function renderEntry(e: JournalEntryDetailResponse) {
	getJournalEntryMock.mockResolvedValue(e);
	const r = render(Page);
	await waitFor(() => expect(screen.getByText(`Écriture n°${e.entryNumber}`)).toBeTruthy());
	return r;
}

beforeEach(() => {
	fetchAccountsMock.mockResolvedValue([]);
	authState.login({ userId: '1', username: 'admin', role: 'Admin', expiresIn: 3600 });
});

afterEach(async () => {
	await authState.logout();
	vi.clearAllMocks();
});

describe('fiche d’écriture — colonne « Lettrage » (15-1c-ii, test 1)', () => {
	it('une ligne lettrée montre son code en lien vers le groupe ; une ligne ouverte, rien', async () => {
		await renderEntry(entry());
		const lettered = screen.getByTestId('entry-line-lettering-1');
		const a = lettered.querySelector('a');
		expect(a?.textContent).toBe('AB');
		expect(a?.getAttribute('href')).toBe('/open-items?group=AB');
		const open = screen.getByTestId('entry-line-lettering-2');
		expect(open.textContent?.trim()).toBe('');
		expect(open.querySelector('a')).toBeNull();
	});

	it('l’en-tête est traduit et c’est la dernière colonne', async () => {
		const { container } = await renderEntry(entry());
		const ths = Array.from(container.querySelectorAll('table thead th')).map((th) =>
			th.textContent?.trim()
		);
		expect(ths.at(-1)).toBe('Lettrage');
		expect(keysAsked.has('journal-entries-column-lettering')).toBe(true);
		// La cellule lettrée est la dernière de sa rangée.
		const cell = screen.getByTestId('entry-line-lettering-1');
		expect(cell.parentElement?.lastElementChild).toBe(cell);
	});

	for (const withProject of [false, true]) {
		it(`chaque rangée, pied « Total » compris, couvre les en-têtes — ${withProject ? 'avec' : 'sans'} projets`, async () => {
			const e = entry();
			if (withProject) e.lines[0].projectId = 3;
			const { container } = await renderEntry(e);
			const headers = container.querySelectorAll('table thead th').length;
			expect(headers).toBe(withProject ? 5 : 4);
			const rows = container.querySelectorAll('table tbody tr, table tfoot tr');
			expect(rows.length).toBe(3);
			for (const tr of rows) expect(span(tr)).toBe(headers);
			// Les totaux tombent sous « Débit » et « Crédit », le libellé avant.
			const ths = Array.from(container.querySelectorAll('table thead th')).map((th) =>
				th.textContent?.trim()
			);
			const cols: string[] = [];
			for (const td of Array.from(container.querySelector('table tfoot tr')!.children)) {
				const n = Number(td.getAttribute('colspan') ?? '1');
				for (let i = 0; i < n; i++) cols.push(td.textContent?.trim() ?? '');
			}
			expect(cols[ths.indexOf('Débit')]).toBe('100.00');
			expect(cols[ths.indexOf('Crédit')]).toBe('100.00');
			expect(cols[ths.indexOf('Débit') - 1]).toBe('Total');
			expect(cols.at(-1)).toBe('');
		});
	}
});

describe('fiche d’écriture — motif `ENTRY_LETTERED` (15-1c-ii, test 2)', () => {
	const lettered = () =>
		entry({
			modifiable: false,
			modificationBlockedBy: 'ENTRY_LETTERED',
			modificationBlockedLabel: 'AB'
		});

	it('le code du motif est un lien vers le groupe', async () => {
		await renderEntry(lettered());
		const reason = screen.getByTestId('modification-blocked-reason');
		// Texte exact : le code n'apparaît qu'une fois, dans le lien — jamais
		// suffixé par `modificationMessage` en plus (« (AB) (AB) »).
		expect(reason.textContent?.replace(/\s+/g, ' ').trim()).toBe(
			'Cette écriture est lettrée : délettrez-la d’abord. (AB)'
		);
		const a = reason.querySelector('a');
		expect(a?.textContent).toBe('AB');
		expect(a?.getAttribute('href')).toBe('/open-items?group=AB');
	});

	it('`ENTRY_LETTERED` sans code (libellé nul) : le motif reste du texte, sans lien vide', async () => {
		await renderEntry(
			entry({
				modifiable: false,
				modificationBlockedBy: 'ENTRY_LETTERED',
				modificationBlockedLabel: null
			})
		);
		const reason = screen.getByTestId('modification-blocked-reason');
		expect(reason.textContent).toContain('Cette écriture est lettrée');
		expect(reason.querySelector('a')).toBeNull();
	});

	it('les autres motifs restent du texte (pièce, exercice)', async () => {
		for (const e of [
			entry({
				modifiable: false,
				reversable: false,
				reversalBlockedBy: 'OWNED_BY_INVOICE',
				reversalBlockedLabel: 'F-2026-0001',
				modificationBlockedBy: 'OWNED_BY_INVOICE',
				modificationBlockedLabel: 'F-2026-0001',
				// Même code que la contre-passation : le motif de modification se
				// tait — le motif de contre-passation le dit, en texte.
			}),
			entry({
				modifiable: false,
				modificationBlockedBy: 'OWNED_BY_INVOICE',
				modificationBlockedLabel: 'F-2026-0002'
			}),
			entry({
				modifiable: false,
				modificationBlockedBy: 'FISCAL_YEAR_CLOSED',
				modificationBlockedLabel: null
			})
		]) {
			const { container, unmount } = await renderEntry(e);
			const reasons = container.querySelectorAll(
				'[data-testid="modification-blocked-reason"], [data-testid="reverse-blocked-reason"]'
			);
			expect(reasons.length).toBeGreaterThan(0);
			for (const r of reasons) expect(r.querySelector('a')).toBeNull();
			unmount();
		}
	});

	it('le rôle Consultation ne voit pas de motif (C-15-8-14), mais voit la colonne', async () => {
		await authState.logout();
		authState.login({ userId: '2', username: 'lecteur', role: 'Consultation', expiresIn: 3600 });
		await renderEntry(lettered());
		expect(screen.queryByTestId('modification-blocked-reason')).toBeNull();
		expect(screen.getByTestId('entry-line-lettering-1').querySelector('a')?.textContent).toBe('AB');
	});
});
