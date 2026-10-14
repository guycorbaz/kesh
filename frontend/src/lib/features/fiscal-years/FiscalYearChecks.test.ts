// Story 29-1a (#232, AC5, T-a12) — le panneau « Préparer la clôture ».
//
// i18nMsg rend son repli, variables substituées : les textes vérifiés sont ceux du
// repli français, qui égale le catalogue fr-CH.

import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/svelte';

vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string, args?: Record<string, string | number>) =>
		args
			? fallback.replace(/\{\s*\$(\w+)\s*\}/g, (_, n) => String(args[n] ?? ''))
			: fallback
}));

import FiscalYearChecks from './FiscalYearChecks.svelte';
import { closingChecks } from './closing-checks.test-fixtures';

const CODES = [
	'EARLIER_FISCAL_YEAR_OPEN',
	'PENDING_BANK_TRANSACTIONS',
	'BANK_STATEMENT_AT_YEAR_END',
	'DRAFT_INVOICES',
	'IMPORTED_SUPPLIER_INVOICES_TO_COMPLETE',
	'INCONSISTENT_ENTRIES',
	'NEXT_FISCAL_YEAR'
];

describe('FiscalYearChecks — les trois états', () => {
	it('pendant le chargement : l’état d’attente, aucune ligne, ni « Rien à signaler » (M-a22)', () => {
		render(FiscalYearChecks, { loadState: 'loading', checks: null });
		expect(screen.getByTestId('fiscal-year-checks-loading').textContent).toMatch(
			/Lecture des contrôles/
		);
		expect(screen.queryByTestId('fiscal-year-checks-list')).toBeNull();
		expect(screen.queryByText(/Rien à signaler/)).toBeNull();
	});

	it('un échec se dit « indisponible », jamais « rien à signaler » (M-a10)', () => {
		render(FiscalYearChecks, { loadState: 'error', checks: null });
		expect(screen.getByTestId('fiscal-year-checks-error').textContent).toMatch(/indisponibles/);
		expect(screen.queryByText(/Rien à signaler/)).toBeNull();
		expect(screen.queryByTestId('fiscal-year-checks-list')).toBeNull();
	});

	it('une réponse : sept lignes, dans l’ordre du serveur', () => {
		render(FiscalYearChecks, { loadState: 'ready', checks: closingChecks() });
		const rows = screen
			.getByTestId('fiscal-year-checks-list')
			.querySelectorAll('[data-testid^="fiscal-year-checks-row-"]');
		expect([...rows].map((r) => r.getAttribute('data-testid')?.replace('fiscal-year-checks-row-', ''))).toEqual(
			CODES
		);
		expect(screen.getAllByText('Rien à signaler')).toHaveLength(6);
		expect(screen.getByTestId('fiscal-year-checks-status-NEXT_FISCAL_YEAR').textContent).toMatch(
			/« Exercice 2027 » suit cet exercice/
		);
	});
});

