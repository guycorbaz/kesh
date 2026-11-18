// Story 43-3 (#823, D2 bis, D3) — les grandeurs du règlement groupé, en entiers de 1/10 000.
//
// Le tableau des six cas de `kesh-gate-logs/e43/decision-difference.md` fait foi : il est
// recopié ici EN DUR (une table calculée par le module serait verte par construction).

import { describe, it, expect } from 'vitest';
import type { AccountResponse } from '$lib/features/accounts/accounts.types';
import {
	NO_CHOICE,
	candidateNumber,
	decompose,
	displayDue,
	effectiveDifferenceAccount,
	formatSignedUnits,
	formatUnits,
	gapOf,
	gapSense,
	parseUnits,
	prefillAccount,
	selectAccount,
	submitBlock,
	submitBlocks,
	type SubmitInput,
} from './group-settlement';

function account(id: number, over: Partial<AccountResponse> = {}): AccountResponse {
	return {
		id,
		companyId: 1,
		number: String(id),
		name: `Compte ${id}`,
		accountType: 'Expense',
		parentId: null,
		active: true,
		role: null,
		postable: true,
		letterable: false,
		version: 1,
		createdAt: '2026-01-01T00:00:00',
		updatedAt: '2026-01-01T00:00:00',
		...over,
	};
}

/** Banque 1020 (identifiant 1020), arrondi 6960, frais bancaires 6940, archivé 6941, groupe 6900. */
const ACCOUNTS = [
	account(1020, { accountType: 'Asset' }),
	account(6960),
	account(6940),
	account(6941, { active: false }),
	account(6900, { postable: false }),
];
const BANK = 1020;

describe('parseUnits / formatUnits', () => {
	it('lit deux et quatre décimales, signe compris', () => {
		expect(parseUnits('1250.00')).toBe(12_500_000n);
		expect(parseUnits('-1250.00')).toBe(-12_500_000n);
		expect(parseUnits('10.0050')).toBe(100_050n);
		expect(parseUnits('0.5')).toBe(5_000n);
		expect(parseUnits('7')).toBe(70_000n);
	});

	it('refuse plus de quatre décimales et toute autre forme', () => {
		expect(parseUnits('1.23456')).toBeNull();
		expect(parseUnits('1,25')).toBeNull();
		expect(parseUnits("1'250.00")).toBeNull();
		expect(parseUnits('')).toBeNull();
		expect(parseUnits(null)).toBeNull();
		expect(parseUnits('abc')).toBeNull();
	});

	// La mutation « Number au lieu des entiers » fait rougir ce test : 0.1 + 0.2 = 0.30000000000000004.
	it('0.10 + 0.20 vaut exactement 0.30', () => {
		const a = parseUnits('0.10');
		const b = parseUnits('0.20');
		expect(a !== null && b !== null && a + b === parseUnits('0.30')).toBe(true);
		const dec = decompose('0.30', ['0.10', '0.20']);
		expect(dec?.difference).toBe(0n);
		expect(dec?.gap).toBe(0n);
		expect(dec?.rounding).toBe(0n);
	});

	it('met en forme au centime et au brut', () => {
		expect(formatUnits(12_500_000n)).toBe('1250.00');
		expect(formatUnits(-4_000n)).toBe('-0.40');
		expect(formatUnits(100_050n, 4)).toBe('10.0050');
		expect(formatUnits(-50n, 4)).toBe('-0.0050');
		expect(formatSignedUnits(50n)).toBe('+0.0050');
		expect(formatSignedUnits(-49n)).toBe('-0.0049');
		expect(formatSignedUnits(0n)).toBe('0.0000');
	});

	it('affiche un reste dû au brut s’il porte une fraction de centime, au centime sinon', () => {
		expect(displayDue('500.0000')).toBe('500.00');
		expect(displayDue('10.0050')).toBe('10.0050');
		expect(displayDue('300.00')).toBe('300.00');
	});
});

