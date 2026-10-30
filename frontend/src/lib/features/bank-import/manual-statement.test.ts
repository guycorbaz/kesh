// Story v016-2b (#816) — tests de la logique pure du relevé saisi à la main :
// totaux et écart (AC-21), raison unique d'un « Enregistrer » inactif (AC-21),
// corps envoyé et normalisation des montants (AC-21, AC-22), correspondance
// entre le rang envoyé et la ligne affichée (AC-20), lecture des refus du
// serveur (AC-22).
//
// Chaque bloc nomme la mutation de la fiche qu'il attrape (M13, M14, M25, M26).

import { describe, expect, it } from 'vitest';
import {
	buildManualStatementBody,
	canVerify,
	computeManualTotals,
	describeManualError,
	emptyManualLine,
	isBlankLine,
	lineFaults,
	manualSnapshotKey,
	saveBlockReason,
	screenRowOf,
	type ManualLineInput,
	type ManualStatementInput,
} from './manual-statement';
import { isValidAmount } from '$lib/features/journal-entries/balance';

function line(o: Partial<ManualLineInput> = {}): ManualLineInput {
	return { ...emptyManualLine(), ...o };
}

/** Relevé équilibré de trois lignes : 1000 + 50 − 120.50 − 9.50 = 920. */
function input(o: Partial<ManualStatementInput> = {}): ManualStatementInput {
	return {
		bankAccountId: 3,
		periodFrom: '2026-09-01',
		periodTo: '2026-09-30',
		statementNumber: '',
		openingBalance: '1000.00',
		closingBalance: '920.00',
		lines: [
			line({ bookingDate: '2026-09-03', details: 'Assurance', debit: '120,50' }),
			line({ bookingDate: '2026-09-15', details: 'Virement reçu', credit: '50' }),
			line({ bookingDate: '2026-09-20', details: 'Frais', debit: '9.50' }),
		],
		...o,
	};
}

const OK = { previewSucceeded: true, confirmationsMissing: false };

describe('isValidAmount généralisée (AC-21)', () => {
	const SOLDE = { maxDecimals: 2, maxIntegerDigits: 13, allowNegative: true, allowEmpty: false };
	const MONTANT = { maxDecimals: 2, maxIntegerDigits: 13 };

	it('les défauts du journal sont intacts', () => {
		expect(isValidAmount('')).toBe(true);
		expect(isValidAmount('10.1234')).toBe(true);
		expect(isValidAmount('-1')).toBe(false);
		expect(isValidAmount('123456789012345')).toBe(true);
	});

	it('un solde accepte le signe, refuse le vide, quatorze chiffres et trois décimales', () => {
		expect(isValidAmount('-120,50', SOLDE)).toBe(true);
		expect(isValidAmount('1234567890123', SOLDE)).toBe(true);
		expect(isValidAmount('12345678901234', SOLDE)).toBe(false);
		expect(isValidAmount('', SOLDE)).toBe(false);
		expect(isValidAmount('1.234', SOLDE)).toBe(false);
	});

	it('un montant de ligne refuse le signe', () => {
		expect(isValidAmount('-1', MONTANT)).toBe(false);
		expect(isValidAmount('', MONTANT)).toBe(true);
	});
});

describe('computeManualTotals (AC-21, M13)', () => {
	it('solde calculé = départ + crédits − débits, écart = arrivée − calculé', () => {
		const t = computeManualTotals(input());
		expect(t.totalDebits.toFixed(2)).toBe('130.00');
		expect(t.totalCredits.toFixed(2)).toBe('50.00');
		expect(t.computedClosing?.toFixed(2)).toBe('920.00');
		expect(t.enteredClosing?.toFixed(2)).toBe('920.00');
		expect(t.gap?.toFixed(2)).toBe('0.00');
	});

	it('un relevé déséquilibré rend l’écart signé', () => {
		const t = computeManualTotals(input({ closingBalance: '900' }));
		expect(t.computedClosing?.toFixed(2)).toBe('920.00');
		expect(t.gap?.toFixed(2)).toBe('-20.00');
	});

	it('un solde vide n’est pas zéro : aucun écart tant que les deux soldes ne sont pas saisis', () => {
		const sansDepart = computeManualTotals(input({ openingBalance: '' }));
		expect(sansDepart.computedClosing).toBeNull();
		expect(sansDepart.gap).toBeNull();
		const sansArrivee = computeManualTotals(input({ closingBalance: '  ' }));
		expect(sansArrivee.enteredClosing).toBeNull();
		expect(sansArrivee.gap).toBeNull();
	});

	it('pas de parseFloat : 0.1 + 0.2 au centime près', () => {
		const t = computeManualTotals(
			input({
				openingBalance: '0',
				closingBalance: '0.30',
				lines: [line({ credit: '0.10' }), line({ credit: '0.20' })],
			}),
		);
		expect(t.gap?.toFixed(2)).toBe('0.00');
		expect(t.gap?.eq(0)).toBe(true);
	});
});

