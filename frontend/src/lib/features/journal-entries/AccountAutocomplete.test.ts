// Story 14-3b — le sélecteur de SAISIE d'écriture ne doit proposer que les
// comptes actifs ET postables (le backend rejette désormais une ligne manuelle
// vers un compte non-postable). Pattern : mock i18n AVANT l'import du composant
// (hoisting Vitest), render via @testing-library/svelte (Svelte 5).
//
// Story 16-1b — les props opt-in `allowClear` / `markInvalid` /
// `requiredAccountType`. **Les tests d'ici sont orientés non-régression des
// consommateurs stricts** (`JournalEntryForm`, `VatPurchaseAssistant`,
// `TransactionSplitModal`, `ManualMatchModal`) : le défaut de chaque prop doit
// préserver le comportement antérieur — à une exception près, voulue : depuis la
// Story 35-7 (#1000), un champ strict quitté jette son brouillon et réaffiche le
// compte retenu, au lieu de montrer un texte que la valeur contredit.
//
// Le mock i18n suit l'emplacement CANONIQUE (`shared/utils/i18n.svelte`, AC3).
// Il visait `features/onboarding/onboarding.svelte` jusqu'à la 16-1b — mock
// devenu mort après le déplacement de l'import, et invisible parce que
// `i18nMsg` retombe de toute façon sur son argument `fallback`.

import { describe, it, expect, vi } from 'vitest';
import { render, fireEvent } from '@testing-library/svelte';
import { tick, type ComponentProps } from 'svelte';
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

/**
 * Laisse les effets Svelte se propager après une interaction.
 *
 * ⚠️ Ce helper décrivait « `handleBlur` diffère de 150 ms » — **devenu faux** à la
 * refonte de la passe 2 : le `blur` est désormais **synchrone**. Le délai ne sert
 * plus qu'à laisser tourner les `$effect`, et il **masque** tout défaut de
 * timing : une opération asynchrone réintroduite dans `handleBlur` ne serait
 * plus détectée. Les tests qui éprouvent une séquence serrée (vider puis
 * enregistrer) ne doivent donc PAS l'utiliser.
 * (Commentaire périmé relevé en passe 3 de revue.)
 */
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

