// Story 21-6b — tests du mapping errorCode → libellé + classification « e-mail parti ».
import { describe, expect, it, vi } from 'vitest';

// i18nMsg renvoie le fallback tel quel en test (pas de bundle chargé).
vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string, args?: Record<string, unknown>) =>
		args ? `${fallback} ${JSON.stringify(args)}` : fallback,
}));

import { reminderErrorLabel } from './reminder-error-label';

describe('reminderErrorLabel', () => {
	it('traduit un code connu', () => {
		expect(reminderErrorLabel('INVOICE_ALREADY_PAID')).toBe('Facture déjà payée');
	});

	it('retombe sur le fallback avec le code brut pour un code inconnu', () => {
		expect(reminderErrorLabel('SOMETHING_NEW')).toContain('SOMETHING_NEW');
	});

	it('donne un libellé dédié aux codes « e-mail parti » du lot (pas le fallback)', () => {
		// Ces libellés indiquent explicitement que l'e-mail est parti (jamais
		// « Réessayer ») — c'est le texte, pas un drapeau, qui porte l'info.
		for (const code of ['REMINDER_SENT_BUT_INVOICE_GONE', 'RECORD_FAILED_EMAIL_SENT', 'SMTP_SEND_FAILED']) {
			expect(reminderErrorLabel(code)).not.toContain('Échec (');
		}
	});
});
