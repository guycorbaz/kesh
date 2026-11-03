// Story v017-3b (#687, AC-12) — le miroir de `purchase_line_total` (kesh-core) et le
// refus serveur `SUPPLIER_INVOICE_LINE_NOT_IN_CENTIMES`.
//
// ⚠️ Chaque test nomme la mutation qu'il attrape. i18nMsg rend son repli, variables
// substituées : les textes vérifiés sont ceux du repli français (= catalogue fr-CH).

import { describe, it, expect, vi } from 'vitest';

vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string, args?: Record<string, string | number>) =>
		args
			? fallback.replace(/\{\s*\$(\w+)\s*\}/g, (_, n) => String(args[n] ?? ''))
			: fallback,
}));

import {
	decimalForServer,
	describeSubmitError,
	dueBeforeInvoiceDate,
	emptyLine,
	firstLineNotInCentimes,
	formatLiveTotal,
	formLineFromResponse,
	lineNotInCentimes,
	linesInvalid,
	paymentFields,
	sumTtc,
} from './supplier-invoice-form';
import type { CreateSupplierInvoiceLineRequest } from './supplier-invoices.types';

function line(quantity: string, unitPrice: string): CreateSupplierInvoiceLineRequest {
	return { ...emptyLine(), description: 'x', quantity, unitPrice, expenseAccountId: 40 };
}

describe('lineNotInCentimes — les cas de l’AC-1', () => {
	it.each([
		['1', '12.50'],
		['2', '0.125'],
		['4', '2.5000'],
		['0.1', '0.1'],
		['0.3', '0.1'],
	])('%s × %s est au centime', (q, p) => {
		expect(lineNotInCentimes(line(q, p))).toBeNull();
	});

	it.each([
		['3', '33.3333', '99.9999'],
		['1.5', '3.333', '4.9995'],
		['1', '0.005', '0.005'],
	])('%s × %s n’est pas au centime (montant %s)', (q, p, total) => {
		expect(lineNotInCentimes(line(q, p))).toBe(total);
	});

	it('accepte la virgule décimale, comme AMOUNT_RE', () => {
		expect(lineNotInCentimes(line('3', '33,3333'))).toBe('99.9999');
		expect(lineNotInCentimes(line('1', '12,50'))).toBeNull();
	});

	it('lit une quantité vide comme 1, comme l’envoi', () => {
		expect(lineNotInCentimes(line('', '0.005'))).toBe('0.005');
		expect(lineNotInCentimes(line('', '12.50'))).toBeNull();
	});

	it('ne refuse rien sur une saisie vide ou illisible', () => {
		expect(lineNotInCentimes(line('1', ''))).toBeNull();
		expect(lineNotInCentimes(line('abc', '1.005'))).toBeNull();
	});

	it('affiche le montant sans notation exponentielle', () => {
		expect(lineNotInCentimes(line('0.0001', '0.0001'))).toBe('0.00000001');
	});

	it('firstLineNotInCentimes rend la première ligne fautive (0-based) et son montant', () => {
		expect(firstLineNotInCentimes([line('1', '10'), line('3', '33.3333')])).toEqual({
			index: 1,
			lineTotal: '99.9999',
		});
		expect(firstLineNotInCentimes([line('1', '10')])).toBeNull();
	});
});

// Story v017-4b (#830, AC-24 (c)) — un montant se lit UNE fois (`parseAmount`) et part
// avec un point décimal ; la quantité vide vaut 1 partout ; un prix vide est refusé.
describe('#830 — la virgule décimale et les valeurs vides', () => {
	it.each([
		['12,50', '12.50'],
		['12.50', '12.50'],
		['1,5', '1.5'],
		['100', '100'],
		[' 3,3333 ', '3.3333'],
	])('decimalForServer(%j) → %j', (raw, sent) => {
		expect(decimalForServer(raw)).toBe(sent);
	});

	it('le montant attendu part avec un point (M32)', () => {
		expect(paymentFields({ iban: '', reference: '', expectedAmount: '10,00' }).expectedPaymentAmount).toBe(
			'10.00',
		);
		expect(paymentFields({ iban: '', reference: '', expectedAmount: '  ' }).expectedPaymentAmount).toBeNull();
	});

	it('le total vivant lit la virgule et la quantité vide comme l’envoi (1)', () => {
		expect(sumTtc([line('', '12,50')]).toFixed()).toBe('12.5');
		expect(sumTtc([line('2', '1,25')]).toFixed()).toBe('2.5');
	});

	it('un prix vide est une ligne invalide, jamais lue 0 (M33)', () => {
		expect(linesInvalid([line('1', '')])).toBe(true);
		expect(linesInvalid([line('', '10')])).toBe(false);
		expect(linesInvalid([line('1', '10,00')])).toBe(false);
	});
});

