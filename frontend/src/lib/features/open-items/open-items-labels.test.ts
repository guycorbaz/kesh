// Story 15-1c-i (#518) — les textes de l'écran des postes ouverts.
//
// `i18nMsg` est remplacé par un double qui rend le repli interpolé ET retient la
// clé demandée : un motif lu sous une autre clé que celle du serveur rougit.
import { describe, it, expect, vi, beforeEach } from 'vitest';

const calls: string[] = [];
vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (key: string, fallback: string, args?: Record<string, string | number>) => {
		calls.push(key);
		return args ? fallback.replace(/\{\s*\$(\w+)\s*\}/g, (_, n) => String(args[n] ?? '')) : fallback;
	},
}));

import {
	amountWithSideLabel,
	dissolutionBlockerLabel,
	documentLabel,
	documentStateLabel,
	entryRefLabel,
	letterBlockerLabel,
	noCheckboxLabel,
	originLabel,
	reasonLabel,
} from './open-items-labels';
import Big from 'big.js';
import { item } from './open-items.test.fixtures';

beforeEach(() => {
	calls.length = 0;
});

describe('AC3 — les six libellés (test 4)', () => {
	it('les deux motifs à la date', () => {
		expect(reasonLabel('unlettered')).toBe('non lettrée');
		expect(reasonLabel('letteredAfterAsOf')).toBe('lettrée après cette date');
	});

	it('les quatre états de la pièce, le reste dû formaté au centime', () => {
		expect(documentStateLabel('unpaid', '1234.5000')).toBe(
			'facture non réglée — reste dû : 1’234.50',
		);
		expect(documentStateLabel('partiallySettled', '40.0000')).toBe(
			'facture partiellement réglée — reste dû : 40.00',
		);
		expect(documentStateLabel('nothingDue', null)).toContain('rien à faire ici');
		expect(documentStateLabel('paidWithoutSettlementEntry', null)).toContain(
			'sans écriture d\'encaissement',
		);
	});

	it('le numéro d’écriture se lit avec son exercice (C127)', () => {
		expect(entryRefLabel('Exercice 2026', 12)).toBe('Exercice 2026 n° 12');
	});
});

describe('AC4 — pourquoi pas de case, par cause et dans l’ordre (test 5)', () => {
	it('lettrée après la date : le code, avant toute autre cause', () => {
		const i = item({
			reason: 'letteredAfterAsOf',
			letteringCode: 'AB',
			documentState: 'nothingDue',
			manuallyLetterable: false,
		});
		expect(noCheckboxLabel(i)).toBe('déjà lettrée (code AB)');
	});

	it('pièce soldée : le texte même d’AC3', () => {
		const i = item({ documentState: 'nothingDue', manuallyLetterable: false });
		expect(noCheckboxLabel(i)).toBe(documentStateLabel('nothingDue', null));
	});

	it('pièce qui bloque : « son lettrage suit sa pièce » ; une transaction bancaire seule, rien', () => {
		const owned = item({
			manuallyLetterable: false,
			document: { type: 'invoice', id: 1, number: 'F-1', invoiceId: null, invoiceNumber: null },
		});
		expect(noCheckboxLabel(owned)).toBe('son lettrage suit sa pièce et ses règlements');
		const bank = item({
			manuallyLetterable: false,
			document: { type: 'bankTransaction', id: 1, number: null, invoiceId: null, invoiceNumber: null },
		});
		expect(noCheckboxLabel(bank)).toBe('');
	});
});

describe('AC4 — ce qui manque au bouton', () => {
	it('quatre messages, dont l’écart formaté', () => {
		expect(letterBlockerLabel({ kind: 'tooFew' })).toBe('Sélectionnez au moins deux lignes.');
		expect(letterBlockerLabel({ kind: 'tooMany' })).toBe('200 lignes au plus.');
		expect(letterBlockerLabel({ kind: 'unbalanced', difference: new Big('-40') })).toBe(
			"La sélection ne s'équilibre pas : écart -40.00.",
		);
		expect(letterBlockerLabel({ kind: 'allClosed' })).toContain('période close');
	});
});

describe('AC2 — le texte d’une pièce', () => {
	const d = { id: 1, number: null, invoiceId: null, invoiceNumber: null };
	it('numéro, « facture fournisseur » sans numéro, « règlement de », « transaction bancaire »', () => {
		expect(documentLabel({ ...d, type: 'invoice', number: 'F-1' })).toBe('F-1');
		expect(documentLabel({ ...d, type: 'creditNote', number: 'AV-2' })).toBe('AV-2');
		expect(documentLabel({ ...d, type: 'supplierInvoice', number: 'FF-3' })).toBe('FF-3');
		expect(documentLabel({ ...d, type: 'supplierInvoice' })).toBe('facture fournisseur');
		expect(documentLabel({ ...d, type: 'settlement', invoiceNumber: 'F-9' })).toBe(
			'règlement de F-9',
		);
		expect(documentLabel({ ...d, type: 'bankTransaction' })).toBe('transaction bancaire');
	});
});

describe('AC6 — origine et motifs du délettrage (test 9)', () => {
	it('trois origines ; « lettrage de la pièce » + numéro ; jamais « règlement »', () => {
		expect(originLabel('manual', null)).toBe('lettrage manuel');
		expect(originLabel('reversal', null)).toBe('contre-passation');
		expect(originLabel('document', 'F-9')).toBe('lettrage de la pièce F-9');
		expect(originLabel('document', null)).toBe("lettrage d'une pièce");
		for (const o of ['manual', 'reversal', 'document'] as const) {
			expect(originLabel(o, 'F-9')).not.toMatch(/règlement/i);
		}
	});

	it('chaque motif est lu sous la clé `error-lettering-*` du refus du serveur', () => {
		dissolutionBlockerLabel('LETTERING_IS_DOCUMENT');
		dissolutionBlockerLabel('LETTERING_LINE_OWNED_BY_DOCUMENT');
		dissolutionBlockerLabel('LETTERING_ALL_LINES_IN_CLOSED_PERIODS');
		expect(calls).toEqual([
			'error-lettering-is-document',
			'error-lettering-line-owned-by-document',
			'error-lettering-all-lines-in-closed-periods',
		]);
	});
});

describe('AC8 — le pied, en valeur absolue suivie du sens (test 10)', () => {
	it('-500.0000 → « 500.00 créditeur » ; 20.0000 → « 20.00 débiteur » ; 0 → « 0.00 »', () => {
		expect(amountWithSideLabel('-500.0000')).toBe('500.00 créditeur');
		expect(amountWithSideLabel('20.0000')).toBe('20.00 débiteur');
		expect(amountWithSideLabel('0.0000')).toBe('0.00');
	});
});

describe('revue P1, E-7 — une fraction de centime ne s’affiche pas « 0.00 »', () => {
	it('l’écart d’une sélection à 0.0040 se lit aux quatre décimales', () => {
		expect(letterBlockerLabel({ kind: 'unbalanced', difference: new Big('0.004') })).toBe(
			"La sélection ne s'équilibre pas : écart 0.0040.",
		);
		expect(amountWithSideLabel('-0.0040')).toBe('0.0040 créditeur');
		expect(amountWithSideLabel('-500.0000')).toBe('500.00 créditeur');
	});
});
