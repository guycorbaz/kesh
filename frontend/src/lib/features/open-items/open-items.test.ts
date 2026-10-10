// Story 15-1c-i (#518) — la logique pure de l'écran des postes ouverts.
//
// Chaque test nomme la mutation qu'il attrape.
import { describe, it, expect } from 'vitest';
import {
	MAX_LINES_PER_GROUP,
	documentHref,
	groupDocument,
	groupHref,
	isIsoDate,
	isStaleRefusal,
	letterBlocker,
	parseScreenState,
	sameAmount,
	screenUrl,
	selectionSum,
	sideOf,
	todayLocal,
} from './open-items';
import { group, groupLine, refusal } from './open-items.test.fixtures';
import type { DocumentRef, SelectedLine } from './open-items.types';

function sel(lineId: number, debit: string, credit: string, inOpenPeriod = true): SelectedLine {
	return { lineId, debit, credit, inOpenPeriod };
}

describe('AC1 — dates et état d’URL (test 1)', () => {
	it('todayLocal lit la date LOCALE, jamais l’UTC (mutation : toISOString)', () => {
		// 23:30 le 31 décembre en heure locale : en UTC+1 c'est déjà… le même jour
		// local ; la date construite par champs locaux ne dépend pas du fuseau.
		const d = new Date(2026, 11, 31, 23, 30);
		expect(todayLocal(d)).toBe('2026-12-31');
		expect(todayLocal(new Date(2026, 0, 5, 0, 10))).toBe('2026-01-05');
	});

	it('isIsoDate refuse les dates impossibles et hors plage du serveur', () => {
		expect(isIsoDate('2026-02-28')).toBe(true);
		expect(isIsoDate('2028-02-29')).toBe(true);
		for (const bad of ['2026-02-30', '2026-13-01', '26-01-01', '0999-01-01', 'hier', '', null]) {
			expect(isIsoDate(bad), String(bad)).toBe(false);
		}
	});

	it('parseScreenState lit accountId, asOf et group ; une valeur invalide rend null', () => {
		expect(parseScreenState(new URLSearchParams('accountId=7&asOf=2026-03-31&group=AA'))).toEqual({
			accountId: 7,
			asOf: '2026-03-31',
			group: 'AA',
		});
		expect(parseScreenState(new URLSearchParams('accountId=x&asOf=2026-02-30&group=%20'))).toEqual({
			accountId: null,
			asOf: null,
			group: null,
		});
		expect(parseScreenState(new URLSearchParams('accountId=0')).accountId).toBeNull();
	});

	it('screenUrl écrit l’état et retire ce qui est nul', () => {
		const base = new URL('http://localhost/open-items?accountId=1&group=AB');
		const u = screenUrl(base, { accountId: 7, asOf: '2026-03-31', group: null });
		expect(u.pathname).toBe('/open-items');
		expect(u.searchParams.get('accountId')).toBe('7');
		expect(u.searchParams.get('asOf')).toBe('2026-03-31');
		expect(u.searchParams.has('group')).toBe(false);
	});

	it('groupHref : le groupe seul, ou avec compte et date', () => {
		expect(groupHref('AA')).toBe('/open-items?group=AA');
		expect(groupHref('AA', 7, '2026-03-31')).toBe(
			'/open-items?accountId=7&asOf=2026-03-31&group=AA',
		);
	});
});

describe('AC2 — la table des liens de pièce (test 3)', () => {
	const doc = (over: Partial<DocumentRef>): DocumentRef => ({
		type: 'invoice',
		id: 5,
		number: 'F-1',
		invoiceId: null,
		invoiceNumber: null,
		...over,
	});

	it('cinq types, cinq cibles', () => {
		expect(documentHref(doc({ type: 'invoice' }))).toBe('/invoices/5');
		expect(documentHref(doc({ type: 'creditNote' }))).toBe('/credit-notes/5');
		expect(documentHref(doc({ type: 'supplierInvoice' }))).toBe('/supplier-invoices/5');
		expect(documentHref(doc({ type: 'settlement', invoiceId: 9 }))).toBe('/invoices/9');
		// Un règlement sans facture n'a PAS de lien (mutation : lien vers /invoices/null).
		expect(documentHref(doc({ type: 'settlement', invoiceId: null }))).toBeNull();
		// Aucune route n'ouvre une transaction (mutation : lien vers /bank-import/{id}).
		expect(documentHref(doc({ type: 'bankTransaction' }))).toBeNull();
	});
});

