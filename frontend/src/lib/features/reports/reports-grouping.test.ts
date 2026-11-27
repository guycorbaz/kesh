// Story 46-2 — état, transitions et mémoire de la vue groupée (AC-2, AC-3, AC-5, AC-10).

import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import {
	closeAll,
	defaultGroupingView,
	detailLevelOptions,
	groupingFromParams,
	groupingParamsOf,
	groupingStorageKey,
	openAll,
	readGroupingView,
	sameGroupingParams,
	sanitizeGroupingView,
	toggleGroup,
	withDetailLevel,
	withEcho,
	writeGroupingView,
	type GroupingView,
} from './reports-grouping';

const vue = (partiel: Partial<GroupingView> = {}): GroupingView => ({
	...defaultGroupingView(),
	...partiel,
});

describe('vue par défaut (AC-1)', () => {
	it('hiérarchie montrée, tous les comptes, sous-totaux sur le titre, aucune liste', () => {
		expect(defaultGroupingView()).toEqual({
			shown: true,
			detailLevel: 'all',
			subtotalPosition: 'header',
			open: [],
			closed: [],
		});
	});
});

describe('paramètres de requête', () => {
	it('hiérarchie masquée : aucun paramètre (écran à plat, C-46-2-2)', () => {
		expect(groupingParamsOf(vue({ shown: false, detailLevel: 2, open: [4] }))).toBeUndefined();
	});
	it('hiérarchie montrée : niveau, position et listes', () => {
		expect(
			groupingParamsOf(vue({ detailLevel: 2, subtotalPosition: 'footer', open: [4], closed: [9] })),
		).toEqual({ detailLevel: 2, subtotalPosition: 'footer', open: [4], closed: [9] });
	});
});

describe('paramètres égaux (revue P1 M2)', () => {
	const p = (partiel = {}) => ({ ...groupingParamsOf(vue())!, ...partiel });
	it('tous deux absents : égaux ; un seul absent : différents', () => {
		expect(sameGroupingParams(undefined, undefined)).toBe(true);
		expect(sameGroupingParams(undefined, p())).toBe(false);
		expect(sameGroupingParams(p(), undefined)).toBe(false);
	});
	it("mêmes listes dans un autre ordre : égaux", () => {
		expect(sameGroupingParams(p({ open: [4, 9], closed: [2] }), p({ open: [9, 4], closed: [2] }))).toBe(true);
	});
	it('niveau, position, ou une liste qui diffère : différents', () => {
		expect(sameGroupingParams(p(), p({ detailLevel: 1 }))).toBe(false);
		expect(sameGroupingParams(p(), p({ subtotalPosition: 'footer' }))).toBe(false);
		expect(sameGroupingParams(p({ open: [4] }), p({ open: [5] }))).toBe(false);
		expect(sameGroupingParams(p({ closed: [4] }), p({ closed: [4, 5] }))).toBe(false);
		expect(sameGroupingParams(p({ closed: [4, 5] }), p({ closed: [4] }))).toBe(false);
	});
});

