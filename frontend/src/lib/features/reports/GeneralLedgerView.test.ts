// Story 24-1 — tests Vitest pour GeneralLedgerView.svelte.
//
// Ce que ces tests tiennent, et qui n'est pas cosmétique : l'encadrement
// ouverture/mouvements/clôture, la rupture d'exercice là où le solde repart de
// zéro, et le bandeau de troncature — sans lequel un livre coupé à 500 lignes
// se lit comme un livre complet.

import { describe, it, expect, vi } from 'vitest';
import { render } from '@testing-library/svelte';

vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	// Le repli, paramètres substitués (Story 37-4 : les mentions en devise en portent).
	i18nMsg: (_key: string, fallback: string, args?: Record<string, string | number>) =>
		fallback.replace(/\{ \$(\w+) \}/g, (_: string, k: string) => String(args?.[k] ?? '')),
}));

import GeneralLedgerView from './GeneralLedgerView.svelte';
import type { GeneralLedgerDto, LedgerLine, LedgerSection } from './reports.types';

function line(over: Partial<LedgerLine> = {}): LedgerLine {
	return {
		lineId: 1,
		entryId: 1,
		entryDate: '2026-03-04',
		fiscalYearId: 1,
		fiscalYearName: 'Exercice 2026',
		entryNumber: 12,
		journal: 'Banque',
		description: 'Loyer mars',
		counterpart: ['6000'],
		debit: '1200.00',
		credit: '0.00',
		runningBalance: '1200.00',
		letteringCode: null,
		...over,
	};
}

function section(over: Partial<LedgerSection> = {}): LedgerSection {
	return {
		accountId: 7,
		accountNumber: '1020',
		accountName: 'Banque',
		accountType: 'Asset',
		active: true,
		balanceSide: 'debit',
		opening: '500.00',
		lines: [line()],
		totalDebit: '1200.00',
		totalCredit: '0.00',
		closing: '1700.00',
		unnaturalBalance: false,
		fiscalYearBreaks: [],
		lineCount: 1,
		...over,
	};
}

function dto(over: Partial<GeneralLedgerDto> = {}): GeneralLedgerDto {
	return {
		period: { from: '2026-01-01', to: '2026-12-31' },
		sections: [section()],
		...over,
	};
}

