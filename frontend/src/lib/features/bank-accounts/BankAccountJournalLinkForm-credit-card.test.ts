// Story 34-11a2 (#972 ; revue P1, A8, M13) — le formulaire de lien d'une carte de
// crédit : seuls les passifs sont proposés, et « Délier » n'est pas offert (le
// serveur refuse de délier une carte, `CREDIT_CARD_LEDGER_MUST_BE_LIABILITY`).

import { describe, it, expect, vi } from 'vitest';
import { render } from '@testing-library/svelte';
import type { AccountResponse } from '$lib/features/accounts/accounts.types';
import type { BankAccountSummary } from './bank-accounts.api';

vi.mock('./bank-accounts.api', () => ({
	updateBankAccountJournalLink: vi.fn(),
}));

vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string) => fallback,
}));

import BankAccountJournalLinkForm from './BankAccountJournalLinkForm.svelte';

function compte(id: number, number: string, accountType: AccountResponse['accountType']): AccountResponse {
	return {
		id,
		companyId: 1,
		number,
		name: `Compte ${number}`,
		accountType,
		parentId: null,
		active: true,
		role: null,
		postable: true,
		letterable: false,
		version: 1,
		createdAt: '2026-01-01T00:00:00Z',
		updatedAt: '2026-01-01T00:00:00Z',
	};
}

const COMPTES = [compte(20, '1020', 'Asset'), compte(30, '2100', 'Liability'), compte(31, '2101', 'Liability')];

function carte(kind: BankAccountSummary['accountKind']): BankAccountSummary {
	return {
		id: 7,
		bankName: 'Visa',
		description: null,
		iban: kind === 'bank' ? 'CH4431999123000889012' : null,
		qrIban: null,
		accountKind: kind,
		cardLast4: kind === 'bank' ? null : '4821',
		isPrimary: false,
		journalAccountId: 30,
		version: 3,
		archived: false,
		closedOn: null,
		currentBalance: null,
		statementClosingBalance: null,
		statementDate: null,
		ledgerBalanceAtStatement: null,
		statementGap: null,
		lastTransactionDate: null,
	};
}

function monter(kind: BankAccountSummary['accountKind']) {
	return render(BankAccountJournalLinkForm, {
		bankAccount: carte(kind),
		accounts: COMPTES,
		claimAccountIds: new Set<number>(),
		onSuccess: vi.fn(),
		onCancel: vi.fn(),
	});
}

const options = (c: HTMLElement) =>
	Array.from((c.querySelector('[data-testid="journal-account-select"]') as HTMLSelectElement).options).map(
		(o) => o.value,
	);

describe('BankAccountJournalLinkForm — carte de crédit (Story 34-11a2)', () => {
	it('une carte : passifs seuls, pas de « Délier »', () => {
		const r = monter('credit_card');
		expect(options(r.container)).toContain('30');
		expect(options(r.container)).toContain('31');
		expect(options(r.container)).not.toContain('20');
		expect(r.queryByTestId('unlink-button')).toBeNull();
	});

	it('un compte bancaire, témoin : l’actif proposé, « Délier » offert', () => {
		const r = monter('bank');
		expect(options(r.container)).toContain('20');
		expect(r.queryByTestId('unlink-button')).not.toBeNull();
	});
});
