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
});