// Story v017-4b (#853, AC-24 (b)) — une échéance antérieure à la date est refusée.
describe('#853 — dueBeforeInvoiceDate', () => {
	it('signale une échéance strictement antérieure, laisse passer égale ou absente', () => {
		expect(dueBeforeInvoiceDate('2026-05-10', '2026-05-09')).toBe(true);
		expect(dueBeforeInvoiceDate('2026-05-10', '2026-05-10')).toBe(false);
		expect(dueBeforeInvoiceDate('2026-05-10', '')).toBe(false);
		expect(dueBeforeInvoiceDate('', '2026-05-09')).toBe(false);
	});

	it('le refus du serveur va sous l’échéance, avec les deux dates', () => {
		const e = describeSubmitError({
			code: 'SUPPLIER_INVOICE_DUE_BEFORE_INVOICE_DATE',
			status: 400,
			message: 'serveur',
			details: { invoiceDate: '2026-05-10', dueDate: '2026-05-09' },
		});
		expect(e.field).toBe('due');
		// Le texte et la clé du serveur (`error-supplier-invoice-due-before-invoice-date`),
		// dates au format suisse.
		expect(e.message).toBe("L'échéance (09.05.2026) précède la date de la facture (10.05.2026).");
	});

	it('un conflit de version invite à recharger (AC-23)', () => {
		const e = describeSubmitError({ code: 'OPTIMISTIC_LOCK_CONFLICT', status: 409, message: 'x' });
		expect(e.field).toBeNull();
		expect(e.message).toMatch(/rechargez la page/);
	});
});

// Story v017-4b (#835, AC-24 (d)) — le total vivant aux décimales exactes.
describe('#835 — formatLiveTotal', () => {
	it('montre les quatre décimales d’un total sous le centime, deux sinon (M26)', () => {
		expect(formatLiveTotal(sumTtc([line('3', '33.3333')]))).toBe('99.9999');
		expect(formatLiveTotal(sumTtc([line('1', '100')]))).toBe('100.00');
	});
});

// Story v017-4b (AC-23) — une ligne lue du serveur, rendue au formulaire.
describe('formLineFromResponse', () => {
	const RATES = [{ rate: '8.10' }, { rate: '2.60' }];
	it('quantité et prix lisibles, taux aligné sur la liste', () => {
		expect(
			formLineFromResponse(
				{
					position: 0,
					description: 'Fourniture',
					quantity: '1.0000',
					unitPrice: '100.2000',
					vatRate: '8.10',
					lineTotal: '100.20',
					expenseAccountId: 40,
				},
				RATES,
			),
		).toEqual({
			description: 'Fourniture',
			quantity: '1',
			unitPrice: '100.20',
			vatRate: '8.10',
			expenseAccountId: 40,
		});
	});

	it('garde les quatre décimales d’un prix, et un taux nul vaut l’option « 0 »', () => {
		const l = formLineFromResponse(
			{
				position: 1,
				description: 'x',
				quantity: '3.0000',
				unitPrice: '33.3333',
				vatRate: '0.00',
				lineTotal: '99.9999',
				expenseAccountId: 41,
			},
			RATES,
		);
		expect(l.quantity).toBe('3');
		expect(l.unitPrice).toBe('33.3333');
		expect(l.vatRate).toBe('0');
	});
});

describe('describeSubmitError — SUPPLIER_INVOICE_LINE_NOT_IN_CENTIMES', () => {
	it('désigne la ligne et dit le texte du refus serveur (même clé qu’errors.rs)', () => {
		const e = describeSubmitError({
			code: 'SUPPLIER_INVOICE_LINE_NOT_IN_CENTIMES',
			status: 400,
			message: 'serveur',
			details: { line: 2, lineTotal: '4.9995' },
		});
		expect(e.field).toBe('line');
		expect(e.line).toBe(2);
		expect(e.message).toBe(
			'La ligne 2 donne un montant de 4.9995, qui n’est pas au centime. Saisissez la ligne au montant facturé par le fournisseur, au centime.',
		);
	});
});
