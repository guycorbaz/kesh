import { describe, it, expect } from 'vitest';
import { toSelectOptions } from './vocabulary-options';

// ⚠️ L'entrée arrive dans l'ordre des CODES, comme la route la rend : une entrée
// déjà rangée par libellé laisserait passer l'absence de tri.
const PAR_CODE = [
	{ code: 'invoice.validated', label: 'Facture validée' },
	{ code: 'journal_entry.created', label: 'Écriture créée' },
];

describe('toSelectOptions', () => {
	it('range par libellé, non par code (mutation : tri par code)', () => {
		const out = toSelectOptions(PAR_CODE, 'fr-CH', undefined);
		expect(out.map((o) => o.label)).toEqual(['Écriture créée', 'Facture validée']);
	});

	it("le paramètre `locale` compte : sv-SE range « Zahlung » avant « Öffnung » (mutation : locale en dur)", () => {
		const items = [
			{ code: 'a', label: 'Öffnung' },
			{ code: 'b', label: 'Zahlung' },
		];
		expect(toSelectOptions(items, 'fr-CH', undefined).map((o) => o.label)).toEqual([
			'Öffnung',
			'Zahlung',
		]);
		expect(toSelectOptions(items, 'sv-SE', undefined).map((o) => o.label)).toEqual([
			'Zahlung',
			'Öffnung',
		]);
	});

	it('un code courant ABSENT du vocabulaire est ajouté, libellé par son code (mutation : non ajouté)', () => {
		const out = toSelectOptions(PAR_CODE, 'fr-CH', 'zz.legacy');
		expect(out).toContainEqual({ value: 'zz.legacy', label: 'zz.legacy' });
		expect(out).toHaveLength(3);
	});

	it("un code courant PRÉSENT n'est pas dupliqué", () => {
		expect(toSelectOptions(PAR_CODE, 'fr-CH', 'invoice.validated')).toHaveLength(2);
	});
});
