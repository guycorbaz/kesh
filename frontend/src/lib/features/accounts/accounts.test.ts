/**
 * Story 14-3a — rôles de comptes : contrat de types et client API.
 *
 * Le rendu de la page (colonne Rôle, badge non-postable, bouton Réactiver
 * réservé aux lignes archivées) est couvert par le E2E Playwright
 * `tests/e2e/accounts.spec.ts` — la page est une route avec `$effect` +
 * store d'auth, peu testable en isolation.
 */
import { describe, expect, it, vi, beforeEach } from 'vitest';
import {
	ACCOUNT_ROLES,
	accountRoleKey,
	type AccountRole,
} from './accounts.types';

vi.mock('$lib/shared/utils/api-client', () => ({
	apiClient: {
		get: vi.fn(),
		post: vi.fn(),
		put: vi.fn(),
	},
	isApiError: vi.fn(),
}));

import { apiClient } from '$lib/shared/utils/api-client';
import {
	reactivateAccount,
	updateAccount,
	createAccount,
} from './accounts.api';

describe('accountRoleKey', () => {
	it('dérive la clé i18n en kebab-case pour les 10 rôles', () => {
		expect(accountRoleKey('Receivable')).toBe('account-role-receivable');
		expect(accountRoleKey('DefaultRevenue')).toBe(
			'account-role-default-revenue',
		);
		expect(accountRoleKey('VatRecoverable')).toBe(
			'account-role-vat-recoverable',
		);
		expect(accountRoleKey('EquityCapital')).toBe('account-role-equity-capital');
		expect(accountRoleKey('RetainedEarnings')).toBe(
			'account-role-retained-earnings',
		);
		expect(accountRoleKey('CurrentYearResult')).toBe(
			'account-role-current-year-result',
		);
	});

	it('couvre exhaustivement ACCOUNT_ROLES sans collision', () => {
		// Un rôle ajouté à l'enum sans clé i18n correspondante produirait un
		// libellé manquant en silence ; ce test force la cohérence de la dérivation.
		const keys = ACCOUNT_ROLES.map(accountRoleKey);
		expect(keys).toHaveLength(10);
		expect(new Set(keys).size).toBe(10);
		for (const k of keys) {
			expect(k).toMatch(/^account-role-[a-z]+(-[a-z]+)*$/);
		}
	});

	it("n'accepte que les 10 rôles du contrat backend", () => {
		// Miroir de kesh_db::entities::account::AccountRole::ALL.
		expect([...ACCOUNT_ROLES]).toEqual([
			'Receivable',
			'DefaultRevenue',
			'Payable',
			'VatRecoverable',
			'VatPayable',
			'VatSettlement',
			'EquityCapital',
			'EquityOther',
			'RetainedEarnings',
			'CurrentYearResult',
		] satisfies AccountRole[]);
	});
});

describe('accounts.api', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('reactivateAccount appelle PUT /accounts/{id}/reactivate avec la version', async () => {
		await reactivateAccount(7, { version: 3 });
		expect(apiClient.put).toHaveBeenCalledWith(
			'/api/v1/accounts/7/reactivate',
			{ version: 3 },
		);
	});

	it('updateAccount transmet role et postable (contrat full-replace)', async () => {
		await updateAccount(7, {
			name: 'Débiteurs',
			accountType: 'Asset',
			role: 'Receivable',
			postable: true,
			version: 2,
		});
		expect(apiClient.put).toHaveBeenCalledWith(
			'/api/v1/accounts/7',
			expect.objectContaining({
				role: 'Receivable',
				postable: true,
				version: 2,
			}),
		);
	});

	it('updateAccount peut retirer un rôle en envoyant null explicitement', async () => {
		await updateAccount(7, {
			name: 'Débiteurs',
			accountType: 'Asset',
			role: null,
			postable: true,
			version: 2,
		});
		expect(apiClient.put).toHaveBeenCalledWith(
			'/api/v1/accounts/7',
			expect.objectContaining({ role: null }),
		);
	});

	it('createAccount transmet le rôle choisi', async () => {
		await createAccount({
			number: '1100',
			name: 'Débiteurs',
			accountType: 'Asset',
			role: 'Receivable',
			postable: true,
		});
		expect(apiClient.post).toHaveBeenCalledWith(
			'/api/v1/accounts',
			expect.objectContaining({ role: 'Receivable' }),
		);
	});
});
