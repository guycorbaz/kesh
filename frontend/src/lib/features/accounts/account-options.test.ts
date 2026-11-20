// Issue #271 — un compte de configuration devenu non-postable APRÈS coup
// disparaissait des options de son `<select>`, y affichait un vide, et se
// laissait effacer au premier `change`. Ces tests éprouvent le helper qui le
// réintroduit, dans les deux sens : ce qu'il doit rendre, et ce qu'il ne doit
// PAS ouvrir.

import { describe, it, expect } from 'vitest';
import type { AccountResponse } from './accounts.types';
import {
	accountIdsWithRole,
	buildAccountTree,
	bankAccountsNotLinkedTo,
	parentCandidates,
	parentFieldForUpdate,
	withCurrentAccount,
	withoutAccountIds,
} from './account-options';

function acc(
	partial: Partial<AccountResponse> & { id: number; number: string; name: string },
): AccountResponse {
	return {
		companyId: 1,
		accountType: 'Asset',
		active: true,
		role: null,
		postable: true,
		parentId: null,
		version: 1,
		createdAt: '',
		updatedAt: '',
		...partial,
	} as AccountResponse;
}

const CAISSE = acc({ id: 1000, number: '1000', name: 'Caisse' });
const BANQUE = acc({ id: 1020, number: '1020', name: 'Banque' });
/** Le cas de l'issue : configuré, puis devenu parent, donc non-postable. */
const CREANCES = acc({ id: 1100, number: '1100', name: 'Créances', postable: false });

const TOUS = [CAISSE, BANQUE, CREANCES];
const POSTABLES = TOUS.filter((a) => a.active && a.postable);

describe('withCurrentAccount — le défaut de #271', () => {
	it("réintroduit la valeur courante écartée par le filtre", () => {
		const options = withCurrentAccount(POSTABLES, CREANCES.id, TOUS);
		expect(options.map((a) => a.id)).toContain(CREANCES.id);
	});

	it('la place en tête, où elle ne se confond pas avec une option légitime', () => {
		const options = withCurrentAccount(POSTABLES, CREANCES.id, TOUS);
		expect(options[0].id).toBe(CREANCES.id);
	});

	it("n'ouvre PAS le filtre aux autres comptes non-postables", () => {
		const autreNonPostable = acc({ id: 2000, number: '2000', name: 'Dettes', postable: false });
		const options = withCurrentAccount(POSTABLES, CREANCES.id, [...TOUS, autreNonPostable]);
		expect(options.map((a) => a.id)).not.toContain(autreNonPostable.id);
	});
});

describe('withCurrentAccount — les cas où il ne doit rien faire', () => {
	it('rend la liste inchangée quand la valeur courante y figure déjà', () => {
		const options = withCurrentAccount(POSTABLES, BANQUE.id, TOUS);
		expect(options).toBe(POSTABLES);
	});

	it("ne duplique pas la valeur courante", () => {
		const options = withCurrentAccount(POSTABLES, BANQUE.id, TOUS);
		expect(options.filter((a) => a.id === BANQUE.id)).toHaveLength(1);
	});

	it('rend la liste inchangée sur un champ vide (null)', () => {
		expect(withCurrentAccount(POSTABLES, null, TOUS)).toBe(POSTABLES);
	});

	it('rend la liste inchangée sur un champ absent (undefined)', () => {
		expect(withCurrentAccount(POSTABLES, undefined, TOUS)).toBe(POSTABLES);
	});

	it("rend la liste inchangée quand l'identifiant n'est résoluble nulle part", () => {
		// Compte supprimé, ou liste complète pas encore chargée : on ne peut pas
		// fabriquer une option pour un compte qu'on ne connaît pas.
		expect(withCurrentAccount(POSTABLES, 999_999, TOUS)).toBe(POSTABLES);
		expect(withCurrentAccount(POSTABLES, CREANCES.id, [])).toBe(POSTABLES);
	});

	it("ne modifie pas la liste reçue", () => {
		const avant = [...POSTABLES];
		withCurrentAccount(POSTABLES, CREANCES.id, TOUS);
		expect(POSTABLES).toEqual(avant);
	});
});