describe('lignes : vides et fautives (AC-20, AC-21)', () => {
	it('une ligne faite de blancs est vide', () => {
		expect(isBlankLine(line({ details: '   ', reference: ' ' }))).toBe(true);
		expect(isBlankLine(line({ debit: '1' }))).toBe(false);
	});

	it('« 0 » est refusé comme montant de ligne', () => {
		expect(lineFaults(line({ bookingDate: '2026-09-03', details: 'x', debit: '0' }), input())).toContain('debit');
	});

	it('zéro ou deux montants, date hors période, libellé vide ou trop long', () => {
		const p = input();
		expect(lineFaults(line({ bookingDate: '2026-09-03', details: 'x' }), p)).toEqual(['debit', 'credit']);
		expect(lineFaults(line({ bookingDate: '2026-09-03', details: 'x', debit: '1', credit: '2' }), p)).toEqual([
			'debit',
			'credit',
		]);
		expect(lineFaults(line({ bookingDate: '2026-10-01', details: 'x', debit: '1' }), p)).toEqual(['bookingDate']);
		expect(lineFaults(line({ bookingDate: '2026-09-03', details: ' ', debit: '1' }), p)).toEqual(['details']);
		expect(lineFaults(line({ bookingDate: '2026-09-03', details: 'é'.repeat(501), debit: '1' }), p)).toEqual([
			'details',
		]);
		expect(lineFaults(line({ bookingDate: '2026-09-03', details: 'é'.repeat(500), debit: '1' }), p)).toEqual([]);
		expect(
			lineFaults(line({ bookingDate: '2026-09-03', details: 'x', reference: 'r'.repeat(256), debit: '1' }), p),
		).toEqual(['reference']);
		expect(
			lineFaults(line({ bookingDate: '2026-09-03', details: 'x', counterpartyName: 'c'.repeat(256), debit: '1' }), p),
		).toEqual(['counterpartyName']);
	});

	it('période illisible : « hors période » n’est pas évalué ; la date de valeur est libre', () => {
		const p = input({ periodTo: '' });
		expect(lineFaults(line({ bookingDate: '2027-01-01', details: 'x', debit: '1' }), p)).toEqual([]);
		expect(
			lineFaults(line({ bookingDate: '2026-09-03', valueDate: '2030-01-01', details: 'x', debit: '1' }), input()),
		).toEqual([]);
	});
});

describe('saveBlockReason — une seule raison, la première de la séquence (AC-21, M14)', () => {
	it('un relevé complet, vérifié et confirmé n’est pas bloqué', () => {
		expect(saveBlockReason(input(), OK)).toBeNull();
	});

	it('M14 — un aperçu réussi sur un formulaire déséquilibré rend la raison « écart »', () => {
		expect(saveBlockReason(input({ closingBalance: '900' }), OK)).toEqual({ kind: 'gap' });
	});

	it('la séquence : compte, période, numéro, soldes, lignes, ligne fautive, écart, aperçu, cases', () => {
		expect(saveBlockReason(input({ bankAccountId: null, periodFrom: '' }), OK)).toEqual({ kind: 'account' });
		expect(saveBlockReason(input({ periodFrom: '' }), OK)).toEqual({ kind: 'period' });
		expect(saveBlockReason(input({ periodFrom: '2026-13-01' }), OK)).toEqual({ kind: 'period' });
		expect(saveBlockReason(input({ periodFrom: '2026-10-01', openingBalance: '' }), OK)).toEqual({
			kind: 'period-inverted',
		});
		expect(saveBlockReason(input({ statementNumber: 'n'.repeat(256), openingBalance: '' }), OK)).toEqual({
			kind: 'statement-number',
		});
		expect(saveBlockReason(input({ openingBalance: '' }), OK)).toEqual({ kind: 'balances' });
		expect(saveBlockReason(input({ closingBalance: '12345678901234' }), OK)).toEqual({ kind: 'balances' });
		const trop = Array.from({ length: 501 }, () => line({ bookingDate: '2026-09-03', details: 'x', debit: '1' }));
		expect(saveBlockReason(input({ lines: trop }), OK)).toEqual({ kind: 'line-count' });
		const fautive = input({
			lines: [line(), line({ bookingDate: '2026-09-03', details: 'x', debit: '0' })],
			closingBalance: '1',
		});
		expect(saveBlockReason(fautive, OK)).toEqual({ kind: 'line', line: 2 });
		expect(saveBlockReason(input(), { previewSucceeded: false, confirmationsMissing: false })).toEqual({
			kind: 'preview',
		});
		expect(saveBlockReason(input(), { previewSucceeded: true, confirmationsMissing: true })).toEqual({
			kind: 'confirmations',
		});
	});

	it('500 lignes pleines sont admises ; les lignes vides ne comptent pas', () => {
		const cinqCents = Array.from({ length: 500 }, () => line({ bookingDate: '2026-09-03', details: 'x', debit: '1' }));
		const r = saveBlockReason(
			input({ lines: [...cinqCents, line()], openingBalance: '500', closingBalance: '0' }),
			OK,
		);
		expect(r).toBeNull();
	});
});