describe('règle des clics (AC-3)', () => {
	it('ouvrir un groupe fermé par le niveau : ajouté à open, retiré de closed', () => {
		expect(toggleGroup(vue({ detailLevel: 2, closed: [7, 8] }), 7, false, 2)).toMatchObject({
			open: [7],
			closed: [8],
		});
	});
	it('fermer un groupe ouvert par le niveau : ajouté à closed, retiré de open', () => {
		expect(toggleGroup(vue({ open: [7, 3] }), 7, true, 1)).toMatchObject({ open: [3], closed: [7] });
	});
	// Story 46-7 (revue d'intégration de l'Epic 46, L3) : un clic qui rend au
	// groupe l'état du niveau vide sa place dans les listes, sans quoi le PDF dit
	// « détail ajusté à la main » alors que l'écran montre l'état du niveau.
	it('fermer puis rouvrir un groupe ouvert par le niveau : listes vides', () => {
		const fermee = toggleGroup(vue({ detailLevel: 2 }), 7, true, 1);
		expect(fermee).toMatchObject({ open: [], closed: [7] });
		expect(toggleGroup(fermee, 7, false, 1)).toMatchObject({ open: [], closed: [] });
	});
	it('ouvrir puis refermer un groupe fermé par le niveau : listes vides', () => {
		const ouverte = toggleGroup(vue({ detailLevel: 2 }), 9, false, 2);
		expect(ouverte).toMatchObject({ open: [9], closed: [] });
		expect(toggleGroup(ouverte, 9, true, 2)).toMatchObject({ open: [], closed: [] });
	});
	it('« Tous les comptes » : rouvrir un groupe fermé à la main le retire de closed', () => {
		expect(toggleGroup(vue({ closed: [7, 8] }), 7, false, 5)).toMatchObject({
			open: [],
			closed: [8],
		});
	});
	it('une entrée redondante (open d\'un groupe déjà ouvert par le niveau) se ferme quand même', () => {
		expect(toggleGroup(vue({ detailLevel: 3, open: [4] }), 4, true, 1)).toMatchObject({
			open: [],
			closed: [4],
		});
	});
	it('changer de niveau vide les deux listes', () => {
		expect(withDetailLevel(vue({ open: [1], closed: [2] }), 3)).toMatchObject({
			detailLevel: 3,
			open: [],
			closed: [],
		});
	});
	it('tout ouvrir = tous les comptes, tout fermer = niveau 1, listes vidées (C-46-2-3)', () => {
		expect(openAll(vue({ detailLevel: 1, open: [1] }))).toMatchObject({ detailLevel: 'all', open: [] });
		expect(closeAll(vue({ closed: [2] }))).toMatchObject({ detailLevel: 1, closed: [] });
	});
	it("l'écho de l'API remplace les listes (groupe disparu retiré, AC-10)", () => {
		const v = withEcho(vue({ detailLevel: 2, open: [5, 999] }), {
			detailLevel: 2,
			subtotalPosition: 'header',
			maxDepth: 3,
			open: [5],
			closed: [],
		});
		expect(v.open).toEqual([5]);
		expect(v.detailLevel).toBe(2);
	});
});

describe('niveaux proposés (AC-2)', () => {
	it('1 à maxDepth − 1 ; « Tous les comptes » est ajouté par le sélecteur', () => {
		expect(detailLevelOptions(4, 'all')).toEqual([1, 2, 3]);
	});
	it('plan sans groupe ou rapport vide : aucun niveau numéroté', () => {
		expect(detailLevelOptions(1, 'all')).toEqual([]);
		expect(detailLevelOptions(0, 'all')).toEqual([]);
	});
	it('borné à 20', () => {
		expect(detailLevelOptions(40, 'all')).toHaveLength(20);
	});
	it('un niveau mémorisé plus profond que le plan reste proposé', () => {
		expect(detailLevelOptions(3, 5)).toEqual([1, 2, 5]);
	});
});

describe('relecture de la mémoire (AC-5)', () => {
	it('champ invalide ignoré, les autres gardés', () => {
		expect(
			sanitizeGroupingView({ shown: false, detailLevel: 21, subtotalPosition: 'footer', open: [3] }),
		).toEqual({ shown: false, detailLevel: 'all', subtotalPosition: 'footer', open: [3], closed: [] });
	});
	it('niveau hors bornes ou non entier ignoré', () => {
		for (const n of [0, -1, 2.5, '2', 'ALL', null]) {
			expect(sanitizeGroupingView({ detailLevel: n }).detailLevel).toBe('all');
		}
		expect(sanitizeGroupingView({ detailLevel: 20 }).detailLevel).toBe(20);
	});
	it('liste non numérique, ou de plus de 500 identifiants, ignorée', () => {
		expect(sanitizeGroupingView({ open: ['a'] }).open).toEqual([]);
		expect(sanitizeGroupingView({ open: [1.5] }).open).toEqual([]);
		const longue = Array.from({ length: 501 }, (_, i) => i + 1);
		expect(sanitizeGroupingView({ closed: longue }).closed).toEqual([]);
		expect(sanitizeGroupingView({ closed: longue.slice(0, 500) }).closed).toHaveLength(500);
	});
	it('un identifiant dans les deux listes : les deux ignorées (400 sinon)', () => {
		const v = sanitizeGroupingView({ open: [1, 2], closed: [2] });
		expect(v.open).toEqual([]);
		expect(v.closed).toEqual([]);
	});
	it('ni objet ni valeur : vue par défaut', () => {
		expect(sanitizeGroupingView(null)).toEqual(defaultGroupingView());
		expect(sanitizeGroupingView('x')).toEqual(defaultGroupingView());
	});
});

