// Issue #271 — un compte de configuration devenu non-postable APRÈS coup
// disparaissait des options de son `<select>`, y affichait un vide, et se
// laissait effacer au premier `change`. Ces tests éprouvent le helper qui le
// réintroduit, dans les deux sens : ce qu'il doit rendre, et ce qu'il ne doit
// PAS ouvrir.

import { describe, it, expect } from 'vitest';
import type { AccountResponse } from './accounts.types';
import { withCurrentAccount } from './account-options';

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
