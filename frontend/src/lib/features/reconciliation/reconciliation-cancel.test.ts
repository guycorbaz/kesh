/**
 * Les textes du dé-rapprochement — Story 25-3-b (#418).
 *
 * ⚠️ Chaque test nomme la MUTATION qu'il attrape.
 */
import { describe, it, expect, vi } from 'vitest';

// Story 43-3, revue P1 (A L-10) : les variables `{ $x }` sont interpolées, pour que le nombre de
// factures rouvertes soit vérifié, non seulement le choix du message.
vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_k: string, fallback: string, args?: Record<string, string | number>) =>
		args ? fallback.replace(/\{ \$(\w+) \}/g, (_m, k) => String(args[k])) : fallback,
}));

import {
	isReconciliationCancelCode,
	reconciliationCancelDoneMessage,
	reconciliationCancelErrorMessage,
	reconciliationCancelMessage,
} from './reconciliation-cancel';
import type { ReconciliationCancelCode } from './reconciliation.types';

// Story 34-11b (#972, AC-4b) — le message de succès du retrait seul.
describe('reconciliationCancelDoneMessage', () => {
	const base = {
		bankTransaction: { id: 42, status: 'pending', matchedEntryId: null },
		invoiceId: null,
		supplierInvoiceId: null,
	};
	it('retrait seul (`reversalJournalEntryId: null`) ⇒ « le lien est retiré », distinct de l’annulation (mutation : un seul message)', () => {
		const seul = reconciliationCancelDoneMessage({ ...base, reversalJournalEntryId: null });
		const annule = reconciliationCancelDoneMessage({ ...base, reversalJournalEntryId: 9 });
		expect(seul).toBe('Le lien est retiré ; l’écriture reste celle de l’autre côté du virement.');
		expect(annule).toBe('Rapprochement annulé.');
	});

	// Story 43-3 (#823, AC5) — un règlement groupé défait dit ses factures rouvertes.
	it('règlement groupé (`cancelledInvoices` non vide) ⇒ le message du groupe (mutation : liste ignorée)', () => {
		const groupe = reconciliationCancelDoneMessage({
			...base,
			reversalJournalEntryId: 9,
			kind: 'grouped_settlement',
			cancelledInvoices: [
				{ kind: 'invoice', id: 12, number: 'F-12' },
				{ kind: 'invoice', id: 15, number: 'F-15' },
			],
		});
		expect(groupe).toBe('Rapprochement annulé : 2 factures redeviennent à régler.');
		const vide = reconciliationCancelDoneMessage({ ...base, reversalJournalEntryId: 9, cancelledInvoices: [] });
		expect(vide).toBe('Rapprochement annulé.');
	});

	// Revue P1 (B-L12) : un groupe fournisseur a son propre message.
	it('règlement groupé fournisseur ⇒ « factures fournisseur redeviennent validées, à payer »', () => {
		const groupe = reconciliationCancelDoneMessage({
			...base,
			reversalJournalEntryId: 9,
			kind: 'grouped_settlement',
			cancelledInvoices: [
				{ kind: 'supplier_invoice', id: 7, number: 'FF-7' },
				{ kind: 'supplier_invoice', id: 8, number: 'FF-8' },
				{ kind: 'supplier_invoice', id: 9, number: null },
			],
		});
		expect(groupe).toBe('Rapprochement annulé : 3 factures fournisseur redeviennent validées, à payer.');
	});
});