describe('canVerify (AC-21)', () => {
	it('actif dès qu’un compte et une période lisible sont saisis, même déséquilibré ou fautif', () => {
		expect(canVerify(input({ closingBalance: '1', lines: [line({ debit: 'abc' })] }))).toBe(true);
		expect(canVerify(input({ bankAccountId: null }))).toBe(false);
		expect(canVerify(input({ periodTo: '' }))).toBe(false);
		expect(canVerify(input({ periodFrom: '2026-10-01' }))).toBe(true);
	});
});

describe('buildManualStatementBody (AC-20, AC-21, AC-22, M26)', () => {
	it('M26 — la virgule est normalisée en point, deux décimales : « -120,50 » part en « -120.50 »', () => {
		const { body } = buildManualStatementBody(input({ openingBalance: '-120,50', closingBalance: '-200' }), {
			confirmDuplicateFile: false,
			confirmDuplicateLines: false,
			confirmOpeningBalanceGap: false,
		});
		expect(body.openingBalance).toBe('-120.50');
		expect(body.closingBalance).toBe('-200.00');
		expect(body.lines[0]).toEqual({
			bookingDate: '2026-09-03',
			valueDate: null,
			details: 'Assurance',
			counterpartyName: null,
			reference: null,
			amount: '120.50',
			direction: 'debit',
		});
		expect(body.lines[1].amount).toBe('50.00');
		expect(body.lines[1].direction).toBe('credit');
	});

	it('les lignes vides sont retirées, la table des rangs garde la ligne affichée', () => {
		const { body, rowMap } = buildManualStatementBody(
			input({
				lines: [
					line({ bookingDate: '2026-09-03', details: 'a', debit: '1' }),
					line({ details: '  ' }),
					line({ bookingDate: '2026-09-04', details: 'b', credit: '1' }),
				],
			}),
			{ confirmDuplicateFile: false, confirmDuplicateLines: false, confirmOpeningBalanceGap: false },
		);
		expect(body.lines.map((l) => l.details)).toEqual(['a', 'b']);
		expect(rowMap).toEqual([0, 2]);
	});

	it('les cases cochées partent dans le corps ; textes trimés, facultatifs vides → null', () => {
		const { body } = buildManualStatementBody(
			input({
				statementNumber: '  9/2026 ',
				lines: [line({ bookingDate: '2026-09-03', details: ' x ', reference: ' ', counterpartyName: ' SA ', debit: '1' })],
			}),
			{ confirmDuplicateFile: true, confirmDuplicateLines: true, confirmOpeningBalanceGap: true },
		);
		expect(body.statementNumber).toBe('9/2026');
		expect(body.lines[0].details).toBe('x');
		expect(body.lines[0].reference).toBeNull();
		expect(body.lines[0].counterpartyName).toBe('SA');
		expect(body.confirmDuplicateFile).toBe(true);
		expect(body.confirmDuplicateLines).toBe(true);
		expect(body.confirmOpeningBalanceGap).toBe(true);
		expect(body.bankAccountId).toBe(3);
	});

	it('une saisie hors forme part telle quelle, pour que le serveur la désigne', () => {
		const { body } = buildManualStatementBody(
			input({ openingBalance: 'abc', lines: [line({ bookingDate: '2026-09-03', details: 'x', debit: '1.234' })] }),
			{ confirmDuplicateFile: false, confirmDuplicateLines: false, confirmOpeningBalanceGap: false },
		);
		expect(body.openingBalance).toBe('abc');
		expect(body.lines[0].amount).toBe('1.234');
	});

	it('l’instantané ignore les cases : les cocher ne périme pas l’aperçu', () => {
		const a = buildManualStatementBody(input(), {
			confirmDuplicateFile: false,
			confirmDuplicateLines: false,
			confirmOpeningBalanceGap: false,
		});
		const b = buildManualStatementBody(input(), {
			confirmDuplicateFile: true,
			confirmDuplicateLines: true,
			confirmOpeningBalanceGap: true,
		});
		expect(manualSnapshotKey(a.body)).toBe(manualSnapshotKey(b.body));
		const c = buildManualStatementBody(input({ statementNumber: 'x' }), {
			confirmDuplicateFile: false,
			confirmDuplicateLines: false,
			confirmOpeningBalanceGap: false,
		});
		expect(manualSnapshotKey(c.body)).not.toBe(manualSnapshotKey(a.body));
	});
});

