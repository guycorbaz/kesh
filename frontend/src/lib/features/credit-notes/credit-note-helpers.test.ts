import { afterEach, describe, expect, it, vi } from 'vitest';

const getMock = vi.fn();
vi.mock('$lib/shared/utils/api-client', () => ({ apiClient: { get: (u: string) => getMock(u) } }));

import { loadI18nMessages } from '$lib/shared/utils/i18n.svelte';
import { creditNoteStatusLabel, formatCreditNoteTotal } from './credit-note-helpers';

// Le dictionnaire est un état de module partagé : le vider après chaque cas.
afterEach(async () => {
	getMock.mockResolvedValue({ locale: 'fr-CH', messages: {} });
	await loadI18nMessages();
});

describe('formatCreditNoteTotal', () => {
	it('formats a swiss amount with thousands separator', () => {
		expect(formatCreditNoteTotal('1000.0000')).toBe('1’000.00');
	});

	it('returns empty string for null/undefined/empty', () => {
		expect(formatCreditNoteTotal(null)).toBe('');
		expect(formatCreditNoteTotal(undefined)).toBe('');
		expect(formatCreditNoteTotal('')).toBe('');
	});

	it('returns empty string for invalid input', () => {
		expect(formatCreditNoteTotal('not-a-number')).toBe('');
	});
});

describe('creditNoteStatusLabel', () => {
	// ⚠️ Ces assertions portent sur le REPLI, pas sur le français en dur. Dans leur rédaction
	// précédente elles verrouillaient le français : la fonction n'appelait pas `i18nMsg`, et
	// le test restait vert sur un statut qui se serait affiché en français dans les quatre
	// langues dès que la story 23-5 aurait traduit l'en-tête de sa colonne.
	it('mappe les statuts sur leur repli français', () => {
		expect(creditNoteStatusLabel('draft')).toBe('Brouillon');
		expect(creditNoteStatusLabel('issued')).toBe('Émis');
		expect(creditNoteStatusLabel('cancelled')).toBe('Annulé');
	});

	// Borne anti-régression : le défaut d'origine était l'ABSENCE d'appel i18n. Seule une
	// assertion empruntant le chemin réel du dictionnaire peut le voir.
	// ⚠️ L'accord suit la langue : `nota di credito` est féminin, d'où `Emessa`/`Annullata`.
	it('passe réellement par le dictionnaire i18n', async () => {
		getMock.mockResolvedValue({
			locale: 'it-CH',
			messages: {
				'credit-notes-status-draft': 'Bozza',
				'credit-notes-status-issued': 'Emessa',
				'credit-notes-status-cancelled': 'Annullata',
			},
		});
		await loadI18nMessages();
		expect(creditNoteStatusLabel('draft')).toBe('Bozza');
		expect(creditNoteStatusLabel('issued')).toBe('Emessa');
		expect(creditNoteStatusLabel('cancelled')).toBe('Annullata');
	});
});
