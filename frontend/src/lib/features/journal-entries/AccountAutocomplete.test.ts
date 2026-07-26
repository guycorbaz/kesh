// Story 14-3b — le sélecteur de SAISIE d'écriture ne doit proposer que les
// comptes actifs ET postables (le backend rejette désormais une ligne manuelle
// vers un compte non-postable). Pattern : mock i18n AVANT l'import du composant
// (hoisting Vitest), render via @testing-library/svelte (Svelte 5).

import { describe, it, expect, vi } from 'vitest';
import { render, fireEvent } from '@testing-library/svelte';
import type { AccountResponse } from '$lib/features/accounts/accounts.types';

vi.mock('$lib/features/onboarding/onboarding.svelte', () => ({
	i18nMsg: (_key: string, fallback: string) => fallback,
}));

import AccountAutocomplete from './AccountAutocomplete.svelte';

function acc(
	partial: Partial<AccountResponse> & { id: number; number: string; name: string },
): AccountResponse {
	return {
		companyId: 1,
		accountType: 'Asset',
		active: true,
		role: null,
		postable: true,
		...partial,
	} as AccountResponse;
}

describe('AccountAutocomplete — filtre postable (Story 14-3b)', () => {
	it("ne propose que les comptes actifs ET postables", async () => {
		const accounts = [
			acc({ id: 1, number: '1000', name: 'Caisse', active: true, postable: true }),
			acc({ id: 2, number: '2979', name: 'Résultat', active: true, postable: false }),
			acc({ id: 3, number: '9999', name: 'Archivé', active: false, postable: true }),
		];
		const { getByRole, getAllByRole, queryByText } = render(AccountAutocomplete, {
			accounts,
			value: null,
			onSelect: () => {},
		});

		// Focus → ouverture du dropdown (filtered = active.slice(0, 20)).
		await fireEvent.focus(getByRole('textbox'));

		const options = getAllByRole('option');
		expect(options).toHaveLength(1);
		expect(queryByText('Caisse')).not.toBeNull(); // actif + postable → visible
		expect(queryByText('Résultat')).toBeNull(); // actif mais non-postable → masqué
		expect(queryByText('Archivé')).toBeNull(); // postable mais inactif → masqué
	});
});
