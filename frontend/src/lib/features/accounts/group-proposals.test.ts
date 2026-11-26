// Story 46-5b (AC-2, AC-8) — le calcul pur des propositions de groupes, sur le
// plan synthétique de la maquette (`kesh-gate-logs/s46-5/maquette/`, hors dépôt).
import { describe, it, expect } from 'vitest';
import type { AccountResponse, AccountType } from './accounts.types';
import type { BankAccountSummary } from '$lib/features/bank-accounts/bank-accounts.api';
import {
	applyEdit,
	appliedProposals,
	bankLinkedAccounts,
	buildGroupsRequest,
	checkProposals,
	commonLeadingWords,
	isGroupAccount,
	longestCommonPrefix,
	parentOptionsFor,
	previewTree,
	proposalSkeletons,
	proposalSummary,
	readGroupRefusal,
	selectableAccounts,
	type CheckedProposal,
	type GroupMode,
	type ProposalEdit,
	type ProposalEdits,
} from './group-proposals';

type Row = [number: string, name: string, type: AccountType, parent: string | null, entries: boolean, currency?: string, active?: boolean];

/** Le plan de la maquette (numéros et libellés génériques). */
const PLAN: Row[] = [
	['1', 'Actifs', 'Asset', null, false],
	['10', 'Actifs circulants', 'Asset', '1', false],
	['1000', 'Caisse', 'Asset', '10', true],
	['1010', 'Poste', 'Asset', '10', true],
	['1020', 'Banque', 'Asset', '10', true],
	['10200', 'Banque D CHF', 'Asset', '10', true],
	['10201', 'Banque D EUR', 'Asset', '10', true, 'EUR'],
	['1030', 'Banque A CHF', 'Asset', '10', true],
	['1031', 'Banque A EUR', 'Asset', '10', true, 'EUR'],
	['1032', 'Banque A USD', 'Asset', '10', true, 'USD'],
	['1040', 'Avoirs à court terme', 'Asset', '10', false],
	['10400', 'Dépôt à terme 1', 'Asset', '10', true],
	['10401', 'Dépôt à terme 2', 'Asset', '10', true],
	['1050', 'Banque B CHF', 'Asset', '10', true],
	['1051', 'Banque B EUR', 'Asset', '10', true, 'EUR'],
	['1058', 'Banque B épargne', 'Asset', '10', true],
	['1060', 'Prestataire P CHF', 'Asset', '10', true],
	['1061', 'Prestataire P EUR', 'Asset', '10', true, 'EUR'],
	['1068', 'Prestataire Q CHF', 'Asset', '10', true],
	['1069', 'Prestataire Q EUR', 'Asset', '10', true, 'EUR'],
	['1070', 'Prestataire Q USD', 'Asset', '10', true, 'USD'],
	['108', 'Comptes de dépôt', 'Asset', '10', false],
	['1080', 'Dépôt de garantie locaux', 'Asset', '108', true],
	['1081', 'Dépôt de garantie véhicule', 'Asset', '10', true],
	['1090', 'Ancien compte', 'Asset', '10', false, undefined, false],
	['6', 'Autres charges', 'Expense', null, false],
	['6000', 'Loyers', 'Expense', '6', true],
	['6900', 'Charges financières', 'Expense', '6', true],
	['6901', 'Frais bancaires', 'Expense', '6', true],
	['6905', "Produits d'intérêts", 'Revenue', '6', true],
	['7', 'Groupe mouvementé', 'Revenue', null, true],
	['7000', 'Ventes A', 'Revenue', '7', true],
	['7900', 'Produits divers 1', 'Revenue', null, true],
	['7901', 'Produits divers 2', 'Revenue', null, true],
];

function makePlan(rows: Row[] = PLAN): AccountResponse[] {
	const ids = new Map(rows.map((r, i) => [r[0], i + 1]));
	return rows.map(([number, name, accountType, parent, entries, currency, active], i) => ({
		id: i + 1,
		companyId: 1,
		number,
		name,
		accountType,
		parentId: parent === null ? null : (ids.get(parent) as number),
		active: active ?? true,
		role: null,
		postable: true,
		letterable: false,
		...(currency ? { currency } : {}),
		...(entries ? { currencyLocked: true } : {}),
		version: 3,
		createdAt: '2026-01-01T00:00:00',
		updatedAt: '2026-01-01T00:00:00',
	}));
}