describe('AC6 — le numéro de la pièce d’un groupe `document` (C-15-1c-25)', () => {
	it('facture + avoir → le numéro de la FACTURE, même si l’avoir vient d’abord', () => {
		const g = group({
			origin: 'document',
			lines: [
				groupLine({
					id: 1,
					document: { type: 'creditNote', id: 3, number: 'AV-1', invoiceId: null, invoiceNumber: null },
				}),
				groupLine({
					id: 2,
					document: { type: 'invoice', id: 4, number: 'F-9', invoiceId: null, invoiceNumber: null },
				}),
			],
		});
		expect(groupDocument(g)).toEqual({ number: 'F-9', href: '/invoices/4' });
	});

	it('sans facture : l’invoiceNumber du premier règlement ; sinon null', () => {
		const settled = group({
			lines: [
				groupLine({
					document: { type: 'settlement', id: 8, number: null, invoiceId: 4, invoiceNumber: 'F-9' },
				}),
			],
		});
		expect(groupDocument(settled)).toEqual({ number: 'F-9', href: '/invoices/4' });
		expect(groupDocument(group())).toBeNull();
	});
});

describe('AC4 — somme et état du bouton (test 6)', () => {
	it('0.1 + 0.2 − 0.3 à quatre décimales donne un zéro EXACT (mutation : parseFloat)', () => {
		const s = selectionSum([sel(1, '0.1000', '0'), sel(2, '0.2000', '0'), sel(3, '0', '0.3000')]);
		expect(s.eq(0)).toBe(true);
		// En flottant, ce test rougit : 0.1 + 0.2 − 0.3 ≠ 0.
		expect(0.1 + 0.2 - 0.3).not.toBe(0);
	});

	it('moins de deux lignes → « au moins deux »', () => {
		expect(letterBlocker([])).toEqual({ kind: 'tooFew' });
		expect(letterBlocker([sel(1, '0', '0')])).toEqual({ kind: 'tooFew' });
	});

	it('201 lignes → « 200 au plus », AVANT l’écart (ordre de la fiche)', () => {
		const lines = Array.from({ length: MAX_LINES_PER_GROUP + 1 }, (_, i) => sel(i, '1', '0'));
		expect(letterBlocker(lines)).toEqual({ kind: 'tooMany' });
		const exactly = Array.from({ length: MAX_LINES_PER_GROUP }, (_, i) =>
			sel(i, i % 2 ? '1' : '0', i % 2 ? '0' : '1'),
		);
		expect(letterBlocker(exactly)).toBeNull();
	});

	it('écart non nul → « ne s’équilibre pas », avec l’écart', () => {
		const b = letterBlocker([sel(1, '100', '0'), sel(2, '0', '60')]);
		expect(b?.kind).toBe('unbalanced');
		expect(b && b.kind === 'unbalanced' && b.difference.toFixed(2)).toBe('40.00');
	});

	it('toutes en période close → « période close » ; une seule ouverte suffit', () => {
		expect(letterBlocker([sel(1, '5', '0', false), sel(2, '0', '5', false)])).toEqual({
			kind: 'allClosed',
		});
		expect(letterBlocker([sel(1, '5', '0', false), sel(2, '0', '5', true)])).toBeNull();
	});
});

describe('C-15-1c-17 — tout 404 ou 409 est un refus périmé (test 7)', () => {
	it('404 et 409 (tout code, CONCURRENT_CHANGE compris) oui ; 400 et une Error non', () => {
		expect(isStaleRefusal(refusal(404, 'NOT_FOUND'))).toBe(true);
		expect(isStaleRefusal(refusal(409, 'LETTERING_CONCURRENT_CHANGE'))).toBe(true);
		expect(isStaleRefusal(refusal(409, 'UN_CODE_NEUF'))).toBe(true);
		expect(isStaleRefusal(refusal(400, 'LETTERING_TOO_FEW_LINES'))).toBe(false);
		expect(isStaleRefusal(new Error('réseau'))).toBe(false);
	});
});

describe('AC8 — le sens, lu sur le seul signe (test 10)', () => {
	it('négatif → créditeur, positif → débiteur, zéro → aucun', () => {
		expect(sideOf('-500.0000')).toMatchObject({ side: 'credit' });
		expect(sideOf('-500.0000').abs.toFixed(2)).toBe('500.00');
		expect(sideOf('20.0000')).toMatchObject({ side: 'debit' });
		expect(sideOf('0.0000').side).toBeNull();
	});

	it('sameAmount compare des NOMBRES, non des chaînes', () => {
		expect(sameAmount('-500.0000', '-500')).toBe(true);
		expect(sameAmount('500.0000', '-500.0000')).toBe(false);
	});
});
