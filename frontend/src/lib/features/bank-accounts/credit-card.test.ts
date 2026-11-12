// Story 34-11a2 (#972) — règles d'écran de la carte de crédit.
import { describe, it, expect } from 'vitest';
import { admitsLedgerType, cardLast4Payload, isValidCardLast4, withoutCreditCards } from './credit-card';

describe('credit-card — règles d’écran (Story 34-11a2)', () => {
	it('une carte n’admet qu’un passif ; un compte bancaire, un actif ou un passif', () => {
		for (const t of ['Asset', 'Liability', 'Revenue', 'Expense']) {
			expect(admitsLedgerType('credit_card', t)).toBe(t === 'Liability');
			expect(admitsLedgerType('bank', t)).toBe(t === 'Asset' || t === 'Liability');
		}
	});

	it('quatre chiffres exactement, ou rien', () => {
		for (const ok of ['', '  ', '1234', ' 0042 ']) expect(isValidCardLast4(ok), ok).toBe(true);
		for (const ko of ['123', '12345', '12a4', '１２３４', '4111 1111 1111 1111']) {
			expect(isValidCardLast4(ko), ko).toBe(false);
		}
		expect(cardLast4Payload('  ')).toBeNull();
		expect(cardLast4Payload(' 4821 ')).toBe('4821');
	});

	it('les cartes sont écartées du règlement client (C-e341-32)', () => {
		const comptes = [
			{ id: 1, accountKind: 'bank' as const },
			{ id: 2, accountKind: 'credit_card' as const },
		];
		expect(withoutCreditCards(comptes).map((c) => c.id)).toEqual([1]);
	});
});
