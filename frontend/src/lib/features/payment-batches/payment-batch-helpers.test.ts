import { afterEach, describe, expect, it, vi } from 'vitest';

const getMock = vi.fn();
vi.mock('$lib/shared/utils/api-client', () => ({ apiClient: { get: (u: string) => getMock(u) } }));

import { loadI18nMessages } from '$lib/shared/utils/i18n.svelte';
import { failedItemLabel, formatBatchTotal, paymentBatchStatusLabel } from './payment-batch-helpers';

// Le dictionnaire est un état de module partagé : le vider après chaque cas, sinon le test
// qui charge de l'allemand fait échouer les assertions de repli des suivants.
afterEach(async () => {
	getMock.mockResolvedValue({ locale: 'fr-CH', messages: {} });
	await loadI18nMessages();
});

describe('formatBatchTotal', () => {
	it('formate suisse', () => {
		expect(formatBatchTotal('300.0000')).toBe('300.00');
	});
	it("vide → ''", () => {
		expect(formatBatchTotal(null)).toBe('');
		expect(formatBatchTotal('x')).toBe('');
	});
});

describe('paymentBatchStatusLabel', () => {
	// ⚠️ Ces assertions portent sur le REPLI, pas sur le français en dur : hors navigateur le
	// dictionnaire est vide et `i18nMsg` rend son 2e argument. Dans leur rédaction précédente
	// elles VERROUILLAIENT le français — un test vert sur un défaut visible à l'écran.
	// ⚠️ « Généré » est devenu « Créé » (arbitrage de Guy, 2026-08-20) : le verbe « créer »
	// est désormais uniforme dans le domaine.
	it('mappe les statuts sur leur repli français', () => {
		expect(paymentBatchStatusLabel('generated')).toBe('Créé');
		expect(paymentBatchStatusLabel('confirmed')).toBe('Confirmé');
		expect(paymentBatchStatusLabel('cancelled')).toBe('Annulé');
	});

	// Borne anti-régression : le défaut d'origine était l'ABSENCE d'appel i18n. Une assertion
	// de valeur ne l'aurait jamais vu — celle-ci le voit, en empruntant le chemin réel.
	it('passe réellement par le dictionnaire i18n', async () => {
		getMock.mockResolvedValue({
			locale: 'de-CH',
			messages: {
				'payment-batches-status-generated': 'Erstellt',
				'payment-batches-status-confirmed': 'Bestätigt',
				'payment-batches-status-cancelled': 'Storniert',
			},
		});
		await loadI18nMessages();
		expect(paymentBatchStatusLabel('generated')).toBe('Erstellt');
		expect(paymentBatchStatusLabel('confirmed')).toBe('Bestätigt');
		expect(paymentBatchStatusLabel('cancelled')).toBe('Storniert');
	});
});

describe('failedItemLabel', () => {
	// ⚠️ L'ancienne rédaction assertait `toContain('coordonnées')` et `toContain('lot')` — plus
	// insidieux qu'un `toBe` : un `toContain` survit à une traduction PARTIELLE, donc il serait
	// resté vert sur un correctif à moitié fait. Les six codes sont désormais assertés en
	// entier, et le chemin réel est éprouvé par le cas suivant.
	it('mappe les six codes sur leur repli français', () => {
		expect(failedItemLabel('SUPPLIER_INVOICE_NOT_FOUND')).toBe('Facture introuvable');
		expect(failedItemLabel('SUPPLIER_INVOICE_NOT_OPEN')).toBe('Facture non ouverte');
		expect(failedItemLabel('NO_PAYMENT_COORDINATES')).toBe(
			'Pas de coordonnées de paiement (IBAN/QR-IBAN)'
		);
		expect(failedItemLabel('ALREADY_IN_GENERATED_BATCH')).toBe('Déjà dans un lot créé');
		expect(failedItemLabel('INVALID_IBAN')).toBe('IBAN invalide');
		expect(failedItemLabel('INVALID_QR_IBAN')).toBe('QR-IBAN invalide');
	});

	it('un code inconnu retombe sur sa valeur brute', () => {
		expect(failedItemLabel('UNKNOWN')).toBe('UNKNOWN');
	});

	it('passe réellement par le dictionnaire i18n', async () => {
		getMock.mockResolvedValue({
			locale: 'de-CH',
			messages: {
				'payment-batches-failed-no-payment-coordinates': 'Keine Zahlungsverbindung (IBAN/QR-IBAN)',
				'payment-batches-failed-already-in-generated-batch': 'Bereits in einem erstellten Stapel',
			},
		});
		await loadI18nMessages();
		expect(failedItemLabel('NO_PAYMENT_COORDINATES')).toBe(
			'Keine Zahlungsverbindung (IBAN/QR-IBAN)'
		);
		expect(failedItemLabel('ALREADY_IN_GENERATED_BATCH')).toBe(
			'Bereits in einem erstellten Stapel'
		);
	});
});
