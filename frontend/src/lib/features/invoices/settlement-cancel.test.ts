/**
 * Les motifs qui empêchent d'annuler un règlement client — Story 25-4-d2a (#384).
 *
 * ⚠️ Chaque test nomme la MUTATION qu'il attrape.
 */
import { describe, it, expect, vi } from 'vitest';

vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_k: string, fallback: string) => fallback,
}));

import { invoiceSettlementCancelMessage } from './settlement-cancel';

describe('invoiceSettlementCancelMessage', () => {
	it('le motif « un solde existe » est traduit, jamais affiché en code brut (mutation : cas absent)', () => {
		const texte = invoiceSettlementCancelMessage('INVOICE_WRITTEN_OFF', null);
		expect(texte).toContain("annulez d'abord le solde");
		expect(texte).not.toContain('INVOICE_WRITTEN_OFF');
	});

	it('la tête existante reste la sienne (mutation : cas confondus)', () => {
		expect(invoiceSettlementCancelMessage('INVOICE_CREDITED', null)).toContain('reste ouvert au compte débiteurs');
	});
});
