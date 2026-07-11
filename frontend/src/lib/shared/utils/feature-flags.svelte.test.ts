// Story 20-3b2 — setters des feature flags (no-op si valeur non-booléenne).
import { describe, expect, it } from 'vitest';
import { featureFlags } from './feature-flags.svelte';

describe('featureFlags.smtpConfigured (20-3b2)', () => {
	it('défaut false (anti-faux-affordance : bouton grisé tant que /health silencieux)', () => {
		expect(featureFlags.smtpConfigured).toBe(false);
	});

	it('setter accepte un booléen et ignore le reste (payload /health non fiable)', () => {
		featureFlags.setSmtpConfigured('true');
		expect(featureFlags.smtpConfigured).toBe(false);
		featureFlags.setSmtpConfigured(undefined);
		expect(featureFlags.smtpConfigured).toBe(false);
		featureFlags.setSmtpConfigured(true);
		expect(featureFlags.smtpConfigured).toBe(true);
		featureFlags.setSmtpConfigured(1);
		expect(featureFlags.smtpConfigured).toBe(true);
		featureFlags.setSmtpConfigured(false);
		expect(featureFlags.smtpConfigured).toBe(false);
	});
});