describe('GeneralLedgerView', () => {
	it('encadre les mouvements par le solde d’ouverture et le solde de clôture', () => {
		const { getByTestId } = render(GeneralLedgerView, { dto: dto() });

		expect(getByTestId('ledger-opening').textContent).toContain('500.00');
		expect(getByTestId('ledger-closing').textContent).toContain('1’700.00');
		expect(getByTestId('ledger-section-1020').textContent).toContain('Loyer mars');
	});

	it('laisse vide la colonne du montant nul plutôt que d’écrire 0.00', () => {
		const { getByTestId } = render(GeneralLedgerView, { dto: dto() });
		// Le crédit vaut 0 sur l'unique ligne : il ne doit apparaître nulle part
		// dans le corps du tableau (le solde, lui, vaut 1'700.00).
		const cells = getByTestId('ledger-section-1020').querySelectorAll('tbody tr td');
		const texts = Array.from(cells).map((c) => c.textContent?.trim());
		expect(texts).toContain('1’200.00'); // le débit s'écrit
		expect(texts.filter((t) => t === '0.00')).toHaveLength(0);
	});

	it('signale un solde contre nature', () => {
		const { getByTestId } = render(GeneralLedgerView, {
			dto: dto({ sections: [section({ unnaturalBalance: true, closing: '-40.00' })] }),
		});
		expect(getByTestId('ledger-unnatural').textContent).toContain('Solde contre nature');
	});

	it('intercale la rupture d’exercice entre deux lignes d’exercices différents', () => {
		const s = section({
			lines: [
				line({ lineId: 1, fiscalYearId: 1, entryDate: '2025-11-02' }),
				line({ lineId: 2, fiscalYearId: 2, entryDate: '2026-02-03', description: 'Loyer février' }),
			],
			fiscalYearBreaks: [
				{ date: '2025-12-31', closingFiscalYearId: 1, closingBalance: '1700.00' },
			],
			lineCount: 2,
		});
		const { getByTestId } = render(GeneralLedgerView, { dto: dto({ sections: [s] }) });
		const html = getByTestId('ledger-section-1020').textContent ?? '';
		expect(html).toContain('le solde repart de zéro');
		// La rupture se place APRÈS la ligne de l'exercice qui se clôt.
		expect(html.indexOf('Loyer mars')).toBeLessThan(html.indexOf('le solde repart de zéro'));
		expect(html.indexOf('le solde repart de zéro')).toBeLessThan(html.indexOf('Loyer février'));
	});

	it('#777 — pose la rupture qui clôt l’ouverture AVANT la première ligne d’un exercice postérieur', () => {
		const s = section({
			accountType: 'Revenue',
			balanceSide: 'credit',
			opening: '400.00',
			lines: [
				line({ lineId: 1, fiscalYearId: 2, entryDate: '2026-02-01', runningBalance: '100.00' }),
			],
			fiscalYearBreaks: [{ date: '2026-02-01', closingFiscalYearId: 1, closingBalance: '400.00' }],
			closing: '100.00',
		});
		const { getByTestId, getAllByTestId } = render(GeneralLedgerView, {
			dto: dto({ sections: [s] }),
		});
		expect(getAllByTestId('ledger-fy-break')).toHaveLength(1);
		const rows = Array.from(getByTestId('ledger-section-1020').querySelectorAll('tbody tr'));
		// ouverture, rupture, ligne
		expect(rows[1].getAttribute('data-testid')).toBe('ledger-fy-break');
		expect(rows[1].textContent).toContain('400.00');
		expect(rows[2].textContent).toContain('Loyer mars');
	});

	it('#778 — rend sous les lignes la fin d’exercice franchie après la dernière', () => {
		const s = section({
			accountType: 'Revenue',
			balanceSide: 'credit',
			lines: [line({ lineId: 1, fiscalYearId: 1, entryDate: '2025-09-01', runningBalance: '700.00' })],
			fiscalYearBreaks: [{ date: '2026-01-01', closingFiscalYearId: 1, closingBalance: '700.00' }],
			closing: '0.00',
		});
		const { getByTestId, getAllByTestId } = render(GeneralLedgerView, {
			dto: dto({ sections: [s] }),
		});
		expect(getAllByTestId('ledger-fy-break')).toHaveLength(1);
		const rows = Array.from(getByTestId('ledger-section-1020').querySelectorAll('tbody tr'));
		// ouverture, ligne, rupture finale
		expect(rows).toHaveLength(3);
		expect(rows[2].getAttribute('data-testid')).toBe('ledger-fy-break');
		expect(rows[2].textContent).toContain('01.01.2026');
		expect(getByTestId('ledger-closing').textContent).toContain('0.00');

		// Sans aucune ligne, la rupture se rend quand même.
		const vide = render(GeneralLedgerView, {
			dto: dto({ sections: [{ ...s, accountNumber: '3000', lines: [], lineCount: 0 }] }),
		});
		expect(vide.container.querySelectorAll('[data-testid="ledger-fy-break"]')).toHaveLength(1);
		// P1 de la revue : l'ouverture ne « reste » pas « due » quand une rupture
		// finale la clôt — le message dit la clôture, puis la rupture la montre.
		const msg = vide.container.querySelector('[data-testid="ledger-no-movement"]')?.textContent ?? '';
		expect(msg).toContain('Aucun mouvement');
		expect(msg).toContain('le solde repart de zéro');
		expect(msg).not.toContain('reste dû');
	});

	it('sans rupture finale, un compte sans mouvement dit que l’ouverture reste due', () => {
		const { getByTestId } = render(GeneralLedgerView, {
			dto: dto({ sections: [section({ lines: [], lineCount: 0, closing: '500.00' })] }),
		});
		expect(getByTestId('ledger-no-movement').textContent).toContain('reste dû');
	});

	it('préfixe la pièce de son exercice dès que la période en traverse deux', () => {
		const mono = render(GeneralLedgerView, { dto: dto() });
		// Un seul exercice : le numéro nu suffit, et le préfixe serait du bruit.
		expect(mono.container.textContent).not.toContain('Exercice 2026/12');
		mono.unmount();

		const s = section({
			lines: [
				line({ lineId: 1, fiscalYearId: 1, fiscalYearName: 'Exercice 2025' }),
				line({ lineId: 2, fiscalYearId: 2, fiscalYearName: 'Exercice 2026' }),
			],
			fiscalYearBreaks: [
				{ date: '2025-12-31', closingFiscalYearId: 1, closingBalance: '1700.00' },
			],
			lineCount: 2,
		});
		const multi = render(GeneralLedgerView, { dto: dto({ sections: [s] }) });
		const txt = multi.container.textContent ?? '';
		expect(txt).toContain('Exercice 2025/12');
		expect(txt).toContain('Exercice 2026/12');
	});

	it('avertit quand la page rendue ne contient pas toutes les lignes', () => {
		const { container } = render(GeneralLedgerView, {
			dto: dto({ sections: [section({ lineCount: 812 })] }),
		});
		expect(container.textContent).toContain('812');
	});

	it('rend un compte sans mouvement en disant que l’ouverture reste due', () => {
		const { getByTestId } = render(GeneralLedgerView, {
			dto: dto({ sections: [section({ lines: [], lineCount: 0, closing: '500.00' })] }),
		});
		expect(getByTestId('ledger-section-1020').textContent).toContain('Aucun mouvement');
	});

	it('affiche l’empty-state quand aucun compte n’est retenu', () => {
		const { container } = render(GeneralLedgerView, { dto: dto({ sections: [] }) });
		expect(container.textContent).toContain('Aucun compte à afficher');
	});

	// ---------------------------------------------------------------------
	// Story 15-1c-ii (AC9, test 3) — la colonne « Lettrage », les `colspan`
	// calculés, le lien « Postes ouverts de ce compte ».
	// ---------------------------------------------------------------------

	/** La somme des `colspan` d'une rangée (1 par cellule sans attribut). */
	function span(tr: Element): number {
		return Array.from(tr.children).reduce(
			(acc, td) => acc + Number(td.getAttribute('colspan') ?? '1'),
			0,
		);
	}

	/** Une section qui porte TOUTES les formes de rangée : rupture, lettrée, ouverte. */
	function fullSection() {
		return section({
			lines: [
				line({ lineId: 1, fiscalYearId: 1, letteringCode: 'AB' }),
				line({ lineId: 2, fiscalYearId: 2, letteringCode: null }),
			],
			fiscalYearBreaks: [
				{ date: '2025-12-31', closingFiscalYearId: 1, closingBalance: '1700.00' },
			],
			lineCount: 2,
		});
	}

	it('15-1c-ii test 3 — « Lettrage » est la DERNIÈRE colonne, après « Solde progressif »', () => {
		const { getByTestId } = render(GeneralLedgerView, { dto: dto() });
		const ths = Array.from(getByTestId('ledger-section-1020').querySelectorAll('thead th')).map(
			(th) => th.textContent?.trim(),
		);
		expect(ths).toHaveLength(9);
		expect(ths.at(-1)).toBe('Lettrage');
		expect(ths.at(-2)).toBe('Solde progressif');
	});

	it('15-1c-ii test 3 — chaque rangée du corps et du pied couvre exactement les en-têtes', () => {
		for (const s of [fullSection(), section({ lines: [], lineCount: 0 })]) {
			const { getByTestId, unmount } = render(GeneralLedgerView, { dto: dto({ sections: [s] }) });
			const table = getByTestId('ledger-section-1020').querySelector('table')!;
			const headers = table.querySelectorAll('thead th').length;
			const rows = table.querySelectorAll('tbody tr, tfoot tr');
			// ouverture, (rupture, deux mouvements | ligne vide), total, clôture
			expect(rows.length).toBe(s.lines.length === 0 ? 4 : 6);
			for (const tr of rows) expect(span(tr)).toBe(headers);
			unmount();
		}
	});

	/** Le texte de chaque colonne d'une rangée, `colspan` déplié. */
	function byColumn(tr: Element): string[] {
		const out: string[] = [];
		for (const td of Array.from(tr.children)) {
			const n = Number(td.getAttribute('colspan') ?? '1');
			for (let i = 0; i < n; i++) out.push(td.textContent?.trim() ?? '');
		}
		return out;
	}

	it('15-1c-ii test 3 — ouverture, rupture et clôture : le solde tombe sous « Solde progressif »', () => {
		const { getByTestId } = render(GeneralLedgerView, { dto: dto({ sections: [fullSection()] }) });
		const table = getByTestId('ledger-section-1020').querySelector('table')!;
		const ths = Array.from(table.querySelectorAll('thead th')).map((th) => th.textContent?.trim());
		const running = ths.indexOf('Solde progressif');
		const opening = getByTestId('ledger-opening').parentElement!;
		const closing = getByTestId('ledger-closing').parentElement!;
		const brk = table.querySelectorAll('tbody tr')[2]; // ouverture, ligne 1, rupture
		expect(brk.textContent).toContain('le solde repart de zéro');
		for (const tr of [opening, brk, closing]) {
			const cols = byColumn(tr);
			// La cellule du solde est seule de son contenu à cet index : le
			// libellé s'étend jusqu'à la colonne qui précède.
			expect(cols[running]).toMatch(/^-?[\d’.]+$/);
			expect(cols[running - 1]).not.toMatch(/^-?[\d’.]+$/);
			expect(cols.at(-1)).toBe('');
		}
	});

	it('15-1c-ii test 3 — le total des mouvements tombe sous « Débit » et « Crédit »', () => {
		const { getByTestId } = render(GeneralLedgerView, {
			dto: dto({ sections: [section({ totalDebit: '1234.00', totalCredit: '567.00' })] }),
		});
		const table = getByTestId('ledger-section-1020').querySelector('table')!;
		const ths = Array.from(table.querySelectorAll('thead th')).map((th) => th.textContent?.trim());
		const cols = byColumn(table.querySelectorAll('tfoot tr')[0]);
		expect(cols[ths.indexOf('Débit')]).toBe('1’234.00');
		expect(cols[ths.indexOf('Crédit')]).toBe('567.00');
	});

	it('15-1c-ii test 3 — le code d’une ligne lettrée est un lien vers son groupe ; une ligne ouverte, rien', () => {
		const { getByTestId } = render(GeneralLedgerView, { dto: dto({ sections: [fullSection()] }) });
		const lettered = getByTestId('ledger-lettering-1');
		const a = lettered.querySelector('a');
		expect(a?.textContent).toBe('AB');
		expect(a?.getAttribute('href')).toBe('/open-items?group=AB');
		expect(getByTestId('ledger-lettering-2').textContent?.trim()).toBe('');
		expect(getByTestId('ledger-lettering-2').querySelector('a')).toBeNull();
		// La cellule est bien dans la dernière colonne de sa rangée.
		expect(lettered.parentElement?.lastElementChild).toBe(lettered);
	});

	it('15-1c-ii test 3 — « Postes ouverts de ce compte » ssi le compte est lettrable, à la fin de la période', () => {
		const two = dto({
			period: { from: '2026-01-01', to: '2026-06-30' },
			sections: [
				section({ accountId: 7, accountNumber: '1100' }),
				section({ accountId: 8, accountNumber: '6000' }),
			],
		});
		const { getByTestId, queryByTestId } = render(GeneralLedgerView, {
			dto: two,
			letterableAccountIds: new Set([7]),
		});
		expect(getByTestId('ledger-open-items-1100').getAttribute('href')).toBe(
			'/open-items?accountId=7&asOf=2026-06-30',
		);
		expect(getByTestId('ledger-open-items-1100').textContent).toBe('Postes ouverts de ce compte');
		expect(queryByTestId('ledger-open-items-6000')).toBeNull();
	});

	it('15-1c-ii test 3 — aucun lien si la liste des comptes est vide (non chargée)', () => {
		const { queryByTestId } = render(GeneralLedgerView, {
			dto: dto(),
			letterableAccountIds: new Set<number>(),
		});
		expect(queryByTestId('ledger-open-items-1020')).toBeNull();
		// … ni sans la prop du tout.
		const bare = render(GeneralLedgerView, { dto: dto() });
		expect(bare.queryByTestId('ledger-open-items-1020')).toBeNull();
	});
});

