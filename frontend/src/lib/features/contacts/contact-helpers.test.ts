import { describe, expect, it } from 'vitest';
import {
	contactTypeI18nKey,
	formatIdeNumber,
	normalizeIdeForApi,
	validateIdeFormat
} from './contact-helpers';

describe('formatIdeNumber', () => {
	it('formate une forme normalisée standard', () => {
		expect(formatIdeNumber('CHE109322551')).toBe('CHE-109.322.551');
	});

	it('retourne une chaîne vide pour null/undefined/vide', () => {
		expect(formatIdeNumber(null)).toBe('');
		expect(formatIdeNumber(undefined)).toBe('');
		expect(formatIdeNumber('')).toBe('');
	});

	it('retourne la chaîne telle quelle si format invalide', () => {
		// Ne casse pas sur des inputs inattendus (défensif).
		expect(formatIdeNumber('not-an-ide')).toBe('not-an-ide');
		expect(formatIdeNumber('CHE12345')).toBe('CHE12345');
	});
});

describe('validateIdeFormat', () => {
	it('accepte la forme normalisée', () => {
		expect(validateIdeFormat('CHE109322551')).toBe(true);
	});

	it('accepte la forme avec séparateurs', () => {
		expect(validateIdeFormat('CHE-109.322.551')).toBe(true);
	});

	it('accepte les minuscules (normalisées en majuscules avant check)', () => {
		expect(validateIdeFormat('che-109.322.551')).toBe(true);
	});

	it('rejette format trop court', () => {
		expect(validateIdeFormat('CHE12345')).toBe(false);
	});

	it('rejette format sans CHE', () => {
		expect(validateIdeFormat('123456789')).toBe(false);
	});

	it('rejette chaîne vide', () => {
		expect(validateIdeFormat('')).toBe(false);
	});

	it('rejette chiffres invalides', () => {
		expect(validateIdeFormat('CHEABCDEFGHI')).toBe(false);
	});
});

describe('normalizeIdeForApi', () => {
	it('retire séparateurs et met en majuscules', () => {
		expect(normalizeIdeForApi('che-109.322.551')).toBe('CHE109322551');
	});

	it('retourne null pour input vide', () => {
		expect(normalizeIdeForApi('')).toBe(null);
		expect(normalizeIdeForApi(null)).toBe(null);
		expect(normalizeIdeForApi(undefined)).toBe(null);
	});

	it('préserve une forme déjà normalisée', () => {
		expect(normalizeIdeForApi('CHE109322551')).toBe('CHE109322551');
	});
});

describe('contactTypeI18nKey', () => {
	it('retourne la clé i18n pour chaque variant', () => {
		expect(contactTypeI18nKey('Personne')).toBe('contact-type-personne');
		expect(contactTypeI18nKey('Entreprise')).toBe('contact-type-entreprise');
	});
});
