/**
 * Preuves du CHEMIN RÉEL de rendu du compteur — passe 4 de revue (#301).
 *
 * ⚠️ **Ce fichier existe parce que « chaque maillon est prouvé » ne veut pas
 * dire « la chaîne est prouvée ».** Le `.ftl` avait sa preuve, le dictionnaire
 * la sienne, `i18nMsg` la sienne — et le TERNAIRE qui choisit entre les deux
 * clés selon `autres === 1`, c'est-à-dire le correctif de la passe 3 lui-même,
 * n'était exercé par rien. Le muter en `autres === 2` laissait **91 preuves
 * vertes** et réaffichait « et 1 autres », le défaut mot pour mot.
 *
 * ⚠️ **Il ne peut PAS vivre dans `contacts-page.test.ts`** : celui-ci mocke
 * `i18nMsg` et rend le repli sans regarder la clé. Une preuve écrite là-bas
 * garderait la bascule et rien du chemin réel — elle ne verrait pas une clé mal
 * orthographiée, ni un dictionnaire qui prime sur un repli correct.
 *
 * D'où : **aucun mock de `i18n.svelte`**, un dictionnaire relevé sur
 * `all_messages(&Locale::FrCh)`, et une lecture du DOM.
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, fireEvent } from '@testing-library/svelte';
import type { ContactResponse } from '$lib/features/contacts/contacts.types';

vi.mock('$app/environment', () => ({ browser: true }));
vi.mock('$app/navigation', () => ({ goto: vi.fn() }));
vi.mock('$app/state', () => ({ page: { url: new URL('http://localhost/contacts') } }));
vi.mock('$lib/shared/utils/notify', () => ({ notifyError: vi.fn(), notifySuccess: vi.fn() }));

const listContactsMock = vi.fn();
vi.mock('$lib/features/contacts/contacts.api', () => ({
	listContacts: (q?: unknown) => listContactsMock(q),
	createContact: vi.fn(),
	updateContact: vi.fn(),
	archiveContact: vi.fn()
}));

// Le dictionnaire RÉEL, relevé sur `all_messages(&Locale::FrCh)` — marques
// d'isolation bidirectionnelle U+2068/U+2069 comprises, que `i18nMsg` consomme.
const DICO: Record<string, string> = {
	'contact-duplicate-heading': 'Contacts déjà enregistrés qui pourraient correspondre',
	'contact-duplicate-others-count-one': 'et 1 autre',
	'contact-duplicate-others-count': 'et ⁨{$count}⁩ autres'
};
vi.mock('$lib/shared/utils/api-client', () => ({
	apiClient: { get: () => Promise.resolve({ locale: 'fr-CH', messages: DICO }) }
}));

import { loadI18nMessages } from '$lib/shared/utils/i18n.svelte';
import Page from './+page.svelte';

function c(id: number, name: string): ContactResponse {
	return {
		id, name, companyId: 1, contactType: 'Entreprise', firstName: null, lastName: null,
		isClient: true, isSupplier: false, address: null,
		addressStructured: { street: '', building: '', postalCode: '', city: '', country: 'CH' },
		email: null, phone: null, ideNumber: null, clientNumber: null,
		defaultPaymentTerms: null, defaultPaymentTermsDays: null, language: null,
		salutation: 'Neutre', active: true, version: 1, createdAt: '', updatedAt: ''
	} as unknown as ContactResponse;
}
const VIDE = { items: [], total: 0, limit: 20, offset: 0 };
const settle = (ms = 400) => new Promise((r) => setTimeout(r, ms));

function repondre(items: ContactResponse[], total: number) {
	listContactsMock.mockImplementation((q: Record<string, unknown>) => {
		if (typeof q?.search !== 'string' || q.search === '') return Promise.resolve(VIDE);
		return Promise.resolve({ ...VIDE, items, total });
	});
}

async function ouvrir() {
	const b = [...document.querySelectorAll('button')].find((x) => /Nouveau contact/i.test(x.textContent ?? ''));
	if (!b) throw new Error('bouton introuvable');
	await fireEvent.click(b);
	await settle(20);
}
function zone(t: string) {
	const el = document.querySelector<HTMLElement>(`[data-testid="${t}"]`);
	if (!el) throw new Error('zone introuvable');
	return el;
}

describe('CHEMIN RÉEL — le compteur « et N autres » avec le vrai dictionnaire', () => {
	beforeEach(async () => {
		document.body.innerHTML = '';
		await loadI18nMessages();
	});

	it('un seul restant ⇒ « et 1 autre » (SINGULIER)', async () => {
		const six = [1, 2, 3, 4, 5, 6].map((i) => c(i, `Garage Test ${i}`));
		repondre(six, 6);
		render(Page);
		await settle(20);
		await ouvrir();
		await fireEvent.input(document.querySelector('#form-name')!, { target: { value: 'Garage Test' } });
		await settle(500);
		const t = zone('contact-duplicate-nearby').textContent ?? '';
		expect(t).toContain('Contacts déjà enregistrés qui pourraient correspondre');
		expect(t).toContain('et 1 autre');
		expect(t).not.toContain('et 1 autres');
	});

	it('trois restants ⇒ « et 3 autres » (PLURIEL interpolé)', async () => {
		const huit = [1, 2, 3, 4, 5, 6, 7, 8].map((i) => c(i, `Garage Test ${i}`));
		repondre(huit, 8);
		render(Page);
		await settle(20);
		await ouvrir();
		await fireEvent.input(document.querySelector('#form-name')!, { target: { value: 'Garage Test' } });
		await settle(500);
		expect(zone('contact-duplicate-nearby').textContent ?? '').toContain('et 3 autres');
	});
});