describe('décomposition de l’écart — le tableau de decision-difference.md (C-43-25)', () => {
	// [cas, montant, restes bruts, e attendu (texte au centime), r attendu (texte brut), compte demandé]
	const SIX: [string, string, string[], string, string, boolean][] = [
		['2 × 10.0050 payés 20.01', '20.01', ['10.0050', '10.0050'], '0.00', '0.0000', false],
		['2 × 10.0050 payés 20.02', '20.02', ['10.0050', '10.0050'], '0.01', '0.0000', true],
		['500.00 + 300.00 payés 799.99', '799.99', ['500.0000', '300.0000'], '-0.01', '0.0000', true],
		['3 × 10.0050 payés 30.06', '30.06', ['10.0050', '10.0050', '10.0050'], '0.05', '-0.0050', true],
		['10.0049 + 10.0000 payés 20.00', '20.00', ['10.0049', '10.0000'], '0.00', '-0.0049', false],
		['10.0050 + 10.0000 payés 20.00', '20.00', ['10.0050', '10.0000'], '-0.01', '+0.0050', true],
	];

	it.each(SIX)('%s → e, r et compte demandé', (_cas, montant, restes, e, r, compte) => {
		const dec = decompose(montant, restes);
		expect(dec).not.toBeNull();
		if (!dec) return;
		expect(formatUnits(dec.gap)).toBe(e);
		expect(formatSignedUnits(dec.rounding)).toBe(r);
		expect(dec.difference).toBe(dec.gap + dec.rounding);
		// Compte demandé : dès que e ≠ 0, et seulement alors (plafond large, compte d'arrondi désigné).
		const block = submitBlock({
			count: restes.length,
			decomposition: dec,
			maxDifference: '5.00',
			roundingDesignated: true,
			differenceAccountId: null,
		});
		expect(block === 'account-required').toBe(compte);
		if (!compte) expect(block).toBeNull();
	});

	it('un débit se calcule sur la valeur absolue du montant', () => {
		const dec = decompose('-799.60', ['500.00', '300.00']);
		expect(dec && formatUnits(dec.gap)).toBe('-0.40');
	});

	it('demi-centime arrondi en s’éloignant de zéro, des deux côtés', () => {
		expect(gapOf(50n)).toBe(100n);
		expect(gapOf(-50n)).toBe(-100n);
		expect(gapOf(49n)).toBe(0n);
		expect(gapOf(-49n)).toBe(0n);
		expect(gapOf(150n)).toBe(200n);
	});

	it('un montant illisible ne se décompose pas', () => {
		expect(decompose('abc', ['1.00'])).toBeNull();
		expect(decompose('1.00', ['1.00001'])).toBeNull();
	});
});