describe('mémoire du navigateur (AC-10)', () => {
	beforeEach(() => localStorage.clear());
	afterEach(() => vi.restoreAllMocks());

	it('clé distincte par société et par rapport', () => {
		expect(groupingStorageKey(3, 'balance-sheet')).toBe('kesh-report-grouping:3:balance-sheet');
		writeGroupingView(3, 'balance-sheet', vue({ detailLevel: 2 }));
		writeGroupingView(3, 'income-statement', vue({ detailLevel: 1 }));
		writeGroupingView(4, 'balance-sheet', vue({ shown: false }));
		expect(readGroupingView(3, 'balance-sheet').detailLevel).toBe(2);
		expect(readGroupingView(3, 'income-statement').detailLevel).toBe(1);
		expect(readGroupingView(4, 'balance-sheet').shown).toBe(false);
		expect(readGroupingView(5, 'balance-sheet')).toEqual(defaultGroupingView());
	});
	it('aller-retour complet : position, hiérarchie et listes', () => {
		const v = vue({ shown: false, detailLevel: 3, subtotalPosition: 'footer', open: [8], closed: [9] });
		writeGroupingView(1, 'balance-sheet', v);
		expect(readGroupingView(1, 'balance-sheet')).toEqual(v);
	});
	it('stockage vide ou illisible : vue par défaut', () => {
		expect(readGroupingView(1, 'balance-sheet')).toEqual(defaultGroupingView());
		localStorage.setItem(groupingStorageKey(1, 'balance-sheet'), '{pas du json');
		expect(readGroupingView(1, 'balance-sheet')).toEqual(defaultGroupingView());
	});
	it('stockage refusé : vue par défaut en lecture, écriture sans erreur', () => {
		vi.spyOn(Storage.prototype, 'getItem').mockImplementation(() => {
			throw new DOMException('refusé', 'SecurityError');
		});
		vi.spyOn(Storage.prototype, 'setItem').mockImplementation(() => {
			throw new DOMException('refusé', 'SecurityError');
		});
		let lue: GroupingView | undefined;
		expect(() => (lue = readGroupingView(1, 'balance-sheet'))).not.toThrow();
		expect(lue).toEqual(defaultGroupingView());
		expect(() => writeGroupingView(1, 'balance-sheet', vue())).not.toThrow();
	});
	it('société inconnue : ni lecture ni écriture', () => {
		const set = vi.spyOn(Storage.prototype, 'setItem');
		writeGroupingView(null, 'balance-sheet', vue({ detailLevel: 2 }));
		expect(set).not.toHaveBeenCalled();
		expect(readGroupingView(null, 'balance-sheet')).toEqual(defaultGroupingView());
	});
});