const accounts = makePlan();
const idOf = (n: string) => (accounts.find((a) => a.number === n) as AccountResponse).id;
const sel = (...numbers: string[]) => new Set(numbers.map(idOf));

function run(selection: Set<number>, mode: GroupMode = 'digit', edits: ProposalEdits = {}, plan = accounts) {
	const set = proposalSkeletons(plan, selection, mode);
	return { set, checked: checkProposals(set, edits, plan, selection, mode) };
}

const byKey = (checked: CheckedProposal[], key: string) => checked.find((p) => p.key === key) as CheckedProposal;
const codes = (p: CheckedProposal, level?: string) =>
	p.messages.filter((m) => level === undefined || m.level === level).map((m) => m.code);
/** Un geste de l'utilisateur, rangé comme le range la page (`applyEdit`). */
const edit = (p: CheckedProposal, patch: Partial<ProposalEdit>, edits: ProposalEdits = {}) => applyEdit(edits, p, patch);

describe('aides', () => {
	it('plus long préfixe commun et mots de tête communs', () => {
		expect(longestCommonPrefix(['1030', '1031', '1032'])).toBe('103');
		expect(longestCommonPrefix([])).toBe('');
		expect(commonLeadingWords(['Banque A CHF', 'Banque A EUR'])).toBe('Banque A');
		expect(commonLeadingWords(['Caisse', 'Poste'])).toBe('');
	});

	it('un compte de groupe a un sous-compte actif et ne se coche pas', () => {
		expect(isGroupAccount(accounts[idOf('10') - 1], accounts)).toBe(true);
		expect(isGroupAccount(accounts[idOf('1040') - 1], accounts)).toBe(false);
		const selectable = selectableAccounts(accounts).map((a) => a.number);
		expect(selectable).not.toContain('10');
		expect(selectable).not.toContain('108');
		expect(selectable).not.toContain('1090'); // archivé
		expect(selectable).toContain('1040');
	});
});

