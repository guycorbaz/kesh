/**
 * `pdfErrorMessage` — Story 25-6-b, revue P2.
 *
 * ⚠️ Le mock d'`i18nMsg` honore un CATALOGUE résolu sans arguments, comme le
 * vrai : une clé connue l'emporte sur le repli. Un mock qui rendrait toujours
 * le repli (`err.message`) cacherait exactement le défaut que ce module ferme.
 */
import { describe, it, expect, vi } from 'vitest';

const CATALOGUE: Record<string, string> = {
	'error-invoice-too-many-lines-for-pdf': 'La facture contient ⁨{ $count }⁩ lignes.',
	'error-invoice-pdf-gone': 'Le fichier ⁨{ $sha256 }⁩.pdf manque.',
	'invoice-pdf-error-invoice-not-pdf-ready': 'Facture pas prête pour le PDF.',
	'invoice-pdf-error-generic': 'Erreur PDF.',
};
vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (k: string, fallback: string) => CATALOGUE[k] ?? fallback,
}));

import { pdfErrorMessage } from './pdf-error';

const err = (code: string, message: string) => ({ code, message, status: 400 });

describe('pdfErrorMessage', () => {
	it('« trop de lignes » : le message du serveur, avec le nombre (mutation : clé du catalogue, `{ $count }` brut)', () => {
		const msg = pdfErrorMessage(err('INVOICE_TOO_MANY_LINES_FOR_PDF', 'La facture contient 12 lignes.'));
		expect(msg).toBe('La facture contient 12 lignes.');
	});

	it('410 : le message du serveur, qui nomme le fichier', () => {
		const msg = pdfErrorMessage(err('INVOICE_PDF_GONE', 'Le fichier abc.pdf manque.'));
		expect(msg).toBe('Le fichier abc.pdf manque.');
	});

	it('un code sans variable : la clé du catalogue', () => {
		expect(pdfErrorMessage(err('INVOICE_NOT_PDF_READY', 'serveur'))).toBe(
			'Facture pas prête pour le PDF.',
		);
	});

	it('un code inconnu : le générique', () => {
		expect(pdfErrorMessage(err('SOMETHING_NEW', 'serveur'))).toBe('Erreur PDF.');
	});

	it('une erreur qui n’est pas une erreur d’API : le générique', () => {
		expect(pdfErrorMessage(new Error('réseau'))).toBe('Erreur PDF.');
	});
});