describe('withCurrentAccount — un compte ARCHIVÉ configuré avant son archivage', () => {
	// Même classe que le non-postable : le filtre des trois sites porte sur
	// `active && postable`, donc un compte archivé après configuration disparaît
	// exactement de la même façon.
	const ARCHIVE = acc({ id: 1030, number: '1030', name: 'Poste', active: false });
	const tous = [...TOUS, ARCHIVE];
	const postables = tous.filter((a) => a.active && a.postable);

	it('est réintroduit lui aussi', () => {
		const options = withCurrentAccount(postables, ARCHIVE.id, tous);
		expect(options[0].id).toBe(ARCHIVE.id);
	});
});

// Story 15-6b (#474, AC8, test 17) — les écrans de règlement ne proposent plus le compte soldé.
describe('fonctions d’écran de la Story 15-6b', () => {
	const DEBITEURS = acc({ id: 1100, number: '1100', name: 'Débiteurs', role: 'Receivable' });
	const CREANCIERS = acc({
		id: 2000,
		number: '2000',
		name: 'Créanciers',
		accountType: 'Liability',
		role: 'Payable',
	});
	const SANS_ROLE = acc({ id: 1000, number: '1000', name: 'Caisse' });
	const PLAN = [SANS_ROLE, DEBITEURS, CREANCIERS];

	function bank(id: number, journalAccountId: number | null) {
		return { id, journalAccountId };
	}

	it('accountIdsWithRole rend les seuls comptes du rôle donné', () => {
		expect([...accountIdsWithRole(PLAN, 'Receivable')]).toEqual([1100]);
		expect([...accountIdsWithRole(PLAN, 'Payable')]).toEqual([2000]);
		expect(accountIdsWithRole(PLAN, 'VatPayable').size).toBe(0);
	});

	it('withoutAccountIds écarte le rôle donné et garde les autres, sans rôle compris', () => {
		const kept = withoutAccountIds(PLAN, accountIdsWithRole(PLAN, 'Receivable'));
		expect(kept.map((a) => a.id)).toEqual([1000, 2000]);
	});

	it('bankAccountsNotLinkedTo écarte les comptes bancaires liés, garde les autres et les non liés', () => {
		const ids = accountIdsWithRole(PLAN, 'Receivable');
		const kept = bankAccountsNotLinkedTo([bank(1, 1100), bank(2, 1020), bank(3, null)], ids);
		expect(kept.map((b) => b.id)).toEqual([2, 3]);
	});

	it('un compte bancaire lié à un débiteurs NON IMPUTABLE est écarté (ids calculés avant le filtre)', () => {
		const nonImputable = acc({ ...DEBITEURS, postable: false });
		const plan = [SANS_ROLE, nonImputable];
		// Le geste correct : les ids sur la liste reçue, puis le filtre `active && postable`.
		const ids = accountIdsWithRole(plan, 'Receivable');
		expect(bankAccountsNotLinkedTo([bank(1, 1100)], ids)).toEqual([]);
		// La mutation à éviter : calculer les ids APRÈS le filtre laisse passer le compte bancaire.
		const idsApresFiltre = accountIdsWithRole(
			plan.filter((a) => a.active && a.postable),
			'Receivable',
		);
		expect(bankAccountsNotLinkedTo([bank(1, 1100)], idsApresFiltre)).toHaveLength(1);
	});
});