describe('mode « numéro sans son dernier chiffre »', () => {
	it('1030, 1031, 1032 → créer 103 « Banque A » sous 10, devises mêlées', () => {
		const { checked } = run(sel('1030', '1031', '1032'));
		expect(checked).toHaveLength(1);
		const p = checked[0];
		expect(p.key).toBe('digit:103');
		expect(p.kind).toBe('create');
		expect(p.edit.number).toBe('103');
		expect(p.edit.name).toBe('Banque A');
		expect(p.edit.parent).toBe(idOf('10'));
		expect(p.hasError).toBe(false);
		expect(codes(p, 'info')).toContain('mixedCurrencies');
		expect(p.moves.map((m) => m.number)).toEqual(['1030', '1031', '1032']);
	});

	it('un préfixe d’un seul compte ne donne aucune proposition, il est rappelé', () => {
		const { set } = run(sel('1000', '1010'));
		expect(set.skeletons).toHaveLength(0);
		expect(set.singles.map((s) => s.number)).toEqual(['1000', '1010']);
	});

	it('règle du dernier chiffre : chaque compte va au groupe de son numéro sans le dernier caractère', () => {
		const { set } = run(sel('1030', '1031', '1050', '1051'));
		expect(set.skeletons.map((s) => [s.key, s.members.map((m) => m.number)])).toEqual([
			['digit:103', ['1030', '1031']],
			['digit:105', ['1050', '1051']],
		]);
	});

	it('1081 seul → rattachement au groupe existant 108', () => {
		const { checked } = run(sel('1081'));
		const p = byKey(checked, 'digit:108');
		expect(p.kind).toBe('attach');
		expect(p.existing?.number).toBe('108');
		expect(codes(p, 'info')).toContain('attachExisting');
		expect(p.hasError).toBe(false);
		expect(p.moves.map((m) => m.number)).toEqual(['1081']);
	});

	it('un groupe existant dont tous les comptes sont déjà les sous-comptes ne donne rien', () => {
		const { set } = run(sel('1080'));
		expect(set.skeletons).toHaveLength(0);
		expect(set.singles).toHaveLength(0);
	});

	it('10200, 10201 → 1020 mouvementé refusé, puis renommé 102 accepté', () => {
		const first = run(sel('10200', '10201'));
		const p = byKey(first.checked, 'digit:1020');
		expect(codes(p, 'error')).toContain('numberTakenEntries');
		expect(p.canReuse).toBe(false);
		const second = run(sel('10200', '10201'), 'digit', edit(p, { number: '102' }));
		const q = byKey(second.checked, 'digit:1020');
		expect(q.kind).toBe('create');
		expect(q.hasError).toBe(false);
	});

	it('10400, 10401 → 1040 sans écriture : erreur, puis réutilisation sur coche', () => {
		const first = run(sel('10400', '10401'));
		const p = byKey(first.checked, 'digit:1040');
		expect(codes(p, 'error')).toEqual(['numberTakenReusable']);
		expect(p.canReuse).toBe(true);
		const second = run(sel('10400', '10401'), 'digit', edit(p, { reuse: true }));
		const q = byKey(second.checked, 'digit:1040');
		expect(q.kind).toBe('reuse');
		expect(q.hasError).toBe(false);
		expect(codes(q, 'warning')).toContain('reuseNotPostable');
	});

	it('1050, 1051, 1058 : un trou dans les numéros, sans erreur', () => {
		const { checked } = run(sel('1050', '1051', '1058'));
		const p = byKey(checked, 'digit:105');
		expect(p.hasError).toBe(false);
		expect(p.members).toHaveLength(3);
	});

	it('1060–1061, 1068–1070 : un bloc de dix qui chevauche une dizaine, avertissement', () => {
		const { checked, set } = run(sel('1060', '1061', '1068', '1069', '1070'));
		const p = byKey(checked, 'digit:106');
		expect(codes(p, 'warning')).toContain('mixedNames');
		expect(p.hasError).toBe(false);
		expect(set.singles.map((s) => s.number)).toEqual(['1070']);
	});

	it('6900, 6901, 6905 : types mêlés, valides après exclusion de 6905', () => {
		const first = run(sel('6900', '6901', '6905'));
		const p = byKey(first.checked, 'digit:690');
		expect(codes(p, 'error')).toContain('mixedTypes');
		expect(codes(p, 'error')).toContain('nameEmpty'); // aucun mot de tête commun
		const second = run(sel('6900', '6901', '6905'), 'digit', edit(p, { excluded: [idOf('6905')], name: 'Charges financières' }));
		const q = byKey(second.checked, 'digit:690');
		expect(q.hasError).toBe(false);
		expect(q.included.map((m) => m.number)).toEqual(['6900', '6901']);
	});

	it('les comptes non cochés de même préfixe sont signalés', () => {
		const { checked } = run(sel('1030', '1031'));
		const p = byKey(checked, 'digit:103');
		expect(p.messages.find((m) => m.code === 'uncheckedSamePrefix')?.params.numbers).toBe('1032');
	});

	it('sans parent commun : parent à choisir (erreur), racine choisie : avertissement', () => {
		const { checked } = run(sel('6000', '7000'), 'one');
		const p = checked[0];
		expect(p.edit.parent).toBeNull();
		expect(codes(p, 'error')).toContain('parentMissing');
		const plan = makePlan([
			['900', 'Compte A', 'Asset', null, true],
			['901', 'Compte B', 'Asset', null, true],
		]);
		const root = run(new Set([1, 2]), 'digit', {}, plan).checked[0];
		expect(root.edit.parent).toBe('root');
		expect(codes(root, 'warning')).toContain('root');
		expect(root.hasError).toBe(false);
	});

	it('numéro d’un compte archivé → refus', () => {
		const { checked } = run(sel('1030', '1031'));
		const p = byKey(checked, 'digit:103');
		const r = run(sel('1030', '1031'), 'digit', edit(p, { number: '1090' }));
		expect(codes(byKey(r.checked, 'digit:103'), 'error')).toEqual(['numberTakenArchived']);
	});

	it('groupe existant mouvementé → refus', () => {
		const { checked } = run(sel('7900', '7901'));
		const p = byKey(checked, 'digit:790');
		const r = run(sel('7900', '7901'), 'digit', edit(p, { number: '7' }));
		const q = byKey(r.checked, 'digit:790');
		expect(q.kind).toBe('attach');
		expect(codes(q, 'error')).toContain('groupHasEntries');
	});

	it('groupe existant d’un autre type → refus', () => {
		const { checked } = run(sel('1030', '1031'));
		const p = byKey(checked, 'digit:103');
		const r = run(sel('1030', '1031'), 'digit', edit(p, { number: '6' }));
		expect(codes(byKey(r.checked, 'digit:103'), 'error')).toContain('groupTypeMismatch');
	});

	it('parent mouvementé → refus', () => {
		const { checked } = run(sel('1030', '1031'));
		const p = byKey(checked, 'digit:103');
		const r = run(sel('1030', '1031'), 'digit', edit(p, { parent: idOf('1000') }));
		expect(codes(byKey(r.checked, 'digit:103'), 'error')).toContain('parentHasEntries');
	});

	it('numéro vide, trop long (octets), nom vide, nom trop long', () => {
		const { checked } = run(sel('1030', '1031'));
		const p = byKey(checked, 'digit:103');
		const r = (patch: Partial<ProposalEdit>) =>
			codes(byKey(run(sel('1030', '1031'), 'digit', edit(p, patch)).checked, 'digit:103'), 'error');
		expect(r({ number: '  ' })).toContain('numberEmpty');
		expect(r({ number: 'éééééé' })).toContain('numberTooLong'); // 12 octets
		expect(r({ number: 'ééééé' })).not.toContain('numberTooLong'); // 10 octets
		expect(r({ name: ' ' })).toContain('nameEmpty');
		expect(r({ name: 'é'.repeat(128) })).toContain('nameTooLong');
	});

	it('numéro d’un compte de la proposition → refus', () => {
		const { checked } = run(sel('1030', '1031'));
		const p = byKey(checked, 'digit:103');
		const r = run(sel('1030', '1031'), 'digit', edit(p, { number: '1031' }));
		expect(codes(byKey(r.checked, 'digit:103'), 'error')).toContain('numberIsMember');
	});

	it('aucun membre retenu → refus', () => {
		const { checked } = run(sel('1030', '1031'));
		const p = byKey(checked, 'digit:103');
		const r = run(sel('1030', '1031'), 'digit', edit(p, { excluded: [idOf('1030'), idOf('1031')] }));
		expect(codes(byKey(r.checked, 'digit:103'), 'error')).toEqual(['empty']);
	});
});

