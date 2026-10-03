// Story 25-6-a (#388, #389) — les trois tuiles de l'accueil, isolément.
// i18nMsg est mocké pour renvoyer le fallback interpolé.
import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/svelte';

vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string, args?: Record<string, string | number>) =>
		fallback.replace(/\{ \$(\w+) \}/g, (_m, k) => String(args?.[k] ?? '')),
}));

import RecentEntriesCard from './RecentEntriesCard.svelte';
import OpenInvoicesCard from './OpenInvoicesCard.svelte';
import BankAccountsCard from './BankAccountsCard.svelte';
import type { JournalEntryResponse } from '$lib/features/journal-entries/journal-entries.types';
import type { BankAccountSummary } from '$lib/features/bank-accounts/bank-accounts.api';

const entry = {
	id: 7,
	entryNumber: 12,
	entryDate: '2026-03-10',
	description: 'Vente comptoir',
	lines: [
		{ debit: '250.00', credit: '0' },
		{ debit: '0', credit: '250.00' },
	],
} as unknown as JournalEntryResponse;

describe('RecentEntriesCard', () => {
	it('liste les écritures avec un lien vers chacune', () => {
		render(RecentEntriesCard, { state: 'ready', entries: [entry], canManage: true, isGuided: false });
		const row = screen.getByTestId('homepage-entry-row');
		expect(row.textContent).toContain('Vente comptoir');
		expect(row.textContent).toContain('250.00');
		expect(row.querySelector('a')?.getAttribute('href')).toBe('/journal-entries/7');
	});

	it('vide : le texte vide ; vide guidé : la variante guidée', () => {
		const { unmount } = render(RecentEntriesCard, {
			state: 'ready',
			entries: [],
			canManage: true,
			isGuided: false,
		});
		expect(screen.getByText('Aucune écriture.')).toBeTruthy();
		unmount();
		render(RecentEntriesCard, { state: 'ready', entries: [], canManage: true, isGuided: true });
		expect(screen.getByText(/première écriture/)).toBeTruthy();
	});

	it('un échec se dit, et prime sur la variante guidée', () => {
		render(RecentEntriesCard, { state: 'error', entries: [], canManage: true, isGuided: true });
		expect(screen.getByTestId('homepage-entries-unavailable')).toBeTruthy();
		expect(screen.queryByText(/Aucune écriture/)).toBeNull();
	});

	it('Consultation : ni bouton de saisie, ni injonction guidée', () => {
		render(RecentEntriesCard, { state: 'ready', entries: [], canManage: false, isGuided: true });
		expect(screen.queryByText('Saisir une écriture')).toBeNull();
		expect(screen.getByText('Aucune écriture.')).toBeTruthy();
	});
});

const summary = { unpaidCount: 3, unpaidTotal: '1250.5000', overdueCount: 1, overdueTotal: '400.00' };

describe('OpenInvoicesCard', () => {
	it('chiffre les factures ouvertes pour un rôle Consultation, sans bouton', () => {
		render(OpenInvoicesCard, {
			state: 'ready',
			summary,
			canManage: false,
			isGuided: false,
			reminderCount: 0,
		});
		const count = screen.getByTestId('homepage-invoices-open-count');
		expect(count.textContent).toMatch(/3 facture\(s\) ouverte\(s\) — 1.250\.50/);
		expect(count.getAttribute('data-amount')).toBe('1250.5000');
		expect(screen.getByTestId('homepage-invoices-overdue').textContent).toMatch(/dont 1 échue\(s\) — 400\.00/);
		expect(screen.queryByText('Créer une facture')).toBeNull();
	});

	it('« dont M échues » absent à zéro', () => {
		render(OpenInvoicesCard, {
			state: 'ready',
			summary: { ...summary, overdueCount: 0, overdueTotal: '0' },
			canManage: true,
			isGuided: false,
			reminderCount: 0,
		});
		expect(screen.queryByTestId('homepage-invoices-overdue')).toBeNull();
	});

	it('vide guidé : sans « première » ; Consultation : texte non guidé', () => {
		const empty = { unpaidCount: 0, unpaidTotal: '0', overdueCount: 0, overdueTotal: '0' };
		const { unmount } = render(OpenInvoicesCard, {
			state: 'ready',
			summary: empty,
			canManage: true,
			isGuided: true,
			reminderCount: 0,
		});
		expect(screen.getByText(/Créez une facture/).textContent).not.toMatch(/première/);
		unmount();
		render(OpenInvoicesCard, {
			state: 'ready',
			summary: empty,
			canManage: false,
			isGuided: true,
			reminderCount: 0,
		});
		expect(screen.getByText('Aucune facture ouverte.')).toBeTruthy();
	});

	it('un échec se dit', () => {
		render(OpenInvoicesCard, {
			state: 'error',
			summary: null,
			canManage: true,
			isGuided: false,
			reminderCount: 0,
		});
		expect(screen.getByTestId('homepage-invoices-unavailable')).toBeTruthy();
	});
});

