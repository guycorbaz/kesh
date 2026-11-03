// Story v016-2b (#816, AC-24) — couverture de la table des `reason` d'un refus
// `BANK_STATEMENT_INVALID`. ⚠️ Ce test ne fait pas foi seul : il recopie
// l'ensemble fermé de la fiche ; la garde qui lie la table au serveur est le
// test Rust croisé (`crates/kesh-api/tests/textes_coherents.rs`), qui l'égale à
// `ManualStatementReason::ALL` dans les deux sens.
import { describe, expect, it, vi } from 'vitest';

// i18nMsg rend la clé demandée : le test vérifie la clé, pas le repli.
vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (key: string) => `[${key}]`,
}));

import { MANUAL_STATEMENT_REASON_LABELS, manualStatementReasonLabel } from './manual-statement-reason-label';

describe('manualStatementReasonLabel', () => {
	it('rend la clé de chaque `reason` de l’ensemble fermé', () => {
		const attendu: Record<string, string> = {
			required: 'bank-import-manual-reason-required',
			invalid_date: 'bank-import-manual-reason-invalid-date',
			invalid_number: 'bank-import-manual-reason-invalid-number',
			period_inverted: 'bank-import-manual-reason-period-inverted',
			too_many_lines: 'bank-import-manual-reason-too-many-lines',
			out_of_period: 'bank-import-manual-reason-out-of-period',
			too_long: 'bank-import-manual-reason-too-long',
			not_positive: 'bank-import-manual-reason-not-positive',
			too_many_decimals: 'bank-import-manual-reason-too-many-decimals',
			too_many_digits: 'bank-import-manual-reason-too-many-digits',
			unknown_direction: 'bank-import-manual-reason-unknown-direction',
		};
		expect(Object.keys(MANUAL_STATEMENT_REASON_LABELS).sort()).toEqual(Object.keys(attendu).sort());
		for (const [reason, cle] of Object.entries(attendu)) {
			expect(manualStatementReasonLabel(reason)).toBe(`[${cle}]`);
		}
	});

	it('rend le code brut pour un `reason` inconnu', () => {
		expect(manualStatementReasonLabel('something_new')).toBe('something_new');
	});
});
