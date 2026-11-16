// Story 21-6b — tests du mapping errorCode → libellé + classification « e-mail parti ».
import { describe, expect, it, vi } from 'vitest';

// i18nMsg renvoie le fallback tel quel en test (pas de bundle chargé).
vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string, args?: Record<string, unknown>) =>
		args ? `${fallback} ${JSON.stringify(args)}` : fallback,
}));

import { reminderErrorLabel, reminderFailureText } from './reminder-error-label';

describe('reminderErrorLabel', () => {
	it('traduit un code connu', () => {
		expect(reminderErrorLabel('INVOICE_ALREADY_PAID')).toBe('Facture déjà payée');
	});

	it('libelle le refus du reste nul (Story 25-4-b2), pas le fallback brut', () => {
		expect(reminderErrorLabel('REMINDER_NOTHING_DUE')).toBe('Rien à réclamer');
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

// Story 35-2b (#955, AC4) — la ligne d'un lot dit pourquoi le PDF est refusé.
describe('reminderFailureText', () => {
	const PDF = 'INVOICE_NOT_PDF_READY';
	const LIBELLE = 'Facture non imprimable en PDF';
	const PHRASE = 'La localité du client contient « – » (U+2013).';

	it('ajoute le message du serveur au libellé, après « : »', () => {
		expect(reminderFailureText({ errorCode: PDF, details: { message: PHRASE } })).toBe(
			`${LIBELLE} : ${PHRASE}`,
		);
	});

	it('rend le libellé seul quand le message manque, est vide, blanc ou n’est pas une chaîne', () => {
		const sans: Array<Record<string, unknown> | null> = [
			null,
			{},
			{ message: '' },
			{ message: '   ' },
			{ message: 42 },
			{ message: { texte: PHRASE } },
		];
		for (const details of sans) {
			expect(reminderFailureText({ errorCode: PDF, details }), JSON.stringify(details)).toBe(
				LIBELLE,
			);
		}
	});

	it('n’affiche pas le message d’un autre code (mutation M7)', () => {
		expect(
			reminderFailureText({ errorCode: 'DATABASE_ERROR', details: { message: 'pool fermé' } }),
		).toBe('Erreur technique');
	});

	it('garde un message balisé tel quel, en texte', () => {
		expect(reminderFailureText({ errorCode: PDF, details: { message: '<b>gras</b>' } })).toBe(
			`${LIBELLE} : <b>gras</b>`,
		);
	});
});