function account(over: Partial<BankAccountSummary> = {}): BankAccountSummary {
	return {
		id: 1,
		bankName: 'UBS',
		iban: 'CH4431999123000889012',
		qrIban: null,
		isPrimary: true,
		journalAccountId: 10,
		version: 1,
		archived: false,
		currentBalance: '800.0000',
		lastTransactionDate: '2026-03-10',
		statementClosingBalance: null,
		statementDate: null,
		ledgerBalanceAtStatement: null,
		...over,
	};
}

describe('BankAccountsCard', () => {
	it('nomme le solde « solde comptable » et le total « Total (solde comptable) »', () => {
		render(BankAccountsCard, { accounts: [account()] });
		expect(screen.getByText('Solde comptable')).toBeTruthy();
		expect(screen.getByText(/Total \(solde comptable\)/)).toBeTruthy();
		expect(screen.queryByText(/liquidités/)).toBeNull();
	});

	it('affiche le relevé, et l’écart seulement s’il est non nul', () => {
		const { unmount } = render(BankAccountsCard, {
			accounts: [
				account({
					statementClosingBalance: '1000.00',
					statementDate: '2026-03-31',
					ledgerBalanceAtStatement: '800.0000',
				}),
			],
		});
		expect(screen.getByTestId('homepage-bank-statement-1').textContent).toMatch(/Relevé du 31\.03\.2026/);
		expect(screen.getByTestId('homepage-bank-gap-1').textContent).toMatch(/Écart/);
		unmount();
		render(BankAccountsCard, {
			accounts: [
				account({
					statementClosingBalance: '800.00',
					statementDate: '2026-03-31',
					ledgerBalanceAtStatement: '800.0000',
				}),
			],
		});
		expect(screen.queryByTestId('homepage-bank-gap-1')).toBeNull();
	});

	it('compte une fois un compte de grand livre partagé, avec sa note', () => {
		render(BankAccountsCard, {
			accounts: [account({ id: 1 }), account({ id: 2, iban: 'CH9300762011623852957' })],
		});
		expect(screen.getByTestId('homepage-bank-total').textContent).toMatch(/800\.00/);
		expect(screen.getByTestId('homepage-bank-total-shared')).toBeTruthy();
	});
});

describe('BankAccountsCard — revue P1', () => {
	it('un échec se dit (la tuile ne disparaît plus en silence)', () => {
		render(BankAccountsCard, { state: 'error', accounts: [] });
		expect(screen.getByTestId('homepage-bank-unavailable')).toBeTruthy();
	});

	it('compte du grand livre partagé : l’écart est dit non calculable, pas absent', () => {
		render(BankAccountsCard, {
			accounts: [
				account({
					statementClosingBalance: '900.00',
					statementDate: '2026-03-31',
					ledgerBalanceAtStatement: null,
				}),
			],
		});
		expect(screen.getByTestId('homepage-bank-gap-unavailable-1')).toBeTruthy();
		expect(screen.queryByTestId('homepage-bank-gap-1')).toBeNull();
	});

	it('compte NON lié avec relevé : ni écart ni « non calculable » (ce n’est pas un partage)', () => {
		render(BankAccountsCard, {
			accounts: [
				account({
					journalAccountId: null,
					currentBalance: null,
					statementClosingBalance: '900.00',
					statementDate: '2026-03-31',
					ledgerBalanceAtStatement: null,
				}),
			],
		});
		expect(screen.getByTestId('homepage-bank-statement-1')).toBeTruthy();
		expect(screen.queryByTestId('homepage-bank-gap-unavailable-1')).toBeNull();
	});
});