describe('contrôles entre propositions acceptées', () => {
	it('deux propositions acceptées du même numéro (sans casse) → erreur des deux côtés', () => {
		const s = sel('1030', '1031', '1050', '1051');
		const { checked } = run(s);
		const a = byKey(checked, 'digit:103');
		const b = byKey(checked, 'digit:105');
		const edits = {
			...edit(a, { number: '10x', state: 'accepted' }),
			...edit(b, { number: '10X', state: 'accepted' }),
		};
		const r = run(s, 'digit', edits).checked;
		expect(codes(byKey(r, 'digit:103'), 'error')).toContain('numberDuplicate');
		expect(codes(byKey(r, 'digit:105'), 'error')).toContain('numberDuplicate');
		// Une seule acceptée : pas de doublon.
		const r2 = run(s, 'digit', { ...edits, ...edit(b, { number: '10X', state: 'pending' }) }).checked;
		expect(codes(byKey(r2, 'digit:103'), 'error')).not.toContain('numberDuplicate');
	});

	it('le compte d’un rattachement membre d’une autre proposition acceptée → un niveau à la fois', () => {
		const plan = makePlan([
			['10', 'Actifs circulants', 'Asset', null, false],
			['1040', 'Avoirs', 'Asset', '10', false],
			['1041', 'Avoirs 2', 'Asset', '10', true],
			['10400', 'Dépôt 1', 'Asset', '10', true],
			['10401', 'Dépôt 2', 'Asset', '10', true],
		]);
		const selection = new Set([2, 3, 4, 5]);
		const first = run(selection, 'digit', {}, plan).checked;
		const reuse = byKey(first, 'digit:1040');
		const upper = byKey(first, 'digit:104');
		const edits = { ...edit(reuse, { reuse: true, state: 'accepted' }), ...edit(upper, { state: 'accepted' }) };
		const r = run(selection, 'digit', edits, plan).checked;
		expect(codes(byKey(r, 'digit:1040'), 'error')).toContain('groupIsMember');
	});
});

