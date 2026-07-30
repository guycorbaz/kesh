// Story 14-3b — le sélecteur de SAISIE d'écriture ne doit proposer que les
// comptes actifs ET postables (le backend rejette désormais une ligne manuelle
// vers un compte non-postable). Pattern : mock i18n AVANT l'import du composant
// (hoisting Vitest), render via @testing-library/svelte (Svelte 5).
//
// Story 16-1b — les props opt-in `allowClear` / `markInvalid` /
// `requiredAccountType`. **Tous les tests d'ici sont orientés non-régression des
// 4 consommateurs existants** (`JournalEntryForm`, `VatPurchaseAssistant`,
// `TransactionSplitModal`, `ManualMatchModal`) : le défaut de chaque prop doit
// préserver strictement le comportement antérieur.
//
// Le mock i18n suit l'emplacement CANONIQUE (`shared/utils/i18n.svelte`, AC3).
// Il visait `features/onboarding/onboarding.svelte` jusqu'à la 16-1b — mock
// devenu mort après le déplacement de l'import, et invisible parce que
// `i18nMsg` retombe de toute façon sur son argument `fallback`.

import { describe, it, expect, vi } from 'vitest';
import { render, fireEvent } from '@testing-library/svelte';
import type { AccountResponse } from '$lib/features/accounts/accounts.types';

