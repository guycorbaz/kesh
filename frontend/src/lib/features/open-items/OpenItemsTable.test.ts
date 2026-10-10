// Story 15-1c-i (#518) — la liste des postes ouverts (AC2, AC3, AC4, AC8, AC10).
import { describe, it, expect, vi, afterEach } from 'vitest';
import { render, cleanup, fireEvent } from '@testing-library/svelte';

vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_k: string, fallback: string, args?: Record<string, string | number>) =>
		args ? fallback.replace(/\{\s*\$(\w+)\s*\}/g, (_, n) => String(args[n] ?? '')) : fallback,
}));

import OpenItemsTable from './OpenItemsTable.svelte';
import { item, view } from './open-items.test.fixtures';
import type { OpenItemsResponse } from './open-items.types';

afterEach(() => cleanup());

function renderTable(data: OpenItemsResponse, over: Record<string, unknown> = {}) {
	const onToggle = vi.fn();
	const onPage = vi.fn();
	const r = render(OpenItemsTable, {
		data,
		today: '2026-03-31',
		canWrite: true,
		isSelected: () => false,
		onToggle,
		onPage,
		...over,
	});
	return { ...r, onToggle, onPage };
}

describe('AC2 — colonnes, exercice et numéro, pièces, ordre (test 3)', () => {
	it('chaque ligne montre date, « exercice n° numéro » en lien vers la fiche, journal, libellé, montants', () => {
		const { getByTestId } = renderTable(view());
		const row = getByTestId('open-item-row-1');
		expect(row.textContent).toContain('04.03.2026');
		expect(row.textContent).toContain('Exercice 2026 n° 12');
		expect(row.querySelector('a[href="/journal-entries/10"]')).not.toBeNull();
		expect(row.textContent).toContain('OD');
		expect(row.textContent).toContain('Avance au fournisseur');
		expect(row.textContent).toContain('100.00');
	});

	it('les cinq types de pièce : quatre liens possibles, règlement sans facture et transaction sans lien', () => {
		const doc = { id: 5, number: 'N-1', invoiceId: null, invoiceNumber: null };
		const data = view({
			items: [
				item({ lineId: 1, document: { ...doc, type: 'invoice' } }),
				item({ lineId: 2, document: { ...doc, type: 'creditNote' } }),
				item({ lineId: 3, document: { ...doc, type: 'supplierInvoice' } }),
				item({ lineId: 4, document: { ...doc, type: 'settlement', invoiceId: 9, invoiceNumber: 'F-9' } }),
				item({ lineId: 5, document: { ...doc, type: 'settlement', invoiceId: null, invoiceNumber: 'F-0' } }),
				item({ lineId: 6, document: { ...doc, type: 'bankTransaction' } }),
			],
		});
		const { getByTestId } = renderTable(data);
		const href = (id: number) =>
			getByTestId(`open-item-row-${id}`).querySelector('[data-testid="open-items-document-link"]')
				?.getAttribute('href') ?? null;
		expect(href(1)).toBe('/invoices/5');
		expect(href(2)).toBe('/credit-notes/5');
		expect(href(3)).toBe('/supplier-invoices/5');
		expect(href(4)).toBe('/invoices/9');
		expect(href(5)).toBeNull();
		expect(getByTestId('open-item-row-5').textContent).toContain('règlement de F-0');
		expect(href(6)).toBeNull();
		expect(getByTestId('open-item-row-6').textContent).toContain('transaction bancaire');
	});

	it('l’ordre du serveur est conservé, même quand il n’est pas celui des dates', () => {
		const data = view({
			items: [
				item({ lineId: 3, date: '2026-03-20' }),
				item({ lineId: 1, date: '2026-01-02' }),
				item({ lineId: 2, date: '2026-02-10' }),
			],
		});
		const { container } = renderTable(data);
		const ids = [...container.querySelectorAll('[data-testid^="open-item-row-"]')].map((r) =>
			r.getAttribute('data-testid'),
		);
		expect(ids).toEqual(['open-item-row-3', 'open-item-row-1', 'open-item-row-2']);
	});

	it('pagination : la page suivante est demandée par son offset', async () => {
		const { getByTestId, onPage } = renderTable(view({ total: 120, limit: 50 }));
		expect(getByTestId('open-items-range').textContent).toContain('1–1 sur 120');
		await fireEvent.click(getByTestId('open-items-next'));
		expect(onPage).toHaveBeenCalledWith(50);
	});
});