describe('mode « un seul groupe pour la sélection »', () => {
	it('1000, 1010, 1020 → 100 sous 10', () => {
		const { checked } = run(sel('1000', '1010', '1020'), 'one');
		expect(checked).toHaveLength(1);
		expect(checked[0].key).toBe('one');
		expect(checked[0].edit.number).toBe('100');
		expect(checked[0].edit.parent).toBe(idOf('10'));
	});

	it('100 déjà pris → numéro vide à saisir', () => {
		const plan = makePlan([...PLAN, ['100', 'Pris', 'Asset', '10', false]]);
		const ids = (ns: string[]) => new Set(ns.map((n) => plan.find((a) => a.number === n)!.id));
		const { checked } = run(ids(['1000', '1010', '1020']), 'one', {}, plan);
		expect(checked[0].edit.number).toBe('');
		expect(codes(checked[0], 'error')).toContain('numberEmpty');
	});

	it('le préfixe d’un seul compte se groupe quand même', () => {
		const { checked } = run(sel('1081'), 'one');
		expect(checked).toHaveLength(1);
	});
});

describe('états, demande, décompte, aperçu', () => {
	const s = sel('1030', '1031', '1032', '1050', '1051', '10400', '10401');
	const base = run(s).checked;
	const a103 = byKey(base, 'digit:103');
	const a105 = byKey(base, 'digit:105');
	const r1040 = byKey(base, 'digit:1040');

	it('rien n’est accepté d’office : Confirmer inactif', () => {
		const sum = proposalSummary(base);
		expect(sum.canConfirm).toBe(false);
		expect(sum.pending).toBe(3);
		expect(buildGroupsRequest(base).request.groups).toHaveLength(0);
	});

	it('la demande ne porte que les propositions acceptées et sans erreur, ni écartées ni en attente', () => {
		const edits = {
			...edit(a103, { state: 'accepted' }),
			...edit(a105, { state: 'rejected' }),
			...edit(r1040, { reuse: true, state: 'accepted' }),
		};
		const checked = run(s, 'digit', edits).checked;
		const { request, keys } = buildGroupsRequest(checked);
		expect(keys).toEqual(['digit:103', 'digit:1040']);
		expect(request.groups).toEqual([
			{
				kind: 'create',
				number: '103',
				name: 'Banque A',
				parentId: idOf('10'),
				accounts: ['1030', '1031', '1032'].map((n) => ({ id: idOf(n), version: 3 })),
			},
			{ kind: 'reuse', accountId: idOf('1040'), accounts: ['10400', '10401'].map((n) => ({ id: idOf(n), version: 3 })) },
		]);
		const sum = proposalSummary(checked);
		expect(sum).toMatchObject({ created: 1, reused: 1, moved: 5, pending: 0, acceptedWithErrors: 0, canConfirm: true });
	});

	it('une acceptée qui porte une erreur bloque Confirmer et sort de la demande', () => {
		const edits = { ...edit(a103, { state: 'accepted', name: '' }), ...edit(a105, { state: 'accepted' }) };
		const checked = run(s, 'digit', edits).checked;
		const sum = proposalSummary(checked);
		expect(sum.acceptedWithErrors).toBe(1);
		expect(sum.canConfirm).toBe(false);
		expect(appliedProposals(checked).map((p) => p.key)).toEqual(['digit:105']);
	});

	it('racine : parentId null dans la demande', () => {
		const checked = run(s, 'digit', edit(a103, { state: 'accepted', parent: 'root' })).checked;
		const g = buildGroupsRequest(checked).request.groups[0];
		expect(g).toMatchObject({ kind: 'create', parentId: null });
	});

	it('un changement de mode garde les choix des propositions de même clé', () => {
		const edits = edit(a103, { state: 'rejected', name: 'Banque A modifiée' });
		const one = run(s, 'one', edits).checked;
		expect(one.map((p) => p.key)).toEqual(['one']);
		const back = run(s, 'digit', edits).checked;
		expect(byKey(back, 'digit:103').edit).toMatchObject({ state: 'rejected', name: 'Banque A modifiée' });
	});

	it('au-delà de 100 propositions retenues : tooMany, Confirmer inactif', () => {
		const rows: Row[] = [];
		for (let i = 0; i < 101; i++) {
			const p = String(100 + i);
			rows.push([`${p}0`, `Compte ${p} A`, 'Asset', null, true], [`${p}1`, `Compte ${p} B`, 'Asset', null, true]);
		}
		const plan = makePlan(rows);
		const selection = new Set(plan.map((a) => a.id));
		const first = run(selection, 'digit', {}, plan).checked;
		const edits = first.reduce<ProposalEdits>((acc, p) => applyEdit(acc, p, { state: 'accepted' }), {});
		const sum = proposalSummary(run(selection, 'digit', edits, plan).checked);
		expect(sum.tooMany).toBe(true);
		expect(sum.canConfirm).toBe(false);
	});

	it('l’aperçu de l’arbre montre le nouveau groupe, les déplacés et le groupe réutilisé', () => {
		const edits = { ...edit(a103, { state: 'accepted' }), ...edit(r1040, { reuse: true, state: 'accepted' }) };
		const checked = run(s, 'digit', edits).checked;
		const rows = previewTree(accounts, checked);
		const row = (n: string) => rows.find((r) => r.number === n)!;
		expect(row('103')).toMatchObject({ mark: 'new', depth: 2, isGroup: true });
		expect(row('1030')).toMatchObject({ mark: 'moved', depth: 3 });
		expect(row('1040')).toMatchObject({ mark: 'becomesGroup', isGroup: true });
		expect(row('10400')).toMatchObject({ mark: 'moved', depth: 3 });
		expect(rows.some((r) => r.number === '1090')).toBe(false); // archivé
		const touched = previewTree(accounts, checked, true).map((r) => r.number);
		expect(touched).toContain('103');
		expect(touched).toContain('1');
		expect(touched).not.toContain('6000');
	});

	it('une proposition écartée n’entre pas dans l’aperçu', () => {
		const checked = run(s, 'digit', edit(a103, { state: 'rejected' })).checked;
		expect(previewTree(accounts, checked).some((r) => r.number === '103')).toBe(false);
	});

	it('les parents offerts : actifs, sans écriture, hors des membres', () => {
		const options = parentOptionsFor(accounts, a103).map((a) => a.number);
		expect(options).toContain('10');
		expect(options).not.toContain('1000'); // mouvementé
		expect(options).not.toContain('1030'); // membre
		expect(options).not.toContain('1090'); // archivé
	});
});