describe('Enregistrer — ce qui le retient (D3)', () => {
	function input(over: Partial<SubmitInput>): SubmitInput {
		return {
			count: 2,
			decomposition: decompose('799.60', ['500.00', '300.00']),
			maxDifference: '5.00',
			roundingDesignated: true,
			differenceAccountId: 6940,
			...over,
		};
	}

	it('une seule facture → « au moins deux »', () => {
		expect(submitBlock(input({ count: 1 }))).toBe('too-few');
		expect(submitBlock(input({ count: 0 }))).toBe('too-few');
	});

	// Revue P1 (A M-1, E-M1) : une cause n'en masque aucune autre ; sous deux factures, le compte
	// n'est pas demandé.
	it('toutes les causes : moins de deux factures n’efface pas le plafond, ni l’arrondi', () => {
		const grand = decompose('293.60', []);
		expect(submitBlocks(input({ count: 0, decomposition: grand }))).toEqual(['too-few', 'gap-too-large']);
		expect(submitBlocks(input({ count: 1, differenceAccountId: null }))).toEqual(['too-few']);
		const arrondiEtGrand = decompose('26.00', ['10.0049', '10.0000']);
		expect(
			submitBlocks(input({ decomposition: arrondiEtGrand, roundingDesignated: false })),
		).toEqual(['rounding-not-configured', 'gap-too-large']);
		expect(submitBlocks(input({}))).toEqual([]);
	});

	it('écart dans le plafond, compte choisi → actif', () => {
		expect(submitBlock(input({}))).toBeNull();
	});

	it('écart sans compte → compte demandé', () => {
		expect(submitBlock(input({ differenceAccountId: null }))).toBe('account-required');
	});

	it('écart au-delà du plafond → trop grand, compte ou non', () => {
		expect(submitBlock(input({ maxDifference: '0.39' }))).toBe('gap-too-large');
		expect(submitBlock(input({ maxDifference: '0.40' }))).toBeNull();
	});

	it('maxDifference lu de la réponse : un réglage à 1.00 borne un écart de 1.01', () => {
		const dec = decompose('798.99', ['500.00', '300.00']);
		expect(submitBlock(input({ decomposition: dec, maxDifference: '1.00' }))).toBe('gap-too-large');
		const ok = decompose('799.00', ['500.00', '300.00']);
		expect(submitBlock(input({ decomposition: ok, maxDifference: '1.00' }))).toBeNull();
	});

	it('arrondi automatique avec un plafond réglé à 0.00 → actif (P5, L1)', () => {
		const dec = decompose('20.00', ['10.0049', '10.0000']);
		expect(
			submitBlock(input({ decomposition: dec, maxDifference: '0.00', differenceAccountId: null })),
		).toBeNull();
	});

	it('r ≠ 0 sans compte d’arrondi désigné → retenu, même si e = 0 (P5, L2)', () => {
		const dec = decompose('20.00', ['10.0049', '10.0000']);
		expect(submitBlock(input({ decomposition: dec, roundingDesignated: false }))).toBe(
			'rounding-not-configured',
		);
		// Réglages illisibles : le serveur tranche (C-e433-2).
		expect(submitBlock(input({ decomposition: dec, roundingDesignated: null }))).toBeNull();
		// r = 0 : le compte d'arrondi ne joue pas.
		expect(submitBlock(input({ roundingDesignated: false }))).toBeNull();
	});

	it('plafond illisible, deux factures → illisible tant qu’un écart est à borner', () => {
		expect(submitBlock(input({ maxDifference: null }))).toBe('unreadable');
		const exact = decompose('800.00', ['500.00', '300.00']);
		expect(submitBlock(input({ decomposition: exact, maxDifference: null }))).toBeNull();
	});

	// Revue P2 (M-1) : sous deux factures, un plafond non lu (chargement) n'est pas un montant
	// illisible ; seule la cause « deux factures » est dite.
	it('plafond pas encore lu, moins de deux factures → seulement « deux factures »', () => {
		expect(submitBlocks(input({ count: 0, maxDifference: null }))).toEqual(['too-few']);
		expect(submitBlocks(input({ count: 1, maxDifference: null }))).toEqual(['too-few']);
	});

	// Revue P2 (L-1) : le serveur a demandé un compte alors que l'écart calculé est nul.
	it('écart nul, le serveur demande un compte → compte demandé tant qu’aucun n’est choisi', () => {
		const exact = decompose('800.00', ['500.00', '300.00']);
		const base = { decomposition: exact, differenceAccountId: null };
		expect(submitBlocks(input(base))).toEqual([]);
		expect(submitBlocks(input({ ...base, serverWantsAccount: true }))).toEqual(['account-required']);
		expect(submitBlocks(input({ ...base, serverWantsAccount: true, differenceAccountId: 6940 }))).toEqual([]);
	});
});

