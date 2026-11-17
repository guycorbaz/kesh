// Story 37-2 (AC-6) — règles pures de l'écran Cours de change : mentions et ↑,
// décision à la validation d'un champ, colonnes et liaisons, colonnes à récupérer,
// exercice par défaut et exercice suivant.

import { describe, expect, it } from 'vitest';
import {
	columnsToFetch,
	columnsToRecheck,
	commitDecision,
	defaultFiscalYear,
	distinctColumns,
	fetchability,
	fetchRequestOf,
	formatRate,
	gridColumns,
	isLinkedToMonth,
	isValidRateInput,
	mentionOf,
	monthLabel,
	nextFiscalYearOf,
} from './exchange-rates-grid';
import { afcRow, cell, fy, published, row, yearGrid } from './exchange-rates.test-fixtures';
import type { Cell } from './exchange-rates.types';

type Cells = Record<string, Partial<Cell>>;

const M = '2024-12-01';

describe('mentionOf — une valeur, sa source et le ↑ (U1, A2, A30)', () => {
	it('ligne afc égale → « AFC ✓ », sans ↑', () => {
		const m = mentionOf(
			cell('month', M, { effectiveRate: '0.8846000000', source: 'afc', row: afcRow(), published: published('0.8846') }),
		);
		expect(m).toMatchObject({ kind: 'afc-ok', arrow: null });
	});

	it('ligne afc corrigée → « AFC corrigé » et ↑ de la valeur corrigée', () => {
		const m = mentionOf(
			cell('month', M, {
				effectiveRate: '0.8846',
				source: 'afc',
				row: afcRow(),
				published: published('0.8850'),
				corrected: true,
			}),
		);
		expect(m).toMatchObject({
			kind: 'afc-corrected',
			arrow: 'corrected',
			published: '0.8850',
			correctedAt: '2024-12-03T09:00:00Z',
		});
	});

	it('ligne afc sans valeur publiée → « valeur publiée non relue », sans ↑', () => {
		const m = mentionOf(cell('month', M, { effectiveRate: '0.8846', source: 'afc', row: afcRow() }));
		expect(m).toMatchObject({ kind: 'afc-unverified', arrow: null });
	});

	it('ligne manual différente → « AFC : x » et ↑ de la valeur publiée', () => {
		const m = mentionOf(
			cell('month', M, { effectiveRate: '0.9', source: 'manual', row: afcRow(), published: published('0.8846') }),
		);
		expect(m).toMatchObject({ kind: 'saisi-differs', arrow: 'published', otherSource: null });
	});

	it('ligne manual égale → « AFC ✓ · saisi » avec ↑ (A2)', () => {
		const m = mentionOf(
			cell('month', M, { effectiveRate: '0.88460', source: 'manual', row: afcRow(), published: published('0.8846') }),
		);
		expect(m).toMatchObject({ kind: 'saisi-equal', arrow: 'published' });
	});

	it('ligne manual sans valeur publiée → « non publié » et ↑ « Retirer »', () => {
		const m = mentionOf(cell('month', M, { effectiveRate: '0.9', source: 'manual', row: afcRow() }));
		expect(m).toMatchObject({ kind: 'saisi-unpublished', arrow: 'remove' });
	});

	it('ligne d’une autre source (ecb) → comme manual, source entre parenthèses (A30)', () => {
		const m = mentionOf(
			cell('month', M, { effectiveRate: '0.9', source: 'ecb', row: afcRow(), published: published('0.8846') }),
		);
		expect(m).toMatchObject({ kind: 'saisi-differs', arrow: 'published', otherSource: 'ecb' });
	});

	it('aucune ligne, valeur publiée → ↑ ; ni ligne ni valeur → sans ↑', () => {
		expect(mentionOf(cell('month', M, { published: published('0.8846') }))).toMatchObject({
			kind: 'none-published',
			arrow: 'published',
		});
		expect(mentionOf(cell('month', M))).toMatchObject({ kind: 'none', arrow: null });
	});
});

