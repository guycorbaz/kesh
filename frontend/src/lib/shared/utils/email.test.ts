// Story 17-4d Pass 1 (PD2) — pré-validation email client, miroir approché du
// backend `is_valid_email_simple`.

import { describe, it, expect } from 'vitest';
import { isPlausibleEmail } from './email';

describe('isPlausibleEmail', () => {
	it('accepte les formats plausibles', () => {
		expect(isPlausibleEmail('a@b.ch')).toBe(true);
		expect(isPlausibleEmail('jean.dupont@example.com')).toBe(true);
		expect(isPlausibleEmail('admin+kesh@nas.local.lan')).toBe(true);
	});

	it('rejette les fautes courantes que le backend rejetterait aussi', () => {
		expect(isPlausibleEmail('@')).toBe(false); // local et domaine vides
		expect(isPlausibleEmail('a@')).toBe(false); // domaine vide
		expect(isPlausibleEmail('@b.ch')).toBe(false); // local vide
		expect(isPlausibleEmail('a@b')).toBe(false); // domaine sans point
		expect(isPlausibleEmail('a b@c.ch')).toBe(false); // espace dans le local
		expect(isPlausibleEmail('a@c .ch')).toBe(false); // espace dans le domaine
		expect(isPlausibleEmail('pas-un-email')).toBe(false); // pas de @
		expect(isPlausibleEmail('')).toBe(false);
	});
});
