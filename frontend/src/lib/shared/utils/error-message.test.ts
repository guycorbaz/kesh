/**
 * Story 28-2 (#520) — `errorMessageOf` : une valeur attrapée rendue en texte.
 *
 * Le client HTTP est simulé **sans** `isApiError` (comme le font 18 fichiers de
 * test du dépôt) : l'utilitaire ne doit pas en dépendre — il lit le garde-type
 * dans `api-error.ts` (C-28-2-2). Le catalogue est chargé par le vrai
 * `loadI18nMessages`, sur ce client simulé.
 */
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';

const { getMock } = vi.hoisted(() => ({ getMock: vi.fn() }));

vi.mock('$lib/shared/utils/api-client', () => ({
	apiClient: { get: getMock },
}));

import { errorMessageOf } from './error-message';
import { loadI18nMessages } from './i18n.svelte';

const GENERIQUE_FR = 'Erreur inattendue.';

async function chargerCatalogue(messages: Record<string, string>): Promise<void> {
	getMock.mockResolvedValueOnce({ locale: 'de-CH', messages });
	await loadI18nMessages();
}

describe('errorMessageOf (Story 28-2, AC1/AC8)', () => {
	let consoleError: ReturnType<typeof vi.spyOn>;

	beforeEach(() => {
		consoleError = vi.spyOn(console, 'error').mockImplementation(() => {});
	});

	afterEach(async () => {
		consoleError.mockRestore();
		// `_messages` est un état de module : catalogue vide entre deux cas.
		await chargerCatalogue({});
	});

	it('ApiError → son message, tel quel, sans journalisation', () => {
		const err = { code: 'VALIDATION_ERROR', message: 'La date doit suivre le verrou.', status: 400 };
		expect(errorMessageOf(err)).toBe('La date doit suivre le verrou.');
		expect(consoleError).not.toHaveBeenCalled();
	});

	it('ApiError à message entouré de blancs → le message tel quel, blancs compris (revue P1 E-L1)', () => {
		expect(errorMessageOf({ code: 'X', message: ' Refus. ', status: 400 })).toBe(' Refus. ');
	});

	it.each([
		['message vide', ''],
		['message blanc', '  '],
		['message undefined', undefined],
		['message numérique', 42],
		['message objet', { a: 1 }],
	])('ApiError au %s → le générique, sans journalisation (revue P1 E-L2)', (_nom, message) => {
		const err = { code: 'X', message, status: 500 };
		const rendu = errorMessageOf(err);
		expect(rendu).toBe(GENERIQUE_FR);
		expect(rendu).not.toBe('[object Object]');
		expect(rendu).not.toBe('');
		expect(consoleError).not.toHaveBeenCalled();
	});

	it.each([
		['Error', new Error('boom')],
		['TypeError', new TypeError('Failed to fetch')],
		['DOMException', new DOMException('aborted', 'AbortError')],
		['chaîne', 'texte'],
		['nombre', 42],
		['null', null],
		['undefined', undefined],
		['objet vide', {}],
		['objet à message sans code ni status', { message: 'm' }],
	])('%s → le générique, journalisé avec la valeur d’origine', (_nom, valeur) => {
		const rendu = errorMessageOf(valeur);
		expect(rendu).toBe(GENERIQUE_FR);
		expect(rendu).not.toBe('[object Object]');
		expect(rendu).not.toBe('');
		expect(rendu).not.toBe('undefined');
		expect(rendu).not.toBe('null');
		expect(consoleError).toHaveBeenCalledWith('[errorMessageOf]', valeur);
	});

	it('Error : le message technique n’est pas affiché', () => {
		expect(errorMessageOf(new Error('No candidate for txId=5'))).not.toContain('txId');
	});

	it('traduction : le générique suit le catalogue chargé', async () => {
		await chargerCatalogue({ 'error-unexpected': 'Unerwarteter Fehler.' });
		expect(errorMessageOf(new Error('boom'))).toBe('Unerwarteter Fehler.');
		expect(errorMessageOf({ code: 'X', message: '', status: 500 })).toBe('Unerwarteter Fehler.');
	});
});

describe('errorMessageOf avec un repli (Story v017-2, #809, AC-1)', () => {
	const REPLI = 'Erreur lors de la sauvegarde des coordonnées';
	let consoleError: ReturnType<typeof vi.spyOn>;

	beforeEach(() => {
		consoleError = vi.spyOn(console, 'error').mockImplementation(() => {});
	});

	afterEach(() => {
		consoleError.mockRestore();
	});

	it('ApiError à message → ce message, et non le repli, sans journalisation', () => {
		const err = { code: 'VALIDATION_ERROR', message: 'adresse : le pays…', status: 400 };
		expect(errorMessageOf(err, REPLI)).toBe('adresse : le pays…');
		expect(consoleError).not.toHaveBeenCalled();
	});

	it.each([
		['message vide', ''],
		['message blanc', '  '],
		['message non-chaîne', 42],
	])('ApiError au %s → le repli, sans journalisation', (_nom, message) => {
		expect(errorMessageOf({ code: 'X', message, status: 400 }, REPLI)).toBe(REPLI);
		expect(consoleError).not.toHaveBeenCalled();
	});

	it('TypeError → le repli, et la valeur journalisée', () => {
		const panne = new TypeError('Failed to fetch');
		expect(errorMessageOf(panne, REPLI)).toBe(REPLI);
		expect(consoleError).toHaveBeenCalledWith('[errorMessageOf]', panne);
	});

	it.each([
		['vide', ''],
		['blanc', '   '],
	])('repli %s → traité comme absent : le générique', (_nom, repli) => {
		expect(errorMessageOf(new TypeError('x'), repli)).toBe(GENERIQUE_FR);
		expect(errorMessageOf({ code: 'X', message: '', status: 400 }, repli)).toBe(GENERIQUE_FR);
	});

	it('sans repli → inchangé (le générique)', () => {
		expect(errorMessageOf(new TypeError('x'))).toBe(GENERIQUE_FR);
		expect(errorMessageOf(new TypeError('x'), undefined)).toBe(GENERIQUE_FR);
	});
});