describe('commitDecision — validation d’un champ', () => {
	const manual = cell('month', M, {
		effectiveRate: '0.9000000000',
		source: 'manual',
		row: afcRow(3),
		published: published('0.8846'),
	});

	it('champ vidé → restauré, sans appel', () => {
		expect(commitDecision(manual, '  ')).toEqual({ action: 'restore' });
	});

	it('valeur égale au cours affiché → rien', () => {
		expect(commitDecision(manual, '0.90')).toEqual({ action: 'none' });
	});

	it('valeur égale à la valeur publiée → reset, jamais put', () => {
		expect(commitDecision(manual, '0,8846')).toEqual({ action: 'reset' });
		expect(commitDecision(cell('month', M, { published: published('0.8846') }), '0.8846')).toEqual({
			action: 'reset',
		});
	});

	it('déjà afc à cette valeur → rien', () => {
		const afc = cell('month', M, { effectiveRate: '0.8846', source: 'afc', row: afcRow(), published: published('0.8846') });
		expect(commitDecision(afc, '0.8846')).toEqual({ action: 'none' });
	});

	it('autre valeur → put, virgule normalisée', () => {
		expect(commitDecision(manual, '0,8912')).toEqual({ action: 'put', rate: '0.8912' });
	});

	it('saisie hors bornes ou illisible → invalid', () => {
		for (const s of ['abc', '0', '-1', '0.12345678901', '12345678', '1000000.5']) {
			expect(commitDecision(manual, s), s).toEqual({ action: 'invalid' });
		}
	});

	it('cours JPY par unité : 0.0056 admis, envoyé tel quel', () => {
		expect(isValidRateInput('0.0056')).toBe(true);
		expect(commitDecision(cell('month', M), '0.0056')).toEqual({ action: 'put', rate: '0.0056' });
	});
});

describe('formatRate', () => {
	it('zéros de fin retirés, quatre décimales au moins', () => {
		expect(formatRate('0.8850000000')).toBe('0.8850');
		expect(formatRate('0.0056780000')).toBe('0.005678');
		expect(formatRate('1')).toBe('1.0000');
		expect(formatRate(null)).toBe('');
	});
});

describe('colonnes et liaisons (S4, T4)', () => {
	const cas: Array<{
		nom: string;
		start: string;
		end: string;
		n: number;
		first: string;
		last: string;
		openingDay: string;
		openingMonth: string;
		openingLinked: boolean;
	}> = [
		{ nom: '1er janvier', start: '2024-01-01', end: '2024-12-31', n: 12, first: 'janv. 2024', last: 'déc. 2024', openingDay: '2023-12-31', openingMonth: '2023-12-01', openingLinked: false },
		{ nom: '1er juillet', start: '2024-07-01', end: '2025-06-30', n: 12, first: 'juil. 2024', last: 'juin 2025', openingDay: '2024-06-30', openingMonth: '2024-06-01', openingLinked: false },
		{ nom: '1er avril', start: '2023-04-01', end: '2024-03-31', n: 12, first: 'avr. 2023', last: 'mars 2024', openingDay: '2023-03-31', openingMonth: '2023-03-01', openingLinked: false },
		{ nom: '15.03', start: '2024-03-15', end: '2025-03-14', n: 13, first: 'mars 2024', last: 'mars 2025', openingDay: '2024-03-14', openingMonth: '2024-03-01', openingLinked: true },
		{ nom: 'six mois', start: '2024-07-01', end: '2024-12-31', n: 6, first: 'juil. 2024', last: 'déc. 2024', openingDay: '2024-06-30', openingMonth: '2024-06-01', openingLinked: false },
		{ nom: 'dix-huit mois', start: '2023-07-01', end: '2024-12-31', n: 18, first: 'juil. 2023', last: 'déc. 2024', openingDay: '2023-06-30', openingMonth: '2023-06-01', openingLinked: false },
	];

	for (const c of cas) {
		it(`exercice ${c.nom} : ${c.n} colonnes, Ouverture et Clôture`, () => {
			const day = yearGrid(c.start, c.end, 'day', 'day');
			const { months, opening, closing } = gridColumns(day);
			expect(months).toHaveLength(c.n);
			expect(monthLabel(months[0].rateDate, 'fr-CH')).toBe(c.first);
			expect(monthLabel(months[c.n - 1].rateDate, 'fr-CH')).toBe(c.last);
			expect(opening).toMatchObject({ kind: 'day', rateDate: c.openingDay });
			expect(closing).toMatchObject({ kind: 'day', rateDate: c.end });

			const month = yearGrid(c.start, c.end, 'month', 'month');
			const cols = gridColumns(month);
			expect(cols.opening.rateDate).toBe(c.openingMonth);
			expect(isLinkedToMonth('month', cols.opening.rateDate, month.months)).toBe(c.openingLinked);
			// La Clôture en genre month est toujours la dernière colonne.
			expect(cols.closing.rateDate).toBe(month.months[c.n - 1]);
			expect(isLinkedToMonth('month', cols.closing.rateDate, month.months)).toBe(true);
			// Colonnes distinctes : les cases liées ne comptent pas deux fois.
			expect(distinctColumns(month)).toHaveLength(c.n + (c.openingLinked ? 0 : 1));
		});
	}
});