// Correctif `fix-api-parent-compte` — le parent se choisit aussi à la
// modification : ni le compte lui-même, ni ses sous-comptes ; le champ ne
// part que s'il change.
describe('parentCandidates', () => {
	const G = acc({ id: 1, number: '10', name: 'Groupe' });
	const E = acc({ id: 2, number: '100', name: 'Enfant', parentId: 1 });
	const PE = acc({ id: 3, number: '1000', name: 'Petit-enfant', parentId: 2 });
	const V = acc({ id: 4, number: '20', name: 'Voisin' });
	const AR = acc({ id: 5, number: '19', name: 'Archivé', active: false });
	const M = acc({ id: 6, number: '1020', name: 'Mouvementé', currencyLocked: true });
	const PLAN = [PE, V, AR, E, G, M];

	it('à la création, tous les comptes actifs, mouvementés compris', () => {
		expect(parentCandidates(PLAN, null).map((a) => a.id)).toEqual([3, 4, 2, 1, 6]);
	});

	it('à la modification, ni le compte ni ses sous-comptes à tous les niveaux', () => {
		expect(parentCandidates(PLAN, G).map((a) => a.id)).toEqual([4]);
		expect(parentCandidates(PLAN, E).map((a) => a.id)).toEqual([4, 1]);
		expect(parentCandidates(PLAN, PE).map((a) => a.id)).toEqual([4, 2, 1]);
	});

	it('à la modification, ni les comptes qui portent des écritures, sauf le parent actuel', () => {
		expect(parentCandidates(PLAN, V).map((a) => a.id)).not.toContain(6);
		const sousM = acc({ id: 7, number: '1021', name: 'Sous M', parentId: 6 });
		expect(parentCandidates([...PLAN, sousM], sousM).map((a) => a.id)).toContain(6);
	});

	it('se termine sur une liste qui porterait un cycle', () => {
		const X = acc({ id: 7, number: '7', name: 'X', parentId: 8 });
		const Y = acc({ id: 8, number: '8', name: 'Y', parentId: 7 });
		expect(parentCandidates([X, Y, V], X).map((a) => a.id)).toEqual([4]);
	});
});

describe('parentFieldForUpdate', () => {
	it('absent quand le parent ne change pas', () => {
		expect(parentFieldForUpdate({ parentId: 1 }, '1')).toEqual({});
		expect(parentFieldForUpdate({ parentId: null }, '')).toEqual({});
	});

	it('l’identifiant choisi, ou null pour « Aucun »', () => {
		expect(parentFieldForUpdate({ parentId: null }, '4')).toEqual({ parentId: 4 });
		expect(parentFieldForUpdate({ parentId: 1 }, '')).toEqual({ parentId: null });
	});
});

// Issue #1439 — l'écran indentait selon la profondeur mais gardait l'ordre des
// numéros : 1100 rangé sous un groupe 29 paraissait rangé sous 1020.
describe('buildAccountTree', () => {
	const G29 = acc({ id: 29, number: '29', name: 'Groupe', postable: false });
	const C1100 = acc({ id: 1100, number: '1100', name: 'Créances', parentId: 29 });
	const C1101 = acc({ id: 1101, number: '1101', name: 'Autres', parentId: 1100 });
	const C1090 = acc({ id: 1090, number: '1090', name: 'Frère', parentId: 29 });

	it('range un compte sous son vrai parent, même si les numéros ne l’encadrent pas', () => {
		const arbre = buildAccountTree([CAISSE, BANQUE, C1100, G29]);
		expect(arbre.map((n) => [n.number, n.level])).toEqual([
			['1000', 0],
			['1020', 0],
			['29', 0],
			['1100', 1],
		]);
	});

	it('parcourt en profondeur, frères triés par numéro, quel que soit l’ordre reçu', () => {
		const arbre = buildAccountTree([C1101, C1100, C1090, G29, BANQUE]);
		expect(arbre.map((n) => [n.number, n.level])).toEqual([
			['1020', 0],
			['29', 0],
			['1090', 1],
			['1100', 1],
			['1101', 2],
		]);
	});

	it('traite un parent absent de la liste comme une racine', () => {
		const arbre = buildAccountTree([C1100]);
		expect(arbre.map((n) => [n.number, n.level])).toEqual([['1100', 0]]);
	});

	it('n’en perd aucun dans un cycle', () => {
		const a = acc({ id: 1, number: '1', name: 'A', parentId: 2 });
		const b = acc({ id: 2, number: '2', name: 'B', parentId: 1 });
		const arbre = buildAccountTree([a, b]);
		expect(arbre.map((n) => n.id).sort()).toEqual([1, 2]);
	});
});