describe('reconciliationCancelMessage', () => {
	it('un texte par motif, qui parle de RAPPROCHEMENT et jamais de « ce règlement » (mutation : famille du règlement réemployée)', () => {
		const cas: [ReconciliationCancelCode, string][] = [
			['BANK_TRANSACTION_NOT_RECONCILED', "n'est pas rapprochée"],
			['INVOICE_CREDITED', 'reste ouvert au compte débiteurs'],
			['INVOICE_WRITTEN_OFF', "annulez d'abord le solde"],
			['FISCAL_YEAR_CLOSED', "en commençant par le plus récent"],
			// Story 15-1a2-0 — extrait PROPRE au texte neuf : « en commençant par le
			// plus récent » est aussi celui de FISCAL_YEAR_CLOSED.
			['LETTERING_ALL_LINES_IN_CLOSED_PERIODS', 'toutes les lignes de ce lettrage'],
			['MATCHED_BANK_TRANSACTION', 'autre transaction bancaire'],
			['ACCOUNT_ARCHIVED', 'réactivez-le'],
			// Story 53-6a (#1674) — le neuvième motif.
			['PERIOD_LOCKED', 'période verrouillée'],
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

	it('Story 53-6a — la BORNE du verrou est ajoutée en date suisse (mutation : `label` ignoré)', () => {
		// Revue P1, L1 — la borne est nommée comme telle, jamais lue comme la date de l'écriture.
		const avec = reconciliationCancelMessage('PERIOD_LOCKED', '2026-06-30');
		expect(avec).toContain("La période est verrouillée jusqu'au 30.06.2026 inclus.");
		expect(avec).not.toContain('(30.06.2026)');
		expect(reconciliationCancelMessage('PERIOD_LOCKED', null)).not.toContain('jusqu');
	});
});

describe('reconciliationCancelErrorMessage', () => {
	it('un code hors des neuf motifs ⇒ le message du SERVEUR (mutation : texte générique)', () => {
		// Story 53-6a : `PERIOD_LOCKED` est devenu un motif ; `LATER_FISCAL_YEAR_CLOSED` reprend
		// l'exemple du code hors motifs (refus du socle, état hérité).
		expect(
			reconciliationCancelErrorMessage({
				code: 'LATER_FISCAL_YEAR_CLOSED',
				message: 'Un exercice postérieur est clôturé.',
				status: 400,
			}),
		).toBe('Un exercice postérieur est clôturé.');
		expect(isReconciliationCancelCode('LATER_FISCAL_YEAR_CLOSED')).toBe(false);
	});

	it('Story 53-6a — le 400 PERIOD_LOCKED du socle ⇒ le motif, la borne lue dans `details.lockedThrough` ; le 409 groupé, sans `details` ⇒ le motif sans borne (mutation : `details` ignoré, message du serveur)', () => {
		expect(isReconciliationCancelCode('PERIOD_LOCKED')).toBe(true);
		const auClic = reconciliationCancelErrorMessage({
			code: 'PERIOD_LOCKED',
			message: 'texte serveur',
			status: 400,
			details: { lockedThrough: '2026-06-30', attempted: '2026-06-15' },
		});
		expect(auClic).toContain('période verrouillée');
		expect(auClic).toContain("verrouillée jusqu'au 30.06.2026 inclus");
		expect(auClic).not.toContain('texte serveur');
		const groupe = reconciliationCancelErrorMessage({
			code: 'PERIOD_LOCKED',
			message: 'texte serveur',
			status: 409,
		});
		expect(groupe).toContain('période verrouillée');
		expect(groupe).not.toContain('(');
	});

	it('un motif ⇒ son texte ; le 400 des comptes archivés nomme les comptes de `details.rejected` (mutation : `details` ignoré)', () => {
		expect(
			reconciliationCancelErrorMessage({
				code: 'FISCAL_YEAR_CLOSED',
				message: 'texte serveur',
				status: 409,
			}),
		).toContain("en commençant par le plus récent");
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

// Story 15-1a2-0 (#518) — le lettrage figé par la période : reconnu, traduit
// dans la famille du dé-rapprochement.
describe('le motif LETTERING_ALL_LINES_IN_CLOSED_PERIODS', () => {
	it('est reconnu et traduit dans SA famille (mutation : code absent de MOTIFS, ou texte du règlement)', () => {
		expect(isReconciliationCancelCode('LETTERING_ALL_LINES_IN_CLOSED_PERIODS')).toBe(true);
		const texte = reconciliationCancelErrorMessage({
			code: 'LETTERING_ALL_LINES_IN_CLOSED_PERIODS',
			message: 'texte serveur',
			status: 409,
		});
		expect(texte).toContain('Ce rapprochement est lettré');
		expect(texte).not.toContain('Ce règlement');
	});
});

// Story 25-4-d2a (#384) — le motif « un solde existe » est un motif du
// dé-rapprochement : traduit, et reconnu comme tel.
describe('le motif INVOICE_WRITTEN_OFF', () => {
	it('est reconnu et traduit — pas affiché brut (mutation : code absent de MOTIFS)', () => {
		expect(isReconciliationCancelCode('INVOICE_WRITTEN_OFF')).toBe(true);
		expect(
			reconciliationCancelErrorMessage({
				code: 'INVOICE_WRITTEN_OFF',
				message: 'texte serveur',
				status: 409,
			}),
		).toContain("annulez d'abord le solde");
	});
});
