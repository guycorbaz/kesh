// Story v0174-correctifs-3 (#889, AC-3.8, l) — la date d'import d'un document, horodatage
// UTC, s'affiche à sa date suisse : importé à 23 h 30 UTC le 4 novembre, il l'est le 5.
import { describe, expect, it } from 'vitest';
import { importDateOf, type ImportedSupplierInvoice } from './imported-supplier-invoices.types';
import { pinTimeZone } from '$lib/shared/utils/pin-time-zone.test-helper';

describe('importDateOf — à l’heure suisse (#889)', () => {
	pinTimeZone('Europe/Zurich');
	it('`2026-11-04T23:30:00` ⇒ `05.11.2026`', () => {
		const row = { createdAt: '2026-11-04T23:30:00' } as ImportedSupplierInvoice;
		expect(importDateOf(row)).toBe('05.11.2026');
	});
	it('`2026-11-04T08:00:00` ⇒ `04.11.2026`', () => {
		const row = { createdAt: '2026-11-04T08:00:00' } as ImportedSupplierInvoice;
		expect(importDateOf(row)).toBe('04.11.2026');
	});
});
