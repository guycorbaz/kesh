/**
 * Les textes du dé-rapprochement — Story 25-3-b (#418).
 *
 * ⚠️ Chaque test nomme la MUTATION qu'il attrape.
 */
import { describe, it, expect, vi } from 'vitest';

vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_k: string, fallback: string) => fallback,
}));

import {
	isReconciliationCancelCode,
	reconciliationCancelErrorMessage,
	reconciliationCancelMessage,
} from './reconciliation-cancel';
import type { ReconciliationCancelCode } from './reconciliation.types';

describe('reconciliationCancelMessage', () => {
	it('un texte par motif, qui parle de RAPPROCHEMENT et jamais de « ce règlement » (mutation : famille du règlement réemployée)', () => {
		const cas: [ReconciliationCancelCode, string][] = [
			['BANK_TRANSACTION_NOT_RECONCILED', "n'est pas rapprochée"],
			['INVOICE_CREDITED', 'paiement à lettrer'],
			['FISCAL_YEAR_CLOSED', "rouvrir l'exercice"],
			['MATCHED_BANK_TRANSACTION', 'autre transaction bancaire'],
			['ACCOUNT_ARCHIVED', 'réactivez-le'],
			['FISCAL_YEAR_INVALID', 'Aucun exercice ouvert'],
		];
		for (const [code, extrait] of cas) {
			const texte = reconciliationCancelMessage(code, null);
			expect(texte).toContain(extrait);
			expect(texte).not.toContain('Ce règlement');
		}
	});

	it('le compte archivé est NOMMÉ (mutation : `label` ignoré)', () => {
		expect(reconciliationCancelMessage('ACCOUNT_ARCHIVED', '1020')).toContain('(1020)');
	});
});

describe('reconciliationCancelErrorMessage', () => {
	it('un code hors des six motifs ⇒ le message du SERVEUR (mutation : texte générique)', () => {
		expect(
			reconciliationCancelErrorMessage({
				code: 'PERIOD_LOCKED',
				message: 'La période du 25.09.2026 est verrouillée.',
				status: 400,
			}),
		).toBe('La période du 25.09.2026 est verrouillée.');
		expect(isReconciliationCancelCode('PERIOD_LOCKED')).toBe(false);
	});

	it('un motif ⇒ son texte ; le 400 des comptes archivés nomme les comptes de `details.rejected` (mutation : `details` ignoré)', () => {
		expect(
			reconciliationCancelErrorMessage({
				code: 'FISCAL_YEAR_CLOSED',
				message: 'texte serveur',
				status: 409,
			}),
		).toContain("rouvrir l'exercice");
		expect(
			reconciliationCancelErrorMessage({
				code: 'ACCOUNT_ARCHIVED',
				message: 'texte serveur',
				status: 400,
				details: { rejected: [{ accountId: 3, accountNumber: '3200' }] },
			}),
		).toContain('(3200)');
	});
});
