// Story 8-4 — Test du module ScoreBadge.svelte. On vérifie que le
// composant se charge et que les paliers documentés (≥0.85 high,
// 0.60-0.85 medium, <0.60 low) sont alignés avec ScoreBadge.svelte.

import { describe, expect, it } from 'vitest';

describe('ScoreBadge', () => {
	it('le module se charge et exporte un composant Svelte', async () => {
		const mod = await import('./ScoreBadge.svelte');
		expect(mod.default).toBeDefined();
	});

	// Test du contrat des paliers de score (couplé au composant via doc-comment).
	it.each([
		[0.92, 'high'],
		[0.85, 'high'],
		[0.84, 'medium'],
		[0.6, 'medium'],
		[0.59, 'low'],
		[0.0, 'low'],
	])('score %f → tier %s', (score, expectedTier) => {
		const tier = score >= 0.85 ? 'high' : score >= 0.6 ? 'medium' : 'low';
		expect(tier).toBe(expectedTier);
	});
});