describe("règle d'ouverture par le niveau, table partagée avec le Rust (46-7, LOW-2)", () => {
	// Le même fichier est lu par `groupes::tests::est_ouvert_suit_la_table_partagee_avec_le_frontend` :
	// `openByLevel` n'est pas exporté, on l'éprouve par `toggleGroup` (listes vides au départ).
	const table = JSON.parse(
		readFileSync(
			resolve(process.cwd(), '../crates/kesh-report/tests/fixtures/ouverture_par_niveau.json'),
			'utf8',
		),
	) as { cas: { niveau: number | 'all'; profondeur: number; ouvert: boolean }[] };

	it('la table est lue en entier', () => {
		expect(table.cas.length).toBeGreaterThan(10);
	});

	for (const c of table.cas) {
		it(`niveau ${c.niveau}, profondeur ${c.profondeur} : ${c.ouvert ? 'ouvert' : 'fermé'}`, () => {
			const v = vue({ detailLevel: c.niveau });
			// Cliquer vers l'état que le niveau donne déjà ne s'inscrit dans aucune liste ;
			// cliquer vers l'autre l'inscrit dans l'une des deux.
			const verslEtatDuNiveau = toggleGroup(v, 5, !c.ouvert, c.profondeur);
			expect(verslEtatDuNiveau).toMatchObject({ open: [], closed: [] });
			const contre = toggleGroup(v, 5, c.ouvert, c.profondeur);
			expect(c.ouvert ? contre.closed : contre.open).toEqual([5]);
		});
	}
});

// Story 55-1b (D4, AC-8) — la présentation lue dans une adresse, par les règles de ce module.
describe('groupingFromParams (Story 55-1b)', () => {
	const lire = (s: string) => groupingFromParams(new URLSearchParams(s));

	it('une présentation complète est lue, les listes par virgule brute ou encodée', () => {
		expect(lire('view=grouped&level=2&subtotals=footer&open=4,3&closed=7')).toEqual({
			view: { shown: true, detailLevel: 2, subtotalPosition: 'footer', open: [4, 3], closed: [7] },
			anyValid: true,
		});
		expect(lire('open=4%2C3').view.open).toEqual([4, 3]);
		expect(lire('level=all').view.detailLevel).toBe('all');
		expect(lire('view=flat').view.shown).toBe(false);
	});

	it('sans présentation : vue par défaut, anyValid faux', () => {
		expect(lire('tab=balance-sheet&fy=2')).toEqual({ view: defaultGroupingView(), anyValid: false });
	});

	it.each([
		'level=0',
		'level=21',
		'level=03',
		'level=3.0',
		'level=%2B3',
		'level=ALL',
		'open=1,a',
		'open=1,,2',
		'open=99999999999999999999',
		'closed=x',
		'view=xyz',
		'subtotals=bottom',
		'open=',
	])('%s seul : ignoré, anyValid faux', (s) => {
		expect(lire(s)).toEqual({ view: defaultGroupingView(), anyValid: false });
	});

	it('501 identifiants : la liste est ignorée ; 500 sont admis', () => {
		const ids = (n: number) => Array.from({ length: n }, (_, i) => i + 1).join(',');
		expect(lire(`open=${ids(501)}`).anyValid).toBe(false);
		expect(lire(`open=${ids(500)}`).view.open).toHaveLength(500);
	});

	it('open=5&closed=5 seul : les deux listes ignorées, aucun champ retenu (anyValid faux)', () => {
		expect(lire('open=5&closed=5')).toEqual({ view: defaultGroupingView(), anyValid: false });
		// avec un autre champ valide, ce champ seul est retenu
		expect(lire('open=5&closed=5&level=2')).toEqual({ view: vue({ detailLevel: 2 }), anyValid: true });
	});

	it('chaque paramètre est jugé seul : un invalide n’emporte pas les autres', () => {
		expect(lire('level=21&subtotals=footer&open=1,a&closed=3').view).toEqual(
			vue({ subtotalPosition: 'footer', closed: [3] }),
		);
	});

	it('doublons retirés', () => {
		expect(lire('open=3,3,2').view.open).toEqual([3, 2]);
	});
});

describe('readIdList — borne Number.isSafeInteger (Story 55-1b)', () => {
	it('un identifiant au-delà des entiers sûrs est refusé, aussi depuis la mémoire', () => {
		expect(sanitizeGroupingView({ open: [1e20] }).open).toEqual([]);
		expect(sanitizeGroupingView({ open: [Number.MAX_SAFE_INTEGER] }).open).toEqual([Number.MAX_SAFE_INTEGER]);
	});
});