describe('choix gardés au recalcul (revue P1 de la 46-5b, M1 et M3)', () => {
	// Un plan à deux parents : 1000 et 1010 sous 10, 1100 sous 11 ; 1032 d'une autre banque.
	const ROWS: Row[] = [
		['1', 'Actifs', 'Asset', null, false],
		['10', 'Liquidités', 'Asset', '1', false],
		['11', 'Créances', 'Asset', '1', false],
		['1000', 'Caisse', 'Asset', '10', true],
		['1010', 'Poste', 'Asset', '10', true],
		['1030', 'Banque A CHF', 'Asset', '10', true],
		['1031', 'Banque A EUR', 'Asset', '10', true, 'EUR'],
		['1032', 'Banque B CHF', 'Asset', '10', true],
		['1050', 'Compte libre', 'Asset', '10', false],
		['1100', 'Débiteurs', 'Asset', '11', true],
	];
	const plan = makePlan(ROWS);
	const id = (n: string) => (plan.find((a) => a.number === n) as AccountResponse).id;
	const pick = (...numbers: string[]) => new Set(numbers.map(id));
	const one = (selection: Set<number>, edits: ProposalEdits) => byKey(run(selection, 'one', edits, plan).checked, 'one');
	const digit = (selection: Set<number>, edits: ProposalEdits) =>
		byKey(run(selection, 'digit', edits, plan).checked, 'digit:103');

	it('un seul groupe : acceptée pour 1000 et 1010, elle repasse en attente quand 1100 s’ajoute, numéro et parent recalculés', () => {
		const first = one(pick('1000', '1010'), {});
		expect(first.edit).toMatchObject({ number: '100', parent: id('10') });
		const edits = edit(first, { state: 'accepted' });
		expect(one(pick('1000', '1010'), edits).edit.state).toBe('accepted');
		const wider = one(pick('1000', '1010', '1100'), edits);
		expect(wider.edit.state).toBe('pending');
		expect(wider.edit.number).toBe('1'); // plus long préfixe commun recalculé, pas « 100 » figé
		expect(wider.edit.parent).toBeNull(); // plus de parent commun
		expect(buildGroupsRequest(run(pick('1000', '1010', '1100'), 'one', edits, plan).checked).request.groups).toHaveLength(0);
	});

	it('un champ modifié par l’utilisateur reste ; les autres suivent la sélection', () => {
		const first = one(pick('1000', '1010'), {});
		const edits = edit(first, { number: '150' });
		const wider = one(pick('1000', '1010', '1100'), edits);
		expect(wider.edit.number).toBe('150');
		expect(wider.edit.parent).toBeNull();
	});

	it('dernier chiffre : 103 acceptée « Banque A », 1032 s’ajoute → en attente, nom recalculé ; acceptée de nouveau, puis 1032 retiré → en attente', () => {
		const first = digit(pick('1030', '1031'), {});
		expect(first.edit.name).toBe('Banque A');
		let edits = edit(first, { state: 'accepted' });
		const wider = digit(pick('1030', '1031', '1032'), edits);
		expect(wider.edit).toMatchObject({ state: 'pending', name: 'Banque' });
		edits = edit(wider, { state: 'accepted' }, edits);
		expect(digit(pick('1030', '1031', '1032'), edits).edit.state).toBe('accepted');
		expect(digit(pick('1030', '1031'), edits).edit.state).toBe('pending');
	});

	it('une proposition écartée le reste quand ses membres changent', () => {
		const edits = edit(digit(pick('1030', '1031'), {}), { state: 'rejected' });
		expect(digit(pick('1030', '1031', '1032'), edits).edit.state).toBe('rejected');
	});

	it('un parent choisi devenu membre de la proposition est remis à vide : erreur parentMissing, rien n’est envoyé', () => {
		const first = one(pick('1030', '1031'), {});
		const edits = edit(first, { parent: id('1050'), state: 'accepted' }, {});
		expect(one(pick('1030', '1031'), edits).edit.parent).toBe(id('1050'));
		const withParent = one(pick('1030', '1031', '1050'), edits);
		expect(withParent.edit.parent).toBeNull();
		expect(codes(withParent, 'error')).toContain('parentMissing');
		const accepted = edit(withParent, { state: 'accepted' }, edits);
		const checked = run(pick('1030', '1031', '1050'), 'one', accepted, plan).checked;
		expect(byKey(checked, 'one').hasError).toBe(true);
		expect(buildGroupsRequest(checked).request.groups).toHaveLength(0);
	});
});

