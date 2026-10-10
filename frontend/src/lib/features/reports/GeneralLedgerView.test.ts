// Story 24-1 — tests Vitest pour GeneralLedgerView.svelte.
//
// Ce que ces tests tiennent, et qui n'est pas cosmétique : l'encadrement
// ouverture/mouvements/clôture, la rupture d'exercice là où le solde repart de
// zéro, et le bandeau de troncature — sans lequel un livre coupé à 500 lignes
// se lit comme un livre complet.

import { describe, it, expect, vi } from 'vitest';
import { render } from '@testing-library/svelte';

vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string) => fallback,
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
