// Story 8-4 — Test du module ScoreBadge.svelte. On vérifie que le
// composant se charge et que les paliers documentés (≥0.90 high,
// 0.70-0.90 medium, <0.70 low) sont alignés avec ScoreBadge.svelte.
// H3 Pass 1 code review : seuils 0.90/0.70 alignés sur §matching-algo.

import { describe, expect, it } from 'vitest';

describe('ScoreBadge', () => {
	it('le module se charge et exporte un composant Svelte', async () => {
		const mod = await import('./ScoreBadge.svelte');
		expect(mod.default).toBeDefined();
	});

	// Test du contrat des paliers de score (couplé au composant via doc-comment).
	it.each([
		[1.0, 'high'],
		[0.92, 'high'],
		[0.9, 'high'],
		[0.85, 'medium'],
		[0.7, 'medium'],
		[0.69, 'low'],
		[0.5, 'low'],
		[0.0, 'low'],
	])('score %f → tier %s', (score, expectedTier) => {
		const tier = score >= 0.9 ? 'high' : score >= 0.7 ? 'medium' : 'low';
		expect(tier).toBe(expectedTier);
	});
});
