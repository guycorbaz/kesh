// Story 21-6b — tests du mapping errorCode → libellé + classification « e-mail parti ».
import { describe, expect, it, vi } from 'vitest';

// i18nMsg renvoie le fallback tel quel en test (pas de bundle chargé).
vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string, args?: Record<string, unknown>) =>
		args ? `${fallback} ${JSON.stringify(args)}` : fallback,
}));

import { reminderErrorLabel, isEmailSent, EMAIL_SENT_CODES } from './reminder-error-label';

describe('reminderErrorLabel', () => {
	it('traduit un code connu', () => {
		expect(reminderErrorLabel('INVOICE_ALREADY_PAID')).toBe('Facture déjà payée');
	});

	it('retombe sur le fallback avec le code brut pour un code inconnu', () => {
		expect(reminderErrorLabel('SOMETHING_NEW')).toContain('SOMETHING_NEW');
	});

	it('couvre tous les codes « e-mail parti »', () => {
		for (const code of EMAIL_SENT_CODES) {
			// Un libellé dédié existe (pas le fallback générique).
			expect(reminderErrorLabel(code)).not.toContain('Échec (');
		}
	});
});

describe('isEmailSent', () => {
	it('identifie les codes où l’e-mail est parti', () => {
		expect(isEmailSent('REMINDER_SENT_BUT_INVOICE_GONE')).toBe(true);
		expect(isEmailSent('RECORD_FAILED_EMAIL_SENT')).toBe(true);
		expect(isEmailSent('SMTP_SEND_FAILED')).toBe(true);
	});

	it('classe les autres codes comme non-envoyés (ré-essayables)', () => {
		expect(isEmailSent('INVOICE_ALREADY_PAID')).toBe(false);
		expect(isEmailSent('CONTACT_EMAIL_MISSING')).toBe(false);
		expect(isEmailSent('RATE_LIMITED')).toBe(false);
	});
});