describe('fetchability — bornes de l’AC-5 (aujourd’hui 10.03.2025)', () => {
	const today = '2025-03-10';
	it('mois : au plus le mois suivant ; jamais avant 2000', () => {
		expect(fetchability('month', '2025-04-01', today)).toEqual({ ok: true });
		expect(fetchability('month', '2025-05-01', today)).toEqual({ ok: false, reason: 'future' });
		expect(fetchability('month', '1999-12-01', today)).toEqual({ ok: false, reason: 'before-2000' });
	});
	it('jour : strictement passé ; jamais avant 2000-01-01', () => {
		expect(fetchability('day', '2025-03-09', today)).toEqual({ ok: true });
		expect(fetchability('day', '2025-03-10', today)).toEqual({ ok: false, reason: 'future' });
		expect(fetchability('day', '1999-12-31', today)).toEqual({ ok: false, reason: 'before-2000' });
	});
	it('corps du POST : {year, month} ou {date}', () => {
		expect(fetchRequestOf({ kind: 'month', rateDate: '2024-07-01' })).toEqual({ year: 2024, month: 7 });
		expect(fetchRequestOf({ kind: 'day', rateDate: '2024-06-30' })).toEqual({ date: '2024-06-30' });
	});
});

describe('columnsToFetch / columnsToRecheck', () => {
	const today = '2025-03-10';

	it('deux devises sans valeur publiée le même mois → une seule colonne', () => {
		const g = yearGrid('2024-07-01', '2025-06-30', 'day', 'day', (g) => {
			const full = (cur: string) => {
				const cells: Cells = {
					opening: { published: published('0.9'), effectiveRate: '0.9', source: 'afc' },
				};
				for (const m of g.months) cells[`month:${m}`] = { published: published('0.9'), effectiveRate: '0.9', source: 'afc' };
				cells['month:2024-09-01'] = {};
				return row(g, cur, cells);
			};
			return [full('USD'), full('EUR')];
		});
		const cols = columnsToFetch(g, g.rows, today);
		expect(cols.map((c) => c.key)).toEqual(['month:2024-09-01']);
	});

	it('devise portée par un compte sans cours retenu → récupérable ; sans compte → non', () => {
		const g = yearGrid('2024-07-01', '2024-12-31', 'day', 'day', (g) => {
			const cells: Cells = { opening: { published: published('1') }, closing: { published: published('1') } };
			for (const m of g.months) cells[`month:${m}`] = { published: published('0.9') };
			return [row(g, 'USD', cells, true), row(g, 'GBP', cells, false)];
		});
		expect(columnsToFetch(g, g.rows, today)).toHaveLength(8);
		const sansCompte = { ...g, rows: g.rows.filter((r) => r.currency === 'GBP') };
		expect(columnsToFetch(sansCompte, sansCompte.rows, today)).toHaveLength(0);
	});

	it('Ouverture liée à la première colonne : une seule fois ; colonnes futures écartées', () => {
		const g = yearGrid('2024-03-15', '2025-03-14', 'month', 'day', (g) => [row(g, 'USD')]);
		const keys = columnsToFetch(g, g.rows, today).map((c) => c.key);
		expect(keys.filter((k) => k === 'month:2024-03-01')).toHaveLength(1);
		// Clôture au 14.03.2025 : jour non passé le 10.03.2025.
		expect(keys).not.toContain('day:2025-03-14');
		expect(keys).toHaveLength(13);
	});

	it('Revérifier : les colonnes récupérables déjà présentes seulement', () => {
		const g = yearGrid('2024-07-01', '2024-12-31', 'day', 'day', (g) => [
			row(g, 'USD', { 'month:2024-08-01': { published: published('0.9') }, opening: { published: published('0.9') } }),
		]);
		expect(columnsToRecheck(g, g.rows, today).map((c) => c.key)).toEqual(['month:2024-08-01', 'day:2024-06-30']);
	});
});

describe('exercice par défaut et exercice suivant', () => {
	const a = fy({ id: 1, name: '2023', startDate: '2023-01-01', endDate: '2023-12-31', status: 'Closed' });
	const b = fy({ id: 2, name: '2024', startDate: '2024-01-01', endDate: '2024-12-31' });
	const c = fy({ id: 3, name: '2025', startDate: '2025-01-01', endDate: '2025-12-31', multiCurrency: false });

	it('celui qui couvre aujourd’hui, sinon le plus récent multi-devises', () => {
		expect(defaultFiscalYear([a, b, c], '2024-05-01')?.id).toBe(2);
		expect(defaultFiscalYear([a, b, c], '2026-05-01')?.id).toBe(2);
		expect(defaultFiscalYear([a, b], '2024-05-01', '2023-06-01')?.id).toBe(1);
		expect(defaultFiscalYear([c], '2025-05-01')).toBeNull();
	});

	it('suivant = commence le lendemain de la clôture ; un trou rompt le lien', () => {
		expect(nextFiscalYearOf(a, [a, b, c])?.id).toBe(2);
		const trou = fy({ id: 4, startDate: '2026-02-01', endDate: '2026-12-31' });
		expect(nextFiscalYearOf(c, [a, b, c, trou])).toBeNull();
	});
});