describe('sens, libellé et préremplissage (C-43-26)', () => {
	const designated = { roundingAccountId: 6960, bankFeesAccountId: 6940 };

	it.each([
		['client', -100n, 'client-under', 6940],
		['client', 100n, 'client-over', 6960],
		['supplier', 100n, 'supplier-over', 6940],
		['supplier', -100n, 'supplier-under', 6960],
	] as const)('%s, e = %s → %s, prérempli %s', (family, gap, sense, prefill) => {
		const s = gapSense(family, gap);
		expect(s).toBe(sense);
		expect(prefillAccount(s, designated, ACCOUNTS, BANK)).toBe(prefill);
	});

	it('e = 0 → aucun sens, aucun préremplissage', () => {
		expect(gapSense('client', 0n)).toBeNull();
		expect(prefillAccount(null, designated, ACCOUNTS, BANK)).toBeNull();
	});

	it('compte désigné archivé, non imputable, absent ou banque → champ vide (#1000)', () => {
		expect(prefillAccount('client-under', { roundingAccountId: null, bankFeesAccountId: 6941 }, ACCOUNTS, BANK)).toBeNull();
		expect(prefillAccount('client-under', { roundingAccountId: null, bankFeesAccountId: 6900 }, ACCOUNTS, BANK)).toBeNull();
		expect(prefillAccount('client-under', { roundingAccountId: null, bankFeesAccountId: 9999 }, ACCOUNTS, BANK)).toBeNull();
		expect(prefillAccount('client-under', { roundingAccountId: null, bankFeesAccountId: 1020 }, ACCOUNTS, BANK)).toBeNull();
		expect(prefillAccount('client-under', { roundingAccountId: null, bankFeesAccountId: null }, ACCOUNTS, BANK)).toBeNull();
		expect(prefillAccount('client-under', null, ACCOUNTS, BANK)).toBeNull();
	});
});

describe('choix gardé (D2 bis, C-e433-3)', () => {
	it('sans choix, le préremplissage suit le signe', () => {
		expect(effectiveDifferenceAccount(NO_CHOICE, 6940, ACCOUNTS, BANK)).toBe(6940);
		expect(effectiveDifferenceAccount(NO_CHOICE, 6960, ACCOUNTS, BANK)).toBe(6960);
	});

	it('une sélection manuelle reste quand le signe change (le préremplissage change, pas le choix)', () => {
		const choice = selectAccount(6960);
		expect(choice.byUser).toBe(true);
		expect(effectiveDifferenceAccount(choice, 6940, ACCOUNTS, BANK)).toBe(6960);
		expect(effectiveDifferenceAccount(choice, 6960, ACCOUNTS, BANK)).toBe(6960);
		expect(effectiveDifferenceAccount(choice, null, ACCOUNTS, BANK)).toBe(6960);
	});

	it('choisir le compte prérempli pose aussi le drapeau', () => {
		expect(selectAccount(6940)).toEqual({ byUser: true, accountId: 6940 });
	});

	// Revue P1 (C-e433-13) : le « × » d'un champ prérempli a un effet réel.
	it('vider le champ le laisse vide, même si un préremplissage existe', () => {
		const choice = selectAccount(null);
		expect(choice).toEqual({ byUser: true, accountId: null });
		expect(effectiveDifferenceAccount(choice, 6940, ACCOUNTS, BANK)).toBeNull();
	});

	it('un choix sorti de la liste (banque) retombe à vide', () => {
		expect(effectiveDifferenceAccount(selectAccount(1020), 6940, ACCOUNTS, BANK)).toBeNull();
	});
});

describe('candidateNumber', () => {
	it('numéro client, numéro interne, numéro du fournisseur, à défaut #id', () => {
		expect(candidateNumber({ kind: 'invoice', id: 3, number: 'F-1' })).toBe('F-1');
		expect(candidateNumber({ kind: 'invoice', id: 3, number: null })).toBe('#3');
		expect(
			candidateNumber({ kind: 'supplier_invoice', id: 4, internalNumber: 'FF-1', supplierInvoiceNumber: 'X' }),
		).toBe('FF-1');
		expect(
			candidateNumber({ kind: 'supplier_invoice', id: 4, internalNumber: null, supplierInvoiceNumber: 'X' }),
		).toBe('X');
	});
});