describe('GeneralLedgerView — Story 37-4, compte en devise', () => {
	function usdSection(): LedgerSection {
		return section({
			currency: 'USD',
			opening: '1370.00',
			openingCurrency: '1500.0000',
			lines: [
				line({
					lineId: 11,
					debit: '0.00',
					credit: '190.00',
					runningBalance: '1180.00',
					debitCurrency: '0.0000',
					creditCurrency: '200.0000',
					exchangeRate: '0.9500000000',
					rateSource: 'manual',
					runningBalanceCurrency: '1300.0000',
				}),
				line({
					lineId: 12,
					debit: '15.00',
					credit: '0.00',
					runningBalance: '1195.00',
					debitCurrency: '0.0000',
					creditCurrency: '0.0000',
					exchangeRate: '0.9312000000',
					rateSource: 'revaluation',
					runningBalanceCurrency: '1300.0000',
				}),
			],
			lineCount: 2,
			totalDebitCurrency: '0.0000',
			totalCreditCurrency: '200.0000',
			closing: '1195.00',
			closingCurrency: '1300.0000',
		});
	}

	it('montre la devise, la seconde ligne de chaque mouvement et les soldes en devise', () => {
		const r = render(GeneralLedgerView, { dto: dto({ sections: [usdSection()] }) });
		expect(r.getByTestId('ledger-currency-1020').textContent).toBe('USD');
		expect(r.getByTestId('ledger-fx-11').textContent?.trim()).toBe(
			'USD 200.00 × 0.9500 · solde USD 1’300.00',
		);
		// La réévaluation : le cours, sans montant en devise (AC-2).
		expect(r.getByTestId('ledger-fx-12').textContent?.trim()).toBe(
			'Réévaluation au cours 0.9312 · solde USD 1’300.00',
		);
		expect(r.getByTestId('ledger-opening-fx').textContent).toBe('USD 1’500.00');
		expect(r.getByTestId('ledger-closing-fx').textContent).toBe('USD 1’300.00');
		expect(r.getByTestId('ledger-totals-fx').textContent).toBe(
			'débit USD 0.00 · crédit USD 200.00',
		);
	});

	it('porte le solde en devise d’une rupture d’exercice', () => {
		const s = usdSection();
		s.accountType = 'Revenue';
		s.lines = [s.lines[0]];
		s.lines[0].fiscalYearId = 2;
		s.fiscalYearBreaks = [
			{
				date: s.lines[0].entryDate,
				closingFiscalYearId: 1,
				closingBalance: '470.00',
				closingBalanceCurrency: '500.0000',
			},
		];
		const r = render(GeneralLedgerView, { dto: dto({ sections: [s] }) });
		expect(r.getByTestId('ledger-fy-break-fx').textContent).toBe('USD 500.00');
	});

	it('mono-devise : rien de visible ne change (aucune mention en devise)', () => {
		const r = render(GeneralLedgerView, { dto: dto() });
		const html = r.container.innerHTML;
		expect(html).not.toContain('×');
		expect(html).not.toMatch(/data-testid="ledger-(fx|currency|opening-fx|closing-fx|totals-fx)/);
		expect(r.container.querySelectorAll('tbody tr')).toHaveLength(2); // ouverture + 1 ligne
	});
});

// Story 55-2 (AC-4, AC-5, D8) — les contreparties deviennent des liens (numéro → compte par la carte de la
// page), même période ; le titre d'une section n'est un lien que si le rapport en rend plusieurs.
describe('GeneralLedgerView — Story 55-2, contreparties et titre', () => {
	const carte = new Map([
		['6000', 60],
		['1020', 7],
		['2000', 20],
	]);

	it('une contrepartie résolue est un lien vers son grand livre, même période', async () => {
		const { LINK_CLASS } = await import('$lib/shared/navigation/link-class');
		const { getByTestId } = render(GeneralLedgerView, { dto: dto(), accountIdByNumber: carte });
		const cellule = getByTestId('ledger-counterpart-1');
		const a = cellule.querySelector('a');
		// Mutation : `counterpart.join(', ')` gardé → aucun lien, rouge.
		expect(a?.getAttribute('href')).toBe('/reports?tab=general-ledger&accountId=60&from=2026-01-01&to=2026-12-31');
		expect(a?.getAttribute('class')).toBe(LINK_CLASS);
		expect(a?.textContent).toBe('6000');
	});

	it('plusieurs contreparties : séparées par « , », une non résolue reste en texte, sans erreur', () => {
		const { getByTestId } = render(GeneralLedgerView, {
			dto: dto({ sections: [section({ lines: [line({ counterpart: ['6000', '9999', '2000'] })] })] }),
			accountIdByNumber: carte,
		});
		const cellule = getByTestId('ledger-counterpart-1');
		expect(cellule.textContent).toBe('6000, 9999, 2000');
		const liens = [...cellule.querySelectorAll('a')].map((a) => a.textContent);
		expect(liens).toEqual(['6000', '2000']);
	});

	it('sans carte (liste des comptes en échec) : les contreparties en texte', () => {
		const { getByTestId } = render(GeneralLedgerView, { dto: dto() });
		expect(getByTestId('ledger-counterpart-1').querySelector('a')).toBeNull();
		expect(getByTestId('ledger-counterpart-1').textContent).toBe('6000');
	});

	it('une seule section : le numéro du titre reste du texte (un lien vers lui-même n’apprend rien)', () => {
		const { getByTestId } = render(GeneralLedgerView, { dto: dto(), accountIdByNumber: carte });
		// Mutation : titre toujours lié → rouge.
		expect(getByTestId('ledger-title-1020').querySelector('a')).toBeNull();
		expect(getByTestId('ledger-title-1020').textContent).toBe('1020');
	});

	it('plusieurs sections : le numéro de chaque titre est un lien vers son grand livre', () => {
		const { getByTestId } = render(GeneralLedgerView, {
			dto: dto({
				sections: [
					section(),
					section({ accountId: 60, accountNumber: '6000', accountName: 'Loyer', lines: [line({ lineId: 2, counterpart: ['1020'] })] }),
				],
			}),
			accountIdByNumber: carte,
		});
		// Mutation : titre jamais lié → rouge.
		expect(getByTestId('ledger-title-1020').querySelector('a')?.getAttribute('href')).toBe(
			'/reports?tab=general-ledger&accountId=7&from=2026-01-01&to=2026-12-31',
		);
		expect(getByTestId('ledger-title-6000').querySelector('a')?.getAttribute('href')).toBe(
			'/reports?tab=general-ledger&accountId=60&from=2026-01-01&to=2026-12-31',
		);
	});

	it('le lien des postes ouverts garde sa classe soulignée en plein (R-7, liens déjà soulignés inchangés)', () => {
		const { getByTestId } = render(GeneralLedgerView, {
			dto: dto(),
			letterableAccountIds: new Set([7]),
		});
		expect(getByTestId('ledger-open-items-1020').getAttribute('class')).toBe('ml-auto text-xs text-primary underline');
	});
});

describe('GeneralLedgerView — Story 55-3b (AC-6), la colonne « Justificatifs »', () => {
	/** La somme des `colspan` d'une rangée. */
	function span(tr: Element): number {
		return Array.from(tr.children).reduce((n, td) => n + Number(td.getAttribute('colspan') ?? '1'), 0);
	}
	/** Le texte de chaque colonne d'une rangée, `colspan` déplié. */
	function byColumn(tr: Element): string[] {
		const out: string[] = [];
		for (const td of Array.from(tr.children)) {
			const n = Number(td.getAttribute('colspan') ?? '1');
			for (let i = 0; i < n; i++) out.push(td.textContent?.trim() ?? '');
		}
		return out;
	}
	function headers(root: HTMLElement, accountNumber = '1020'): string[] {
		return Array.from(
			root.querySelector(`[data-testid="ledger-section-${accountNumber}"] thead`)!.querySelectorAll('th'),
		).map((th) => th.textContent?.trim() ?? '');
	}
	/** Une section à rupture, la première ligne avec justificatif. */
	function sectionAvecJustificatif(over: Partial<LedgerSection> = {}): LedgerSection {
		return section({
			opening: '500.00',
			lines: [
				line({ lineId: 1, fiscalYearId: 1, letteringCode: 'AB', hasDocuments: true }),
				line({ lineId: 2, fiscalYearId: 2, letteringCode: null }),
			],
			fiscalYearBreaks: [{ date: '2025-12-31', closingFiscalYearId: 1, closingBalance: '1700.00' }],
			lineCount: 2,
			...over,
		});
	}

	it('sans justificatif : aucune colonne, aucune icône (9 en-têtes)', () => {
		const { container, queryByTestId } = render(GeneralLedgerView, { dto: dto() });
		expect(headers(container)).toHaveLength(9);
		expect(headers(container)).not.toContain('Justificatifs');
		expect(queryByTestId('ledger-entry-documents-1')).toBeNull();
	});

	it('une ligne à justificatif : 10 en-têtes, « Justificatifs » après « Libellé », l’icône sur cette ligne seule', () => {
		const { container, getByTestId, queryByTestId } = render(GeneralLedgerView, {
			dto: dto({ sections: [sectionAvecJustificatif()] }),
		});
		const ths = headers(container);
		expect(ths).toHaveLength(10);
		expect(ths.indexOf('Justificatifs')).toBe(ths.indexOf('Libellé') + 1);
		expect(ths.at(-1)).toBe('Lettrage');
		const bouton = getByTestId('ledger-entry-documents-1');
		// Le numéro de piece() : préfixé de l'exercice, la période en traversant deux.
		expect(bouton.getAttribute('aria-label')).toBe("Justificatifs de l'écriture n° Exercice 2026/12");
		expect(queryByTestId('ledger-entry-documents-2')).toBeNull();
		// La cellule de l'icône tombe sous l'en-tête « Justificatifs ».
		const tr = bouton.closest('tr')!;
		expect(Array.from(tr.children).indexOf(bouton.closest('td')!)).toBe(ths.indexOf('Justificatifs'));
	});

	it('rejoué avec la colonne — chaque rangée du corps et du pied couvre exactement les en-têtes (devise comprise)', () => {
		const usd = section({
			accountId: 9,
			accountNumber: '1021',
			currency: 'USD',
			openingCurrency: '0.0000',
			lines: [
				line({
					lineId: 31,
					hasDocuments: true,
					debitCurrency: '0.0000',
					creditCurrency: '200.0000',
					exchangeRate: '0.9500000000',
					rateSource: 'manual',
					runningBalanceCurrency: '1300.0000',
				}),
			],
			totalDebitCurrency: '0.0000',
			totalCreditCurrency: '200.0000',
			closingCurrency: '1300.0000',
		});
		for (const s of [sectionAvecJustificatif(), usd]) {
			const { container, unmount } = render(GeneralLedgerView, { dto: dto({ sections: [s] }) });
			const table = container.querySelector(`[data-testid="ledger-section-${s.accountNumber}"] table`)!;
			const n = table.querySelectorAll('thead th').length;
			expect(n).toBe(10);
			const rows = table.querySelectorAll('tbody tr, tfoot tr');
			expect(rows.length).toBeGreaterThan(3);
			for (const tr of rows) expect(span(tr)).toBe(n);
			unmount();
		}
	});

	it('rejoué avec la colonne — le solde tombe sous « Solde progressif », le total sous « Débit » et « Crédit »', () => {
		const { container, getByTestId } = render(GeneralLedgerView, {
			dto: dto({ sections: [sectionAvecJustificatif({ totalDebit: '1234.00', totalCredit: '567.00' })] }),
		});
		const table = getByTestId('ledger-section-1020').querySelector('table')!;
		const ths = headers(container);
		const running = ths.indexOf('Solde progressif');
		for (const tr of [getByTestId('ledger-opening').parentElement!, getByTestId('ledger-closing').parentElement!]) {
			const cols = byColumn(tr);
			expect(cols[running]).toMatch(/^-?[\d’.]+$/);
			expect(cols[running - 1]).not.toMatch(/^-?[\d’.]+$/);
		}
		const total = byColumn(table.querySelectorAll('tfoot tr')[0]);
		expect(total[ths.indexOf('Débit')]).toBe('1’234.00');
		expect(total[ths.indexOf('Crédit')]).toBe('567.00');
	});

	it('deux sections, une seule avec justificatif : même nombre d’en-têtes (condition sur le rapport)', () => {
		const { container } = render(GeneralLedgerView, {
			dto: dto({
				sections: [sectionAvecJustificatif(), section({ accountId: 8, accountNumber: '6000' })],
			}),
		});
		expect(headers(container, '1020')).toHaveLength(10);
		expect(headers(container, '6000')).toHaveLength(10);
	});
});