describe('AC3 — motifs, code lettré, note « état d’aujourd’hui » (test 4)', () => {
	it('lettrée après la date : le code en lien vers le groupe, et la date du lettrage', () => {
		const data = view({
			items: [
				item({
					reason: 'letteredAfterAsOf',
					letteringCode: 'AB',
					letteredOn: '2026-04-02',
					manuallyLetterable: false,
				}),
			],
		});
		const { getByTestId } = renderTable(data);
		const reason = getByTestId('open-item-reason-1');
		expect(reason.textContent).toContain('lettrée après cette date');
		expect(reason.textContent).toContain('le 02.04.2026');
		expect(getByTestId('lettering-code-link-AB').getAttribute('href')).toBe(
			'/open-items?accountId=7&asOf=2026-03-31&group=AB',
		);
	});

	it('documentState nul → aucune seconde ligne ; sinon l’état et le reste dû', () => {
		const data = view({
			items: [
				item({ lineId: 1 }),
				item({ lineId: 2, documentState: 'unpaid', amountDue: '80.5000' }),
			],
		});
		const { queryByTestId, getByTestId } = renderTable(data);
		expect(queryByTestId('open-item-state-1')).toBeNull();
		expect(getByTestId('open-item-state-2').textContent).toContain('reste dû : 80.50');
	});

	it('la note « état d’aujourd’hui » paraît pour une date passée, et pas pour aujourd’hui', () => {
		const past = renderTable(view({ asOf: '2026-01-31' }));
		expect(past.queryByTestId('open-items-today-note')).not.toBeNull();
		past.unmount();
		const now = renderTable(view({ asOf: '2026-03-31' }));
		expect(now.queryByTestId('open-items-today-note')).toBeNull();
	});
});

describe('AC4, AC10 — la case (test 5, test 11)', () => {
	it('case présente SSI manuallyLetterable — dont une ligne de transaction bancaire', () => {
		const data = view({
			items: [
				item({
					lineId: 1,
					document: { type: 'bankTransaction', id: 3, number: null, invoiceId: null, invoiceNumber: null },
				}),
				item({
					lineId: 2,
					manuallyLetterable: false,
					document: { type: 'invoice', id: 4, number: 'F-4', invoiceId: null, invoiceNumber: null },
				}),
			],
		});
		const { queryByTestId, getByTestId } = renderTable(data);
		expect(queryByTestId('open-item-select-1')).not.toBeNull();
		expect(queryByTestId('open-item-select-2')).toBeNull();
		expect(getByTestId('open-item-no-select-2').getAttribute('title')).toBe(
			'son lettrage suit sa pièce et ses règlements',
		);
	});

	it('cocher remonte la ligne', async () => {
		const { getByTestId, onToggle } = renderTable(view());
		await fireEvent.click(getByTestId('open-item-select-1'));
		expect(onToggle).toHaveBeenCalledWith(expect.objectContaining({ lineId: 1 }));
	});

	it('Consultation : aucune case, aucun motif d’absence de case', () => {
		const data = view({
			items: [item({ lineId: 1 }), item({ lineId: 2, manuallyLetterable: false, documentState: 'nothingDue' })],
		});
		const { container } = renderTable(data, { canWrite: false });
		expect(container.querySelector('input[type="checkbox"]')).toBeNull();
		expect(container.querySelector('[data-testid^="open-item-no-select-"]')).toBeNull();
	});
});

describe('AC8 — le pied (test 10)', () => {
	it('compte fournisseurs : « 500.00 créditeur » aux deux, et la phrase d’égalité', () => {
		const { getByTestId } = renderTable(view({ openTotal: '-500.0000', balance: '-500.0000' }));
		expect(getByTestId('open-items-open-total').textContent?.trim()).toBe('500.00 créditeur');
		expect(getByTestId('open-items-balance').textContent?.trim()).toBe('500.00 créditeur');
		expect(getByTestId('open-items-balance-equal').textContent).toContain('du côté naturel du compte');
	});

	it('débiteur et zéro sans sens', () => {
		const a = renderTable(view({ openTotal: '20.0000', balance: '20.0000' }));
		expect(a.getByTestId('open-items-open-total').textContent?.trim()).toBe('20.00 débiteur');
		a.unmount();
		const b = renderTable(view({ openTotal: '0.0000', balance: '0.0000', items: [] }));
		expect(b.getByTestId('open-items-open-total').textContent?.trim()).toBe('0.00');
	});

	it('openTotal ≠ balance : les deux montants, sans phrase d’égalité (défaut jamais masqué)', () => {
		const { getByTestId, queryByTestId } = renderTable(
			view({ openTotal: '-500.0000', balance: '-480.0000' }),
		);
		expect(getByTestId('open-items-open-total').textContent).toContain('500.00');
		expect(getByTestId('open-items-balance').textContent).toContain('480.00');
		expect(queryByTestId('open-items-balance-equal')).toBeNull();
	});
});
