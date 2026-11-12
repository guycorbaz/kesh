// Story 34-3 (#967, AC-2, AC-3, AC-4) — la liste des contreparties d'une écriture manuelle ou
// d'un éclatement, et le rôle qui appelle l'avis débiteurs / créanciers.
//
// Le plan de test porte un compte de chaque classe 1 à 9, un compte non imputable, un compte
// archivé et le compte de la banque : la liste rendue se compare à l'ENSEMBLE attendu, non à
// son cardinal (un échange d'un compte contre un autre ne passerait pas).

import { describe, it, expect } from 'vitest';
import type { AccountResponse } from '$lib/features/accounts/accounts.types';
import { claimRoleOf, claimRoleOfId, counterpartyAccounts } from './reconciliation-counterparties';

function compte(
	id: number,
	number: string,
	overrides: Partial<AccountResponse> = {},
): AccountResponse {
	return {
		id,
		companyId: 1,
		number,
		name: `Compte ${number}`,
		accountType: 'Asset',
		parentId: null,
		active: true,
		role: null,
		postable: true,
		letterable: false,
		version: 1,
		createdAt: '2026-01-01T00:00:00Z',
		updatedAt: '2026-01-01T00:00:00Z',
		...overrides,
	};
}

/** Un compte de chaque classe 1 à 9, plus les trois comptes qui ne doivent jamais figurer. */
function planDeTest(): AccountResponse[] {
	return [
		compte(1, '1000', { name: 'Caisse' }),
		compte(2, '2100', { accountType: 'Liability', name: 'Emprunt' }),
		compte(3, '3200', { accountType: 'Revenue', name: 'Ventes de marchandises' }),
		compte(4, '4200', { accountType: 'Expense', name: 'Achats de marchandises' }),
		compte(5, '5000', { accountType: 'Expense', name: 'Salaires' }),
		compte(6, '6900', { accountType: 'Expense', name: 'Frais financiers' }),
		compte(7, '7500', { accountType: 'Revenue', name: 'Produits annexes' }),
		compte(8, '8500', { accountType: 'Revenue', name: 'Produits hors exploitation' }),
		compte(9, '9100', { accountType: 'Liability', name: 'Compte de bilan d’ouverture' }),
		compte(10, '1020', { name: 'Banque' }), // le compte de la banque
		compte(11, '2800', { accountType: 'Liability', name: 'Fonds propres', postable: false }),
		compte(12, '6500', { accountType: 'Expense', name: 'Ancien compte', active: false }),
	];
}

const ATTENDU = ['1000', '2100', '3200', '4200', '5000', '6900', '7500', '8500', '9100'];

describe('counterpartyAccounts (Story 34-3)', () => {
	it('AC-2 — tout compte actif et imputable, de toute classe, sauf celui de la banque', () => {
		const numeros = counterpartyAccounts(planDeTest(), 10).map((a) => a.number);
		expect(new Set(numeros)).toEqual(new Set(ATTENDU));
		expect(numeros).toHaveLength(ATTENDU.length);
	});

	it('AC-3 — repli null du compte de la banque : aucun filtrage par identifiant', () => {
		const numeros = counterpartyAccounts(planDeTest(), null).map((a) => a.number);
		expect(new Set(numeros)).toEqual(new Set([...ATTENDU, '1020']));
	});

	it('le compte de la banque est écarté par son identifiant, quel que soit son numéro', () => {
		const plan = [...planDeTest(), compte(13, '5999', { name: 'Banque atypique' })];
		const numeros = counterpartyAccounts(plan, 13).map((a) => a.number);
		expect(numeros).not.toContain('5999');
		expect(numeros).toContain('1020');
	});
});

describe('claimRoleOf (Story 34-3, AC-4)', () => {
	it('rend Receivable ou Payable, et seulement eux', () => {
		expect(claimRoleOf(compte(1, '1100', { role: 'Receivable' }))).toBe('Receivable');
		expect(claimRoleOf(compte(2, '2000', { role: 'Payable' }))).toBe('Payable');
	});

	it.each([
		['VatRecoverable'],
		['VatPayable'],
		['VatSettlement'],
		['DefaultRevenue'],
		['EquityCapital'],
	] as const)('le rôle %s n’avertit pas (hors périmètre, C-e342-1)', (role) => {
		expect(claimRoleOf(compte(1, '1170', { role }))).toBeNull();
	});

	it('un compte 1100 ou 2000 sans rôle posé n’avertit pas (rôle explicite, Story 14-3a)', () => {
		expect(claimRoleOf(compte(1, '1100'))).toBeNull();
		expect(claimRoleOf(compte(2, '2000', { accountType: 'Liability' }))).toBeNull();
	});

	it('aucun compte choisi : aucun rôle', () => {
		expect(claimRoleOf(undefined)).toBeNull();
		expect(claimRoleOf(null)).toBeNull();
	});
});

// Revue de code P1 (B1-1) — la seule règle d'affichage de l'avis, par identifiant.
describe('claimRoleOfId (Story 34-3, AC-4)', () => {
	const plan = [
		compte(1, '1100', { role: 'Receivable' }),
		compte(2, '2000', { accountType: 'Liability', role: 'Payable' }),
		compte(3, '6900', { accountType: 'Expense' }),
	];

	it('rend le rôle du compte de la liste', () => {
		expect(claimRoleOfId(plan, 1)).toBe('Receivable');
		expect(claimRoleOfId(plan, 2)).toBe('Payable');
		expect(claimRoleOfId(plan, 3)).toBeNull();
	});

	it('aucun compte choisi, ou compte hors de la liste : aucun rôle', () => {
		expect(claimRoleOfId(plan, null)).toBeNull();
		expect(claimRoleOfId(plan, 99)).toBeNull();
	});
});