describe('AccountAutocomplete — consommateurs stricts (Story 16-1b, AC13 ; Story 35-7)', () => {
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

	it("sans `allowClear` : vider le champ puis le quitter ne nullifie rien (dette #271) et réaffiche le compte (Story 35-7)", async () => {
		const onSelect = vi.fn();
		const { getByRole } = render(AccountAutocomplete, { accounts, value: 1, onSelect });

		const input = getByRole('textbox') as HTMLInputElement;
		await fireEvent.input(input, { target: { value: '' } });
		await fireEvent.blur(input);
		await afterBlurDelay();

		expect(onSelect).not.toHaveBeenCalled();
		// Story 35-7 (#1000, AC1) : le champ ne contredit pas la valeur qui partira.
		expect(input.value).toBe('1000 — Caisse');
	});

	it('sans `allowClear` : un texte libre jamais choisi est jeté au `blur` (Story 35-7, AC1)', async () => {
		const onSelect = vi.fn();
		const { getByRole } = render(AccountAutocomplete, { accounts, value: 1, onSelect });

		const input = getByRole('textbox') as HTMLInputElement;
		await fireEvent.input(input, { target: { value: 'zzz' } });
		await fireEvent.blur(input);

		expect(input.value).toBe('1000 — Caisse');
		expect(onSelect).not.toHaveBeenCalled();
	});

	it('sans `allowClear`, valeur nulle : un texte libre est jeté au `blur`, le champ reste vide (Story 35-7, AC1)', async () => {
		const onSelect = vi.fn();
		const { getByRole } = render(AccountAutocomplete, { accounts, value: null, onSelect });

		const input = getByRole('textbox') as HTMLInputElement;
		await fireEvent.input(input, { target: { value: 'zzz' } });
		await fireEvent.blur(input);

		expect(input.value).toBe('');
		expect(onSelect).not.toHaveBeenCalled();
	});

	it('mode dégradé (`loadError`) : le brouillon tapé reste affiché au `blur` (Story 35-7, AC1)', async () => {
		// `onSelect` nu : `value` n'est PAS mise à jour, sans quoi l'effet qui suit `value`
		// réafficherait `7` de toute façon et le test ne distinguerait rien.
		const onSelect = vi.fn();
		const { getByRole } = render(AccountAutocomplete, {
			accounts: [],
			value: null,
			loadError: true,
			onSelect,
		});

		const input = getByRole('textbox') as HTMLInputElement;
		await fireEvent.input(input, { target: { value: '7' } });
		await fireEvent.blur(input);

		expect(input.value).toBe('7');
		expect(onSelect).toHaveBeenCalledWith(7);
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
		// Comportement des consommateurs existants : inchangé.
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

// ── Story 36-1 (#1095, #1165) — la touche Entrée ─────────────────────────────
//
// Deux principes (fiche, D1) : (i) tant qu'un brouillon est affiché, Entrée ne
// laisse jamais partir le formulaire englobant — elle résout le brouillon, et
// une seconde Entrée envoie ; (ii) une surbrillance ne vaut choix que si elle a
// été DÉSIGNÉE, par une saisie non blanche ou par une flèche.
//
// La soumission implicite d'un `<form>` n'existe pas dans jsdom : l'envoi se
// constate par la valeur rendue de `fireEvent.keyDown` — `false` quand
// `preventDefault` a été appelé (le navigateur n'enverrait rien), `true` sinon.
// Chaque cas porte la lettre de l'AC1 de la fiche qu'il prouve.
describe('AccountAutocomplete — Entrée (Story 36-1, #1095, #1165)', () => {
	const V3000 = acc({ id: 1, number: '3000', name: 'Ventes', accountType: 'Revenue' });
	const V3001 = acc({ id: 2, number: '3001', name: 'Prestations', accountType: 'Revenue' });
	const COMPTES = [V3000, V3001];
	const NO_MATCH = /Aucun compte ne correspond/;

	type Props = ComponentProps<typeof AccountAutocomplete>;

	/**
	 * Monte le champ et RÉINJECTE la valeur choisie, comme le fait un parent
	 * (fiche, AC1, V2-5) : sans cela, l'affichage retomberait sur la valeur de
	 * départ après un choix ou un effacement, et le test constaterait autre
	 * chose que ce que voit l'utilisateur.
	 */
	function monter(props: Partial<Omit<Props, 'onSelect'>> = {}) {
		const onSelect = vi.fn();
		let reinjecter: (p: Partial<Props>) => void = () => {};
		const r = render(AccountAutocomplete, {
			accounts: COMPTES,
			value: null,
			...props,
			onSelect: (id: number | null) => {
				onSelect(id);
				reinjecter({ value: id });
			},
		});
		reinjecter = r.rerender;
		const input = r.getByRole('textbox') as HTMLInputElement;
		return { ...r, input, onSelect };
	}

	const taper = (input: HTMLElement, v: string) => fireEvent.input(input, { target: { value: v } });
	const entree = (input: HTMLElement, init: KeyboardEventInit = {}) =>
		fireEvent.keyDown(input, { key: 'Enter', ...init });
	const touche = (input: HTMLElement, key: string) => fireEvent.keyDown(input, { key });

	/** Options portant `aria-selected="true"` — la désignation. */
	function designees(container: HTMLElement): HTMLElement[] {
		return Array.from(container.querySelectorAll('[role="option"][aria-selected="true"]'));
	}
	function statut(container: HTMLElement): HTMLElement {
		const el = container.querySelector('[role="status"]');
		if (!el) throw new Error('région role="status" absente');
		return el as HTMLElement;
	}

	it('a. texte sans correspondance, Entrée : rien ne part, le brouillon reste, le message est annoncé', async () => {
		const { input, onSelect, container, queryByText } = monter({ value: 1, allowClear: true });
		await taper(input, 'zzz');

		expect(await entree(input)).toBe(false);
		expect(onSelect).not.toHaveBeenCalled();
		expect(input.value).toBe('zzz');
		const msg = queryByText(NO_MATCH);
		expect(msg).not.toBeNull();
		const region = statut(container);
		expect(region.contains(msg)).toBe(true);
		expect(region.id).not.toBe('');
		expect((input.getAttribute('aria-describedby') ?? '').split(' ')).toContain(region.id);
	});

	it('b. texte sans correspondance, Échap, Entrée : idem (liste fermée)', async () => {
		const { input, onSelect, queryByText } = monter({ value: 1, allowClear: true });
		await taper(input, 'zzz');
		await touche(input, 'Escape');

		expect(await entree(input)).toBe(false);
		expect(onSelect).not.toHaveBeenCalled();
		expect(input.value).toBe('zzz');
		expect(queryByText(NO_MATCH)).not.toBeNull();
	});

	it('c. saisie qui correspond, Entrée : le premier compte désigné est choisi (non-régression)', async () => {
		const { input, onSelect } = monter();
		await taper(input, '300');

		expect(await entree(input)).toBe(false);
		expect(onSelect).toHaveBeenCalledTimes(1);
		expect(onSelect).toHaveBeenLastCalledWith(1);
		expect(input.value).toBe('3000 — Ventes');

		await taper(input, '300');
		await touche(input, 'ArrowDown');
		expect(await entree(input)).toBe(false);
		expect(onSelect).toHaveBeenLastCalledWith(2);
	});

	it('d. correspondance unique, Échap, Entrée : la liste se rouvre, rien n’est choisi ; une seconde Entrée choisit', async () => {
		const { input, onSelect, queryByRole } = monter();
		await taper(input, '3001');
		await touche(input, 'Escape');
		expect(queryByRole('listbox')).toBeNull();

		expect(await entree(input)).toBe(false);
		expect(onSelect).not.toHaveBeenCalled();
		expect(queryByRole('listbox')).not.toBeNull();

		expect(await entree(input)).toBe(false);
		expect(onSelect).toHaveBeenCalledTimes(1);
		expect(onSelect).toHaveBeenCalledWith(2);
	});

	it('e. allowClear : vidé, Échap, Entrée = effacement explicite, une fois', async () => {
		const { input, onSelect } = monter({ value: 1, allowClear: true });
		await taper(input, '');
		await touche(input, 'Escape');

		expect(await entree(input)).toBe(false);
		expect(onSelect).toHaveBeenCalledTimes(1);
		expect(onSelect).toHaveBeenCalledWith(null);
		expect(input.value).toBe('');
	});

	it('f. strict : vidé, Échap, Entrée = brouillon jeté, compte retenu réaffiché', async () => {
		const { input, onSelect } = monter({ value: 1 });
		await taper(input, '');
		await touche(input, 'Escape');

		expect(await entree(input)).toBe(false);
		expect(onSelect).not.toHaveBeenCalled();
		expect(input.value).toBe('3000 — Ventes');
	});

	it('g. non touché à valeur résolue : Entrée laisse partir le formulaire', async () => {
		const { input, onSelect } = monter({ value: 1, allowClear: true });
		await fireEvent.focus(input);

		expect(await entree(input)).toBe(true);
		expect(onSelect).not.toHaveBeenCalled();
		expect(input.value).toBe('3000 — Ventes');
	});

	describe('h. le message disparaît', () => {
		async function apresA() {
			const m = monter({ value: 1, allowClear: true });
			await taper(m.input, 'zzz');
			await entree(m.input);
			expect(m.queryByText(NO_MATCH)).not.toBeNull();
			return m;
		}
		function sansMessage(m: Awaited<ReturnType<typeof apresA>>) {
			expect(m.queryByText(NO_MATCH)).toBeNull();
			expect(statut(m.container).textContent?.trim()).toBe('');
		}

		it('à la frappe', async () => {
			const m = await apresA();
			await taper(m.input, 'zz');
			sansMessage(m);
		});

		it('au blur', async () => {
			const m = await apresA();
			await fireEvent.blur(m.input);
			sansMessage(m);
		});

		it('au bouton ×', async () => {
			const m = await apresA();
			await fireEvent.click(m.getByLabelText(/Effacer le compte/));
			expect(m.onSelect).toHaveBeenCalledWith(null);
			sansMessage(m);
		});

		it('au choix d’un compte', async () => {
			const m = await apresA();
			await taper(m.input, '30');
			const option = m.getAllByRole('option').find((o) => o.textContent?.includes('3000'));
			await fireEvent.mouseDown(option!);
			expect(m.onSelect).toHaveBeenCalledWith(1);
			sansMessage(m);
		});

		it('retaper le texte refusé ne le fait pas reparaître sans nouvelle Entrée', async () => {
			const m = await apresA();
			await taper(m.input, 'zzzx');
			await taper(m.input, 'zzz');
			sansMessage(m);
		});

		it('quand les comptes rechargés rendent la saisie résoluble', async () => {
			const m = await apresA();
			m.rerender({
				accounts: [...COMPTES, acc({ id: 9, number: '3900', name: 'zzz Conseil', accountType: 'Revenue' })],
			});
			await tick();
			expect(m.input.value).toBe('zzz');
			sansMessage(m);
		});
	});

	it('i. mode dégradé : Entrée inchangée (l’événement suit son cours)', async () => {
		const { input } = monter({ value: null, loadError: true, accounts: [] });
		await taper(input, '7');
		expect(await entree(input)).toBe(true);
	});

	it('j. composition IME en cours : Entrée n’est pas interceptée, aucun message', async () => {
		const { input, queryByText } = monter({ value: 1, allowClear: true });
		await taper(input, 'zzz');
		expect(await entree(input, { isComposing: true })).toBe(true);
		expect(queryByText(NO_MATCH)).toBeNull();
	});

	it('k. sans saisie, la première flèche bas désigne le PREMIER compte', async () => {
		const a = monter();
		await fireEvent.focus(a.input);
		await touche(a.input, 'ArrowDown');
		expect(await entree(a.input)).toBe(false);
		expect(a.onSelect).toHaveBeenCalledWith(1);
		a.unmount();

		const b = monter({ value: 1, allowClear: true });
		await taper(b.input, '');
		await touche(b.input, 'ArrowDown');
		expect(await entree(b.input)).toBe(false);
		expect(b.onSelect).toHaveBeenCalledTimes(1);
		expect(b.onSelect).toHaveBeenCalledWith(1);
	});

	it('l. (#1165) ligne neuve, focus : liste sans désignation, Entrée ne choisit rien et laisse partir', async () => {
		const { input, onSelect, container, queryByRole } = monter({ value: null, allowClear: true });
		await fireEvent.focus(input);
		expect(queryByRole('listbox')).not.toBeNull();
		expect(designees(container)).toHaveLength(0);

		expect(await entree(input)).toBe(true);
		expect(onSelect).not.toHaveBeenCalled();
		expect(input.value).toBe('');
	});

	it('m. comptes pas encore chargés : Entrée retient le brouillon, sans message', async () => {
		const { input, onSelect, queryByText } = monter({ value: null, accounts: [] });
		await taper(input, 'zzz');
		expect(await entree(input)).toBe(false);
		expect(onSelect).not.toHaveBeenCalled();
		expect(queryByText(NO_MATCH)).toBeNull();
	});

	it('n. (#1165) allowClear : vidé, liste ouverte, Entrée = effacement, non le premier compte', async () => {
		const { input, onSelect } = monter({ value: 1, allowClear: true });
		await taper(input, '');
		expect(await entree(input)).toBe(false);
		expect(onSelect).toHaveBeenCalledTimes(1);
		expect(onSelect).toHaveBeenCalledWith(null);
		expect(input.value).toBe('');
	});

	it('o. strict : vidé, liste ouverte, Entrée = compte retenu réaffiché', async () => {
		const { input, onSelect } = monter({ value: 1 });
		await taper(input, '');
		expect(await entree(input)).toBe(false);
		expect(onSelect).not.toHaveBeenCalled();
		expect(input.value).toBe('3000 — Ventes');
	});

	it('p. flèche haut depuis une liste fermée : ouvre la liste sur le DERNIER compte', async () => {
		const { input, onSelect, container, queryByRole } = monter({ value: 1, allowClear: true });
		await taper(input, '');
		await touche(input, 'Escape');
		await touche(input, 'ArrowUp');

		expect(queryByRole('listbox')).not.toBeNull();
		const options = Array.from(container.querySelectorAll('[role="option"]'));
		expect(designees(container)).toEqual([options[options.length - 1]]);
		expect(await entree(input)).toBe(false);
		expect(onSelect).toHaveBeenCalledWith(2);
	});

	it('q. liste arrivée sous un brouillon : la saisie désigne le premier compte', async () => {
		const { input, onSelect, container, rerender } = monter({ value: null, accounts: [] });
		await taper(input, '300');
		await touche(input, 'ArrowDown');
		rerender({ accounts: COMPTES });
		await tick();

		const d = designees(container);
		expect(d).toHaveLength(1);
		expect(d[0].textContent).toContain('3000');
		expect(await entree(input)).toBe(false);
		expect(onSelect).toHaveBeenCalledWith(1);
	});

	it('r. le bouton × annule la désignation', async () => {
		const { input, onSelect, container, queryByRole, getByLabelText } = monter({
			value: 1,
			allowClear: true,
		});
		await taper(input, '30');
		expect(designees(container)).toHaveLength(1);

		await fireEvent.click(getByLabelText(/Effacer le compte/));
		expect(onSelect).toHaveBeenCalledTimes(1);
		expect(onSelect).toHaveBeenCalledWith(null);
		expect(queryByRole('listbox')).not.toBeNull();
		expect(designees(container)).toHaveLength(0);

		expect(await entree(input)).toBe(true);
		expect(onSelect).toHaveBeenCalledTimes(1);
	});

	it('s. la désignation se voit (fond primary plein), distincte du survol (border)', async () => {
		const { input, container } = monter();
		await fireEvent.focus(input);
		const options = () => Array.from(container.querySelectorAll('[role="option"]'));

		for (const o of options()) {
			expect(o.classList.contains('bg-primary')).toBe(false);
			expect(o.classList.contains('text-primary-foreground')).toBe(false);
			expect(o.classList.contains('hover:bg-border')).toBe(true);
		}

		await touche(input, 'ArrowDown');
		const [premiere, ...autres] = options();
		expect(premiere.getAttribute('aria-selected')).toBe('true');
		expect(premiere.classList.contains('bg-primary')).toBe(true);
		expect(premiere.classList.contains('text-primary-foreground')).toBe(true);
		expect(premiere.classList.contains('hover:bg-border')).toBe(false);
		for (const o of autres) {
			expect(o.classList.contains('hover:bg-border')).toBe(true);
			expect(o.classList.contains('bg-primary')).toBe(false);
			expect(o.classList.contains('text-primary-foreground')).toBe(false);
		}
		for (const o of options()) expect(o.classList.contains('bg-accent')).toBe(false);
	});

	it('t. une frappe qui ne change pas la liste désigne quand même par la saisie', async () => {
		const { input, onSelect, container } = monter();
		await taper(input, '30');
		await touche(input, 'Escape');
		await taper(input, '300');

		const d = designees(container);
		expect(d).toHaveLength(1);
		expect(d[0].textContent).toContain('3000');
		expect(await entree(input)).toBe(false);
		expect(onSelect).toHaveBeenCalledTimes(1);
		expect(onSelect).toHaveBeenCalledWith(1);
	});

	it('u. Entrée maintenue : les répétitions n’envoient ni ne choisissent rien', async () => {
		const a = monter({ value: 1, allowClear: true });
		await taper(a.input, '');
		expect(await entree(a.input)).toBe(false);
		expect(a.onSelect).toHaveBeenCalledWith(null);
		expect(await entree(a.input, { repeat: true })).toBe(false);
		expect(a.onSelect).toHaveBeenCalledTimes(1);
		a.unmount();

		const b = monter({ value: 1, allowClear: true });
		await fireEvent.focus(b.input);
		expect(await entree(b.input, { repeat: true })).toBe(false);
		expect(b.onSelect).not.toHaveBeenCalled();
	});

	it('v. les mêmes comptes dans un nouveau tableau ne défont pas la désignation', async () => {
		const { input, onSelect, container, rerender } = monter();
		await fireEvent.focus(input);
		await touche(input, 'ArrowDown');
		await touche(input, 'ArrowDown');
		expect(designees(container)[0]?.textContent).toContain('3001');

		rerender({ accounts: [...COMPTES] });
		await tick();
		expect(designees(container)[0]?.textContent).toContain('3001');
		expect(await entree(input)).toBe(false);
		expect(onSelect).toHaveBeenCalledWith(2);
	});

	it('w. une valeur posée par le parent, même non résoluble, annule la désignation', async () => {
		const { input, onSelect, container, rerender } = monter();
		await fireEvent.focus(input);
		await touche(input, 'ArrowDown');
		expect(designees(container)).toHaveLength(1);

		rerender({ value: 999 });
		await tick();
		expect(designees(container)).toHaveLength(0);
		expect(await entree(input)).toBe(true);
		expect(onSelect).not.toHaveBeenCalled();
	});

	it('x. la région de statut ne prend aucune place sans message', async () => {
		const { input, container } = monter({ value: 1, allowClear: true });
		expect(statut(container).children).toHaveLength(0);

		await taper(input, 'zzz');
		await entree(input);
		expect(statut(container).children).toHaveLength(1);
		expect(statut(container).children[0].tagName).toBe('P');
	});
});