describe('rangs du serveur → lignes de l’écran (AC-20, M25)', () => {
	// Une ligne vide au milieu : la troisième ligne affichée part au rang 2.
	const rowMap = [0, 2, 3];

	it('M25 — le rang 1-based envoyé désigne la ligne affichée, non le rang tel quel', () => {
		expect(screenRowOf(rowMap, 2)).toBe(2);
		expect(screenRowOf(rowMap, 3)).toBe(3);
		expect(screenRowOf(rowMap, 1)).toBe(0);
	});

	it('un rang absent ou hors table ne désigne rien', () => {
		expect(screenRowOf(rowMap, null)).toBeNull();
		expect(screenRowOf(rowMap, 0)).toBeNull();
		expect(screenRowOf(rowMap, 4)).toBeNull();
	});

	it('une faute sur la ligne qui suit une ligne vide désigne CETTE ligne', () => {
		const d = describeManualError(
			{
				status: 400,
				code: 'BANK_STATEMENT_INVALID',
				message: 'Le relevé saisi contient une valeur absente ou invalide.',
				details: { line: 2, field: 'details', reason: 'required' },
			},
			rowMap,
		);
		expect(d).toEqual({ kind: 'cell', row: 2, field: 'details', reason: 'required' });
	});

	it('un doublon sur la ligne qui suit une ligne vide désigne CETTE ligne', () => {
		const d = describeManualError(
			{
				status: 422,
				code: 'BANK_STATEMENT_DUPLICATE_LINES',
				message: 'Des lignes de ce relevé existent déjà sur ce compte.',
				details: { lines: [{ line: 2, existingTransactionId: 9 }] },
			},
			rowMap,
		);
		expect(d).toEqual({ kind: 'rows', rows: [2], message: 'Des lignes de ce relevé existent déjà sur ce compte.' });
	});
});

describe('describeManualError — les autres refus (AC-22)', () => {
	it('un champ d’entête désigne le champ du formulaire', () => {
		expect(
			describeManualError(
				{
					status: 400,
					code: 'BANK_STATEMENT_INVALID',
					message: 'm',
					details: { line: null, field: 'openingBalance', reason: 'invalid_number' },
				},
				[],
			),
		).toEqual({ kind: 'field', field: 'openingBalance', reason: 'invalid_number' });
	});

	it('un 400 VALIDATION_ERROR sans field : refus générique, aucune ligne, pas le message du serveur', () => {
		expect(
			describeManualError({ status: 400, code: 'VALIDATION_ERROR', message: 'Corps illisible (fr)' }, [0]),
		).toEqual({ kind: 'rejected' });
	});

	it('le relevé identique : clé de l’écran, non le message « fichier » du serveur', () => {
		expect(
			describeManualError(
				{ status: 422, code: 'BANK_IMPORT_DUPLICATE_FILE', message: 'Ce fichier a déjà été importé.' },
				[0],
			),
		).toEqual({ kind: 'duplicate-file' });
	});

	it('les autres refus du serveur gardent leur message traduit', () => {
		expect(
			describeManualError(
				{ status: 422, code: 'BANK_STATEMENT_UNBALANCED', message: 'Déséquilibré (traduit)' },
				[0],
			),
		).toEqual({ kind: 'message', message: 'Déséquilibré (traduit)' });
		expect(describeManualError(new Error('réseau'), [0])).toEqual({ kind: 'unexpected' });
	});
});