describe('FiscalYearChecks — contenus et liens', () => {
	it('K2 : compteur, détail par compte, lien vers le rapprochement', () => {
		render(FiscalYearChecks, {
			loadState: 'ready',
			checks: closingChecks({
				PENDING_BANK_TRANSACTIONS: {
					status: 'attention',
					count: 3,
					details: {
						accounts: [{ bankAccountId: 5, name: 'UBS', count: 3, rejectedCount: 1 }]
					}
				}
			})
		});
		const status = screen.getByTestId('fiscal-year-checks-status-PENDING_BANK_TRANSACTIONS');
		expect(status.textContent).toMatch(/3 transaction\(s\) bancaire\(s\)/);
		expect(status.textContent).toMatch(/UBS : 3, dont 1 écartée\(s\)/);
		expect(screen.getByTestId('fiscal-year-checks-link-PENDING_BANK_TRANSACTIONS').getAttribute('href')).toBe(
			'/reconciliation'
		);
	});

	it('K3 : l’écart rendu par le serveur, au format suisse ; les non vérifiables à part', () => {
		render(FiscalYearChecks, {
			loadState: 'ready',
			checks: closingChecks({
				BANK_STATEMENT_AT_YEAR_END: {
					status: 'attention',
					count: 1,
					details: {
						accounts: [
							{
								bankAccountId: 5,
								name: 'UBS',
								reason: 'GAP',
								statementDate: '2026-12-31',
								statementClosingBalance: '100.00',
								ledgerBalanceAtStatement: '0',
								gap: '-100.00'
							}
						],
						notCheckable: [{ bankAccountId: 6, name: 'Poste', reason: 'NO_BALANCE_STATEMENT' }]
					}
				}
			})
		});
		const status = screen.getByTestId('fiscal-year-checks-status-BANK_STATEMENT_AT_YEAR_END');
		expect(status.textContent).toMatch(/UBS : écart de .*100\.00.* entre les livres et le relevé du 31\.12\.2026/);
		expect(screen.getByTestId('fiscal-year-checks-not-checkable').textContent).toMatch(
			/Poste : non vérifiable — aucun relevé importé ne porte de solde/
		);
		expect(screen.getByTestId('fiscal-year-checks-link-BANK_STATEMENT_AT_YEAR_END').getAttribute('href')).toBe(
			'/bank-accounts'
		);
	});

	it('K3 « ok » avec des comptes non vérifiables : « Rien à signaler » et la liste à part', () => {
		render(FiscalYearChecks, {
			loadState: 'ready',
			checks: closingChecks({
				BANK_STATEMENT_AT_YEAR_END: {
					details: {
						accounts: [],
						notCheckable: [{ bankAccountId: 6, name: 'Poste', reason: 'NOT_COMPARABLE' }]
					}
				}
			})
		});
		const status = screen.getByTestId('fiscal-year-checks-status-BANK_STATEMENT_AT_YEAR_END');
		expect(status.textContent).toMatch(/Rien à signaler/);
		expect(status.textContent).toMatch(/Poste : non vérifiable — sans compte de grand livre lié/);
	});

	it('K4 : lien vers les brouillons filtrés sur l’exercice', () => {
		render(FiscalYearChecks, {
			loadState: 'ready',
			checks: closingChecks({
				DRAFT_INVOICES: {
					status: 'attention',
					count: 1,
					details: { invoices: [{ id: 9, label: 'Dubarde SàRL', date: '2026-12-31' }] }
				}
			})
		});
		expect(screen.getByTestId('fiscal-year-checks-link-DRAFT_INVOICES').getAttribute('href')).toBe(
			'/invoices?status=draft&dateFrom=2026-01-01&dateTo=2026-12-31'
		);
		expect(screen.getByText('Dubarde SàRL').getAttribute('href')).toBe('/invoices/9');
	});

	it('K5 : lien vers les factures fournisseur importées', () => {
		render(FiscalYearChecks, {
			loadState: 'ready',
			checks: closingChecks({
				IMPORTED_SUPPLIER_INVOICES_TO_COMPLETE: { status: 'attention', count: 2 }
			})
		});
		expect(
			screen.getByTestId('fiscal-year-checks-link-IMPORTED_SUPPLIER_INVOICES_TO_COMPLETE').getAttribute('href')
		).toBe('/supplier-invoices/import');
	});

	it('K6 : chaque écriture mène à sa fiche, avec ses motifs', () => {
		render(FiscalYearChecks, {
			loadState: 'ready',
			checks: closingChecks({
				INCONSISTENT_ENTRIES: {
					status: 'attention',
					count: 1,
					details: {
						countByReason: {
							UNBALANCED: 1,
							TOO_FEW_LINES: 0,
							DATE_OUTSIDE_ITS_YEAR: 1,
							OTHER_YEAR_DATED_INSIDE: 0
						},
						entries: [
							{
								id: 42,
								entryNumber: 7,
								entryDate: '2027-01-01',
								fiscalYearName: 'Exercice 2026',
								reasons: ['UNBALANCED', 'DATE_OUTSIDE_ITS_YEAR']
							}
						]
					}
				}
			})
		});
		const link = screen.getByTestId('fiscal-year-checks-entry-42');
		expect(link.getAttribute('href')).toBe('/journal-entries/42');
		expect(link.textContent).toMatch(/N° 7 \(Exercice 2026, 01\.01\.2027\)/);
		expect(screen.getByTestId('fiscal-year-checks-status-INCONSISTENT_ENTRIES').textContent).toMatch(
			/déséquilibrée, datée hors de son exercice/
		);
	});

	it('K7 absent : le lien « Créer l’exercice suivant » seulement si le rôle peut créer', async () => {
		const missing = closingChecks({
			NEXT_FISCAL_YEAR: { status: 'attention', count: 1, details: { fiscalYear: null } }
		});
		const onCreate = vi.fn();
		const { unmount } = render(FiscalYearChecks, {
			loadState: 'ready',
			checks: missing,
			onCreateFiscalYear: onCreate
		});
		screen.getByTestId('fiscal-year-checks-link-NEXT_FISCAL_YEAR').click();
		expect(onCreate).toHaveBeenCalledOnce();
		unmount();
		render(FiscalYearChecks, { loadState: 'ready', checks: missing });
		expect(screen.queryByTestId('fiscal-year-checks-link-NEXT_FISCAL_YEAR')).toBeNull();
		expect(screen.getByTestId('fiscal-year-checks-status-NEXT_FISCAL_YEAR').textContent).toMatch(
			/Aucun exercice ne suit celui-ci/
		);
	});

	it('K1 : le plus ancien antérieur ouvert est nommé', () => {
		render(FiscalYearChecks, {
			loadState: 'ready',
			checks: closingChecks({
				EARLIER_FISCAL_YEAR_OPEN: {
					status: 'attention',
					count: 1,
					details: { fiscalYear: { id: 3, name: 'Exercice 2024' } }
				}
			})
		});
		expect(screen.getByTestId('fiscal-year-checks-status-EARLIER_FISCAL_YEAR_OPEN').textContent).toMatch(
			/« Exercice 2024 », plus ancien, est encore ouvert/
		);
	});
});
