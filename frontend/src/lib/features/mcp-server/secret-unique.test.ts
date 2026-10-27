/**
 * Story 32-7 (refs #723) — `SecretUnique` (AC-11 (ii)).
 *
 * Les champs sont privés : `JSON.stringify` et `Object.getOwnPropertyNames` ne les
 * voient pas ; `lire()` est donc l'unique preuve que `oublier()` efface (mutation M2).
 */
import { describe, it, expect } from 'vitest';
import { SecretUnique } from './secret-unique.svelte';

const SECRET = 'kesh_mcp_SECRET0123456789abcdefghijk';

describe('SecretUnique', () => {
	it('montrer puis lire', () => {
		const s = new SecretUnique();
		expect(s.lire()).toBeNull();
		s.montrer('agent', SECRET);
		expect(s.lire()).toEqual({ nom: 'agent', valeur: SECRET });
	});

	it('oublier efface tout', () => {
		const s = new SecretUnique();
		s.montrer('agent', SECRET);
		s.oublier();
		expect(s.lire()).toBeNull();
		expect(JSON.stringify(s)).not.toContain(SECRET);
		for (const p of Object.getOwnPropertyNames(s)) {
			expect(String((s as unknown as Record<string, unknown>)[p])).not.toContain(SECRET);
		}
	});

	it('montrer remplace le secret précédent', () => {
		const s = new SecretUnique();
		s.montrer('a', SECRET);
		s.montrer('b', 'kesh_mcp_AUTRE');
		expect(s.lire()).toEqual({ nom: 'b', valeur: 'kesh_mcp_AUTRE' });
	});
});
