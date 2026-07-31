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

	it("le bouton vide le champ même quand `value` est DÉJÀ null", async () => {
		// Sans ce cas : `onSelect(null)` ne change rien pour le parent, le `$effect`
		// ne se redéclenche pas, et le `preventDefault` du bouton a supprimé le
		// `blur` — le champ garderait un texte libre contredisant la valeur liée.
		// (Convergence Blind Hunter + Edge Case Hunter, passe 1 de revue.)
		const onSelect = vi.fn();
		const { getByRole, getByLabelText } = render(AccountAutocomplete, {
			accounts,
			value: null,
			allowClear: true,
			onSelect,
		});

		const input = getByRole('textbox') as HTMLInputElement;
		await fireEvent.input(input, { target: { value: 'zzz introuvable' } });
		await fireEvent.click(getByLabelText(/Effacer le compte/));

		expect(input.value).toBe('');
		// Rien à notifier au parent : la valeur était déjà `null`.
		expect(onSelect).not.toHaveBeenCalled();
	});

	it("tabuler à travers un champ RENSEIGNÉ sans rien taper n'efface rien", async () => {
		// Le geste le plus banal d'un formulaire — et le plus dangereux si le
		// `blur` agit sans distinguer « l'utilisateur a édité » de « l'utilisateur
		// est passé ». Ce cas n'était couvert par AUCUN test : découvert en passe 2
		// par mutation (traiter un brouillon `null` comme une chaîne vide laissait
		// toute la suite verte).
		const onSelect = vi.fn();
		const { getByRole } = render(AccountAutocomplete, {
			accounts,
			value: 1,
			allowClear: true,
			onSelect,
		});

		const input = getByRole('textbox') as HTMLInputElement;
		expect(input.value).toBe('3000 — Ventes');

		await fireEvent.focus(input);
		await fireEvent.blur(input);
		await afterBlurDelay();

		expect(onSelect).not.toHaveBeenCalled();
		expect(input.value).toBe('3000 — Ventes');
	});

	it("un `blur` SANS frappe n'efface rien, même si le compte n'est pas résoluble", async () => {
		// LE SECOND CHEMIN DE PERTE SILENCIEUSE (passe 2 de revue, Edge Case
		// Hunter). Tant que `fetchAccounts` n'a pas répondu, `accounts` est vide :
		// le libellé ne se résout pas et le champ paraît vide. L'ancienne
		// réconciliation y voyait un effacement volontaire et nullifiait la ligne
		// — sans le moindre geste destructif de l'utilisateur.
		const onSelect = vi.fn();
		const { getByRole } = render(AccountAutocomplete, {
			accounts: [], // liste pas encore arrivée
			value: 42,
			allowClear: true,
			onSelect,
		});

		const input = getByRole('textbox');
		await fireEvent.focus(input);
		await fireEvent.blur(input);
		await afterBlurDelay();

		expect(onSelect).not.toHaveBeenCalled();
	});

	it("un champ vidé alors que le compte n'est PAS résoluble ne nullifie pas non plus", async () => {
		const onSelect = vi.fn();
		const { getByRole } = render(AccountAutocomplete, {
			accounts: [],
			value: 42,
			allowClear: true,
			onSelect,
		});

		const input = getByRole('textbox');
		// L'utilisateur « efface » un champ qui était déjà vide à l'écran : ce
		// n'est pas une intention d'effacer un compte qu'il ne voit pas.
		await fireEvent.input(input, { target: { value: '' } });
		await fireEvent.blur(input);
		await afterBlurDelay();

		expect(onSelect).not.toHaveBeenCalled();
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

describe('AccountAutocomplete — `requiredAccountType` filtre le dropdown (passe 2)', () => {
	const accounts = [
		acc({ id: 1, number: '3000', name: 'Ventes', accountType: 'Revenue' }),
		acc({ id: 2, number: '4000', name: 'Achats', accountType: 'Expense' }),
	];

	it('sans `requiredAccountType` : tous les comptes imputables sont proposés', async () => {
		const { getByRole, queryByText } = render(AccountAutocomplete, {
			accounts,
			value: null,
			onSelect: () => {},
		});
		await fireEvent.focus(getByRole('textbox'));
		// Comportement des 4 consommateurs : inchangé.
		expect(queryByText('Achats')).not.toBeNull();
	});

	it("avec `requiredAccountType` : un compte du mauvais type n'est PAS proposé", async () => {
		// Sinon l'interface conduit l'utilisateur dans l'état bloquant qu'elle
		// vient de créer : il sélectionne « 4000 — Achats », la ligne est
		// aussitôt marquée invalide et l'enregistrement se désactive.
		const { getByRole, queryByText } = render(AccountAutocomplete, {
			accounts,
			value: null,
			requiredAccountType: 'Revenue',
			onSelect: () => {},
		});
		await fireEvent.focus(getByRole('textbox'));
		expect(queryByText('Ventes')).not.toBeNull();
		expect(queryByText('Achats')).toBeNull();
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

	it("le message d'invalidité est LIÉ au champ (aria-describedby)", () => {
		// Sans le lien, un lecteur d'écran annonce « champ invalide » sans jamais
		// énoncer POURQUOI (WCAG 3.3.1). Le patch de la passe 1 était livré SANS
		// test — il était retirable sans qu'aucune assertion ne rougisse.
		const { getByRole, getByText } = render(AccountAutocomplete, {
			accounts,
			value: 9,
			markInvalid: true,
			requiredAccountType: 'Revenue',
			onSelect: () => {},
		});

		const input = getByRole('textbox');
		const describedBy = input.getAttribute('aria-describedby');
		expect(input.getAttribute('aria-invalid')).toBe('true');
		expect(describedBy).toBeTruthy();
		expect(getByText(MARKER).getAttribute('id')).toBe(describedBy);
	});

	it("sans invalidité, aucun `aria-describedby` pendant", () => {
		const { getByRole } = render(AccountAutocomplete, {
			accounts,
			value: 8,
			markInvalid: true,
			requiredAccountType: 'Revenue',
			postableExemptAccountId: 8,
			onSelect: () => {},
		});
		expect(getByRole('textbox').getAttribute('aria-describedby')).toBeNull();
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