describe('readGroupRefusal', () => {
	it('lit groupStep, ou groupIndex/accountId d’ACCOUNT_GROUP_INVALID, sinon null', () => {
		expect(readGroupRefusal({ groupStep: { groupIndex: 1, accountId: 7 } })).toEqual({ groupIndex: 1, accountId: 7 });
		expect(readGroupRefusal({ groupIndex: 0, reason: 'mixedTypes' })).toEqual({ groupIndex: 0, accountId: null });
		expect(readGroupRefusal({ groupStep: { groupIndex: '1' } })).toBeNull();
		expect(readGroupRefusal(undefined)).toBeNull();
		expect(readGroupRefusal({})).toBeNull();
	});
});

describe('comptes désignés par un compte bancaire (Epic 46, garde du mainteneur du 2026-11-26)', () => {
	/** Un compte bancaire minimal : seuls les champs que lit `bankLinkedAccounts`. */
	const banque = (b: Partial<BankAccountSummary> & { id: number }): BankAccountSummary =>
		({
			bankName: `Banque ${b.id}`,
			description: null,
			journalAccountId: null,
			archived: false,
			paypal: null,
			wise: null,
			stripe: null,
			...b,
		}) as BankAccountSummary;

	it('compte du plan et compte de frais, premier compte bancaire par identifiant, archivés ignorés', () => {
		const links = bankLinkedAccounts([
			banque({ id: 3, bankName: 'Wise', wise: { account: 'w', feeAccountId: 50 } }),
			banque({ id: 1, bankName: 'Banque A', description: 'Courant', journalAccountId: 10 }),
			banque({ id: 2, bankName: 'PayPal', paypal: { account: 'p', feeAccountId: 50 }, journalAccountId: 11 }),
			banque({ id: 4, bankName: 'Ancienne', journalAccountId: 12, archived: true }),
		]);
		expect(links.get(10)).toBe('Banque A — Courant');
		expect(links.get(11)).toBe('PayPal');
		expect(links.get(50)).toBe('PayPal'); // frais partagés : le premier par identifiant
		expect(links.has(12)).toBe(false);
	});

	it('groupe à réutiliser désigné → refus, la coche « réutiliser » ne s’offre pas', () => {
		const set = proposalSkeletons(accounts, sel('10400', '10401'), 'digit');
		const links = new Map([[idOf('1040'), 'Banque A']]);
		const p = byKey(checkProposals(set, {}, accounts, sel('10400', '10401'), 'digit', links), 'digit:1040');
		expect(codes(p, 'error')).toEqual(['numberBankLinked']);
		expect(p.canReuse).toBe(false);
		expect(p.messages[0].params.bank).toBe('Banque A');
	});

	it('groupe à rattacher désigné → refus', () => {
		const set = proposalSkeletons(accounts, sel('1081'), 'digit');
		const links = new Map([[idOf('108'), 'Banque A']]);
		const p = byKey(checkProposals(set, {}, accounts, sel('1081'), 'digit', links), 'digit:108');
		expect(p.kind).toBe('attach');
		expect(codes(p, 'error')).toContain('numberBankLinked');
	});

	it('parent désigné → refus, et absent des parents offerts ; un membre désigné reste accepté', () => {
		const selection = sel('1030', '1031');
		const set = proposalSkeletons(accounts, selection, 'digit');
		const first = byKey(checkProposals(set, {}, accounts, selection, 'digit'), 'digit:103');
		const links = new Map([
			[idOf('1040'), 'Banque B'],
			[idOf('1030'), 'Banque A'],
		]);
		const choisi = checkProposals(set, edit(first, { parent: idOf('1040') }), accounts, selection, 'digit', links);
		const p = byKey(choisi, 'digit:103');
		expect(codes(p, 'error')).toContain('parentBankLinked');
		// 1030, membre désigné, ne lève rien d'autre.
		const sans = byKey(checkProposals(set, {}, accounts, selection, 'digit', links), 'digit:103');
		expect(sans.hasError).toBe(false);
		const options = parentOptionsFor(accounts, sans, links).map((a) => a.number);
		expect(options).not.toContain('1040');
		expect(parentOptionsFor(accounts, sans).map((a) => a.number)).toContain('1040');
	});
});