vi.mock('$lib/shared/utils/i18n.svelte', () => ({
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

/** `handleBlur` diffère de 150 ms pour laisser passer un clic sur le dropdown. */
async function afterBlurDelay() {
	await new Promise((r) => setTimeout(r, 200));
}

const MARKER = /Compte invalide/;

describe('AccountAutocomplete — filtre postable (Story 14-3b)', () => {
	it('ne propose que les comptes actifs ET postables', async () => {
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

describe('AccountAutocomplete — non-régression des 4 consommateurs (Story 16-1b, AC13)', () => {
	// Le compte 4400 est le témoin qui rend ce bloc discriminant : `Expense` ET
	// non-postable, donc invalide sur les DEUX critères. Sans lui, les tests
	// « aucun marqueur » passeraient aussi avec un marqueur inconditionnel.
	const accounts = [
		acc({ id: 1, number: '1000', name: 'Caisse' }),
		acc({ id: 44, number: '4400', name: 'Charges groupées', accountType: 'Expense', postable: false }),
	];

	it("sans `allowClear` : aucun bouton d'effacement", () => {
		const { queryByLabelText } = render(AccountAutocomplete, {
			accounts,
			value: 1,
			onSelect: () => {},
		});
		expect(queryByLabelText(/Effacer le compte/)).toBeNull();
	});

	it("sans `allowClear` : vider le champ au clavier ne nullifie RIEN (dette #271)", async () => {
		const onSelect = vi.fn();
		const { getByRole } = render(AccountAutocomplete, { accounts, value: 1, onSelect });

		const input = getByRole('textbox');
		await fireEvent.input(input, { target: { value: '' } });
		await fireEvent.blur(input);
		await afterBlurDelay();

		expect(onSelect).not.toHaveBeenCalled();
	});

	it("sans `markInvalid` : AUCUN marqueur, même sur un compte `Expense` non-postable", () => {
		const { queryByText } = render(AccountAutocomplete, {
			accounts,
			value: 44,
			onSelect: () => {},
		});
		expect(queryByText(MARKER)).toBeNull();
	});
});

describe('AccountAutocomplete — `allowClear` (Story 16-1b, AC1 / AC1-bis)', () => {
	const accounts = [acc({ id: 1, number: '3000', name: 'Ventes', accountType: 'Revenue' })];

	it("le bouton d'effacement appelle `onSelect(null)` une seule fois", async () => {
		const onSelect = vi.fn();
		const { getByLabelText } = render(AccountAutocomplete, {
			accounts,
			value: 1,
			allowClear: true,
			onSelect,
		});

		await fireEvent.click(getByLabelText(/Effacer le compte/));
		await afterBlurDelay();

		expect(onSelect).toHaveBeenCalledTimes(1);
		expect(onSelect).toHaveBeenCalledWith(null);
	});

	it('vider le champ au clavier vaut effacement explicite, une seule fois', async () => {
		const onSelect = vi.fn();
		const { getByRole } = render(AccountAutocomplete, {
			accounts,
			value: 1,
			allowClear: true,
			onSelect,
		});

		const input = getByRole('textbox');
		await fireEvent.input(input, { target: { value: '' } });
		await fireEvent.blur(input);
		await afterBlurDelay();

		expect(onSelect).toHaveBeenCalledTimes(1);
		expect(onSelect).toHaveBeenCalledWith(null);
	});

	it("un texte libre jamais validé est restauré au `blur`, sans nullifier la valeur", async () => {
		const onSelect = vi.fn();
		const { getByRole } = render(AccountAutocomplete, {
			accounts,
			value: 1,
			allowClear: true,
			onSelect,
		});

		const input = getByRole('textbox') as HTMLInputElement;
		await fireEvent.input(input, { target: { value: 'zzz introuvable' } });
		await fireEvent.blur(input);
		await afterBlurDelay();

		// Le champ ne peut jamais afficher un texte qui contredit `value`.
		expect(input.value).toBe('3000 — Ventes');
		expect(onSelect).not.toHaveBeenCalled();
	});
});

describe('AccountAutocomplete — `markInvalid` (Story 16-1b, AC2)', () => {
	const archived = acc({
		id: 9,
		number: '3900',
		name: 'Ventes closes',
		accountType: 'Revenue',
		active: false,
	});
	const collective = acc({
		id: 8,
		number: '3800',
		name: 'Groupe produits',
		accountType: 'Revenue',
		postable: false,
	});
	const expense = acc({ id: 7, number: '4000', name: 'Achats', accountType: 'Expense' });
	const accounts = [archived, collective, expense];

	it('un compte archivé garde son libellé ET reçoit le marqueur', () => {
		const { getByRole, queryByText } = render(AccountAutocomplete, {
			accounts,
			value: 9,
			markInvalid: true,
			requiredAccountType: 'Revenue',
			onSelect: () => {},
		});

		// Le libellé est résolu sur la liste COMPLÈTE (D11) : il s'affiche.
		expect((getByRole('textbox') as HTMLInputElement).value).toBe('3900 — Ventes closes');
		expect(queryByText(MARKER)).not.toBeNull();
	});

	it("un compte du mauvais type reçoit le marqueur", () => {
		const { queryByText } = render(AccountAutocomplete, {
			accounts,
			value: 7,
			markInvalid: true,
			requiredAccountType: 'Revenue',
			onSelect: () => {},
		});
		expect(queryByText(MARKER)).not.toBeNull();
	});

	it('un compte non-postable reçoit le marqueur…', () => {
		const { queryByText } = render(AccountAutocomplete, {
			accounts,
			value: 8,
			markInvalid: true,
			requiredAccountType: 'Revenue',
			onSelect: () => {},
		});
		expect(queryByText(MARKER)).not.toBeNull();
	});

	it("…SAUF s'il est le défaut société (exemption miroir de 16-1a D3-bis)", () => {
		const { queryByText } = render(AccountAutocomplete, {
			accounts,
			value: 8,
			markInvalid: true,
			requiredAccountType: 'Revenue',
			postableExemptAccountId: 8,
			onSelect: () => {},
		});
		// Le backend accepte ce cas : le frontend ne doit JAMAIS bloquer ce que
		// 16-1a accepte, sinon l'utilisateur est enfermé.
		expect(queryByText(MARKER)).toBeNull();
	});

	it("l'exemption ne couvre QUE `postable` : archivé reste marqué", () => {
		const { queryByText } = render(AccountAutocomplete, {
			accounts,
			value: 9,
			markInvalid: true,
			requiredAccountType: 'Revenue',
			postableExemptAccountId: 9,
			onSelect: () => {},
		});
		expect(queryByText(MARKER)).not.toBeNull();
	});
});
